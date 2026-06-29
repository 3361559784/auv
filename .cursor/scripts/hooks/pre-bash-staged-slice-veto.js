'use strict';

const {
  analyzeStagedSlice,
  evaluateGitCommitHook,
  hookResult,
} = require('../lib/git-commit-gate');

function evaluate(rawInput) {
  return evaluateGitCommitHook(rawInput, ({ stagedFiles, subject, rawInput: input }) => {
    const lines = [];
    const violations = analyzeStagedSlice({ stagedFiles, subject });

    if (violations.length === 0) {
      return hookResult(input, { exitCode: 0, lines });
    }

    lines.push('[AUV gate] BLOCKED: staged slice veto triggered.');
    for (const violation of violations) {
      lines.push(`- ${violation.code}: ${violation.message}`);
      if (violation.detail) {
        lines.push(`  ${violation.detail}`);
      }
    }
    lines.push('Shrink the slice per CONTRIBUTING.local.md before committing.');

    return hookResult(input, { exitCode: 2, lines });
  });
}

function run(rawInput) {
  const result = evaluate(rawInput);
  return {
    stdout: result.output,
    stderr: result.stderr,
    exitCode: result.exitCode,
  };
}

if (require.main === module) {
  let data = '';
  process.stdin.setEncoding('utf8');
  process.stdin.on('data', chunk => {
    if (data.length < 1024 * 1024) {
      data += chunk.substring(0, 1024 * 1024 - data.length);
    }
  });
  process.stdin.on('end', () => {
    const result = evaluate(data);
    if (result.stderr) {
      process.stderr.write(`${result.stderr}\n`);
    }
    process.stdout.write(result.output);
    process.exit(result.exitCode);
  });
}

module.exports = { run, evaluate, analyzeStagedSlice };

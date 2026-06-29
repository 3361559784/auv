'use strict';

const {
  evaluateGitCommitHook,
  hookResult,
  isRustPath,
  runCargo,
  runGit,
} = require('../lib/git-commit-gate');

function evaluate(rawInput) {
  return evaluateGitCommitHook(rawInput, ({ stagedFiles, rawInput: input }) => {
    const lines = [];
    const rustTouched = stagedFiles.some(isRustPath);
    if (!rustTouched) {
      return hookResult(input, { exitCode: 0, lines });
    }

    lines.push('[AUV gate] Rust/Cargo paths staged; running validation floor.');

    const whitespace = runGit(['diff', '--cached', '--check']);
    if (whitespace.status !== 0) {
      lines.push('[AUV gate] BLOCKED: git diff --cached --check failed.');
      if (whitespace.stderr) {
        lines.push(whitespace.stderr.trim());
      }
      return hookResult(input, { exitCode: 2, lines });
    }

    const fmt = runCargo(['fmt', '--', '--check']);
    if (fmt.status !== 0) {
      lines.push('[AUV gate] BLOCKED: cargo fmt --check failed.');
      const fmtOut = `${fmt.stdout || ''}${fmt.stderr || ''}`.trim();
      if (fmtOut) {
        lines.push(fmtOut.slice(-4000));
      }
      return hookResult(input, { exitCode: 2, lines });
    }

    const check = runCargo(['check']);
    if (check.status !== 0) {
      lines.push('[AUV gate] BLOCKED: cargo check failed.');
      const checkOut = `${check.stdout || ''}${check.stderr || ''}`.trim();
      if (checkOut) {
        lines.push(checkOut.slice(-4000));
      }
      return hookResult(input, { exitCode: 2, lines });
    }

    const test = runCargo(['test'], { timeout: 900_000 });
    if (test.status !== 0) {
      lines.push('[AUV gate] BLOCKED: cargo test failed.');
      const testOut = `${test.stdout || ''}${test.stderr || ''}`.trim();
      if (testOut) {
        lines.push(testOut.slice(-6000));
      }
      return hookResult(input, { exitCode: 2, lines });
    }

    lines.push('[AUV gate] PASS: git diff --cached --check, cargo fmt --check, cargo check, cargo test.');
    return hookResult(input, { exitCode: 0, lines });
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

module.exports = { run, evaluate };

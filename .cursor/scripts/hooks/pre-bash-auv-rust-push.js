'use strict';

const {
  evaluateGitPushHook,
  hookResult,
  isRustPath,
  runCargo,
} = require('../lib/git-commit-gate');

function evaluate(rawInput) {
  return evaluateGitPushHook(rawInput, ({ outgoingFiles, rawInput: input }) => {
    const lines = [];
    const rustTouched = outgoingFiles.some(isRustPath);
    if (!rustTouched) {
      return hookResult(input, { exitCode: 0, lines });
    }

    lines.push('[AUV gate] Outgoing push touches Rust/Cargo; running validation floor.');

    const fmt = runCargo(['fmt', '--', '--check']);
    if (fmt.status !== 0) {
      lines.push('[AUV gate] BLOCKED: cargo fmt --check failed before push.');
      const fmtOut = `${fmt.stdout || ''}${fmt.stderr || ''}`.trim();
      if (fmtOut) {
        lines.push(fmtOut.slice(-4000));
      }
      return hookResult(input, { exitCode: 2, lines });
    }

    const check = runCargo(['check']);
    if (check.status !== 0) {
      lines.push('[AUV gate] BLOCKED: cargo check failed before push.');
      const checkOut = `${check.stdout || ''}${check.stderr || ''}`.trim();
      if (checkOut) {
        lines.push(checkOut.slice(-4000));
      }
      return hookResult(input, { exitCode: 2, lines });
    }

    const test = runCargo(['test'], { timeout: 900_000 });
    if (test.status !== 0) {
      lines.push('[AUV gate] BLOCKED: cargo test failed before push.');
      const testOut = `${test.stdout || ''}${test.stderr || ''}`.trim();
      if (testOut) {
        lines.push(testOut.slice(-6000));
      }
      return hookResult(input, { exitCode: 2, lines });
    }

    lines.push('[AUV gate] PASS: cargo fmt --check, cargo check, cargo test.');
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

#!/usr/bin/env node
'use strict';

const { readStdin } = require('./adapter');
const { run: runSliceVeto } = require('../scripts/hooks/pre-bash-staged-slice-veto');
const { run: runRustCommit } = require('../scripts/hooks/pre-bash-auv-rust-commit');
const { run: runCommitQuality } = require('../scripts/hooks/pre-bash-commit-quality');

const COMMIT_GATES = [runSliceVeto, runRustCommit, runCommitQuality];

function extractShellCommand(raw) {
  try {
    const parsed = JSON.parse(raw || '{}');
    return String(parsed.command || parsed.args?.command || '');
  } catch {
    return String(raw || '');
  }
}

readStdin()
  .then(raw => {
    const command = extractShellCommand(raw);
    if (!/\bgit\s+commit\b/.test(command)) {
      process.stdout.write(raw);
      return;
    }

    const claudeInput = JSON.stringify({ tool_input: { command } });
    for (const runGate of COMMIT_GATES) {
      const result = runGate(claudeInput);
      if (result.exitCode === 2) {
        if (result.stderr) {
          process.stderr.write(result.stderr.endsWith('\n') ? result.stderr : `${result.stderr}\n`);
        }
        process.exit(2);
      }
    }

    process.stdout.write(raw);
  })
  .catch(() => {
    process.exit(0);
  });

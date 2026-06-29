#!/usr/bin/env node
'use strict';

const { readStdin } = require('./adapter');
const { run: runRustPush } = require('../scripts/hooks/pre-bash-auv-rust-push');

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
    if (!/\bgit\s+push\b/.test(command)) {
      process.stdout.write(raw);
      return;
    }

    const claudeInput = JSON.stringify({ tool_input: { command } });
    const result = runRustPush(claudeInput);
    if (result.exitCode === 2) {
      if (result.stderr) {
        process.stderr.write(result.stderr.endsWith('\n') ? result.stderr : `${result.stderr}\n`);
      }
      process.exit(2);
    }

    process.stdout.write(raw);
  })
  .catch(() => {
    process.exit(0);
  });

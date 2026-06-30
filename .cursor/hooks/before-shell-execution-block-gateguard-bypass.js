#!/usr/bin/env node
'use strict';

const { readStdin, hookEnabled } = require('./adapter');
const { run } = require('../scripts/hooks/block-gateguard-bypass');

readStdin()
  .then(raw => {
    if (!hookEnabled('pre:bash:block-gateguard-bypass', ['standard', 'strict'])) {
      process.stdout.write(raw);
      return;
    }

    let command = '';
    try {
      const parsed = JSON.parse(raw || '{}');
      command = String(parsed.command || parsed.args?.command || '');
    } catch {
      command = String(raw || '');
    }

    const result = run(JSON.stringify({ tool_input: { command } }));
    if (result.exitCode === 2) {
      if (result.stderr) {
        process.stderr.write(`${result.stderr}\n`);
      }
      process.exit(2);
    }

    process.stdout.write(raw);
  })
  .catch(() => process.exit(0));

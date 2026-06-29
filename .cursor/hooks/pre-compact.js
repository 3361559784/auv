#!/usr/bin/env node
const { readStdin, runExistingHook, transformToClaude, hookEnabled } = require('./adapter');
const { buildProjectContext, workspaceRootsFromInput } = require('../scripts/lib/read-project-context');

readStdin()
  .then(raw => {
    let input = {};
    try {
      input = JSON.parse(raw || '{}');
    } catch {
      input = {};
    }

    const claudeInput = transformToClaude(input);
    runExistingHook('pre-compact.js', claudeInput);

    if (!hookEnabled('pre:compact:inject-project-context', ['minimal', 'standard', 'strict'])) {
      process.stdout.write(raw);
      return;
    }

    const additionalContext = buildProjectContext({
      extraStarts: workspaceRootsFromInput(input),
    });

    if (!additionalContext) {
      process.stdout.write(raw);
      return;
    }

    const payload = {
      additional_context: additionalContext,
      user_message: 'Context compacted — project gate docs re-injected for the next turn.',
    };
    process.stdout.write(`${JSON.stringify(payload)}\n`);
  })
  .catch(() => process.exit(0));

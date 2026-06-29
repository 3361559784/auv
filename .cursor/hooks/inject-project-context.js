#!/usr/bin/env node
/**
 * Inject project context documents each turn and after compaction.
 *
 * Injected: CONTRIBUTING.local.md, cursor.md, AGENTS.md
 * Never injected: codex.md
 */

'use strict';

const { readStdin, hookEnabled } = require('./adapter');
const { buildProjectContext, workspaceRootsFromInput } = require('../scripts/lib/read-project-context');

const MAX_STDIN = 1024 * 1024;

function hookEventName(input) {
  return String(
    input.hook_event_name || input.hookEventName || input._cursor?.hook_event_name || ''
  ).trim();
}

function hookIdForEvent(eventName) {
  if (eventName === 'preCompact') {
    return 'pre:compact:inject-project-context';
  }
  return 'pre:prompt:inject-project-context';
}

readStdin()
  .then(raw => {
    let input = {};
    try {
      input = JSON.parse(raw || '{}');
    } catch {
      input = {};
    }

    const eventName = hookEventName(input);
    const hookId = hookIdForEvent(eventName);

    if (!hookEnabled(hookId, ['minimal', 'standard', 'strict'])) {
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

    const payload = { additional_context: additionalContext };

    if (eventName === 'beforeSubmitPrompt') {
      payload.continue = true;
    }

    if (eventName === 'preCompact') {
      payload.user_message = 'Context compacted — project gate docs re-injected for the next turn.';
    }

    process.stdout.write(`${JSON.stringify(payload)}\n`);
  })
  .catch(() => process.exit(0));

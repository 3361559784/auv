#!/usr/bin/env node
/**
 * beforeSubmitPrompt: inject cursor.md into each user turn.
 *
 * Cursor's documented beforeSubmitPrompt output is { continue, user_message }.
 * We also emit additional_context (supported on some builds; harmless when ignored).
 * Reliable fallback: .cursor/rules/cursor-project.mdc (alwaysApply + @cursor.md).
 */

'use strict';

const { readStdin, hookEnabled } = require('./adapter');
const { readCursorMd, formatInjectedContext } = require('../scripts/lib/read-cursor-md');

const MAX_STDIN = 1024 * 1024;

function workspaceRootsFromInput(input) {
  const roots = [];
  const push = value => {
    if (value === undefined || value === null) return;
    if (Array.isArray(value)) {
      for (const item of value) push(item);
      return;
    }
    const raw = String(value).trim();
    if (raw) roots.push(raw);
  };

  push(input.workspace_roots);
  push(input.workspaceRoots);
  push(input._cursor?.workspace_roots);
  push(process.env.CURSOR_PROJECT_DIR);
  push(process.cwd());

  return [...new Set(roots)];
}

readStdin()
  .then(raw => {
    let input = {};
    try {
      input = JSON.parse(raw || '{}');
    } catch {
      input = {};
    }

    if (!hookEnabled('pre:prompt:inject-cursor-md', ['minimal', 'standard', 'strict'])) {
      process.stdout.write(raw);
      return;
    }

    const { path: cursorPath, content } = readCursorMd({
      extraStarts: workspaceRootsFromInput(input),
    });
    const additionalContext = formatInjectedContext(content, cursorPath);

    if (!additionalContext) {
      process.stdout.write(raw);
      return;
    }

    const payload = {
      continue: true,
      additional_context: additionalContext,
    };
    process.stdout.write(`${JSON.stringify(payload)}\n`);
  })
  .catch(() => process.exit(0));

#!/usr/bin/env node
const {
  readStdin,
  runExistingHook,
  transformToClaude,
  hookEnabled,
  getPluginRoot,
} = require('./adapter');
const {
  AGENT_DATA_HOME_ENV,
  getCursorSessionEnvPayload,
} = require('../scripts/lib/agent-data-home');
const { buildProjectContext, workspaceRootsFromInput } = require('../scripts/lib/read-project-context');

readStdin()
  .then(raw => {
    const input = JSON.parse(raw || '{}');
    const pluginRoot = getPluginRoot();
    const envPayload = {
      CLAUDE_PLUGIN_ROOT: pluginRoot,
      ECC_PLUGIN_ROOT: pluginRoot,
      ...getCursorSessionEnvPayload({ preferCursorDefault: true }),
    };

    for (const [key, value] of Object.entries(envPayload)) {
      if (value !== undefined && value !== null && String(value).length > 0) {
        process.env[key] = String(value);
      }
    }

    const claudeInput = transformToClaude(input);
    if (hookEnabled('session:start', ['minimal', 'standard', 'strict'])) {
      runExistingHook('session-start.js', claudeInput);
    }

    const projectContext = buildProjectContext({
      extraStarts: workspaceRootsFromInput(input),
    });
    const payload = {
      env: envPayload,
      additional_context: [
        projectContext,
        'ECC Cursor runtime initialized for this session.',
        `CLAUDE_PLUGIN_ROOT=${pluginRoot}`,
        `${AGENT_DATA_HOME_ENV}=${envPayload[AGENT_DATA_HOME_ENV]}`,
      ].filter(Boolean).join('\n\n'),
    };
    process.stdout.write(`${JSON.stringify(payload)}\n`);
  })
  .catch(() => process.exit(0));

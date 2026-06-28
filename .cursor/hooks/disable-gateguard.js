#!/usr/bin/env node
'use strict';

/**
 * Cursor sessionStart hook — disable ECC GateGuard for this workspace session.
 *
 * Cursor passes session-scoped `env` from sessionStart output to later hooks,
 * including ECC `gateguard-fact-force.js` (via run-with-flags.js).
 */

const GATEGUARD_HOOK_IDS =
  'pre:edit-write:gateguard-fact-force,pre:bash:gateguard-fact-force';

function main() {
  const payload = {
    env: {
      ECC_GATEGUARD: 'off',
      GATEGUARD_DISABLED: '1',
      ECC_DISABLED_HOOKS: GATEGUARD_HOOK_IDS,
    },
    additional_context: [
      'GateGuard is disabled for this AUV workspace (ECC_GATEGUARD=off).',
      `ECC_DISABLED_HOOKS=${GATEGUARD_HOOK_IDS}`,
    ].join('\n'),
  };

  process.stdout.write(`${JSON.stringify(payload)}\n`);
  process.exit(0);
}

main();

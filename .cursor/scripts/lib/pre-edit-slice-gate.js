'use strict';

/**
 * AUV pre-edit slice gate — shared text for GateGuard and project docs.
 * See CONTRIBUTING.local.md and cursor.md § Pre-edit hard stop.
 */

const SLICE_TYPES =
  'bug fix | test-only | docs-only | narrow refactor | owner-approved feature';

function sliceGateLines({ verb, filePath }) {
  const target = filePath || '<path>';
  return [
    '[AUV Pre-Edit Slice Gate]',
    '',
    `Before ${verb} ${target}, present ALL sections below in chat, then retry the same tool call.`,
    '',
    '## Slice gate (required — pick exactly one classification)',
    `- Classification: (${SLICE_TYPES})`,
    '- Veto: any CONTRIBUTING.local.md implementation veto = yes? If yes, shrink the slice first — do not edit.',
    '- Non-goals: what this diff explicitly does NOT do',
    '- Callers: who imports/uses this; confirm no duplicate purpose (Grep/Glob)',
    '- Regression: behavior change → which test would fail; docs-only → "n/a"',
    '- Validation: minimal commands for this single diff',
    '',
    '## File facts (required)',
  ];
}

function buildEditGateMessage(filePath) {
  const lines = sliceGateLines({ verb: 'editing', filePath });
  lines.push(
    '1. List ALL files that import/require this file (Grep)',
    '2. List the public functions/classes affected by this change',
    '3. If this file reads/writes data files, show field names, structure, and date format (synthetic/redacted only)',
    "4. Quote the user's current instruction verbatim",
    '',
    'GateGuard bypass (ECC_GATEGUARD=off, etc.) is not available in this workspace.',
    'Present everything, then retry the same operation.'
  );
  return lines.join('\n');
}

function buildWriteGateMessage(filePath) {
  const lines = sliceGateLines({ verb: 'creating', filePath });
  lines.push(
    '1. Name the file(s) and line(s) that will call this new file',
    '2. Confirm no existing file serves the same purpose (Glob/Grep)',
    '3. If this file reads/writes data files, show field names, structure, and date format (synthetic/redacted only)',
    "4. Quote the user's current instruction verbatim",
    '',
    'GateGuard bypass (ECC_GATEGUARD=off, etc.) is not available in this workspace.',
    'Present everything, then retry the same operation.'
  );
  return lines.join('\n');
}

function buildCondensedGateMessage(action, filePath, ordinal) {
  const safe = filePath || '<path>';
  return (
    `[AUV Pre-Edit Slice Gate] (denial #${ordinal} this session) ` +
    `Complete Classification/Veto/Non-goals/Callers/Regression/Validation for ${action} ${safe}, then retry.`
  );
}

module.exports = {
  SLICE_TYPES,
  buildEditGateMessage,
  buildWriteGateMessage,
  buildCondensedGateMessage,
};

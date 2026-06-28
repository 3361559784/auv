#!/usr/bin/env node
/**
 * Project-local GateGuard opt-out.
 *
 * Cursor sessionStart env (ECC_GATEGUARD=off) does not always reach ECC
 * PreToolUse hooks loaded from the global plugin cache. Workspaces that ship
 * `.cursor/hooks/disable-gateguard.js` disable GateGuard by filesystem marker
 * instead, walking up from cwd / workspace env / optional edit targets.
 */

'use strict';

const fs = require('fs');
const path = require('path');

const DISABLE_MARKER_SEGMENTS = ['.cursor', 'hooks', 'disable-gateguard.js'];

function disableMarkerPath(projectRoot) {
  return path.join(projectRoot, ...DISABLE_MARKER_SEGMENTS);
}

function hasDisableMarkerAt(projectRoot) {
  const marker = disableMarkerPath(projectRoot);
  try {
    return fs.existsSync(marker) && fs.statSync(marker).isFile();
  } catch (_) {
    return false;
  }
}

function collectSearchRoots(extraStarts = []) {
  const roots = [];
  const push = value => {
    const raw = String(value || '').trim();
    if (!raw) {
      return;
    }
    roots.push(raw);
  };

  push(process.cwd());
  for (const key of ['CLAUDE_PROJECT_DIR', 'CURSOR_WORKSPACE', 'WORKSPACE_FOLDER']) {
    push(process.env[key]);
  }

  for (const value of extraStarts) {
    push(value);
  }

  return roots;
}

function isProjectGateGuardDisabled(extraStarts = []) {
  const seen = new Set();

  for (const start of collectSearchRoots(extraStarts)) {
    let dir = path.resolve(start);
    if (seen.has(dir)) {
      continue;
    }
    seen.add(dir);

    try {
      if (fs.existsSync(dir) && fs.statSync(dir).isFile()) {
        dir = path.dirname(dir);
      }
    } catch (_) {
      /* ignore */
    }

    while (dir && dir !== path.dirname(dir)) {
      if (hasDisableMarkerAt(dir)) {
        return true;
      }
      dir = path.dirname(dir);
    }
  }

  return false;
}

module.exports = {
  DISABLE_MARKER_SEGMENTS,
  disableMarkerPath,
  hasDisableMarkerAt,
  isProjectGateGuardDisabled,
};

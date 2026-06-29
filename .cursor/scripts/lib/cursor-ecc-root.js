#!/usr/bin/env node
/**
 * Resolve the ECC plugin root for Cursor hook subprocesses.
 *
 * AUV vendors ECC under `<workspace>/.cursor/scripts/`. Upstream ECC monorepo
 * keeps `scripts/` at the repository root and `.cursor/hooks/adapter.js` walks
 * up two levels. This helper unifies both layouts for Cursor execution.
 */

'use strict';

const fs = require('fs');
const path = require('path');

const PROBE_RELATIVE = path.join('scripts', 'lib', 'utils.js');
const VENDORED_HOOKS_RELATIVE = path.join('scripts', 'hooks');

function hasEccProbe(rootDir) {
  if (!rootDir) return false;
  try {
    return fs.existsSync(path.join(rootDir, PROBE_RELATIVE));
  } catch {
    return false;
  }
}

function hasVendoredCursorScripts(cursorRoot) {
  if (!cursorRoot) return false;
  try {
    return fs.existsSync(path.join(cursorRoot, VENDORED_HOOKS_RELATIVE));
  } catch {
    return false;
  }
}

function collectWorkspaceRoots(extraStarts = []) {
  const roots = [];
  const push = value => {
    const raw = String(value || '').trim();
    if (!raw) return;
    roots.push(path.resolve(raw));
  };

  push(process.cwd());
  for (const key of [
    'CURSOR_PROJECT_DIR',
    'CLAUDE_PROJECT_DIR',
    'WORKSPACE_FOLDER',
    'CURSOR_WORKSPACE',
  ]) {
    push(process.env[key]);
  }

  for (const value of extraStarts) {
    push(value);
  }

  return [...new Set(roots)];
}

function findVendoredCursorRoot(extraStarts = []) {
  for (const workspaceRoot of collectWorkspaceRoots(extraStarts)) {
    let dir = workspaceRoot;
    try {
      if (fs.existsSync(dir) && fs.statSync(dir).isFile()) {
        dir = path.dirname(dir);
      }
    } catch {
      continue;
    }

    while (dir && dir !== path.dirname(dir)) {
      const cursorRoot = path.join(dir, '.cursor');
      if (hasVendoredCursorScripts(cursorRoot)) {
        return cursorRoot;
      }
      dir = path.dirname(dir);
    }
  }

  return null;
}

function resolveCursorEccPluginRoot(options = {}) {
  for (const key of ['CLAUDE_PLUGIN_ROOT', 'ECC_PLUGIN_ROOT']) {
    const candidate = String(options[key] || process.env[key] || '').trim();
    if (candidate && hasEccProbe(candidate)) {
      return path.resolve(candidate);
    }
  }

  const vendored = findVendoredCursorRoot(options.extraStarts || []);
  if (vendored) {
    return vendored;
  }

  const hostRoot = path.resolve(options.hostRoot || process.cwd());
  if (hasEccProbe(hostRoot)) {
    return hostRoot;
  }

  const adjacentCursorRoot = path.resolve(__dirname, '..', '..');
  if (hasVendoredCursorScripts(adjacentCursorRoot)) {
    return adjacentCursorRoot;
  }

  return adjacentCursorRoot;
}

module.exports = {
  PROBE_RELATIVE,
  VENDORED_HOOKS_RELATIVE,
  hasEccProbe,
  hasVendoredCursorScripts,
  findVendoredCursorRoot,
  resolveCursorEccPluginRoot,
};

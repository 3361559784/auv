'use strict';

const fs = require('fs');
const path = require('path');

const DEFAULT_MAX_CHARS = 12_000;
const MARKER = '\n\n[cursor.md truncated — raise CURSOR_MD_MAX_CHARS or shorten cursor.md]';

function isDisabled() {
  const raw = String(process.env.ECC_CURSOR_MD || process.env.CURSOR_MD_INJECT || 'on')
    .trim()
    .toLowerCase();
  return raw === 'off' || raw === '0' || raw === 'false';
}

function getMaxChars() {
  const parsed = Number.parseInt(String(process.env.CURSOR_MD_MAX_CHARS || ''), 10);
  return Number.isFinite(parsed) && parsed > 0 ? parsed : DEFAULT_MAX_CHARS;
}

function collectSearchRoots(extraStarts = []) {
  const roots = [];
  const push = value => {
    const raw = String(value || '').trim();
    if (!raw) return;
    roots.push(path.resolve(raw));
  };

  push(process.cwd());
  for (const key of ['CURSOR_PROJECT_DIR', 'CLAUDE_PROJECT_DIR', 'WORKSPACE_FOLDER', 'CURSOR_WORKSPACE']) {
    push(process.env[key]);
  }
  for (const value of extraStarts) {
    push(value);
  }

  return [...new Set(roots)];
}

function resolveCursorMdPath(extraStarts = []) {
  const explicit = String(process.env.CURSOR_MD_PATH || '').trim();
  if (explicit) {
    return path.resolve(explicit);
  }

  for (const root of collectSearchRoots(extraStarts)) {
    let dir = root;
    try {
      if (fs.existsSync(dir) && fs.statSync(dir).isFile()) {
        dir = path.dirname(dir);
      }
    } catch {
      continue;
    }

    while (dir && dir !== path.dirname(dir)) {
      const candidate = path.join(dir, 'cursor.md');
      if (fs.existsSync(candidate)) {
        return candidate;
      }
      dir = path.dirname(dir);
    }
  }

  return null;
}

function limitCursorMd(content, maxChars = getMaxChars()) {
  const text = String(content || '');
  if (text.length <= maxChars) {
    return text;
  }
  const prefixLength = Math.max(0, maxChars - MARKER.length);
  return `${text.slice(0, prefixLength).trimEnd()}${MARKER}`;
}

function readCursorMd(options = {}) {
  if (isDisabled()) {
    return { path: null, content: '' };
  }

  const filePath = resolveCursorMdPath(options.extraStarts || []);
  if (!filePath) {
    return { path: null, content: '' };
  }

  try {
    const raw = fs.readFileSync(filePath, 'utf8');
    return {
      path: filePath,
      content: limitCursorMd(raw),
    };
  } catch {
    return { path: filePath, content: '' };
  }
}

function formatInjectedContext(content, filePath) {
  const body = String(content || '').trim();
  if (!body) {
    return '';
  }
  const header = filePath ? `[cursor.md — ${filePath}]\n\n` : '[cursor.md]\n\n';
  return `${header}${body}`;
}

module.exports = {
  DEFAULT_MAX_CHARS,
  collectSearchRoots,
  resolveCursorMdPath,
  limitCursorMd,
  readCursorMd,
  formatInjectedContext,
};

'use strict';

const fs = require('fs');
const path = require('path');

const BLOCKED_FILENAMES = new Set(['codex.md']);

const INJECTED_DOCS = [
  {
    filename: 'CONTRIBUTING.local.md',
    envPath: 'CONTRIBUTING_LOCAL_PATH',
    maxCharsEnv: 'CONTRIBUTING_LOCAL_MAX_CHARS',
    defaultMaxChars: 16_000,
  },
  {
    filename: 'cursor.md',
    envPath: 'CURSOR_MD_PATH',
    maxCharsEnv: 'CURSOR_MD_MAX_CHARS',
    defaultMaxChars: 12_000,
  },
  {
    filename: 'AGENTS.md',
    envPath: 'AGENTS_MD_PATH',
    maxCharsEnv: 'AGENTS_MD_MAX_CHARS',
    defaultMaxChars: 32_000,
  },
];

function isGloballyDisabled() {
  const raw = String(process.env.ECC_PROJECT_CONTEXT_INJECT || 'on').trim().toLowerCase();
  return raw === 'off' || raw === '0' || raw === 'false';
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

function resolveDocPath(filename, envPathKey, extraStarts = []) {
  if (BLOCKED_FILENAMES.has(filename)) {
    return null;
  }

  const explicit = String(process.env[envPathKey] || '').trim();
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
      const candidate = path.join(dir, filename);
      if (fs.existsSync(candidate)) {
        return candidate;
      }
      dir = path.dirname(dir);
    }
  }

  return null;
}

function getMaxChars(doc) {
  const parsed = Number.parseInt(String(process.env[doc.maxCharsEnv] || ''), 10);
  return Number.isFinite(parsed) && parsed > 0 ? parsed : doc.defaultMaxChars;
}

function limitContent(filename, content, maxChars) {
  const text = String(content || '');
  if (text.length <= maxChars) {
    return text;
  }
  const marker = `\n\n[${filename} truncated — raise ${maxChars} cap or shorten the file]`;
  const prefixLength = Math.max(0, maxChars - marker.length);
  return `${text.slice(0, prefixLength).trimEnd()}${marker}`;
}

function readDoc(doc, extraStarts = []) {
  const filePath = resolveDocPath(doc.filename, doc.envPath, extraStarts);
  if (!filePath) {
    return { filename: doc.filename, path: null, content: '' };
  }

  try {
    const raw = fs.readFileSync(filePath, 'utf8');
    return {
      filename: doc.filename,
      path: filePath,
      content: limitContent(doc.filename, raw, getMaxChars(doc)),
    };
  } catch {
    return { filename: doc.filename, path: filePath, content: '' };
  }
}

function formatDocSection({ filename, path: filePath, content }) {
  const body = String(content || '').trim();
  if (!body) {
    return '';
  }
  const header = filePath ? `[${filename} — ${filePath}]\n\n` : `[${filename}]\n\n`;
  return `${header}${body}`;
}

function buildProjectContext(options = {}) {
  if (isGloballyDisabled()) {
    return '';
  }

  const extraStarts = options.extraStarts || [];
  const sections = [];

  for (const doc of INJECTED_DOCS) {
    const section = formatDocSection(readDoc(doc, extraStarts));
    if (section) {
      sections.push(section);
    }
  }

  return sections.join('\n\n---\n\n');
}

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

  push(input?.workspace_roots);
  push(input?.workspaceRoots);
  push(input?._cursor?.workspace_roots);
  push(process.env.CURSOR_PROJECT_DIR);
  push(process.cwd());

  return [...new Set(roots)];
}

module.exports = {
  BLOCKED_FILENAMES,
  INJECTED_DOCS,
  collectSearchRoots,
  resolveDocPath,
  readDoc,
  formatDocSection,
  buildProjectContext,
  workspaceRootsFromInput,
};

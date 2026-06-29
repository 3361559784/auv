/**
 * Session-scoped accumulator for edited file paths processed at Stop time.
 */

'use strict';

const crypto = require('crypto');
const fs = require('fs');
const os = require('os');
const path = require('path');

const JS_TS_EXT = /\.(ts|tsx|js|jsx)$/;
const RUST_EXT = /\.rs$/;

function getAccumFile() {
  const raw =
    process.env.CLAUDE_SESSION_ID ||
    crypto.createHash('sha1').update(process.cwd()).digest('hex').slice(0, 12);
  const sessionId = raw.replace(/[^a-zA-Z0-9_-]/g, '_').slice(0, 64);
  return path.join(os.tmpdir(), `ecc-edited-${sessionId}.txt`);
}

function parseAccumulator(raw) {
  return [...new Set(String(raw || '').split('\n').map(l => l.trim()).filter(Boolean))];
}

function readAccumulator() {
  const accumFile = getAccumFile();
  try {
    return { accumFile, files: parseAccumulator(fs.readFileSync(accumFile, 'utf8')) };
  } catch {
    return { accumFile, files: [] };
  }
}

function writeAccumulator(accumFile, files) {
  const unique = [...new Set(files.map(f => String(f || '').trim()).filter(Boolean))];
  if (unique.length === 0) {
    try {
      fs.unlinkSync(accumFile);
    } catch {
      /* best-effort */
    }
    return;
  }
  fs.writeFileSync(accumFile, `${unique.join('\n')}\n`, 'utf8');
}

module.exports = {
  JS_TS_EXT,
  RUST_EXT,
  getAccumFile,
  parseAccumulator,
  readAccumulator,
  writeAccumulator,
};

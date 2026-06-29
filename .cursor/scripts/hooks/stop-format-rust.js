#!/usr/bin/env node
/**
 * Stop Hook: Batch rustfmt via `cargo fmt` for edited .rs files.
 *
 * Reads the accumulator written by post-edit-accumulator.js, formats Rust
 * sources once per Cargo workspace root, then leaves non-Rust paths in the
 * accumulator for stop-format-typecheck.js.
 */

'use strict';

const fs = require('fs');
const path = require('path');
const { execFileSync, spawnSync } = require('child_process');

const {
  RUST_EXT,
  readAccumulator,
  writeAccumulator,
} = require('../lib/edited-file-accumulator');

const MAX_STDIN = 1024 * 1024;
const TOTAL_BUDGET_MS = 120_000;
const UNSAFE_PATH_CHARS = /[&|<>^%!\s()]/;

function findCargoWorkspaceRoot(filePath) {
  let dir = path.dirname(path.resolve(filePath));
  const fsRoot = path.parse(dir).root;
  let nearest = null;
  let workspaceRoot = null;
  let depth = 0;

  while (dir !== fsRoot && depth < 40) {
    const cargoToml = path.join(dir, 'Cargo.toml');
    if (fs.existsSync(cargoToml)) {
      nearest = dir;
      try {
        const content = fs.readFileSync(cargoToml, 'utf8');
        if (content.includes('[workspace]')) {
          workspaceRoot = dir;
        }
      } catch {
        /* keep nearest */
      }
    }
    dir = path.dirname(dir);
    depth += 1;
  }

  return workspaceRoot || nearest;
}

function toWorkspaceRelative(workspaceRoot, filePath) {
  const resolved = path.resolve(filePath);
  const rel = path.relative(workspaceRoot, resolved);
  return rel.split(path.sep).join('/');
}

function resolveCargoBin() {
  return process.platform === 'win32' ? 'cargo.cmd' : 'cargo';
}

function formatRustBatch(workspaceRoot, files, timeoutMs) {
  const existingFiles = files.filter(f => fs.existsSync(f));
  if (existingFiles.length === 0) {
    return;
  }

  const relPaths = existingFiles.map(f => toWorkspaceRelative(workspaceRoot, f));
  const cargoBin = resolveCargoBin();
  const args = ['fmt', '--', ...relPaths];
  const opts = {
    cwd: workspaceRoot,
    stdio: ['pipe', 'pipe', 'pipe'],
    timeout: timeoutMs,
  };

  try {
    if (process.platform === 'win32' && cargoBin.endsWith('.cmd')) {
      if (existingFiles.some(f => UNSAFE_PATH_CHARS.test(f))) {
        process.stderr.write('[Hook] stop-format-rust: skipping batch — unsafe path chars\n');
        return;
      }
      const result = spawnSync(cargoBin, args, { ...opts, shell: true });
      if (result.error) {
        throw result.error;
      }
    } else {
      execFileSync(cargoBin, args, opts);
    }
  } catch (err) {
    const message = err && err.message ? err.message : String(err);
    process.stderr.write(`[Hook] stop-format-rust: cargo fmt failed in ${workspaceRoot}: ${message}\n`);
  }
}

function partitionEditedFiles(files) {
  const rustFiles = [];
  const otherFiles = [];

  for (const filePath of files) {
    if (RUST_EXT.test(filePath)) {
      rustFiles.push(filePath);
    } else {
      otherFiles.push(filePath);
    }
  }

  return { rustFiles, otherFiles };
}

function groupRustFilesByWorkspace(rustFiles) {
  const byWorkspaceRoot = new Map();

  for (const filePath of rustFiles) {
    const resolved = path.resolve(filePath);
    if (!fs.existsSync(resolved)) {
      continue;
    }
    const workspaceRoot = findCargoWorkspaceRoot(resolved);
    if (!workspaceRoot) {
      continue;
    }
    if (!byWorkspaceRoot.has(workspaceRoot)) {
      byWorkspaceRoot.set(workspaceRoot, []);
    }
    byWorkspaceRoot.get(workspaceRoot).push(resolved);
  }

  return byWorkspaceRoot;
}

function main() {
  const { accumFile, files } = readAccumulator();
  if (files.length === 0) {
    return;
  }

  const { rustFiles, otherFiles } = partitionEditedFiles(files);
  if (rustFiles.length === 0) {
    return;
  }

  const byWorkspaceRoot = groupRustFilesByWorkspace(rustFiles);
  const totalBatches = byWorkspaceRoot.size;
  const perBatchMs = totalBatches > 0 ? Math.floor(TOTAL_BUDGET_MS / totalBatches) : 60_000;

  for (const [workspaceRoot, batch] of byWorkspaceRoot) {
    formatRustBatch(workspaceRoot, batch, perBatchMs);
  }

  writeAccumulator(accumFile, otherFiles);
}

function run(rawInput) {
  try {
    main();
  } catch (err) {
    process.stderr.write(`[Hook] stop-format-rust error: ${err.message}\n`);
  }
  return rawInput;
}

if (require.main === module) {
  let stdinData = '';
  let truncated = false;
  process.stdin.setEncoding('utf8');
  process.stdin.on('data', chunk => {
    if (stdinData.length < MAX_STDIN) {
      const remaining = MAX_STDIN - stdinData.length;
      stdinData += chunk.substring(0, remaining);
      if (chunk.length > remaining) {
        truncated = true;
      }
    } else {
      truncated = true;
    }
  });
  process.stdin.on('end', () => {
    const output = run(stdinData);
    if (truncated) {
      process.stderr.write('[Hook] stop-format-rust: stdin exceeded 1MB; suppressing pass-through (fail-open)\n');
      process.exit(0);
    }
    if (!output) {
      process.exit(0);
    }
    process.stdout.write(output, () => process.exit(0));
  });
}

module.exports = { run, partitionEditedFiles, findCargoWorkspaceRoot };

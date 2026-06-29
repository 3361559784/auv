#!/usr/bin/env node
'use strict';

const assert = require('assert');
const fs = require('fs');
const os = require('os');
const path = require('path');
const { spawnSync } = require('child_process');

const repoRoot = path.resolve(__dirname, '..', '..', '..');
const adapter = require(path.join(repoRoot, '.cursor', 'hooks', 'adapter'));
const { resolveCursorEccPluginRoot } = require(path.join(repoRoot, '.cursor', 'scripts', 'lib', 'cursor-ecc-root'));

function runHook(scriptName, input) {
  return spawnSync('node', [path.join(repoRoot, '.cursor', 'hooks', scriptName)], {
    input: JSON.stringify(input),
    encoding: 'utf8',
    cwd: repoRoot,
    env: {
      ...process.env,
      ECC_HOOK_PROFILE: 'standard',
      CURSOR_PROJECT_DIR: repoRoot,
    },
  });
}

function test(name, fn) {
  try {
    fn();
    console.log(`ok ${name}`);
    return true;
  } catch (error) {
    console.error(`fail ${name}: ${error.message}`);
    return false;
  }
}

let passed = 0;
let failed = 0;
function check(name, fn) {
  if (test(name, fn)) passed += 1; else failed += 1;
}

check('resolveCursorEccPluginRoot points at vendored .cursor', () => {
  const pluginRoot = resolveCursorEccPluginRoot({ hostRoot: repoRoot });
  assert.equal(pluginRoot, path.join(repoRoot, '.cursor'));
  assert.ok(fs.existsSync(path.join(pluginRoot, 'scripts', 'hooks', 'post-edit-accumulator.js')));
});

check('adapter.getPluginRoot matches vendored layout', () => {
  const pluginRoot = adapter.getPluginRoot();
  assert.ok(fs.existsSync(path.join(pluginRoot, 'scripts', 'hooks', 'session-start.js')));
});

check('before-shell-execution loads shell-split', () => {
  require(path.join(repoRoot, '.cursor', 'hooks', 'before-shell-execution.js'));
});

check('session-start emits CLAUDE_PLUGIN_ROOT env payload', () => {
  const result = runHook('session-start.js', {
    hook_event_name: 'sessionStart',
    workspace_roots: [repoRoot],
  });
  assert.equal(result.status, 0, result.stderr);
  const payload = JSON.parse(result.stdout.trim());
  assert.ok(payload.env.CLAUDE_PLUGIN_ROOT.includes('.cursor'));
  assert.ok(payload.env.ECC_AGENT_DATA_HOME);
});

check('after-file-edit reaches post-edit accumulator', () => {
  const tmpFile = path.join(os.tmpdir(), `ecc-smoke-${process.pid}.ts`);
  fs.writeFileSync(tmpFile, 'export const x = 1\n');
  fs.writeFileSync(tmpFile, 'export const x = 1\n');
  const result = runHook('after-file-edit.js', {
    hook_event_name: 'afterFileEdit',
    path: tmpFile,
    workspace_roots: [repoRoot],
  });
  assert.equal(result.status, 0, result.stderr);
  const accum = path.join(
    os.tmpdir(),
    `ecc-edited-${require('crypto').createHash('sha1').update(repoRoot).digest('hex').slice(0, 12)}.txt`
  );
  const raw = fs.existsSync(accum) ? fs.readFileSync(accum, 'utf8') : '';
  assert.ok(raw.includes(tmpFile), `accumulator missing edited path: ${accum}`);
  fs.unlinkSync(tmpFile);
});

check('stop hook runs without throwing', () => {
  const result = runHook('stop.js', {
    hook_event_name: 'stop',
    session_id: `smoke-${process.pid}`,
    transcript_path: path.join(repoRoot, 'missing-transcript.jsonl'),
    cwd: repoRoot,
    last_assistant_message: 'smoke',
  });
  assert.equal(result.status, 0, result.stderr);
});

console.log(`cursor-ecc-smoke: ${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);

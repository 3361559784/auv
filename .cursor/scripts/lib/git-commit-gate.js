'use strict';

const { spawnSync } = require('child_process');

const MAX_STDIN = 1024 * 1024;
const RUST_PATH_RE = /\.(rs|toml|lock)$/i;
const CARGO_MANIFEST = /(^|\/)Cargo\.(toml|lock)$/;

const PAUSED_LANE_PATHS = [
  'src/candidate_action_decision.rs',
  'src/candidate_action_command.rs',
];

const CORE_RUNTIME_PATH_PREFIXES = [
  'src/runtime.rs',
  'src/contract.rs',
  'src/catalog.rs',
  'src/invoke',
  'crates/auv-runtime',
];

function extractCommand(rawInput) {
  const trimmed = String(rawInput || '').trim();
  if (!trimmed.startsWith('{')) {
    return trimmed;
  }

  try {
    const parsed = JSON.parse(trimmed);
    if (typeof parsed !== 'object' || parsed === null) {
      return trimmed;
    }

    const cmd = parsed.tool_input?.command;
    if (typeof cmd === 'string') {
      return cmd;
    }

    for (const key of ['command', 'cmd', 'input', 'shell', 'script']) {
      if (typeof parsed[key] === 'string') {
        return parsed[key];
      }
    }

    return trimmed;
  } catch {
    return trimmed;
  }
}

function isGitCommitCommand(command) {
  return /\bgit\s+commit\b/.test(String(command || ''));
}

function isGitPushCommand(command) {
  return /\bgit\s+push\b/.test(String(command || ''));
}

function isAmendCommit(command) {
  return /\b--amend\b/.test(String(command || ''));
}

function getStagedFiles() {
  const result = spawnSync('git', ['diff', '--cached', '--name-only', '--diff-filter=ACMR'], {
    encoding: 'utf8',
    stdio: ['pipe', 'pipe', 'pipe'],
  });
  if (result.status !== 0) {
    return [];
  }
  return result.stdout.trim().split('\n').filter(Boolean);
}

function isRustPath(filePath) {
  const value = String(filePath || '');
  return RUST_PATH_RE.test(value) || CARGO_MANIFEST.test(value);
}

function runCargo(args, options = {}) {
  const timeout = options.timeout ?? 600_000;
  return spawnSync('cargo', args, {
    encoding: 'utf8',
    stdio: ['pipe', 'pipe', 'pipe'],
    timeout,
    cwd: options.cwd || process.cwd(),
  });
}

function runGit(args) {
  return spawnSync('git', args, {
    encoding: 'utf8',
    stdio: ['pipe', 'pipe', 'pipe'],
  });
}

function extractCommitSubject(command) {
  const value = String(command || '');
  const quoted = value.match(/(?:^|\s)(?:-m|--message)(?:=|\s+)(['"])([\s\S]*?)\1/);
  if (quoted) {
    return quoted[2].split('\n')[0].trim();
  }

  const bare = value.match(/(?:^|\s)(?:-m|--message)(?:=|\s+)(\S+)/);
  if (bare) {
    return bare[1].trim();
  }

  return '';
}

function getOutgoingFiles() {
  const upstream = spawnSync('git', ['rev-parse', '--abbrev-ref', '@{u}'], {
    encoding: 'utf8',
    stdio: ['pipe', 'pipe', 'pipe'],
  });
  if (upstream.status === 0 && upstream.stdout.trim()) {
    const diff = spawnSync('git', ['diff', '--name-only', '@{u}..HEAD'], {
      encoding: 'utf8',
      stdio: ['pipe', 'pipe', 'pipe'],
    });
    if (diff.status === 0) {
      return diff.stdout.trim().split('\n').filter(Boolean);
    }
  }

  for (const base of ['origin/main', 'origin/master', 'main']) {
    const mergeBase = spawnSync('git', ['merge-base', 'HEAD', base], {
      encoding: 'utf8',
      stdio: ['pipe', 'pipe', 'pipe'],
    });
    if (mergeBase.status !== 0 || !mergeBase.stdout.trim()) {
      continue;
    }
    const diff = spawnSync('git', ['diff', '--name-only', `${mergeBase.stdout.trim()}..HEAD`], {
      encoding: 'utf8',
      stdio: ['pipe', 'pipe', 'pipe'],
    });
    if (diff.status === 0) {
      return diff.stdout.trim().split('\n').filter(Boolean);
    }
  }

  return [];
}

function hookResult(rawInput, { exitCode, lines }) {
  return {
    output: rawInput,
    exitCode,
    stderr: Array.isArray(lines) ? lines.join('\n') : '',
  };
}

function evaluateGitCommitHook(rawInput, handler) {
  const command = extractCommand(rawInput);
  if (!isGitCommitCommand(command)) {
    return hookResult(rawInput, { exitCode: 0, lines: [] });
  }

  if (isAmendCommit(command)) {
    return hookResult(rawInput, { exitCode: 0, lines: [] });
  }

  const stagedFiles = getStagedFiles();
  if (stagedFiles.length === 0) {
    return hookResult(rawInput, {
      exitCode: 0,
      lines: ['[AUV gate] No staged files; skipping commit validation.'],
    });
  }

  const inner = handler({
    command,
    stagedFiles,
    subject: extractCommitSubject(command),
    rawInput,
  });
  return inner;
}

function evaluateGitPushHook(rawInput, handler) {
  const command = extractCommand(rawInput);
  if (!isGitPushCommand(command)) {
    return hookResult(rawInput, { exitCode: 0, lines: [] });
  }

  const outgoingFiles = getOutgoingFiles();
  return handler({
    command,
    outgoingFiles,
    rawInput,
  });
}

function touchesPausedLane(stagedFiles) {
  return stagedFiles.filter(file =>
    PAUSED_LANE_PATHS.some(marker => file === marker || file.startsWith(`${marker}/`)),
  );
}

function touchesCoreRuntime(stagedFiles) {
  return stagedFiles.filter(file =>
    CORE_RUNTIME_PATH_PREFIXES.some(prefix => file === prefix || file.startsWith(prefix)),
  );
}

function isDocsOnlyPath(filePath) {
  return (
    filePath.startsWith('docs/') ||
    filePath.endsWith('.md') ||
    (filePath.startsWith('.cursor/') && filePath.endsWith('.md'))
  );
}

function isCodePath(filePath) {
  return (
    filePath.startsWith('src/') ||
    filePath.startsWith('crates/') ||
    filePath.endsWith('.rs') ||
    filePath.endsWith('Cargo.toml')
  );
}

function parseRenameSummary(summaryText) {
  const renames = [];
  for (const line of String(summaryText || '').split('\n')) {
    const match = line.match(/^\s*rename\s+(.+?)\s+=>\s+(.+?)\s+\((\d+)%\)/);
    if (match) {
      renames.push({
        from: match[1].trim(),
        to: match[2].trim(),
        similarity: Number(match[3]),
      });
    }
  }
  return renames;
}

function analyzeStagedSlice({ stagedFiles, subject }) {
  const violations = [];
  const message = String(subject || '').trim();

  const paused = touchesPausedLane(stagedFiles);
  const core = touchesCoreRuntime(stagedFiles);
  if (paused.length > 0 && core.length > 0) {
    violations.push({
      code: 'paused-lane-mix',
      message:
        'Paused candidate-action lane changes are staged together with core runtime paths. Split the slice.',
      detail: `paused: ${paused.join(', ')}; core: ${core.join(', ')}`,
    });
  }

  const docs = stagedFiles.filter(isDocsOnlyPath);
  const code = stagedFiles.filter(isCodePath);
  const docsOnlyCommit = /^(docs|test)(\([^)]*\))?:/i.test(message);
  if (docs.length > 0 && code.length > 0 && !docsOnlyCommit) {
    violations.push({
      code: 'docs-code-mix',
      message:
        'Docs and code are staged in one commit without a docs(...) or test(...) subject. Split or relabel the slice.',
      detail: `docs: ${docs.slice(0, 5).join(', ')}${docs.length > 5 ? '…' : ''}; code: ${code.slice(0, 5).join(', ')}${code.length > 5 ? '…' : ''}`,
    });
  }

  const summary = spawnSync('git', ['diff', '--cached', '--summary'], {
    encoding: 'utf8',
    stdio: ['pipe', 'pipe', 'pipe'],
  });
  const renames = parseRenameSummary(summary.stdout);
  const impureRename = renames.some(entry => entry.similarity < 100);
  const numstat = spawnSync('git', ['diff', '--cached', '--numstat'], {
    encoding: 'utf8',
    stdio: ['pipe', 'pipe', 'pipe'],
  });
  const substantive = String(numstat.stdout || '')
    .split('\n')
    .filter(Boolean)
    .map(line => {
      const parts = line.split('\t');
      return {
        file: parts[2],
        added: Number(parts[0]) || 0,
        deleted: Number(parts[1]) || 0,
      };
    })
    .filter(entry => entry.added + entry.deleted > 0);

  const renameTargets = new Set(renames.flatMap(entry => [entry.from, entry.to]));
  const nonRenameSubstantive = substantive.filter(entry => !renameTargets.has(entry.file));
  const moveSlice = /^(refactor|chore)(\([^)]*\))?:/i.test(message) || /\bmove-only\b/i.test(message);

  if (renames.length > 0 && (impureRename || nonRenameSubstantive.length > 0) && !moveSlice) {
    violations.push({
      code: 'rename-behavior-mix',
      message:
        'Rename/move noise is mixed with substantive staged edits. Use refactor(...) or move-only in the subject, or split commits.',
      detail: `renames: ${renames.length}; other edited files: ${nonRenameSubstantive.length}`,
    });
  }

  return violations;
}

module.exports = {
  MAX_STDIN,
  extractCommand,
  isGitCommitCommand,
  isGitPushCommand,
  isAmendCommit,
  getStagedFiles,
  getOutgoingFiles,
  isRustPath,
  runCargo,
  extractCommitSubject,
  hookResult,
  evaluateGitCommitHook,
  evaluateGitPushHook,
  analyzeStagedSlice,
  touchesPausedLane,
  touchesCoreRuntime,
};

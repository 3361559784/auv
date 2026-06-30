'use strict';

const { isGateGuardEnforced } = require('../lib/gateguard-project-disable');

const BYPASS_PATTERNS = [
  { code: 'ecc-gateguard-off', pattern: /\bECC_GATEGUARD\s*=\s*(?:off|0|false|disabled|no)\b/i },
  { code: 'gateguard-disabled', pattern: /\bGATEGUARD_DISABLED\s*=\s*(?:1|true|on|yes|enabled)\b/i },
  {
    code: 'ecc-disabled-hooks-gateguard',
    pattern: /\bECC_DISABLED_HOOKS\s*=\s*[^\s;|&"']*gateguard/i,
  },
];

function extractCommand(rawInput) {
  if (!rawInput) return '';
  const trimmed = String(rawInput).trim();
  if (!trimmed) return '';
  try {
    const parsed = JSON.parse(trimmed);
    return String(parsed.tool_input?.command || parsed.command || parsed.args?.command || '');
  } catch {
    return trimmed;
  }
}

function analyzeCommand(command) {
  if (!isGateGuardEnforced()) {
    return { blocked: false, violations: [] };
  }

  const cmd = String(command || '');
  if (!cmd.trim()) {
    return { blocked: false, violations: [] };
  }

  const violations = [];
  for (const { code, pattern } of BYPASS_PATTERNS) {
    if (pattern.test(cmd)) {
      violations.push({
        code,
        message:
          'GateGuard bypass env vars are blocked while `.cursor/hooks/gateguard-enforced.js` is present.',
      });
    }
  }

  return { blocked: violations.length > 0, violations };
}

function evaluate(rawInput) {
  const command = extractCommand(rawInput);
  const analysis = analyzeCommand(command);
  if (!analysis.blocked) {
    return { exitCode: 0, stderr: '', output: '' };
  }

  const lines = [
    '[AUV gate] BLOCKED: GateGuard bypass attempt in shell command.',
    ...analysis.violations.map(v => `- ${v.code}: ${v.message}`),
    'Complete the AUV pre-edit slice gate in chat, then retry the same tool call.',
    'Do not prefix commands with ECC_GATEGUARD=off or disable gateguard hook ids.',
  ];

  return { exitCode: 2, stderr: lines.join('\n'), output: '' };
}

function run(rawInput) {
  const result = evaluate(rawInput);
  return {
    stdout: result.output,
    stderr: result.stderr,
    exitCode: result.exitCode,
  };
}

if (require.main === module) {
  let data = '';
  process.stdin.setEncoding('utf8');
  process.stdin.on('data', chunk => {
    if (data.length < 1024 * 1024) {
      data += chunk.substring(0, 1024 * 1024 - data.length);
    }
  });
  process.stdin.on('end', () => {
    const result = evaluate(data);
    if (result.stderr) {
      process.stderr.write(`${result.stderr}\n`);
    }
    process.stdout.write(result.output);
    process.exit(result.exitCode);
  });
}

module.exports = { run, evaluate, analyzeCommand };

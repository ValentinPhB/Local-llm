import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { existsSync, readFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

// npm's parent lockfile loses the dev flags of SpecDD's nested shrinkwrap.
// Only exclude an advisory when every affected path is absent AND marked dev
// in that upstream shrinkwrap. An installed package is never excluded.
export function classifyAudit(report, installed, upstreamDev) {
  assert.equal(report.auditReportVersion, 2, 'Unsupported or unavailable npm audit report');
  assert.ok(!report.error && report.vulnerabilities && report.metadata?.vulnerabilities,
    'Incomplete npm audit report');
  const active = [];
  const excluded = [];
  for (const [name, finding] of Object.entries(report.vulnerabilities)) {
    assert.ok(Array.isArray(finding.nodes) && finding.nodes.length > 0, `Missing paths: ${name}`);
    for (const path of finding.nodes) {
      assert.ok(typeof path === 'string' && path.startsWith('node_modules/')
        && !path.split('/').some(part => part === '..' || part === '.')
        && !path.includes('\\'), `Invalid dependency path: ${name}`);
    }
    const safelyExcluded = finding.nodes.every(path => !installed(path) && upstreamDev(path));
    (safelyExcluded ? excluded : active).push({ name, severity: finding.severity });
  }
  const blocking = active.filter(finding => !['info', 'low', 'moderate'].includes(finding.severity));
  return { active, excluded, blocking };
}

const script = fileURLToPath(import.meta.url);
if (process.argv[1] && resolve(process.argv[1]) === script) {
  try {
    const tooling = dirname(script);
    assert.ok(process.env.npm_execpath, 'Run this check through npm run audit');
    const result = spawnSync(process.execPath, [process.env.npm_execpath,
      'audit', '--omit=dev', '--json'], {
      cwd: tooling, encoding: 'utf8', timeout: 45_000,
      env: process.env,
    });
    assert.ifError(result.error);
    assert.ok(result.status === 0 || result.status === 1, result.stderr);
    const shrinkwrap = JSON.parse(readFileSync(join(tooling, 'node_modules/specdd/npm-shrinkwrap.json')));
    const prefix = 'node_modules/specdd/';
    const summary = classifyAudit(JSON.parse(result.stdout),
      path => existsSync(join(tooling, path)),
      path => path.startsWith(prefix)
        && shrinkwrap.packages?.[path.slice(prefix.length)]?.dev === true);
    console.log(JSON.stringify(summary, null, 2));
    process.exitCode = summary.blocking.length ? 1 : 0;
  } catch (error) {
    console.error(`Dependency audit failed: ${error.message}`);
    process.exitCode = 1;
  }
}

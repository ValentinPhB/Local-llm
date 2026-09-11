import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { runtimeLock, sha256, toolRoot } from './launch-policy.mjs';

export function checkAuditResults(npm, osv, expectedCount) {
  assert.equal(npm.auditReportVersion, 2, 'Invalid npm audit response');
  assert.ok(npm.vulnerabilities && typeof npm.vulnerabilities === 'object');
  assert.equal(npm.metadata?.vulnerabilities?.total, 0, 'npm vulnerabilities');
  assert.equal(Object.keys(npm.vulnerabilities).length, 0, 'npm findings');
  assert.ok(!npm.error && !osv.error, 'Audit unavailable');
  assert.ok(Number.isInteger(expectedCount) && expectedCount > 0);
  assert.ok(Array.isArray(osv.results));
  assert.equal(osv.results.length, expectedCount, 'Incomplete OSV audit');
  for (const result of osv.results) {
    assert.ok(result && typeof result === 'object' && !Array.isArray(result));
    assert.ok(!result.error && !result.next_page_token, 'Incomplete OSV result');
    if ('vulns' in result) assert.ok(Array.isArray(result.vulns) && result.vulns.length === 0, 'OSV vulnerabilities');
  }
}

export function checkBundledFiles(packageRoot = toolRoot) {
  const review = JSON.parse(readFileSync(join(toolRoot, 'bundled-review.json')));
  assert.equal(review.puppeteerVersion, runtimeLock.puppeteerVersion);
  for (const c of review.components.filter(c => c.file)) {
    assert.equal(sha256(readFileSync(join(packageRoot, 'node_modules/puppeteer-core', c.file))), c.sha256,
      `Bundled component changed: ${c.name}`);
  }
  return review.components;
}

export function auditQueries(lock, components) {
  const pairs = Object.entries(lock.packages).filter(([path]) => path).map(([path, entry]) => ({
    name: path.slice(path.lastIndexOf('node_modules/') + 13), version: entry.version,
  })).concat(components);
  return [...new Map(pairs.map(p => [`${p.name}@${p.version}`, {
    package: { name: p.name, ecosystem: 'npm' }, version: p.version,
  }])).values()];
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const lock = JSON.parse(readFileSync(join(toolRoot, 'package-lock.json')));
  const queries = auditQueries(lock, checkBundledFiles());
  const npmCli = resolve(dirname(process.execPath), '../lib/node_modules/npm/bin/npm-cli.js');
  const npm = JSON.parse(execFileSync(process.execPath, [npmCli, 'audit', '--package-lock-only', '--include=dev', '--json',
    '--cache', resolve(toolRoot, '../../.local/browser-tools-review/npm-cache')], {
    cwd: toolRoot, encoding: 'utf8', timeout: 45_000, maxBuffer: 2_000_000,
  }));
  const osv = JSON.parse(execFileSync('curl', ['-fsS', '--max-time', '30',
    'https://api.osv.dev/v1/querybatch', '-H', 'Content-Type: application/json', '--data-binary', '@-'], {
    input: JSON.stringify({ queries }), encoding: 'utf8', timeout: 35_000, maxBuffer: 2_000_000,
  }));
  checkAuditResults(npm, osv, queries.length);
  console.log(JSON.stringify({ packages: Object.keys(lock.packages).length - 1, uniqueIncludingBundled: queries.length,
    npm: 'zero alerts', osv: 'zero alerts', date: new Date().toISOString() }));
}

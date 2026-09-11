// Read-only inspection of the reviewed npm archives; no package execution.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { join, resolve } from 'node:path';
import { toolRoot, runtimeLock, sha256 } from './launch-policy.mjs';

const lock = JSON.parse(readFileSync(join(toolRoot, 'package-lock.json')));
const review = JSON.parse(readFileSync(join(toolRoot, 'bundled-review.json')));
assert.equal(review.puppeteerVersion, runtimeLock.puppeteerVersion);
const archives = resolve(toolRoot, '../../.local/browser-tools-review/puppeteer-candidate/archives');
const tar = args => execFileSync('tar', args, { encoding: 'utf8', timeout: 10_000, maxBuffer: 16_000_000 });
const results = [];
for (const [path, entry] of Object.entries(lock.packages)) {
  if (!path) continue;
  const archive = join(archives, path.replaceAll('/', '_') + '.tgz');
  assert.equal('sha512-' + createHash('sha512').update(readFileSync(archive)).digest('base64'), entry.integrity);
  const paths = tar(['-tf', archive]).trim().split('\n');
  assert.ok(paths.every(p => p.startsWith('package/') && !p.split('/').includes('..') && !p.includes('\\')));
  const manifest = JSON.parse(tar(['-xOf', archive, 'package/package.json']));
  assert.equal(manifest.version, entry.version);
  for (const script of ['preinstall', 'install', 'postinstall']) assert.equal(manifest.scripts?.[script], undefined);
  if (manifest.name === 'puppeteer-core') {
    const bundledPaths = paths.filter(p => /^package\/lib\/third_party\/[^/]+\/[^/]+\.js$/.test(p));
    assert.deepEqual(bundledPaths.sort(), review.components.filter(c => c.file).map(c => 'package/' + c.file).sort());
    for (const c of review.components.filter(c => c.file)) {
      assert.equal(sha256(tar(['-xOf', archive, 'package/' + c.file])), c.sha256, `Changed bundle ${c.name}`);
    }
  }
  results.push({ name: manifest.name, version: entry.version, license: entry.license });
}
console.log(JSON.stringify({ distributions: results.length, bundledVersions: review.components.length, results }, null, 2));

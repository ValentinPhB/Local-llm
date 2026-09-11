import test from 'node:test';
import assert from 'node:assert/strict';
import { chmodSync, mkdirSync, mkdtempSync, readFileSync, rmSync, symlinkSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { dirname, join } from 'node:path';
import { checkedLaunchOptions, checkedBrowserArguments, checkedEnvironment, checkedFixtureUrls, runtimeLock, sha256 } from './launch-policy.mjs';

function fixture(t) {
  const root = mkdtempSync(join(tmpdir(), 'chatpurp-browser-policy-'));
  t.after(() => rmSync(root, { recursive: true }));
  const runtimeRoot = join(root, 'runtime'), packageRoot = join(root, 'packages');
  mkdirSync(runtimeRoot);
  const packages = { '': {}, 'node_modules/puppeteer-core': { version: '25.10.0' }, 'node_modules/ws': { version: '8.21.3' } };
  for (const [path, entry] of Object.entries(packages)) {
    if (!path) continue;
    mkdirSync(join(packageRoot, path), { recursive: true });
    writeFileSync(join(packageRoot, path, 'package.json'), JSON.stringify({ name: path.slice(13), version: entry.version }));
  }
  writeFileSync(join(packageRoot, 'package-lock.json'), JSON.stringify({ packages }));
  const binary = join(runtimeRoot, runtimeLock.browser.executableRelativePath);
  mkdirSync(dirname(binary), { recursive: true });
  writeFileSync(binary, 'FIXTURE NEVER EXECUTED', { mode: 0o700 });
  return { root, binary, options: { platform: 'darwin', arch: 'arm64', nodeVersion: '22.23.2', runtimeRoot,
    packageRoot, environment: {}, expectedBrowserHash: sha256('FIXTURE NEVER EXECUTED') } };
}
test('active pins match manifests and exclude Playwright', () => {
  const npmLock = JSON.parse(readFileSync(new URL('package-lock.json', import.meta.url)));
  assert.equal(runtimeLock.puppeteerVersion, '25.10.0');
  assert.equal(runtimeLock.browser.version, '153.0.8010.36');
  assert.equal(npmLock.packages['node_modules/puppeteer-core'].version, runtimeLock.puppeteerVersion);
  assert.ok(Object.keys(npmLock.packages).every(p => !p.includes('playwright')));
  assert.equal(readFileSync(new URL('.node-version', import.meta.url), 'utf8').trim(), runtimeLock.node.version);
  for (const h of [runtimeLock.node.archiveSha256, runtimeLock.node.executableSha256,
    runtimeLock.browser.archiveSha256, runtimeLock.browser.executableSha256]) assert.match(h, /^[a-f0-9]{64}$/);
});
test('dedicated shell, pipe, bounded deadlines and downloads denied', t => {
  const { options } = fixture(t), result = checkedLaunchOptions(options);
  assert.equal(result.headless, 'shell');
  assert.equal(result.pipe, true);
  assert.equal(result.timeout, 10_000);
  assert.equal(result.protocolTimeout, 5_000);
  assert.equal(result.downloadBehavior.policy, 'deny');
  assert.ok(!('channel' in result));
  checkedBrowserArguments(result.args);
});
test('wrong Node rejected', t => {
  const { options } = fixture(t);
  assert.throws(() => checkedLaunchOptions({ ...options, nodeVersion: '22.22.0' }), /Node 22.23.2 requis/);
});
test('wrong platform or CPU rejected', t => {
  const { options } = fixture(t);
  assert.throws(() => checkedLaunchOptions({ ...options, platform: 'linux' }), /macOS ARM64/);
  assert.throws(() => checkedLaunchOptions({ ...options, arch: 'x64' }), /macOS ARM64/);
});
test('missing dependency rejected', t => {
  const { options } = fixture(t);
  rmSync(join(options.packageRoot, 'node_modules/ws/package.json'));
  assert.throws(() => checkedLaunchOptions(options), /ENOENT/);
});
test('wrong transitive dependency version rejected', t => {
  const { options } = fixture(t);
  writeFileSync(join(options.packageRoot, 'node_modules/ws/package.json'), JSON.stringify({ name: 'ws', version: '0.0.0' }));
  assert.throws(() => checkedLaunchOptions(options), /Version de paquet non conforme/);
});
test('missing browser has no fallback', t => {
  const { options, binary } = fixture(t);
  rmSync(binary);
  assert.throws(() => checkedLaunchOptions(options), /ENOENT/);
});
test('directory instead of browser rejected', t => {
  const { options, binary } = fixture(t); rmSync(binary); mkdirSync(binary);
  assert.throws(() => checkedLaunchOptions(options), /fichier régulier/);
});
test('direct browser symlink rejected', t => {
  const { options, binary, root } = fixture(t); rmSync(binary);
  writeFileSync(join(root, 'other'), 'FAKE'); symlinkSync(join(root, 'other'), binary);
  assert.throws(() => checkedLaunchOptions(options), /fichier régulier/);
});
test('parent browser symlink rejected', t => {
  const { options, binary, root } = fixture(t); rmSync(dirname(binary), { recursive: true });
  mkdirSync(join(root, 'other')); writeFileSync(join(root, 'other/chrome-headless-shell'), 'FAKE');
  symlinkSync(join(root, 'other'), dirname(binary));
  assert.throws(() => checkedLaunchOptions(options), /fichier régulier/);
});
test('non executable browser rejected', t => {
  const { options, binary } = fixture(t); chmodSync(binary, 0o600);
  assert.throws(() => checkedLaunchOptions(options), /EACCES/);
});
test('modified binary at correct path rejected', t => {
  const { options, binary } = fixture(t); writeFileSync(binary, 'TAMPERED');
  assert.throws(() => checkedLaunchOptions(options), /Empreinte navigateur/);
});
test('environment cannot inject options or disable sandbox', () => {
  for (const key of ['PUPPETEER_DANGEROUS_NO_SANDBOX', 'PUPPETEER_EXECUTABLE_PATH', 'NODE_OPTIONS', 'NODE_PATH', 'DYLD_INSERT_LIBRARIES', 'LD_PRELOAD']) {
    assert.throws(() => checkedEnvironment({ [key]: 'true' }), /Variable/);
  }
  checkedEnvironment({ PATH: '/usr/bin' });
});
test('unsafe flags and debug ports rejected, including value forms', () => {
  for (const flag of ['--no-sandbox', '--disable-setuid-sandbox', '--disable-gpu-sandbox', '--disable-web-security',
    '--single-process', '--no-zygote', '--remote-debugging-port=0', '--remote-debugging-address=0.0.0.0']) {
    assert.throws(() => checkedBrowserArguments([flag]), /interdits/);
  }
  assert.throws(() => checkedBrowserArguments(['--headless'], { requirePipe: true }), /Pipe/);
  checkedBrowserArguments(['--headless', '--remote-debugging-pipe'], { requirePipe: true });
});

test('fixture network whitelist is exact, single-origin and excludes lab services', () => {
  assert.equal(checkedFixtureUrls(['http://127.0.0.1:54321/', 'http://127.0.0.1:54321/file.js']).size, 2);
  for (const urls of [['https://example.com/'], ['http://localhost:54321/'], ['http://127.0.0.1:3210/'],
    ['http://127.0.0.1:11434/'], ['http://127.0.0.1:6333/'], ['http://127.0.0.1:54321/?x=1'],
    ['http://127.0.0.1:54321/', 'http://127.0.0.1:54322/']]) assert.throws(() => checkedFixtureUrls(urls));
});

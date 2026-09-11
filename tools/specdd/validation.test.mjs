import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { existsSync, mkdtempSync, readFileSync, realpathSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { dirname, isAbsolute, join, relative, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import test from 'node:test';
import { classifyAudit } from './audit-runtime.mjs';

const tooling = dirname(fileURLToPath(import.meta.url));
const root = resolve(tooling, '../..');
const cli = join(tooling, 'node_modules/specdd/dist/main.js');
const subject = 'specs/001-demo-session-document-read';
const filename = `${subject}/001-demo-session-document-read.sdd`;
const expectedIds = [
  'SES-01', 'SES-02', 'SES-03', 'ACL-01', 'ACL-02', 'ACL-03',
  'DOC-01', 'DOC-02', 'DOC-03', 'AUD-01', 'AUD-02', 'UI-01', 'ISO-01', 'HTTP-01',
];

function run(...args) {
  const result = spawnSync(process.execPath, [cli, ...args], {
    cwd: root, encoding: 'utf8', timeout: 15_000,
    env: { ...process.env, NO_COLOR: '1' },
  });
  assert.ifError(result.error);
  return result;
}

function resolvedContract(directory = subject) {
  const result = run('resolve', '--root', root, directory, '--sections', 'all', '--format', 'json');
  assert.equal(result.status, 0, result.stdout + result.stderr);
  const specs = JSON.parse(result.stdout).directories.flatMap(directory => directory.specs);
  const name = directory === subject ? filename : `${directory}/${directory.split('/').at(-1)}.sdd`;
  const matching = specs.filter(spec => spec.path === name);
  assert.equal(matching.length, 1, 'The pilot must be discovered exactly once');
  assert.equal(matching[0].directoryLevel, true, 'The spec must govern its directory');
  return matching[0];
}

function entries(spec, section) {
  return (spec.sections[section] ?? []).flatMap(block => block.body);
}

function assertRequirementPairs(spec, required = expectedIds) {
  for (const section of ['Must', 'Done when']) {
    const ids = entries(spec, section)
      .map(line => line.match(/^([A-Z]+-\d{2}):\s+\S/))
      .filter(Boolean).map(match => match[1]);
    assert.deepEqual(ids.sort(), [...required].sort(), `${section}: missing or duplicate requirement`);
  }
}

test('the pinned official CLI validates all pilot specs', () => {
  const installed = JSON.parse(readFileSync(join(tooling, 'node_modules/specdd/package.json')));
  const manifest = JSON.parse(readFileSync(join(tooling, 'package.json')));
  assert.equal(installed.version, manifest.dependencies.specdd);
  for (const unused of ['jest', '@babel/core', 'baseline-browser-mapping', 'brace-expansion', 'browserslist', 'js-yaml']) {
    for (const modules of ['node_modules', 'node_modules/specdd/node_modules']) {
      assert.equal(existsSync(join(tooling, modules, unused)), false, `Unexpected upstream dev dependency: ${unused}`);
    }
  }
  const result = run('lint', 'specs');
  assert.equal(result.status, 0, result.stdout + result.stderr);
});

test('resolution discovers SPEC-001 and its 14 requirement/proof pairs', () => {
  assertRequirementPairs(resolvedContract());
});
test('resolution discovers SPEC-002 and its seven requirement/proof pairs', () => {
  assertRequirementPairs(resolvedContract('specs/002-chat-and-retrieval'), ['CHAT-01','CHAT-02','RET-01','RAG-01','NET-01','SEM-01','UI-02']);
});

test('explicit pilot references exist and stay inside the repository', () => {
  const spec = resolvedContract();
  const paths = [...entries(spec, 'References'), ...entries(spec, 'Can read')];
  assert.ok(paths.length > 0);
  for (const path of paths) {
    assert.match(path, /^(\.\/|\.\.\/|\/)/, `Expected an explicit path: ${path}`);
    const target = realpathSync(path.startsWith('/')
      ? resolve(root, `.${path}`) : resolve(root, subject, path));
    const local = relative(realpathSync(root), target);
    assert.ok(local !== '..' && !local.startsWith('../') && !isAbsolute(local), path);
  }
});

test('a malformed spec makes the official validator fail', () => {
  const temporary = mkdtempSync(join(tmpdir(), 'chatpurp-specdd-'));
  try {
    const badSpec = join(temporary, 'invalid.sdd');
    writeFileSync(badSpec, 'Spec: Invalid fixture\n\nMust:\n   Invalid three-space indentation\n');
    const result = run('lint', badSpec);
    assert.equal(result.status, 1, result.stdout + result.stderr);
    assert.match(result.stdout + result.stderr, /indent/i);
  } finally {
    rmSync(temporary, { recursive: true, force: true });
  }
});

test('the traceability check rejects a missing proof and a duplicate requirement', () => {
  const spec = resolvedContract();
  const missing = structuredClone(spec);
  missing.sections['Done when'][0].body = entries(spec, 'Done when')
    .filter(line => !line.startsWith('ACL-02:'));
  assert.throws(() => assertRequirementPairs(missing), /missing or duplicate/);
  const duplicate = structuredClone(spec);
  duplicate.sections.Must[0].body.push(entries(spec, 'Must')[0]);
  assert.throws(() => assertRequirementPairs(duplicate), /missing or duplicate/);
  const missingHttp = structuredClone(spec);
  missingHttp.sections['Done when'][0].body = entries(spec, 'Done when')
    .filter(line => !line.startsWith('HTTP-01:'));
  assert.throws(() => assertRequirementPairs(missingHttp), /missing or duplicate/);
  const unexpected = structuredClone(spec);
  unexpected.sections.Must[0].body.push('NEW-01: unexpected requirement');
  assert.throws(() => assertRequirementPairs(unexpected), /missing or duplicate/);
});

const auditFixture = {
  auditReportVersion: 2,
  metadata: { vulnerabilities: { high: 1 } },
  vulnerabilities: { example: { severity: 'high', nodes: ['node_modules/specdd/node_modules/example'] } },
};

test('an audit finding is excluded only for an absent upstream dev dependency', () => {
  assert.equal(classifyAudit(auditFixture, () => false, () => true).excluded.length, 1);
  assert.equal(classifyAudit(auditFixture, () => false, () => false).blocking.length, 1);
});

test('an installed vulnerable dependency blocks even if marked dev upstream', () => {
  const summary = classifyAudit(auditFixture, () => true, () => true);
  assert.equal(summary.blocking.length, 1);
  assert.equal(summary.excluded.length, 0);
});

test('unavailable audit data and invalid package paths fail closed', () => {
  assert.throws(() => classifyAudit({ error: 'offline' }, () => false, () => true));
  const bad = structuredClone(auditFixture);
  bad.vulnerabilities.example.nodes = ['node_modules/../../outside'];
  assert.throws(() => classifyAudit(bad, () => false, () => true), /Invalid dependency path/);
});

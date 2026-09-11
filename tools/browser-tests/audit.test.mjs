import test from 'node:test';
import assert from 'node:assert/strict';
import { checkAuditResults, auditQueries } from './audit.mjs';
const clean = () => ({ auditReportVersion: 2, vulnerabilities: {}, metadata: { vulnerabilities: { total: 0 } } });
test('complete clean audits pass; empty inventory does not', () => {
  checkAuditResults(clean(), { results: [{}] }, 1);
  assert.throws(() => checkAuditResults(clean(), { results: [] }, 0));
});
test('npm finding or unavailable audit fails closed', () => {
  for (const npm of [{}, { error: 'offline' }, { ...clean(), vulnerabilities: { foo: {} } },
    { ...clean(), metadata: { vulnerabilities: { total: 1 } } }]) {
    assert.throws(() => checkAuditResults(npm, { results: [{}] }, 1));
  }
});
test('OSV finding, pagination, malformed or incomplete result fails closed', () => {
  for (const osv of [{}, { results: [] }, { error: 'offline', results: [{}] },
    { results: [null] }, { results: [{ next_page_token: 'more' }] }, { results: [{ error: 'error' }] },
    { results: [{ vulns: [{ id: 'TEST' }] }] }, { results: [{ vulns: 'bad' }] }]) {
    assert.throws(() => checkAuditResults(clean(), osv, 1));
  }
});
test('bundled dependencies join the graph; only equal name/version pairs deduplicate', () => {
  const queries = auditQueries({ packages: { '': {}, 'node_modules/foo': { version: '1.0.0' },
    'node_modules/bar/node_modules/foo': { version: '1.0.0' } } }, [
    { name: 'foo', version: '1.0.0' }, { name: 'foo', version: '2.0.0' }, { name: 'embedded', version: '3.0.0' },
  ]);
  assert.equal(queries.length, 3);
  assert.ok(queries.some(q => q.package.name === 'embedded'));
});

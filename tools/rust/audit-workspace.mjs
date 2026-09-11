// Resolve/inspect metadata and query advisories; never compile package code.
import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { mkdirSync, writeFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { join } from 'node:path';
const root = fileURLToPath(new URL('../../', import.meta.url));
const metadata = JSON.parse(execFileSync('sh', [join(root, 'tools/rust/run.sh'), 'cargo', 'metadata', '--locked', '--offline', '--format-version', '1'], { encoding:'utf8', maxBuffer:20_000_000, timeout:60_000 }));
const buildMetadata = JSON.parse(execFileSync('sh', [join(root, 'tools/rust/run.sh'), 'cargo', 'metadata', '--manifest-path',join(root,'tools/wasm-build/Cargo.toml'),'--locked','--offline','--format-version','1'], {encoding:'utf8',maxBuffer:20_000_000,timeout:60_000}));
const packages = [...new Map([...metadata.packages,...buildMetadata.packages].filter(p=>p.source).map(p=>[`${p.name}@${p.version}`,p])).values()];
assert.ok(packages.length > 0);
assert.ok(packages.every(p => p.source.startsWith('registry+') && p.license));
const queries = packages.map(p => ({ package:{ name:p.name, ecosystem:'crates.io' }, version:p.version }));
const response = JSON.parse(execFileSync('curl', ['-fsS','--max-time','45','https://api.osv.dev/v1/querybatch','-H','Content-Type: application/json','--data-binary','@-'], { input:JSON.stringify({queries}), encoding:'utf8', timeout:50_000, maxBuffer:8_000_000 }));
const output = join(root,'.local/rust/workspace-review');
mkdirSync(output,{recursive:true});
writeFileSync(join(output,'metadata.json'),JSON.stringify(metadata));
writeFileSync(join(output,'osv-query.json'),JSON.stringify({queries}));
writeFileSync(join(output,'osv.json'),JSON.stringify(response));
assert.ok(Array.isArray(response.results), 'Invalid OSV result list');
assert.equal(response.results.length,queries.length);
const findings = response.results.flatMap((r,i) => {
  assert.ok(r && typeof r==='object' && !Array.isArray(r) && !r.error && !r.next_page_token,'Incomplete OSV audit');
  if ('vulns' in r) assert.ok(Array.isArray(r.vulns),'Invalid OSV vulnerability list');
  return (r.vulns ?? []).map(v => ({name:packages[i].name,version:packages[i].version,id:v.id}));
});
console.log(JSON.stringify({packages:packages.length,findings},null,2));
assert.deepEqual(findings,[], 'Workspace dependencies have advisories');

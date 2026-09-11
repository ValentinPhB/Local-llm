import test from 'node:test';
import assert from 'node:assert/strict';
import {execFileSync} from 'node:child_process';
import {readFileSync,readdirSync,existsSync,statSync} from 'node:fs';
import {fileURLToPath} from 'node:url';
import {join,resolve,dirname} from 'node:path';
const root=fileURLToPath(new URL('../',import.meta.url));
function files(dir){return readdirSync(dir,{withFileTypes:true}).flatMap(e=>e.isDirectory()?files(join(dir,e.name)):[join(dir,e.name)]);}
test('domain and browser dependency boundaries are enforced by the resolved Cargo graph',()=>{
  const data=JSON.parse(execFileSync('sh',['tools/rust/run.sh','cargo','metadata','--locked','--offline','--filter-platform','wasm32-unknown-unknown','--format-version','1'],{cwd:root,encoding:'utf8',maxBuffer:20000000}));
  const byName=name=>data.packages.find(p=>p.name===name);
  // cargo metadata unifies the whole workspace, including native members.
  // Select the actual package/build target for the browser reachability proof.
  const tree=execFileSync('sh',['tools/rust/run.sh','cargo','tree','-p','chatpurp-web','--target','wasm32-unknown-unknown','--locked','--offline','--edges','normal,build','--prefix','none','--format','{p}'],{cwd:root,encoding:'utf8'});
  const reached=new Set(tree.split('\n').map(l=>l.split(' ')[0]));
  for(const name of ['chatpurp-api','chatpurp-core','jsonwebtoken','aws-lc-rs','rustix','hyper','axum'])assert.ok(!reached.has(name),name+' reached from frontend');
  const core=byName('chatpurp-core');assert.ok(core.dependencies.every(d=>['serde','serde_json'].includes(d.name)));
  for(const p of data.packages.filter(p=>!p.source))assert.equal(p.publish?.length,0,'Lab packages must not be published to crates.io');
});
test('all documented local links resolve and the root README remains intentionally absent',()=>{
  assert.equal(existsSync(join(root,'README.md')),false);
  const candidates=['AGENTS.md','STATUS.example.md',...files(join(root,'docs')),...files(join(root,'specs')),'tools/specdd/README.md','tools/browser-tests/README.md'].map(p=>resolve(root,p)).filter(p=>p.endsWith('.md'));
  for(const source of candidates)for(const match of readFileSync(source,'utf8').matchAll(/(?<!!)\[[^\]]+\]\(([^)#]+)(?:#[^)]*)?\)/g)){
    const target=match[1];if(target.includes('://')||target.startsWith('mailto:'))continue;
    assert.ok(existsSync(resolve(dirname(source),target)),`${source}: missing ${target}`);
  }
});
test('runtime scripts parse, local data stays ignored and configuration is valid JSON',()=>{
  for(const script of ['tools/check.sh','tools/rust/run.sh','tools/rust/install.sh','tools/rust/build-app.sh','tools/browser-tests/run.sh'])execFileSync('sh',['-n',script],{cwd:root});
  for(const path of files(join(root,'config')).filter(p=>p.endsWith('.json')))JSON.parse(readFileSync(path));
  for(const path of ['STATUS.md','.local/example','tools/browser-tests/node_modules/example'])assert.equal(execFileSync('git',['check-ignore',path],{cwd:root,encoding:'utf8'}).trim(),path);
});
test('fifteen actual documents match the policy and the 60/20/20 split',()=>{
  const policy=JSON.parse(readFileSync(join(root,'config/access-control/demo-policy.json')));
  assert.equal(policy.resources.length,15);
  for(const [category,count] of [['PUBLIC',9],['RH',3],['IT',3]])assert.equal(policy.resources.filter(r=>r.classification===category).length,count);
  assert.deepEqual(files(join(root,'demo-documents')).filter(p=>p.endsWith('.md')).sort(),policy.resources.map(r=>join(root,r.path)).sort());
});
test('publishable files contain neither archives nor private runtime artifacts',()=>{
  const listed=execFileSync('git',['ls-files','--cached','--others','--exclude-standard'],{cwd:root,encoding:'utf8'}).trim().split('\n');
  for(const path of listed.filter(p=>existsSync(join(root,p)))){
    assert.doesNotMatch(path,/(^|\/)(history|archives?|node_modules|target|\.local|secrets|credentials)(\/|$)/,path);
    assert.notEqual(path,'STATUS.md');
    assert.ok(!/\.pem$|\.key$/.test(path),path);
    if(path.split('/').at(-1).startsWith('.env'))assert.equal(path.split('/').at(-1),'.env.example');
  }
});

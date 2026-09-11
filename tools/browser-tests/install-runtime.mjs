// Bootstrap de développement : archives officielles verrouillées avant extraction.
import assert from 'node:assert/strict';
import {execFileSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {existsSync,mkdirSync,mkdtempSync,readFileSync,renameSync,rmSync} from 'node:fs';
import {fileURLToPath} from 'node:url';
import {join} from 'node:path';
const root=fileURLToPath(new URL('../../',import.meta.url));
const lock=JSON.parse(readFileSync(new URL('runtime-lock.json',import.meta.url)));
assert.equal(process.platform,lock.platform);assert.equal(process.arch,lock.arch);
const runtime=join(root,'.local/browser-runtime');mkdirSync(runtime,{recursive:true,mode:0o700});
const sha=p=>createHash('sha256').update(readFileSync(p)).digest('hex');
function install(info,destination,executable,kind,inner) {
  if(existsSync(executable)){assert.equal(sha(executable),info.executableSha256);return;}
  assert.equal(existsSync(destination),false,'Incomplete installation: inspect before replacement');
  const temp=mkdtempSync(join(runtime,'install-'));
  try {
    const archive=join(temp,'archive');execFileSync('curl',['--proto','=https','--tlsv1.2','-fL','--max-time','180',info.archiveUrl,'-o',archive],{stdio:'inherit',timeout:190000});
    assert.equal(sha(archive),info.archiveSha256,'Downloaded archive changed');
    const output=join(temp,'unpacked');mkdirSync(output,{mode:0o700});
    execFileSync(kind==='tar'?'tar':'unzip',kind==='tar'?['-xzf',archive,'-C',output]:['-q',archive,'-d',output],{stdio:'inherit'});
    const extracted=inner?join(output,inner):output;
    const relative=executable.slice(destination.length+1);assert.equal(sha(join(extracted,relative)),info.executableSha256);
    renameSync(extracted,destination);
  }finally{rmSync(temp,{recursive:true,force:true});} // Exact mkdtemp owned by this run.
}
const nodeDir=join(runtime,`node-v${lock.node.version}-darwin-arm64`);
install(lock.node,nodeDir,join(nodeDir,'bin/node'),'tar',`node-v${lock.node.version}-darwin-arm64`);
const browserDir=join(runtime,lock.browser.executableRelativePath.split('/')[0]);
install(lock.browser,browserDir,join(runtime,lock.browser.executableRelativePath),'zip');
console.log('Node et Chromium verrouillés disponibles dans .local/browser-runtime.');

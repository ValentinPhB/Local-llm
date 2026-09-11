import test from 'node:test';
import assert from 'node:assert/strict';
import {spawn} from 'node:child_process';
import {once} from 'node:events';
import {createInterface} from 'node:readline';
import {fileURLToPath} from 'node:url';
import {resolve} from 'node:path';
import {withQualifiedBrowser} from '../qualified-browser.mjs';

const repo=fileURLToPath(new URL('../../../',import.meta.url));
async function fixture(action) {
  assert.ok(process.env.CHATPURP_WEB_ASSETS,'Build the actual Rust frontend first');
  const child=spawn(resolve(repo,'.local/rust/target/aarch64-apple-darwin/debug/browser-fixture'),[process.env.CHATPURP_WEB_ASSETS],{stdio:['pipe','pipe','pipe']});
  const exited=once(child,'exit');const lines=createInterface({input:child.stdout});let stderr='';child.stderr.on('data',b=>stderr=(stderr+b).slice(-8000));
  try {
    const announcement=await Promise.race([once(lines,'line',{signal:AbortSignal.timeout(10000)}).then(([l])=>JSON.parse(l)),exited.then(()=>{throw new Error(stderr);})]);
    await action(announcement);
  } finally {
    lines.close();child.stdin.end();const timer=setTimeout(()=>child.kill('SIGKILL'),5000);
    try {const [code,signal]=await exited;assert.equal(code,0,stderr);assert.equal(signal,null,stderr);}finally{clearTimeout(timer);}
  }
}
async function replace(page,selector,value) {await page.locator(selector).fill(value);}
test('actual Rust UI: sessions, permissions, retrieval, chat, inert HTML and logout',{timeout:60000},async()=>{
  await fixture(async({origin,paths})=>{
    const ids=['public-welcome','rh-onboarding','it-workstation'];
    const allowedUrls=[...paths,'/api/session',...ids.map(id=>'/api/documents/'+id)].map(p=>origin+p);
    const allowedPostUrls=['demo-session','logout','retrieve','chat','rag-chat'].map(p=>origin+'/api/'+p);
    await withQualifiedBrowser(async({page,blocked})=>{
      const errors=[];page.on('pageerror',e=>errors.push(e.message));
      await page.goto(origin+'/');await page.waitForSelector('#login');await page.waitForFunction(()=>!document.querySelector('#login').disabled);
      for(const id of ['alice','bob','charlie','oscar']) {
        await page.select('#identity-select',id);await page.click('#login');
        await page.waitForFunction(id=>document.querySelector('#identity').textContent.toLowerCase().includes(id),{},id);
        await replace(page,'#resource','public-welcome');await page.click('#read');await page.waitForFunction(()=>document.querySelector('#document').textContent.includes('id: public-welcome'));
        for(const [resource,allow] of [['rh-onboarding',id==='alice'],['it-workstation',id==='bob']]) {
          await replace(page,'#resource',resource);await page.click('#read');
          if(allow) await page.waitForFunction(r=>document.querySelector('#document').textContent.includes('id: '+r),{},resource);
          else {await page.waitForFunction(()=>document.querySelector('#feedback').textContent.includes('refusé'));assert.equal(await page.$eval('#document',n=>n.textContent),'');}
        }
      }
      await replace(page,'#query','bienvenue');await page.click('#retrieve');await page.waitForFunction(()=>document.querySelector('#results').textContent.length>0);
      assert.match(await page.$eval('#results',n=>n.textContent),/public-welcome/);
      await replace(page,'#message','bienvenue');
      for(const selector of ['#chat','#rag']) {await page.click(selector);await page.waitForFunction(()=>document.querySelector('#answer').textContent.includes('Réponse fictive'));assert.equal(await page.evaluate(()=>window.bad),undefined);assert.equal(await page.$('#answer script'),null);}
      assert.match(await page.$eval('#answer',n=>n.textContent),/Sources : public-welcome/);
      const cookies=await page.cookies();const session=cookies.find(c=>c.name==='chatpurp_demo_session');assert.equal(session.httpOnly,true);assert.equal(session.sameSite,'Strict');assert.ok(!await page.evaluate(()=>document.cookie.includes('chatpurp_demo_session')));
      await page.click('#logout');await page.waitForFunction(()=>document.querySelector('#identity').textContent==='Aucune session');
      assert.equal(await page.$eval('#document',n=>n.textContent),'');assert.equal(await page.$eval('#answer',n=>n.textContent),'');
      assert.deepEqual(errors,[]);assert.deepEqual(blocked,[]);
    },{allowedUrls,allowedPostUrls});
  });
});
test('missing actual application WASM is a visible failure, not a false UI success',{timeout:20000},async()=>{
  await fixture(async({origin,paths})=>{
    await withQualifiedBrowser(async({page,blocked})=>{
      await page.goto(origin+'/');await page.waitForSelector('#load-status[data-state="error"]');
      assert.equal(await page.$('#login'),null);
      assert.deepEqual(blocked,[origin+'/chatpurp_web_bg.wasm']);
    },{allowedUrls:paths.filter(p=>!p.endsWith('.wasm')).map(p=>origin+p)});
  });
});

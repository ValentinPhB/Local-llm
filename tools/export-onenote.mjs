// Rapport HTML autonome et PNG : régénérés depuis les guides courants, hors Git.
import assert from 'node:assert/strict';
import {mkdirSync,readFileSync,writeFileSync} from 'node:fs';
import {createHash} from 'node:crypto';
import {execFileSync} from 'node:child_process';
import {join} from 'node:path';
import {fileURLToPath} from 'node:url';
import {withQualifiedBrowser} from './browser-tests/qualified-browser.mjs';
const root=fileURLToPath(new URL('../',import.meta.url));
const output=join(root,'.local/reports/chatpurp');mkdirSync(output,{recursive:true,mode:0o700});
const escape=s=>s.replaceAll('&','&amp;').replaceAll('<','&lt;').replaceAll('>','&gt;').replaceAll('"','&quot;');
const box=(x,y,w,h,title,lines,color='#e8f0fa')=>`<rect x="${x}" y="${y}" width="${w}" height="${h}" rx="12" fill="${color}" stroke="#60758c"/><text x="${x+18}" y="${y+32}" font-weight="bold" font-size="22">${escape(title)}</text>${lines.map((line,i)=>`<text x="${x+18}" y="${y+62+i*25}" font-size="17">${escape(line)}</text>`).join('')}`;
const arrow=(x1,y1,x2,y2,dashed=false)=>`<path d="M ${x1} ${y1} L ${x2} ${y2}" fill="none" stroke="#315a89" stroke-width="3" ${dashed?'stroke-dasharray="8 6"':''} marker-end="url(#arrow)"/>`;
const svg=(title,content)=>`<svg xmlns="http://www.w3.org/2000/svg" width="1200" height="660" viewBox="0 0 1200 660"><defs><marker id="arrow" markerWidth="9" markerHeight="9" refX="8" refY="3" orient="auto"><path d="M0,0 L0,6 L9,3 z" fill="#315a89"/></marker></defs><rect width="1200" height="660" fill="white"/><g font-family="Arial,sans-serif" fill="#182c43"><text x="40" y="50" font-size="30" font-weight="bold">${escape(title)}</text>${content}</g></svg>`;
const diagrams={
  architecture:svg('ChatPurp — composants et frontières',
    box(40,100,310,125,'Navigateur / Dioxus',['Rust → WebAssembly','UI seule ; aucune décision ACL'])+
    box(430,100,340,190,'API Rust / Axum + Hyper',['127.0.0.1:3211','Session → ACL → audit','Lecteur / recherche / chat','Seule autorité applicative'])+
    box(850,100,305,125,'Ollama local',['127.0.0.1:11434','qwen3:4b : génération'])+
    box(40,365,310,125,'Documents fictifs',['15 fichiers : 9 PUBLIC / 3 RH / 3 IT','Seulement après autorisation'])+
    box(430,365,340,125,'Audit minimal',['.local/rust-api/audit','1 MiB + une sauvegarde'])+
    box(850,365,305,125,'Qdrant préparé',['127.0.0.1:6333','Hors routes actives'],'#fff2d7')+
    arrow(350,165,425,165)+arrow(770,165,845,165)+arrow(475,292,290,360)+arrow(595,292,595,360)+arrow(745,292,915,360,true)+
    '<text x="40" y="555" font-size="20">Indexeur Rust distinct : plan hors réseau ou remplacement administratif explicite.</text><text x="40" y="590" font-size="20">Aucun MCP. Le modèle ne reçoit ni accès au disque, ni outil, ni clé de signature.</text><text x="40" y="625" font-size="17" fill="#755600">Pointillés : capacité sémantique préparée, pas une route actuellement activée.</text>'),
  flux:svg('Lecture documentaire — ordre de sécurité',
    box(40,110,245,110,'1. Admission HTTP',['Host / Origin / limites'])+
    box(330,110,245,110,'2. Session signée',['JWT et annuaire serveur'])+
    box(620,110,245,110,'3. Identifiant',['Pas de chemin client'])+
    box(910,110,245,110,'4. ACL',['Groupes → rôles → droit'])+
    box(910,345,245,110,'5. Audit réussi',['Avant toute lecture'])+
    box(620,345,245,110,'6. Lecture bornée',['openat / fichier / metadata'])+
    box(330,345,245,110,'7. JSON public',['Pas de chemin système'])+
    box(40,345,245,110,'8. Texte dans l’UI',['HTML non interprété'])+
    arrow(285,165,325,165)+arrow(575,165,615,165)+arrow(865,165,905,165)+arrow(1030,220,1030,338)+arrow(910,400,870,400)+arrow(620,400,580,400)+arrow(330,400,290,400)+
    '<text x="40" y="550" font-size="22" fill="#9a3535">Tout refus arrête le flux ; aucun document partiel n’est retourné.</text><text x="40" y="590" font-size="20">Oscar : public-welcome autorisé ; tout autre document refusé.</text><text x="40" y="628" font-size="18">Le RAG réutilise cette frontière : seuls les passages autorisés vont au modèle.</text>'),
};
await withQualifiedBrowser(async({page,blocked})=>{
  await page.setViewport({width:1200,height:660,deviceScaleFactor:1});
  for(const [name,content] of Object.entries(diagrams)){
    await page.setContent(`<html><head><meta http-equiv="Content-Security-Policy" content="default-src 'none'; style-src 'unsafe-inline'"></head><body style="margin:0">${content}</body></html>`);
    await page.screenshot({path:join(output,`${name}.png`),type:'png'});
  }
  assert.deepEqual(blocked,[]);
});
function inline(text){return escape(text).replace(/\[([^\]]+)\]\(([^)]+)\)/g,(_,label,target)=>target.startsWith('https://')?`<a href="${target}">${label}</a>`:`${label} <small>(${target})</small>`).replace(/`([^`]+)`/g,'<code>$1</code>');}
function markdown(text){
  let html='',code=false,table=false;
  for(const line of text.split('\n')){
    if(/^~~~|^```/.test(line)){if(table){html+='</table>';table=false;}html+=code?'</pre>':'<pre>';code=!code;continue;}
    if(code){html+=escape(line)+'\n';continue;}
    if(line.startsWith('|')){if(!table){html+='<table>';table=true;}const cells=line.split('|').slice(1,-1);if(cells.every(c=>/^\s*:?-+:?\s*$/.test(c)))continue;html+='<tr>'+cells.map(c=>'<td>'+inline(c.trim())+'</td>').join('')+'</tr>';continue;}
    if(table){html+='</table>';table=false;}
    const heading=line.match(/^(#{1,3}) (.+)$/);if(heading){const n=heading[1].length;html+=`<h${n}>${inline(heading[2])}</h${n}>`;}
    else if(line.trim()){html+=`<p>${inline(line)}</p>`;}
  }
  if(table)html+='</table>';if(code)html+='</pre>';return html;
}
const paths=['system-architecture.md','local-api-architecture.md','api-request-flow.md','security-requirements.md','demo-document-catalog.md','semantic-rag-design.md','release-management.md','security-test-strategy.md'];
const sources=paths.map(p=>({path:p,text:readFileSync(join(root,'docs',p),'utf8')}));
const fingerprint=createHash('sha256').update(sources.map(s=>s.path+'\n'+s.text).join('\n')).digest('hex');
const commit=execFileSync('git',['rev-parse','--short','HEAD'],{cwd:root,encoding:'utf8'}).trim();
const dirty=execFileSync('git',['status','--porcelain'],{cwd:root,encoding:'utf8'}).trim().length>0;
const images=Object.keys(diagrams).map(name=>`<p><img width="1100" alt="${name}" src="data:image/png;base64,${readFileSync(join(output,`${name}.png`)).toString('base64')}"></p>`).join('');
const report=`<!doctype html><html lang="fr"><head><meta charset="utf-8"><meta http-equiv="Content-Security-Policy" content="default-src 'none'; img-src data:; style-src 'unsafe-inline'"><title>ChatPurp — rapport OneNote</title><style>body{font:12pt Calibri,Arial,sans-serif;color:#182c43;max-width:1150px;margin:32px auto}table{border-collapse:collapse;width:100%}td{border:1px solid #aab9c9;padding:8px;vertical-align:top}h1{margin-top:32px}h2{margin-top:24px}pre{white-space:pre-wrap;background:#f0f4f8;padding:14px}p{margin:6px 0;line-height:1.45}small{color:#5e6f82}img{max-width:100%;height:auto}</style></head><body><h1>ChatPurp — rapport d’architecture et d’exploitation</h1><p>Export du ${new Date().toISOString().slice(0,10)}. Base Git : ${commit}${dirty?' ; modifications locales non publiées':''}.</p><p>Rapport généré depuis les guides courants, pas une source documentaire parallèle. La bascule, les résultats CI et les capacités préparées sont distingués dans le texte.</p><p>Pour OneNote : ouvrir ce HTML, copier son contenu puis le coller ; les PNG peuvent aussi être insérés séparément. Aucun transfert vers un compte n’a été effectué.</p>${images}${sources.map(s=>markdown(s.text)).join('<hr>')}<p><small>Empreinte SHA-256 des guides : ${fingerprint}</small></p></body></html>`;
writeFileSync(join(output,'rapport.html'),report,{mode:0o600});
console.log('Rapport autonome et deux schémas PNG : .local/reports/chatpurp/');

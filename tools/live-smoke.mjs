// Vérification locale explicite : utilise Ollama, jamais exécutée par la CI.
// Aucun secret ni contenu de réponse affiché ou enregistré.
import assert from 'node:assert/strict';
const origin = 'http://127.0.0.1:3211';
let cookie = '';
async function request(path, body, expected = 200) {
  const response = await fetch(origin + path, {
    method: body === undefined && path !== '/api/logout' ? 'GET' : 'POST',
    headers: { Origin: origin, ...(cookie ? { Cookie: cookie } : {}),
      ...(body === undefined ? {} : { 'Content-Type': 'application/json' }) },
    body: body === undefined ? undefined : JSON.stringify(body),
    redirect: 'error', signal: AbortSignal.timeout(125_000),
  });
  assert.equal(response.status, expected, path);
  return response;
}
assert.equal((await (await request('/healthz')).json()).status, 'ok');
await request('/api/session', undefined, 401);
const session = await request('/api/demo-session', { identity_id: 'oscar' });
assert.match(session.headers.get('set-cookie'), /HttpOnly; SameSite=Strict/);
cookie = session.headers.get('set-cookie').split(';')[0];
assert.equal((await session.json()).identity.id, 'oscar');
assert.equal((await (await request('/api/session')).json()).identity.id, 'oscar');
assert.equal((await (await request('/api/documents/public-welcome')).json()).resource_id, 'public-welcome');
for (const id of ['public-glossary', 'rh-onboarding', 'it-workstation']) {
  await request(`/api/documents/${id}`, undefined, 403);
}
const retrieval = await (await request('/api/retrieve', { query: 'bienvenue' })).json();
assert.ok(retrieval.results.length > 0);
assert.ok(retrieval.results.every(r => r.resource_id === 'public-welcome'));
console.log('Santé, session, lecture, refus et récupération : OK');
for (const path of ['/api/chat', '/api/rag-chat']) {
  const result = await (await request(path, { message: 'Bienvenue : présente ce laboratoire fictif en une seule phrase courte.' })).json();
  assert.equal(typeof result.content, 'string');
  assert.ok(result.content.trim().length > 0);
  assert.ok(!/<\/?think>/i.test(result.content));
  if (path === '/api/rag-chat') {
    assert.ok(result.sources.length > 0);
    assert.ok(result.sources.every(id => id === 'public-welcome'));
  }
  console.log(`${path} → Ollama réel : réponse non vide, contrat OK`);
}
await request('/api/logout');
cookie = '';
await request('/api/session', undefined, 401);
console.log('Déconnexion : OK. Ce test ne mesure pas la qualité des réponses.');

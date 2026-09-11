import test from 'node:test';
import assert from 'node:assert/strict';
import { existsSync } from 'node:fs';
import { withQualifiedBrowser } from '../qualified-browser.mjs';

test('real dedicated browser: version, sandbox arguments, click, denied request and cleanup', { timeout: 35_000 }, async t => {
  let profile, pid;
  await withQualifiedBrowser(async info => {
    ({ profile, pid } = info);
    const { page, blocked } = info;
    assert.deepEqual(await info.context.cookies(), []);
    await page.setContent('<!doctype html><meta charset="utf-8"><title>ChatPurp fixture</title><button id="increment">Compteur : 0</button><script>document.querySelector("button").onclick = e => e.target.textContent = "Compteur : 1";</script>');
    assert.equal(await page.title(), 'ChatPurp fixture');
    await page.locator('#increment').click();
    await page.waitForFunction(() => document.querySelector('#increment').textContent === 'Compteur : 1');
    const result = await page.evaluate(async () => {
      try { await fetch('https://chatpurp-test.invalid/blocked'); return 'unexpected'; }
      catch { return 'blocked'; }
    });
    assert.equal(result, 'blocked');
    assert.deepEqual(blocked, ['https://chatpurp-test.invalid/blocked']);
    t.diagnostic(`Browser ${info.version}; pipe; no sandbox-disabling arguments; synthetic click OK`);
  });
  assert.equal(existsSync(profile), false);
  assert.throws(() => process.kill(pid, 0), { code: 'ESRCH' });
});

test('intentional scenario failure still closes browser and removes profile', { timeout: 35_000 }, async () => {
  let profile, pid;
  await assert.rejects(withQualifiedBrowser(async info => {
    ({ profile, pid } = info);
    throw new Error('SYNTHETIC_SCENARIO_FAILURE');
  }), /SYNTHETIC_SCENARIO_FAILURE/);
  assert.equal(existsSync(profile), false);
  assert.throws(() => process.kill(pid, 0), { code: 'ESRCH' });
});

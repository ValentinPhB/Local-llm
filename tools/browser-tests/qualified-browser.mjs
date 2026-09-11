import assert from 'node:assert/strict';
import { mkdirSync, mkdtempSync, readFileSync, rmSync, statSync } from 'node:fs';
import { join, resolve } from 'node:path';
import { checkedLaunchOptions, checkedBrowserArguments, checkedFixtureUrls, runtimeLock, sha256, defaultRuntimeRoot } from './launch-policy.mjs';
import { checkBundledFiles } from './audit.mjs';

// Only used with fixed synthetic pages. Interception is not an OS firewall.
export async function withQualifiedBrowser(action, { allowedUrls = [], allowedPostUrls = [] } = {}) {
  checkedFixtureUrls([...allowedUrls, ...allowedPostUrls]); // One ephemeral origin, including POST.
  const allowed = checkedFixtureUrls(allowedUrls);
  const posts = checkedFixtureUrls(allowedPostUrls);
  for (const url of posts) assert.match(new URL(url).pathname, /^\/api\/(demo-session|logout|retrieve|chat|rag-chat)$/);
  const options = checkedLaunchOptions();
  assert.equal(sha256(readFileSync(process.execPath)), runtimeLock.node.executableSha256, 'Node binary changed');
  checkBundledFiles(); // Before importing any Puppeteer code.
  const { default: puppeteer } = await import('puppeteer-core');
  // defaultArgs mutates feature arguments: use a copy, then freeze the effective list.
  const args = (await puppeteer.defaultArgs(structuredClone(options)))
    .filter(arg => !options.ignoreDefaultArgs.includes(arg));
  args.push('--remote-debugging-pipe');
  checkedBrowserArguments(args, { requirePipe: true });
  const profiles = join(defaultRuntimeRoot, 'test-profiles');
  mkdirSync(profiles, { recursive: true, mode: 0o700 });
  const profile = mkdtempSync(join(profiles, 'qualification-'));
  assert.equal(statSync(profile).mode & 0o777, 0o700);
  args.push(`--user-data-dir=${profile}`);
  let browser, context, child;
  let forced = false;
  const abort = new AbortController();
  const deadline = setTimeout(() => abort.abort(new Error('Browser qualification exceeded 30s')), 30_000);
  try {
    browser = await puppeteer.launch({ ...options, ignoreDefaultArgs: true, args,
      signal: abort.signal, env: { PATH: '/usr/bin:/bin:/usr/sbin:/sbin', TMPDIR: profile } });
    child = browser.process();
    assert.ok(child?.pid, 'Dedicated child process required');
    assert.equal(resolve(child.spawnfile), options.executablePath);
    checkedBrowserArguments(child.spawnargs.slice(1), { requirePipe: true });
    const version = await browser.version();
    assert.equal(version.split('/').at(-1), runtimeLock.browser.version, 'Unexpected actual browser version');
    const cdp = await browser.target().createCDPSession();
    const actual = await cdp.send('Browser.getBrowserCommandLine');
    checkedBrowserArguments(actual.arguments, { requirePipe: true });
    await cdp.detach();
    context = await browser.createBrowserContext({ downloadBehavior: { policy: 'deny' } });
    const page = await context.newPage();
    page.setDefaultTimeout(5_000);
    page.setDefaultNavigationTimeout(5_000);
    await page.setBypassServiceWorker(true);
    await page.setRequestInterception(true);
    const blocked = [], requested = [], interceptionErrors = [];
    page.on('request', request => {
      requested.push(request.url());
      if ((allowed.has(request.url()) && request.method() === 'GET') || (posts.has(request.url()) && request.method() === 'POST')) {
        request.continue().catch(error => interceptionErrors.push(error));
      } else {
        blocked.push(request.url());
        request.abort('blockedbyclient').catch(error => interceptionErrors.push(error));
      }
    });
    const value = await action({ page, browser, context, blocked, requested, version, profile, pid: child.pid,
      arguments: actual.arguments });
    assert.deepEqual(interceptionErrors, []);
    assert.ok(!abort.signal.aborted, 'Qualification deadline exceeded');
    return value;
  } finally {
    clearTimeout(deadline);
    // Kill only the child created above if graceful shutdown cannot finish.
    const killTimer = setTimeout(() => {
      if (child && child.exitCode === null && child.signalCode === null) {
        forced = true;
        child.kill('SIGKILL');
      }
    }, 5_000);
    try {
      try { if (context) await context.close(); }
      finally { if (browser) await browser.close(); }
    } finally {
      clearTimeout(killTimer);
      rmSync(profile, { recursive: true, force: true }); // Exact fresh mkdtemp output owned by this invocation.
    }
    assert.equal(forced, false, 'Browser required forced termination');
    if (child) assert.ok(child.exitCode !== null || child.signalCode !== null, 'Browser process still alive');
  }
}

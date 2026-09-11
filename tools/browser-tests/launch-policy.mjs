// No dependency import, download or process launch during prerequisite checks.
import { createHash } from 'node:crypto';
import { accessSync, constants, lstatSync, readFileSync, realpathSync } from 'node:fs';
import { dirname, isAbsolute, join, relative, resolve, sep } from 'node:path';
import { fileURLToPath } from 'node:url';

export const toolRoot = dirname(fileURLToPath(import.meta.url));
const lock = JSON.parse(readFileSync(join(toolRoot, 'runtime-lock.json'), 'utf8'));
Object.freeze(lock.node);
Object.freeze(lock.browser);
export const runtimeLock = Object.freeze(lock);
export const defaultRuntimeRoot = resolve(toolRoot, '../../.local/browser-runtime');
export const sha256 = data => createHash('sha256').update(data).digest('hex');

export function checkedEnvironment(env) {
  for (const key of Object.keys(env)) {
    if (/^(PUPPETEER_|NODE_OPTIONS$|NODE_PATH$|DYLD_|LD_PRELOAD$)/.test(key) && env[key]) {
      throw new Error(`Variable d'injection/refonte de lancement refusée : ${key}`);
    }
  }
}

export function checkedFixtureUrls(urls) {
  if (!Array.isArray(urls) || urls.length > 100) throw new Error('Liste URLs fictives invalide.');
  const origins = new Set();
  for (const raw of urls) {
    const url = new URL(raw);
    if (url.protocol !== 'http:' || url.hostname !== '127.0.0.1' || !url.port
      || ['3210', '11434', '6333'].includes(url.port) || url.username || url.password || url.search || url.hash
      || url.href !== raw) throw new Error('URL hors serveur fictif dédié.');
    origins.add(url.origin);
  }
  if (origins.size > 1) throw new Error('Une seule origine fictive autorisée.');
  return new Set(urls);
}

export function checkedBrowserArguments(args, { requirePipe = false } = {}) {
  const prohibited = /^(--no-sandbox|--disable-.*sandbox|--single-process|--no-zygote|--disable-web-security|--remote-debugging-port|--remote-debugging-address)(=|$)/;
  if (!Array.isArray(args) || args.some(a => typeof a !== 'string' || prohibited.test(a))) {
    throw new Error('Arguments navigateur interdits.');
  }
  if (requirePipe && !args.includes('--remote-debugging-pipe')) throw new Error('Pipe DevTools requis.');
}

// Overrides exist only for unit fixtures; the runner supplies none.
export function checkedLaunchOptions({
  platform = process.platform, arch = process.arch, nodeVersion = process.versions.node,
  runtimeRoot = defaultRuntimeRoot, packageRoot = toolRoot, environment = process.env,
  expectedBrowserHash = lock.browser.executableSha256,
} = {}) {
  checkedEnvironment(environment);
  if (platform !== lock.platform || arch !== lock.arch) throw new Error('Qualification limitée à macOS ARM64.');
  if (nodeVersion !== lock.node.version) throw new Error(`Node ${lock.node.version} requis ; aucune substitution automatique.`);
  const npmLock = JSON.parse(readFileSync(join(packageRoot, 'package-lock.json'), 'utf8'));
  if (npmLock.packages?.['node_modules/puppeteer-core']?.version !== lock.puppeteerVersion) throw new Error('Lockfile Puppeteer non conforme.');
  for (const [path, entry] of Object.entries(npmLock.packages)) {
    if (!path) continue;
    if (!path.startsWith('node_modules/') || path.split('/').includes('..')) throw new Error('Chemin de paquet invalide.');
    const manifest = JSON.parse(readFileSync(join(packageRoot, path, 'package.json'), 'utf8'));
    const expectedName = path.slice(path.lastIndexOf('node_modules/') + 13);
    if (manifest.name !== expectedName || manifest.version !== entry.version) throw new Error(`Version de paquet non conforme : ${path}`);
  }
  const root = realpathSync(runtimeRoot);
  const executablePath = resolve(root, lock.browser.executableRelativePath);
  const local = relative(root, executablePath);
  if (!local || isAbsolute(local) || local.split(sep).includes('..')) throw new Error('Chemin navigateur hors runtime dédié.');
  if (!lstatSync(executablePath).isFile() || realpathSync(executablePath) !== executablePath) throw new Error('Navigateur requis : fichier régulier sans lien.');
  accessSync(executablePath, constants.X_OK);
  if (sha256(readFileSync(executablePath)) !== expectedBrowserHash) throw new Error('Empreinte navigateur non conforme.');
  return {
    executablePath, browser: 'chrome', headless: 'shell', pipe: true, protocol: 'cdp',
    timeout: 10_000, protocolTimeout: 5_000,
    downloadBehavior: { policy: 'deny' },
    ignoreDefaultArgs: ['--disable-ipc-flooding-protection', '--disable-popup-blocking'],
    args: ['--enable-features=IsolateSandboxedIframes', '--disable-component-update'],
  };
}

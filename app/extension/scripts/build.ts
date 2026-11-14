import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { execFileSync } from 'node:child_process';
import { createRequire } from 'node:module';
import { fileURLToPath } from 'node:url';
import { build as esbuild } from 'esbuild';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const extensionRoot = path.resolve(__dirname, '..');
const srcDir = path.join(extensionRoot, 'src');
const manifestsDir = path.join(extensionRoot, 'manifests');
const distDir = path.join(extensionRoot, 'dist');

const BROWSER_TARGETS = Object.freeze({
  chrome: 'chrome116',
  firefox: 'firefox128'
} as const);

type Browser = keyof typeof BROWSER_TARGETS;

export const BROWSERS = Object.freeze(Object.keys(BROWSER_TARGETS) as Browser[]);

interface BuildOptions {
  browser?: Browser | null;
  quiet?: boolean;
}

interface BuildResult {
  distDir: string;
  outputs: Partial<Record<Browser, string>>;
  elapsed: number;
}

function copyRecursive(src: string, dest: string): void {
  const stat = fs.statSync(src);
  if (stat.isDirectory()) {
    fs.mkdirSync(dest, { recursive: true });
    for (const child of fs.readdirSync(src)) {
      copyRecursive(path.join(src, child), path.join(dest, child));
    }
    return;
  }
  fs.mkdirSync(path.dirname(dest), { recursive: true });
  fs.copyFileSync(src, dest);
}

function resolveTsc(): string {
  const require = createRequire(import.meta.url);
  try {
    return require.resolve('typescript/bin/tsc');
  } catch {
    throw new Error('TypeScript is not installed. Run `bun install` in app/extension before building.');
  }
}

function compileModules(outDir: string, quiet: boolean): void {
  const tsc = resolveTsc();
  const config = path.join(extensionRoot, 'tsconfig.build.json');
  execFileSync(process.execPath, [tsc, '-p', config, '--outDir', outDir], {
    cwd: extensionRoot,
    stdio: quiet ? 'pipe' : 'inherit'
  });
}

async function bundleContentScript(browser: Browser, destDir: string): Promise<void> {
  await esbuild({
    entryPoints: [path.join(srcDir, 'content', 'content.ts')],
    outfile: path.join(destDir, 'content.js'),
    bundle: true,
    format: 'iife',
    platform: 'browser',
    target: BROWSER_TARGETS[browser],
    charset: 'utf8',
    legalComments: 'none',
    treeShaking: true,
    allowOverwrite: true,
    logLevel: 'silent'
  });

  const contentPath = path.join(destDir, 'content.js');
  const bundled = fs.readFileSync(contentPath, 'utf8');
  fs.writeFileSync(contentPath, bundled.replace(/^\s*\/\/ [\w./-]*\.(?:ts|js)\s*\n/gm, ''), 'utf8');
}

function enterPath(label: string, relative: string): string {
  return `${label}/${relative}`;
}

const REQUIRED_FILES = Object.freeze([
  'manifest.json',
  'background.js',
  'content.js',
  'popup.html',
  'popup.js',
  'popup.css',
  'src/background/background.js',
  'src/shared/browser-api.js',
  'src/features/connection/connection.js',
  'src/features/connection/onboarding.js',
  'src/content/content.js',
  'src/popup/popup.js',
  'src/services/storage.js'
]);

export async function build(options: BuildOptions = {}): Promise<BuildResult> {
  const quiet = options.quiet === true;
  const only = options.browser ?? null;
  const selected: Browser[] = only ? [only] : [...BROWSERS];
  for (const browser of selected) {
    if (!BROWSERS.includes(browser)) {
      throw new Error(`Unknown browser target: ${browser}`);
    }
  }

  const started = Date.now();
  if (!quiet) console.log('Building Mensetsu browser extension from TypeScript sources...');

  if (fs.existsSync(distDir)) {
    fs.rmSync(distDir, { recursive: true, force: true });
  }
  fs.mkdirSync(distDir, { recursive: true });

  const stagingDir = fs.mkdtempSync(path.join(os.tmpdir(), 'mensetsu-extension-'));
  const outputs: Partial<Record<Browser, string>> = {};

  try {
    compileModules(stagingDir, quiet);

    for (const browser of selected) {
      const browserDist = path.join(distDir, browser);
      fs.mkdirSync(browserDist, { recursive: true });

      const manifestSource = path.join(manifestsDir, `${browser}.json`);
      if (!fs.existsSync(manifestSource)) {
        throw new Error(`Missing browser manifest: ${enterPath('manifests', `${browser}.json`)}`);
      }
      fs.copyFileSync(manifestSource, path.join(browserDist, 'manifest.json'));

      copyRecursive(stagingDir, path.join(browserDist, 'src'));

      fs.copyFileSync(path.join(srcDir, 'popup', 'popup.html'), path.join(browserDist, 'popup.html'));
      fs.copyFileSync(path.join(srcDir, 'popup', 'popup.css'), path.join(browserDist, 'popup.css'));

      fs.writeFileSync(
        path.join(browserDist, 'background.js'),
        `export * from './src/background/background.js';\nimport './src/background/background.js';\n`,
        'utf8'
      );

      fs.writeFileSync(
        path.join(browserDist, 'popup.js'),
        `export * from './src/popup/popup.js';\nimport './src/popup/popup.js';\n`,
        'utf8'
      );

      await bundleContentScript(browser, browserDist);

      for (const rel of REQUIRED_FILES) {
        const full = path.join(browserDist, rel);
        if (!fs.existsSync(full)) {
          throw new Error(`Build verification failed: missing ${enterPath(browser, rel)}`);
        }
      }

      outputs[browser] = browserDist;
    }
  } finally {
    fs.rmSync(stagingDir, { recursive: true, force: true });
  }

  const elapsed = Date.now() - started;
  if (!quiet) console.log(`Build complete in ${elapsed}ms -> ${distDir} (${selected.join(', ')})`);
  return { distDir, outputs, elapsed };
}

if (process.argv[1] === __filename) {
  const flag = process.argv.find((arg) => arg.startsWith('--browser='));
  const browser = (flag ? flag.slice('--browser='.length) : null) as Browser | null;
  build({ browser }).catch((error: unknown) => {
    console.error(error instanceof Error ? error.message : String(error));
    process.exitCode = 1;
  });
}

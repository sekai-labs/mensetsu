import { describe, it } from 'node:test';
import assert from 'node:assert/strict';
import crypto from 'node:crypto';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

import { BROWSERS, build } from '../scripts/build.ts';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const extensionRoot = path.resolve(__dirname, '..');
const distDir = path.join(extensionRoot, 'dist');

interface ManifestShape {
  manifest_version: number;
  background: { service_worker?: string; scripts?: string[]; type?: string };
  minimum_chrome_version?: string;
  permissions: string[];
  optional_host_permissions: string[];
  host_permissions?: string[];
  browser_specific_settings?: { gecko: { strict_min_version: string; id: string } };
}

function walk(root: string): string[] {
  const files: string[] = [];
  for (const entry of fs.readdirSync(root, { withFileTypes: true })) {
    const full = path.join(root, entry.name);
    if (entry.isDirectory()) files.push(...walk(full));
    else files.push(full);
  }
  return files;
}

function hashTree(root: string): Record<string, string> {
  const hashes: Record<string, string> = {};
  for (const full of walk(root).sort()) {
    const relative = path.relative(root, full).split(path.sep).join('/');
    hashes[relative] = crypto.createHash('sha256').update(fs.readFileSync(full)).digest('hex');
  }
  return hashes;
}

function readManifest(browser: string): ManifestShape {
  return JSON.parse(fs.readFileSync(path.join(distDir, browser, 'manifest.json'), 'utf8')) as ManifestShape;
}

describe('Deterministic Chrome and Firefox distribution builds', () => {
  it('emits separate installable directories with browser-appropriate manifests', async () => {
    await build({ quiet: true });

    assert.deepStrictEqual(
      fs.readdirSync(distDir).sort(),
      [...BROWSERS].sort(),
      'dist contains one directory per browser'
    );

    const chrome = readManifest('chrome');
    assert.strictEqual(chrome.manifest_version, 3);
    assert.strictEqual(chrome.background.service_worker, 'background.js');
    assert.strictEqual(chrome.background.type, 'module');
    assert.ok(!('scripts' in chrome.background), 'Chrome must not use Firefox background scripts');
    assert.strictEqual(chrome.minimum_chrome_version, '116');
    assert.deepStrictEqual(chrome.permissions, ['storage', 'scripting', 'activeTab']);
    assert.ok(!('host_permissions' in chrome), 'job-site host permissions stay least-privilege');
    assert.deepStrictEqual(chrome.optional_host_permissions, [
      'https://*/*',
      'http://localhost/*',
      'http://*.localhost/*',
      'http://127.0.0.1/*'
    ]);

    const firefox = readManifest('firefox');
    assert.strictEqual(firefox.manifest_version, 3);
    assert.deepStrictEqual(firefox.background.scripts, ['background.js']);
    assert.strictEqual(firefox.background.type, 'module');
    assert.ok(!('service_worker' in firefox.background), 'Firefox must not claim Chrome service-worker support');
    assert.strictEqual(firefox.browser_specific_settings?.gecko.strict_min_version, '128.0');
    assert.strictEqual(firefox.browser_specific_settings?.gecko.id, 'extension@mensetsu.app');
    assert.deepStrictEqual(firefox.permissions, ['storage', 'scripting', 'activeTab']);
    assert.ok(!('host_permissions' in firefox));
    assert.deepStrictEqual(firefox.optional_host_permissions, chrome.optional_host_permissions);

    for (const browser of BROWSERS) {
      const root = path.join(distDir, browser);
      for (const relative of [
        'manifest.json',
        'background.js',
        'content.js',
        'popup.html',
        'popup.css',
        'popup.js',
        'src/background/background.js',
        'src/shared/browser-api.js',
        'src/content/content.js',
        'src/features/connection/connection.js',
        'src/features/connection/onboarding.js',
        'src/popup/popup.js',
        'src/services/storage.js'
      ]) {
        assert.ok(fs.existsSync(path.join(root, relative)), `${browser} build includes ${relative}`);
      }

      const backgroundShim = fs.readFileSync(path.join(root, 'background.js'), 'utf8');
      assert.match(backgroundShim, /from '\.\/src\/background\/background\.js'/);

      const popupShim = fs.readFileSync(path.join(root, 'popup.js'), 'utf8');
      assert.match(popupShim, /from '\.\/src\/popup\/popup\.js'/);

      const content = fs.readFileSync(path.join(root, 'content.js'), 'utf8');
      assert.doesNotThrow(() => new Function(content), `${browser} content bundle parses as classic script`);
      assert.match(content, /mensetsu\/ping/);
      assert.doesNotMatch(content, /^\s*(?:import|export)\s/m, `${browser} content bundle stays a classic script`);
    }
  });

  it('build output is byte-for-byte deterministic across runs', async () => {
    await build({ quiet: true });
    const first = hashTree(distDir);
    await build({ quiet: true });
    const second = hashTree(distDir);
    assert.deepStrictEqual(second, first);
  });

  it('ships no runtime JavaScript sources beside the TypeScript originals', () => {
    const runtimeDir = path.join(extensionRoot, 'src');
    const offenders = walk(runtimeDir)
      .filter((full) => full.endsWith('.js') || full.endsWith('.mjs') || full.endsWith('.cjs'))
      .map((full) => path.relative(extensionRoot, full));
    assert.deepStrictEqual(offenders, [], 'app/extension/src must contain TypeScript runtime sources only');
  });
});

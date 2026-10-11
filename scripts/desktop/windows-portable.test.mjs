import { test } from 'node:test';
import assert from 'node:assert/strict';
import { existsSync, mkdirSync, mkdtempSync, readFileSync, readdirSync, rmSync, writeFileSync } from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { stageWindowsPortable } from './windows-portable.mjs';

function fixture(t) {
  const root = mkdtempSync(path.join(os.tmpdir(), 'portable-'));
  t.after(() => rmSync(root, { recursive: true, force: true }));
  const files = {
    'apps/desktop/src-tauri/tauri.conf.json': JSON.stringify({ version: '1.2.3', bundle: {
      resources: { 'localizations/en.lproj/InfoPlist.strings': 'en.lproj/InfoPlist.strings' },
    } }),
    'apps/desktop/src-tauri/binaries/speclink-x86_64-pc-windows-msvc.exe': 'same-source CLI',
    'apps/desktop/src-tauri/localizations/en.lproj/InfoPlist.strings': 'resource',
    'target/x86_64-pc-windows-msvc/release/speclink-desktop.exe': 'desktop GUI',
    'target/x86_64-pc-windows-msvc/release/unrelated.pdb': 'do not distribute',
    'LICENSE': 'license',
    'docs/development/windows-portable.md': 'usage instructions',
  };
  for (const [relative, contents] of Object.entries(files)) {
    const file = path.join(root, relative);
    mkdirSync(path.dirname(file), { recursive: true });
    writeFileSync(file, contents);
  }
  return root;
}

test('packages distinct GUI and CLI names, mapped resources, license and build version', (t) => {
  const folder = stageWindowsPortable(fixture(t));
  assert.equal(path.basename(folder), 'Speclink_1.2.3_x64-portable');
  assert.ok(existsSync(path.join(folder, 'speclink-portable')));
  assert.equal(existsSync(path.join(folder, 'data')), false, 'never distribute personal data');
  assert.equal(readFileSync(path.join(folder, 'speclink-desktop.exe'), 'utf8'), 'desktop GUI');
  assert.equal(readFileSync(path.join(folder, 'speclink.exe'), 'utf8'), 'same-source CLI');
  assert.equal(readFileSync(path.join(folder, 'en.lproj/InfoPlist.strings'), 'utf8'), 'resource');
  assert.equal(readFileSync(path.join(folder, 'LICENSE'), 'utf8'), 'license');
  assert.equal(readFileSync(path.join(folder, 'README.md'), 'utf8'), 'usage instructions');
  assert.equal(JSON.parse(readFileSync(path.join(folder, 'build-info.json'))).version, '1.2.3');
  assert.equal(existsSync(path.join(folder, 'unrelated.pdb')), false);
  const names = readdirSync(folder).map((name) => name.toLowerCase());
  assert.equal(new Set(names).size, names.length);
});

test('fails packaging when the matching CLI sidecar is missing', (t) => {
  const root = fixture(t);
  rmSync(path.join(root, 'apps/desktop/src-tauri/binaries/speclink-x86_64-pc-windows-msvc.exe'));
  assert.throws(() => stageWindowsPortable(root), /ENOENT/);
});

test('rebuilding removes obsolete package files without touching the installer', (t) => {
  const root = fixture(t);
  const folder = stageWindowsPortable(root);
  writeFileSync(path.join(folder, 'obsolete.dll'), 'stale');
  const installer = path.resolve(folder, '../../installer.exe');
  writeFileSync(installer, 'installer');
  stageWindowsPortable(root);
  assert.equal(existsSync(path.join(folder, 'obsolete.dll')), false);
  assert.equal(readFileSync(installer, 'utf8'), 'installer');
});

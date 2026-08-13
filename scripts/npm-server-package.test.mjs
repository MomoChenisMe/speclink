import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { existsSync, mkdirSync, mkdtempSync, readFileSync, statSync, writeFileSync } from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const script = path.join(root, 'scripts', 'npm-server-package.mjs');

// release 管線的 server artifact 命名契約（任務 10.3）：server-<target>/ 底下一個 binary。
const TARGETS = [
  ['server-aarch64-apple-darwin', 'speclink-server', 'server-darwin-arm64', 'darwin', 'arm64'],
  ['server-x86_64-apple-darwin', 'speclink-server', 'server-darwin-x64', 'darwin', 'x64'],
  ['server-x86_64-unknown-linux-gnu', 'speclink-server', 'server-linux-x64', 'linux', 'x64'],
  ['server-aarch64-unknown-linux-gnu', 'speclink-server', 'server-linux-arm64', 'linux', 'arm64'],
  ['server-x86_64-pc-windows-msvc', 'speclink-server.exe', 'server-win32-x64', 'win32', 'x64'],
];

function makeBinaries(tmp, { omit } = {}) {
  const dir = path.join(tmp, 'bins');
  for (const [artifact, binaryName] of TARGETS) {
    if (artifact === omit) continue;
    mkdirSync(path.join(dir, artifact), { recursive: true });
    writeFileSync(path.join(dir, artifact, binaryName), `fake ${artifact}`);
  }
  return dir;
}

function run(args) {
  return spawnSync(process.execPath, [script, ...args], { encoding: 'utf8' });
}

test('物化主套件與五平台子套件，欄位齊備且版本對齊', () => {
  const tmp = mkdtempSync(path.join(os.tmpdir(), 'speclink-pkg-'));
  const out = path.join(tmp, 'out');
  const result = run(['--version', '0.2.0', '--binaries', makeBinaries(tmp), '--out', out]);
  assert.equal(result.status, 0, result.stderr);

  // 主套件：發布副本不帶 private、版本蓋為 0.2.0、optionalDependencies 五組同版。
  const main = JSON.parse(readFileSync(path.join(out, 'server', 'package.json'), 'utf8'));
  assert.equal(main.name, '@speclink/server');
  assert.equal(main.version, '0.2.0');
  assert.equal(main.private, undefined, '發布副本不得帶 private');
  assert.deepEqual(
    Object.fromEntries(Object.entries(main.optionalDependencies)),
    {
      '@speclink/server-darwin-arm64': '0.2.0',
      '@speclink/server-darwin-x64': '0.2.0',
      '@speclink/server-linux-x64': '0.2.0',
      '@speclink/server-linux-arm64': '0.2.0',
      '@speclink/server-win32-x64': '0.2.0',
    },
  );
  assert.ok(
    existsSync(path.join(out, 'server', 'bin', 'speclink-server.mjs')),
    '主套件缺 launcher',
  );

  // 平台子套件：os/cpu 圈定平台、binary 就位。
  for (const [, binaryName, pkgDir, osName, cpu] of TARGETS) {
    const pkg = JSON.parse(readFileSync(path.join(out, pkgDir, 'package.json'), 'utf8'));
    assert.equal(pkg.name, `@speclink/${pkgDir}`);
    assert.equal(pkg.version, '0.2.0');
    assert.deepEqual(pkg.os, [osName]);
    assert.deepEqual(pkg.cpu, [cpu]);
    const binary = path.join(out, pkgDir, binaryName);
    assert.ok(existsSync(binary), `${pkgDir} 缺 binary`);
    if (process.platform !== 'win32') {
      assert.ok(statSync(binary).mode & 0o111, `${pkgDir} 的 binary 應為可執行`);
    }
  }
});

test('--scope 覆寫主套件與子套件的 scope', () => {
  const tmp = mkdtempSync(path.join(os.tmpdir(), 'speclink-pkg-'));
  const out = path.join(tmp, 'out');
  const result = run([
    '--version', '0.2.0', '--binaries', makeBinaries(tmp), '--out', out, '--scope', '@acme',
  ]);
  assert.equal(result.status, 0, result.stderr);
  const main = JSON.parse(readFileSync(path.join(out, 'server', 'package.json'), 'utf8'));
  assert.equal(main.name, '@acme/server');
  assert.ok(main.optionalDependencies['@acme/server-darwin-arm64']);
  const sub = JSON.parse(readFileSync(path.join(out, 'server-darwin-arm64', 'package.json'), 'utf8'));
  assert.equal(sub.name, '@acme/server-darwin-arm64');
});

test('缺任一 target 的 binary 即非零結束並點名（fail-closed）', () => {
  const tmp = mkdtempSync(path.join(os.tmpdir(), 'speclink-pkg-'));
  const result = run([
    '--version', '0.2.0',
    '--binaries', makeBinaries(tmp, { omit: 'server-aarch64-apple-darwin' }),
    '--out', path.join(tmp, 'out'),
  ]);
  assert.notEqual(result.status, 0);
  assert.match(result.stderr, /aarch64-apple-darwin/);
});

test('版本不符 X.Y.Z 格式即非零結束', () => {
  const tmp = mkdtempSync(path.join(os.tmpdir(), 'speclink-pkg-'));
  const result = run([
    '--version', 'v0.2.0', '--binaries', makeBinaries(tmp), '--out', path.join(tmp, 'out'),
  ]);
  assert.notEqual(result.status, 0);
});

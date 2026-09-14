// CLI npm 套件物化腳本的單元測試（cli-distribution spec「CLI 以 npm 套件發布」；
// release-assets-trim 設計 D4）。形狀比照 npm-server-package.test.mjs：以子行程執行
// 物化腳本，驗主套件與五平台子套件的欄位；缺任一 binary 或版號不合形即 fail-closed。
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { existsSync, mkdirSync, mkdtempSync, readdirSync, readFileSync, rmSync, statSync, writeFileSync } from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');
const script = path.join(root, 'scripts', 'npm', 'npm-cli-package.mjs');

// release 管線的 CLI artifact 命名契約：cli-<target>/ 底下一個 raw binary（build job 上傳）。
const TARGETS = [
  ['cli-aarch64-apple-darwin', 'speclink', 'cli-darwin-arm64', 'darwin', 'arm64'],
  ['cli-x86_64-apple-darwin', 'speclink', 'cli-darwin-x64', 'darwin', 'x64'],
  ['cli-x86_64-unknown-linux-gnu', 'speclink', 'cli-linux-x64', 'linux', 'x64'],
  ['cli-aarch64-unknown-linux-gnu', 'speclink', 'cli-linux-arm64', 'linux', 'arm64'],
  ['cli-x86_64-pc-windows-msvc', 'speclink.exe', 'cli-win32-x64', 'win32', 'x64'],
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

/// 每個測試自己的暫存目錄，離開時整份刪掉。
function tempDir(t) {
  const dir = mkdtempSync(path.join(os.tmpdir(), 'speclink-cli-pkg-'));
  t.after(() => rmSync(dir, { recursive: true, force: true }));
  return dir;
}

test('物化主套件與五平台子套件，欄位齊備且版本對齊', (t) => {
  const tmp = tempDir(t);
  const out = path.join(tmp, 'out');
  const result = run(['--version', '0.5.0', '--binaries', makeBinaries(tmp), '--out', out]);
  assert.equal(result.status, 0, result.stderr);

  // 主套件：發布副本不帶 private、版本蓋為 0.5.0、bin 為 shim、postinstall 置換 shim、
  // optionalDependencies 五組同版。
  const main = JSON.parse(readFileSync(path.join(out, 'cli', 'package.json'), 'utf8'));
  assert.equal(main.name, '@speclink/cli');
  assert.equal(main.version, '0.5.0');
  assert.equal(main.private, undefined, '發布副本不得帶 private');
  assert.equal(main['//'], undefined, '發布副本不得帶 repo 內的說明欄');
  assert.deepEqual(main.bin, { speclink: 'bin/speclink' });
  assert.equal(main.scripts?.postinstall, 'node postinstall.mjs');
  assert.deepEqual(main.optionalDependencies, {
    '@speclink/cli-darwin-arm64': '0.5.0',
    '@speclink/cli-darwin-x64': '0.5.0',
    '@speclink/cli-linux-x64': '0.5.0',
    '@speclink/cli-linux-arm64': '0.5.0',
    '@speclink/cli-win32-x64': '0.5.0',
  });
  assert.ok(existsSync(path.join(out, 'cli', 'bin', 'speclink')), '主套件缺 shim');
  assert.ok(existsSync(path.join(out, 'cli', 'postinstall.mjs')), '主套件缺 postinstall');
  // 主套件帶上的檔就是 packages/cli-npm/package.json 的 files 清單，一份清單不抄兩處。
  const files = JSON.parse(readFileSync(path.join(root, 'packages/cli-npm/package.json'), 'utf8')).files;
  for (const file of files) assert.ok(existsSync(path.join(out, 'cli', file)), `主套件缺 files 列的 ${file}`);
  if (process.platform !== 'win32') {
    assert.ok(statSync(path.join(out, 'cli', 'bin', 'speclink')).mode & 0o111, 'shim 應為可執行');
  }

  // 平台子套件：os/cpu 圈定平台、內容物只有 binary 與 package.json。
  for (const [, binaryName, pkgDir, osName, cpu] of TARGETS) {
    const pkg = JSON.parse(readFileSync(path.join(out, pkgDir, 'package.json'), 'utf8'));
    assert.equal(pkg.name, `@speclink/${pkgDir}`);
    assert.equal(pkg.version, '0.5.0');
    assert.deepEqual(pkg.os, [osName]);
    assert.deepEqual(pkg.cpu, [cpu]);
    const binary = path.join(out, pkgDir, binaryName);
    assert.ok(existsSync(binary), `${pkgDir} 缺 binary`);
    if (process.platform !== 'win32') {
      assert.ok(statSync(binary).mode & 0o111, `${pkgDir} 的 binary 應為可執行`);
    }
    assert.deepEqual(readdirSync(path.join(out, pkgDir)).sort(), ['package.json', binaryName].sort());
  }
});

test('缺任一 target 的 binary 即非零結束並點名，且不產生任何套件（fail-closed）', (t) => {
  const tmp = tempDir(t);
  const out = path.join(tmp, 'out');
  const result = run([
    '--version', '0.5.0',
    '--binaries', makeBinaries(tmp, { omit: 'cli-aarch64-unknown-linux-gnu' }),
    '--out', out,
  ]);
  assert.notEqual(result.status, 0);
  assert.match(result.stderr, /aarch64-unknown-linux-gnu/);
  assert.equal(existsSync(out), false, '缺平台時不得留下半套輸出');
});

test('版本不符 X.Y.Z 格式即非零結束', (t) => {
  const tmp = tempDir(t);
  const result = run([
    '--version', 'v0.5.0', '--binaries', makeBinaries(tmp), '--out', path.join(tmp, 'out'),
  ]);
  assert.notEqual(result.status, 0);
  assert.match(result.stderr, /X\.Y\.Z/);
});

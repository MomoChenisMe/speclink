import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { mkdirSync, mkdtempSync, readFileSync, writeFileSync } from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const script = path.join(root, 'scripts', 'npm-engine-package.mjs');

// napi create-npm-dir 的產物形狀：主套件根一份 package.json，npm/<平台>/ 各一份。
function makeTree(tmp, platforms) {
  const dir = path.join(tmp, 'pkg');
  mkdirSync(dir, { recursive: true });
  writeFileSync(
    path.join(dir, 'package.json'),
    `${JSON.stringify(
      { name: '@speclink/engine', version: '0.1.0', license: 'MIT', '//': '版號為佔位符' },
      null,
      2,
    )}\n`,
  );
  for (const platform of platforms) {
    const subDir = path.join(dir, 'npm', platform);
    mkdirSync(subDir, { recursive: true });
    writeFileSync(
      path.join(subDir, 'package.json'),
      `${JSON.stringify(
        { name: `@speclink/engine-${platform}`, version: '0.1.0', os: [platform.split('-')[0]] },
        null,
        2,
      )}\n`,
    );
  }
  return dir;
}

function run(args) {
  return spawnSync(process.execPath, [script, ...args], { encoding: 'utf8' });
}

function readPkg(...segments) {
  return JSON.parse(readFileSync(path.join(...segments), 'utf8'));
}

const FIVE = [
  'win32-x64-msvc',
  'darwin-x64',
  'darwin-arm64',
  'linux-x64-gnu',
  'linux-arm64-gnu',
];

test('蓋章後主套件與每個平台子套件同版，optionalDependencies 恰為子套件名各釘同版', () => {
  const tmp = mkdtempSync(path.join(os.tmpdir(), 'speclink-engine-pkg-'));
  const dir = makeTree(tmp, FIVE);
  const result = run(['--version', '0.2.0', '--dir', dir]);
  assert.equal(result.status, 0, result.stderr);

  const main = readPkg(dir, 'package.json');
  assert.equal(main.version, '0.2.0');
  // 佔位符註解只對 repo 內的版號成立，蓋章後的發布產物不得帶著它。
  assert.equal(main['//'], undefined, '蓋章後不得保留佔位符註解');
  assert.deepEqual(main.optionalDependencies, {
    '@speclink/engine-win32-x64-msvc': '0.2.0',
    '@speclink/engine-darwin-x64': '0.2.0',
    '@speclink/engine-darwin-arm64': '0.2.0',
    '@speclink/engine-linux-x64-gnu': '0.2.0',
    '@speclink/engine-linux-arm64-gnu': '0.2.0',
  });

  for (const platform of FIVE) {
    const sub = readPkg(dir, 'npm', platform, 'package.json');
    assert.equal(sub.version, '0.2.0', `${platform} 子套件版號未蓋章`);
    assert.equal(sub.name, `@speclink/engine-${platform}`, `${platform} 子套件名不得被改寫`);
  }
});

test('子套件清單來自目錄列舉：目錄增減時 optionalDependencies 跟著變', () => {
  const tmp = mkdtempSync(path.join(os.tmpdir(), 'speclink-engine-pkg-'));
  const dir = makeTree(tmp, ['darwin-arm64', 'linux-x64-gnu']);
  assert.equal(run(['--version', '0.2.0', '--dir', dir]).status, 0);
  assert.deepEqual(Object.keys(readPkg(dir, 'package.json').optionalDependencies), [
    '@speclink/engine-darwin-arm64',
    '@speclink/engine-linux-x64-gnu',
  ]);

  // 同一棵樹再長出一個平台目錄，重跑後清單應含三筆——不得硬編碼 triple。
  const extra = path.join(dir, 'npm', 'win32-x64-msvc');
  mkdirSync(extra, { recursive: true });
  writeFileSync(
    path.join(extra, 'package.json'),
    `${JSON.stringify({ name: '@speclink/engine-win32-x64-msvc', version: '0.1.0' }, null, 2)}\n`,
  );
  assert.equal(run(['--version', '0.3.0', '--dir', dir]).status, 0);
  assert.deepEqual(readPkg(dir, 'package.json').optionalDependencies, {
    '@speclink/engine-darwin-arm64': '0.3.0',
    '@speclink/engine-linux-x64-gnu': '0.3.0',
    '@speclink/engine-win32-x64-msvc': '0.3.0',
  });
});

test('缺 --version 即非零結束', () => {
  const tmp = mkdtempSync(path.join(os.tmpdir(), 'speclink-engine-pkg-'));
  const result = run(['--dir', makeTree(tmp, FIVE)]);
  assert.notEqual(result.status, 0);
  assert.match(result.stderr, /--version/);
});

test('版本不符 X.Y.Z 格式即非零結束（tag 名須先去 v 前綴）', () => {
  const tmp = mkdtempSync(path.join(os.tmpdir(), 'speclink-engine-pkg-'));
  const result = run(['--version', 'v0.2.0', '--dir', makeTree(tmp, FIVE)]);
  assert.notEqual(result.status, 0);
});

test('沒有任何平台子套件目錄即非零結束（fail-closed）', () => {
  const tmp = mkdtempSync(path.join(os.tmpdir(), 'speclink-engine-pkg-'));
  const result = run(['--version', '0.2.0', '--dir', makeTree(tmp, [])]);
  assert.notEqual(result.status, 0);
  assert.match(result.stderr, /npm/);
});

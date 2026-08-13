import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { chmodSync, mkdirSync, readFileSync, symlinkSync, writeFileSync } from 'node:fs';
import { mkdtempSync } from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';
import { fileURLToPath, pathToFileURL } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const launcherPath = path.join(root, 'packages/server-npm/bin/speclink-server.mjs');
const { platformPackageSuffix, buildQuickstart, decideSpawnArgs } = await import(
  pathToFileURL(launcherPath).href
);

// --- 平台對映（server-release spec「npm 套件一行啟動 server」） ---

test('os/cpu 對映到五個平台子套件，之外的平台回 null', () => {
  const matrix = [
    ['darwin', 'arm64', 'server-darwin-arm64'],
    ['darwin', 'x64', 'server-darwin-x64'],
    ['linux', 'x64', 'server-linux-x64'],
    ['linux', 'arm64', 'server-linux-arm64'],
    ['win32', 'x64', 'server-win32-x64'],
    ['freebsd', 'x64', null],
    ['win32', 'arm64', null],
  ];
  for (const [platform, arch, expected] of matrix) {
    assert.equal(platformPackageSuffix(platform, arch), expected, `${platform}/${arch}`);
  }
});

// --- 快速啟動的組態插值（設計 D9：仿 compose，環境變數 → YAML 落地） ---

// 路徑在 YAML 裡是雙引號 scalar（JSON 跳脫——Windows 反斜線成 \\），斷言要比對
// 跳脫後的字面，拿原始路徑 includes() 在 Windows 必然對不上。
const asYamlScalar = (p) => JSON.stringify(p);

test('零環境變數時走 sqlite 預設、資料目錄 ./speclink-data、public_url 連動預設埠', () => {
  const plan = buildQuickstart({});
  assert.equal(plan.error, undefined);
  assert.equal(plan.dataDir, path.resolve('./speclink-data'));
  assert.equal(plan.addr, '127.0.0.1:8080');
  assert.match(plan.configYaml, /driver: "sqlite"/);
  assert.ok(plan.configYaml.includes(asYamlScalar(path.join(plan.dataDir, 'store.db'))));
  assert.ok(plan.configYaml.includes(asYamlScalar(path.join(plan.dataDir, 'identity.db'))));
  assert.match(plan.configYaml, /public_url: "http:\/\/localhost:8080"/);
});

test('SPECLINK_PORT 連動 addr 與 public_url，SPECLINK_PUBLIC_URL 設定時勝出', () => {
  const ported = buildQuickstart({ SPECLINK_PORT: '9123' });
  assert.equal(ported.addr, '127.0.0.1:9123');
  assert.match(ported.configYaml, /public_url: "http:\/\/localhost:9123"/);

  const explicit = buildQuickstart({ SPECLINK_PORT: '9123', SPECLINK_PUBLIC_URL: 'https://spec.example.com' });
  assert.match(explicit.configYaml, /public_url: "https:\/\/spec\.example\.com"/);
});

test('SPECLINK_STORE=serverfs 產出 serverfs 目錄宣告', () => {
  const plan = buildQuickstart({ SPECLINK_STORE: 'serverfs', SPECLINK_DATA_DIR: '/tmp/x' });
  assert.match(plan.configYaml, /driver: "serverfs"/);
  assert.ok(plan.configYaml.includes(asYamlScalar(path.join(path.resolve('/tmp/x'), 'store'))));
});

test('SPECLINK_STORE=postgres 帶 URL 產出 url 欄位、缺 URL 即錯誤點名', () => {
  const ok = buildQuickstart({
    SPECLINK_STORE: 'postgres',
    SPECLINK_POSTGRES_URL: 'postgres://u@h/db',
  });
  assert.match(ok.configYaml, /driver: "postgres"/);
  assert.match(ok.configYaml, /url: "postgres:\/\/u@h\/db"/);
  assert.ok(!ok.configYaml.includes('store.db'), 'postgres 不應產出 sqlite 路徑');

  const missing = buildQuickstart({ SPECLINK_STORE: 'postgres' });
  assert.match(missing.error, /SPECLINK_POSTGRES_URL/);
});

test('未知 SPECLINK_STORE 即錯誤並列出合法值', () => {
  const plan = buildQuickstart({ SPECLINK_STORE: 'mongodb' });
  assert.match(plan.error, /mongodb/);
  assert.match(plan.error, /sqlite/);
  assert.match(plan.error, /serverfs/);
  assert.match(plan.error, /postgres/);
});

// --- 模式判定：零參數 → 快速啟動；帶參數或 SPECLINK_CONFIG → 透傳 ---

test('零參數且無 SPECLINK_CONFIG 走快速啟動', () => {
  assert.equal(decideSpawnArgs([], {}).mode, 'quickstart');
});

test('帶 --config 或子命令即純透傳、不產組態', () => {
  const config = decideSpawnArgs(['--config', 'x.yaml'], {});
  assert.equal(config.mode, 'passthrough');
  assert.deepEqual(config.args, ['--config', 'x.yaml']);

  const sub = decideSpawnArgs(['invite', '--email', 'a@b.c'], {});
  assert.equal(sub.mode, 'passthrough');
  assert.deepEqual(sub.args, ['invite', '--email', 'a@b.c']);
});

test('SPECLINK_CONFIG 補上 --config，args 已有 --config 時不重複', () => {
  const bare = decideSpawnArgs([], { SPECLINK_CONFIG: '/etc/s.yaml' });
  assert.equal(bare.mode, 'passthrough');
  assert.deepEqual(bare.args, ['--config', '/etc/s.yaml']);

  const withAddr = decideSpawnArgs(['--addr', '0.0.0.0:80'], { SPECLINK_CONFIG: '/etc/s.yaml' });
  assert.deepEqual(withAddr.args, ['--config', '/etc/s.yaml', '--addr', '0.0.0.0:80']);

  const explicit = decideSpawnArgs(['--config', 'mine.yaml'], { SPECLINK_CONFIG: '/etc/s.yaml' });
  assert.deepEqual(explicit.args, ['--config', 'mine.yaml']);
});

// --- 整合：假 binary 驗證透傳與 exit code（Windows 無法以 script 假冒 .exe，跳過） ---

const intTest = process.platform === 'win32' ? test.skip : test;

function plantFakePackage(tmp, { exitCode = 0 } = {}) {
  const suffix = platformPackageSuffix(process.platform, process.arch);
  assert.ok(suffix, '整合測試需在支援平台上執行');
  const pkgDir = path.join(tmp, 'node_modules', '@speclink', suffix);
  mkdirSync(pkgDir, { recursive: true });
  writeFileSync(path.join(pkgDir, 'package.json'), JSON.stringify({ name: `@speclink/${suffix}`, version: '0.0.0' }));
  const bin = path.join(pkgDir, 'speclink-server');
  // 假 binary：把收到的參數逐行寫到 SPECLINK_FAKE_OUT，以指定碼結束。
  writeFileSync(bin, `#!/bin/sh\nfor a in "$@"; do echo "$a" >> "$SPECLINK_FAKE_OUT"; done\nexit ${exitCode}\n`);
  chmodSync(bin, 0o755);
  return tmp;
}

intTest('透傳模式：參數逐字到達 binary、exit code 一致', () => {
  const tmp = mkdtempSync(path.join(os.tmpdir(), 'speclink-npm-'));
  plantFakePackage(tmp, { exitCode: 7 });
  const out = path.join(tmp, 'args.txt');
  const result = spawnSync(process.execPath, [launcherPath, '--config', 'x.yaml', '--addr', '1.2.3.4:9'], {
    encoding: 'utf8',
    env: { ...process.env, NODE_PATH: path.join(tmp, 'node_modules'), SPECLINK_FAKE_OUT: out },
  });
  assert.equal(result.status, 7, result.stderr);
  assert.deepEqual(readFileSync(out, 'utf8').trim().split('\n'), ['--config', 'x.yaml', '--addr', '1.2.3.4:9']);
});

intTest('快速啟動：組態 YAML 落地資料目錄並以 --config 傳入', () => {
  const tmp = mkdtempSync(path.join(os.tmpdir(), 'speclink-npm-'));
  plantFakePackage(tmp);
  const out = path.join(tmp, 'args.txt');
  const dataDir = path.join(tmp, 'data');
  const result = spawnSync(process.execPath, [launcherPath], {
    encoding: 'utf8',
    env: {
      ...process.env,
      NODE_PATH: path.join(tmp, 'node_modules'),
      SPECLINK_FAKE_OUT: out,
      SPECLINK_DATA_DIR: dataDir,
      SPECLINK_PORT: '8321',
    },
  });
  assert.equal(result.status, 0, result.stderr);
  const args = readFileSync(out, 'utf8').trim().split('\n');
  assert.equal(args[0], '--config');
  assert.equal(args[1], path.join(dataDir, 'config.yaml'));
  assert.deepEqual(args.slice(2), ['--addr', '127.0.0.1:8321']);
  const yaml = readFileSync(path.join(dataDir, 'config.yaml'), 'utf8');
  assert.match(yaml, /driver: "sqlite"/);
  assert.match(yaml, /public_url: "http:\/\/localhost:8321"/);
});

intTest('經 npm bin shim 的 symlink 呼叫時 main 仍執行（argv[1] 是 symlink、import.meta.url 是實體路徑）', () => {
  const tmp = mkdtempSync(path.join(os.tmpdir(), 'speclink-npm-'));
  plantFakePackage(tmp, { exitCode: 7 });
  const out = path.join(tmp, 'args.txt');
  // npm 在 node_modules/.bin 放的是指向 bin 檔的 symlink——node 預設解析主模組
  // 的 realpath，於是 import.meta.url 與 argv[1] 不同字面；入口判斷若比對原始
  // argv[1]，main() 靜默不執行、exit 0。
  const shim = path.join(tmp, 'speclink-server-shim');
  symlinkSync(launcherPath, shim);
  const result = spawnSync(process.execPath, [shim, '--config', 'x.yaml'], {
    encoding: 'utf8',
    env: { ...process.env, NODE_PATH: path.join(tmp, 'node_modules'), SPECLINK_FAKE_OUT: out },
  });
  assert.equal(result.status, 7, `main 未執行（status=${result.status}）：${result.stderr}`);
  assert.deepEqual(readFileSync(out, 'utf8').trim().split('\n'), ['--config', 'x.yaml']);
});

intTest('平台子套件缺席時以可讀錯誤點名', () => {
  const tmp = mkdtempSync(path.join(os.tmpdir(), 'speclink-npm-'));
  const result = spawnSync(process.execPath, [launcherPath, '--config', 'x.yaml'], {
    encoding: 'utf8',
    env: { ...process.env, NODE_PATH: path.join(tmp, 'node_modules') },
  });
  assert.notEqual(result.status, 0);
  const suffix = platformPackageSuffix(process.platform, process.arch);
  assert.ok(result.stderr.includes(suffix), `錯誤訊息應點名 ${suffix}：${result.stderr}`);
});

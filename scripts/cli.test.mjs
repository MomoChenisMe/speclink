// scripts/cli.mjs 的 node --test 測試（規格「checkout 內 CLI 測試入口」）。
// 注入假的 runSync／platform／env 觀察程序邊界：binary path、argv、stdio、
// environment、cwd 與 exit code 轉送；不執行真實 CLI。
import { test } from 'node:test';
import assert from 'node:assert/strict';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { checkoutCliPath, runCheckoutCli } from './cli.mjs';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');

/// 記錄每次 runSync 的呼叫，並依序回傳 syncResults（不足時視為 exit code 0）。
function trackedRunSync(syncResults = []) {
  const calls = [];
  let count = 0;
  return {
    calls,
    runSync: (cmd, args, options) => {
      calls.push({ cmd, args, options });
      return syncResults[count++] ?? { status: 0 };
    },
  };
}

/// 沒有 stdout 斷言需求時的預設呼叫參數：不落地任何真實環境值。
function baseDeps(overrides = {}) {
  return {
    args: [],
    env: { PATH: '/usr/local/bin' },
    platform: 'linux',
    fallbackCwd: '/repo/speclink',
    logError: () => {},
    ...overrides,
  };
}

// --- binary 解析（design 決策「以 checkout 內 binary 提供 CLI wrapper」） ---

test('checkoutCliPath：非 win32 解析到 checkout 的 target/debug/speclink', () => {
  assert.equal(checkoutCliPath('darwin'), path.join(ROOT, 'target', 'debug', 'speclink'));
  assert.equal(checkoutCliPath('linux'), path.join(ROOT, 'target', 'debug', 'speclink'));
});

test('checkoutCliPath：win32 解析到 target/debug/speclink.exe', () => {
  assert.equal(checkoutCliPath('win32'), path.join(ROOT, 'target', 'debug', 'speclink.exe'));
});

test('PATH 中另有 speclink 時仍只執行 checkout binary', () => {
  const { calls, runSync } = trackedRunSync();
  runCheckoutCli(baseDeps({
    args: ['status'],
    env: { PATH: '/opt/old-speclink/bin:/usr/local/bin' },
    runSync,
  }));

  assert.equal(calls.length, 1);
  assert.equal(calls[0].cmd, path.join(ROOT, 'target', 'debug', 'speclink'));
  assert.equal(path.isAbsolute(calls[0].cmd), true, '必須是絕對路徑，不得由 PATH 解析');
});

// --- 程序契約轉送 ---

test('argv 原序轉送、stdio 全 inherit，且不經 shell', () => {
  const { calls, runSync } = trackedRunSync();
  runCheckoutCli(baseDeps({ args: ['list', '--json', '--no-color'], runSync }));

  assert.deepEqual(calls[0].args, ['list', '--json', '--no-color']);
  assert.equal(calls[0].options.stdio, 'inherit');
  // shell 會讓 <args> 被再解析一次（引號、$、&），必須關閉。
  assert.notEqual(calls[0].options.shell, true);
});

test('environment 原樣繼承', () => {
  const env = { PATH: '/usr/local/bin', SPECLINK_TOKEN: 'x', LANG: 'zh_TW.UTF-8' };
  const { calls, runSync } = trackedRunSync();
  runCheckoutCli(baseDeps({ env, runSync }));

  assert.deepEqual(calls[0].options.env, env);
});

test('child cwd 優先採用 npm 的 INIT_CWD', () => {
  const { calls, runSync } = trackedRunSync();
  runCheckoutCli(baseDeps({
    args: ['list'],
    env: { PATH: '/usr/local/bin', INIT_CWD: '/tmp/remote-client' },
    fallbackCwd: '/repo/speclink',
    runSync,
  }));

  assert.equal(calls[0].options.cwd, '/tmp/remote-client');
  // binary 仍來自 checkout，不隨呼叫端 cwd 移動。
  assert.equal(calls[0].cmd, path.join(ROOT, 'target', 'debug', 'speclink'));
});

test('INIT_CWD 缺席時退回 wrapper 的 process.cwd()', () => {
  const { calls, runSync } = trackedRunSync();
  runCheckoutCli(baseDeps({ fallbackCwd: '/repo/speclink', runSync }));

  assert.equal(calls[0].options.cwd, '/repo/speclink');
});

test('INIT_CWD 為空字串時視同未提供，同樣退回 process.cwd()', () => {
  const { calls, runSync } = trackedRunSync();
  runCheckoutCli(baseDeps({
    env: { PATH: '/usr/local/bin', INIT_CWD: '' },
    fallbackCwd: '/repo/speclink',
    runSync,
  }));

  // 空的 cwd 會讓 spawn 以 ENOENT 失敗，錯誤看起來像 binary 不存在。
  assert.equal(calls[0].options.cwd, '/repo/speclink');
});

// --- exit code 與失敗轉送 ---

test('CLI 成功結束時回傳 0，且 wrapper 不增加任何 stdout', () => {
  const { runSync } = trackedRunSync([{ status: 0 }]);
  const written = [];
  const originalWrite = process.stdout.write;
  process.stdout.write = (chunk) => {
    written.push(String(chunk));
    return true;
  };

  let code;
  try {
    code = runCheckoutCli(baseDeps({ args: ['--version'], runSync }));
  } finally {
    process.stdout.write = originalWrite;
  }

  assert.equal(code, 0);
  assert.deepEqual(written, [], 'machine-readable 輸出不得被 wrapper 汙染');
});

test('CLI 以非零狀態結束時回傳相同狀態', () => {
  const { runSync } = trackedRunSync([{ status: 3 }]);
  assert.equal(runCheckoutCli(baseDeps({ args: ['validate', 'nope'], runSync })), 3);
});

test('checkout binary 無法執行時以非零狀態結束，且不改用 PATH 的 speclink', () => {
  const { calls, runSync } = trackedRunSync([
    { status: null, error: new Error('spawn ENOENT') },
  ]);
  const errors = [];

  const code = runCheckoutCli(baseDeps({
    args: ['status'],
    env: { PATH: '/opt/old-speclink/bin' },
    runSync,
    logError: (message) => errors.push(message),
  }));

  assert.notEqual(code, 0);
  assert.equal(calls.length, 1, '失敗後不得再嘗試第二個 binary');
  assert.equal(calls[0].cmd, path.join(ROOT, 'target', 'debug', 'speclink'));
  const stderr = errors.join('\n');
  assert.match(stderr, /target[/\\]debug[/\\]speclink/);
  assert.match(stderr, /cargo build -p speclink-cli/);
  // spawn 的 ENOENT 可能來自 binary 也可能來自 cwd——兩者都要點名才不會誤導。
  assert.match(stderr, /\/repo\/speclink/);
});

test('CLI 被 signal 收束時不當成成功', () => {
  const { runSync } = trackedRunSync([{ status: null, signal: 'SIGTERM' }]);
  const errors = [];

  const code = runCheckoutCli(baseDeps({
    args: ['auth', 'login'],
    runSync,
    logError: (message) => errors.push(message),
  }));

  assert.notEqual(code, 0);
  assert.match(errors.join('\n'), /SIGTERM/);
});

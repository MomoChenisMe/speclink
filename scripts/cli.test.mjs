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
/// exists 預設 true——binary 存在路徑；自動建置測試自行注入 false。
function baseDeps(overrides = {}) {
  return {
    args: [],
    env: { PATH: '/usr/local/bin' },
    platform: 'linux',
    fallbackCwd: '/repo/speclink',
    exists: () => true,
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
  // 自動建置落地後，走到這裡代表 binary 剛確認存在卻無法執行——提示應指向
  // cwd／執行權限，而不是已過期的「先建置」指引。
  assert.match(stderr, /執行權限/);
  // spawn 的 ENOENT 可能來自 binary 也可能來自 cwd——兩者都要點名才不會誤導。
  assert.match(stderr, /\/repo\/speclink/);
});

// --- 自動建置（規格「checkout binary 不存在時自動建置且禁止 fallback」） ---

test('binary 不存在：先於 checkout root 建置 speclink-cli，再執行 debug binary', () => {
  const { calls, runSync } = trackedRunSync();
  const code = runCheckoutCli(baseDeps({
    args: ['status'],
    env: { PATH: '/opt/old-speclink/bin' },
    exists: () => false,
    runSync,
  }));

  assert.equal(code, 0);
  assert.equal(calls.length, 2);
  assert.equal(calls[0].cmd, 'cargo');
  assert.deepEqual(calls[0].args, ['build', '-p', 'speclink-cli']);
  assert.equal(calls[0].options.cwd, ROOT, '自動建置固定於 checkout root，不受呼叫端 cwd 影響');
  assert.equal(calls[1].cmd, path.join(ROOT, 'target', 'debug', 'speclink'));
  assert.deepEqual(calls[1].args, ['status'], 'args 原序傳入建置出的 binary');
  for (const call of calls) {
    assert.notEqual(call.cmd, 'speclink', '絕不執行 PATH 中的 speclink');
  }
});

test('存在性檢查針對 checkout binary 路徑本身', () => {
  const seen = [];
  const { runSync } = trackedRunSync();
  runCheckoutCli(baseDeps({
    exists: (p) => {
      seen.push(p);
      return true;
    },
    runSync,
  }));

  assert.deepEqual(seen, [path.join(ROOT, 'target', 'debug', 'speclink')]);
});

test('自動建置的進度輸出不寫入 stdout（build 的 stdout 導向 stderr）', () => {
  const { calls, runSync } = trackedRunSync();
  runCheckoutCli(baseDeps({ exists: () => false, runSync }));

  assert.deepEqual(calls[0].options.stdio, ['inherit', 2, 'inherit']);
});

test('自動建置失敗：stderr 顯示原因、以建置狀態收場、不執行任何 CLI', () => {
  const { calls, runSync } = trackedRunSync([{ status: 101 }]);
  const errors = [];
  const code = runCheckoutCli(baseDeps({
    args: ['status'],
    env: { PATH: '/opt/old-speclink/bin' },
    exists: () => false,
    runSync,
    logError: (message) => errors.push(message),
  }));

  assert.equal(code, 101);
  assert.equal(calls.length, 1, '建置失敗後不得執行任何 CLI，也不得 fallback 到 PATH');
  assert.match(errors.join('\n'), /speclink-cli/);
});

test('自動建置無法啟動（cargo 缺席）：非零收場且不執行任何 CLI', () => {
  const { calls, runSync } = trackedRunSync([
    { status: null, error: new Error('spawn cargo ENOENT') },
  ]);
  const errors = [];
  const code = runCheckoutCli(baseDeps({
    exists: () => false,
    runSync,
    logError: (message) => errors.push(message),
  }));

  assert.notEqual(code, 0);
  assert.equal(calls.length, 1);
  assert.match(errors.join('\n'), /spawn cargo ENOENT/);
});

test('binary 已存在：不觸發建置，直接執行', () => {
  const { calls, runSync } = trackedRunSync();
  runCheckoutCli(baseDeps({ args: ['--version'], runSync }));

  assert.equal(calls.length, 1);
  assert.equal(calls[0].cmd, path.join(ROOT, 'target', 'debug', 'speclink'));
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

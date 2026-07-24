// scripts/dev.mjs 純函式層的 node --test 測試（規格「env 到設定的生成邏輯可測」）。
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { parseDotenv, buildDevConfig, startDevEnvironment } from './dev.mjs';

// --- .env 解析（design 決策 3：逐行 KEY=VALUE、跳過註解與空行、不支援展開） ---

test('parseDotenv：解析 KEY=VALUE、跳過註解與空行', () => {
  const parsed = parseDotenv(
    '# 註解行\n\nSPECLINK_STORE_DRIVER=sqlite\n  \nSPECLINK_PORT=8080\n',
  );
  assert.deepEqual(parsed, {
    SPECLINK_STORE_DRIVER: 'sqlite',
    SPECLINK_PORT: '8080',
  });
});

test('parseDotenv：值保留第一個 = 之後的全部內容，且不做變數展開', () => {
  const parsed = parseDotenv(
    'SPECLINK_POSTGRES_URL=postgres://u@h:5432/db?sslmode=require\nSPECLINK_STORE_PATH=$HOME/store.db\n',
  );
  assert.deepEqual(parsed, {
    SPECLINK_POSTGRES_URL: 'postgres://u@h:5432/db?sslmode=require',
    SPECLINK_STORE_PATH: '$HOME/store.db',
  });
});

// --- env → {configYaml, addr} 生成（design 決策 2 的映射表） ---

test('sqlite 全預設：store/identity 落 .dev、public_url 預設埠、addr 127.0.0.1:8080', () => {
  const { configYaml, addr } = buildDevConfig({}, {});
  assert.equal(addr, '127.0.0.1:8080');
  assert.match(configYaml, /^# 本檔由 npm run dev 生成/);
  assert.ok(
    configYaml.includes(
      'store:\n  driver: sqlite\n  path: .dev/store.db\nidentity:\n  driver: sqlite\n  path: .dev/identity.db\npublic_url: http://localhost:8080\n',
    ),
    `sqlite 全預設形狀不符：\n${configYaml}`,
  );
});

test('serverfs：預設 path 為 .dev/store', () => {
  const { configYaml } = buildDevConfig({}, { SPECLINK_STORE_DRIVER: 'serverfs' });
  assert.ok(
    configYaml.includes('store:\n  driver: serverfs\n  path: .dev/store\nidentity:'),
    `serverfs 形狀不符：\n${configYaml}`,
  );
});

test('postgres：store.url 取自 SPECLINK_POSTGRES_URL、無 path', () => {
  const url = 'postgres://speclink@localhost:5432/speclink';
  const { configYaml } = buildDevConfig(
    {},
    { SPECLINK_STORE_DRIVER: 'postgres', SPECLINK_POSTGRES_URL: url },
  );
  assert.ok(
    configYaml.includes(`store:\n  driver: postgres\n  url: ${url}\nidentity:`),
    `postgres 形狀不符：\n${configYaml}`,
  );
});

test('memory：store 段無 path 也無 url', () => {
  const { configYaml } = buildDevConfig({}, { SPECLINK_STORE_DRIVER: 'memory' });
  assert.ok(
    configYaml.includes('store:\n  driver: memory\nidentity:'),
    `memory 形狀不符：\n${configYaml}`,
  );
});

test('process env 蓋過 .env 檔值', () => {
  const { configYaml } = buildDevConfig(
    { SPECLINK_STORE_DRIVER: 'sqlite' },
    { SPECLINK_STORE_DRIVER: 'memory' },
  );
  assert.ok(configYaml.includes('store:\n  driver: memory\n'));
});

test('SPECLINK_PORT 連動 addr 與預設 public_url（design 決策 2）', () => {
  const { configYaml, addr } = buildDevConfig({}, { SPECLINK_PORT: '3000' });
  assert.equal(addr, '127.0.0.1:3000');
  assert.ok(configYaml.includes('public_url: http://localhost:3000\n'));
});

test('postgres 缺 SPECLINK_POSTGRES_URL：錯誤點名該鍵、不生成', () => {
  assert.throws(
    () => buildDevConfig({}, { SPECLINK_STORE_DRIVER: 'postgres' }),
    /SPECLINK_POSTGRES_URL/,
  );
});

test('未知 driver（mysql）：錯誤點名 SPECLINK_STORE_DRIVER 並列出四個合法值', () => {
  assert.throws(
    () => buildDevConfig({}, { SPECLINK_STORE_DRIVER: 'mysql' }),
    (err) => {
      assert.match(err.message, /SPECLINK_STORE_DRIVER/);
      for (const valid of ['sqlite', 'serverfs', 'postgres', 'memory']) {
        assert.match(err.message, new RegExp(valid));
      }
      return true;
    },
  );
});

// --- 啟動編排（design 決策「先建置 CLI 再啟動長時間程序」） ---
// 注入假的 runSync／spawnChild 觀察程序邊界：命令順序、argv、stdio、shell 與
// 失敗時的長時間 child 數；不啟動真實 cargo、server 或 desktop。

/// 依序回傳 syncResults 當作各 prerequisite 的結果（不足時視為成功），
/// 並把每次 prerequisite 與 child spawn 記進同一份 calls 以保留呼叫順序。
function trackedDeps(syncResults = []) {
  const calls = [];
  const errors = [];
  let syncCount = 0;
  return {
    calls,
    errors,
    deps: {
      addr: '127.0.0.1:8080',
      isWindows: false,
      runSync: (cmd, args, options) => {
        calls.push({ kind: 'sync', cmd, args, options });
        return syncResults[syncCount++] ?? { status: 0 };
      },
      spawnChild: (cmd, args) => {
        calls.push({ kind: 'spawn', cmd, args });
        return { cmd, args };
      },
      log: () => {},
      logError: (message) => errors.push(message),
    },
  };
}

function spawnCount(calls) {
  return calls.filter((call) => call.kind === 'spawn').length;
}

test('startDevEnvironment：CLI build 先於前端 build，兩者都成功才 spawn 長時間 child', () => {
  const { calls, deps } = trackedDeps();
  const result = startDevEnvironment(deps);

  assert.deepEqual(
    calls.map((call) => [call.kind, call.cmd, ...call.args]),
    [
      ['sync', 'cargo', 'build', '-p', 'speclink-cli'],
      ['sync', 'npm', 'run', 'build', '-w', 'apps/desktop'],
      [
        'spawn', 'cargo',
        'run', '-p', 'speclink-server', '--',
        '--config', '.dev/config.yaml', '--addr', '127.0.0.1:8080',
      ],
      ['spawn', 'npm', 'run', 'tauri', '-w', 'apps/desktop', '--', 'dev'],
    ],
  );
  assert.equal(result.status, null);
  // 長時間 child 仍只有 server 與 desktop 兩個——CLI 不進 lifecycle。
  assert.equal(result.children.length, 2);
});

test('startDevEnvironment：prerequisite 輸出直通終端', () => {
  const { calls, deps } = trackedDeps();
  startDevEnvironment(deps);

  for (const call of calls.filter((entry) => entry.kind === 'sync')) {
    assert.equal(call.options.stdio, 'inherit');
  }
});

test('startDevEnvironment：CLI build 非零狀態即中止，不建前端也不 spawn 長時間 child', () => {
  const { calls, deps } = trackedDeps([{ status: 101 }]);
  const result = startDevEnvironment(deps);

  assert.equal(result.status, 101);
  assert.deepEqual(result.children, []);
  assert.equal(spawnCount(calls), 0);
  assert.deepEqual(calls.map((call) => call.cmd), ['cargo']);
});

test('startDevEnvironment：CLI build 無法啟動時以 1 退出並點名 speclink-cli', () => {
  const { calls, errors, deps } = trackedDeps([
    { status: null, error: new Error('spawn cargo ENOENT') },
  ]);
  const result = startDevEnvironment(deps);

  assert.equal(result.status, 1);
  assert.deepEqual(result.children, []);
  assert.equal(spawnCount(calls), 0);
  assert.match(errors.join('\n'), /speclink-cli/);
});

test('startDevEnvironment：前端 build 失敗回傳其狀態且長時間 child 數為零', () => {
  const { calls, deps } = trackedDeps([{ status: 0 }, { status: 2 }]);
  const result = startDevEnvironment(deps);

  assert.equal(result.status, 2);
  assert.deepEqual(result.children, []);
  assert.equal(spawnCount(calls), 0);
});

test('startDevEnvironment：Windows 上只有 npm prerequisite 走 shell', () => {
  const { calls, deps } = trackedDeps();
  startDevEnvironment({ ...deps, isWindows: true });

  const syncCalls = calls.filter((call) => call.kind === 'sync');
  assert.equal(syncCalls[0].options.shell, false, 'cargo 是真 binary，不需要 shell');
  assert.equal(syncCalls[1].options.shell, true, 'Windows 的 npm 是 npm.cmd，需要 shell');
});

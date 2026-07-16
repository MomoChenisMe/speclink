// scripts/dev.mjs 純函式層的 node --test 測試（規格「env 到設定的生成邏輯可測」）。
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { parseDotenv, buildDevConfig } from './dev.mjs';

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

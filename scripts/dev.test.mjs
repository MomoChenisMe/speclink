// scripts/dev.mjs 純函式層的 node --test 測試（規格「env 到設定的生成邏輯可測」）。
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { parseDotenv, buildDevConfig, parseDevMode, startDevEnvironment } from './dev.mjs';

const SCRIPTS_DIR = path.dirname(fileURLToPath(import.meta.url));
const DEV_SCRIPT = path.resolve(SCRIPTS_DIR, 'dev.mjs');
const ROOT = path.resolve(SCRIPTS_DIR, '..');

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

test('startDevEnvironment：CLI build 成功後直接 spawn 長時間 child，不另建前端', () => {
  const { calls, deps } = trackedDeps();
  const result = startDevEnvironment(deps);

  assert.deepEqual(
    calls.map((call) => [call.kind, call.cmd, ...call.args]),
    [
      ['sync', 'cargo', 'build', '-p', 'speclink-cli'],
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

test('startDevEnvironment：prerequisite 只有 cargo，不走 shell', () => {
  const { calls, deps } = trackedDeps();
  startDevEnvironment(deps);

  const syncCalls = calls.filter((call) => call.kind === 'sync');
  assert.equal(syncCalls.length, 1, 'full 模式的 prerequisite 只剩 CLI build');
  assert.equal(syncCalls[0].options.shell, false, 'cargo 是真 binary，不需要 shell');
});

// --- 模式分流（規格「單獨啟動 server」「單獨啟動 desktop」） ---

test('parseDevMode：--server → server、--desktop → desktop、無旗標 → full', () => {
  assert.equal(parseDevMode(['node', 'scripts/dev.mjs', '--server']), 'server');
  assert.equal(parseDevMode(['node', 'scripts/dev.mjs', '--desktop']), 'desktop');
  assert.equal(parseDevMode(['node', 'scripts/dev.mjs']), 'full');
});

test('server 模式：不建 CLI、不建前端，只 spawn server', () => {
  const { calls, deps } = trackedDeps();
  const result = startDevEnvironment({ ...deps, mode: 'server' });

  assert.deepEqual(
    calls.map((call) => [call.kind, call.cmd, ...call.args]),
    [
      [
        'spawn', 'cargo',
        'run', '-p', 'speclink-server', '--',
        '--config', '.dev/config.yaml', '--addr', '127.0.0.1:8080',
      ],
    ],
  );
  assert.equal(result.status, null);
  assert.equal(result.children.length, 1);
});

test('desktop 模式：無任何 prerequisite，只 spawn tauri dev，不建 CLI 也不含 server', () => {
  const { calls, deps } = trackedDeps();
  const result = startDevEnvironment({ ...deps, mode: 'desktop' });

  // 前端由 tauri dev 的 beforeDevCommand 起 vite dev server 供應，編排層不再預先建置。
  assert.deepEqual(
    calls.map((call) => [call.kind, call.cmd, ...call.args]),
    [['spawn', 'npm', 'run', 'tauri', '-w', 'apps/desktop', '--', 'dev']],
  );
  assert.equal(result.status, null);
  assert.equal(result.children.length, 1);
});

// --- dev server 設定一致性（規格「dev 模式前端由 dev server 供應且變更免重編」） ---
// tauri.conf.json 的 devUrl 是寫死字串，vite 的 server.port 決定實際監聽埠——兩份設定
// 各改一邊時 dev 視窗只會載入失敗，沒有任何錯誤指向真正的原因。此測試把兩者釘在一起。

/// vite.config.ts 是 TypeScript，無法 import；以 server 區塊為界抓設定值。
function viteServerBlock() {
  const source = readFileSync(path.join(ROOT, 'apps/desktop/vite.config.ts'), 'utf8');
  return source.match(/server:\s*\{[\s\S]*?\}/)?.[0] ?? '';
}

function tauriBuildConfig() {
  const source = readFileSync(
    path.join(ROOT, 'apps/desktop/src-tauri/tauri.conf.json'),
    'utf8',
  );
  return JSON.parse(source).build ?? {};
}

test('tauri devUrl 的埠號與 vite server.port 相同', () => {
  const { devUrl } = tauriBuildConfig();
  assert.ok(
    devUrl,
    'tauri.conf.json 缺少 build.devUrl——沒有它 tauri dev 會載入編譯期嵌入的靜態 dist，前端改動不重編 Rust 就進不了視窗',
  );

  const vitePort = viteServerBlock().match(/port:\s*(\d+)/)?.[1];
  assert.ok(vitePort, 'apps/desktop/vite.config.ts 的 server 區塊未固定 port');
  assert.equal(
    new URL(devUrl).port,
    vitePort,
    `devUrl（${devUrl}）與 vite server.port（${vitePort}）不一致，dev 視窗會載入失敗`,
  );
});

test('devUrl 指向本機 loopback：dev 視窗不得載入外部來源', () => {
  const { devUrl } = tauriBuildConfig();
  assert.ok(
    ['localhost', '127.0.0.1', '[::1]'].includes(new URL(devUrl).hostname),
    `devUrl（${devUrl}）指向非本機位址——dev 視窗會載入遠端內容並以 webview 的權限執行`,
  );
});

test('vite dev server 啟用 strictPort：埠被占用時明確失敗而非靜默改埠', () => {
  assert.match(
    viteServerBlock(),
    /strictPort:\s*true/,
    'strictPort 未啟用——vite 會在埠被占用時自動換埠，devUrl 隨即指向錯誤位址，視窗開出空白頁而無錯誤訊息',
  );
});

test('tauri.conf.json 設定 beforeDevCommand 以啟動 vite dev server', () => {
  const { beforeDevCommand } = tauriBuildConfig();
  assert.ok(
    beforeDevCommand,
    'tauri.conf.json 缺少 build.beforeDevCommand——devUrl 指向的 dev server 不會有人啟動',
  );
  // Tauri 於 apps/desktop（tauri.conf.json 所屬的 npm 專案）執行此指令，不是 repo root：
  // 帶 -w／--workspace 會以「No workspaces found」收場，且只有實際啟動才看得出來。
  assert.doesNotMatch(
    beforeDevCommand,
    /\s(-w|--workspace)[\s=]/,
    `beforeDevCommand（${beforeDevCommand}）帶了 workspace 旗標——其 cwd 已是 apps/desktop，該旗標會讓 npm 找不到 workspace 而中止啟動`,
  );
});

test('tauri.conf.json 保留 frontendDist：release 與 bundle 仍讀靜態產物', () => {
  const { frontendDist } = tauriBuildConfig();
  assert.equal(frontendDist, '../dist', 'frontendDist 遭改動會影響 release bundle 的前端來源');
});

// --- 兩模式沿用既有設定驗證（規格「設定不合法即拒絕啟動」） ---
// 黑箱子程序測試：驗證失敗發生在任何 build／spawn 之前，子程序立即以 1 收場。

for (const flag of ['--server', '--desktop']) {
  test(`dev.mjs ${flag}：postgres 缺 URL 時非零拒絕啟動，錯誤同 npm run dev`, () => {
    const result = spawnSync(process.execPath, [DEV_SCRIPT, flag], {
      env: {
        ...process.env,
        SPECLINK_STORE_DRIVER: 'postgres',
        SPECLINK_POSTGRES_URL: '',
      },
      encoding: 'utf8',
      timeout: 15_000,
    });

    assert.equal(result.status, 1);
    assert.match(result.stderr, /SPECLINK_POSTGRES_URL/);
  });
}

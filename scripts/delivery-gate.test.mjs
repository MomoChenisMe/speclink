// Speclink Server 交付 gate 的靜態契約測試（delivery-baseline／server-release spec，
// design D5／D7）。以解析檔案內容驗證 root `test:all`、主 CI、release workflow、
// Dockerfile、.dockerignore 與部署文件的關鍵屬性；不執行任何 build。
//
// RED 先行：對現況這些斷言應失敗，改好設定後才轉綠。所有斷言只做靜態解析，
// 讓 gate 的結構性保證（順序、fail-fast、無 Node runtime、內嵌來源）可回歸。
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');

function read(relativePath) {
  return readFileSync(path.join(root, relativePath), 'utf8');
}

/// 回傳 needle 在 haystack 的起始索引，找不到時以帶標籤的訊息 fail——把「缺片段」
/// 與「順序錯」兩種失敗分開，避免 indexOf 回 -1 讓順序斷言假性通過。
function requireIndex(haystack, needle, label) {
  const i = haystack.indexOf(needle);
  assert.notEqual(i, -1, `${label}: 找不到片段 ${JSON.stringify(needle)}`);
  return i;
}

// --- root test:all（delivery-baseline「root 單一指令全量驗證」） ---

test('test:all 依序納入五個測試面，且 apps/server-web build 排在 cargo test --workspace 之前', () => {
  const pkg = JSON.parse(read('package.json'));
  const script = pkg.scripts?.['test:all'];
  assert.ok(script, 'package.json 缺少 test:all script');

  // 五個測試面都在鏈上。
  assert.match(script, /npm test -w packages\/ui/, 'test:all 缺少 packages/ui 測試面');
  assert.match(script, /npm test -w apps\/desktop/, 'test:all 缺少 apps/desktop 測試面');
  assert.match(script, /npm test -w apps\/server-web/, 'test:all 缺少 apps/server-web 測試面');
  assert.match(script, /cargo test --workspace/, 'test:all 缺少 Rust workspace 測試面');
  assert.match(script, /speclink-node/, 'test:all 缺少 crates/speclink-node 測試面');

  // server-web production build 存在，且排在 cargo test --workspace 之前——
  // Rust 的 embedded asset／route 測試才驗到當次 source 產出的 index 與 manifest。
  const webBuild = requireIndex(script, 'npm run build -w apps/server-web', 'test:all');
  const cargoTest = requireIndex(script, 'cargo test --workspace', 'test:all');
  assert.ok(
    webBuild < cargoTest,
    `test:all：apps/server-web build 必須排在 cargo test --workspace 之前\n${script}`,
  );
});

test('test:all 全程以 && 串接，任一面失敗即以非零 exit 中止', () => {
  const pkg = JSON.parse(read('package.json'));
  const script = pkg.scripts['test:all'];

  assert.match(script, /&&/, 'test:all 應以 && 串接');
  assert.doesNotMatch(script, /;/, 'test:all 不得用 ; 串接，否則前段失敗仍續跑');
  // 新增的 server-web 測試面前後都掛 &&，確保它也在 fail-fast 鏈上。
  assert.match(
    script,
    /&&\s*npm test -w apps\/server-web\s*&&/,
    'apps/server-web 測試面必須夾在 && 之間',
  );
});

// --- 主 CI（delivery-baseline「CI 執行完整測試」） ---

test('ci.yml 三平台跑三個 React workspace 測試、server-web build 先於 cargo test，且測試步驟不允許失敗', () => {
  const ci = read('.github/workflows/ci.yml');

  for (const os of ['ubuntu-latest', 'macos-latest', 'windows-latest']) {
    assert.ok(ci.includes(os), `ci.yml 缺少作業系統 ${os}`);
  }

  assert.match(ci, /npm test -w packages\/ui/, 'ci.yml 缺少 packages/ui 測試步驟');
  assert.match(ci, /npm test -w apps\/desktop/, 'ci.yml 缺少 apps/desktop 測試步驟');
  assert.match(ci, /npm test -w apps\/server-web/, 'ci.yml 缺少 apps/server-web 測試步驟');

  // server-web production build 排在 cargo test --workspace 之前。
  const webBuild = requireIndex(ci, 'npm run build -w apps/server-web', 'ci.yml');
  const cargoTest = requireIndex(ci, 'cargo test --workspace', 'ci.yml');
  assert.ok(webBuild < cargoTest, 'ci.yml：server-web build 必須排在 cargo test --workspace 之前');

  // 測試與 build 步驟不得標記為允許失敗。
  assert.doesNotMatch(
    ci,
    /continue-on-error:\s*true/,
    'ci.yml 測試／build 步驟不得設 continue-on-error: true',
  );
});

/// 把 ci.yml 依頂層 job 切段（`jobs:` 底下的兩空格 key）。前置產物的順序必須逐
/// job 檢查——用全檔 indexOf 會讓 A job 的建置步驟冒充 B job 的前置，斷言看起來
/// 通過、CI 上該 job 仍因缺產物而編不動。
function ciJobs() {
  const ci = read('.github/workflows/ci.yml');
  const body = ci.slice(requireIndex(ci, '\njobs:', 'ci.yml'));
  const jobs = new Map();
  let current = null;
  for (const line of body.split('\n')) {
    const header = line.match(/^ {2}([\w-]+):\s*$/);
    if (header) {
      current = header[1];
      jobs.set(current, []);
    } else if (current) {
      jobs.get(current).push(line);
    }
  }
  return [...jobs].map(([name, lines]) => [name, lines.join('\n')]);
}

/// job 內每個會編到該 crate 的 cargo 指令之前，都必須先備好指定產物。
function assertStagedBeforeCargo(job, body, prerequisite, cargoPatterns, artifact) {
  for (const pattern of cargoPatterns) {
    const hit = body.match(pattern);
    if (!hit) continue;
    const stagedAt = body.indexOf(prerequisite);
    assert.notEqual(
      stagedAt,
      -1,
      `ci.yml job ${job}：「${hit[0]}」會編到需要${artifact}的 crate，但 job 內沒有 ${prerequisite}`,
    );
    assert.ok(
      stagedAt < body.indexOf(hit[0]),
      `ci.yml job ${job}：${prerequisite} 必須排在「${hit[0]}」之前`,
    );
  }
}

// speclink-desktop 的 build script 於編譯期檢查 tauri.conf.json 的 externalBin
// 是否存在，而 binaries/ 是 gitignored——CI 沒佈就不是測試紅字，是 build script
// 直接讓整個 job 掛掉。
const COMPILES_DESKTOP = [/cargo test --workspace/, /cargo test\b[^\n]*-p speclink-desktop/];

// speclink-server 的 lib 以 RustEmbed 內嵌 apps/server-web/dist（同樣 gitignored），
// 缺資料夾則 derive 於編譯期失敗。desktop 的 dev-dependencies 帶 speclink-server，
// 所以編 desktop 的 job 也吃這份前置。
const COMPILES_SERVER = [...COMPILES_DESKTOP, /cargo test\b[^\n]*-p speclink-server/];

test('ci.yml：凡編譯 speclink-desktop 的 job，都在 cargo 之前佈好 CLI sidecar', () => {
  for (const [job, body] of ciJobs()) {
    assertStagedBeforeCargo(
      job,
      body,
      'node scripts/desktop-sidecar.mjs',
      COMPILES_DESKTOP,
      ' CLI sidecar',
    );
  }
});

test('ci.yml：凡編譯 speclink-server 的 job，都在 cargo 之前建好 server-web dist', () => {
  for (const [job, body] of ciJobs()) {
    assertStagedBeforeCargo(
      job,
      body,
      'npm run build -w apps/server-web',
      COMPILES_SERVER,
      ' server-web dist',
    );
  }
});

test('ci.yml：每個跑 vite build 的 job 都先以 lockfile 安裝依賴', () => {
  for (const [job, body] of ciJobs()) {
    for (const build of ['npm run build -w apps/server-web', 'npm run build -w apps/desktop']) {
      const buildAt = body.indexOf(build);
      if (buildAt === -1) continue;
      const installAt = body.indexOf('npm ci');
      assert.notEqual(installAt, -1, `ci.yml job ${job}：有 ${build} 卻沒有 npm ci`);
      assert.ok(installAt < buildAt, `ci.yml job ${job}：npm ci 必須排在 ${build} 之前`);
    }
  }
});

// --- Release（server-release） ---

test('release.yml 每個 server artifact 先建 apps/server-web 再 cargo build，並有無-dist 的 /login＋JSON-404 smoke', () => {
  const release = read('.github/workflows/release.yml');

  // server-web production build 先於 release 的 cargo build。
  const webBuild = requireIndex(release, 'npm run build -w apps/server-web', 'release.yml');
  const cargoBuild = requireIndex(release, 'cargo build --release', 'release.yml');
  assert.ok(
    webBuild < cargoBuild,
    'release.yml：apps/server-web build 必須排在 release 的 cargo build 之前',
  );

  // smoke：無相鄰 dist 啟動 binary，GET /login 載入內嵌 index。
  assert.match(release, /\/login/, 'release.yml 缺少 /login smoke 斷言');

  // 未知 /api 路徑回 JSON 404，不被 SPA fallback 吞成 200。
  assert.match(release, /\/api\/speclink\/v1\/web\//, 'release.yml 缺少未知 /api 路徑的 smoke');
  assert.match(release, /404/, 'release.yml 缺少 JSON-404 斷言');
});

// --- Dockerfile（server-release「Docker multi-stage 不攜帶 Node runtime」） ---

test('Dockerfile 以 Node stage 產 dist、Rust stage 內嵌，最終 runtime 僅 server binary＋non-root、無 Node runtime', () => {
  const dockerfile = read('crates/speclink-server/Dockerfile');
  const stages = dockerfile.split(/^FROM /m).slice(1); // 每段開頭是 image[ AS name]

  // node build → rust build → runtime 至少三段。
  assert.ok(
    stages.length >= 3,
    `Dockerfile 應為多階段（node→rust→runtime），實際 ${stages.length} 段`,
  );

  // Node stage：以 lockfile 安裝並執行 server-web production build。
  assert.match(dockerfile, /FROM node:/, 'Dockerfile 缺少 Node build stage');
  assert.match(dockerfile, /npm ci/, 'Node stage 必須以 lockfile 安裝（npm ci）');
  assert.match(
    dockerfile,
    /npm run build -w apps\/server-web/,
    'Node stage 必須執行 apps/server-web production build',
  );

  // Rust stage 帶入 Node stage 產出的 dist 供 compile-time embedding。
  assert.match(
    dockerfile,
    /COPY --from=\w+ [^\n]*apps\/server-web\/dist/,
    'Rust stage 必須帶入 Node stage 建好的 dist 供內嵌',
  );

  // 最終 runtime stage：無 Node 映像、無攜帶 Node runtime、以 non-root 執行。
  const runtime = 'FROM ' + stages[stages.length - 1];
  assert.doesNotMatch(runtime, /^FROM node/m, '最終 runtime 不得基於 Node 映像');
  assert.doesNotMatch(
    runtime,
    /COPY --from=\w+ [^\n]*node/,
    '最終 runtime 不得從其他 stage 攜帶 Node runtime',
  );
  assert.match(runtime, /^USER (?!root\b)\S+/m, '最終 runtime 必須以 non-root USER 執行');
});

// --- .dockerignore 與內嵌來源（server-release fail-closed embedding） ---

test('.dockerignore 保留常規排除且不排除 apps/server-web/dist，內嵌 dist 由 image 內建', () => {
  const dockerignore = read('.dockerignore');
  const dockerfile = read('crates/speclink-server/Dockerfile');

  // 常規排除仍在：不把 host 的 node_modules／target 帶進 build context。
  assert.match(dockerignore, /node_modules/, '.dockerignore 應排除 node_modules');
  assert.match(dockerignore, /^target$/m, '.dockerignore 應排除 target');

  // 不得以任何非註解行排除 server-web dist——若改採 host build 內嵌會斷。
  const ignoresWebDist = dockerignore
    .split('\n')
    .map((line) => line.trim())
    .some((line) => line && !line.startsWith('#') && line.includes('apps/server-web/dist'));
  assert.equal(ignoresWebDist, false, '.dockerignore 不得排除 apps/server-web/dist');

  // 內嵌來源由 image 內的 Node stage 現建，embedding 永不依賴 host 的 dist。
  assert.match(
    dockerfile,
    /npm run build -w apps\/server-web/,
    'Dockerfile 必須在 image 內建 apps/server-web dist（不依賴 host build）',
  );
});

// --- 部署文件（server-release／delivery-baseline 部署面） ---

test('server-deployment 文件涵蓋內嵌交付、npm→web→rust 建置順序、runtime 無 Node/dist/CDN 與 rollback 契約', () => {
  const doc = read('docs/server-deployment.zh-TW.md');

  for (const required of [
    'apps/server-web', // SPA 來源 workspace
    '內嵌', // 資產編入單一 binary/image
    'npm ci', // 建置順序起點：lockfile 安裝
    'npm run build -w apps/server-web', // web production build
    'CDN', // runtime 不需外部 CDN
    '回退', // rollback
    '上一版', // 部署上一版 binary/image
  ]) {
    assert.ok(doc.includes(required), `server-deployment 文件缺少「${required}」`);
  }

  // 建置順序：npm ci → apps/server-web build → 編譯 speclink-server（同一句內，避免跨段誤判）。
  assert.match(
    doc,
    /npm ci[\s\S]{0,160}npm run build -w apps\/server-web[\s\S]{0,160}speclink-server/,
    'server-deployment 文件應呈現 npm ci → web build → speclink-server 的建置順序',
  );
});

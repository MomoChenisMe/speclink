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

// --- release.yml 桌面 bundle 目標（desktop-release「更新描述檔隨 release 發布」） ---

test('release.yml 每個桌面平台都建置至少一個 updater-enabled bundle 目標', () => {
  const release = read('.github/workflows/release.yml');

  // Tauri 只為 app／appimage／msi／nsis 這幾個目標產出更新包；dmg 與 deb 不在其中。
  // macOS 若只建 dmg，bundler 會印警告並且不產出 Speclink.app.tar.gz——收集步驟的
  // cp 隨即找不到檔案，desktop job 紅燈，Release 全有全無的閘門讓整次發版落空。
  const UPDATER_ENABLED = ['app', 'appimage', 'msi', 'nsis'];

  const declared = [...release.matchAll(/^\s*bundles:\s*(\S+)\s*$/gm)].map((m) => m[1]);
  assert.ok(declared.length > 0, 'release.yml：找不到任何 bundles 宣告');

  for (const entry of declared) {
    const targets = entry.split(',');
    assert.ok(
      targets.some((t) => UPDATER_ENABLED.includes(t)),
      `release.yml：bundles「${entry}」不含任何 updater-enabled 目標（${UPDATER_ENABLED.join('／')}），` +
        '該平台不會產出更新包，收集步驟會找不到檔案',
    );
  }
});

test('release.yml 的 Linux 桌面建置安裝 AppImage 打包所需的 xdg-utils 與 FUSE', () => {
  const release = read('.github/workflows/release.yml');

  // AppImage 的 bundler 會呼叫 xdg-open；ubuntu-24.04-arm 映像未內建 xdg-utils
  // （22.04 有），只裝 FUSE 會讓 arm64 的桌面 job 在打包最後一步才炸。
  assert.match(release, /xdg-utils/, 'release.yml：Linux 依賴缺 xdg-utils（AppImage 需要 xdg-open）');
  assert.match(release, /libfuse2/, 'release.yml：Linux 依賴缺 FUSE（AppImage 打包需要）');
});

test('ci.yml 跑 scripts 測試面，且以 bash 展開 glob 以相容 CI 釘選的 Node 版本', () => {
  const ci = read('.github/workflows/ci.yml');

  const step = requireIndex(ci, 'node --test scripts/*.test.mjs', 'ci.yml');

  // Node 20 的 --test 不自行展開 glob（該能力 Node 21 才有），Windows runner 的
  // 預設 shell 也不展開——兩者相加會讓步驟以「找不到檔案」失敗而非跑測試。
  // 指定 shell: bash 讓 glob 在傳給 node 之前就被展開，三平台與釘選版本都成立。
  const shellDecl = ci.lastIndexOf('shell: bash', step);
  assert.notEqual(shellDecl, -1, 'ci.yml：scripts 測試步驟必須指定 shell: bash');
  assert.ok(
    ci.slice(shellDecl, step).split('\n').length <= 3,
    'ci.yml：shell: bash 必須屬於 scripts 測試步驟本身，而非更上面的其他步驟',
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

test('release.yml 的 Package 步驟只打包 CLI，server binary 不進 Release assets', () => {
  const release = read('.github/workflows/release.yml');
  const packageStart = requireIndex(release, '- name: Package', 'release.yml');
  const tail = release.slice(packageStart);
  const nextStep = tail.slice(1).search(/\n\s*- (?:name:|uses:)/);
  const packageStep = nextStep >= 0 ? tail.slice(0, nextStep + 1) : tail;

  // server 通路收斂為 Docker＋npm（server-release spec 修改後；設計 D6）：
  // Package 步驟打包上 Release 的只有 CLI。
  assert.ok(
    !packageStep.includes('speclink-server'),
    'release.yml：Package 步驟不得打包 speclink-server（server 不隨 Release 發布）',
  );
  assert.match(packageStep, /speclink/, 'release.yml：Package 步驟缺 CLI 打包');
});

test('release.yml 的 release job 以 body_path 前置下載指南，changelog 接續其後', () => {
  const release = read('.github/workflows/release.yml');

  // 下載指南由產生器落檔（設計 D7），body_path 指向同一個檔。
  const generate = /node scripts\/release-notes\.mjs --tag [^>\n]+> *(\S+)/.exec(release);
  assert.ok(generate, 'release.yml：缺 release-notes.mjs 產生下載指南的步驟');
  const bodyPath = /body_path:\s*(\S+)/.exec(release);
  assert.ok(bodyPath, 'release.yml：action-gh-release 缺 body_path');
  assert.equal(
    bodyPath[1],
    generate[1],
    'release.yml：body_path 必須指向 release-notes 產生器的輸出檔',
  );
  // 自動 changelog 保留，接續在指南之後。
  assert.match(release, /generate_release_notes:\s*true/, 'release.yml：generate_release_notes 不得移除');
});

test('release.yml 的 tap-publish job 於 Release 後以 TAP_PUSH_TOKEN 為開關更新 tap formula', () => {
  const release = read('.github/workflows/release.yml');
  const jobStart = requireIndex(release, 'tap-publish:', 'release.yml');
  const tail = release.slice(jobStart);
  const nextJob = tail.slice(1).search(/\n  [a-z][\w-]*:\s*\n/);
  const job = nextJob >= 0 ? tail.slice(0, nextJob + 1) : tail;

  // 通路推送排在 Release 之後，不是發布的前置條件（cli-distribution spec
  // 「Formula 隨發版自動推送 tap」；設計 D8）。
  assert.match(job, /needs:\s*\[?release\]?/, 'tap-publish 必須 needs: release');
  assert.match(job, /TAP_PUSH_TOKEN/, 'tap-publish 必須以 TAP_PUSH_TOKEN 為開關');
  assert.match(job, /scripts\/homebrew-formula\.mjs/, 'tap-publish 必須用 formula 產生器產出內容');
  assert.match(job, /Formula\/speclink\.rb/, 'tap-publish 必須更新 tap repo 的 Formula/speclink.rb');
});

test('release.yml 的 npm-publish job 於 NPM_TOKEN 存在時物化並發布 server 套件', () => {
  const release = read('.github/workflows/release.yml');

  // build job 把 server binary 以獨立 artifact 上傳供發布 job 使用（任務 10.3 的
  // 命名契約，與 npm-server-package.mjs 的 --binaries 佈局對齊）。
  assert.match(
    release,
    /name:\s*server-\$\{\{ matrix\.target \}\}/,
    'release.yml：build job 缺 server-<target> artifact 上傳',
  );

  const jobStart = requireIndex(release, 'npm-publish:', 'release.yml');
  const tail = release.slice(jobStart);
  const nextJob = tail.slice(1).search(/\n  [a-z][\w-]*:\s*\n/);
  const job = nextJob >= 0 ? tail.slice(0, nextJob + 1) : tail;
  assert.match(job, /needs:\s*\[?release\]?/, 'npm-publish 必須 needs: release');
  assert.match(job, /NPM_TOKEN/, 'npm-publish 必須以 NPM_TOKEN 為開關');
  assert.match(job, /scripts\/npm-server-package\.mjs/, 'npm-publish 必須以物化腳本產出套件');
  assert.match(job, /npm publish --access public/, 'npm-publish 必須公開發布');
});

test('release.yml 的 Release 資產收集逐一圈定 artifact 種類，server-* 不落入 dist', () => {
  const release = read('.github/workflows/release.yml');
  // 每個把 artifact 下載進 dist 的步驟都必須帶 pattern（明示圈定），且不得圈到
  // server-*——server binary 只給 npm-publish job，不進 Release assets。
  const blocks = release.split(/- uses: actions\/download-artifact@v4/).slice(1);
  const distBlocks = blocks
    .map((block) => block.split(/\n\s*- (?:name:|uses:)/)[0])
    .filter((block) => /path:\s*dist\b/.test(block));
  assert.ok(distBlocks.length > 0, 'release.yml：找不到下載進 dist 的步驟');
  for (const block of distBlocks) {
    const pattern = /pattern:\s*(\S+)/.exec(block);
    assert.ok(pattern, 'release.yml：下載進 dist 的步驟必須帶 pattern 圈定 artifact');
    assert.ok(
      !pattern[1].startsWith('server-'),
      'release.yml：dist 收集不得圈到 server-* artifact',
    );
  }
});

test('release.yml 的 docker 建置逐架構原生分開建、合併 job 打 tag（禁 QEMU 編譯路徑）', () => {
  const release = read('.github/workflows/release.yml');

  // 單一 job 的 platforms 同時宣告雙架構＝其中一個在 QEMU 模擬下編譯（設計 D10
  // 要根絕的路徑）；per-arch 各綁原生 runner。
  for (const m of release.matchAll(/platforms:\s*(\S+)/g)) {
    assert.ok(
      !(m[1].includes('amd64') && m[1].includes('arm64')),
      `release.yml：platforms「${m[1]}」同時宣告雙架構（QEMU 編譯路徑）`,
    );
  }
  assert.ok(!release.includes('setup-qemu'), 'release.yml：不應再安裝 QEMU');
  assert.match(
    release,
    /os:\s*ubuntu-latest\n\s*platform:\s*linux\/amd64/,
    'release.yml：缺 amd64 的原生建置 matrix 項',
  );
  assert.match(
    release,
    /os:\s*ubuntu-24\.04-arm\n\s*platform:\s*linux\/arm64/,
    'release.yml：缺 arm64 的原生建置 matrix 項',
  );

  // per-arch 以 digest 推、合併 job 以 imagetools 打 tag，發布原子性由合併承擔。
  assert.match(release, /push-by-digest=true/, 'release.yml：per-arch 建置必須 push by digest');
  const manifestStart = requireIndex(release, 'docker-manifest:', 'release.yml');
  const tail = release.slice(manifestStart);
  const nextJob = tail.slice(1).search(/\n  [a-z][\w-]*:\s*\n/);
  const manifest = nextJob >= 0 ? tail.slice(0, nextJob + 1) : tail;
  assert.match(manifest, /needs:\s*\[?docker\]?/, 'docker-manifest 必須 needs: docker');
  assert.match(manifest, /imagetools create/, 'docker-manifest 必須以 imagetools create 合併 digest');

  // 全有全無閘門改指向合併 job——兩架構 digest 都成功打上 tag 才發布。
  assert.match(
    release,
    /needs:\s*\[build, desktop, docker-manifest\]/,
    'release.yml：release job 的 needs 閘門必須指向 docker-manifest',
  );
});

test('tauri.conf.json 的 NSIS 安裝器內建繁中英文語系並用 Speclink 圖示', () => {
  const conf = JSON.parse(read('apps/desktop/src-tauri/tauri.conf.json'));
  const nsis = conf.bundle?.windows?.nsis ?? {};
  // 語系跟隨系統（desktop-release spec 修訂後；設計 D11）：繁中系統顯示繁中、
  // 其餘落回英文——兩者都要在清單內。
  assert.ok(Array.isArray(nsis.languages), 'nsis.languages 必須宣告');
  assert.ok(nsis.languages.includes('TradChinese'), 'nsis.languages 缺 TradChinese');
  assert.ok(nsis.languages.includes('English'), 'nsis.languages 缺 English');
  assert.match(nsis.installerIcon ?? '', /icon\.ico$/, 'nsis.installerIcon 必須指向 icon.ico');
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
  // web 階段必須釘在建置機原生架構（--platform=$BUILDPLATFORM）：Vite 產出的
  // dist 架構無關，而 Node/V8 的 JIT 在 QEMU 模擬 arm64 下會偶發 SIGILL
  // （v0.1.1／v0.1.2 首跑連兩次 exit 132，任務 2.7）。
  assert.match(
    dockerfile,
    /FROM --platform=\$BUILDPLATFORM node:/,
    'Dockerfile 的 Node build stage 必須帶 --platform=$BUILDPLATFORM（避免 QEMU 下跑 V8）',
  );
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

// --- engine 的 npm 發布通路（node-sdk-release spec；設計 D1／D2／D3） ---

/// 從 workflow 全文切出一個 job 的區塊（沿用本檔既有的「下一個 job 開頭」切法）。
function jobBlock(workflow, jobName, label) {
  const start = requireIndex(workflow, `\n  ${jobName}:\n`, label);
  const tail = workflow.slice(start + 1);
  const next = tail.slice(1).search(/\n  [a-z][\w-]*:\s*\n/);
  return next >= 0 ? tail.slice(0, next + 1) : tail;
}

/// 把 job 區塊切成一個個 step（步驟起首為 6 空格的 `- name:`／`- uses:`）。
/// 斷言錨定到單一步驟，否則「整個 job 內出現過這串字」會被註解或別的步驟滿足。
function jobSteps(jobText) {
  return jobText.split(/\n {6}- (?=name:|uses:)/).slice(1);
}

/// 找出符合全部條件的唯一步驟，找不到即以帶標籤的訊息 fail。
function requireStep(jobText, predicates, label) {
  const found = jobSteps(jobText).filter((step) => predicates.every((p) => p.test(step)));
  assert.equal(found.length, 1, `${label}: 應恰有一個步驟符合 ${predicates.join('、')}，實際 ${found.length} 個`);
  return found[0];
}

test('node-sdk.yml 可被 release 管線重用，且版號蓋章不經字串內插', () => {
  const nodeSdk = read('.github/workflows/node-sdk.yml');

  // 同檔雙角色（設計 D1）：push／PR 保留，另開 workflow_call 的選填 version 輸入。
  assert.match(nodeSdk, /^on:/m, 'node-sdk.yml 缺 on: 區塊');
  assert.match(nodeSdk, /\n  push:/, 'node-sdk.yml：push 觸發不得移除');
  assert.match(nodeSdk, /\n  pull_request:/, 'node-sdk.yml：pull_request 觸發不得移除');
  assert.match(nodeSdk, /\n  workflow_call:\n\s+inputs:\n\s+version:/, 'node-sdk.yml 缺 workflow_call 的 version 輸入');

  const pack = jobBlock(nodeSdk, 'package', 'node-sdk.yml');

  // 蓋章排在 napi artifacts 之後、npm pack 之前（設計 D2）。
  const iArtifacts = requireIndex(pack, 'npx napi artifacts', 'node-sdk.yml package job');
  const iStamp = requireIndex(pack, 'npm-engine-package.mjs', 'node-sdk.yml package job');
  // 以步驟名錨定，否則會命中註解裡提到的 npm pack。
  const iPack = requireIndex(pack, 'Pack the main package', 'node-sdk.yml package job');
  assert.ok(iArtifacts < iStamp && iStamp < iPack, 'node-sdk.yml：蓋章須落在 napi artifacts 與 npm pack 之間');

  // 版號經 env 傳入，不得把 ${{ inputs.version }} 直接內插進 run:——tag 名可含
  // 引號與分號，內插等於把 tag 名當 shell 原始碼執行。
  assert.ok(
    !/run:[^\n]*\$\{\{\s*inputs\.version\s*\}\}/.test(pack),
    'node-sdk.yml：version 不得內插進 run:，須以 env: 傳入',
  );
  assert.match(pack, /env:\s*\n\s+VERSION:\s*\$\{\{ inputs\.version \}\}/, 'node-sdk.yml：蓋章步驟須以 env VERSION 傳入版號');

  // 平台二進位的下載面須圈定——被 release 重用後與 caller 同一個 run，不圈定
  // 會把 server-*／desktop-* 一起灌進 napi 的掃描目錄。
  const binaryDownload = requireStep(
    pack,
    [/actions\/download-artifact@v4/, /path:\s*crates\/speclink-node\/artifacts/],
    'node-sdk.yml package job：平台二進位下載步驟',
  );
  assert.match(binaryDownload, /pattern:\s*bindings-\*/, 'node-sdk.yml：平台二進位下載須以 pattern: bindings-* 圈定');

  // binding.js 由 napi build 產生且被 gitignore，主套件的 files 列了它——
  // build 要帶一份出去、pack 要取回，否則 npm pack 靜默略過，發出去的套件 require 就炸。
  requireStep(
    jobBlock(nodeSdk, 'build', 'node-sdk.yml'),
    [/actions\/upload-artifact@v4/, /name:\s*binding-js/],
    'node-sdk.yml build job：JS loader 上傳步驟',
  );
  requireStep(
    pack,
    [/actions\/download-artifact@v4/, /name:\s*binding-js/],
    'node-sdk.yml package job：JS loader 還原步驟',
  );
  assert.match(pack, /tar -tzf[\s\S]{0,200}binding\.js/, 'node-sdk.yml：打包後須斷言主套件 tarball 含 binding.js');
});

test('release.yml 的 engine 三 job：版號前置把關、重用建置、發布冪等且子套件先發', () => {
  const release = read('.github/workflows/release.yml');

  // 版號與可發布性在單一 job 算好（Actions 表達式沒有字串裁切函式，with: 內
  // 展不開 ${GITHUB_REF_NAME#v}）；非 X.Y.Z 的 tag 不進入五平台建置。
  const version = jobBlock(release, 'engine-version', 'release.yml');
  assert.match(version, /outputs:\s*\n\s+version:/, 'engine-version 須輸出 version');
  assert.match(version, /publishable:/, 'engine-version 須輸出 publishable 作為前置把關');
  assert.match(version, /GITHUB_REF_NAME#v/, 'engine-version 須自 tag 名去掉 v 前綴');

  const build = jobBlock(release, 'engine-npm-build', 'release.yml');
  assert.match(build, /uses:\s*\.\/\.github\/workflows\/node-sdk\.yml/, 'engine-npm-build 須重用 node-sdk.yml，不得複製 build matrix');
  assert.match(build, /version:\s*\$\{\{ needs\.engine-version\.outputs\.version \}\}/, 'engine-npm-build 須傳入 engine-version 算出的版號');
  assert.match(build, /if:\s*needs\.engine-version\.outputs\.publishable == 'true'/, 'engine-npm-build 須以 publishable 前置把關');

  const publish = jobBlock(release, 'engine-npm-publish', 'release.yml');
  assert.match(publish, /needs:\s*\[engine-npm-build, release\]/, 'engine-npm-publish 必須 needs: [engine-npm-build, release]');
  assert.match(publish, /NPM_TOKEN/, 'engine-npm-publish 必須以 NPM_TOKEN 為開關');
  assert.match(publish, /name:\s*npm-tarballs/, 'engine-npm-publish 須下載 npm-tarballs artifact');
  assert.match(publish, /npm publish "\$tgz" --access public/, 'engine-npm-publish 必須公開發布 tarball');

  // 冪等：部分失敗後重跑不得因「同版已存在」永遠紅燈。
  assert.match(publish, /npm view/, 'engine-npm-publish 須先查 registry 以支援重跑（同版已存在即跳過）');

  // 子套件先發、主套件最後，optionalDependencies 於主套件上架時皆可解析。
  const iSub = requireIndex(publish, 'tarballs/npm/', 'release.yml engine-npm-publish');
  const iMain = publish.lastIndexOf('tarballs/*.tgz');
  assert.notEqual(iMain, -1, 'release.yml：engine-npm-publish 缺主套件 tarball 發布');
  assert.ok(iSub < iMain, 'engine-npm-publish：平台子套件須排在主套件之前發布');
});

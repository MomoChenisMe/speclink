## Why

remote 模式的手動測試目前要人肉四步：手寫 config YAML、cargo run 起 server、瀏覽器開 /setup、另開終端起 desktop 的 tauri dev——沒有 pnpm dev 等價物，Phase 3（desktop remote workspace）開工後這條迴圈每把刀每天都要走。docker compose 是部署形態不是開發迴圈：改一行 Rust 就重建映像不可行。

## What Changes

- repo root 新增一鍵編排：npm run dev 同時拉起 speclink-server（dev 設定）與 desktop 的 tauri dev；server 首跑於終端印出的一次性 /setup 連結即為 web 初始化入口。
- dev 設定走 env：編排 script 讀取 repo root 的 .env（不入版控），插值生成 .dev/config.yaml 後以 --config 啟動 server——server 產品碼零改動，「組態 YAML 不做環境變數展開」的既有 fail-closed 決策保持成立（與 deploy compose 的插值作法同構）。
- 新增 committed 的 .env.example 列出全部可調鍵與預設：SPECLINK_STORE_DRIVER（sqlite｜serverfs｜postgres｜memory，預設 sqlite）、SPECLINK_STORE_PATH、SPECLINK_POSTGRES_URL、SPECLINK_IDENTITY_PATH、SPECLINK_PORT、SPECLINK_PUBLIC_URL；命名沿用 deploy compose 既有的 SPECLINK_PORT／SPECLINK_PUBLIC_URL 詞彙。
- dev 資料持久化：.dev/（gitignored）跨重啟保留，setup／invite／PAT 做一次即可；npm run dev:reset 清空 .dev/ 回到全新 /setup。
- 正典文件補充：架構文件 §13.4 補「本地開發啟動」一段（native 直跑、同一條 /setup 流程；措辭避開「dev server」以免撞上 example/dev server 定位條款）；roadmap §4.2 記入此刀（定位 Phase 3 前置基建）。

## Capabilities

### New Capabilities

- `dev-harness`: repo root 一鍵開發編排的行為保證——npm run dev 同起 server 與 desktop、env 驅動的 store/identity 設定與 .env.example 對照、.dev/ 持久化語意與 dev:reset、不依賴 docker。

### Modified Capabilities

(none)

## Impact

- 相容性影響：純新增開發工具與文件；不動任何產品程式碼與既有測試；與 phase2-e2e-chain 零共檔可平行。
- Affected specs: `dev-harness`（新增）
- Affected code:
  - New: scripts/dev.mjs、scripts/dev.test.mjs、.env.example
  - Modified: package.json（root scripts：dev、dev:reset、scripts 測試併入）、.gitignore（.dev/ 與 .env）、docs/platform-architecture.zh-TW.md（§13.4 本地開發啟動段）、docs/implementation-refactor-roadmap.zh-TW.md（§4.2 刀組記入）
  - Removed: 無

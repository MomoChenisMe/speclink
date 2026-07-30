# dev-harness Specification

## Purpose

TBD - created by archiving change 'remote-dev-harness'. Update Purpose after archive.

## Requirements

### Requirement: 一鍵啟動 remote 開發環境

repo root 的 npm run dev SHALL 先驗證 dev 設定並建置目前 checkout 的 speclink-cli；CLI build 成功後，才 SHALL 建置 Desktop 前端並同時啟動 speclink-server（依 env 驅動的 dev 設定）與 desktop 的 tauri dev，且不依賴 docker 或 PATH 中已安裝的 speclink。編排 script SHALL 將兩個長時間 child process 的輸出直通終端——server 首跑印出的一次性 /setup 連結必須原樣可見。收到 SIGINT/SIGTERM 或任一 child 先退出時，script SHALL 終止另一個 child 一併收束，不留殘留 process。npm run dev:reset SHALL 保持只執行重置，不觸發 CLI 或 Desktop build。

#### Scenario: 全新 checkout 且未安裝 CLI 仍可啟動

- **WHEN** 在沒有 .env、沒有 .dev/ 且 PATH 中沒有 speclink 的全新 checkout 執行 npm run dev
- **THEN** script 先於 target/debug 建置目前 checkout 的 speclink-cli，再讓 server 以全預設（sqlite、.dev/store.db、identity .dev/identity.db、127.0.0.1:8080）啟動，終端出現含 /setup?token= 的連結行，desktop dev 視窗同時開啟

#### Scenario: CLI build 失敗即拒絕啟動

- **WHEN** 目前 checkout 的 speclink-cli build 以非零狀態結束或無法啟動
- **THEN** npm run dev 以非零 exit code 結束，且 speclink-server 與 desktop dev 的長時間 process 皆未啟動

#### Scenario: Ctrl+C 同殺兩個 child

- **WHEN** npm run dev 執行中於終端按 Ctrl+C
- **THEN** server 與 desktop dev 兩個 process 皆終止，無任一 process 殘留，且 CLI 不在長時間 child 清單中


<!-- @trace
source: dev-harness-cli-access
updated: 2026-07-24
code:
  - apps/desktop/core/src/project.rs
  - apps/desktop/core/src/settings.rs
  - apps/desktop/src-tauri/src/connections.rs
  - apps/desktop/src-tauri/src/lib.rs
  - apps/desktop/src/__tests__/remoteResilience.test.tsx
  - apps/desktop/src/__tests__/serversPanel.test.tsx
  - apps/desktop/src/__tests__/workspaceChooser.test.tsx
  - apps/desktop/src/adapter/connections.ts
  - apps/desktop/src/components/WorkspaceChooser.tsx
  - apps/desktop/src/i18n/messages.ts
  - crates/speclink-cli/src/commands.rs
  - crates/speclink-cli/src/main.rs
  - crates/speclink-cli/src/remote_commands.rs
  - crates/speclink-cli/tests/init_tools.rs
  - crates/speclink-cli/tests/remote_section.rs
  - crates/speclink-core/src/config.rs
  - crates/speclink-core/src/init.rs
  - docs/platform-architecture.zh-TW.md
  - docs/remote-getting-started.md
  - docs/remote-getting-started.zh-TW.md
  - package.json
  - scripts/cli.mjs
  - scripts/cli.test.mjs
  - scripts/dev.mjs
  - scripts/dev.test.mjs
  - scripts/remote-docs.test.mjs
-->

---
### Requirement: env 驅動的 dev 設定與 .env.example 對照

編排 script SHALL 讀取 repo root 的 .env（若存在）並與 process env 合併（process env 優先），插值生成 .dev/config.yaml 後以 --config 啟動 server；server 產品碼 SHALL NOT 因本能力而改動（組態 YAML 不做環境變數展開的既有決策保持成立）。repo SHALL 內含 committed 的 .env.example，逐鍵列出 SPECLINK_STORE_DRIVER（sqlite｜serverfs｜postgres｜memory，預設 sqlite）、SPECLINK_STORE_PATH、SPECLINK_POSTGRES_URL、SPECLINK_IDENTITY_PATH、SPECLINK_PORT、SPECLINK_PUBLIC_URL 與其預設及適用 driver；.env 與 .dev/ SHALL 列入 .gitignore。設定不合法時（未知 driver、postgres 缺 URL）script SHALL 在啟動任何 process 之前以可讀錯誤退出並指出鍵名——fail-closed。

#### Scenario: 切換 postgres driver

- **WHEN** .env 設 SPECLINK_STORE_DRIVER=postgres 且 SPECLINK_POSTGRES_URL 指向可用資料庫，執行 npm run dev
- **THEN** 生成的 .dev/config.yaml 之 store 段為 driver: postgres 與該 url，server 對該資料庫啟動

#### Scenario: postgres 缺 URL 即拒絕啟動

- **WHEN** SPECLINK_STORE_DRIVER=postgres 但 SPECLINK_POSTGRES_URL 未設，執行 npm run dev
- **THEN** script 在啟動 server 與 desktop 之前以非零 exit code 退出，錯誤訊息點名 SPECLINK_POSTGRES_URL

#### Scenario: process env 覆寫 .env 檔

- **WHEN** .env 內 SPECLINK_STORE_DRIVER=sqlite，而指令以 SPECLINK_STORE_DRIVER=memory npm run dev 執行
- **THEN** 生效的 driver 為 memory（process env 蓋過 .env 檔值）


<!-- @trace
source: remote-dev-harness
updated: 2026-07-16
code:
  - .env.example
  - docs/implementation-refactor-roadmap.zh-TW.md
  - docs/platform-architecture.zh-TW.md
  - package.json
  - scripts/dev.mjs
  - scripts/dev.test.mjs
-->

---
### Requirement: dev 資料持久化與顯式重置

.dev/ 下的 store 與 identity 資料 SHALL 跨 npm run dev 重啟保留——setup、invite、PAT 完成一次後，後續啟動直接進入可測狀態、不再印 setup 連結。npm run dev:reset SHALL 遞迴刪除 .dev/ 目錄且僅刪除該目錄（不碰 .env 與 deploy/），對不存在的 .dev/ 冪等成功。

#### Scenario: 重啟不重跑 setup

- **WHEN** 完成 /setup 建立 Admin 後結束 npm run dev，再次執行 npm run dev
- **THEN** server 啟動且不印新的 setup 連結，既有帳號與 PAT 可直接使用

#### Scenario: reset 後回到全新 setup

- **WHEN** 執行 npm run dev:reset 後再執行 npm run dev
- **THEN** .dev/ 自全新狀態重建，終端再次出現一次性 /setup?token= 連結，而 .env 檔內容不受影響


<!-- @trace
source: remote-dev-harness
updated: 2026-07-16
code:
  - .env.example
  - docs/implementation-refactor-roadmap.zh-TW.md
  - docs/platform-architecture.zh-TW.md
  - package.json
  - scripts/dev.mjs
  - scripts/dev.test.mjs
-->

---
### Requirement: env 到設定的生成邏輯可測

env→config 的解析與生成 SHALL 以純函式暴露於編排 script，並由 node --test 的測試覆蓋：四種 driver 各自的 YAML 輸出形狀、全預設值、process env 蓋 .env 檔、postgres 缺 URL 的錯誤、未知 driver 的錯誤（錯誤訊息列出四個合法值）。該測試 SHALL 併入 root 的 test:all 鏈。

#### Scenario: 未知 driver 的錯誤可讀

- **WHEN** 以 SPECLINK_STORE_DRIVER=mysql 呼叫生成函式
- **THEN** 回傳錯誤點名 SPECLINK_STORE_DRIVER 並列出 sqlite、serverfs、postgres、memory 四個合法值

<!-- @trace
source: remote-dev-harness
updated: 2026-07-16
code:
  - .env.example
  - docs/implementation-refactor-roadmap.zh-TW.md
  - docs/platform-architecture.zh-TW.md
  - package.json
  - scripts/dev.mjs
  - scripts/dev.test.mjs
-->

---
### Requirement: checkout 內 CLI 測試入口

repo root SHALL 提供 npm run cli -- <args>，固定執行同一 checkout 的 target/debug/speclink；Windows SHALL 使用 target/debug/speclink.exe。該 binary 不存在時，wrapper SHALL 先於 checkout root 建置 speclink-cli 再執行建置產物；建置進度輸出 SHALL NOT 寫入 stdout；建置失敗時 SHALL 於 stderr 顯示原因並以非零 exit code 結束。wrapper SHALL NOT 查詢或 fallback 到 PATH 中的 speclink，SHALL 原序轉送 `<args>`、繼承 environment 與 stdin/stdout/stderr，並回傳既有 CLI 的 exit code。child 工作目錄 SHALL 優先採用 npm 的 INIT_CWD，該值不存在時 SHALL 採用 wrapper 的 process.cwd()；自動建置 SHALL 於 checkout root 執行、不受呼叫端工作目錄影響。wrapper 不新增子指令、旗標、stdin 格式、輸出 envelope 或檔案系統效果（target/debug 的建置產物除外）；既有 --json camelCase payload、--no-color 與人眼輸出行為 SHALL 保持不變。

#### Scenario: PATH 中舊版 CLI 不影響 checkout binary

- **WHEN** PATH 中已有另一版 speclink，且目前 checkout 的 target/debug/speclink 已由 npm run dev 建置後執行 npm run cli -- status
- **THEN** wrapper 只執行目前 checkout 的 debug binary，並將 status 參數原序傳入

#### Scenario: 從外部測試 repo 保留呼叫端工作目錄

- **WHEN** 使用者位於 /tmp/remote-client，透過 npm --prefix <speclink-checkout> run cli -- list 呼叫 wrapper，且 INIT_CWD 為 /tmp/remote-client
- **THEN** CLI child 的工作目錄為 /tmp/remote-client，而 binary 仍來自 <speclink-checkout>/target/debug

#### Scenario: 互動輸入輸出與成功狀態透明轉送

- **WHEN** CLI 子指令讀取 stdin、寫入 stdout/stderr 並以 exit code 0 結束
- **THEN** wrapper 以 inherit 模式轉送 stdin/stdout/stderr，且 npm CLI script 以 exit code 0 結束

#### Scenario: CLI 失敗狀態透明轉送

- **WHEN** checkout CLI 因錯誤輸入、找不到變更或驗證失敗而以非零 exit code 結束
- **THEN** wrapper 保留 CLI 寫入 stdout/stderr 的內容並回傳相同的非零 exit code

#### Scenario: checkout binary 不存在時自動建置且禁止 fallback

- **WHEN** target/debug/speclink（Windows 為 speclink.exe）不存在，且 PATH 中存在可執行的 speclink，執行 npm run cli -- status
- **THEN** wrapper 先於 checkout root 建置 speclink-cli，再執行建置出的 debug binary 並將 status 原序傳入，SHALL NOT 執行 PATH 中的 speclink

#### Scenario: 自動建置失敗以非零收場

- **WHEN** binary 不存在且自動建置以非零狀態結束
- **THEN** wrapper 於 stderr 顯示建置失敗原因、以非零 exit code 結束，且 SHALL NOT 執行 PATH 中的 speclink

#### Scenario: machine-readable 輸出維持既有契約

- **WHEN** 使用 npm run --silent cli -- <args> 傳入既有 --json 或 --no-color 旗標（含觸發自動建置的情況）
- **THEN** wrapper 與自動建置皆不增加 stdout 內容，CLI 的 --json camelCase payload、--no-color 人眼文字與 exit code 維持既有位元級輸出契約


<!-- @trace
source: dev-quickstart-and-docs
updated: 2026-07-30
code:
  - README.en.md
  - README.md
  - docs/development.md
  - docs/development.zh-TW.md
  - package.json
  - scripts/cli.mjs
  - scripts/cli.test.mjs
  - scripts/dev.mjs
  - scripts/dev.test.mjs
  - scripts/remote-docs.test.mjs
-->

---
### Requirement: 單獨啟動 server

repo root SHALL 提供 npm run dev:server：只驗證 dev 設定並啟動 speclink-server，SHALL NOT 建置 CLI、SHALL NOT 建置 desktop 前端、SHALL NOT 啟動 desktop。設定來源與預設值（.env 合併 process env、sqlite、.dev/store.db、identity .dev/identity.db、127.0.0.1:8080）、輸出直通（server 首跑的一次性 /setup 連結原樣可見）、SIGINT/SIGTERM 收束與 .dev 持久化 SHALL 與 npm run dev 完全一致。

#### Scenario: 全新 checkout 零設定啟動後端

- **WHEN** 在沒有 .env、沒有 .dev/ 的全新 checkout 執行 npm run dev:server
- **THEN** server 以全預設啟動、終端出現含 /setup?token= 的連結行，過程中沒有 CLI 建置、沒有前端建置、沒有 desktop 視窗

#### Scenario: 設定不合法即拒絕啟動

- **WHEN** SPECLINK_STORE_DRIVER=postgres 且未設 SPECLINK_POSTGRES_URL 時執行 npm run dev:server
- **THEN** script 以非零 exit code 結束並顯示與 npm run dev 相同的錯誤訊息，server 未啟動

#### Scenario: 中斷收束無殘留

- **WHEN** npm run dev:server 執行中收到 SIGINT
- **THEN** server process 終止且無任何 process 殘留

<!-- @trace
source: dev-quickstart-and-docs
updated: 2026-07-30
code:
  - README.en.md
  - README.md
  - docs/development.md
  - docs/development.zh-TW.md
  - package.json
  - scripts/cli.mjs
  - scripts/cli.test.mjs
  - scripts/dev.mjs
  - scripts/dev.test.mjs
  - scripts/remote-docs.test.mjs
-->

---
### Requirement: 單獨啟動 desktop

repo root SHALL 提供 npm run dev:desktop：先建置 desktop 前端（vite 產出 dist）再啟動 desktop 的 tauri dev，SHALL NOT 啟動 speclink-server、SHALL NOT 要求任何 remote 設定。設定驗證 SHALL 與 npm run dev 共用——.env 不合法時（例如 postgres 缺 SPECLINK_POSTGRES_URL）SHALL 以非零 exit code 拒絕啟動。前端建置失敗時 SHALL 以非零結束且不啟動 tauri dev——tauri dev 載入靜態 dist，跳過建置會靜默沿用過期畫面。

#### Scenario: 前端先建置再啟動

- **WHEN** 修改 desktop 前端原始碼後執行 npm run dev:desktop
- **THEN** 前端建置先完成，tauri dev 開啟的視窗呈現本次修改後的畫面，而非過期 dist

#### Scenario: 前端建置失敗即拒絕啟動

- **WHEN** desktop 前端建置以非零狀態結束
- **THEN** npm run dev:desktop 以非零 exit code 結束，tauri dev 未啟動

#### Scenario: 無 server 亦可用

- **WHEN** 機器上沒有任何 speclink-server 在跑時執行 npm run dev:desktop
- **THEN** desktop 視窗以本地模式開啟並可瀏覽本地 openspec/ 看板，不因 remote 不可達而阻擋啟動

<!-- @trace
source: dev-quickstart-and-docs
updated: 2026-07-30
code:
  - README.en.md
  - README.md
  - docs/development.md
  - docs/development.zh-TW.md
  - package.json
  - scripts/cli.mjs
  - scripts/cli.test.mjs
  - scripts/dev.mjs
  - scripts/dev.test.mjs
  - scripts/remote-docs.test.mjs
-->
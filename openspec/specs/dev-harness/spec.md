# dev-harness Specification

## Purpose

TBD - created by archiving change 'remote-dev-harness'. Update Purpose after archive.

## Requirements

### Requirement: 一鍵啟動 remote 開發環境

repo root 的 npm run dev SHALL 以單一指令同時啟動 speclink-server（依 env 驅動的 dev 設定）與 desktop 的 tauri dev，且不依賴 docker。編排 script SHALL 將兩個 child process 的輸出直通終端——server 首跑印出的一次性 /setup 連結必須原樣可見。收到 SIGINT/SIGTERM 或任一 child 先退出時，script SHALL 終止另一個 child 一併收束，不留殘留 process。

#### Scenario: 全新 checkout 零設定可啟動

- **WHEN** 在沒有 .env 也沒有 .dev/ 的全新 checkout 執行 npm run dev
- **THEN** server 以全預設（sqlite、.dev/store.db、identity .dev/identity.db、127.0.0.1:8080）啟動，終端出現含 /setup?token= 的連結行，desktop dev 視窗同時開啟

#### Scenario: Ctrl+C 同殺兩個 child

- **WHEN** npm run dev 執行中於終端按 Ctrl+C
- **THEN** server 與 desktop dev 兩個 process 皆終止，無任一 process 殘留


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
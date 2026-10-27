## Why

server 目前有兩個「開箱即卡」的缺口。其一：第一位使用者必須由運維者 ssh 進主機跑 invite 子命令——藍圖 §13.2/§13.4 規定的正式開箱是 /setup：server 首次啟動輸出一次性 bootstrap token，運維者在瀏覽器完成第一位 Admin、檢視 Store 能力與 migration 狀態、建立第一組 Project/Repo，完成後 token 立即失效且 setup route 關閉。其二：Project/Repo registry 還躺在組態檔的靜態 projects 段——§13.2 把 registry 列為 Admin 管理對象，靜態組態意味著每次增減 project 都要改檔重啟，且後續 admin 子刀無從管理。本刀把 registry 遷入持久儲存、binding 裁決改讀庫、組態 projects 段依既有汰換模式退場（與 bootstrap tokens 段同路），並交付 /setup 流程。

目標使用者：初次架設 server 的運維者（瀏覽器完成開箱，不再需要主機 shell 建帳號）與後續 admin 子刀（registry 已在庫中，管理介面有的放矢）。

## What Changes

- Project/Repo registry 遷入 server 自有資料庫（identity 資料庫 schema 演進新增 projects 與 repos 表，沿用既有版本守門與 migrate 路徑）：identity 層新增 registry 讀寫介面；binding 裁決（project key 查核、repo 裁決）改讀庫，錯誤分類不變（未註冊 404、多義拒絕）。
- **BREAKING（server 組態）**：組態檔 projects 段退場——殘留即啟動失敗並指出已由 registry 取代；store、identity、public url 與事件段不變。既有測試與 e2e 的 registry 播種改走 registry 介面。
- 新增 /setup 首次啟動流程：server 啟動時若不存在任何 admin 使用者，生成一次性 bootstrap token（hash 落庫、有到期）並印於 stdout，/setup 以該 token 進入——單一流程完成：建立第一位 Admin（email、顯示名、密碼）、顯示 Store 能力與 schema/migration 狀態（manifest、health）、建立第一組 Project 與 Repo、顯示初始連線資訊（public url 出自部署組態，setup 顯示不寫）。完成即耗用 token 並關閉 setup route；已有 admin 的 server 上 /setup 一律回 404，bootstrap token 不再生成。
- invite 子命令與 /setup 建立的邀請 URL 一致以組態 public url 為基底；invite 子命令的 --project 參數改對 registry 查核，未註冊的 project key 拒絕。

## Capabilities

### New Capabilities

- `server-setup`: 首次啟動的 bootstrap token 與 /setup 流程、registry 的持久化管理介面與 binding 讀庫、setup 完成後的關閉語意。

### Modified Capabilities

- `reference-server`: 啟動組態需求改述——組態不再宣告 Project/Repo registry（殘留 projects 段拒絕啟動），registry 事實來源為 server 資料庫。

## Impact

- 相容性影響：server 組態檔不相容（projects 段移除）——尚無正式部署，遷移範圍是 repo 內測試組態與播種 helper。API 行為與錯誤分類不變（binding 的 404/403/refused 語意照舊）；CLI/桌面/本地模式零變更，parity 31 項、color 16 項、twin 8 情境凍結不動。identity 資料庫 schema version 遞增一版（migrate 自動升級，既有 users/PATs/sessions/device 憑證完整保留）。
- Affected specs: `server-setup`（新增）、`reference-server`（修改）
- Affected code:
  - New: crates/speclink-server/src/setup.rs、crates/speclink-server/tests/setup_flow.rs
  - Modified: crates/speclink-server/src/identity.rs、crates/speclink-server/src/identity_sqlite.rs、crates/speclink-server/src/config.rs、crates/speclink-server/src/auth.rs、crates/speclink-server/src/state.rs、crates/speclink-server/src/app.rs、crates/speclink-server/src/web.rs、crates/speclink-server/src/main.rs、crates/speclink-server/tests/common、crates/speclink-server/tests/binding.rs、crates/speclink-server/tests/startup.rs、crates/speclink-server/tests/e2e_cli.rs
  - Removed: 無

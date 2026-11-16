# reference-server Specification

## Purpose

Speclink Host 的參考 server 實作：路由一律服務 protocol DTO、錯誤取自封閉的 reason registry、binding 與認證作為前置且 fail closed、寫入原子提交且 CAS 衝突可辨，以及健康檢查與 ETag 輪詢的地基。本 capability 保證這個實作是 wire contract 的活基準——由真實 CLI 端到端跑過驗證，且啟動組態不齊時直接拒絕啟動而非帶著預設值上線。

## Requirements

### Requirement: 路由服務 protocol DTO 且錯誤屬封閉 registry

server SHALL 以 /api/speclink/v1/projects/ 加 project key 為基底，服務 typed client 現有全部查詢與命令路徑（binding、changes 清單與狀態、instructions、artifacts 讀寫、tasks done/undone、claim、archive、discussions 系列、specs、config、whoami、language），請求與回應 SHALL 為 speclink-protocol DTO，SHALL NOT 以 raw JSON 重組 payload。錯誤回應 SHALL 為 status、reason、message 三元組，reason SHALL 屬 protocol 的八值封閉 registry；message SHALL 沿用 engine 現行錯誤訊息文字。engine 五碼、store 六類與 wire 八值的映射 SHALL 在 server 內單點實作。

#### Scenario: 動詞經正路執行且回 DTO

- **WHEN** 對真 server 以 typed client 建立 change 後查詢清單
- **THEN** 建立回應與清單回應皆可被 protocol DTO 反序列化；清單含該 change 且與 stub 對測的欄位形狀一致

#### Scenario: 錯誤三元組與 reason 映射

- **WHEN** 對不存在的 change 查詢狀態
- **THEN** 回應 status 404、reason 為 not_found、message 與 engine 現行 not_found 訊息文字一致

---

<!-- @trace
source: server-http-adapter
updated: 2026-07-14
code:
  - Cargo.lock
  - Cargo.toml
  - crates/speclink-host/Cargo.toml
  - crates/speclink-host/src/bridge.rs
  - crates/speclink-host/src/lib.rs
  - crates/speclink-host/tests/bridge_dual_path.rs
  - crates/speclink-server/Cargo.toml
  - crates/speclink-server/src/app.rs
  - crates/speclink-server/src/auth.rs
  - crates/speclink-server/src/config.rs
  - crates/speclink-server/src/error.rs
  - crates/speclink-server/src/lib.rs
  - crates/speclink-server/src/main.rs
  - crates/speclink-server/src/routes.rs
  - crates/speclink-server/src/state.rs
  - crates/speclink-server/src/verb.rs
  - crates/speclink-server/tests/binding.rs
  - crates/speclink-server/tests/command_routes.rs
  - crates/speclink-server/tests/common/mod.rs
  - crates/speclink-server/tests/discussion_routes.rs
  - crates/speclink-server/tests/e2e_cli.rs
  - crates/speclink-server/tests/health.rs
  - crates/speclink-server/tests/query_routes.rs
  - crates/speclink-server/tests/startup.rs
  - crates/speclink-server/tests/sync_state.rs
-->

---
### Requirement: binding 與認證前置 fail closed

所有路由 SHALL 前置認證與 binding 裁決：bearer 憑證缺失或無效 SHALL 回 401 permission_denied——憑證查驗對 identity 儲存逐請求進行（hash 命中、未撤銷、未過期、所屬 user 為 active），SHALL NOT 存在組態檔靜態 token 表的認證路徑；actor 非 URL project 的 member SHALL 回 403 permission_denied。project key 未註冊 SHALL 回 404 not_found；X-Speclink-Repo 標頭指向未註冊 repo SHALL 回 not_found；缺標頭且該 project 註冊多個 repo SHALL 拒絕並於 message 指出候選需明示，SHALL NOT 自動選擇；恰一個 repo 時 SHALL 綁定之。X-Speclink-Api-Version 與 server 不相容 SHALL 拒絕並帶版本原因。前置任一步失敗 SHALL NOT 執行動詞。/binding SHALL 回 actor、project、repo、apiVersion、engineVersion 與 capabilities——宣告 polling 端點與 etag 支援，並宣告 sse push transport（事件端點 url 與 resume 支援），宣告 SHALL 與實際服務的端點一致。

#### Scenario: 未知 token 拒於門外

- **WHEN** 以 identity 儲存中不存在的 token 呼叫任一查詢路由
- **THEN** 回 401 且 reason 為 permission_denied；server 未執行任何 engine 動詞

#### Scenario: repo 多義拒絕不代選

- **WHEN** 對註冊兩個 repo 的 project 不帶 X-Speclink-Repo 呼叫 /binding
- **THEN** 回拒絕且 message 指出需明示 repo；SHALL NOT 回任一候選的成功 binding

#### Scenario: 有效 PAT 完成 binding

- **WHEN** 以 /account 建立的有效 PAT 對 actor 具 membership 的 project 呼叫 /binding
- **THEN** 回成功 binding，actor 為該 PAT 所屬 user 的身分

#### Scenario: capabilities 宣告含 sse 與 polling

- **WHEN** 完成相容的 /binding handshake
- **THEN** capabilities 的 events 宣告同時含 sse transport（resume 為 true）與既有 polling 宣告


<!-- @trace
source: server-sse-events
updated: 2026-07-14
code:
  - Cargo.lock
  - crates/speclink-protocol/src/events.rs
  - crates/speclink-server/Cargo.toml
  - crates/speclink-server/src/app.rs
  - crates/speclink-server/src/auth.rs
  - crates/speclink-server/src/config.rs
  - crates/speclink-server/src/events.rs
  - crates/speclink-server/src/lib.rs
  - crates/speclink-server/src/main.rs
  - crates/speclink-server/src/routes.rs
  - crates/speclink-server/src/state.rs
  - crates/speclink-server/src/verb.rs
  - crates/speclink-server/tests/auth_device.rs
  - crates/speclink-server/tests/auth_pat.rs
  - crates/speclink-server/tests/binding.rs
  - crates/speclink-server/tests/common/mod.rs
  - crates/speclink-server/tests/device_e2e.rs
  - crates/speclink-server/tests/device_flow.rs
  - crates/speclink-server/tests/query_routes.rs
  - crates/speclink-server/tests/refresh_rotation.rs
  - crates/speclink-server/tests/sse_events.rs
  - crates/speclink-server/tests/web_account.rs
  - crates/speclink-server/tests/web_activate.rs
  - crates/speclink-server/tests/web_device_sessions.rs
  - crates/speclink-server/tests/web_invite.rs
-->

---
### Requirement: 寫入原子提交且 CAS 衝突可辨

命令路由的寫入 SHALL 經宿主套件以儲存接口的一次提交寫入：文件與事件在同一次提交裡，事件同時進入儲存的事件記錄，SHALL NOT 有繞過宿主套件直寫儲存的路徑。帶 If-Match 的寫入在 revision 不符時 SHALL 回 409 revision_conflict 且帶 expected 與 actual revision；衝突的寫入 SHALL NOT 留下部分變更。server SHALL NOT 執行任何 shell-out git 操作。

#### Scenario: 競寫敗方得衝突且無殘留

- **WHEN** 兩個 client 以同一 If-Match revision 先後寫同一 artifact
- **THEN** 先者成功；後者回 409 revision_conflict 帶 expected/actual；重讀該 artifact 內容為先者版本，store 無後者的部分寫入

#### Scenario: 成功寫入事件同步可見

- **WHEN** 成功執行 task done 命令路由
- **THEN** 該範圍的事件記錄含對應的 task-completed 事件，與任務勾選在同一次提交可見


<!-- @trace
source: server-on-port-examples
updated: 2026-11-16T23:24:33+08:00
-->

---
### Requirement: 健康檢查與 ETag 輪詢地基

server SHALL 提供 /healthz（程序存活）與 /readyz（store health 可用性）；store 不可用時 /readyz SHALL 回非 2xx。查詢回應與 /sync-state SHALL 攜 scope 級 ETag——該 scope 任何成功 commit 後 SHALL 改變；/sync-state SHALL 支援 If-None-Match：記號未變回 304，變了回新記號。漏接全部事件的 client SHALL 能僅憑輪詢 /sync-state 與 Query 重讀收斂到最新狀態。

#### Scenario: 輪詢偵測變更

- **WHEN** client 取得 /sync-state 的 ETag 後另一 client 完成一筆寫入，client 以 If-None-Match 重詢
- **THEN** 第一次重詢在寫入前回 304；寫入後重詢回 200 與新 ETag

#### Scenario: store 失聯時 readyz 轉紅

- **WHEN** store 後端不可用（資料庫檔被移除或鎖死）時呼叫 /readyz 與任一查詢路由
- **THEN** /readyz 回非 2xx；查詢路由回 503 且 reason 為 unavailable

---

<!-- @trace
source: server-http-adapter
updated: 2026-07-14
code:
  - Cargo.lock
  - Cargo.toml
  - crates/speclink-host/Cargo.toml
  - crates/speclink-host/src/bridge.rs
  - crates/speclink-host/src/lib.rs
  - crates/speclink-host/tests/bridge_dual_path.rs
  - crates/speclink-server/Cargo.toml
  - crates/speclink-server/src/app.rs
  - crates/speclink-server/src/auth.rs
  - crates/speclink-server/src/config.rs
  - crates/speclink-server/src/error.rs
  - crates/speclink-server/src/lib.rs
  - crates/speclink-server/src/main.rs
  - crates/speclink-server/src/routes.rs
  - crates/speclink-server/src/state.rs
  - crates/speclink-server/src/verb.rs
  - crates/speclink-server/tests/binding.rs
  - crates/speclink-server/tests/command_routes.rs
  - crates/speclink-server/tests/common/mod.rs
  - crates/speclink-server/tests/discussion_routes.rs
  - crates/speclink-server/tests/e2e_cli.rs
  - crates/speclink-server/tests/health.rs
  - crates/speclink-server/tests/query_routes.rs
  - crates/speclink-server/tests/startup.rs
  - crates/speclink-server/tests/sync_state.rs
-->

---
### Requirement: 啟動組態 fail closed

server SHALL 以組態檔啟動，宣告 store driver、identity 資料庫（sqlite 路徑；memory 變體 SHALL 僅供測試組態）、public url 與事件參數。組態檔缺失、不可解析、宣告未知 driver、或任一段形狀不合 SHALL 使啟動失敗並印出指向錯誤的原因，SHALL NOT 以部分預設啟動；組態 SHALL NOT 含 bootstrap token 對 actor 的映射段，SHALL NOT 含 Project/Repo registry 段——registry 的事實來源是 server 資料庫（見 server-setup 能力），殘留 projects 段 SHALL 使啟動失敗並指出已由 registry 取代。sqlite driver SHALL 為預設持久層選項，memory driver SHALL 僅供測試組態。

儲存位置已經有資料、而且是 0.12.0 以前的版本建立的配置時，啟動 SHALL 失敗：exit code 非零、SHALL NOT 綁定任何連接埠、SHALL NOT 改動該資料，stderr SHALL 說明這份資料由較舊的版本建立，並列出三個步驟——以舊版執行 `speclink-server backup --config <file> --output <file>`、把組態指向空的儲存位置與空的 identity 資料庫、以這個版本執行 `speclink-server restore --config <file> --input <file>`。backup 子命令開啟儲存時遇到同樣的資料 SHALL 以同樣的方式失敗。儲存位置的資料帶有其他無法辨認的標記、沒有標記或較新的配置版本時，啟動 SHALL 失敗並印出讀到的標記與配置版本，SHALL NOT 印出上述三個步驟，SHALL NOT 改動該資料。

#### Scenario: 壞組態拒絕啟動

- **WHEN** 以 YAML 不可解析的組態檔啟動 server
- **THEN** 程序以非零 exit code 結束，stderr 指出組態檔路徑與解析原因；不綁定任何連接埠

#### Scenario: 未知 driver 拒絕啟動

- **WHEN** 組態宣告 store driver 為未支援的名稱
- **THEN** 啟動失敗且原因列出支援的 driver 名稱

#### Scenario: 殘留 tokens 段拒絕啟動

- **WHEN** 以仍含舊 bootstrap tokens 段的組態檔啟動 server
- **THEN** 啟動失敗且原因指出該段已由 identity 儲存取代

#### Scenario: 殘留 projects 段拒絕啟動

- **WHEN** 以仍含舊 projects registry 段的組態檔啟動 server
- **THEN** 啟動失敗且原因指出該段已由 server 資料庫的 registry 取代

#### Scenario: 舊版建立的 SQLite 資料拒絕啟動

- **WHEN** 組態的 store driver 是 sqlite，路徑指向 0.12.0 建立的資料庫檔，啟動 server
- **THEN** 程序以非零 exit code 結束，不綁定任何連接埠；stderr 說明資料由較舊的版本建立並列出 backup、指向空位置、restore 三個步驟；資料庫檔逐位元不變

#### Scenario: 舊版建立的資料夾與 PostgreSQL 資料同樣拒絕

- **WHEN** 組態的 store driver 是 serverfs 或 postgres，位置是 0.12.0 建立的資料，啟動 server
- **THEN** 結果與 SQLite 相同：非零 exit code、三個步驟、資料不變

#### Scenario: backup 子命令遇到舊版的資料

- **WHEN** 以這個版本對 0.12.0 建立的資料執行 backup 子命令
- **THEN** 非零 exit code；stderr 列出同樣的三個步驟；沒有產生備份檔

#### Scenario: 無法辨認的資料拒絕啟動

- **WHEN** 組態的 store driver 是 serverfs，路徑指向一個放著其他檔案的資料夾，啟動 server
- **THEN** 程序以非零 exit code 結束，stderr 說明這個位置的資料不是 speclink 的儲存；沒有印出三個步驟；資料夾裡沒有新增任何檔案


<!-- @trace
source: server-on-port-examples
updated: 2026-11-16T23:24:33+08:00
-->

---
### Requirement: 真實 CLI 端到端一致

以真實 CLI binary 對真 server（SQLite store）執行 remote 動詞流程 SHALL 與 fs 模式（形狀權威）一致。儲存決定型輸出（無本地路徑欄位者，如 status）SHALL stdout/stderr/exit code 逐位元一致；帶本地路徑或投影欄位的輸出（如 apply 的 changeDir/contextFiles，以及 fs-only 的 preflight）SHALL 在剔除該類欄位後內容一致——此與 stub 對測對同類欄位採欄位形狀（key）比對的語意一致。`crates/adapters/speclink-cli/tests/remote_read_path.rs` 全部對照情境的欄位形狀由 stub 對測凍結（設計決策 7：stub 驗 client、e2e 驗 server，互補不互代）；e2e SHALL 以代表性 remote 動詞重放驗證 server 端到端行為並驗證重啟持久性。e2e 的資料播種 SHALL 經命令路由完成，SHALL NOT 直接寫入 store 後端。

#### Scenario: 代表性動詞對真 server 重放

- **WHEN** 啟動 tempdir SQLite 組態的真 server，將 CLI 指向它並重放代表性 remote 動詞（list、status、instructions apply、discuss list）
- **THEN** 儲存決定型輸出與 fs 模式逐位元一致、帶路徑欄位者剔除該類欄位後內容一致；server 重啟後既建立的資料仍可完整查詢


<!-- @trace
source: stale-verification-vehicles
updated: 2026-07-27
code:
  - docs/implementation-refactor-roadmap.zh-TW.md
-->
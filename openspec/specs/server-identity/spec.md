# server-identity Specification

## Purpose

TBD - created by archiving change 'server-identity-pat'. Update Purpose after archive.

## Requirements

### Requirement: 邀請一次性且到期失效

邀請 SHALL 由 server binary 的 invite 子命令於主機上建立（email、顯示名、指派的 project memberships、可選 admin 旗標、到期時限），並輸出一次性 invite URL；對已有 active user 或未過期邀請的 email SHALL 拒絕重複建立。開啟有效邀請 SHALL 呈現設定密碼表單，提交後 SHALL 原子地建立 active user（含指派 memberships）並耗用邀請；已用、過期或未知的邀請 token SHALL 得到同一「邀請無效」回應，SHALL NOT 區分原因。

#### Scenario: 邀請走完即建立帳號

- **WHEN** 以 invite 子命令對新 email 建立含一個 project membership 的邀請，開啟 URL 設定密碼提交
- **THEN** user 建立為 active 且具該 membership；同一 URL 再開啟得到「邀請無效」

#### Scenario: 過期邀請不可用

- **WHEN** 開啟已過到期時限的邀請 URL
- **THEN** 回應與已用邀請相同的「邀請無效」頁；不建立任何 user

#### Scenario: 重複 email 拒絕

- **WHEN** 對已有 active user 的 email 執行 invite 子命令
- **THEN** 子命令以非零 exit code 拒絕並說明原因；不建立邀請

---

<!-- @trace
source: server-identity-pat
updated: 2026-07-14
code:
  - Cargo.lock
  - crates/speclink-server/Cargo.toml
  - crates/speclink-server/src/app.rs
  - crates/speclink-server/src/auth.rs
  - crates/speclink-server/src/config.rs
  - crates/speclink-server/src/identity.rs
  - crates/speclink-server/src/identity_sqlite.rs
  - crates/speclink-server/src/lib.rs
  - crates/speclink-server/src/main.rs
  - crates/speclink-server/src/state.rs
  - crates/speclink-server/src/web.rs
  - crates/speclink-server/tests/auth_pat.rs
  - crates/speclink-server/tests/binding.rs
  - crates/speclink-server/tests/command_routes.rs
  - crates/speclink-server/tests/common/mod.rs
  - crates/speclink-server/tests/discussion_routes.rs
  - crates/speclink-server/tests/e2e_cli.rs
  - crates/speclink-server/tests/identity.rs
  - crates/speclink-server/tests/invite.rs
  - crates/speclink-server/tests/query_routes.rs
  - crates/speclink-server/tests/startup.rs
  - crates/speclink-server/tests/sync_state.rs
  - crates/speclink-server/tests/web_account.rs
  - crates/speclink-server/tests/web_invite.rs
-->

---
### Requirement: 本機密碼登入與 session 安全屬性

一般使用者 SHALL 能以 email 與本機密碼登入取得 session；密碼 SHALL 以 argon2id 儲存。session cookie SHALL 具 HttpOnly、Secure 與 SameSite=Strict 屬性；全部變更型 POST SHALL 驗證同源，不符 SHALL 回 403。登入失敗 SHALL 回統一錯誤訊息，SHALL NOT 洩漏 email 是否存在。登出 SHALL 撤銷 server 端 session 記錄；被撤銷或過期的 session 後續請求 SHALL 視同未登入。未登入訪問帳號頁 SHALL 導向登入頁。

#### Scenario: 登入失敗訊息不洩漏帳號存在性

- **WHEN** 分別以不存在的 email 與存在但密碼錯誤的 email 提交登入
- **THEN** 兩者的回應狀態與錯誤訊息文字相同

#### Scenario: 登出後 session 立即失效

- **WHEN** 登入後執行登出，再以同一 cookie 請求帳號頁
- **THEN** 請求被視同未登入導向登入頁；server 端該 session 記錄已標記撤銷

---

<!-- @trace
source: server-identity-pat
updated: 2026-07-14
code:
  - Cargo.lock
  - crates/speclink-server/Cargo.toml
  - crates/speclink-server/src/app.rs
  - crates/speclink-server/src/auth.rs
  - crates/speclink-server/src/config.rs
  - crates/speclink-server/src/identity.rs
  - crates/speclink-server/src/identity_sqlite.rs
  - crates/speclink-server/src/lib.rs
  - crates/speclink-server/src/main.rs
  - crates/speclink-server/src/state.rs
  - crates/speclink-server/src/web.rs
  - crates/speclink-server/tests/auth_pat.rs
  - crates/speclink-server/tests/binding.rs
  - crates/speclink-server/tests/command_routes.rs
  - crates/speclink-server/tests/common/mod.rs
  - crates/speclink-server/tests/discussion_routes.rs
  - crates/speclink-server/tests/e2e_cli.rs
  - crates/speclink-server/tests/identity.rs
  - crates/speclink-server/tests/invite.rs
  - crates/speclink-server/tests/query_routes.rs
  - crates/speclink-server/tests/startup.rs
  - crates/speclink-server/tests/sync_state.rs
  - crates/speclink-server/tests/web_account.rs
  - crates/speclink-server/tests/web_invite.rs
-->

---
### Requirement: PAT 自助生命週期

登入的使用者 SHALL 能在帳號頁建立 PAT（名稱與到期日）與撤銷自己的 PAT。PAT 明文 SHALL 以可辨識 prefix 開頭且只在建立回應顯示一次；儲存 SHALL 僅含 token id、顯示用 prefix、hash、名稱、到期、撤銷時戳與 last-used，任何頁面或介面 SHALL NOT 能讀回明文。帳號頁的 PAT 清單 SHALL 顯示 prefix、名稱、到期與 last-used。撤銷 SHALL 即時生效。

#### Scenario: 明文只出現一次

- **WHEN** 建立 PAT 後重新載入帳號頁
- **THEN** 建立回應含完整明文；重新載入後的清單只含 prefix 與 metadata，無任何途徑再取得明文

#### Scenario: 撤銷即時生效

- **WHEN** 以某 PAT 成功呼叫 API 後於帳號頁撤銷它，再以同一 PAT 呼叫
- **THEN** 撤銷後的呼叫回 401 且 reason 為 permission_denied

---

<!-- @trace
source: server-identity-pat
updated: 2026-07-14
code:
  - Cargo.lock
  - crates/speclink-server/Cargo.toml
  - crates/speclink-server/src/app.rs
  - crates/speclink-server/src/auth.rs
  - crates/speclink-server/src/config.rs
  - crates/speclink-server/src/identity.rs
  - crates/speclink-server/src/identity_sqlite.rs
  - crates/speclink-server/src/lib.rs
  - crates/speclink-server/src/main.rs
  - crates/speclink-server/src/state.rs
  - crates/speclink-server/src/web.rs
  - crates/speclink-server/tests/auth_pat.rs
  - crates/speclink-server/tests/binding.rs
  - crates/speclink-server/tests/command_routes.rs
  - crates/speclink-server/tests/common/mod.rs
  - crates/speclink-server/tests/discussion_routes.rs
  - crates/speclink-server/tests/e2e_cli.rs
  - crates/speclink-server/tests/identity.rs
  - crates/speclink-server/tests/invite.rs
  - crates/speclink-server/tests/query_routes.rs
  - crates/speclink-server/tests/startup.rs
  - crates/speclink-server/tests/sync_state.rs
  - crates/speclink-server/tests/web_account.rs
  - crates/speclink-server/tests/web_invite.rs
-->

---
### Requirement: bearer 驗證逐請求生效且分類明確

API 的 bearer 驗證 SHALL 逐請求對 identity 儲存查驗 PAT：hash 命中、未撤銷、未過期、所屬 user 為 active、且 user 為 URL project 的 member 全數成立才得 actor；成功請求 SHALL 更新該 PAT 的 last-used。停權 user 或移除 membership SHALL 在下一個請求即生效，SHALL NOT 有使失效延後的快取。token 無效、過期、撤銷或 user 停權 SHALL 回 401 permission_denied 且 SHALL NOT 區分原因；token 有效但非該 project 成員 SHALL 回 403 permission_denied。

#### Scenario: 停權即時擋下既有 PAT

- **WHEN** 持有效 PAT 的 user 被標記 suspended 後，以該 PAT 呼叫查詢路由
- **THEN** 回 401 permission_denied；server 未執行任何 engine 動詞

#### Scenario: 非成員回 403

- **WHEN** 以有效 PAT 呼叫 actor 不具 membership 的 project 路由
- **THEN** 回 403 且 reason 為 permission_denied，與 token 無效的 401 可區分

---

<!-- @trace
source: server-identity-pat
updated: 2026-07-14
code:
  - Cargo.lock
  - crates/speclink-server/Cargo.toml
  - crates/speclink-server/src/app.rs
  - crates/speclink-server/src/auth.rs
  - crates/speclink-server/src/config.rs
  - crates/speclink-server/src/identity.rs
  - crates/speclink-server/src/identity_sqlite.rs
  - crates/speclink-server/src/lib.rs
  - crates/speclink-server/src/main.rs
  - crates/speclink-server/src/state.rs
  - crates/speclink-server/src/web.rs
  - crates/speclink-server/tests/auth_pat.rs
  - crates/speclink-server/tests/binding.rs
  - crates/speclink-server/tests/command_routes.rs
  - crates/speclink-server/tests/common/mod.rs
  - crates/speclink-server/tests/discussion_routes.rs
  - crates/speclink-server/tests/e2e_cli.rs
  - crates/speclink-server/tests/identity.rs
  - crates/speclink-server/tests/invite.rs
  - crates/speclink-server/tests/query_routes.rs
  - crates/speclink-server/tests/startup.rs
  - crates/speclink-server/tests/sync_state.rs
  - crates/speclink-server/tests/web_account.rs
  - crates/speclink-server/tests/web_invite.rs
-->

---
### Requirement: identity 儲存獨立且版本守門

identity 資料（users、memberships、invitations、PATs、sessions）SHALL 存於 server 自有的資料庫，與 TeamStore 的資料庫分離，SHALL NOT 寫入 TeamStore driver 的檔案。identity 資料庫 SHALL 記錄 schema version：空資料庫初始化為現行版本；version 較新或 schema 非本 server 所建 SHALL 使啟動失敗並印出原因，SHALL NOT 寫入。密碼、PAT、邀請 token 與 session 識別 SHALL 僅以 hash 落庫，SHALL NOT 出現在 log。

#### Scenario: 陌生 identity 資料庫拒啟動

- **WHEN** 組態的 identity 路徑指向一個由其他應用建立的 SQLite 檔並啟動 server
- **THEN** 啟動失敗、stderr 指出該檔與原因；檔案內容位元不變

#### Scenario: 憑證不落明文

- **WHEN** 完成邀請接受、登入與 PAT 建立後檢視 identity 資料庫內容
- **THEN** 密碼、邀請 token、session 識別與 PAT 欄位皆為 hash 形式，庫內不存在任何憑證明文

<!-- @trace
source: server-identity-pat
updated: 2026-07-14
code:
  - Cargo.lock
  - crates/speclink-server/Cargo.toml
  - crates/speclink-server/src/app.rs
  - crates/speclink-server/src/auth.rs
  - crates/speclink-server/src/config.rs
  - crates/speclink-server/src/identity.rs
  - crates/speclink-server/src/identity_sqlite.rs
  - crates/speclink-server/src/lib.rs
  - crates/speclink-server/src/main.rs
  - crates/speclink-server/src/state.rs
  - crates/speclink-server/src/web.rs
  - crates/speclink-server/tests/auth_pat.rs
  - crates/speclink-server/tests/binding.rs
  - crates/speclink-server/tests/command_routes.rs
  - crates/speclink-server/tests/common/mod.rs
  - crates/speclink-server/tests/discussion_routes.rs
  - crates/speclink-server/tests/e2e_cli.rs
  - crates/speclink-server/tests/identity.rs
  - crates/speclink-server/tests/invite.rs
  - crates/speclink-server/tests/query_routes.rs
  - crates/speclink-server/tests/startup.rs
  - crates/speclink-server/tests/sync_state.rs
  - crates/speclink-server/tests/web_account.rs
  - crates/speclink-server/tests/web_invite.rs
-->
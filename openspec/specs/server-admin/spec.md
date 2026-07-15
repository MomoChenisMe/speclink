# server-admin Specification

## Purpose

TBD - created by archiving change 'server-admin-audit'. Update Purpose after archive.

## Requirements

### Requirement: admin 門禁前置且非 admin 一律 403

admin 路由（/admin 頁面與 admin API）SHALL 在既有認證（session 或 bearer）成功後檢查使用者的 admin 旗標，非 admin SHALL 回 403 permission_denied，SHALL NOT 新增 wire reason。admin API SHALL 套用既有 API version 檢查；/admin 頁面 SHALL 沿用 session cookie 與 POST 同源驗證。被停權的 admin SHALL 在下一請求即失去管理面通行。

#### Scenario: 一般成員不可入管理面

- **WHEN** 無 admin 旗標的登入使用者訪問 /admin 頁面，與以其 PAT 呼叫 admin API
- **THEN** 兩者皆回 403 permission_denied；不執行任何管理動作

#### Scenario: 停權 admin 即時失效

- **WHEN** admin A 停權 admin B 後，B 以既有 session 訪問 /admin
- **THEN** B 被視同未授權；不能執行管理動作

---

<!-- @trace
source: server-admin-audit
updated: 2026-07-15
code:
  - crates/speclink-server/src/admin.rs
  - crates/speclink-server/src/app.rs
  - crates/speclink-server/src/audit.rs
  - crates/speclink-server/src/auth.rs
  - crates/speclink-server/src/error.rs
  - crates/speclink-server/src/identity.rs
  - crates/speclink-server/src/identity_sqlite.rs
  - crates/speclink-server/src/lib.rs
  - crates/speclink-server/src/main.rs
  - crates/speclink-server/src/setup.rs
  - crates/speclink-server/tests/admin_api.rs
  - crates/speclink-server/tests/admin_e2e.rs
  - crates/speclink-server/tests/admin_pages.rs
  - crates/speclink-server/tests/admin_system.rs
  - crates/speclink-server/tests/admin_three_entry.rs
  - crates/speclink-server/tests/audit.rs
  - crates/speclink-server/tests/cli_admin.rs
  - crates/speclink-server/tests/identity.rs
-->

---
### Requirement: 管理動作三入口同一實作且功能完備

每個管理動作 SHALL 為單點實作，admin API、/admin 表單與 server CLI 子命令 SHALL 呼叫同一路徑。功能集 SHALL 涵蓋：使用者列表與邀請、停權/復權、membership 與 admin 旗標調整、registry 的 project/repo 建立與顯示名變更（key SHALL NOT 可改）、全站憑證 metadata 檢視與強制撤銷。headless 部署 SHALL 能以 CLI 子命令完成停權/復權、token 撤銷與 registry 建立。停權最後一位 active admin SHALL 被拒絕並明示原因。/admin SHALL NOT 提供任何規格內容（changes、specs、discussions）的檢視或編輯。

#### Scenario: 三入口等效停權

- **WHEN** 分別經 admin API、/admin 表單與 CLI 子命令停權三個不同使用者
- **THEN** 三者的下一個 API 請求皆 401；三筆動作皆入 audit（來源分別為 api、web、cli）

#### Scenario: 最後一位 admin 不可自斷

- **WHEN** 全站僅剩一位 active admin 時嘗試停權該 admin
- **THEN** 動作被拒絕且原因明示；該 admin 仍可通行

#### Scenario: registry key 不可改

- **WHEN** 嘗試變更既有 project 的顯示名與 key
- **THEN** 顯示名變更成功；key 無變更介面，binding 以原 key 照常運作

---

<!-- @trace
source: server-admin-audit
updated: 2026-07-15
code:
  - crates/speclink-server/src/admin.rs
  - crates/speclink-server/src/app.rs
  - crates/speclink-server/src/audit.rs
  - crates/speclink-server/src/auth.rs
  - crates/speclink-server/src/error.rs
  - crates/speclink-server/src/identity.rs
  - crates/speclink-server/src/identity_sqlite.rs
  - crates/speclink-server/src/lib.rs
  - crates/speclink-server/src/main.rs
  - crates/speclink-server/src/setup.rs
  - crates/speclink-server/tests/admin_api.rs
  - crates/speclink-server/tests/admin_e2e.rs
  - crates/speclink-server/tests/admin_pages.rs
  - crates/speclink-server/tests/admin_system.rs
  - crates/speclink-server/tests/admin_three_entry.rs
  - crates/speclink-server/tests/audit.rs
  - crates/speclink-server/tests/cli_admin.rs
  - crates/speclink-server/tests/identity.rs
-->

---
### Requirement: 憑證監督不可讀回明文

admin 的憑證檢視 SHALL 僅含 metadata：所屬使用者、prefix、名稱、到期、last-used 與建立時間；SHALL NOT 存在讀回 PAT、access token、refresh credential 明文或 hash 的介面。強制撤銷 SHALL 與自助撤銷同一即時生效語意，並記 audit（含操作者與 token 識別，SHALL NOT 記祕密值）。

#### Scenario: 強制撤銷即時且留痕

- **WHEN** admin 於憑證頁強制撤銷某成員的 PAT
- **THEN** 該 PAT 的下一次使用回 401；audit 含一筆 token-revoked 記錄，記 token id 與 prefix 而無 hash 或明文

---

<!-- @trace
source: server-admin-audit
updated: 2026-07-15
code:
  - crates/speclink-server/src/admin.rs
  - crates/speclink-server/src/app.rs
  - crates/speclink-server/src/audit.rs
  - crates/speclink-server/src/auth.rs
  - crates/speclink-server/src/error.rs
  - crates/speclink-server/src/identity.rs
  - crates/speclink-server/src/identity_sqlite.rs
  - crates/speclink-server/src/lib.rs
  - crates/speclink-server/src/main.rs
  - crates/speclink-server/src/setup.rs
  - crates/speclink-server/tests/admin_api.rs
  - crates/speclink-server/tests/admin_e2e.rs
  - crates/speclink-server/tests/admin_pages.rs
  - crates/speclink-server/tests/admin_system.rs
  - crates/speclink-server/tests/admin_three_entry.rs
  - crates/speclink-server/tests/audit.rs
  - crates/speclink-server/tests/cli_admin.rs
  - crates/speclink-server/tests/identity.rs
-->

---
### Requirement: audit log 只增不改且動作全覆蓋

identity 資料庫 SHALL 含 audit 表（schema 演進沿用既有守門），每筆記錄 SHALL 含操作者、封閉集合的動作種類、對象識別、UTC 時間與來源（web、api、cli）。全部變更型管理動作（含 invite 子命令與 setup 流程的建立動作）SHALL 恰寫一筆 audit，與資料變更同 transaction；SHALL NOT 存在更新或刪除 audit 記錄的介面。/admin 的 audit 檢視 SHALL 唯讀倒序；一般使用者 SHALL NOT 可見。

#### Scenario: 管理動作皆留痕

- **WHEN** 依序執行邀請、membership 調整、project 建立、token 撤銷各一筆後開啟 audit 頁
- **THEN** 四筆記錄倒序在列，動作種類、對象與來源正確；無任何編輯或刪除控制

#### Scenario: audit 與動作同生死

- **WHEN** 某管理動作因資料層錯誤失敗
- **THEN** audit 無該動作的記錄——不存在「動作成功無 audit」或「audit 存在動作未生效」的組合

---

<!-- @trace
source: server-admin-audit
updated: 2026-07-15
code:
  - crates/speclink-server/src/admin.rs
  - crates/speclink-server/src/app.rs
  - crates/speclink-server/src/audit.rs
  - crates/speclink-server/src/auth.rs
  - crates/speclink-server/src/error.rs
  - crates/speclink-server/src/identity.rs
  - crates/speclink-server/src/identity_sqlite.rs
  - crates/speclink-server/src/lib.rs
  - crates/speclink-server/src/main.rs
  - crates/speclink-server/src/setup.rs
  - crates/speclink-server/tests/admin_api.rs
  - crates/speclink-server/tests/admin_e2e.rs
  - crates/speclink-server/tests/admin_pages.rs
  - crates/speclink-server/tests/admin_system.rs
  - crates/speclink-server/tests/admin_three_entry.rs
  - crates/speclink-server/tests/audit.rs
  - crates/speclink-server/tests/cli_admin.rs
  - crates/speclink-server/tests/identity.rs
-->

---
### Requirement: 系統資訊唯讀聚合

admin 的系統狀態檢視 SHALL 唯讀聚合：engine 與 API 版本、store manifest（driver、contract version、capabilities、等級）、store health 即時結果、identity schema version、每個 registry scope 的 outbox 積壓量。store 失聯時 SHALL 如實顯示 health 失敗，SHALL NOT 使頁面整體失效；identity 庫的管理功能照常。

#### Scenario: store 失聯不癱管理面

- **WHEN** store 後端不可用時開啟系統狀態頁並執行一筆使用者停權
- **THEN** 頁面顯示 store health 失敗與可得的其餘資訊；停權照常成功且入 audit

<!-- @trace
source: server-admin-audit
updated: 2026-07-15
code:
  - crates/speclink-server/src/admin.rs
  - crates/speclink-server/src/app.rs
  - crates/speclink-server/src/audit.rs
  - crates/speclink-server/src/auth.rs
  - crates/speclink-server/src/error.rs
  - crates/speclink-server/src/identity.rs
  - crates/speclink-server/src/identity_sqlite.rs
  - crates/speclink-server/src/lib.rs
  - crates/speclink-server/src/main.rs
  - crates/speclink-server/src/setup.rs
  - crates/speclink-server/tests/admin_api.rs
  - crates/speclink-server/tests/admin_e2e.rs
  - crates/speclink-server/tests/admin_pages.rs
  - crates/speclink-server/tests/admin_system.rs
  - crates/speclink-server/tests/admin_three_entry.rs
  - crates/speclink-server/tests/audit.rs
  - crates/speclink-server/tests/cli_admin.rs
  - crates/speclink-server/tests/identity.rs
-->
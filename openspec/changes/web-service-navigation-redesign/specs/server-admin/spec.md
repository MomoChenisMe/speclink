## MODIFIED Requirements

### Requirement: admin 門禁前置且非 admin 一律 403
<!-- BEFORE: admin 頁面以 session cookie 與同源 POST 表單存取，外部 admin API 使用既有認證。 -->

admin browser route 與 `/api/speclink/v1/web/admin/*` SHALL 在 active session 認證成功後檢查使用者的 admin 旗標；未登入 browser API SHALL 回 401，已登入但非 admin SHALL 回 403 `permission_denied`，SHALL NOT 新增 wire reason。全部 browser mutation SHALL 在 session 與 admin 檢查前驗證 Origin 或 Referer 同源；既有 bearer admin API SHALL 繼續套用 API version、bearer 與 admin 檢查，SHALL NOT 接受 session cookie 取代 bearer。被停權的 admin SHALL 在下一請求即失去管理面通行。

#### Scenario: 一般成員不可入管理面

- **WHEN** 無 admin 旗標的登入使用者訪問 `/admin` 並呼叫 browser admin API，另以其 PAT 呼叫 bearer admin API
- **THEN** SPA 呈現無權限狀態，兩個 API 皆回 403 `permission_denied`，且不執行任何管理動作

#### Scenario: 停權 admin 即時失效

- **WHEN** admin A 停權 admin B 後，B 以既有 session 呼叫 browser admin API
- **THEN** B 被視同未授權並收到 401；不能讀取或執行管理動作

#### Scenario: 跨 origin mutation 在權限裁決前拒絕

- **WHEN** 已登入 admin 從不同 origin 提交 browser admin mutation
- **THEN** Server 回 403 且不執行管理動作、不新增成功 audit event

### Requirement: 管理動作三入口同一實作且功能完備
<!-- BEFORE: admin API、server-rendered `/admin` 表單與 server CLI 子命令呼叫同一管理實作。 -->

每個管理動作 SHALL 為單點實作，既有 bearer admin API、SPA 使用的 browser admin API 與 server CLI 子命令 SHALL 呼叫同一路徑。功能集 SHALL 涵蓋：使用者列表與邀請、停權／復權、membership 與 admin 旗標調整、registry 的 project／repo 建立與顯示名變更（key SHALL NOT 可改）、全站憑證 metadata 檢視與強制撤銷。headless 部署 SHALL 能以 CLI 子命令完成停權／復權、token 撤銷與 registry 建立。停權最後一位 active admin SHALL 被拒絕並明示原因。管理 SPA SHALL NOT 提供任何規格內容（changes、specs、discussions）的檢視或編輯。

#### Scenario: 三入口等效停權

- **WHEN** 分別經 bearer admin API、browser admin API 與 CLI 子命令停權三個不同使用者
- **THEN** 三者的下一個 API 請求皆 401；三筆動作皆入 audit，來源分別為 `api`、`web`、`cli`

#### Scenario: 最後一位 admin 不可自斷

- **WHEN** 全站僅剩一位 active admin 時經任一入口嘗試停權該 admin
- **THEN** 動作被拒絕且原因明示；該 admin 仍可通行

#### Scenario: registry key 不可改

- **WHEN** 管理員在 SPA 嘗試變更既有 project 的顯示名與 key
- **THEN** 顯示名可變更；UI 與 browser API 均無 key 變更操作，binding 以原 key 照常運作

## ADDED Requirements

### Requirement: 管理 browser API 提供最小且完整的頁面 view model

`/api/speclink/v1/web/admin` SHALL 提供總覽、users、registry、credentials、data、system 與 audit 的獨立讀取操作。Overview SHALL 回 active／suspended user 數、project／repo 數、active credential 數、store health、identity schema version 與 setup welcome connection metadata；清單 SHALL 回穩定 id、顯示欄位與 action eligibility。回應 SHALL NOT 包含 PAT hash、PAT plaintext、password hash、refresh credential、setup token 或 invite token。Store health 失敗時，overview、system 與 data SHALL 回傳仍可取得的 identity 資料、`storeHealthy: false` 與可公開的 `storeHealthError`；users 與 credentials 管理 SHALL 保持可用。

#### Scenario: 管理導覽各頁獨立載入

- **WHEN** admin 依序開啟 users、registry、credentials、data、system 與 audit route
- **THEN** 每個 route 只呼叫對應 view-model API 並呈現頁面所需欄位，不取得祕密值

#### Scenario: Store 不健康時 identity 管理仍可用

- **WHEN** TeamStore health check 失敗但 identity store 可讀
- **THEN** overview 明確顯示 `storeHealthy: false`，users 與 credentials API 仍成功，data 與 system 呈現可得資料與可公開錯誤

#### Scenario: 清單回傳 action eligibility

- **WHEN** admin 讀取包含最後一位 active admin 的 users view model
- **THEN** 該使用者項目明確標示不可停權或移除 admin 旗標，且 server mutation 仍獨立執行相同安全檢查

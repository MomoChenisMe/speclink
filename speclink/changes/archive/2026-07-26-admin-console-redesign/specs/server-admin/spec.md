## MODIFIED Requirements

### Requirement: 管理 browser API 提供最小且完整的頁面 view model

<!-- BEFORE: data 與 system 各自提供獨立 view model；overview 僅回計數與 store health；audit 不接受篩選與分頁參數 -->

`/api/speclink/v1/web/admin` SHALL 提供總覽、users、registry、credentials、system 與 audit 的獨立讀取操作，SHALL NOT 保留獨立的 data view model。System view model SHALL 於單次回應內同時提供引擎與 API 版本、identity schema version、store 驅動、契約版本、等級、能力、store health、outbox backlog、可匯出 scope 清單與資料結構遷移可用性。Overview SHALL 回 active／suspended user 數、project／repo 數、active credential 數、待啟用邀請數、store health、identity schema version、setup welcome connection metadata、待處理事項清單與最近稽核事件清單；每則待處理事項 SHALL 標示其類型與對應目的地。Audit view model SHALL 接受關鍵字、動作、來源、時間區間與頁碼參數，於伺服器端套用後回傳當頁事件與總頁數；頁碼小於 1 或時間區間起始晚於結束 SHALL 回 400 `invalid_argument`，頁碼超出總頁數 SHALL 回空事件清單與正確總頁數。清單 SHALL 回穩定 id、顯示欄位與 action eligibility。回應 SHALL NOT 包含 PAT hash、PAT plaintext、password hash、refresh credential、setup token 或 invite token。Store health 失敗時，overview 與 system SHALL 回傳仍可取得的 identity 資料、`storeHealthy: false` 與可公開的 `storeHealthError`；users 與 credentials 管理 SHALL 保持可用。既有欄位名稱 SHALL 維持不變，新增欄位 SHALL 以 camelCase 輸出。

#### Scenario: 管理導覽各頁獨立載入

- **WHEN** admin 依序開啟 users、registry、credentials、system 與 audit route
- **THEN** 每個 route 只呼叫對應 view-model API 並呈現頁面所需欄位，不取得祕密值

#### Scenario: 系統頁單次回應涵蓋四組資料

- **WHEN** admin 開啟 system route
- **THEN** 單次 view-model 回應同時包含執行環境版本、store 狀態與 outbox backlog、可匯出 scope 清單與遷移可用性，SPA 不再呼叫第二支 API 取得其中任何一組

#### Scenario: 總覽回傳待處理事項與最近稽核

- **WHEN** admin 開啟 overview route 且系統沒有任何 active credential
- **THEN** overview 回傳標示該事項與對應目的地的待處理項目，並回傳最近稽核事件清單

#### Scenario: 稽核篩選與分頁由伺服器套用

- **WHEN** client 以動作篩選與第二頁頁碼呼叫 audit view model
- **THEN** 回應只含符合該篩選的當頁事件與總頁數，未符合篩選的事件不出現在回應中

##### Example: 篩選與分頁組合

- **GIVEN** 稽核事件依時間新到舊為 E1(user.invite)、E2(project.create)、E3(user.invite)、E4(user.suspend)、E5(user.invite)，每頁 2 筆
- **WHEN** client 以動作篩選 `user.invite` 與頁碼 2 呼叫 audit view model
- **THEN** 回應事件為 E5、總頁數為 2；E2 與 E4 不出現在回應中

##### Example: 參數邊界

| 參數 | 值 | 預期 |
|------|----|------|
| 頁碼 | 0 或負數 | 回 400 `invalid_argument`，不回傳事件 |
| 頁碼 | 大於總頁數 | 回空事件清單與正確總頁數，不回 404 |
| 動作篩選 | 未知動作名稱 | 回空事件清單與總頁數 0 |
| 時間區間 | 起始晚於結束 | 回 400 `invalid_argument` |
| 全部篩選 | 省略 | 回第一頁全部事件與總頁數 |

#### Scenario: Store 不健康時 identity 管理仍可用

- **WHEN** TeamStore health check 失敗但 identity store 可讀
- **THEN** overview 明確顯示 `storeHealthy: false`，users 與 credentials API 仍成功，system 呈現可得資料與可公開錯誤

#### Scenario: 清單回傳 action eligibility

- **WHEN** admin 讀取包含最後一位 active admin 的 users view model
- **THEN** 該使用者項目明確標示不可停權或移除 admin 旗標，且 server mutation 仍獨立執行相同安全檢查

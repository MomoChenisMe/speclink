## ADDED Requirements

### Requirement: 認領的識別與時間欄位及釋放回應

變更清單（GET /changes 的每一項）與單 change 讀取回應（GET /changes/{name}）SHALL 在既有的 `claimedBy` 之外增列兩個選填欄位：`claimedById`（字串，meta 的 claimed_by_id）與 `claimedAt`（字串，meta 的 claimed_at 原樣）。兩者 SHALL 只在 meta 有對應的值時出現；沒有被認領的 change SHALL 三個欄位都不出現。既有欄位的名稱、型別與出現條件 SHALL 不變。

釋放認領的回應（DELETE /changes/{name}/claim）SHALL 是一個物件：`released`（布林，必填）與 `previousHolder`（字串，選填，釋放前的 claimed_by）。認領的回應（POST /changes/{name}/claim）SHALL 不變。

這些型別 SHALL 定義在 protocol 型別裡並出現在匯出的 JSON Schema。舊版用戶端遇到新欄位 SHALL 照常解析（忽略不認得的欄位）；新版用戶端讀不到 `claimedById` 或 `claimedAt` 時 SHALL 把它當成沒有記錄，SHALL NOT 失敗。

#### Scenario: 帶識別的認領出現在兩個讀取回應

- **WHEN** change demo 的 meta 有 claimed_by "Alice"、claimed_by_id "u_1"、claimed_at "2026-11-18T10:00:00+00:00"，呼叫 GET /changes 與 GET /changes/demo
- **THEN** 兩個回應的 demo 都含 `claimedBy: "Alice"`、`claimedById: "u_1"`、`claimedAt: "2026-11-18T10:00:00+00:00"`

#### Scenario: 沒有識別時不出現 claimedById

- **WHEN** change demo 的 meta 有 claimed_by 與 claimed_at，沒有 claimed_by_id，呼叫 GET /changes/demo
- **THEN** 回應含 `claimedBy` 與 `claimedAt`，沒有 `claimedById` 這個鍵

#### Scenario: 沒有被認領時三個欄位都不出現

- **WHEN** 對一個沒有認領欄位的 change 呼叫 GET /changes/{name}
- **THEN** 回應沒有 `claimedBy`、`claimedById`、`claimedAt`，與引入新欄位之前逐位元相同

#### Scenario: 新版用戶端讀舊版 Server 的回應

- **WHEN** 新版用戶端解析一個只有 `claimedBy`、沒有另外兩個欄位的單 change 讀取回應
- **THEN** 解析成功，識別與時間視為沒有記錄

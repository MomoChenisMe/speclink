## ADDED Requirements

### Requirement: claim 釋放端點

DELETE /changes/{name}/claim SHALL 經 Command gateway 直通引擎的釋放認領命令。成功時 SHALL 回 HTTP 200，回應含 `released`（布林）與選填的 `previousHolder`（字串，釋放前的持有人）；移除認領的寫入 SHALL 隨 Unit of Work 落盤。change 沒有被認領時 SHALL 回 200、`released` 為 false、零寫入。change 不存在 SHALL 回 404。本端點 SHALL 比照其他寫入動詞為 editor 限定（reader 收 403、scope 零改動）；editor SHALL 不必是認領的人。宿主的執行前掛鉤拒絕時 SHALL 回既有的拒絕回應（reason 為 refused、message 是宿主給的原因），SHALL NOT 擴充 error reason registry。釋放之後，變更清單與單 change 讀取回應 SHALL 不再含該 change 的 claimedBy、claimedById 與 claimedAt。

請求帶的 revision 前置條件不符時 SHALL 沿既有的樂觀並行規則回 409；認證失效 SHALL 回 401。

#### Scenario: 釋放落盤且讀取回應不再有認領人

- **WHEN** change demo 已被 Alice 認領，另一位 editor Bob 呼叫 DELETE /changes/demo/claim，隨後呼叫 GET /changes 與 GET /changes/demo
- **THEN** 釋放的回應是 200，含 `released: true` 與 `previousHolder: "Alice"`；兩個讀取回應的 demo 都沒有 claimedBy、claimedById、claimedAt；server 重啟後讀取結果不變

#### Scenario: 沒有被認領與 reader 拒絕

- **WHEN** 一位 editor 對未認領的 change 呼叫本端點，接著一位 reader 對已認領的 change 呼叫本端點
- **THEN** 前者收 200、`released: false`、沒有 `previousHolder`、scope 零改動；後者收 403、認領欄位不變

#### Scenario: 變更不存在

- **WHEN** editor 呼叫 DELETE /changes/ghost/claim，而沒有名為 ghost 的 change
- **THEN** 回 404，scope 零改動

#### Scenario: 掛鉤拒絕釋放

- **WHEN** 宿主的執行前掛鉤對命令 `claim-release` 回答拒絕、原因「只有認領的人可以釋放」，editor 呼叫本端點
- **THEN** 回應的 reason 是 refused、message 是「只有認領的人可以釋放」，認領欄位不變

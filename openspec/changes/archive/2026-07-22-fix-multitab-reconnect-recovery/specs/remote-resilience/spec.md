## MODIFIED Requirements

### Requirement: 恢復自動收斂並清除 stale
<!-- BEFORE: server 恢復後由事件 worker 自動重連並全量重查，但未規範同來源多 session 同時換發 credential 的協調方式。 -->

server 恢復可達後 SHALL 全自動：事件 worker 以既有 Polling 加 ETag 收斂機制重連，runtime 回 online 並發全量失效通知，store 全量重查後清除 stale 標示——SHALL NOT 要求使用者手動重整或任何操作。同一 connection 的多個 remote session 或 worker 同時需要認證恢復時，Desktop SHALL 讓同一時刻最多一個呼叫消耗已儲存的 refresh credential，其他呼叫 SHALL 共用該次成功換發的 access token 與已輪替 credential，再各自重試原讀取；本機併發 SHALL NOT 被伺服器視為舊 credential 重放，SHALL NOT 因而撤銷 credential family 或進入 `needs-reauth`。伺服器明確拒絕已撤銷、失效或真正遭重放的 credential 時，Desktop SHALL 維持既有 `needs-reauth` 行為。

#### Scenario: server 重啟後自動復原

- **WHEN** offline 期間另一 client 於同 scope 建立新 change，隨後 server 恢復
- **THEN** 分頁自動回 online、stale 標示消失，看板含恢復期間的新 change，全程無使用者操作

#### Scenario: 同來源多分頁併發恢復只輪替一次

- **WHEN** 同一 connection 的兩個 remote 分頁在 server 恢復後同時以失效 access token 發出讀取，且 Keychain 中只有同一枚可用 refresh credential
- **THEN** Desktop 只讓一個 refresh 請求消耗該 credential，兩個分頁共用成功結果後自動回 online，credential family 維持有效且全程不呈現 `needs-reauth`

#### Scenario: 明確撤銷仍進入重新驗證

- **WHEN** server 已明確撤銷該 connection 的 credential family，任一 remote session 嘗試恢復
- **THEN** Desktop 進入 `needs-reauth` 並提供既有重新登入路徑，不持續重試被拒絕的 credential

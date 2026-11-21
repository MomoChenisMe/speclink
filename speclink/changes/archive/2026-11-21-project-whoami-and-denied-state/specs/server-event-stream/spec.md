## ADDED Requirements

### Requirement: 使用者失去權限時關閉串流

示範用 Server 的事件串流在開著的期間，SHALL 每隔一個心跳間隔（設定檔的 `events.heartbeat_secs`，預設 15 秒）確認訂閱的使用者仍是可用的帳號，且仍是該 project 的成員。任一項不成立時 Server SHALL 結束該串流（關閉連線），結束之後 SHALL NOT 再對它送出任何事件。之後以同一憑證重新訂閱 SHALL 依既有的認證與成員資格規則回應（不是成員為 403 permission_denied）。

這項確認 SHALL 只看帳號狀態與成員資格，SHALL NOT 因訂閱時使用的 access token 到期而結束串流。確認時身分儲存無法讀取 SHALL NOT 結束串流，Server SHALL 於下一個間隔再確認。帳號與成員資格沒有變動的串流 SHALL NOT 因這項確認而中斷，事件 SHALL 維持不漏不重。

自行提供串流的宿主 SHALL 由宿主自己在使用者失去權限時關閉串流；宿主套件不提供串流，因此不代為關閉。使用者文件 SHALL 寫明兩件事：宿主要在使用者失去權限時關閉自己的串流；用戶端在串流被關閉後會重新訂閱並輪詢，兩者都被拒時 Speclink Desktop 顯示「沒有存取權限」。

#### Scenario: 成員資格被移除後串流結束

- **WHEN** 成員 alice 訂閱 project demo 的事件串流，之後管理者把 alice 移出 demo
- **THEN** 在一個心跳間隔內 Server 結束這條串流；alice 以同一憑證重新訂閱時回 403 且 reason 為 permission_denied

#### Scenario: 帳號被停用後串流結束

- **WHEN** 成員 alice 訂閱事件串流，之後管理者停用 alice 的帳號
- **THEN** 在一個心跳間隔內 Server 結束這條串流

#### Scenario: 權限沒有變動的串流不中斷

- **WHEN** 成員 bob 訂閱事件串流並維持三個心跳間隔以上，期間另一個請求建立一個變更
- **THEN** 串流沒有被 Server 結束，bob 收到該變更的 invalidate 事件恰一次

#### Scenario: 身分儲存暫時無法讀取

- **WHEN** 某一次確認時身分儲存回報錯誤，下一次確認時恢復，使用者仍是成員
- **THEN** 串流沒有被結束

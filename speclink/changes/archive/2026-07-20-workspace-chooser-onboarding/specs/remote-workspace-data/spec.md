## MODIFIED Requirements

### Requirement: handshake 成功後才建立 remote session

remote session SHALL 僅在 binding handshake 成功後建立：開啟入口（chooser 的 scopes 清單選擇或 remote marker 探測）以選定的 repo 發起 handshake，成功時以回應中的 project/repo 識別建構 remote locator 與分頁；失敗（未授權、不存在、多義）SHALL 原樣呈現 server 錯誤且 SHALL NOT 建立分頁或 session——scopes 清單雖經 membership 過濾，選擇與 handshake 之間權限可能變化，handshake 仍為最終防線。重啟後恢復 remote 分頁 SHALL 重走 handshake，失敗時該分頁 SHALL 呈現需重新認證或錯誤狀態、SHALL NOT 靜默消失或退回本地模式。

#### Scenario: handshake 失敗不建分頁

- **WHEN** 於 scopes 清單選定 repo 後、handshake 前該使用者的 membership 被撤銷
- **THEN** handshake 被拒並原樣呈現 server 錯誤，分頁列不出現新分頁

#### Scenario: 重啟後 remote 分頁恢復需重驗

- **WHEN** 含 remote 分頁的 app 重啟且 credential 仍有效
- **THEN** 該分頁重走 handshake 後恢復呈現；credential 失效時分頁呈現需重新認證狀態而非消失

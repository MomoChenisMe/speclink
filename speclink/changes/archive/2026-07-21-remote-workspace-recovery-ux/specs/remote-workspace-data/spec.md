## MODIFIED Requirements

### Requirement: handshake 成功後才建立 remote session
<!-- BEFORE: 重啟恢復 handshake 失敗只讓分頁呈現錯誤，未規範分頁可選取、loading、復原頁與 retry。 -->

remote session SHALL 僅在 binding handshake 成功後建立：新開啟入口（chooser 的 scopes 清單選擇或 remote marker 探測）以選定的 repo 發起 handshake，成功時以回應中的 project/repo 識別建構 remote locator 與分頁；失敗（未授權、不存在、多義）SHALL 原樣提供 technical detail 且 SHALL NOT 建立分頁或 session。scopes 清單雖經 membership 過濾，選擇與 handshake 之間權限可能變化，handshake 仍為最終防線。

重啟後恢復或選取既有 remote 分頁 SHALL 重走 handshake：分頁須先成為作用中並呈 restoring；成功 SHALL 於同一 locator key 原地建立 session，失敗 SHALL 保留作用中分頁並呈現 error 復原頁與 retry／設定或重新登入動作，SHALL NOT 靜默消失、退回本地模式或顯示上一 workspace 資料。retry SHALL 重走相同 handshake 前置，成功前 SHALL NOT 建立 session。

#### Scenario: 新開啟入口 handshake 失敗不建分頁

- **WHEN** 於 scopes 清單選定 repo 後、handshake 前該使用者的 membership 被撤銷
- **THEN** handshake 被拒並於開啟入口呈現錯誤，分頁列不出現新分頁，session 清單不新增項目

#### Scenario: 重啟後 remote 分頁恢復成功

- **WHEN** 含 remote 分頁的 app 重啟且 credential 與 scope 仍有效
- **THEN** 該分頁先呈 restoring，handshake 成功後於原位恢復 server 資料，分頁列不新增重複項目

#### Scenario: 重啟後 credential 失效進入復原頁

- **WHEN** 含 remote 分頁的 app 重啟，而該 connection 的 credential 已失效
- **THEN** 該分頁保持存在且成為作用中，呈現需要重新認證的復原頁與對應動作，不顯示上一 workspace 資料

#### Scenario: server 不可達時 retry 原地恢復

- **WHEN** 重啟恢復因 server 不可達而進入 error 復原頁，server 恢復後使用者選擇重新連線
- **THEN** 同一分頁呈 restoring 並重走 handshake，成功後原地建立 session、清除 error 且顯示 server 資料

## ADDED Requirements

### Requirement: remote_open 失敗保留 machine-readable reason

Desktop 的 remote_open 邊界失敗時 SHALL 提供 camelCase 的 message、reason、status 欄位：message 為 technical detail 字串，reason 為 protocol reason 字串或 null，status 為 HTTP status 整數或 null。Desktop SHALL 依 reason／status 與 Rust runtime 狀態正規化為 unreachable、needs-reauth、access-denied、not-found、unknown 五種封閉復原分類；UI 摘要 SHALL 由繁體中文 i18n 文案產生，SHALL NOT 以英文 message 比對分類。失敗 payload SHALL NOT 含 access token、refresh credential、PAT、authorization header 或 Keychain 內容；server HTTP API 與成功 payload SHALL 維持不變。

#### Scenario: transport failure 分類為 unreachable

- **WHEN** remote_open 在取得 HTTP response 前因 server 不可達而失敗
- **THEN** failure status 為 null 且 Desktop 呈 unreachable 摘要、重新連線與伺服器設定動作，technical detail 可由使用者展開

#### Scenario: HTTP status 對應復原分類

- **WHEN** remote_open 分別回傳 401、403、404
- **THEN** Desktop 分別呈 needs-reauth、access-denied、not-found 復原分類，不以 message 文字判斷

##### Example: status 對應

| status | recovery kind | 主要復原方向 |
| ------ | ------------- | ------------ |
| 401 | needs-reauth | 重新登入 |
| 403 | access-denied | 檢查帳號或伺服器設定 |
| 404 | not-found | 檢查 workspace 或移除分頁 |
| null transport | unreachable | 重新連線 |

#### Scenario: 無法解析的 rejection 安全降階

- **WHEN** 測試 adapter 或舊版邊界拒絕一個不符合 structured payload 的字串或未知物件
- **THEN** Desktop 呈 unknown 復原分類並保留可展開 technical detail，app 不崩潰且仍可 retry

#### Scenario: 失敗 payload 不洩漏 credential

- **WHEN** remote_open 因認證失敗回傳 structured rejection
- **THEN** payload 只含 message、reason、status，不含 token、PAT、refresh credential、authorization header 或 Keychain 值

## ADDED Requirements

### Requirement: handshake 成功後才建立 remote session

remote session SHALL 僅在 binding handshake 成功後建立：開啟入口以 repo 識別發起 handshake，成功時以回應中的 project/repo 識別建構 remote locator 與分頁；失敗（未授權、不存在、多義）SHALL 原樣呈現 server 錯誤且 SHALL NOT 建立分頁或 session。重啟後恢復 remote 分頁 SHALL 重走 handshake，失敗時該分頁 SHALL 呈現需重新認證或錯誤狀態、SHALL NOT 靜默消失或退回本地模式。

#### Scenario: handshake 失敗不建分頁

- **WHEN** 於開啟入口輸入無權限的 repo 識別
- **THEN** 呈現 server 的拒絕訊息，分頁列不出現新分頁

#### Scenario: 重啟後 remote 分頁恢復需重驗

- **WHEN** 含 remote 分頁的 app 重啟且 credential 仍有效
- **THEN** 該分頁重走 handshake 後恢復呈現；credential 失效時分頁呈現需重新認證狀態而非消失

### Requirement: Query 加 ETag 為重讀正典且 push 只做 invalidate

remote session 的資料讀取 SHALL 一律走 Query（清單、文件、狀態）；SSE 事件 SHALL 只作為失效提示觸發重讀，SHALL NOT 攜帶被消費的資料實體。同一 server 的多個 session SHALL 共用單一 SSE 訂閱，失效提示 SHALL 以 locator 對應分發。

#### Scenario: server 側變更經 invalidate 反映

- **WHEN** remote 分頁開啟期間，另一 client 於同 repo 建立新 change
- **THEN** 桌面收到失效提示後重新查詢，看板數秒內出現該 change

### Requirement: 斷線以 Polling 加 ETag 收斂後續訂

SSE 中斷時 Desktop SHALL：停流、以 /sync-state 的 ETag 比對偵測錯過的變更（不同即觸發重載）、以退避重連並帶 Last-Event-ID 續傳；server 回 reset 信號時 SHALL 觸發全量重載後自新事件位點續訂。SSE 持續不可用期間 SHALL 以輪詢維持收斂；恢復後 SHALL 回到事件驅動。全程 SHALL NOT 產生資料遺漏——完全漏掉 push 事件後仍能經 Query 收斂到 server 現況。

#### Scenario: server 重啟後自動收斂

- **WHEN** remote 分頁開啟期間 server 程序重啟，期間該 repo 發生過變更
- **THEN** Desktop 於重連後（經 ETag 比對或 reset 信號）重載至 server 現況，錯過的變更全部反映，無需使用者操作

### Requirement: capability 驅動停用且不偽造缺口

RemoteDataSource SHALL 附帶逐操作的 capability 描述（來源＝handshake 回應與端點覆蓋矩陣）；server 無對應端點的操作（封存瀏覽、全文搜尋、正典 spec 內文、validate/analyze 動詞、刪除變更、任務拖排、看板拖排）SHALL 於 UI 停用並附繁體中文說明，對應 DataSource 方法 SHALL 回拒絕錯誤；SHALL NOT 於 client 端偽造或近似實作缺口。本地 session 的全部操作 SHALL 維持可用且行為零改動。批次任務操作以逐任務寫回組合時，中途失敗 SHALL 中止並回報已完成筆數。

#### Scenario: 不支援操作呈現停用

- **WHEN** 於 remote 分頁開啟看板與 archived 頁
- **THEN** 刪除、拖排與搜尋呈現停用附說明，archived 頁呈現 server 尚未提供的提示卡；同時本地分頁上述操作照常可用

### Requirement: token 換發全程 Rust 側且 401 語意固定

remote 請求的 access token SHALL 僅存於 Rust 記憶體；請求遇 401 SHALL 以 Keychain 的 refresh credential 換發一次並重試一次，rotation 後的新 refresh credential SHALL 立即回寫 Keychain；再失敗 SHALL 令該連線進入需重新認證狀態——TS 層 SHALL 只見狀態布林與訊息，SHALL NOT 接觸任何 token。SSE 訂閱的 401 SHALL 同語意。

#### Scenario: access token 過期自動換發

- **WHEN** access token 過期後使用者於 remote 分頁觸發查詢
- **THEN** 查詢經自動 refresh 後成功回應，使用者無感；Keychain 內為 rotation 後的新 refresh credential

#### Scenario: refresh 亦失效即需重新認證

- **WHEN** refresh credential 已被撤銷時觸發查詢
- **THEN** 連線進入需重新認證狀態，操作回拒絕錯誤與繁中訊息，app 不崩潰、本地分頁不受影響

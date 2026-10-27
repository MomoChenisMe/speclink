## ADDED Requirements

### Requirement: 討論定案搜尋端點

server SHALL 提供綁定 scope 下的唯讀端點 GET /discussions/search，query 參數 q 為以空白分隔的關鍵字。server SHALL 以空白切詞後執行與本機 speclink discuss search 相同語意的搜尋（範圍涵蓋在途與封存記錄、比對限 topic、slug 與四種決定行、排序 topic 或 slug 命中優先），回應為 hits 陣列，每筆為該討論的資訊欄位加 matches 陣列（每個 match 含 kind、where、text）。q 缺席或全空白 SHALL 回 HTTP 400、error reason 為 invalid_argument。端點 SHALL 對具讀取權限者開放（reader role 可呼叫），SHALL NOT 寫入任何資料。既有 GET /search 端點的語意（在途記錄全文、每卡首個命中、與桌面本地搜尋對齊）SHALL 維持不變。未綁定、離線與認證失效的可觀察行為 SHALL 沿既有讀取端點的錯誤分類。

#### Scenario: 在途與封存各一筆命中

- **WHEN** scope 內有一筆在途記錄的 topic 含 drawer、一筆封存記錄第 2 輪的 `**Ruled out**:` 行含 drawer，呼叫 GET /discussions/search?q=drawer
- **THEN** HTTP 200；hits 恰兩筆，topic 命中的在途記錄在前（archived 為 false、matches 含 kind 為 topic 的項目），封存記錄在後（archived 為 true、matches 含 kind 為 ruled-out、where 為 round-2 的項目）

#### Scenario: q 缺席或全空白

- **WHEN** 呼叫 GET /discussions/search 不帶 q，或帶 q=%20
- **THEN** HTTP 400、error reason 為 invalid_argument；不寫入任何資料

#### Scenario: reader role 可呼叫

- **WHEN** 以 reader role 憑證呼叫 GET /discussions/search?q=golden
- **THEN** HTTP 200，回應形狀與 editor role 相同

#### Scenario: 既有全文搜尋端點不變

- **WHEN** 於本變更前後對同一 scope 呼叫 GET /search?q=drawer
- **THEN** 回應逐位元一致：仍只含在途記錄與變更 artifacts 的首個命中，不含封存記錄

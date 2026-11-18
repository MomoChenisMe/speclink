## ADDED Requirements

### Requirement: 詞彙文件寫入與討論搬回的端點

PUT /language SHALL 經 Command gateway 直通引擎的詞彙文件寫入命令。請求本文 SHALL 是一個物件，含 `content`（字串，詞彙文件的全文）。成功時 SHALL 回 HTTP 200，回應含 `created`（布林）；寫入 SHALL 隨 Unit of Work 落盤，之後 GET /language SHALL 回傳該內容。`content` 是空的或只有空白時 SHALL 回 400、reason 為 invalid_argument、文件不變。

POST /discussions/{slug}/restore SHALL 經 Command gateway 直通引擎的討論搬回命令。成功時 SHALL 回 HTTP 200，回應含 `slug`（字串）與 `restoredFrom`（字串，封存時的名稱）。該 slug 已有進行中的記錄時 SHALL 回 409、reason 為 refused；沒有封存的記錄時 SHALL 回 404。

兩個端點 SHALL 比照其他寫入動詞為 editor 限定（reader 收 403、scope 零改動），SHALL 沿既有的樂觀並行規則處理 revision 前置條件（不符時回 409），認證失效 SHALL 回 401。宿主的執行前掛鉤拒絕時 SHALL 回既有的拒絕回應（reason 為 refused、message 是宿主給的原因）。兩個端點 SHALL NOT 擴充 error reason registry。PUT /language SHALL NOT 受「政策鍵寫入」的權限限制：有寫入權限而沒有政策寫入權限的身分 SHALL 能寫入。

#### Scenario: 寫入詞彙文件後讀回

- **WHEN** editor 以 `{"content":"# Language\n"}` 呼叫 PUT /language，隨後呼叫 GET /language
- **THEN** 寫入的回應是 200 並含 `created`；讀取回應的內容是 `# Language\n`；server 重啟後讀取結果不變

#### Scenario: 空內容與 reader 被拒絕

- **WHEN** editor 以 `{"content":"  \n"}` 呼叫 PUT /language，接著一位 reader 以有效的內容呼叫同端點
- **THEN** 前者收 400、reason 為 invalid_argument；後者收 403；兩次之後詞彙文件都不變

#### Scenario: 沒有政策寫入權限的 editor 可以寫詞彙文件

- **WHEN** 一位有寫入權限、沒有政策寫入權限的身分呼叫 PUT /language
- **THEN** 回 200，詞彙文件被寫入

#### Scenario: 搬回討論的三種結果

- **WHEN** editor 對一個只有封存記錄的 slug、一個同時有進行中記錄的 slug、以及一個沒有任何記錄的 slug 分別呼叫 POST /discussions/{slug}/restore
- **THEN** 第一個回 200，含 `slug` 與 `restoredFrom`，之後 GET /discussions 列出該討論；第二個回 409、reason 為 refused、兩份記錄不變；第三個回 404

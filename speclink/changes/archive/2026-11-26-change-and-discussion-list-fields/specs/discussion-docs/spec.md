## ADDED Requirements

### Requirement: 討論記錄記下建立者的識別

執行環境帶有呼叫者的識別（宿主套件的身分識別）時，`speclink discuss new` SHALL 在討論記錄的 frontmatter 寫入 `created_by_id`，值為那個識別，寫出時 SHALL 經 YAML 純量跳脫；讀回的值 SHALL 與寫入的識別逐字相同，YAML 會讀成別種值的字（例如 `null`、`true`、`123`）也一樣。frontmatter 的 `created_by_id:` 沒有值時 SHALL 讀作沒有記錄，SHALL NOT 輸出空字串。執行環境沒有識別時（本機 CLI 以 git 身分執行）SHALL NOT 寫入這個鍵。`created_by` 既有的寫入 SHALL 不變。`discuss list` 與 `discuss show` 的 `--json` 輸出 SHALL 以 camelCase `createdById` 帶出這個值，沒有記錄時省略；`discuss search --json` 的命中項是清單的項目加上 `matches`，所以每一筆命中 SHALL 帶出與清單相同的 `createdById`。既有沒有 `created_by_id` 的討論記錄 SHALL 照常運作；系統 SHALL NOT 事後補上。封存與還原討論 SHALL NOT 改動這個鍵。

#### Scenario: 經宿主套件建立的討論記下識別

- **WHEN** 宿主以識別為 `u_1`、顯示名稱為 `Ann` 的呼叫者執行 `discuss new "登入流程" --slug auth-flow`，再執行 `discuss show auth-flow --json`
- **THEN** 討論記錄的 frontmatter 含 `created_by: Ann` 與 `created_by_id: u_1`；`info` 的 `createdBy` 是 `Ann`、`createdById` 是 `u_1`

#### Scenario: 識別讀回原值、空值讀作沒有

- **WHEN** 宿主以識別為 `true` 的呼叫者執行 `discuss new "旗標" --slug flag-talk`；另有一份手寫的討論 hand-talk，frontmatter 有一行 `created_by_id:` 但沒有值；執行 `discuss list --json`
- **THEN** flag-talk 的 frontmatter 含 `created_by_id: "true"`，清單裡這一筆的 `createdById` 是字串 `"true"`；hand-talk 這一筆沒有 `createdById` 這個鍵

#### Scenario: 本機 CLI 建立的討論沒有識別

- **WHEN** 在設有 git 身分的本機專案執行 `speclink discuss new "登入流程" --slug auth-flow`，再執行 `speclink discuss list --json`
- **THEN** 兩個指令的 exit code 都是 0；frontmatter 不含 `created_by_id`，清單裡這一筆沒有 `createdById` 這個鍵

### Requirement: 討論的清單與單項輸出最後修改時間

`speclink discuss list --json`（含 `--archived`）的每一筆與 `speclink discuss show <slug> --json` 的 `info` SHALL 輸出 `modifiedAt`：這份討論記錄最後一次被修改的時間，RFC 3339 的 UTC 時間、整秒，格式 SHALL 與 `speclink list --json` 的變更 `modifiedAt` 相同。值 SHALL 取自儲存回報的修改時間：本機專案為記錄檔案的修改時間，經儲存接口時為該鍵的修改時間。儲存答不出來時 SHALL 不輸出這個鍵，SHALL NOT 輸出 0、空字串或 null，指令 SHALL NOT 失敗。搜尋的命中項是清單的項目加上 `matches`，所以 `speclink discuss search --json` 的每一筆命中 SHALL 帶出與清單相同的 `modifiedAt`。`created` 既有的輸出 SHALL 不變。人眼輸出 SHALL 不變。連到遠端的 CLI SHALL 以討論清單回應與單一討論回應帶出這個值，SHALL NOT 對每一筆另外發一次請求；連到不提供這個欄位的舊版 Server 時鍵 SHALL 不出現。

#### Scenario: 加了一輪之後修改時間更新

- **WHEN** 討論 auth-flow 建立於較早的日期，今天執行 `speclink discuss add-round auth-flow --stdin` 加了一輪，再執行 `speclink discuss list --json`
- **THEN** exit code 為 0；auth-flow 這一筆的 `modifiedAt` 是符合 RFC 3339、以 `Z` 結尾、整秒的時間，日期是今天；`created` 仍是建立的日期

#### Scenario: 單項與已封存的清單同樣帶出

- **WHEN** 執行 `speclink discuss show auth-flow --json`，之後封存該討論並執行 `speclink discuss list --archived --json`
- **THEN** `info` 有 `modifiedAt`；已封存清單裡這一筆也有 `modifiedAt`

#### Scenario: 搜尋的命中項同樣帶出

- **WHEN** 討論 auth-flow 的 topic 含「登入」、frontmatter 有 `created_by_id: u_1`，執行 `speclink discuss search 登入 --json`
- **THEN** exit code 為 0；auth-flow 這一筆命中有 `modifiedAt` 與 `"createdById": "u_1"`，值與 `discuss list --json` 的同一筆相同

#### Scenario: 儲存答不出來時省略

- **WHEN** 儲存對討論 auth-flow 的修改時間回報「不知道」，執行 `discuss list --json`
- **THEN** exit code 為 0；auth-flow 這一筆沒有 `modifiedAt` 這個鍵，其他鍵照常輸出

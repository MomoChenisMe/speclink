## ADDED Requirements

### Requirement: 變更、討論與已封存清單的回應帶識別、階段與時間欄位

協定的回應型別 SHALL 增列下列選填欄位（皆 camelCase、serde default、沒有值時省略）：

- ChangeSummary：`whyExcerpt`（提案第一個 Why 段落裡第一個有內容的行的原文，規則與 verb-contract「清單的詳細輸出與 show 帶說明、階段與時間」的 `whyExcerpt` 相同）、`stage`（`proposed`、`in-progress` 或 `ready`）、`createdById`（metadata 的 `created_by_id`）、`readyAt`（就緒的日期，只在變更目前至少有一個任務並且全部勾完時有值）。
- ChangeStatus：`createdById` 與 `readyAt`，語意與 ChangeSummary 的相同。
- DiscussionInfo：`modifiedAt`（RFC 3339 的 UTC 時間、整秒）與 `createdById`（frontmatter 的 `created_by_id`）。
- ArchivedItem：`createdById`。

Server 端 SHALL 以引擎的同一組函式算出 `whyExcerpt`、`stage` 與 `readyAt`，SHALL NOT 另寫一份判斷。變更清單的回應 SHALL 在既有的那一次批次讀取裡取得算這些欄位所需的文件，SHALL NOT 為每個變更多發一次對儲存的呼叫。既有欄位的名稱、型別與出現條件 SHALL 不變。舊 Server 不送這些欄位時，client SHALL 以缺席容錯，SHALL NOT 偽造預設值；舊 client 收到這些欄位時 SHALL 能照常解析回應。桌面 app 的遠端變更清單 SHALL 把 `whyExcerpt` 帶到卡片的描述列，與本機專案的卡片相同。

#### Scenario: 新欄位序列化與舊回應相容

- **WHEN** 序列化一筆 `whyExcerpt`、`stage`、`createdById`、`readyAt` 都有值的 ChangeSummary，再序列化一筆四者皆缺的，並反序列化一筆沒有這四個鍵的舊回應
- **THEN** 前者的 JSON 含這四個 camelCase 鍵；後者的 JSON 不含這四個鍵；舊回應解析成功，四個欄位都是缺席

#### Scenario: 變更清單的回應帶出新欄位

- **WHEN** 變更 demo 的提案有 Why 段落、metadata 有 `created_by_id: u_1` 與 `ready_at`、3 個任務全部勾完，對 Server 發出取得變更清單的請求
- **THEN** 回應裡 demo 這一筆的 `whyExcerpt` 是 Why 段落的第一行、`stage` 是 `ready`、`createdById` 是 `u_1`、`readyAt` 是 `ready_at` 的日期部分

#### Scenario: 討論清單的回應帶出修改時間與識別

- **WHEN** 討論 auth-flow 的 frontmatter 有 `created_by_id: u_1`，對 Server 發出取得討論清單的請求
- **THEN** 回應裡 auth-flow 這一筆有 `modifiedAt`（RFC 3339、以 `Z` 結尾）與 `"createdById": "u_1"`

#### Scenario: 遠端專案的卡片有描述列

- **WHEN** 桌面 app 連到提供 `whyExcerpt` 的 Server，開啟看板
- **THEN** 提案有 Why 段落的變更卡顯示描述列，內容是 `whyExcerpt`；連到不提供這個欄位的舊版 Server 時描述列缺席

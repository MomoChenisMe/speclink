## ADDED Requirements

### Requirement: 已封存清單的呈現輔助欄位

desktop 協定的已封存清單項 SHALL 增列兩個選填欄位：`whyExcerpt`（字串——封存目錄 proposal.md 的 Why 區段首個非空行）與 `created`（字串 YYYY-MM-DD——封存目錄 metadata 的建立日期）。任一欄位的來源不可讀或缺席（proposal.md 不存在、無 Why 區段、metadata 無建立日期）時該欄位 SHALL 缺席（不序列化），SHALL NOT 以空字串或 null 佔位，清單其餘欄位照常回傳。兩欄位 SHALL 由清單載入一次帶出，SHALL NOT 要求前端逐項讀取封存文件。

#### Scenario: 封存項帶 Why 首句與建立日期

- **WHEN** desktop 載入已封存清單且某項的封存目錄含有 Why 區段的 proposal.md 與含建立日期的 metadata
- **THEN** 該項 `whyExcerpt` 為 Why 區段首個非空行、`created` 為該建立日期

#### Scenario: 來源缺席時欄位缺席

- **WHEN** desktop 載入已封存清單且某項的封存目錄無 proposal.md、metadata 無建立日期
- **THEN** 該項無 `whyExcerpt` 與 `created` 鍵，日期、名稱、任務數等既有欄位照常存在

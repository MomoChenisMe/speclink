## ADDED Requirements

### Requirement: 已封存清單的流程成本欄位

desktop 協定的已封存清單項 SHALL 增列五個選填數字欄位，由清單載入一次帶出，SHALL NOT 要求前端逐項讀取封存文件：

- `docLines`（整數）：封存目錄中 `proposal.md`、`design.md`、`tasks.md` 與每個 `specs/<capability>/spec.md` 的行數總和；行數為換行字元數，最後一行沒有換行時也算一行；不存在的檔不計；四類檔案都不存在時缺席。
- `leadDays`（整數）：封存目錄名的日期前綴減去 metadata `created` 的日期部分，以天計、最小為 0；`created` 缺席或無法辨識時缺席。
- `leadHours`（數字，小數一位）：metadata `archived_at` 減去 `created`，以小時計、四捨五入到小數一位、最小為 0；只有兩者都是 RFC 3339 時戳時才有，否則缺席。
- `reviewRounds`（整數）：metadata 的 `reviewed_rounds`；缺席時缺席。
- `verifyRounds`（整數）：metadata 的 `verified_rounds`；缺席時缺席。

來源缺席或無法解析時，該欄位 SHALL 缺席（不序列化），SHALL NOT 以 0、空字串或 null 佔位，清單其餘欄位照常回傳。這五個欄位 SHALL 由桌面 core 產出；server 的已封存清單回應 SHALL NOT 因本需求改變形狀。

#### Scenario: 新格式封存項帶齊流程成本欄位

- **WHEN** desktop 載入已封存清單，某項封存目錄名為 `2026-10-22-demo`，含 120 行的 proposal.md、80 行的 tasks.md 與一份 50 行的 delta spec、沒有 design.md，metadata 含 `created: 2026-10-21T10:00:00+08:00`、`archived_at: 2026-10-22T16:30:00+08:00`、`reviewed_rounds: 2`、`verified_rounds: 3`
- **THEN** 該項 `docLines` 為 250、`leadDays` 為 1、`leadHours` 為 30.5、`reviewRounds` 為 2、`verifyRounds` 為 3

#### Scenario: 舊格式封存項只帶得出的欄位

- **WHEN** desktop 載入已封存清單，某項封存目錄名為 `2026-08-14-old`，metadata 含 `created: 2026-08-12`、沒有 `archived_at` 與輪數欄位，目錄內有 proposal.md 與 tasks.md
- **THEN** 該項有 `docLines` 與值為 2 的 `leadDays`，沒有 `leadHours`、`reviewRounds` 與 `verifyRounds` 鍵，日期、名稱、任務數等既有欄位照常存在

#### Scenario: 沒有 metadata 與文件時欄位全部缺席

- **WHEN** desktop 載入已封存清單，某項封存目錄沒有 `.openspec.yaml`，也沒有 proposal.md、design.md、tasks.md 與 delta spec
- **THEN** 該項沒有 `docLines`、`leadDays`、`leadHours`、`reviewRounds`、`verifyRounds` 任一鍵，清單照常回傳

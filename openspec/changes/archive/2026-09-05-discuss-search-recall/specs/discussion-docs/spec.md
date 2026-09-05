## ADDED Requirements

### Requirement: 討論定案以 search 動詞可查

speclink discuss search SHALL 接受一個以上的位置參數作為關鍵字，旗標僅 --json 與 --no-color，不吃 stdin。比對 SHALL 為不分大小寫的子字串比對，多個關鍵字任一命中即算命中。比對範圍 SHALL 限於記錄的 topic、slug 與四種決定行：各輪內以 `**Ruled out**:` 起頭的行，以及 Conclusion 區內以 `**Decision**:`、`**Rejected alternatives**:`、`**Deferred**:` 起頭的行；每個決定行 SHALL 連同其後緊接的條列行（以 `- `、`* `、`+ ` 或 `N. ` 起頭、直到第一個非條列行為止）一併參與比對，每個命中的條列行各為一筆 match，kind 與 where 同其標記行。其他行（Focus、Position、Open、Evidence 與散文）SHALL NOT 參與比對。搜尋範圍 SHALL 預設同時涵蓋在途（openspec/discussions/）與封存（openspec/discussions/archive/）記錄，無旗標可縮減。每筆命中 SHALL 帶該記錄的既有列表欄位（slug、topic、status、rounds、created、createdBy 有才出、kind 有才出、path、archived）與 matches 陣列；每個 match SHALL 帶 kind（topic、slug、ruled-out、decision、rejected、deferred 之一）、where（frontmatter、round-N 或 conclusion）與 text（該行原文去前後空白）。排序 SHALL 為 topic 或 slug 命中者排前、其餘其後；兩群內各依 created 由新到舊、同日依 slug 字典序；同一記錄的 matches 依文件順序。人眼輸出：零命中 SHALL 於 stdout 印 `No discussions match "<關鍵字以空白接起>".`；有命中 SHALL 以標題行 `Discussions matching "<關鍵字>":` 起頭，每筆一行 `  • <slug> [<status>, archived|live] (<created>) — <topic>`，其下每個 match 各一行縮排的 `<where> <kind>: <text>`；--no-color 下 SHALL 無 ANSI 色彩。--json SHALL 輸出 `{ "hits": [...] }`（欄位 camelCase），零命中為空陣列。成功（含零命中）exit code SHALL 為 0；未帶任何關鍵字 SHALL 以非零 exit code 結束、stderr 說明用法、stdout 無輸出。記錄缺輪標題或 Conclusion 區時 SHALL 仍以 topic 與 slug 參與比對，缺的區段視為無決定行，SHALL NOT 使整個查詢失敗。本動詞 SHALL NOT 寫入任何檔案。既有 discuss list 與 discuss show 的人眼與 --json 輸出 SHALL 逐位元不變。remote 模式下本動詞 SHALL 可用且人眼與 --json 輸出與本機同形（path 缺席與 promotedTo、concluded 增欄沿用 discuss list 的既定分歧）；離線、認證失效與 revision 衝突的可觀察行為 SHALL 沿既有 remote 讀取動詞的錯誤分類與訊息，不另立訊息。人眼輸出為英文，與既有 discuss 動詞一致，不隨 locale 設定變動。本動詞為 Speclink 自有延伸。

#### Scenario: 決定行命中回傳輪號與原文

- **WHEN** 某封存記錄第 2 輪含一行 `**Ruled out**: RichDetailDrawer 加 readOnly 旗標（分支地獄）`，執行 speclink discuss search drawer --json
- **THEN** exit code 0；hits 含該記錄，其 archived 為 true，matches 含一筆 kind 為 ruled-out、where 為 round-2、text 為該行原文（去前後空白）的項目

#### Scenario: 標記獨占一行時其下條列行命中

- **WHEN** 某封存記錄第 1 輪的 `**Ruled out**:` 獨占一行，其下兩行條列 `- 只在 tray.ts 修落頁` 與 `- 把 drawer 拿掉`，接著空行與 `**Open**: drawer naming`，執行 speclink discuss search drawer --json
- **THEN** 該記錄的 matches 恰含一筆 kind 為 ruled-out、where 為 round-1、text 為 `- 把 drawer 拿掉` 的項目；`**Open**:` 行不命中

#### Scenario: 非決定行不命中且零命中回空

- **WHEN** 唯一含關鍵字 sidecar 的記錄只在 Evidence 行提到它，執行 speclink discuss search sidecar 與 speclink discuss search sidecar --json
- **THEN** 人眼輸出恰為一行 `No discussions match "sidecar".`、exit code 0；--json 輸出 `{ "hits": [] }`、exit code 0

#### Scenario: 多關鍵字任一命中並依 topic 命中優先排序

- **WHEN** 記錄 A（created 2026-07-01）的 topic 含 golden、記錄 B（created 2026-08-01）只在 Conclusion 的 `**Deferred**:` 行含 SSE，執行 speclink discuss search golden sse --json
- **THEN** hits 依序為 A、B：A 的 matches 含 kind 為 topic、where 為 frontmatter 的項目；B 的 matches 含 kind 為 deferred、where 為 conclusion 的項目；大小寫差異（SSE 對 sse）不影響命中

##### Example: 排序規則

| 記錄 | 命中位置 | created | 輸出順位 |
| ---- | -------- | ------- | -------- |
| A | topic | 2026-07-01 | 1 |
| C | slug | 2026-06-01 | 2 |
| B | conclusion Deferred | 2026-08-01 | 3 |
| D | round-1 Ruled out | 2026-05-01 | 4 |

#### Scenario: 人眼輸出格式與 --no-color

- **WHEN** 對前述 A、B 兩筆命中執行 speclink discuss search golden sse --no-color
- **THEN** stdout 第一行為 `Discussions matching "golden sse":`，接著每筆一行 `  • <slug> [<status>, archived|live] (<created>) — <topic>`，其下每個 match 各一行縮排的 `<where> <kind>: <text>`；全程無 ANSI 色彩；exit code 0

#### Scenario: 未帶關鍵字

- **WHEN** 執行 speclink discuss search 不帶任何位置參數
- **THEN** 以非零 exit code 結束、stderr 說明用法、stdout 無輸出；不建立或改動任何檔案

#### Scenario: 記錄缺區段時不使查詢失敗

- **WHEN** 某在途記錄尚無任何輪與 Conclusion 內文，其 topic 含關鍵字，執行 speclink discuss search 該關鍵字 --json
- **THEN** exit code 0；該記錄出現在 hits，matches 僅含 kind 為 topic 的項目

#### Scenario: remote 模式輸出同形

- **WHEN** workspace 綁定 server 後執行 speclink discuss search drawer --json，與本機對同一組記錄執行同指令
- **THEN** 兩者 hits 的順序、每筆的 slug 與 matches 陣列相同；差異僅限 path 缺席與 promotedTo、concluded 增欄（與 discuss list 的既定分歧一致）

#### Scenario: 既有 list 與 show 輸出不變

- **WHEN** 於本變更前後對同一 workspace 執行 speclink discuss list、speclink discuss list --archived 與 speclink discuss show 某 slug（人眼與 --json）
- **THEN** stdout、stderr 與 exit code 逐位元一致

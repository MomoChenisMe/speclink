## Purpose

speclink trace 動詞的行為契約：以封存目錄與既有 metadata 組裝單一 capability 的演進鏈（動過它的封存 change、來源討論與其扇出、逐 task evidence），含 --json 形狀、evidence 缺失的 null 語意與髒資料的寬容組裝。本 capability 保證溯源讀取只組裝既存事實、不回讀已停用的 @trace code 清單。

## ADDED Requirements

### Requirement: 溯源鏈組裝

`speclink trace <capability>` SHALL 組裝並輸出該 capability 的封存演進鏈：動過它的封存 change 集合以「封存目錄含該 capability 的 delta 子目錄」列舉，依封存日期由舊至新排序；每個 change SHALL 帶封存目錄名與來源討論（.openspec.yaml 的 from_discussion，缺欄時為無）；來源討論 SHALL 帶其轉出的全部變更清單（frontmatter promoted_to）與每個兄弟變更觸及的 capability 名集合；另 SHALL 列出正典規格內每條 Requirement 現行 @trace 註記的歸屬 change。進行中（未封存）的 change SHALL NOT 出現在鏈中。人讀輸出 SHALL 為縮排樹寫至 stdout，`--no-color` 下 SHALL 僅省略色碼、內容不變，成功 exit code SHALL 為 0。

#### Scenario: 完整鏈的人讀輸出

- **WHEN** 對存在正典規格且有封存演進的 capability 執行 speclink trace
- **THEN** stdout SHALL 依封存日期由舊至新列出各封存 change，每項含封存目錄名與來源討論 slug（無則標示無），來源討論項下 SHALL 列出其轉出的兄弟變更及各自觸及的 capability，末段 SHALL 列出每條 Requirement 的現行歸屬 change，exit code 為 0

#### Scenario: 進行中 change 不入鏈

- **WHEN** 某進行中 change 的 delta 目錄含該 capability 而尚未封存
- **THEN** 該 change SHALL NOT 出現在 trace 輸出的鏈中

##### Example: 列舉與排序

| 封存目錄 | 是否入鏈 | 排序 |
| ----- | --------------- | ----- |
| 2026-07-10-a（含 specs/x/） | 是 | 第 1 |
| 2026-08-02-b（含 specs/x/） | 是 | 第 2 |
| 進行中 change c（含 specs/x/） | 否 | — |
| 2026-08-05-d（不含 specs/x/） | 否 | — |

### Requirement: --json 輸出形狀

`speclink trace <capability> --json` SHALL 輸出穩定的 camelCase JSON 至 stdout：`capability`（字串）、`requirements`（陣列，元素含 `name` 與 `source` 字串）、`changes`（陣列，依封存日期由舊至新，元素含 `name` 字串、`archivedDir` 字串、`fromDiscussion` 字串或 null、`evidence` 為陣列或 null，evidence 元素含 `taskId` 字串與 `files` 字串陣列）、`discussions`（陣列，元素含 `slug` 字串、`archived` 布林、`promotedTo` 陣列，其元素含 `change` 字串與 `capabilities` 字串陣列）。成功 exit code SHALL 為 0，payload 外 SHALL NOT 混入其他 stdout 文字。

#### Scenario: JSON 欄位齊備

- **WHEN** 對有封存演進的 capability 執行 speclink trace --json
- **THEN** stdout SHALL 為單一 JSON 物件，含 capability、requirements、changes、discussions 四鍵，changes 依封存日期由舊至新，欄位名皆為 camelCase

### Requirement: evidence 的存在性偵測

每個入鏈 change 的 evidence SHALL 以該封存目錄是否存在 .evidence.json 逐一判定：存在則輸出其逐 task 的檔案清單；不存在則該 change 的 evidence SHALL 為 null（--json）或標示無記錄（人讀），SHALL NOT 因此失敗或警告。正典規格 @trace 註記中的任何 code 檔案清單 SHALL NOT 被讀取或輸出。

#### Scenario: 有 evidence 的 change

- **WHEN** 入鏈 change 的封存目錄含 .evidence.json
- **THEN** 輸出 SHALL 含該 change 逐 task 的觸及檔案清單，內容與 .evidence.json 一致

#### Scenario: 無 evidence 的 change 靜默為 null

- **WHEN** 入鏈 change 的封存目錄無 .evidence.json
- **THEN** --json 中該 change 的 evidence SHALL 為 null，人讀輸出標示無記錄，exit code 仍為 0 且 stderr 無警告

#### Scenario: 舊 @trace 的 code 清單不被採用

- **WHEN** 正典規格的 @trace 註記帶有 code 檔案清單且該 change 無 .evidence.json
- **THEN** 輸出的 evidence SHALL 為 null，@trace 內的 code 清單 SHALL NOT 出現在任何輸出欄位

### Requirement: 找不到 capability 的近似建議

`speclink trace <capability>` 於 capability 無正典規格時 SHALL 以非零 exit code 失敗，stderr SHALL 含至多三筆近似的既有 capability 名建議（無近似時僅報不存在），stdout SHALL NOT 輸出成功 payload。

#### Scenario: 不存在的 capability

- **WHEN** 對正典規格中不存在的 capability 執行 speclink trace
- **THEN** exit code SHALL 非零，stderr SHALL 報該 capability 不存在並列出至多三筆近似名建議，--json 模式下 stdout 無成功 payload

### Requirement: 單環髒資料的寬容組裝

鏈中單一 change 的欄位缺漏或指涉失效 SHALL NOT 使整體輸出失敗：@trace 歸屬指向的 change 找不到對應封存目錄時，該歸屬 SHALL 照列於 requirements、changes 清單缺其明細；.openspec.yaml 缺 from_discussion 時該 change 的來源討論欄位 SHALL 為 null；from_discussion 指向的討論檔不存在時，該討論 SHALL NOT 出現於 discussions 清單、change 的來源討論欄位仍照列其 slug，其餘鏈環照常輸出。

#### Scenario: 歸屬指向不存在的封存目錄

- **WHEN** 某 Requirement 的 @trace 歸屬 change 在封存目錄中找不到
- **THEN** 該歸屬 SHALL 照列於 requirements，changes 清單不含該 change 的明細，其餘 change 照常輸出，exit code 為 0

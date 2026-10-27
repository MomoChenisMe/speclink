## MODIFIED Requirements

### Requirement: 溯源鏈組裝

<!-- BEFORE: 以 openspec/ 與 .openspec.yaml 描述規格資料夾與變更資料檔 -->

`speclink trace <capability>` SHALL 組裝並輸出該 capability 的封存演進鏈：動過它的封存 change 集合以「封存目錄含該 capability 的 delta 子目錄」列舉，依封存日期由舊至新排序；每個 change SHALL 帶封存目錄名與來源討論（.meta.yaml 的 from_discussion，缺欄時為無）；來源討論 SHALL 帶其轉出的全部變更清單（frontmatter promoted_to）與每個兄弟變更觸及的 capability 名集合；另 SHALL 列出正式規格內每條 Requirement 現行 @trace 註記的歸屬 change。進行中（未封存）的 change SHALL NOT 出現在鏈中。人讀輸出 SHALL 為縮排樹寫至 stdout，`--no-color` 下 SHALL 僅省略色碼、內容不變，成功 exit code SHALL 為 0。

#### Scenario: 完整鏈的人讀輸出

- **WHEN** 對存在正式規格且有封存演進的 capability 執行 speclink trace
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

### Requirement: 單環髒資料的寬容組裝

<!-- BEFORE: 以 openspec/ 與 .openspec.yaml 描述規格資料夾與變更資料檔 -->

鏈中單一 change 的欄位缺漏或指涉失效 SHALL NOT 使整體輸出失敗：@trace 歸屬指向的 change 找不到對應封存目錄時，該歸屬 SHALL 照列於 requirements、changes 清單缺其明細；.meta.yaml 缺 from_discussion 時該 change 的來源討論欄位 SHALL 為 null；from_discussion 指向的討論檔不存在時，該討論 SHALL NOT 出現於 discussions 清單、change 的來源討論欄位仍照列其 slug，其餘鏈環照常輸出。

#### Scenario: 歸屬指向不存在的封存目錄

- **WHEN** 某 Requirement 的 @trace 歸屬 change 在封存目錄中找不到
- **THEN** 該歸屬 SHALL 照列於 requirements，changes 清單不含該 change 的明細，其餘 change 照常輸出，exit code 為 0

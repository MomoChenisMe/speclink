# trace-skill Specification

## Purpose

/speclink-trace 技能的內容契約：自然語言問題對應 capability 後沿 trace 動詞的鏈組出附來源路徑的敘事答案，evidence 缺失走 git 反查、查無規格走程式碼庫考古，兩種降級對使用者不可見。本 capability 保證使用者問「功能怎麼來的」得到同一種敘事格式，無論底層資料齊不齊。

## Requirements

### Requirement: 問題對應與敘事答案

內嵌 speclink-trace 技能（事實來源 crates/speclink-core/assets/skills/trace.md，經 init 與 update 渲染至各工具技能目錄）SHALL 規定以下流程：接受自然語言問題，先以正典規格清單比對出目標 capability（canon pass）；命中時呼叫 speclink trace 的 --json 輸出取得演進鏈，讀取來源討論的結論與輪次、各 change 提案的動機段落，再依 evidence 取得觸及檔案並最終閱讀現行程式碼確認現況；答案 SHALL 為敘事形式，含：決策內容與理由、被否決的替代方案、一起演進的關聯 capability、以及每項陳述的來源路徑。

#### Scenario: 渲染產物規定命中規格的完整流程

- **WHEN** 執行 speclink init 或 speclink update 渲染技能檔
- **THEN** 產出的 speclink-trace 技能檔 SHALL 依序規定 canon pass、呼叫 trace --json、讀討論結論與提案動機、依 evidence 讀檔、最後閱讀現行程式碼，且 SHALL 規定答案附來源路徑

#### Scenario: 答案以現行程式碼收尾

- **WHEN** 技能檔描述答案的組成
- **THEN** 技能檔 SHALL 規定任何關於「現在的行為」的陳述以現行程式碼的閱讀結果為準，evidence 與提交記錄僅作為歷史快照引用


<!-- @trace
source: feature-provenance-skill
updated: 2026-08-22
-->

---
### Requirement: evidence 缺失的靜默補查

技能檔 SHALL 規定：trace --json 中某 change 的 evidence 為 null 時，改以版本控制歷史反查該 change 的觸及檔案（提交訊息帶 change 名的慣例），反查結果 SHALL 作為盡力線索而非保證；反查無所獲時 SHALL 以討論與提案內容作答，SHALL NOT 因缺 evidence 而中止或要求使用者補資料。

#### Scenario: 渲染產物含 git 反查降級

- **WHEN** 檢視渲染產出的 speclink-trace 技能檔
- **THEN** 技能檔 SHALL 含「evidence 為 null 時以 git 提交記錄反查觸及檔案、無所獲則以討論與提案作答」的規定，且 SHALL 標明反查是盡力線索而非保證


<!-- @trace
source: feature-provenance-skill
updated: 2026-08-22
-->

---
### Requirement: 查無規格的考古降級

技能檔 SHALL 規定：canon pass 未命中任何 capability 時，改以程式碼庫考古作答（以關鍵字檢索程式碼、以 git log 與 git blame 追引入該行為的提交、以提交訊息為決策線索），答案格式與命中規格時相同——敘事、附來源路徑；SHALL NOT 回覆「查無規格」了事。

#### Scenario: 渲染產物含無規格降級

- **WHEN** 檢視渲染產出的 speclink-trace 技能檔
- **THEN** 技能檔 SHALL 含「查無對應規格時改走程式碼庫考古並以同格式作答」的規定


<!-- @trace
source: feature-provenance-skill
updated: 2026-08-22
-->

---
### Requirement: 降級不可見原則

技能檔 SHALL 規定：兩種降級（evidence 缺失、查無規格)皆為內部管線，答案文案 SHALL NOT 出現「降級」「舊時期」「資料不完整」等內部處置字眼；不同資料來源（討論、提案、提交記錄、程式碼）SHALL 以自然引用方式呈現於同一種敘事格式。

#### Scenario: 渲染產物禁止內部管線字眼

- **WHEN** 檢視渲染產出的 speclink-trace 技能檔
- **THEN** 技能檔 SHALL 含答案文案禁用內部處置字眼的守則，並 SHALL 規定各資料來源以自然引用呈現


<!-- @trace
source: feature-provenance-skill
updated: 2026-08-22
-->

---
### Requirement: 技能資產發佈

speclink-trace 技能 SHALL 隨 init 與 update 發佈：執行 speclink update 後，各工具技能目錄 SHALL 含 speclink-trace 技能檔；渲染產物內容由 golden 快照測試保護，快照更新屬刻意變更。

#### Scenario: update 後技能檔存在

- **WHEN** 於已初始化的專案執行 speclink update
- **THEN** 各工具技能目錄 SHALL 含 speclink-trace 技能檔，其內容與資產事實來源一致

<!-- @trace
source: feature-provenance-skill
updated: 2026-08-22
-->
# manual-task-marker Specification

## Purpose

任務行 [M] 手動測試標記的語意：標記的解析與顯示剝離、把任務分成寫碼與手動兩類後的寫碼完成度預測子，以及任務 payload 的 manual 欄位。本 capability 保證代理不會替使用者勾掉自己無法觀察的驗證項——apply 在寫碼任務全數完成時即回報完成，並把手動項明列給使用者親自執行。

## Requirements

### Requirement: 任務行的手動測試標記與解析

tasks.md 的任務行 SHALL 支援手動測試標記 `[M]`:位於 checkbox 之後的前綴槽。`[M]` SHALL 為唯一承載語意的前綴標記,解析為該任務的 manual 旗標;歷史遺留的 `[P] ` 前綴 SHALL 仍被剝除但不承載任何旗標(舊檔顯示容忍——封存區與外部使用者 repo 的既有檔案不因移除而滲出字面前綴)。解析器 SHALL 以重複剝除方式接受兩種前綴(順序不敏感、各至多出現一次);任務的顯示描述 SHALL 剝除全部前綴標記。無任何標記的任務行為 SHALL 逐位元不變。

#### Scenario: 解析手動測試任務

- **WHEN** tasks.md 含一行「- [ ] [M] 手動驗證匯入結果」
- **THEN** 該任務解析為 manual=true,描述為「手動驗證匯入結果」(不含標記)

#### Scenario: 舊 [P] 前綴只剝不承載

- **WHEN** tasks.md 含一行「- [x] [P] 舊任務」或「- [x] [P] [M] 混用行」
- **THEN** 描述不含任何前綴標記;`[P]` 不落任何旗標,混用行的 manual 仍為 true

#### Scenario: 無標記行為不變

- **WHEN** tasks.md 僅含無前綴標記的任務行
- **THEN** 解析結果(描述、done、stable ID)與標記引入前逐位元一致

##### Example: 前綴解析

| 任務行 | manual | 描述 |
| ------ | ------ | ---- |
| - [ ] [M] 手測匯入 | true | 手測匯入 |
| - [x] [P] 舊任務 | false | 舊任務 |
| - [x] [P] [M] 混用 | true | 混用 |
| - [ ] 寫解析器 | false | 寫解析器 |


<!-- @trace
source: task-marker-ui-and-parallel-removal
updated: 2026-08-11
-->

---
### Requirement: 寫碼任務全完成預測子

系統 SHALL 提供單一實作的「寫碼任務全完成」預測子:所有 manual=false 的任務皆已勾選。零任務的 change 與全數為 `[M]` 任務的 change,預測子 SHALL 空真成立。消費此預測子的守門 SHALL 僅在「寫碼任務總數大於零且寫碼任務未全完成」時拒絕。任務進度統計 SHALL 同時回傳全量計數與寫碼計數,各守門與失效判定共用同一份統計,SHALL NOT 各自過濾。

#### Scenario: 僅餘手動任務時預測子成立

- **WHEN** change 有 10 個任務,9 個寫碼任務全勾、1 個 `[M]` 任務未勾
- **THEN** 寫碼任務全完成預測子成立

#### Scenario: 寫碼任務未完成時預測子不成立

- **WHEN** change 有 10 個任務,8 個寫碼任務僅勾 7 個
- **THEN** 預測子不成立,消費守門拒絕並點名寫碼任務計數(7/8)

##### Example: 預測子判定

| 全量(完成/總數) | 寫碼(完成/總數) | 預測子 |
| --------------- | --------------- | ------ |
| 9/10 | 9/9 | 成立 |
| 7/10 | 7/8 | 不成立 |
| 0/0 | 0/0 | 成立(空真) |
| 0/2 | 0/0(全為 [M]) | 成立(空真) |


<!-- @trace
source: manual-task-marker-gates
updated: 2026-08-11
-->

---
### Requirement: 任務 payload 的 manual 欄位與寫碼進度

`speclink instructions apply --json` 的任務項欄位集合 SHALL 為 `id`/`description`/`done`/`manual`——`parallel` 欄位 SHALL NOT 出現(已隨 `[P]` 語意移除);progress SHALL 含 `codeTotal`/`codeComplete`/`codeRemaining` 三欄(寫碼任務的總數/完成數/剩餘數)。既有欄位名與語意 SHALL NOT 改變。無 `[M]` 任務的 change,三個 code 欄位之值 SHALL 與全量計數一致。

#### Scenario: 手動任務上線 payload

- **WHEN** 對含一個未勾 `[M]` 任務、九個已勾寫碼任務的 change 執行 instructions apply --json
- **THEN** 該 `[M]` 任務項 manual=true 且無 parallel 欄位,progress 為 total=10、complete=9、remaining=1、codeTotal=9、codeComplete=9、codeRemaining=0

#### Scenario: 無手動任務時欄位一致

- **WHEN** 對不含 `[M]` 任務的 change 執行 instructions apply --json
- **THEN** codeTotal=total、codeComplete=complete、codeRemaining=remaining,任務項欄位集合為 id/description/done/manual


<!-- @trace
source: task-marker-ui-and-parallel-removal
updated: 2026-08-11
-->

---
### Requirement: apply 技能的手動任務處理

apply 技能文字 SHALL 指示:`[M]` 任務不由 agent 代勾——手動測試由使用者執行與勾選;寫碼任務全數完成時 SHALL 回報 apply 完成並向使用者點名尚餘的 `[M]` 任務。技能模板 SHALL 於 claude 與 codex 兩工具正典化生成,golden 對照涵蓋。

#### Scenario: 寫碼任務完成即回報

- **WHEN** apply 進行至寫碼任務全數勾選、僅餘一個 `[M]` 任務未勾
- **THEN** 技能回報實作完成,點名該手動測試任務留待使用者執行,不代勾該任務

#### Scenario: 技能模板生成

- **WHEN** 執行 speclink update
- **THEN** claude 與 codex 的 apply 技能檔含手動任務處理原則,與 golden 對照一致

<!-- @trace
source: manual-task-marker-gates
updated: 2026-08-11
-->

---
### Requirement: 標記位置的 change 驗證檢查

change 驗證 SHALL 對 tasks.md 的每個解析後任務檢查手動標記位置,命中下列任一錯型即報 error、使該 change 驗證結果為 invalid:(A)「編號在前」——顯示描述的首個空白分隔 token 僅含 ASCII 數字與句點且至少含一個數字,且下一個 token 為字面 `[M]`;(B)「行首殘留」——顯示描述以字面 `[M]` 開頭(後接空白或即為結尾)。檢查 SHALL 僅施於描述開頭,SHALL NOT 掃描描述中段(行文或反引號提及 `[M]` 不構成違規);已勾與未勾任務 SHALL 同等檢查。錯誤訊息 SHALL 自帶修復指引:含 tasks.md 邏輯路徑(正斜線)、任務序號、描述前綴引文,並以正誤例並列指明 `[M]` 須緊接 checkbox。tasks.md 缺席或無命中時 SHALL NOT 產生任何 error 或 warning,既有驗證輸出逐位元不變;既有錯誤 SHALL 先列,本檢查的錯誤後附。

#### Scenario: 編號在前報 error

- **WHEN** 某 change 的 tasks.md 含一行「- [ ] 6.2 [M] 手動驗收」,執行該 change 的驗證
- **THEN** 驗證結果 invalid,error 訊息含任務序號與描述引文,並以正誤例並列指明 `[M]` 須緊接 checkbox

#### Scenario: 行首殘留報 error

- **WHEN** 某 change 的 tasks.md 含一行「- [ ]  [M] 手測匯入」(checkbox 後兩個空格,前綴槽漏接),執行該 change 的驗證
- **THEN** 驗證結果 invalid,error 訊息點名 checkbox 後恰一個空格

#### Scenario: 正確前綴與中段字面提及不報

- **WHEN** 某 change 的 tasks.md 僅含「- [ ] [M] 手測匯入」與「- [x] 1.1 前綴剝除迴圈同時接受 `[P]` 與 `[M]` 的說明文字」兩行,執行該 change 的驗證
- **THEN** 驗證不因標記位置產生任何 error 或 warning

##### Example: 誤置判定

| 任務行 | 判定 |
| ------ | ---- |
| - [ ] [M] 3.2 手測匯入 | 通過 |
| - [ ] 3.2 [M] 手測匯入 | 錯型 A |
| - [ ] 1.10 [M] 手測 | 錯型 A |
| - [ ]  [M] 手測 | 錯型 B |
| - [ ] 說明 `[M]` 剝除規則 | 通過 |

<!-- @trace
source: manual-marker-placement-lint
updated: 2026-08-12
-->

---
### Requirement: ingest 技能的起草標記指引

ingest 技能文字 SHALL 含手動測試任務的 `[M]` 起草指引,且 SHALL 以正誤例並列(對比對)呈現:正例(`[M]` 緊接 checkbox、編號在後)與誤例(編號在前)並列,附後果說明(引擎不認、任務被算成寫碼任務而卡住完成度)與「checkbox 後恰一個空格」規則。技能模板 SHALL 於 claude 與 codex 兩工具正典化生成,golden 對照涵蓋。

#### Scenario: ingest 補任務時的指引

- **WHEN** ingest 流程因需求變更補起草含人工驗收的任務行
- **THEN** 技能文字指示該任務行以 `[M]` 緊接 checkbox 起草,並可對照誤例辨識錯型

#### Scenario: 技能模板生成

- **WHEN** 執行 speclink update
- **THEN** claude 與 codex 的 ingest 技能檔含對比對起草指引,與 golden 對照一致

<!-- @trace
source: manual-marker-placement-lint
updated: 2026-08-12
-->
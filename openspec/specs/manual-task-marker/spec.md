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
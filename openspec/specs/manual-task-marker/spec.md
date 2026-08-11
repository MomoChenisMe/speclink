# manual-task-marker Specification

## Purpose

TBD - created by archiving change 'manual-task-marker-gates'. Update Purpose after archive.

## Requirements

### Requirement: 任務行的手動測試標記與解析

tasks.md 的任務行 SHALL 支援手動測試標記 `[M]`:位於 checkbox 之後的前綴槽,與既有 `[P]` 平行標記同槽。解析器 SHALL 以重複剝除方式同時接受 `[P]` 與 `[M]`(順序不敏感、各至多出現一次),將 `[M]` 解析為該任務的 manual 旗標;任務的顯示描述 SHALL 剝除全部前綴標記(與 `[P]` 既有行為一致)。無任何標記的任務行為 SHALL 逐位元不變。

#### Scenario: 解析手動測試任務

- **WHEN** tasks.md 含一行「- [ ] [M] 手動驗證匯入結果」
- **THEN** 該任務解析為 manual=true,描述為「手動驗證匯入結果」(不含標記)

#### Scenario: 兩標記並存且順序不敏感

- **WHEN** 任務行前綴為「[P] [M]」或「[M] [P]」
- **THEN** 兩種順序皆解析為 parallel=true 且 manual=true,描述相同

#### Scenario: 無標記行為不變

- **WHEN** tasks.md 僅含無前綴標記的任務行
- **THEN** 解析結果(描述、done、stable ID)與標記引入前逐位元一致

##### Example: 前綴解析

| 任務行 | manual | parallel | 描述 |
| ------ | ------ | -------- | ---- |
| - [ ] [M] 手測匯入 | true | false | 手測匯入 |
| - [x] [M] [P] 手測匯入 | true | true | 手測匯入 |
| - [ ] 寫解析器 | false | false | 寫解析器 |


<!-- @trace
source: manual-task-marker-gates
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

`speclink instructions apply --json` 的任務 payload SHALL 逐任務增列 `manual` 欄位(鏡射既有 `parallel` 欄位形狀),progress SHALL 增列 `codeTotal`/`codeComplete`/`codeRemaining` 三欄(寫碼任務的總數/完成數/剩餘數)。既有欄位名與語意 SHALL NOT 改變。無 `[M]` 任務的 change,三個 code 欄位之值 SHALL 與全量計數一致。

#### Scenario: 手動任務上線 payload

- **WHEN** 對含一個未勾 `[M]` 任務、九個已勾寫碼任務的 change 執行 instructions apply --json
- **THEN** 該 `[M]` 任務項 manual=true,progress 為 total=10、complete=9、remaining=1、codeTotal=9、codeComplete=9、codeRemaining=0

#### Scenario: 無手動任務時欄位一致

- **WHEN** 對不含 `[M]` 任務的 change 執行 instructions apply --json
- **THEN** codeTotal=total、codeComplete=complete、codeRemaining=remaining,既有欄位逐位元不變


<!-- @trace
source: manual-task-marker-gates
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
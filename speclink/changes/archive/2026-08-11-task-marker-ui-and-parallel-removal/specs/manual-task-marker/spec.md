## MODIFIED Requirements

### Requirement: 任務行的手動測試標記與解析

<!-- REMOVED-SCENARIO: 兩標記並存且順序不敏感 -->

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

### Requirement: 任務 payload 的 manual 欄位與寫碼進度

`speclink instructions apply --json` 的任務項欄位集合 SHALL 為 `id`/`description`/`done`/`manual`——`parallel` 欄位 SHALL NOT 出現(已隨 `[P]` 語意移除);progress SHALL 含 `codeTotal`/`codeComplete`/`codeRemaining` 三欄(寫碼任務的總數/完成數/剩餘數)。既有欄位名與語意 SHALL NOT 改變。無 `[M]` 任務的 change,三個 code 欄位之值 SHALL 與全量計數一致。

#### Scenario: 手動任務上線 payload

- **WHEN** 對含一個未勾 `[M]` 任務、九個已勾寫碼任務的 change 執行 instructions apply --json
- **THEN** 該 `[M]` 任務項 manual=true 且無 parallel 欄位,progress 為 total=10、complete=9、remaining=1、codeTotal=9、codeComplete=9、codeRemaining=0

#### Scenario: 無手動任務時欄位一致

- **WHEN** 對不含 `[M]` 任務的 change 執行 instructions apply --json
- **THEN** codeTotal=total、codeComplete=complete、codeRemaining=remaining,任務項欄位集合為 id/description/done/manual

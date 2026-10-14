## ADDED Requirements

### Requirement: 審查規則寫入後的技能檔過期提示

speclink workflow-config review <critical|warning|suggestion> --stdin 與 speclink workflow-config review baseline <extend|replace> 實際寫入成功後，CLI SHALL 以技能檔過期探測（workspace-tools「技能檔過期探測」）檢查本機工作區的技能檔：探測整體不是較新、也不是無法判定，且逐工具結果中至少一個工具判為過期時，CLI SHALL 於 stderr 輸出恰一行以 `speclink: warning: ` 開頭、不帶 ANSI 色彩的提示——內容 SHALL 列出每個過期工具的名稱與其技能版號、引擎現版號，說明舊版審查技能可能不套用審查規則，並指示執行 speclink update。stdout SHALL 維持既有的單行成功訊息，exit code SHALL 為 0。

以下情況 SHALL NOT 輸出該提示：所有工具的技能檔為現版或整組缺失；探測整體為較新（工作區檔案領先引擎，update 會降級）或無法判定；帶 --dry-run（沒有寫入，也 SHALL NOT 探測）；workflow-config set、context、rules 的寫入（它們由 CLI 在指令注入時讀取，技能檔新舊不影響生效）。remote 模式寫入成功後 SHALL 以本機工作區——解析 store 模式時找到的工作區——的技能檔判定，條件與本機模式相同。探測失敗 SHALL NOT 改變寫入結果與 exit code；提示只是診斷，寫入不回滾。

#### Scenario: 技能檔過期時寫入後提示

- **GIVEN** .speclink.yaml 的 tools 為 claude，.claude/skills/ 下技能檔的 frontmatter 版號舊於引擎現版
- **WHEN** 執行 speclink workflow-config review warning --stdin 並提供一行規則
- **THEN** exit code 0；review.warning 已寫入；stdout 恰為單行成功訊息；stderr 恰有一行以 `speclink: warning: ` 開頭的提示，含 claude、該技能版號、引擎現版號與 speclink update

##### Example: 提示行

| 工作區技能檔 | 引擎現版 | stderr 提示行 |
| ------------ | -------- | ------------- |
| claude v1.44.0 | v1.45.0 | speclink: warning: skill files are older than this engine (claude v1.44.0; engine v1.45.0) — an older review skill may not apply these rules; run `speclink update` |
| claude v1.44.0、codex v1.43.0 | v1.45.0 | speclink: warning: skill files are older than this engine (claude v1.44.0, codex v1.43.0; engine v1.45.0) — an older review skill may not apply these rules; run `speclink update` |

#### Scenario: 一個工具缺失、另一個過期時仍提示過期的工具

- **GIVEN** tools 為 claude 與 codex，.claude/skills/ 技能版號舊於現版，.agents/skills/ 下無任何 speclink- 技能檔（探測整體為缺失）
- **WHEN** 執行 speclink workflow-config review baseline replace
- **THEN** exit code 0；stderr 的提示列出 claude 與其技能版號，不列 codex

#### Scenario: 技能檔現版或整組缺失時不提示

- **WHEN** 分別在技能檔為現版、以及 tools 宣告 claude 但 .claude/skills/ 下無任何 speclink- 技能檔的工作區，執行 speclink workflow-config review critical --stdin 並提供一行規則
- **THEN** 兩者 exit code 0、stdout 單行成功訊息；stderr 都不含 `speclink: warning: skill files are older` 提示

#### Scenario: 工作區較新時不提示

- **GIVEN** 工作區技能檔版號新於引擎現版
- **WHEN** 執行 speclink workflow-config review suggestion --stdin 並提供一行規則
- **THEN** exit code 0；stderr 不含過期提示

#### Scenario: dry-run 與其他寫入子指令不提示

- **GIVEN** 工作區技能檔版號舊於引擎現版
- **WHEN** 分別執行 speclink workflow-config review warning --stdin --dry-run、speclink workflow-config set tdd true、speclink workflow-config rules tasks --stdin
- **THEN** 三者 stderr 都不含過期提示；dry-run 的 openspec/config.yaml 逐位元不變

#### Scenario: remote 模式以本機工作區判定

- **GIVEN** remote 模式的工作區，.speclink.yaml 的 tools 含 claude，本機 .claude/skills/ 技能版號舊於引擎現版
- **WHEN** 執行 speclink workflow-config review critical --stdin 並寫入 server 成功
- **THEN** exit code 0；stdout 單行成功訊息；stderr 含列出 claude 的過期提示

### Requirement: dry-run 差異只標示實際改動的行

workflow-config 各寫入子指令（set、context、rules、review）帶 --dry-run 時於 stdout 印出的 unified diff SHALL 為單一 hunk：以 `--- a/<label>`、`+++ b/<label>` 兩行開頭，接一行 `@@ -<舊側起點,行數> +<新側起點,行數> @@` 標頭，hunk 前後各帶至多三行 context。hunk 內的行 SHALL 依新舊文字的最長共同子序列逐行對齊：對齊上的行（未改動且相對順序不變）SHALL 以 context 行（前綴一個空白）呈現，SHALL NOT 以成對的 `-` 與 `+` 出現；其餘只在舊文字一側的行 SHALL 以 `-` 開頭，只在新文字一側的行 SHALL 以 `+` 開頭——被刪除、新增或移動位置的行才帶 `-`／`+`；同一位置同時有刪除與新增時，`-` 行 SHALL 先於 `+` 行。標頭的舊側行數 SHALL 等於 hunk 內 context 行數加 `-` 行數，新側行數 SHALL 等於 context 行數加 `+` 行數。新舊文字相同時 SHALL 輸出空字串。只有單一連續改動時，輸出 SHALL 與只標示該段改動的單一 hunk 相同。--dry-run 的其他行為（不寫檔、exit code 0、本機與 remote 模式共用同一改寫）不變。

#### Scenario: 兩處改動之間的未改行以 context 呈現

- **GIVEN** openspec/config.yaml 的 rules.tasks 依序為三條規則「甲」「乙」「丙」
- **WHEN** 執行 speclink workflow-config rules tasks --stdin --dry-run，stdin 依序為「甲二」「乙」「丙」「丁」
- **THEN** exit code 0；stdout 的 diff 中「乙」「丙」兩條只以 context 行出現；`-` 行恰為「甲」那一條，`+` 行恰為「甲二」與「丁」兩條；openspec/config.yaml 逐位元不變

##### Example: rules.tasks 的差異

- **GIVEN** rules.tasks 為 `- 甲`、`- 乙`、`- 丙` 三行
- **WHEN** 以「甲二、乙、丙、丁」整節代換並帶 --dry-run
- **THEN** hunk 內依序為 `-  - 甲`、`+  - 甲二`、`   - 乙`、`   - 丙`、`+  - 丁`，其前後為至多三行未改的 context

#### Scenario: hunk 標頭行數與內容一致

- **WHEN** 對任一寫入子指令帶 --dry-run 產生非空 diff
- **THEN** @@ 標頭的舊側行數等於 hunk 內以空白或 `-` 開頭的行數，新側行數等於以空白或 `+` 開頭的行數

#### Scenario: 單一連續改動的輸出不變

- **WHEN** 執行 speclink workflow-config set locale ja --dry-run（只改 locale 一行）
- **THEN** stdout 的 diff 只有一行 `-` 與一行 `+`，其餘為 context，與改版前輸出位元級相同

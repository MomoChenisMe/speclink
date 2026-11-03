## MODIFIED Requirements

### Requirement: plan 動詞輸出波次與阻擋清單

`speclink plan [--strict-overlap]` SHALL 為唯讀查詢。`--json` 時 stdout SHALL 為單一 JSON 物件：`waves`（陣列，每項含 `index` 整數與 `changes` 字串陣列，陣列內為該波配置順序）、`changes`（陣列，依整體配置順序，每項含 `name` 字串、`wave` 整數、`stage` 字串三值 proposed／in-progress／ready、`dependsOn` 字串陣列、`overlaps` 陣列（目錄級，每項 `change` 字串與 `capabilities` 字串陣列升冪，原樣保留）、`blockedBy` 字串陣列（只含作用中宣告前置；`--strict-overlap` 時另含配置在前的目錄級重疊者）、`ready` 布林、`requirementOverlap` 陣列（每項 `change`、`capability`、`requirement` 字串、`ownOperation` 與 `otherOperation` 字串四值 ADDED／MODIFIED／REMOVED／RENAMED、`conflict` 布林；依對方配置順序、同一對方依 capability 再依 requirement 升冪）、`archiveAfter` 字串陣列（依配置順序））、`next`（字串或 null）、`skipped`（陣列，每項 `change` 與 `reason` 字串，空時為 `[]`）、`unpromotedCuts`（陣列，每項含 `discussion`、`cut`、`title` 字串與 `changes`、`unpromoted` 字串陣列；列出條件、順序與欄位語意依「plan 列出討論結論中尚未轉出的刀」，空時為 `[]`）。人眼輸出 SHALL 逐波印 `Wave N` 標題（同波兩個以上附 `(parallel)`）、每個 change 一行含名稱與階段，尾註依序為有阻擋時的 `blocked by: …`、有封存順序時的 `archive after: …`、有衝突時的 `conflicts with: …`，各自空時不印；有尚未轉出的刀時 SHALL 於 `next:` 行之前印 `Unpromoted cuts` 段（格式依「plan 列出討論結論中尚未轉出的刀」），沒有時 SHALL NOT 印、其餘輸出位元級不變；末行 `next: <name>` 或 `next: none`；`--no-color` 下 SHALL 無 ANSI 序列。無作用中 change 時 `waves` 與 `changes` SHALL 為空陣列、`next` 為 null，`unpromotedCuts` 照常列出。depends_on 成環時 SHALL 以非零 exit code 結束，stderr 為 `dependency cycle: <a> -> <b> -> … -> <a>`，stdout SHALL 為空。在主 checkout 且 worktree 政策開啟時，plan 的階段判定 SHALL 讀取各 worktree 內的開工標記與任務進度，與 list 動詞同一聚合面。remote 模式下的臂 SHALL 依 verb-contract「模式分岔的單點宣告」對 plan 的歸屬決定；`--strict-overlap` 只限本機模式，remote 模式 SHALL 以固定訊息明確拒絕且不發出任何請求（verb-contract「動詞人眼輸出的兩模式同形」明文分歧第 6 項）。

#### Scenario: JSON 輸出形狀

- **WHEN** 於含兩個提案中 change 的 fs 專案執行 speclink plan --json
- **THEN** stdout 為含 waves、changes、next、skipped、unpromotedCuts 五鍵的物件，changes 每項含 name、wave、stage、dependsOn、overlaps、blockedBy、ready、requirementOverlap、archiveAfter 九鍵，exit code 為 0

#### Scenario: 人眼輸出與 --no-color

- **WHEN** 執行 speclink plan --no-color
- **THEN** stdout 含 `Wave 1` 行與 `next:` 行，不含 ANSI 序列，exit code 為 0

#### Scenario: 依賴成環

- **WHEN** change a 的 depends_on 含 b、b 的 depends_on 含 a，執行 speclink plan --json
- **THEN** exit code 非零，stderr 含 `dependency cycle: a -> b -> a`，stdout 為空

#### Scenario: 壞 metadata 列入 skipped

- **WHEN** 某 change 的 .meta.yaml 無法解析，執行 speclink plan --json
- **THEN** 該 change 不在 waves 與 changes 內，skipped 含其名稱與原因，其餘 change 照常配置，exit code 為 0

#### Scenario: 人眼輸出的封存順序與衝突尾註

- **WHEN** b 的 archiveAfter 為 [a]、c 與 d 互為 ADDED 同名衝突，執行 speclink plan --no-color
- **THEN** b 的行尾含 `archive after: a`，c 的行尾含 `conflicts with: d`，兩者行內皆不含 `blocked by:`

##### Example: 一個 change 三種尾註的順序

- **GIVEN** e 的 depends_on 含 a、與 b MODIFIED 同一 requirement、與 c ADDED 同名 requirement
- **WHEN** 執行 speclink plan --no-color
- **THEN** e 那一行去掉行首縮排後為 `• e [proposed] — blocked by: a — archive after: b — conflicts with: c`（行首的 `• <名稱> [<階段>]` 沿用既有形狀，本變更只追加尾註）

## ADDED Requirements

### Requirement: plan 列出討論結論中尚未轉出的刀

`speclink plan` SHALL 讀取每份未封存討論的結論，解析其 Decision 欄位裡規劃的刀，並在輸出中列出尚未轉出的刀；解析與判定 SHALL 由引擎執行一次，本機 CLI 與 server 的 GET /plan SHALL 共用同一份實作。

Decision 欄位為 Conclusion 內以 `**Decision**:` 起頭的行，到下一個以 `**Rationale**:`、`**Rejected alternatives**:`、`**Deferred**:`、`**Capture to**:` 或 `**Next**:` 起頭的行之前。一把刀 SHALL 為 Decision 內行首為 `- ` 的頂層列點，其首行以成對的粗體開頭，且粗體內去掉反引號片段並壓縮空白後，以下列三種寫法之一起頭：`cut` 加以數字起頭的編號（不分大小寫，例 `cut 2`、`cut 3a`；`cutover` 不算）、`第 <編號> 刀`、`刀` 加編號（例 `刀二`）。刀的各欄位 SHALL 為：`cut` 為上述起頭片段的原文；`changes` 為粗體內不含空白的反引號片段，依出現順序去重；`title` 在粗體內編號之後還有文字時取該文字（去掉前導空白與 `:：、—–-`），否則取粗體之後去掉前導冒號的文字、遇到第一個「——」即截斷。

粗體內沒有任何反引號變更名的刀 SHALL NOT 列出（無從判定是否轉出）。一把刀 SHALL 在至少一個變更名不在該討論 `promoted_to` 時列出；`unpromoted` 為其中不在 `promoted_to` 的變更名，依 `changes` 的順序。列出順序 SHALL 依 `speclink discuss list` 的討論順序，同一討論依結論內的刀序。已封存的討論、沒有 Conclusion 或沒有 Decision 欄位的討論 SHALL NOT 產生項目；一份討論的結論讀不到時 SHALL 略過該討論，SHALL NOT 讓 plan 失敗。plan SHALL NOT 推算刀與刀之間的依賴，也 SHALL NOT 改寫任何討論檔。

JSON 輸出的每一項 SHALL 含 `discussion`（討論 slug）、`cut`、`title`、`changes`、`unpromoted` 五鍵。人眼輸出 SHALL 在有項目時於 `next:` 行之前印粗體標題 `Unpromoted cuts`，每把刀一行：兩格縮排、`•`、空格、`<slug> <cut>: <unpromoted 以 ", " 相接>`，title 非空時再接 ` — <title>`；`--no-color` 下 SHALL 無 ANSI 序列；沒有項目時 SHALL NOT 印這一段。

#### Scenario: 列出尚未轉出的刀

- **WHEN** 未封存討論 talk-a 的結論 Decision 含 ``- **cut 1 `add-a`**: 第一刀``、``- **cut 2 `add-b`**: 第二刀``，promoted_to 為 add-a，執行 speclink plan --json
- **THEN** unpromotedCuts 為 `[{"discussion":"talk-a","cut":"cut 2","title":"第二刀","changes":["add-b"],"unpromoted":["add-b"]}]`，exit code 為 0

#### Scenario: 全部轉出的討論不產生項目

- **WHEN** 上例的 promoted_to 為 add-a, add-b，執行 speclink plan --json
- **THEN** unpromotedCuts 為 `[]`

#### Scenario: 一刀多個變更時只列還沒轉出的

- **WHEN** 結論含 ``- **cut 1 `add-a` `add-c`**: 兩個變更``，promoted_to 為 add-a，執行 speclink plan --json
- **THEN** 該項 changes 為 ["add-a","add-c"]、unpromoted 為 ["add-c"]

#### Scenario: 粗體內沒有變更名的刀不列出

- **WHEN** 結論只含 ``- **cut 2 外殼**（`add-x`、`add-y`）``，promoted_to 為空，執行 speclink plan --json
- **THEN** unpromotedCuts 為 `[]`

#### Scenario: 已封存的討論不產生項目

- **WHEN** 只有已封存的討論含未轉出的刀，執行 speclink plan --json
- **THEN** unpromotedCuts 為 `[]`

#### Scenario: 人眼輸出的 Unpromoted cuts 段

- **WHEN** 有一項尚未轉出的刀（talk-a 的 cut 2，unpromoted 為 add-b，title 為 第二刀）且有一個提案中變更 add-a，執行 speclink plan --no-color
- **THEN** stdout 依序為 `Wave 1`、`  • add-a [proposed]`、`Unpromoted cuts`、`  • talk-a cut 2: add-b — 第二刀`、`next: add-a` 五行，不含 ANSI 序列

#### Scenario: 沒有尚未轉出的刀時人眼輸出不變

- **WHEN** 所有討論的刀都已轉出，執行 speclink plan --no-color
- **THEN** stdout 不含 `Unpromoted cuts`，與加入本需求前的輸出逐位元相同

##### Example: 分刀解析

| Decision 內的頂層列點首行 | cut | changes | title |
| ------------------------- | --- | ------- | ----- |
| ``**cut 2 `plan-unpromoted-cuts`**: 引擎解析討論結論的分刀列`` | cut 2 | plan-unpromoted-cuts | 引擎解析討論結論的分刀列 |
| ``**Cut 3a `add-x`**: 範圍——補充說明`` | Cut 3a | add-x | 範圍 |
| ``**第 2 刀 `add-y`**`` | 第 2 刀 | add-y | （空） |
| ``**刀二 收尾 `add-z`**`` | 刀二 | add-z | 收尾 |
| ``**cut 1 `speclink plan` `add-w`**: 說明`` | cut 1 | add-w | 說明 |
| ``**cutover `add-v`**: 說明`` | 不是刀 | — | — |
| ``**cut 2 外殼**（`add-x`）`` | 是刀，但粗體內沒有變更名，不列出 | — | — |
| `` `add-u` 不以粗體開頭`` | 不是刀 | — | — |

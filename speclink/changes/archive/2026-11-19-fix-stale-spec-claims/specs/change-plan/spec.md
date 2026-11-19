## MODIFIED Requirements

### Requirement: plan 動詞輸出波次與阻擋清單

<!-- BEFORE: --strict-overlap 只限本機模式，remote 模式以固定訊息拒絕且不發出任何請求 -->

`speclink plan [--strict-overlap]` SHALL 為唯讀查詢。`--json` 時 stdout SHALL 為單一 JSON 物件：`waves`（陣列，每項含 `index` 整數與 `changes` 字串陣列，陣列內為該波配置順序）、`changes`（陣列，依整體配置順序，每項含 `name` 字串、`wave` 整數、`stage` 字串三值 proposed／in-progress／ready、`dependsOn` 字串陣列、`overlaps` 陣列（目錄級，每項 `change` 字串與 `capabilities` 字串陣列升冪，原樣保留）、`blockedBy` 字串陣列（只含作用中宣告前置；`--strict-overlap` 時另含配置在前的目錄級重疊者）、`ready` 布林、`requirementOverlap` 陣列（每項 `change`、`capability`、`requirement` 字串、`ownOperation` 與 `otherOperation` 字串四值 ADDED／MODIFIED／REMOVED／RENAMED、`conflict` 布林；依對方配置順序、同一對方依 capability 再依 requirement 升冪）、`archiveAfter` 字串陣列（依配置順序））、`next`（字串或 null）、`skipped`（陣列，每項 `change` 與 `reason` 字串，空時為 `[]`）、`unpromotedCuts`（陣列，每項含 `discussion`、`cut`、`title` 字串與 `changes`、`unpromoted` 字串陣列；列出條件、順序與欄位語意依「plan 列出討論結論中尚未轉出的刀」，空時為 `[]`）。人眼輸出 SHALL 逐波印 `Wave N` 標題（同波兩個以上附 `(parallel)`）、每個 change 一行含名稱與階段，尾註依序為有阻擋時的 `blocked by: …`、有封存順序時的 `archive after: …`、有衝突時的 `conflicts with: …`，各自空時不印；有尚未轉出的刀時 SHALL 於 `next:` 行之前印 `Unpromoted cuts` 段（格式依「plan 列出討論結論中尚未轉出的刀」），沒有時 SHALL NOT 印、其餘輸出位元級不變；末行 `next: <name>` 或 `next: none`；`--no-color` 下 SHALL 無 ANSI 序列。無作用中 change 時 `waves` 與 `changes` SHALL 為空陣列、`next` 為 null，`unpromotedCuts` 照常列出。depends_on 成環時 SHALL 以非零 exit code 結束，stderr 為 `dependency cycle: <a> -> <b> -> … -> <a>`，stdout SHALL 為空。在主 checkout 且 worktree 政策開啟時，plan 的階段判定 SHALL 讀取各 worktree 內的開工標記與任務進度，與 list 動詞同一聚合面。remote 模式下的臂 SHALL 依 verb-contract「模式分岔的單點宣告」對 plan 的歸屬決定；`--strict-overlap` 在 remote 模式 SHALL 同樣可用：CLI SHALL 把旗標送到 Server、由 Server 算出結果，人眼輸出與 `--json` 輸出 SHALL 與本機模式同形（verb-contract「動詞人眼輸出的兩模式同形」的同形範圍含 plan 的 `--strict-overlap`），SHALL NOT 讀取本機的規格資料夾。Server 的回應沒有表明它套用了這個旗標時（不認得旗標的舊版 Server），CLI SHALL 以非零 exit code 結束，stdout SHALL 為空，stderr SHALL 含 `this server does not support plan --strict-overlap`，SHALL NOT 把一般的計畫當成嚴格重疊的結果印出。

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

#### Scenario: remote 模式的 --strict-overlap 由 Server 算出

- **WHEN** Server 上有兩個提案中的變更 add-a 與 add-b，兩者的 delta 動到同一個 capability 的不同 requirement，於 remote 模式執行 speclink plan --strict-overlap --json
- **THEN** exit code 為 0；stdout 的 JSON 裡 add-b 在第 2 波、`blockedBy` 為 `["add-a"]`；過程只向 Server 查詢，沒有讀取本機的規格資料夾

#### Scenario: 舊版 Server 不認得 --strict-overlap

- **WHEN** 於 remote 模式分別執行 speclink plan --strict-overlap 與加 --json 的同一指令，Server 回的是一般的計畫、沒有表明套用了旗標
- **THEN** 兩者 exit code 都非零，stdout 為空，stderr 含 `this server does not support plan --strict-overlap`

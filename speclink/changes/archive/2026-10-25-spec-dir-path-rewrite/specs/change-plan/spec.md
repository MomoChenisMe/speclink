## MODIFIED Requirements

### Requirement: plan 動詞輸出波次與阻擋清單

<!-- BEFORE: 以 openspec/ 與 .openspec.yaml 描述規格資料夾與變更資料檔 -->

`speclink plan [--strict-overlap]` SHALL 為唯讀查詢。`--json` 時 stdout SHALL 為單一 JSON 物件：`waves`（陣列，每項含 `index` 整數與 `changes` 字串陣列，陣列內為該波配置順序）、`changes`（陣列，依整體配置順序，每項含 `name` 字串、`wave` 整數、`stage` 字串三值 proposed／in-progress／ready、`dependsOn` 字串陣列、`overlaps` 陣列（目錄級，每項 `change` 字串與 `capabilities` 字串陣列升冪，原樣保留）、`blockedBy` 字串陣列（只含作用中宣告前置；`--strict-overlap` 時另含配置在前的目錄級重疊者）、`ready` 布林、`requirementOverlap` 陣列（每項 `change`、`capability`、`requirement` 字串、`ownOperation` 與 `otherOperation` 字串四值 ADDED／MODIFIED／REMOVED／RENAMED、`conflict` 布林；依對方配置順序、同一對方依 capability 再依 requirement 升冪）、`archiveAfter` 字串陣列（依配置順序））、`next`（字串或 null）、`skipped`（陣列，每項 `change` 與 `reason` 字串，空時為 `[]`）。人眼輸出 SHALL 逐波印 `Wave N` 標題（同波兩個以上附 `(parallel)`）、每個 change 一行含名稱與階段，尾註依序為有阻擋時的 `blocked by: …`、有封存順序時的 `archive after: …`、有衝突時的 `conflicts with: …`，各自空時不印；末行 `next: <name>` 或 `next: none`；`--no-color` 下 SHALL 無 ANSI 序列。無作用中 change 時 `waves` 與 `changes` SHALL 為空陣列、`next` 為 null。depends_on 成環時 SHALL 以非零 exit code 結束，stderr 為 `dependency cycle: <a> -> <b> -> … -> <a>`，stdout SHALL 為空。在主 checkout 且 worktree 政策開啟時，plan 的階段判定 SHALL 讀取各 worktree 內的開工標記與任務進度，與 list 動詞同一聚合面。remote 模式下的臂 SHALL 依 verb-contract「模式分岔的單點宣告」對 plan 的歸屬決定；`--strict-overlap` 只限本機模式，remote 模式 SHALL 以固定訊息明確拒絕且不發出任何請求（verb-contract「動詞人眼輸出的兩模式同形」明文分歧第 6 項）。

#### Scenario: JSON 輸出形狀

- **WHEN** 於含兩個提案中 change 的 fs 專案執行 speclink plan --json
- **THEN** stdout 為含 waves、changes、next、skipped 四鍵的物件，changes 每項含 name、wave、stage、dependsOn、overlaps、blockedBy、ready、requirementOverlap、archiveAfter 九鍵，exit code 為 0

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

### Requirement: change depends 動詞寫入宣告依賴

<!-- BEFORE: 以 openspec/ 與 .openspec.yaml 描述規格資料夾與變更資料檔 -->

`speclink change depends <name> --on <other>... [--remove] [--json]` SHALL 把前置寫進該 change 的 `.meta.yaml` 頂層 `depends_on` 欄位（逗號清單，以 `, ` 串接），其餘位元組逐字保留、每行沿用自身行尾。守門 SHALL 依序：`<name>` 非作用中 change 即錯；非 `--remove` 時任一 `<other>` 已封存或不存在即錯（訊息分辨兩者），`--remove` 時 SHALL 允許移除指向已封存或不存在名稱的殘留項、只拒絕不合法名稱（空白、含路徑分隔符或 `..`）；`<other>` 等於 `<name>` 即錯；非 `--remove` 時加邊後若有經過 `<name>` 的環即錯並印環（自 `<name>` 起讀），別處既有的環 SHALL NOT 阻擋、由 plan 回報。作用中 change 的判定 SHALL 以 change 名逐字比對，SHALL NOT 依賴檔案系統的存在探針（大小寫不分的檔案系統、`archive` 目錄與空字串皆不得放行）。任一守門失敗 SHALL 以非零 exit code 結束且零寫入。多個 `--on` SHALL 於單次寫入完成；加已存在或移除不存在的邊 SHALL 不改檔且成功。`--remove` 後清單為空時 SHALL 整行移除。成功時 stdout SHALL 為一行 `✓ <name> depends on: …`（`--remove` 時 `✓ <name> no longer depends on: …`）；`--json` 時 stdout SHALL 為 `{ "change": string, "dependsOn": string[] }`（寫入後的完整清單）。壞 metadata 的 `<name>` SHALL 拒絕寫入。在主 checkout 且 worktree 政策開啟時，`<name>` 已映射到 worktree 者 SHALL 以該 worktree 副本為準——守門、成環檢查與寫入皆作用於副本，與 plan 同一聚合面；無映射時作用於 CLI 所在 checkout。remote 模式下的臂 SHALL 依 verb-contract「模式分岔的單點宣告」對 change 的歸屬決定。

#### Scenario: 寫入單一前置

- **WHEN** 對無 depends_on 的 change c 執行 speclink change depends c --on a（a 為作用中）
- **THEN** c 的 .meta.yaml 多一行 `depends_on: a`，其餘行逐位元不變，stdout 為 `✓ c depends on: a`，exit code 為 0

#### Scenario: 追加與移除

- **WHEN** c 已有 `depends_on: a`，依序執行 --on b 再 --on a --remove
- **THEN** 第一次後該行為 `depends_on: a, b`，第二次後為 `depends_on: b`，兩次 exit code 皆為 0

#### Scenario: 移除至空即整行移除

- **WHEN** c 只有 `depends_on: a`，執行 speclink change depends c --on a --remove
- **THEN** c 的 .meta.yaml 不再含 depends_on 行，其餘行逐位元不變

#### Scenario: 守門拒絕零寫入

- **WHEN** 分別執行 --on 指向自己、指向不存在的名稱、指向已封存的名稱，以及會成環的邊
- **THEN** 四次皆 exit code 非零、stderr 說明原因，.meta.yaml 逐位元不變

#### Scenario: 移除指向已封存名稱的殘留項

- **WHEN** c 有 `depends_on: old, ghost`，old 已封存、ghost 不存在，依序執行 speclink change depends c --on old --remove 再 --on ghost --remove
- **THEN** 兩次 exit code 皆為 0，第一次後該行為 `depends_on: ghost`，第二次後整行移除；再執行 --on old（不帶 --remove）仍以 exit code 非零拒絕且零寫入

#### Scenario: 名稱逐字比對

- **WHEN** 作用中 change 為 add-auth 與 c，分別執行 speclink change depends c --on Add-Auth、--on 空字串、--on archive
- **THEN** 三次皆 exit code 非零、stderr 說明沒有該名稱的作用中 change，c 的 .meta.yaml 逐位元不變

#### Scenario: 目標 change 在 worktree 內時寫入副本

- **WHEN** 主 checkout 且 worktree 政策開啟，add-dark-mode 已映射到 worktree，執行 speclink change depends add-dark-mode --on add-auth
- **THEN** worktree 副本的 .meta.yaml 多一行 `depends_on: add-auth`，主副本不變，隨後 speclink plan --json 的 add-dark-mode 之 dependsOn 與 blockedBy 皆含 add-auth

#### Scenario: CRLF meta 檔保留行尾

- **WHEN** c 的 .meta.yaml 以 CRLF 換行，執行 speclink change depends c --on a
- **THEN** 新增的 depends_on 行以 CRLF 結尾，既有各行的位元組不變

### Requirement: plan 與 change depends 的 remote 臂

<!-- BEFORE: 以 openspec/ 與 .openspec.yaml 描述規格資料夾與變更資料檔 -->

remote 模式下 `speclink plan` SHALL 呼叫 server 的 GET /plan 並把回應轉回引擎的 plan 報告後以 fs 模式同一渲染輸出，人眼與 --json 的 stdout 形狀 SHALL 與 fs 模式對同一內容一致；server 回 409（成環，reason refused）時 SHALL 以非零 exit code 結束、stderr 為 server 轉發的引擎訊息 `dependency cycle: …`、stdout 為空。remote 模式下 `speclink change depends` SHALL 呼叫 POST /changes/{name}/depends，成功時 stdout 與 fs 模式同一行 `✓ …`（--json 同形），server 的 404 與守門 409 SHALL 以 server 轉發的引擎單行訊息印於 stderr、非零 exit code。兩動詞在 remote 模式 SHALL NOT 讀寫本機 store。remote 拖排的宣告依賴檢查 SHALL 以引擎的純序列檢查函式對拖放後同階段、plan 有配置的名稱序列（plan 略過的壞 meta 卡不列入，與 local 同構）與 plan 回應的 dependsOn 圖執行，只計涉及被拖卡的配對，違反時 SHALL NOT 發出 PUT /board-order、SHALL 以與 local 相同的單行訊息呈現；plan 不可得（舊 server、成環、請求失敗）時 SHALL 略過此檢查。

#### Scenario: remote plan 同形

- **WHEN** 於 remote 模式執行 speclink plan --json，server 回兩個 change 的 plan
- **THEN** stdout 的 JSON 鍵與 fs 模式一致（waves、changes、next、skipped），exit code 為 0，未讀取本機 speclink/

#### Scenario: remote 成環

- **WHEN** 於 remote 模式執行 speclink plan，server 回 HTTP 409、reason 為 refused、message 為 `dependency cycle: add-a -> add-b -> add-a`
- **THEN** exit code 非零，stderr 含 `dependency cycle: add-a -> add-b -> add-a`，stdout 為空

#### Scenario: remote 寫入前置

- **WHEN** 於 remote 模式執行 speclink change depends add-b --on add-a
- **THEN** 對 server 發出 POST /changes/add-b/depends、body `{"on":["add-a"],"remove":false}`，stdout 為 `✓ add-b depends on: add-a`

#### Scenario: remote 拖排違反依賴不發 PUT

- **WHEN** remote 分頁中 add-b 的 dependsOn 含 add-a，使用者把 add-b 拖到 add-a 之前放開
- **THEN** 桌面顯示單行錯誤 `cannot move 'add-b' there: it depends on add-a`、未發出 PUT /board-order、看板刷新回 add-a 在前

### Requirement: change rank 動詞寫入看板順序鍵

<!-- BEFORE: 以 openspec/ 與 .openspec.yaml 描述規格資料夾與變更資料檔 -->
<!-- REMOVED-SCENARIO: 缺 .openspec.yaml 的 change 零寫入拒絕 -->

`speclink change rank <name> (--before <other> | --after <other>) [--force] [--json]` SHALL 把 `<name>` 的 `board_rank` 改寫為排在 `<other>` 之前或之後的位置，使 plan 的配置順序與看板欄內順序同步改變。`--before` 與 `--after` SHALL 互斥且必擇一。守門 SHALL 依序：`<name>` 或 `<other>` 非作用中 change、metadata 壞掉、或兩者相同即錯；兩者階段不同即錯（rank 只在同欄內有意義）；`<name>` 已有 `board_rank` 且未帶 `--force` 即錯，訊息含 `already has a board rank` 與 `--force` 字樣；以新 rank 重跑 rank 移動的依賴檢查被擋即錯並印出 must_follow／must_precede。要寫入的 change（`<name>` 或補章對象）缺 `.meta.yaml` 時即錯，訊息含該檔路徑與 `is missing`。任一守門失敗 SHALL 以非零 exit code 結束且零寫入——所有檢查皆在第一筆寫入之前。新 rank SHALL 以看板排序鍵的中點演算（小寫英文字母、位元組字典序）落在 `<other>` 與其鄰居之間，鄰居與欄序 SHALL 取看板顯示序（plan 配置順序投影到該欄，即經宣告依賴修正後的基底序）；該欄（`<name>` 除外）任一 change 缺 rank、或 rank 序與顯示序不一致時，SHALL 先依顯示序對整欄補章（`<name>` 除外），再寫 `<name>`——欄序、補章、中點與依賴檢查 SHALL 與看板拖排為同一份引擎實作（board-card-order「欄內存在缺 rank 卡時整欄補章」）；寫入順序 SHALL 為補章逐一寫入、最後寫 `<name>`，中途失敗 SHALL 停止並回報、已寫入的補章保留（部分補章的欄仍是合法順序，重跑可續行）。每次寫入 SHALL 沿 depends 動詞的文字手術規則：其餘位元組逐字保留、純量經跳脫。成功時 stdout SHALL 為一行 `✓ <name> ranked before <other>`（或 `after`）；`--json` 時 stdout SHALL 為 `{ "change": string, "anchor": string, "position": "before" | "after" }`，SHALL NOT 含 rank 值。remote 模式 SHALL 以非零 exit code 明確拒絕，stderr 說明 remote 的順序由 board resource 決定，SHALL NOT 發出任何請求。在主 checkout 且 worktree 政策開啟時，欄序與守門 SHALL 讀 plan 同一視角（主 checkout 的 change 名冊，映射到 worktree 者取其副本的值；worktree 分支後才在主 checkout 新建的 change 同樣在欄內），每筆寫入（補章與 `<name>`）SHALL 落在該 change 自己的副本——有映射寫其 worktree 副本、無映射寫主 checkout，與桌面卡片拖排同一路由。

#### Scenario: 無 rank 的 change 排到另一個之前

- **WHEN** 同為提案中的 a（rank: n）與 c（無 rank）之間執行 speclink change rank c --before a，欄內只有這兩個 change
- **THEN** c 的 .meta.yaml 多一行 board_rank，其值字典序小於 n，stdout 為 `✓ c ranked before a`，exit code 為 0，隨後 speclink plan --json 的 changes 順序 c 在 a 前

#### Scenario: 已有 rank 未帶 --force 被拒

- **WHEN** c 已有 board_rank，執行 speclink change rank c --after a
- **THEN** exit code 非零，stderr 含 `already has a board rank` 與 `--force`，c 的檔案逐位元不變

#### Scenario: 帶 --force 覆寫既有 rank

- **WHEN** c 已有 board_rank，執行 speclink change rank c --after a --force
- **THEN** c 的 board_rank 改為字典序大於 a 的值，exit code 為 0

#### Scenario: 跨宣告依賴被拒

- **WHEN** c 的 depends_on 含 a，執行 speclink change rank c --before a
- **THEN** exit code 非零，stderr 含 `cannot move 'c' there` 與 `it depends on a`，零寫入

#### Scenario: 缺 rank 的欄先整欄補章

- **WHEN** 提案中欄內 a、b、c 皆無 rank，基底序為 a、b、c，執行 speclink change rank c --before b
- **THEN** a 與 b 各獲得一個 rank（a 小於 b），c 的 rank 落在 a 與 b 之間，plan 的順序為 a、c、b

#### Scenario: 不同階段被拒

- **WHEN** a 為進行中、c 為提案中，執行 speclink change rank c --before a
- **THEN** exit code 非零，stderr 說明兩者不在同一欄，零寫入

#### Scenario: remote 模式拒絕

- **WHEN** 於 remote 模式執行 speclink change rank c --before a
- **THEN** exit code 非零，stderr 說明順序由 board resource 決定，且未發出任何請求

#### Scenario: 缺 .meta.yaml 的 change 零寫入拒絕

- **WHEN** 提案中欄內 p1、p3 有 .meta.yaml 且皆無 rank，p2 的目錄只有 proposal.md，分別執行 speclink change rank p2 --before p1 與 speclink change rank p3 --before p1
- **THEN** 兩次皆 exit code 非零、stderr 含 `speclink/changes/p2/.meta.yaml is missing`，p1 與 p3 皆未被補章

#### Scenario: worktree 內的 change 與主 checkout 新建的錨點

- **WHEN** 主 checkout 且 worktree 政策開啟，x 已映射到 worktree、y 在 worktree 分支後才於主 checkout 新建，兩者同為提案中且皆無 rank，執行 speclink change rank y --after x
- **THEN** exit code 為 0，x 的補章寫入 x 的 worktree 副本、主 checkout 內 x 的 .meta.yaml 逐位元不變，y 的 rank 寫入主 checkout，隨後 plan 的順序 x 在 y 前；反向執行 speclink change rank x --before y 同樣成功

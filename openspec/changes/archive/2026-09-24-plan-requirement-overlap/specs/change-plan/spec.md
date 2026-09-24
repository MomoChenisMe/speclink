## ADDED Requirements

### Requirement: change rank 動詞寫入看板順序鍵

`speclink change rank <name> (--before <other> | --after <other>) [--force] [--json]` SHALL 把 `<name>` 的 `board_rank` 改寫為排在 `<other>` 之前或之後的位置，使 plan 的配置順序與看板欄內順序同步改變。`--before` 與 `--after` SHALL 互斥且必擇一。守門 SHALL 依序：`<name>` 或 `<other>` 非作用中 change、metadata 壞掉、或兩者相同即錯；兩者階段不同即錯（rank 只在同欄內有意義）；`<name>` 已有 `board_rank` 且未帶 `--force` 即錯，訊息含 `already has a board rank` 與 `--force` 字樣；以新 rank 重跑 rank 移動的依賴檢查被擋即錯並印出 must_follow／must_precede。要寫入的 change（`<name>` 或補章對象）缺 `.openspec.yaml` 時即錯，訊息含該檔路徑與 `is missing`。任一守門失敗 SHALL 以非零 exit code 結束且零寫入——所有檢查皆在第一筆寫入之前。新 rank SHALL 以看板排序鍵的中點演算（小寫英文字母、位元組字典序）落在 `<other>` 與其鄰居之間，鄰居與欄序 SHALL 取看板顯示序（plan 配置順序投影到該欄，即經宣告依賴修正後的基底序）；該欄（`<name>` 除外）任一 change 缺 rank、或 rank 序與顯示序不一致時，SHALL 先依顯示序對整欄補章（`<name>` 除外），再寫 `<name>`——欄序、補章、中點與依賴檢查 SHALL 與看板拖排為同一份引擎實作（board-card-order「欄內存在缺 rank 卡時整欄補章」）；寫入順序 SHALL 為補章逐一寫入、最後寫 `<name>`，中途失敗 SHALL 停止並回報、已寫入的補章保留（部分補章的欄仍是合法順序，重跑可續行）。每次寫入 SHALL 沿 depends 動詞的文字手術規則：其餘位元組逐字保留、純量經跳脫。成功時 stdout SHALL 為一行 `✓ <name> ranked before <other>`（或 `after`）；`--json` 時 stdout SHALL 為 `{ "change": string, "anchor": string, "position": "before" | "after" }`，SHALL NOT 含 rank 值。remote 模式 SHALL 以非零 exit code 明確拒絕，stderr 說明 remote 的順序由 board resource 決定，SHALL NOT 發出任何請求。在主 checkout 且 worktree 政策開啟時，欄序與守門 SHALL 讀 plan 同一視角（主 checkout 的 change 名冊，映射到 worktree 者取其副本的值；worktree 分支後才在主 checkout 新建的 change 同樣在欄內），每筆寫入（補章與 `<name>`）SHALL 落在該 change 自己的副本——有映射寫其 worktree 副本、無映射寫主 checkout，與桌面卡片拖排同一路由。

#### Scenario: 無 rank 的 change 排到另一個之前

- **WHEN** 同為提案中的 a（rank: n）與 c（無 rank）之間執行 speclink change rank c --before a，欄內只有這兩個 change
- **THEN** c 的 .openspec.yaml 多一行 board_rank，其值字典序小於 n，stdout 為 `✓ c ranked before a`，exit code 為 0，隨後 speclink plan --json 的 changes 順序 c 在 a 前

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

#### Scenario: 缺 .openspec.yaml 的 change 零寫入拒絕

- **WHEN** 提案中欄內 p1、p3 有 .openspec.yaml 且皆無 rank，p2 的目錄只有 proposal.md，分別執行 speclink change rank p2 --before p1 與 speclink change rank p3 --before p1
- **THEN** 兩次皆 exit code 非零、stderr 含 `openspec/changes/p2/.openspec.yaml is missing`，p1 與 p3 皆未被補章

#### Scenario: worktree 內的 change 與主 checkout 新建的錨點

- **WHEN** 主 checkout 且 worktree 政策開啟，x 已映射到 worktree、y 在 worktree 分支後才於主 checkout 新建，兩者同為提案中且皆無 rank，執行 speclink change rank y --after x
- **THEN** exit code 為 0，x 的補章寫入 x 的 worktree 副本、主 checkout 內 x 的 .openspec.yaml 逐位元不變，y 的 rank 寫入主 checkout，隨後 plan 的順序 x 在 y 前；反向執行 speclink change rank x --before y 同樣成功

## MODIFIED Requirements

### Requirement: 執行順序的基底與拓樸修正

<!-- BEFORE: 波次同時被宣告前置與 delta capability（目錄級）重疊推後；缺 rank 者只依 created 排 -->

引擎 SHALL 對所有未封存的 change 算出一個全序的配置順序與波次。基底順序 SHALL 依三鍵排列：(1) 階段——已就緒（任務全勾且總數大於零）在前、進行中（有 started_at 或任一已勾任務）次之、提案中最後；(2) 同階段內有 board_rank 者排前、以位元組字典序升冪，缺 board_rank 者排後、彼此依「被依賴數多者在前 → task 總數少者在前（無 tasks.md 或其中沒有任何 task 者殿後）→ created 升冪、缺 created 者殿後」；(3) 同值以 change 名字典序決斷。被依賴數 SHALL 為作用中且可配置的 change 中 depends_on 含該 change 者的個數（同名去重）。拓樸修正 SHALL 沿基底順序反覆配置「所有作用中宣告前置皆已配置」的第一個 change；宣告前置 SHALL 只計 depends_on 中仍未封存且 metadata 可解析的名稱，指到不存在、已封存或 metadata 壞掉（列入 skipped）的名稱 SHALL 視為已滿足。change 的波次 SHALL 為 1 加上其作用中宣告前置的最大波次；無前置者 SHALL 為第 1 波。delta 重疊 SHALL NOT 推後波次、SHALL NOT 構成有向邊、SHALL NOT 進入 blockedBy。requirement 級重疊 SHALL 自 delta 內文判定：同一 capability 下同一個 requirement 名，兩個 change 各以 ADDED／MODIFIED／REMOVED／RENAMED（FROM 與 TO 皆計，輸出皆記為 RENAMED）之一動到。每一方對該名稱的角色 SHALL 為下列之一：MODIFIED 為「修改」（封存前後該名稱都在）；REMOVED 與 RENAMED 的 FROM 端為「拿走」（封存前在、封存後不在）；ADDED 與 RENAMED 的 TO 端為「帶進」（封存前不在、封存後在）。角色的先後 SHALL 依該名稱此刻是否在正式規格（該 capability 的 canonical spec）中：在時為「修改 → 拿走 → 帶進」，不在時（含該 capability 尚無正式規格）為「帶進 → 修改 → 拿走」。雙方角色不同時為重疊，角色在後者「等」角色在前者，角色在前者 SHALL 先封存（不論基底順序）；雙方皆為「修改」時為重疊、先後照封存序；雙方皆為「拿走」或皆為「帶進」時 SHALL 標為衝突（conflict 為 true）。archiveAfter SHALL 以一對 change 整體判定：任一同名 requirement 上我方等對方時，對方 SHALL 列入我方 archiveAfter；否則任一同名 requirement 上對方等我方時 SHALL NOT 列；否則有雙方皆為「修改」的同名 requirement 且對方在封存序中在前時才列。封存序 SHALL 為配置順序經「等」修正的全序：沿配置順序反覆取出「它等的 change 皆已取出」的第一個 change；互等而取不出時取配置最前者。衝突 SHALL NOT 使任一方列入 archiveAfter，也 SHALL NOT 改變其餘同名 requirement 的先後；兩個方向都有「等」時雙方 SHALL 互列（封存互卡，須調整拆分）。delta 讀不到或空白 SHALL 視為零觸碰。`--strict-overlap` 時 SHALL 退回本變更前的規則：delta capability 重疊推後波次（波次取已配置重疊者的最大波次加一）並併入 blockedBy，缺 board_rank 者 SHALL 只依 created 升冪（缺 created 殿後）排、不看被依賴數與 task 總數，使 waves、changes 順序、blockedBy、overlaps 與 next 與本變更前逐位元一致；requirementOverlap 與 archiveAfter 照常計算。壞 metadata 的 change SHALL 排除於配置與重疊判定之外並列入 skipped。

#### Scenario: 無依賴無重疊時全部同波

- **WHEN** 三個提案中 change 皆無 depends_on、delta 目錄互不重疊
- **THEN** 三者皆為第 1 波，配置順序依基底順序

#### Scenario: 宣告依賴推後波次

- **WHEN** change c 的 depends_on 含 a，a 為提案中且未封存
- **THEN** c 的波次為 a 的波次加 1，c 的 blockedBy 含 a

#### Scenario: 重疊互斥依基底順序定先後

- **WHEN** change a 與 b 皆含 specs/desktop-app/ 的 delta，各自 MODIFIED 同一個 requirement「看板與任務」，基底順序 a 在 b 前，兩者皆無 depends_on
- **THEN** a 與 b 皆為第 1 波，兩者 blockedBy 皆為空，b 的 archiveAfter 為 [a]，a 的 archiveAfter 為空，雙方 requirementOverlap 各含對方且 conflict 為 false

#### Scenario: 指向已封存的依賴視為滿足

- **WHEN** change c 的 depends_on 含已封存的 old-change，其餘無前置
- **THEN** c 為第 1 波，blockedBy 為空，dependsOn 仍原文列出 old-change

#### Scenario: 已開工者排在未開工者之前

- **WHEN** change a 為進行中、b 為提案中，兩者 MODIFIED 同一個 requirement 且 b 的 created 早於 a
- **THEN** a 配置在 b 之前，b 的 archiveAfter 為 [a]，兩者同為第 1 波

#### Scenario: 同 capability 不同 requirement 不算重疊

- **WHEN** change a 與 b 皆含 specs/desktop-app/ 的 delta，a MODIFIED「Run rewind point」、b MODIFIED「Conversation pin」
- **THEN** 兩者同為第 1 波，requirementOverlap 與 archiveAfter 皆為空

#### Scenario: ADDED 先於動到同名的 MODIFIED

- **WHEN** change a ADDED requirement「匯出」、change b MODIFIED requirement「匯出」，基底順序 b 在 a 前
- **THEN** b 的 archiveAfter 為 [a]，a 的 archiveAfter 為空，兩者同波

#### Scenario: 兩個 ADDED 同名標衝突

- **WHEN** change a 與 b 皆 ADDED 同一 capability 的 requirement「匯出」
- **THEN** 雙方 requirementOverlap 各含對方且 conflict 為 true，雙方 archiveAfter 皆為空

#### Scenario: strict-overlap 退回目錄級推波

- **WHEN** change a 與 b 皆含 specs/desktop-app/ 的 delta 但動到不同 requirement，執行 speclink plan --strict-overlap --json
- **THEN** a 為第 1 波、b 為第 2 波，b 的 blockedBy 為 [a]，與本變更前的輸出一致

#### Scenario: strict-overlap 下缺 rank 者只依 created

- **WHEN** 提案中 a（12 個 task、created 2026-09-01）與 b（5 個 task、created 2026-09-02）皆無 rank 且共用 desktop-app 的 delta，執行 speclink plan --strict-overlap --json
- **THEN** a 為第 1 波、b 為第 2 波且 blockedBy 為 [a]；不帶旗標時 b 配置在 a 前、兩者同為第 1 波

#### Scenario: 帶進名稱的方向優先於基底序

- **WHEN** aa ADDED「R1」並 MODIFIED「R2」，bb MODIFIED「R1」與「R2」，bb 的 created 較早而配置在前
- **THEN** bb 的 archiveAfter 為 [aa]，aa 的 archiveAfter 為空，雙方 requirementOverlap 各含兩項

#### Scenario: 三個 change 的封存順序不繞環

- **WHEN** 提案中 a（3 個 task）MODIFIED「R」「S」、b（10 個 task）MODIFIED「S」「T」、c（20 個 task）ADDED「R」並 MODIFIED「T」，三者皆無 rank、created 相同，配置順序為 a、b、c
- **THEN** a 的 archiveAfter 為 [b, c]、b 的為空、c 的為 [b]（封存序 b、c、a），不形成 a 等 c、c 等 b、b 等 a 的環

#### Scenario: 衝突不改變另一個同名 requirement 的先後

- **WHEN** a 與 b 都 ADDED「R1」、都 MODIFIED「R2」，a 的 created 較早而配置在前
- **THEN** 雙方 requirementOverlap 對「R1」的一項 conflict 為 true，b 的 archiveAfter 為 [a]、a 的為空

#### Scenario: 改名成同名與新增同名標衝突

- **WHEN** a 以 RENAMED 把「舊名」改成「匯出」，b ADDED「匯出」
- **THEN** 雙方 requirementOverlap 各含對方且 conflict 為 true，a 的 ownOperation 為 RENAMED，雙方 archiveAfter 皆為空；若 b 改為 MODIFIED「匯出」，則 b 的 archiveAfter 為 [a]、不論基底順序

#### Scenario: 正式規格已有的名稱先拿走再帶進

- **WHEN** 正式規格 cap 已有 requirement「X」與「Y」；c REMOVED「Y」、d ADDED「Y」，d 的 created 較早而配置在前；e 以 RENAMED 把「X」改成「Z」、f ADDED「X」，f 的 created 較早而配置在前
- **THEN** d 的 archiveAfter 為 [c]、c 的為空；f 的 archiveAfter 為 [e]、e 的為空；四者的 requirementOverlap 皆無 conflict

#### Scenario: 修改者先於拿走者

- **WHEN** 正式規格 cap 已有 requirement「X」；a REMOVED「X」、b MODIFIED「X」，a 的 created 較早而配置在前
- **THEN** a 的 archiveAfter 為 [b]、b 的為空

#### Scenario: 兩個拿走同名標衝突

- **WHEN** 正式規格 cap 已有 requirement「X」；a REMOVED「X」、b 以 RENAMED 把「X」改成「W」
- **THEN** 雙方 requirementOverlap 對「X」的一項 conflict 為 true，雙方 archiveAfter 皆為空

#### Scenario: 沒有任何 task 的 tasks.md 與沒有 tasks.md 同樣殿後

- **WHEN** 提案中 empty（created 2026-08-01，tasks.md 只有標題「## 1. 準備」）與 some（created 2026-09-01，3 個 task）皆無 rank、無人依賴
- **THEN** 基底順序為 some、empty，next 為 some

#### Scenario: 缺 rank 者依被依賴數與 task 數排

- **WHEN** 提案中 x、y、z 皆無 rank，x 被 2 個 change 依賴、y 與 z 無人依賴，y 有 12 個 task、z 有 5 個 task
- **THEN** 基底順序為 x、z、y

##### Example: 五個 change 的基底順序與波次

| change | 階段 | board_rank | created | depends_on | delta 觸碰 | 基底序 | 波次 | blockedBy | archiveAfter |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| r1 | 已就緒 | 無 | 2026-09-01 | 無 | tray-status-menu MODIFIED「列首波次」 | 1 | 1 | 無 | 無 |
| p2 | 進行中 | n | 2026-09-03 | 無 | desktop-app MODIFIED「看板與任務」 | 2 | 1 | 無 | 無 |
| n5 | 提案中 | f | 2026-09-02 | 無 | archive-skill ADDED「候選清單」 | 3 | 1 | 無 | 無 |
| n3 | 提案中 | 無 | 2026-09-10 | 無 | desktop-app MODIFIED「看板與任務」 | 4 | 1 | 無 | p2 |
| n4 | 提案中 | 無 | 2026-09-12 | n3 | discuss-skill MODIFIED「輪次」 | 5 | 2 | n3 | 無 |

- **GIVEN** 上表五個未封存 change
- **WHEN** 引擎計算 plan
- **THEN** waves 為 [[r1, p2, n5, n3], [n4]]，next 為 n5（第一個提案中且 blockedBy 為空者），n3 的 archiveAfter 為 [p2]

### Requirement: plan 動詞輸出波次與阻擋清單

<!-- BEFORE: changes 每項七鍵、blockedBy 含重疊者、人眼只有 blocked by 尾註、無 --strict-overlap -->

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

- **WHEN** 某 change 的 .openspec.yaml 無法解析，執行 speclink plan --json
- **THEN** 該 change 不在 waves 與 changes 內，skipped 含其名稱與原因，其餘 change 照常配置，exit code 為 0

#### Scenario: 人眼輸出的封存順序與衝突尾註

- **WHEN** b 的 archiveAfter 為 [a]、c 與 d 互為 ADDED 同名衝突，執行 speclink plan --no-color
- **THEN** b 的行尾含 `archive after: a`，c 的行尾含 `conflicts with: d`，兩者行內皆不含 `blocked by:`

##### Example: 一個 change 三種尾註的順序

- **GIVEN** e 的 depends_on 含 a、與 b MODIFIED 同一 requirement、與 c ADDED 同名 requirement
- **WHEN** 執行 speclink plan --no-color
- **THEN** e 那一行去掉行首縮排後為 `• e [proposed] — blocked by: a — archive after: b — conflicts with: c`（行首的 `• <名稱> [<階段>]` 沿用既有形狀，本變更只追加尾註）

### Requirement: archive 技能以 plan 建議封存順序

<!-- BEFORE: 提示讀 blockedBy，字面為「plan 建議先封存前置」 -->

內嵌 archive 技能（事實來源 crates/engine/speclink-core/assets/skills/archive.md）SHALL 於封存前規定：執行 `speclink plan --json`，目標 change 的 `archiveAfter` 非空時提醒使用者「plan 建議先封存 <archiveAfter 的名稱>；它們封存後，重讀本 change 對同名 requirement 的 MODIFIED／REMOVED／RENAMED 區塊、對照正式規格重寫（走 ingest）再封存」，並列出 `requirementOverlap` 中對應的 capability 與 requirement 名；`requirementOverlap` 有 `conflict` 為 true 者時 SHALL 提醒「與 <change> 都新增（或改名成）同名的 requirement <名稱>，或都移除（或改名掉）它；後封存的那一個會封存失敗（名字重複，或名字已不存在），先改掉其中一邊」；`blockedBy` 非空時 SHALL 另提「宣告前置 <名稱> 尚未封存」。三者皆為建議，SHALL NOT 阻擋封存，SHALL NOT 依賴引擎新增守門，SHALL NOT 寫「跑 drift 即可」（drift 只查 requirement 名存在，抓不到內文被覆蓋）。技能 SHALL 同時記下 `archiveAfter` 含目標 change 的其他 change（它們排在目標之後、動到同名 requirement）；目標封存成功後 SHALL 提醒它們先重讀同名 requirement 的區塊、對照正式規格重寫（走 ingest）再封存——目標封存後 plan 不再列出它，這是最後一次點名的機會。此提醒僅提醒，SHALL NOT 單獨觸發詢問，SHALL NOT 代跑 ingest。

#### Scenario: 技能檔含封存順序建議

- **WHEN** 技能再生後讀取 speclink-archive 的 SKILL.md
- **THEN** 內文含執行 `speclink plan --json`、archiveAfter 非空時提醒先封存並於其封存後對照正式規格重寫再封存、conflict 時提醒改掉其中一邊、blockedBy 非空時提宣告前置的指示，且明載僅建議不阻擋，且不含「跑 drift 即可」的字面

#### Scenario: 封存後提醒排在後面的 change 重寫

- **WHEN** 目標 change 為 x，plan 中 y 的 archiveAfter 為 [x]，x 封存成功
- **THEN** 技能檔指示封存前記下 y，封存後提醒 y 先對照正式規格重寫（走 ingest）再封存，且不因此詢問使用者

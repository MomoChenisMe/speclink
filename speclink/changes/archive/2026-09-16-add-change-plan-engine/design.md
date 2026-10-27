## Context

順序判定今天分散在兩處：propose 技能收尾要代理人讀全部提案判定「可並行／須依序」但不落檔；桌面看板與系統匣以 `board_rank`（缺 rank 回退修改時間）排卡片，與執行順序無關。引擎已有材料：`Store::delta_capabilities` 能算兩個 change 是否動到同一份正式規格（硬信號）；`ChangeMeta` 以 `#[serde(default)]` 欄位向後相容，`edit_meta` 配 `KeyLines` 做逐行手術（`restale_from` 就是逗號清單累加器）；`in-progress add／remove` 是 meta 寫入動詞的既有樣板；CLI 的 dispatch 層以 ModeFree／Dual／FsOnly／RemoteOnly 單點宣告模式；`list` 在主 checkout 以 `WorktreeOverlay` 聚合各 worktree 的任務與開工狀態。本刀只動 speclink-core（演算法與 meta 欄位）、speclink-cli（兩個動詞）與技能資產；桌面（第二刀）與 server／remote（第三刀）不動。

## Goals / Non-Goals

**Goals:**

- 一份計算、一個落點：排序基底＋依賴修正的演算法只寫在 speclink-core，CLI 與後續的桌面 core、server 都呼叫它。
- 軟依賴一次落檔：`depends_on` 寫進 change meta，之後任何消費者零 token 讀取。
- apply 開工前的守門與挑選都由 `speclink plan --json` 回答，不再讀提案。
- 既有輸出零變動：`list --json`、`status --json`、人眼輸出逐位元不變。

**Non-Goals:**

- 不動桌面看板與系統匣的排序與呈現（第二刀）。
- 不加 server 端點、不讓兩個新動詞走 remote（第三刀）；本刀兩動詞 FsOnly。
- 封存不加軟依賴硬守門；只在技能文字加建議。
- 不做任務層的並行標記（先前討論已否決）。
- 不重算或搬移既有 `board_rank` 值。

## Decisions

### D1 排序基底與拓樸修正落 speclink-core 的 plan 模組

新模組 crates/engine/speclink-core/src/lifecycle/plan.rs 提供 `compute(store) -> Result<Plan, PlanError>`，內部委派 `compute_with_ranks(store, ranks: &BTreeMap<String, String>)`——`compute` 以各 change meta 的 `board_rank` 當 ranks；rank 來源不在 meta 的呼叫端（remote 的 board resource，第三刀）直接呼叫 `compute_with_ranks`。輸入只經 `Store` 契約（`list_changes`、`delta_capabilities`、任務計數），不含 ANSI、不寫死儲存媒介。

**基底順序**（決定同波內與無依賴時的先後）依三鍵排：

1. 階段：已就緒 → 進行中 → 提案中（已開工的工作不能被還沒開工的擋住；已就緒最接近封存，封存順序才是合併閘在乎的）。
2. 同階段內：有 `board_rank` 的排前、以位元組字典序升冪（使用者親手排過的優先）；缺 rank 的排後、彼此以 `created` 升冪、缺 `created` 者殿後（新工作排到隊尾，不插到使用者排好的卡前面；看板「缺 rank 置欄頂」的規則由第二刀改為與此同構）。
3. 同值以 change 名字典序決斷，使順序為全序且跨機器確定。

**拓樸修正**以貪婪配置：沿基底順序反覆取「所有宣告前置都已配置」的第一個 change 配置之；宣告前置只看仍在作用中（未封存）且 meta 可解析的 change，`depends_on` 指到不存在、已封存或 meta 壞掉（列入 `skipped`，不參與配置）的名稱視為已滿足。delta 重疊不是邊而是配置時的互斥：某 change 的波次 = 1 + max（其作用中宣告前置的波次、已配置且與其重疊的 change 的波次），無前置無重疊者為第 1 波。因此成環只可能來自 `depends_on`；重疊的兩個 change 誰先由基底順序決定，永遠不會與宣告方向打架。

替代案：把重疊當有向邊（依基底順序定向）——與 `depends_on` 混用時三個 change 就能拼出環，被否決。替代案：只算提案中的 change——已就緒與進行中的 change 未封存前仍可能被後續 change 的封存踩到合併閘，被否決。

### D2 depends_on 為逗號清單欄位，走 KeyLines 的清單手術

`ChangeMeta` 新增 `#[serde(default)] pub depends_on: Option<String>`，讀取器 `depends_on() -> Vec<String>` 與 `restale_from()` 同形（逗號切分、trim、去空）。寫入以 `edit_meta` 配 `KeyLines::push_list／remove_list`，其餘位元組逐字保留。change 名不含需跳脫字元（單一路徑段守門），不經 `yaml_scalar`；名稱含 `/`、`\`、`:` 或 `..` 一律拒絕。缺席讀作空清單；`list --json` 與 `status --json` 的序列化結構不含此欄位，輸出逐位元不變（以 listing 既有的「payload 不受新欄位影響」測試家族釘住）。

替代案：獨立的 `depends_on:` YAML 區塊序列——`KeyLines` 的 `set_block` 可寫，但 `restale_from` 已立逗號清單先例，且 clap 端的多值 `--on` 天然對應扁平清單，不另開格式。

### D3 plan 動詞的 payload 形狀

`speclink plan --json` 的 stdout 為單一 JSON 物件，欄位 camelCase：

- `waves`：陣列，每項 `{ "index": 1, "changes": ["a", "b"] }`；`changes` 內為該波的配置順序。
- `changes`：陣列，依整體配置順序排列，每項 `{ "name", "wave", "stage", "dependsOn", "overlaps", "blockedBy", "ready" }`。`stage` 為 `"proposed"`／`"in-progress"`／`"ready"` 三值；`dependsOn` 為 meta 宣告的原文清單（含指到已封存者）；`overlaps` 為陣列，每項 `{ "change", "capabilities" }`（capabilities 升冪）；`blockedBy` 為作用中的宣告前置 ∪ 配置在前且重疊的 change（去重、配置序）；`ready` 等價於 `blockedBy` 為空。
- `next`：第一個 `stage` 為 `proposed` 且 `ready` 為 true 的 change 名，無則 `null`。
- `skipped`：陣列，壞 metadata 的 change `{ "change", "reason" }`；不參與波次與重疊，清單為空時仍序列化為 `[]`。

人眼輸出逐波列印：`Wave N` 標題行（同波兩個以上時附 `(parallel)`），每個 change 一行含名稱、階段、`blocked by: …`（有才印），末行 `next: <name>` 或 `next: none`；`--no-color` 下無 ANSI。依賴成環：stderr 印 `dependency cycle: a -> b -> a`、exit code 非零、stdout 為空（`--json` 亦同）。非專案目錄與既有動詞同句拒絕。

替代案：`changes` 用以名稱為鍵的物件——會丟掉配置順序，AI 得自己從 waves 反查，被否決。替代案：塞進 `list --json`——動到凍結輸出，被否決。

### D4 change depends 動詞的守門與零半套

`speclink change depends <name> --on <other>... [--remove] [--json]`。守門依序：`<name>` 必須是作用中 change（找不到即錯）；加邊時每個 `<other>` 必須是作用中 change（已封存或不存在即錯，訊息分辨兩者），`--remove` 時放行指向已封存或不存在名稱的殘留項、只擋不合法名稱（空白、含 `/`、`\`、`:` 或 `..`）——否則前置封存後的殘留永遠清不掉；`<other>` 不得等於 `<name>`；非 `--remove` 時，把新邊加進現有 `depends_on` 圖後自 `<name>` 起沿宣告順序深度優先走，走回 `<name>` 即錯並印環（`c -> a -> c`），別處既有的環不擋加邊、由 plan 回報。作用中的判定以 `list_changes()` 的名稱逐字比對，不用 store 的存在探針：探針跟著檔案系統的大小寫規則走（macOS、Windows 會放行 `Add-Auth`），plan 的配置不會，且 `archive/` 目錄與空字串也會被探針放行。任一守門失敗 → exit code 非零、零寫入。全部通過才呼叫一次 `edit_meta`，多個 `--on` 在同一次閉包內逐一 `push_list`／`remove_list`，所以檔案只寫一次、沒有半套。冪等：加已存在的或移除不存在的邊不改檔、仍成功。成功時 stdout 一行 `✓ <name> depends on: a, b`（`--remove` 為 `✓ <name> no longer depends on: a`），`--json` 為 `{ "change", "dependsOn" }`（寫入後的完整清單）。壞 metadata 的 `<name>` 由 `edit_meta` fail-closed 拒絕。

### D5 rank 移動的依賴檢查函式

核心是純函式 `plan::violations(sequence: &[&str], depends_on: &BTreeMap<String, Vec<String>>) -> Vec<(String, String)>`：給定一個同階段的名稱序列與宣告依賴圖，回傳所有「依賴者排在前置之前」的配對；`plan::check_rank_move(store, name, rank) -> Result<(), RankMoveBlocked>` 以新 rank 取代該 change 的 rank 重算 D1 的基底順序後呼叫它，rank 來源不在 meta 的呼叫端（remote 拖排）可直接以自己的序列呼叫 `violations`。`check_rank_move`（只看同階段的 change），若任一作用中宣告前置會排在它之後、或任一宣告依賴它的 change 會排在它之前，即回 `RankMoveBlocked { change, must_follow: Vec<String>, must_precede: Vec<String> }`，`Display` 為一句人眼訊息。**重疊夥伴不在檢查範圍**：兩個只因 delta 重疊而須依序的 change，誰先誰後是使用者的合法選擇，拖排翻轉只改配置順序、不破壞任何約束；這是對討論記錄「有依賴或重疊的卡不得拖到前置之前」的收窄——「不得跨越」只對宣告依賴成立。本刀提供函式與測試，不改 `set_board_rank`（它被桌面拿來對 worktree 副本逐卡補章，內建檢查會算錯範圍）；第二刀由桌面 core 的 reorder 於寫 rank 前呼叫。

### D6 兩動詞為 FsOnly，經 command runtime 派發

`Command::Plan` 與 `Command::ChangeDepends { name, on, remove }` 加進 command runtime，outcome 分別為 `PlanReport` 與 `DependsOutcome`（typed_outcomes 註冊）。`ChangeDepends` 實際改動 meta 時發布領域事件 `DomainEvent::ChangeDependsChanged { change, depends_on, occurred_at }`（kind 字串 `change-depends-changed`），冪等無改動時零事件——與 `in-progress add` 同一姿態，讓 server 的 bridge commit 與 SSE invalidate 在第三刀直接接上。CLI 於 dispatch 宣告層以 `fs_only` 註冊 `plan` 與 `change`，remote 模式以既有 FsOnly 語意拒絕（只解析模式、零 server 請求、離線同拒），拒絕句為 `plan is not available in remote mode yet — it reads the local openspec/ tree` 與 `change depends is not available in remote mode yet — it writes the local change metadata`。`plan` 在主 checkout 且 worktree 政策開啟時，與 `list` 同一套 `observed_facts` → `WorktreeOverlay` 組裝 store，讓 worktree 內的開工與任務進度進入階段判定；`change depends` 不經 overlay 寫入（overlay 的寫一律落主副本，會把映射 change 的讀寫拆成兩份），而是沿桌面拖排的「逐 change 解析落點」作法：目標 change 已映射到 worktree 時，守門、成環檢查與寫入都以該 worktree 副本的 store 為準（plan 讀的正是這份副本，宣告立刻可見；分支後才建立的 change 不在副本 roster 內，要先同步分支），無映射時用 CLI 所在 checkout 的 store。

### D7 階段判定移入引擎

`model::stage(store, &Change) -> Stage`（`Ready`／`InProgress`／`Proposed`）：任務全勾且總數大於零為 Ready；有 `started_at` 或任一已勾任務為 InProgress；其餘 Proposed。與桌面 core 現行 `change_stage` 同一規則；本刀只新增引擎函式供 plan 使用，桌面改呼叫此函式留給第二刀。

### D8 三技能改寫與資產三連動

- propose.md 的「Pending-change landscape check」改為：propose 完成後，只針對本次建立的 change 判定軟依賴——讀其他提案中 change 的 proposal Impact，判定本 change 是否建立在某個 change 的成果上或動到同一段程式碼；有則執行 `speclink change depends <本 change> --on <前置>...`；然後執行 `speclink plan --json`，依有效 worktree 政策呈現：開啟時列出「第 1 波可並行（各開 session 走 apply-with-worktree）」與後續波次，關閉時列出 `changes` 的配置順序。作用中 change 只有本 change 時跳過整段。硬信號不再由代理人判定。
- apply.md 第 1 步「Select the change」改為：先執行 `speclink plan --json`；未指名時取 `next`，`next` 為 null 時列出每個 change 的 `blockedBy` 並停止；指名時找該 change，`blockedBy` 非空即印出前置清單並 STOP、不得執行 `review prepare` 與 `in-progress add`；指名的 change 不在 plan 內（已封存或不存在）沿既有錯誤處理。對話上下文推斷只用來決定「指名」，不再繞過 plan。apply-with-worktree 與 apply 同源（B_APPLY 拼接），自動帶到。
- archive.md 於封存前置檢查加一段「Plan order hint」：執行 `speclink plan --json`，目標 change 的 `blockedBy` 非空時提醒使用者「plan 建議先封存 X」，僅建議、使用者確認後照常封存。
- `ASSET_VERSION` 自 v1.35.0 升為 v1.36.0；claude、claude-worktree、codex、neutral-cli、neutral-tool-call 五份 golden 快照（render_golden 對五個目標都渲染技能內文）與 assets.lock 同批更新；再以 `speclink update` 再生 .claude/skills 與 .agents/skills 下的 SKILL.md（這批再生檔不進 evidence，收尾 commit 前以 git status 盤點帶上）。

## Implementation Contract

**行為**

- 執行 `speclink plan` 於 fs 模式專案：stdout 依 D3 印波次與 next，exit code 0；`--json` 印 D3 的物件；無作用中 change 時 `waves` 為 `[]`、`changes` 為 `[]`、`next` 為 null、人眼印 `next: none`。
- 依賴成環：exit code 非零、stderr 為 `dependency cycle: <a> -> <b> -> ... -> <a>`、stdout 空。
- 執行 `speclink change depends a --on b`：`openspec/changes/a/.openspec.yaml` 多一行 `depends_on: b`（已有則以 `, ` 追加），其餘位元組不變；stdout 一行成功訊息；exit code 0。
- `--remove` 移除後清單為空時整行移除。
- 守門失敗（D4 四種）exit code 非零、stderr 說明、meta 檔逐位元不變。
- remote 模式執行兩動詞：exit code 非零、stderr 為 D6 的拒絕句、零 server 請求。
- `speclink list --json` 與 `speclink status --change x --json` 的輸出於 meta 含 `depends_on` 時，與不含時逐位元一致。

**介面**

- `speclink_core::plan::compute(&dyn Store) -> Result<Plan, PlanError>`；`Plan { waves, changes, next, skipped }` 依 D3 序列化；`PlanError::Cycle(Vec<String>)`。
- `speclink_core::plan::compute_with_ranks(&dyn Store, &BTreeMap<String, String>) -> Result<Plan, PlanError>`；`speclink_core::plan::violations(&[&str], &BTreeMap<String, Vec<String>>) -> Vec<(String, String)>`；`speclink_core::plan::check_rank_move(&dyn Store, &str, &str) -> Result<(), RankMoveBlocked>`。
- `speclink_core::model::stage(&dyn Store, &Change) -> Stage`；`ChangeMeta::depends_on() -> Vec<String>`。
- `Command::Plan`、`Command::ChangeDepends { name: String, on: Vec<String>, remove: bool }`；`CommandOutcome::Plan(PlanReport)`、`CommandOutcome::ChangeDepends(DependsOutcome { change, depends_on, changed: bool })`；`DomainEvent::ChangeDependsChanged`。
- CLI：`plan [--json] [--no-color]`；`change depends <NAME> --on <OTHER>... [--remove] [--json]`。

**驗收**

- speclink-core 單元測試：基底順序三鍵、缺 rank 置後、created 補位、重疊互斥的波次、depends_on 指向已封存視為滿足、成環偵測與環的列印、`skipped` 排除壞 meta、`check_rank_move` 的允許與拒絕、`stage` 三態、`depends_on()` 讀取器、`list --json` 不受 `depends_on` 影響。
- speclink-cli 整合測試（`--test it`）：plan 人眼與 `--json` 的 payload 欄位（camelCase、型別）、成環 exit code、`change depends` 的四種守門與零寫入、冪等、`--remove` 清空整行移除、meta 其餘位元組保留、mode_dispatch 加入兩動詞的 remote 拒絕。
- golden：`cargo test -p speclink-core` 的 render_golden 與 assets.lock 於刻意更新後全綠；`speclink update` 後 git status 顯示 37 份 SKILL.md 再生。

**範圍**

- In：speclink-core 的 plan 模組、meta 欄位、stage 函式、command runtime 兩筆；speclink-host 的事件記錄補臂（`commit.rs` 的 `event_record_of` 窮舉 match 隨新事件變體必補）；speclink-cli 兩動詞與測試；三個技能資產與 golden；docs/verb-contract.md 的模式表；上述規格 delta。
- Out：apps/desktop 任何檔案；speclink-server、speclink-remote、speclink-protocol；`set_board_rank` 的行為；archive 引擎的守門。

## Risks / Trade-offs

- [golden 三連動漏一份] → render_golden 一次渲染五個目標（claude、claude-worktree、codex、neutral-cli、neutral-tool-call），`UPDATE_GOLDEN=1` 五份同批再生、assets.lock 同一個 task 內完成並以 render_golden 測試驗證；tasks 4.4 原只列三份，實作時以測試輸出為準。
- [基底順序與現行看板顯示不一致（看板缺 rank 置欄頂且回退修改時間，plan 缺 rank 置後且回退建立日期）] → 第二刀把看板改為 plan 順序即收斂；本刀在 change-plan 規格明載回退規則，避免兩邊各自解讀。
- [Windows 的 CRLF meta 檔] → `KeyLines` 已沿用每行自身行尾；整合測試以 LF 與 CRLF 各一份 meta 驗證 `depends_on` 寫入後其餘位元組不變。
- [worktree overlay 下 stage 判定] → plan 與 list 共用同一 overlay 組裝，worktree 內已勾任務進入階段判定；整合測試沿 worktree_overlay 既有夾具補一例。
- [跨機器順序不確定] → 每一層排序都以名稱決斷，D1 明定全序。
- [對討論記錄「重疊不得跨越」的收窄] → D5 記載理由；第二刀的拖排灰化只對宣告依賴生效。

## Migration Plan

無資料遷移：`depends_on` 缺席讀作空，舊 meta 檔照常解析。回滾即移除兩動詞與欄位讀取，已寫入的 `depends_on` 行由 serde 忽略、不影響任何既有動詞。

## Open Questions

無。

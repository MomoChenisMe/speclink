## Context

`speclink plan` 的引擎在 speclink-core 的 lifecycle plan 模組：基底順序（階段 → board_rank → created → 名字）之上，以 depends_on 做有向邊、以「delta 目錄碰到同一個 capability」做互斥，波次 = 1 + max(前置波次, 已配置且重疊者的波次)。目錄級重疊由 Store 的 delta_capabilities 判定，只看 `specs/<cap>/spec.md` 存不存在，不讀內文。封存合併閘（archive 模組的 merge_violations）已有 delta 區段解析（parse_delta 取 ADDED／MODIFIED／REMOVED 的 requirement 名，model 模組的 rename_pairs 取 RENAMED 的 FROM／TO），plan 可以直接重用。看板排序鍵的中點演算目前在 apps/desktop/core 的 rank 模組，只有桌面拖排會寫 board_rank，CLI 沒有動詞。技能 propose／ingest 的收尾盤點寫 depends_on，archive／commit 的封存前提示讀 `blockedBy`。plan 的消費者：CLI 動詞、apply／apply-with-worktree 守門、archive／commit 提示、桌面三欄與系統匣（經 desktop core 直接呼叫 speclink-core）、server GET /plan 與 remote convert（兩個新欄位於本刀提前帶上 wire）。

## Goals / Non-Goals

**Goals:**

- 波次只反映 depends_on；重疊降到 requirement 級並只產生封存順序提示。
- 缺 rank 的 change 用「被依賴數 → task 總數 → created」排，讓「很多人等它」與「很快做完」的先出來。
- plan 輸出說明原因：blockedBy 只剩宣告前置，新增 requirementOverlap 與 archiveAfter。
- 給 agent 一個排序把手 `change rank`，且不覆寫使用者拖排過的順序。
- 技能資產跟上新語意；apply 守門字面不動、行為自動放寬。

**Non-Goals:**

- 桌面排程分頁與桌面清單 payload 的排程欄位（第二刀）。wire 型別、server 的 GET /plan 與 remote convert 原屬第二刀，品質站階段提前到本刀（見「wire 帶上兩個新欄位」）。
- 封存硬守門、LLM 進引擎、wave 覆寫欄位、blockedBy 拆物件、remote 臂寫 rank、桌面與 server 的 strict 開關。

## Decisions

### 波次公式只看宣告前置，重疊改列 archiveAfter

placement 的 greedy 配置迴圈不變（沿基底順序取「前置皆已配置」的第一個），但波次只取 1 + max(前置波次)。requirement 級重疊不進波次、不進 blockedBy；archiveAfter 依下一節的配對規則，以一對 change 整體判定。`--strict-overlap` 時退回舊行為：目錄級重疊推波次並併入 blockedBy，缺 rank 者的排序也退回只看 created（不看被依賴數與 task 總數），waves、changes 順序、blockedBy、overlaps 與 next 和本 change 之前逐位元一致；archiveAfter 與 requirementOverlap 照常計算。strict 入口是 crate 內的 compute_strict，由 command 層的 Command::PlanStrict 呼叫（Command::Plan 維持原本的 ranks 單欄，寫不出「外部 rank 表＋strict」這種沒有意義的組合），compute 與 compute_with_ranks 的簽名不變（桌面與 server 的呼叫端不必改）。這個旗標只限本機模式：server 沒有對應開關，CLI 在 dispatch 層（同 `change rank`）以固定訊息 `plan --strict-overlap is not available in remote mode: the server plans with requirement-level overlap only` 拒絕，只解析模式、不發任何請求，並以 verb-contract delta 列為兩模式輸出的第 6 項明文分歧（比照第 3 項 status 的 schema 旗標）。捨棄「requirement 級重疊也推波次」：它會把同一個 requirement 的兩個小修排成兩波，而兩者本可平行開發、封存時對照一次即可。

### requirement 級重疊自 delta 內文解析，重用封存合併閘的解析函式

plan 模組新增 delta_touches(store, change)：對 delta_capabilities 的每個 capability 讀 read_artifact 的 delta 文字，經 archive 模組的 delta_operations 取 ADDED／MODIFIED／REMOVED 的 requirement 名與 RENAMED 的 FROM、TO 名（合併閘讀 delta 的同一套：換行正規化、parse_delta、rename_pairs、略過標題式 RENAMED），得到 (capability, touch, requirement) 三元組清單；改名的 FROM 與 TO 分開記（輸出的操作兩端都寫 RENAMED）。每個觸碰對該名稱有一個角色：MODIFIED 為「修改」（封存前後名稱都在）；REMOVED 與 RENAMED 的 FROM 為「拿走」（封存前在、封存後不在）；ADDED 與 RENAMED 的 TO 為「帶進」（封存前不在、封存後在）。每次封存都要名稱處在前一次封存留下的狀態，所以同一個名稱上的角色有固定先後，先後取決於該名稱此刻在不在正式規格裡（archive 模組的 canonical_names 讀出；capability 還沒有正式規格時一律視為不在）：

| 名稱此刻 | 角色先後 |
| --- | --- |
| 在正式規格裡 | 修改 → 拿走 → 帶進 |
| 不在正式規格裡 | 帶進 → 修改 → 拿走 |

兩個 change 對同一 (capability, requirement) 的判定：角色不同時為重疊，角色在後者「等」角色在前者，角色在前者一律先封存（例：正式規格有 Y 時，移除 Y 的 change 先於新增 Y 的 change；新增 X 的 change 先於修改 X 的 change；修改 X 的 change 先於移除 X 的 change）；雙方都是修改時為重疊，先後照封存序；雙方都是拿走或都是帶進時為衝突（conflict 為 true），不進 archiveAfter——後封存者一定失敗（名字重複，或名字已不存在），排順序沒用，要改掉其中一邊。

archiveAfter 以一對 change 整體判定，不逐 requirement 各算：任一同名 requirement 上我方等對方時，對方列入我方 archiveAfter；否則任一同名 requirement 上對方等我方時不列；否則有雙方都是修改的同名 requirement，且對方在封存序中在前才列。逐 requirement 各算會讓同一對 change 兩個方向都成立（一個 requirement 由角色推出、另一個照順序），兩邊互等；角色推出的方向是硬限制，所以優先。衝突不決定方向，也不改變其餘同名 requirement 的先後。

「照順序」的那一類不直接用配置順序，而用封存序：沿配置順序反覆取出「它等的 change 都已取出」的第一個 change（穩定的拓樸排序）。直接用配置順序時，一條違反配置順序的「等」加上兩條照配置順序的重疊，就能在三個 change 之間繞成環（a 等 c、b 在 a 前、c 在 b 後），封存建議因此無解；改用封存序後，照順序的那一類都跟著同一個全序走，不會繞環。兩個方向都有「等」時（互等），封存序取不出，就取配置最前者；這兩個 change 雙方互列：那是真的互卡，要調整拆分。

delta_operations 與 canonical_names 是 archive 模組的 pub(crate) 函式，合併閘自己也經 delta_operations 讀 delta，兩邊讀出的名稱不會分岔；normalize_newlines 維持私有。不動 Store 介面（讀取沿 read_artifact 與 read_canonical_spec，維持 storage 解耦）；每個被 delta 動到的 capability 只讀一次正式規格。delta 讀不到或空白視為零觸碰。捨棄「在 Store 加 delta_requirements 方法」：解析是領域邏輯，不該下沉到儲存層。捨棄「只看帶進名稱、不查正式規格」：移除與新增同名、改名掉與新增同名時，先後取決於名稱此刻在不在，只看帶進會把方向排反（照做時封存被擋），兩個移除同名也標不出衝突。

### 缺 rank 的 tie-break 改為被依賴數、task 總數、created

Entry 的基底序改由 basis_cmp 比較：階段 → 有 rank 者依 rank → 缺 rank 者依被依賴數多、task 總數少 → created（缺者殿後）→ 名字；strict 時缺 rank 者跳過被依賴數與 task 總數。被依賴數 = 作用中且可配置的 change 中 depends_on 含它者的個數（去重）。task 總數 = tasks.md 解析後的 task 個數；無 tasks.md、或 tasks.md 裡沒有任何 task 者於同層殿後（等同無限大），因為兩者都還沒有東西可 apply。有 rank 者順序完全不受新鍵影響。此比較同時是 rank 移動依賴檢查（check_ranks，move_rank 寫入前呼叫）重算同欄順序的依據，檢查的規則不變。plan 模組另公開 basis_order(store)（可配置 change 依此鍵排列的名稱序），apps/desktop/core 的 query 模組以它取代自己那份「去掉 stage 的排序鍵複本」，plan 成環時看板的退回序因此同樣照新鍵排（board-card-order「plan 成環時退回此基底序」）；壞 meta 的卡不在其中，照舊以名稱序接在後面。看板顯示序另由 plan 模組的 board_order(store) 提供（配置順序，成環時退回基底序）：桌面看板清單照它排，拖排與 change rank 的欄序也由同一個函式投影，兩處不各寫一份。

### plan 輸出追加欄位，既有欄位保留

PlanChange 新增 `requirement_overlap: Vec<RequirementOverlap>`（serde rename requirementOverlap；每項 change、capability、requirement、ownOperation、otherOperation、conflict）與 `archive_after: Vec<String>`（archiveAfter）。`overlaps`（目錄級）原樣保留，讓第二刀前的桌面讀者不變。人眼輸出每個 change 一行後的尾註依序：` — blocked by: …`、` — archive after: …`、` — conflicts with: …`，各自空時不印。

### wire 帶上兩個新欄位（自第二刀提前）

speclink-protocol 的 PlanChangeEntry 追加 `requirement_overlap: Vec<PlanRequirementOverlap>` 與 `archive_after: Vec<String>`，皆 `#[serde(default)]`、camelCase，舊 server 缺席時讀作空陣列；PlanRequirementOverlap 六欄，兩個操作以字串搬運（同 `stage` 的做法：字串表只在引擎，DeltaOp 提供 as_str 與 parse）。server 的 plan handler 從引擎報告逐欄映射；speclink-remote 的 plan_report 逐欄搬運，操作字串不認得即回錯誤、不猜。這樣 remote 模式的 `plan` 與本機同形，archive 與 commit 技能在 remote 也拿得到 archiveAfter。原因：server 已改新語意（blockedBy 不再含重疊者），wire 若不帶新欄位，remote 專案在兩刀之間會無聲失去重疊提醒。捨棄「server 在第二刀前沿用舊算法」：remote 與本機會用兩套規則，且 remote 拒絕 --strict-overlap 的理由就不成立了。

### 排序鍵演算搬進 speclink-core，看板拖排與 change rank 共用同一步

apps/desktop/core 的 rank 模組（midpoint、spread、neighbor_midpoint、ranked_in_order）整檔搬到 speclink-core 的 lifecycle rank 模組（lifecycle 資料夾模組本身私有，rank 與其他成員一樣由 crate 根 re-export 為 `speclink_core::rank`）；桌面原檔刪除，apps/desktop/core 的 manage 模組與 src-tauri 的 remote 模組直接引用 `speclink_core::rank`。

plan 模組新增兩個函式：

- **move_rank(store, write, name, prev, next)**：看板拖排與 change rank 共用的一步。
  1. 欄序為看板顯示序（board_order 背後的同一個函式：plan 配置順序，成環時為基底序）投影到 name 所在的欄，只含 metadata 有效者，name 自己除外（它的舊位置不算數）。
  2. 欄內任一 change 缺 rank、或 rank 序與欄序不一致（ranked_in_order 為假）、或 rank 不合生成格式（rank 模組的 generated_shape：合法的 board_rank 且不以 a 結尾；手改的鍵不合時算不出中點）時，以 spread 依欄序算整欄補章。格式檢查只在這一步（本機寫入）：ranked_in_order 仍是 remote 拖排共用的判定，只看有無與順序，remote 的補章規則不在本刀。
  3. 以 neighbor_midpoint 算 name 落在 prev 與 next 之間的新 rank（不在欄內的鄰居視為開放端）。
  4. 寫入之前做完所有檢查：要寫的 change（補章對象與 name）都有 `.openspec.yaml`，補章加新 rank 的整張表跑 rank 移動的依賴檢查（check_ranks）；任一不過即拒絕、零寫入。
  5. 寫入經呼叫端傳入的 write 逐 change 落在該 change 自己的副本：先逐一寫補章，最後寫 name。任一步失敗即停止並回報；已寫入的補章保留（部分補章的欄仍是合法順序，重跑可從該狀態續行）。
- **set_rank(store, write, name, anchor, side, force)**：change rank 的守門，依序為 name、再 anchor 須為作用中且 metadata 可解析的 change、兩者不相同、同階段（不同欄時 rank 無法改變 plan 順序）；name 已有 board_rank 且未帶 --force 即拒絕，訊息 `'<name>' already has a board rank; pass --force to move it anyway`。之後以 anchor 與 Side（Before／After）在同一個欄序裡找出 prev 與 next，照 move_rank 的做法寫入：兩者共用 move_start（查 name、取 rank 表與可配置清單），寫入同經 write_between。

桌面看板拖排改呼叫 move_rank，刪去自己那份補章、中點與檢查。行為因此有兩處改變，都寫進 board-card-order 的 delta：被拖卡不參與「缺 rank」判定（只有它缺 rank 時不整欄補章）；依賴檢查被拒時零寫入（原本先寫補章再檢查，補章會留下）。

CLI 子指令 `speclink change rank <name> --before <other> | --after <other> [--force] [--json]`，--before 與 --after 互斥且必擇一。它只在本機、沒有 server 臂，所以不經 command 層，直接呼叫 set_rank：讀取走 worktree overlay（與 plan 同一視角：主 checkout 的名冊，映射到 worktree 者取其副本；worktree 分支後才在主 checkout 新建的 change 也在欄內），寫入依 worktree 映射逐 change 導向各自的副本：CLI 的 home_store_for 在副本仍持有該 change 時取副本，探查後被移除時退回主 checkout，與 overlay 讀取的回退同一個存在判定；change depends 的寫入也用它。overlay 本身的寫入一律落主副本（它的契約是只重導讀取），所以寫入目的地由呼叫端決定，與桌面拖排同一做法。成功 stdout 一行 `✓ <name> ranked before <other>`（或 after）；`--json` 為 `{ "change": string, "anchor": string, "position": "before" | "after" }`，不含 rank 值（board_rank 不進 CLI 輸出）。remote 模式由 dispatch 明確拒絕，stderr `change rank is not available in remote mode: the board order lives in the board resource`，非零 exit、不發請求（verb-contract「模式分岔的單點宣告」的 FsOnly 子情形）。

### 技能資產以有無 board_rank 為 rank 代寫分界

propose 與 ingest 收尾：判定並寫 depends_on 的段落之後，新增 rank 判定——本 change 為提案中且規模明顯小於它前面的 change（task 數少且無人依賴它時視為小）而急，執行 `speclink change rank <本 change> --before <第一個比它大的提案中 change>`；動詞以「已有 rank」拒絕時只口頭建議、不加 --force；本 change 為進行中或已就緒時不執行。「硬信號」段落改寫為：requirement 級重疊由引擎算成 archiveAfter 與 conflict，代理人不自行判定，但 conflict 要向使用者提出其中一邊要改。archive 與 commit 的封存前提示改讀 `archiveAfter`，字面改為「plan 建議先封存 X；X 封存後，重讀本 change 對 requirement R 的 MODIFIED／REMOVED／RENAMED 區塊、對照正式規格重寫（走 ingest）再封存」（依 LANGUAGE.md 的詞條用「正式規格」）；conflict 提示用白話：「兩邊都新增（或改名成）同名的 requirement，或都移除（或改名掉）它；後封存的那一個會封存失敗（名字重複，或名字已不存在），先改掉其中一邊」；目標封存後 plan 不再列出它，排在它之後的 change 就看不到「封存後重寫」的提示，所以封存前先記下 archiveAfter 含目標的其他 change，封存成功後提醒它們先對照正式規格重寫（走 ingest）再封存；`blockedBy` 非空另提「宣告前置尚未封存」。archive 未指名候選改標 archiveAfter（空時「可封存」）。apply 與 apply-worktree-pre 兩份 asset 的「a blocker that comes from delta-capability overlap keeps its place until it lands」子句一併刪除：blockedBy 不再含重疊者，這句描述的情況已不存在。propose 與 ingest 的插隊判定段落逐字相同，與落檔段一樣由 render_golden 的共用字面測試鎖住。六份 asset 改動走 ASSET_VERSION、golden、assets.lock 三連動。

## Implementation Contract

**Behavior**

- 無 depends_on 的 change 一律第 1 波，不論 delta 碰到多少 capability。有 depends_on 者波次 = 前置最大波次 + 1。
- 兩個 change 對同 capability 同 requirement 名各有觸碰時，依角色先後（名稱在正式規格裡：修改 → 拿走 → 帶進；不在：帶進 → 修改 → 拿走）角色在後者的 archiveAfter 含角色在前者，不論基底序；雙方都是修改時封存序在後者列前者；兩者同波、blockedBy 皆不含對方。雙方都帶進或都拿走時雙方 requirementOverlap 各含對方且 conflict 為 true，archiveAfter 不含。
- 正式規格有 Y：c REMOVED Y、d ADDED Y 時 d 的 archiveAfter 為 [c]；正式規格有 X：a REMOVED X、b MODIFIED X 時 a 的 archiveAfter 為 [b]；a REMOVED X、b RENAMED X→W 時雙方 conflict 為 true。
- tasks.md 沒有任何 task 的 change 與沒有 tasks.md 者同樣在缺 rank 者中殿後，不會搶到 next。
- 欄內有不合生成格式的 rank（如大寫 N、只有 a）時，change rank 與拖排先整欄補章再放新鍵，不 panic、不陷入迴圈。
- 封存 x 時，archiveAfter 含 x 的其他 change 在 x 封存成功後被提醒先對照正式規格重寫再封存。
- 一對 change 的 archiveAfter 以整對判定：aa（ADDED R1、MODIFIED R2）與 bb（MODIFIED R1、R2，配置在前）時，bb 的 archiveAfter 為 [aa]、aa 的為空。
- 照順序的重疊依封存序定先後：a MODIFIED R、S，b MODIFIED S、T，c ADDED R 並 MODIFIED T，配置順序 a、b、c 時，a 的 archiveAfter 為 [b, c]、b 為空、c 為 [b]，不繞環。
- `--strict-overlap` 時 waves、changes 順序、blockedBy、overlaps、next 與本 change 前逐位元一致（缺 rank 者只依 created），只多兩個新欄位。
- 缺 rank 的三個提案中 change：被依賴 2 次者在前；同為 0 次時 task 數 5 者在 task 數 12 者前；同 task 數時 created 早者前。
- `change rank c --before a`（a、c 同欄、c 無 rank）後 plan 的 changes 順序 c 在 a 前；c 有 rank 時無 --force 拒絕；c 的 depends_on 含 a 時拒絕並印 must_follow。
- 主 checkout 且 worktree 政策開啟時，change rank 的欄序含 worktree 分支後才於主 checkout 新建的 change；補章與新 rank 寫進各 change 自己的副本，plan 立即反映。
- 桌面看板拖排與 change rank 用同一個 move_rank：被拒時零寫入，只有被拖卡缺 rank 時只寫被拖卡。
- remote 模式 `plan --json` 與本機對同一內容逐位元一致，含 requirementOverlap 與 archiveAfter。

**Interface / data shape**

- `speclink plan [--strict-overlap] [--json] [--no-color]`；`--json` 的 changes 每項九鍵：name、wave、stage、dependsOn、overlaps、blockedBy、ready、requirementOverlap、archiveAfter。
- requirementOverlap 每項六鍵：change、capability、requirement、ownOperation、otherOperation（值域 ADDED／MODIFIED／REMOVED／RENAMED）、conflict。
- `speclink change rank <name> (--before <other> | --after <other>) [--force] [--json]`；成功輸出如 Decisions 所述。change rank 不經 command 層。
- command 層：`Command::Plan { ranks }` 與 `Command::PlanStrict`。
- 引擎：`plan::board_order(store)`（看板顯示序）、`plan::set_rank(store, write, name, anchor, side, force)`、`plan::move_rank(store, write, name, prev, next)`，`write: &dyn Fn(&str, &str) -> anyhow::Result<()>` 把一個 change 的 board rank 寫進它自己的副本；`plan::Side { Before, After }`；`DeltaOp::as_str`／`DeltaOp::parse`。
- wire：PlanChangeEntry 追加 requirementOverlap（PlanRequirementOverlap 六欄，操作為字串）與 archiveAfter，皆 serde default。
- Store 介面不變；PlanChange 與 Plan 為 serde Serialize，新欄位追加於尾端。

**Failure modes**

- plan：depends_on 成環維持既有錯誤與 exit code；delta 讀不到視為零觸碰，不報錯。
- change rank：未知或已封存名稱、壞 metadata、不同階段、自我錨定、已有 rank 未帶 --force、要寫入的 change 缺 `.openspec.yaml`、rank 移動被依賴檢查擋下：皆非零 exit、stderr 一行、零寫入（寫入途中失敗除外，見 Decisions）。remote 模式拒絕不連線。
- remote convert：stage 或操作字串不認得、wave 列了 changes 沒有的名稱時回錯誤，不猜。

**Acceptance criteria**

- speclink-core plan 模組單元測試：無依賴多 change 同波；requirement 級重疊列 archiveAfter 不推波；ADDED 對 ADDED 標 conflict；ADDED 先於 MODIFIED；改名的新名與 ADDED 同一套；依正式規格排角色先後（拿走先於帶進、修改先於拿走、兩個拿走標 conflict）；一對 change 以角色推出的方向優先；tasks.md 沒有 task 者殿後；strict 旗標重現舊波次與舊排序；新 tie-break 三段；delta_touches 對 RENAMED 取 FROM 與 TO。
- speclink-core rank 相關單元測試：change rank 成功、已有 rank 拒絕、--force 覆寫、跨依賴拒絕、整欄補章寫入順序；move_rank 的鄰居落點、被移動者不參與補章判定、被拒零寫入；不合生成格式的 rank 整欄補章。
- speclink-cli 整合測試：`--json` 九鍵形狀、人眼三種尾註、`--strict-overlap`、`change rank` 的成功行與各拒絕（含缺 `.openspec.yaml`）、remote 模式拒絕、worktree 的兩種情境、remote plan 與 fs 同形（含新欄位）；home_store_for 的存在判定單元測試。
- speclink-protocol、speclink-remote、speclink-server plan_api 測試：兩個新欄位的序列化、缺席容忍、逐欄搬運、不認得的操作回錯誤。
- render_golden 五份快照與 assets.lock 更新後 `cargo test -p speclink-core --test it render_golden::` 綠；`speclink update` 再生的 SKILL.md 含新字面。
- apps/desktop/core 的 manage 與 query 測試綠（拖排被拒零寫入；plan 成環時看板清單的退回序照新 tie-break）；`cargo test -p speclink-desktop` 綠。

**Scope boundaries**

- In：speclink-core plan／rank／command／archive 解析入口、speclink-cli plan 與 change 動詞、四份技能 asset 改寫與 apply／apply-worktree-pre 兩份刪去重疊殘句（六份同一次三連動）、wire 的兩個新欄位（protocol、server plan handler、remote convert，自第二刀提前）、desktop core 的拖排改用 move_rank 與成環退回序改用引擎基底序、desktop src-tauri 改引用 speclink_core::rank。
- Out：桌面排程分頁與桌面清單 payload 的排程欄位（第二刀）、server 端的 strict 開關、remote 拖排（board resource）的補章規則。

## Risks / Trade-offs

- [golden 與 CLI 測試基線變動] → 同批更新五份 golden 與 plan_verbs 期望值；以 `--strict-overlap` 測試證明舊輸出仍可重現。
- [整欄補章把「引擎排的」變成「有 rank 的」，之後技能對這些 change 不再代寫 rank] → 接受：補章只在使用者或技能主動用 change rank 時發生，且 --force 仍可覆寫；記在技能字面。
- [task 總數會隨 ingest 改變，缺 rank 卡片順序跟著變] → 接受：只影響缺 rank 者；要固定順序就拖排或 change rank。
- [plan 多讀正式規格] → 每個被 delta 動到的 capability 只讀一次 canonical spec；capability 還沒有正式規格時名稱一律視為不在。
- [看板顯示序多算一次配置] → 桌面每次刷新多一次可配置清單與配置；change 數量少，接受，換來清單與拖排欄序只有一份規則。
- [跨平台] → delta 解析先 normalize 換行（沿 capability_violations 的做法）；rank 文字手術沿 set_board_rank 逐行保留行尾；測試不假設路徑分隔。
- [第二刀前桌面排程分頁讀 blockedBy 顯示「可以開工」變常態、重疊段仍顯示目錄級] → 接受，第二刀切換。
- [搬 rank 模組] → 沿「搬模組的守門盲點」：桌面原檔刪除、引用改為 speclink_core::rank；搬完跑 desktop core 與 speclink-desktop 測試。
- [桌面拖排行為改變：被拒時不再留下補章，只有被拖卡缺 rank 時不再整欄補章] → 接受：兩者都讓寫入更少、順序結果相同；寫進 board-card-order 的 delta，桌面測試同批改寫。
- [remote 拖排（board resource）仍把被拖卡算入補章判定，也不檢查 rank 格式（手改的鍵在 remote 仍算不出中點，是本來就有的問題）] → 接受：remote 的寫入是一次 PUT 整份文件，沒有「留下一半補章」的問題；兩者只差改寫哪些鍵，順序相同。對齊 remote 規則屬 remote-board-order，超出本刀。

## Migration Plan

1. 引擎與 CLI 落地，跑 core 與 cli 測試。
2. 改四份 asset，bump ASSET_VERSION，重生 golden 與 assets.lock，cargo build 後 `speclink update` 再生 SKILL.md（37 份不進 evidence，收尾以 git status 盤點）。
3. 回退：整個 change 為單一 commit 序列，revert 即回舊行為；`--strict-overlap` 為執行期回退。

## Open Questions

（無。）

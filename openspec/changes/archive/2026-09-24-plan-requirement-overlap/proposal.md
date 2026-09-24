## Why

在某個有 18 個進行中 change 的專案上，兩個無依賴、無人依賴的小型 UI change 被 `speclink plan` 排到第 14、15 波（共 15 波）：引擎把「delta 目錄碰到同一個 capability」當成必須依序的硬信號並推後波次，再以建立時間決定先後，新建的小 change 因此永遠墊底。真正會在封存時互相覆蓋的邊界只有「同一個 requirement 被兩個 change 用 MODIFIED／REMOVED／RENAMED 動到」，而且它只影響封存順序、不影響開工順序。目標使用者是透過 AI 代理跑 SDD 的開發者：apply 開工挑選、archive 封存提示、propose／ingest 收尾盤點與桌面看板順序都消費 plan 的結果，這條錯誤的排序會讓「使用者實機回饋後的小修」等上好幾個月。

本 change 為討論 plan-overlap-granularity 的第一刀（引擎、CLI、技能資產，以及品質站階段自第二刀提前的 wire：protocol、server、remote convert）；第二刀 plan-requirement-overlap-desktop 接桌面清單 payload 與排程分頁。

## What Changes

- **波次只由 depends_on 推後**：波次 = 1 + 作用中宣告前置的最大波次；delta 重疊不再推波次。新增 `speclink plan --strict-overlap` 旗標退回本變更前的舊行為（目錄級重疊推波次、缺 rank 者只依 created 排），只作用於 plan 動詞，且只限本機模式（remote 模式明確拒絕，列為 verb-contract 兩模式輸出的第 6 項明文分歧）。
- **重疊降到 requirement 級**：引擎讀每個 change 的 delta 內文，同一 capability 下同一個 requirement 名被兩個 change 動到才算重疊。每一方對該名字的角色是「修改」（MODIFIED）、「拿走」（REMOVED、改名的舊名）或「帶進」（ADDED、改名的新名）；角色的先後看這個名字此刻在不在正式規格裡（在：修改 → 拿走 → 帶進；不在：帶進 → 修改 → 拿走），角色在前者先封存。雙方都帶進或都拿走時另標「衝突」（後封存的那一個會因名字重複或名字已不存在而封存失敗，排順序沒用，要改掉其中一邊）；雙方都是修改時先後照基底順序，後者標 `archiveAfter`。一對 change 由角色推出的方向優先於基底順序，不會兩邊互等；其餘照順序的重疊依封存序（配置順序經角色推出的方向修正）定先後，三個以上的 change 也不會繞成環。
- **缺 rank 的 change 改 tie-break**：階段 → 有 board_rank 者照 rank → 缺 rank 者依「被依賴數多的先 → task 總數少的先（沒有 tasks.md 或其中沒有任何 task 者殿後）→ created 早的先 → 名字」。有 rank 者順序不受影響。
- **plan 輸出加欄位**（`--json` 與人眼）：保留 `blockedBy`，語意窄化為只含作用中宣告前置；新增 `requirementOverlap`（change、capability、requirement、ownOperation、otherOperation、conflict）與 `archiveAfter`（change 名陣列）。人眼輸出對應加 `archive after:` 與 `conflicts with:` 兩種尾註。
- **新動詞 `speclink change rank <name> --before <other> | --after <other> [--force] [--json]`**：以看板排序鍵（board_rank）的中點演算把 change 排到另一個 change 之前或之後，沿用 rank 移動的依賴檢查（違反 depends_on 即拒絕、零寫入）；目標已有 rank 時預設拒絕、`--force` 才覆寫，讓技能不會蓋掉使用者拖排過的順序。排序鍵演算自 apps/desktop/core 搬進 speclink-core；欄序（與看板清單同一個顯示序）、補章（欄內缺 rank、順序不一致或本機寫入時 rank 不合生成格式就整欄補）、中點與依賴檢查是引擎的同一個函式，桌面看板拖排改呼叫它（被拖卡不參與補章判定、所有檢查在寫入前，被拒時零寫入）。讀取與 plan 同一視角，寫入逐 change 落在各自的副本（映射到 worktree 且副本仍持有該 change 者寫其 worktree 副本，與讀取同一個存在判定）；要寫入的 change 缺 `.openspec.yaml` 即拒絕。remote 模式明確拒絕（remote 的順序真相是 board resource，不是卡片 meta）。
- **技能資產改寫**（propose、ingest、archive、commit 四份，claude 與 codex 兩種工具的 SKILL.md 皆再生）：propose 與 ingest 收尾的「硬信號＝delta capability 重疊交引擎」改成 requirement 級重疊與封存順序提示；兩者在寫 depends_on 之外，對提案中且無 rank 的 change 可用 `change rank --before` 代寫插隊，被拒（已有 rank）時只口頭建議；已開工的 change 不碰 rank。archive 與 commit 的先封存順序提示改讀 `archiveAfter`，並附「先封存 X，X 封存後對照正式規格重寫 requirement R 再封存」的重寫提示，不再寫「跑 drift 即可」；衝突提示用白話說明後封存的那一個會失敗、要改掉其中一邊；封存前記下 archiveAfter 含本 change 的其他 change，封存後提醒它們先對照正式規格重寫再封存（本 change 封存後 plan 不再列出它，這是最後一次點名）。propose 與 ingest 的插隊判定段落逐字一致。apply 與 apply-worktree-pre 兩份 asset 刪去「a blocker that comes from delta-capability overlap keeps its place until it lands」子句（blockedBy 不再含重疊者，這句描述的情況已不存在）。
- **wire 帶上兩個新欄位**（自第二刀提前）：protocol 的 plan 回應每項增列 `requirementOverlap` 與 `archiveAfter`（serde default，讀舊 server 時缺席為空陣列），server GET /plan 逐欄映射，remote convert 逐欄搬運（操作字串不認得即回錯誤）。remote 模式的 plan 因此與本機同形，archive／commit 技能在 remote 也拿得到封存順序。
- **相容性影響**：`speclink plan --json` 的 `blockedBy` 值域變窄（只剩宣告前置），`waves` 會變扁（多數 change 回到第 1 波）；新增欄位對既有讀者是追加。人眼輸出的 `blocked by:` 尾註只列宣告前置，新增兩種尾註。apply 守門讀 `blockedBy` 的字面不變，行為自動放寬。桌面看板缺 rank 卡片的順序會跟著新 tie-break 重排（plan 成環時的退回序亦同，改由引擎提供基底序）。桌面拖排被依賴檢查拒絕時不再留下補章，只有被拖卡缺 rank 時不再整欄補章。GET /plan 回應為追加欄位。golden（render_golden）與 CLI plan 測試同批更新。ASSET_VERSION、golden 快照與 assets.lock 三連動。
- **不涉及設定欄位**：openspec/config.yaml 與 .speclink.yaml 無新增或變更欄位。

## Non-Goals

- 不改桌面清單 payload 的排程欄位與桌面排程分頁——第二刀 plan-requirement-overlap-desktop 處理。
- 不改 remote 拖排（board resource）的補章規則。
- 不加引擎守門擋封存：archiveAfter 只提示（討論 change-execution-order 已否決封存硬守門）。
- 不把 LLM 判斷放進 plan 引擎，不新增 wave 覆寫欄位，不把 `blockedBy` 拆成物件。
- 不提供 `change rank` 的 remote 臂寫入（remote 順序由 board resource 決定）。
- 桌面與 server 不加 `--strict-overlap` 的對應開關。
- `speclink list --json` 與 `discuss list --json` 的輸出逐位元不變（board_rank 仍不進 CLI 輸出；`change rank` 的輸出不含 rank 值）。

## Capabilities

### New Capabilities

（無。步驟 3 掃描命中 change-plan、board-card-order、archive-merge、drift-computation、propose-skill、ingest-skill、archive-skill、commit-skill、worktree-apply-skill；本 change 全部落在既有 capability 的 requirement 修改。）

### Modified Capabilities

- `change-plan`：「執行順序的基底與拓樸修正」改為 depends_on 推波次、requirement 級重疊、新 tie-break 與 `--strict-overlap`；「plan 動詞輸出波次與阻擋清單」加 `requirementOverlap`、`archiveAfter` 與人眼尾註；新增「change rank 動詞寫入看板順序鍵」；「archive 技能以 plan 建議封存順序」改讀 archiveAfter 並附重寫提示。「apply 技能以 plan 挑選與守門」「rank 移動的依賴檢查」「change depends 動詞」「remote 臂」字面不變。
- `board-card-order`：「看板卡片順序以 board_rank 欄位為真相」的同欄基底序改引用 change-plan 的新 tie-break（缺 rank 卡不再只依 created）；「欄內存在缺 rank 卡時整欄補章」改為被拖卡不參與缺 rank 判定、所有檢查在寫入前且被拒時零寫入。
- `propose-skill`：「收尾盤點提案中變更的執行順序」改硬信號字面並加 rank 代寫規則。
- `ingest-skill`：「ingest 收尾重判本變更的軟依賴」改硬信號字面並加 rank 代寫規則（以有無 board_rank 為分界）。
- `archive-skill`：「封存完成後的收尾提交提醒」不變；「未指名時的候選清單依 plan 順序」候選改標 archiveAfter；「archive 技能以 plan 建議封存順序」在 change-plan 內修改。
- `commit-skill`：「先封存子流程的順序提示與收尾提醒」封存前提示改讀 archiveAfter 並附重寫提示。
- `verb-contract`：「動詞人眼輸出的兩模式同形」明文分歧清單加第 6 項——plan 的 `--strict-overlap` 於 remote 模式以固定訊息明確拒絕、不發任何請求；「模式分岔的單點宣告」把 change rank 與 plan 的 `--strict-overlap` 列為 Dual 家族內的 FsOnly 子情形。
- `client-protocol`：「plan 回應 payload」加 `requirementOverlap` 與 `archiveAfter` 兩欄、轉換逐欄搬運（自第二刀提前）。
- `server-verb-api`：「plan 唯讀衍生查詢端點」DTO 加兩欄，壞 board 情境的順序措辭改引用 change-plan 的缺 rank 順序（自第二刀提前）。

## Impact

- Affected specs: change-plan、board-card-order、propose-skill、ingest-skill、archive-skill、commit-skill、verb-contract、client-protocol、server-verb-api
- Affected crates／apps: speclink-core（引擎與技能資產）、speclink-cli（plan 與 change 動詞、dispatch）、speclink-protocol（plan 回應兩欄）、speclink-remote（convert 逐欄搬運）、speclink-server（plan 路由映射兩欄）、apps/desktop/core（拖排改用引擎 move_rank、看板顯示序改照引擎 board_order、成環退回序改用引擎基底序）、apps/desktop/src-tauri（改引用 speclink_core::rank、測試替身補兩欄）
- Affected code:
  - New:
    - crates/engine/speclink-core/src/lifecycle/rank.rs（看板排序鍵演算，自 apps/desktop/core/src/rank.rs 搬入；不合生成格式的鍵當成缺 rank）
    - openspec/changes/plan-requirement-overlap/specs/verb-contract/spec.md（兩模式輸出明文分歧第 6 項、模式分岔的 FsOnly 子情形）
    - openspec/changes/plan-requirement-overlap/specs/client-protocol/spec.md（plan 回應 payload，自第二刀提前）
    - openspec/changes/plan-requirement-overlap/specs/server-verb-api/spec.md（plan 唯讀衍生查詢端點，自第二刀提前）
  - Modified:
    - crates/engine/speclink-core/src/lifecycle/plan.rs（重疊粒度、依正式規格排角色先後與整對封存方向、波次公式、tie-break、basis_order 與 board_order、新欄位、compute_strict、set_rank 與 move_rank、DeltaOp 的字串轉換）
    - crates/engine/speclink-core/src/lifecycle/archive.rs（delta_operations 與 canonical_names：合併閘讀 delta 與正式規格名稱的同一套，供 plan 模組使用）
    - crates/engine/speclink-core/src/lifecycle/mod.rs（宣告 rank 模組）
    - crates/engine/speclink-core/src/lib.rs（re-export speclink_core::rank）
    - crates/engine/speclink-core/src/command/mod.rs（新增 Command::PlanStrict）
    - crates/engine/speclink-core/src/command/tests.rs
    - crates/engine/speclink-core/src/teststore.rs（meta 寫入順序記錄與單筆寫入失敗開關）
    - crates/engine/speclink-core/src/workspace/init.rs（ASSET_VERSION）
    - crates/engine/speclink-core/assets/skills/propose.md
    - crates/engine/speclink-core/assets/skills/ingest.md
    - crates/engine/speclink-core/assets/skills/archive.md
    - crates/engine/speclink-core/assets/skills/commit.md
    - crates/engine/speclink-core/assets/skills/apply.md（刪去重疊殘句）
    - crates/engine/speclink-core/assets/skills/apply-worktree-pre.md（刪去重疊殘句）
    - crates/engine/speclink-core/tests/golden/claude.snapshot.md
    - crates/engine/speclink-core/tests/golden/codex.snapshot.md
    - crates/engine/speclink-core/tests/golden/claude-worktree.snapshot.md
    - crates/engine/speclink-core/tests/golden/neutral-cli.snapshot.md
    - crates/engine/speclink-core/tests/golden/neutral-tool-call.snapshot.md
    - crates/engine/speclink-core/tests/golden/assets.lock
    - crates/engine/speclink-core/tests/it/render_golden.rs（archive／commit／ingest／propose 的新字面釘選）
    - crates/adapters/speclink-cli/src/verbs/plan.rs（--strict-overlap、rank 子指令：讀 overlay、逐 change 寫回各自副本、人眼尾註；home_store_for 為 change 所在副本的單一裁決，change depends 與 change rank 共用）
    - crates/adapters/speclink-cli/src/main.rs（change rank 與 plan --strict-overlap 在 dispatch 宣告為只限本機，拒絕訊息常數）
    - crates/adapters/speclink-cli/tests/it/plan_verbs.rs
    - crates/adapters/speclink-cli/tests/it/remote_plan.rs
    - crates/adapters/speclink-cli/tests/it/worktree_overlay.rs
    - crates/protocol/speclink-protocol/src/query.rs（PlanChangeEntry 加兩欄與 PlanRequirementOverlap 型別）
    - crates/protocol/speclink-remote/src/convert.rs（plan_report 逐欄搬運兩欄）
    - crates/protocol/speclink-remote/tests/it/typed_client.rs
    - crates/host/speclink-server/src/api/routes.rs（plan handler 映射兩欄）
    - crates/host/speclink-server/tests/it/api/plan_api.rs
    - apps/desktop/core/src/lib.rs（刪去 rank 模組宣告）
    - apps/desktop/core/src/manage.rs（看板拖排改呼叫引擎 move_rank、討論卡拖排改引用 speclink_core::rank）
    - apps/desktop/core/src/query.rs（看板顯示序改照 board_order、成環退回序改用 basis_order、plan 注入測試改為新波次語意）
    - apps/desktop/src-tauri/src/remote.rs（改引用 speclink_core::rank、plan 測試替身補兩欄）
    - docs/verb-contract.md、docs/verb-contract.zh-TW.md（FsOnly 列與第 6 項明文分歧）
    - .claude/skills/*/SKILL.md、.agents/skills/*/SKILL.md（`speclink update` 再生，37 份）
  - Removed:
    - apps/desktop/core/src/rank.rs（排序鍵演算已搬進 speclink-core，桌面直接引用）

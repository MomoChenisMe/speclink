## Context

`/speclink-improve` 的正典是 crates/speclink-core/assets/skills/improve.md（英文），經 `speclink init`／`speclink update` 渲染成 claude（.claude/skills/speclink-improve/SKILL.md）與 codex（.agents/skills/speclink-improve/SKILL.md）兩份實例；渲染內容由 crates/speclink-core/tests/it/render_golden.rs 的四份 golden 快照（claude、claude-worktree、codex、neutral-cli、neutral-tool-call）與 assets.lock 保護，asset 內文改動必須同批 bump `ASSET_VERSION`（crates/speclink-core/src/init.rs）並以 `UPDATE_GOLDEN=1`／`UPDATE_ASSETS_LOCK=1` 再生。規格 improve-skill 以「掃描精髓段逐字保留」與「candidates 以討論記錄承載」兩條要求釘住技能字面。

在途變更 discussion-hold-until-release（來源討論 multi-cut-discussion-hold，13/13、雙章已落、待封存）改動同一份 improve.md 的扇出段落（`--hold` 語意）、`ASSET_VERSION` v1.31.0→v1.32.0 與四份 golden、assets.lock；其規格 delta 不含 improve-skill。

## Goals / Non-Goals

**Goals:**

- 第六條摩擦訊號與它的准入測試進技能字面，兩工具實例同步。
- 其餘五訊號、刪除測試、六步骨架、防重提、subagent 上限字面零改動。
- 規格 improve-skill 釘住新字面：技能檔缺第六訊號或其准入測試時，驗證站對得出。

**Non-Goals:**

- 獨立模式或旗標；review 技能的 Standards 軸；渲染機制。
- 不改 improve.md 中 discussion-hold-until-release 正在改的扇出段落。

## Decisions

### D1 第六訊號與准入測試的字面（英文，與 asset 同語）

Step 3 的引言「The five friction signals below」改「The six friction signals below」；清單標題「**The five friction signals:**」改「**The six friction signals:**」；於第 5 條之後新增：

6. **Grouping you can only guess from file names.** Files that belong to one concept sit side by side with unrelated ones in a flat directory, and the reader reassembles the grouping every time — from prefixes, from alphabetical order, or from a design document whose layering the directory does not follow.

刪除測試段落之後新增一段（原段落字面不動）：

**The sixth signal has its own admission criterion.** Deleting a folder concentrates nothing, so the deletion test does not apply to it. Ask the reader-prediction test instead: could someone opening this directory for the first time predict where a piece of behaviour lives and which files form one group, without searching? A layout candidate qualifies only when all three hold: (a) the grouping has an objective source — dependency direction between modules, a layering an existing design document already states, or which files are linked from outside; (b) it is invisible to callers — existing paths stay valid through a root-level re-export, and a guard test asserts no stale path remains where re-export is not possible (scripts, docs, CI); (c) moving the files is the whole change — a candidate that needs every caller to change its paths is a move, not a grouping. Drop it. When a candidate trips both a code signal and the sixth, the deletion test decides first: if deleting the scattered pieces would concentrate the behaviour, it is a code candidate and the layout is only the symptom. The sixth signal covers only what stays as it is and merely moves.

「Aim for 3-6 candidates …」一句的結尾由「the deletion test was not applied」改「the admission criteria were not applied」（deletion test 不再是唯一數量閘）。Step 4 的 `strongly recommended` 定義由「the deletion test is clearly passed」改「the admission criterion is clearly met — the deletion test, or the reader-prediction test for a sixth-signal candidate」，否則第六訊號候選永遠拿不到最高強度。以上兩處與第六訊號同碰訊號 1 時的裁決句（deletion test 先判）為 review 站 Round 1 的修正。

### D2 Step 5 深度四問的第六訊號對應版與兩條做法

interface depth check 那一條之後新增兩條（四問明說「取代」上一條的四問；第三問命名 Path hazards 避免與 Step 3「The hard limit is 2 subagents」撞名；第四問與 Step 3 的讀者預測測試同一題；硬限制例子改通用說法不帶 Rust 專屬字眼；兩條做法獨立成一顆）：

- **For a sixth-signal candidate these four questions replace the four above** — the seam is a directory boundary, not a call boundary: (1) **Grouping source** — which objective source (dependency direction, documented layering, external links) draws each boundary? (2) **Path stability** — what keeps existing paths valid: a root re-export, a guard test for stale paths, or both? (3) **Path hazards** — which were inventoried before moving anything: public URLs that embed a path, tag-triggered or path-filtered CI workflows, build-time file includes written as relative paths, cross-package path dependencies? (4) **Reader-prediction test** — could someone opening the directory for the first time predict where behaviour lives and which files form one group, without searching? Flatten it again in your head to check: what does that reader lose? Surface these four answers instead of the four above.
- **Two practices ride along with a sixth-signal candidate** — tests move with the code (integration tests mirror the source grouping; inline test modules past a size threshold move to a sibling tests file), and cuts follow path dependence — the cut that moves directories lands first, the cuts that group inside them come after, never in parallel worktrees.

### D3 Guardrails

「**Deletion test gates every candidate** — concentrating complexity counts, moving it does not」改為「**An admission criterion gates every candidate** — the deletion test for the first five signals, the reader-prediction test for the sixth, and moving complexity around never counts」（粗體開頭不再與例外自相矛盾，維持單句格式）。其餘六條不動。

### D4 版號三連動與排程

`ASSET_VERSION` 於 discussion-hold-until-release 落地後為 v1.32.0，本案 bump 為 v1.33.0；`UPDATE_GOLDEN=1 cargo test -p speclink-core --test it render_golden` 再生四份快照、`UPDATE_ASSETS_LOCK=1` 同一測試目標再生 assets.lock；再 `cargo build -p speclink-cli` 後以該 binary 執行 `speclink update`，讓 repo 內 .claude 與 .agents 共 37 份 SKILL.md 帶新版號（只有 improve 兩份內文變、其餘 35 份僅版號行）。本案 SHALL 於 discussion-hold-until-release 封存並提交後才 apply：兩案改動 improve.md 的段落不同（扇出段 vs Step 3／Step 5／Guardrails），但 `ASSET_VERSION` 行、四份 golden 與 assets.lock 必撞，平行必生版號對撞。

替代案：不 bump 版號、只改 golden——已初始化專案不會收到過期提示，兩工具實例停在舊字面；獨立模式——使用者裁定否決。

### D5 規格 delta 形狀

improve-skill 的「掃描精髓段逐字保留」MODIFIED：(2) 由五條改六條並列出第六條，新增 (5) 第六訊號准入測試三條件；scenario「五條 friction 訊號逐條在列」改名「六條 friction 訊號逐條在列」，以 `<!-- REMOVED-SCENARIO: 五條 friction 訊號逐條在列 -->` 宣告放棄舊名（archive-merge 規格的漏抄守門）；新增 scenario「第六訊號帶讀者預測測試」。「candidates 以討論記錄承載」MODIFIED：interface depth check 一句補「對第六訊號 SHALL 改問分組來源、路徑維持、硬限制盤點、讀者預測四問」；既有三條 scenario 中「candidates 記錄形式」「全數否決仍留記錄」逐字保留；「分期立案帶 --hold」與同要求的 hold 子句對齊 discussion-hold-until-release 落地後的技能字面（hold 只由不帶 --hold 的 conclude 或 discuss archive 解除，最後一刀封存後執行 discuss archive 收尾），因該案的規格 delta 未含 improve-skill、主線規格仍留「旗標由下一次轉出清除」舊句，本案順手收齊以免封存時寫回舊字面；新增 scenario「第六訊號的深度四問對應版」。

## Implementation Contract

**行為（可觀察）**

- `speclink init`／`speclink update` 渲染的 claude 與 codex speclink-improve 技能檔 SHALL 含 D1 的第六條訊號與准入測試段、D2 的對應四問條、D3 的 Guardrails 字面；其餘段落與改動前逐字相同（扇出段落為 discussion-hold-until-release 落地後的字面）。
- `speclink --version` 顯示的 engine 版號為 v1.33.0；已初始化於 v1.32.0 的工作區執行 `speclink update` 會再生技能檔。
- 人眼輸出與 `--json`：無任何指令的輸出改變。

**介面／資料形狀**：無程式介面變動；只有 asset 內文、常數字串、golden 快照、assets.lock。

**失敗模式**：golden 或 assets.lock 未同批更新時 `cargo test -p speclink-core --test it render_golden` 紅（訊息指名以 UPDATE_GOLDEN／UPDATE_ASSETS_LOCK 再生）；版號未 bump 時同一測試的版號斷言紅。

**驗收**

- `cargo test -p speclink-core --test it render_golden` 綠，且四份快照與 assets.lock 的 diff 只含本案段落與版號。
- .claude/skills/speclink-improve/SKILL.md 與 .agents/skills/speclink-improve/SKILL.md 各含「The six friction signals」（2 次：引言與清單標題）「reader-prediction test」（小寫 3 次：准入段、Step 4 `strongly recommended` 定義、Guardrails；Step 5 第四問是大寫開頭的 `Reader-prediction test`，不分大小寫則為 4 次）「Grouping source」（1 次）；37 份 SKILL.md 的版號行皆為 v1.33.0。
- `speclink validate improve-layout-signal` 綠；`node --test scripts/*.test.mjs` 綠（vocabulary-guard 掃 assets/skills）。

**範圍**：In scope＝上列檔案。Out of scope＝review 技能、渲染機制、其餘五訊號字面、discussion-hold-until-release 的扇出段落。

## Risks / Trade-offs

- **回歸對照**：四份 golden 與 assets.lock 是唯一守門，以 UPDATE_GOLDEN 刻意再生後人工核 diff 只含本案段落；CLI 測試不受影響。
- **版號對撞**：與 discussion-hold-until-release 同批檔案——排程序列化（D4）解決；若對方尚未提交就 apply，`ASSET_VERSION` 與 golden 會出現雙方改動混在同一工作樹，commit 時無法拆分。
- **跨平台**：純文字與快照，無平台差異；golden 比對以 `\n` 為準（既有）。
- **技能篇幅**：improve.md 增約 20 行，仍在單檔可讀範圍；不加範例（Deferred）。

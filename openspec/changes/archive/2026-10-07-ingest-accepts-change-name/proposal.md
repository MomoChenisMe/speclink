## Problem

`/speclink-ingest` 把參數一律當成計畫檔：`/speclink-ingest add-auth` 會去找 `<計畫目錄>add-auth.md`，找不到就回報錯誤並停止。可是其他技能與引擎都叫使用者這樣用：

- drift 技能的建議選項、drift 引擎算出的 `primary_recommendation`（`/speclink-ingest <name>`）。
- apply、archive、apply-with-worktree（worktree 收尾段）與 discuss 技能的下一步建議（`/speclink-ingest <change-name>`）。

使用者照建議操作，ingest 就停下來；Agent 也無法用參數明確指定要更新哪個變更，只能每次被問一次。使用者是透過 Claude Code 或 Codex 跑 SDD 的開發者／PO／PM，情境是實作途中需求改變（apply ⇄ ingest），以及 drift 之後回頭更新計畫。

## Root Cause

ingest 技能的「Locate the requirement source」步驟只定義了「參數＝計畫檔」一種解讀；選擇要更新的變更（「Check for active changes」）則永遠以 AskUserQuestion 詢問，沒有讀參數。

## Proposed Solution

只改 ingest 技能的事實來源（crates/engine/speclink-core/assets/skills/ingest.md），其他技能與 drift 引擎的 `/speclink-ingest <變更名稱>` 寫法不動：

1. 參數解析順序：
   - 參數看起來是路徑（含 `/` 或以 `.md` 結尾）→ 照舊當成計畫檔。
   - 否則先執行 `speclink list --json`，參數等於某個作用中變更的名稱 → 把它當成要更新的變更，需求來源是目前的對話內容（以及該變更連結的討論結論，沿用既有段落），不再詢問要更新哪個變更。
   - 都不是 → 照舊當成計畫檔名稱；找不到檔案時回報錯誤並停止，錯誤訊息同時說明「不是作用中的變更名稱，也找不到同名計畫檔」。
2. 依技能資產規則：`ASSET_VERSION` 升一版，重新產生 5 份 render golden 與 assets.lock，並執行 `speclink update` 重新產生本 repo 的技能檔。
3. 文件：docs/workflow 兩版的 ingest 段落改寫參數說明。

影響的工具：claude（`/speclink-ingest`）與 codex（`$speclink-ingest`）的 ingest 技能檔。沒有 CLI 指令、旗標、設定欄位或 `--json` 輸出的變更。

## Non-Goals

- 不改其他技能與 drift 引擎的建議寫法：它們本來就是想要的用法，改 ingest 一處最小。
- 不改 ingest 的 artifacts 更新、軟依賴重判與 seal 流程。
- 不支援一次指定多個變更或同時指定變更與計畫檔。
- 不新增 CLI 動詞或旗標；本變更只動技能文字。
- 不做 design.md：只改一份技能文字與其衍生物，沒有跨模組的技術決定。

## Success Criteria

- `/speclink-ingest add-auth`（`add-auth` 是作用中變更）直接更新 `add-auth`，不再找 `add-auth.md`、不再詢問要更新哪個變更。
- `/speclink-ingest <計畫目錄>agile-discovering-rocket.md` 與 `/speclink-ingest agile-discovering-rocket`（沒有同名變更時）照舊使用計畫檔。
- `/speclink-ingest` 不帶參數的行為不變。
- 參數既不是作用中變更、也找不到計畫檔時，技能回報錯誤並停止，訊息說明兩種都找不到。
- golden、assets.lock 與本 repo 的 37 份技能檔都已同批更新。

### 相容性影響

- 技能檔內容改變：`ASSET_VERSION` 升一版，使用者升級後需執行 `speclink update` 取得新版技能檔（引擎既有的技能檔過期提示會提醒）。
- 參數剛好與作用中變更同名、又想當計畫檔用的情況，改由變更優先；要指定計畫檔，可以寫完整路徑。
- CLI 人眼輸出、`--json` 與 exit code 不變。

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `ingest-skill`: 新增「ingest 參數可指定要更新的變更」的需求。

## Impact

- Affected specs: ingest-skill
- 掃描到但不需修改的正式規格：skill-routing（apply 與 ingest 之間的路由表不變）、drift-computation（drift 的建議寫法不變）
- Affected code:
  - Modified: crates/engine/speclink-core/assets/skills/ingest.md
  - Modified: crates/engine/speclink-core/src/workspace/init.rs
  - Modified: crates/engine/speclink-core/tests/it/render_golden.rs
  - Modified: crates/engine/speclink-core/tests/golden/claude.snapshot.md
  - Modified: crates/engine/speclink-core/tests/golden/claude-worktree.snapshot.md
  - Modified: crates/engine/speclink-core/tests/golden/codex.snapshot.md
  - Modified: crates/engine/speclink-core/tests/golden/neutral-cli.snapshot.md
  - Modified: crates/engine/speclink-core/tests/golden/neutral-tool-call.snapshot.md
  - Modified: crates/engine/speclink-core/tests/golden/assets.lock
  - Modified: .claude/skills/（本 repo 安裝的 19 份 SKILL.md，版號行與 ingest 內文）
  - Modified: .agents/skills/（本 repo 安裝的 18 份 SKILL.md，版號行與 ingest 內文）
  - Modified: docs/workflow.zh-TW.md
  - Modified: docs/workflow.md

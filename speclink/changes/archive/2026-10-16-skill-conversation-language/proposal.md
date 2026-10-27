## Why

透過 AI 代理跑 SDD 的開發者／PO／PM 在 apply、quality、worktree-merge 這類長流程中，常遇到 Agent 中途改用英文回覆：GitHub issue #2 回報 10 天約 95 個對話出現 8 次，而且專案已設 `locale: tw`、工具本身也設成繁中。根源是 skill 內文是幾百行英文指令與英文回報範例，卻沒有一句話告訴 Agent「對話語言跟著使用者」。另外，discuss、improve、config、audit 四個會產出文件或報告的 skill 還沒綁定 `locale`，使「`locale`＝AI 產出文件與報告的語言」在各 skill 之間不一致。

## What Changes

- 新增一份共用的對話語言段落（技能原稿 `crates/engine/speclink-core/assets/skills/conversation-language.md`）。渲染每個產品 skill 的 SKILL.md 時，這段接在內文最前面；claude、codex、copilot 三個內建工具與自訂描述子都一樣。段落規定五件事：
  - 對使用者說的話（進度、提問、摘要）用使用者跟 Agent 講話的語言。
  - skill 用英文寫，回報範例只示範格式：範例的文字要翻譯，指令、檔案路徑、程式碼與固定 token（嚴重度籤、狀態詞）維持原樣。
  - 某個步驟規定自己產出的語言（文件、報告，或像 review 那樣呈現結果與提問的方式；例如經 `locale`）時，以該步驟為準。
  - 修改既有文件、且沒有步驟規定語言時，沿用該文件原本的語言。
  - 在 fork 情境中執行、看不到對話時，用收到的請求所用的語言。
- discuss 與 improve：新建的討論記錄，其散文（Context、各輪、結論；improve 另含 topic）依 `locale` 撰寫；已存在的記錄沿用它原本的語言。欄位標記（`**Focus**:`、`**Ruled out**:`、`**Decision**:` 等）、固定標記（`Prior discussions:`、`Source doc:`、`none`、cut 刀頭等）、slug、指令與程式碼維持英文；discuss 的 topic 照既有規格維持使用者原文。improve 的防重提搜尋改用 `locale` 語言與英文識別字一起搜尋。
- config：起草的 `context` 與 `rules` 依 Step 3 確認後的 `locale` 撰寫（Step 3 改了就先改寫草稿再顯示 diff）；review 規則照使用者原話寫。
- audit：獨立模式的報告依 `locale` 撰寫；嚴重度（Critical／High／Medium／Low）、檔案路徑與指令維持英文。
- 上述四個 skill 從 `speclink workflow-config show --json` 的 `locale` 欄位取得語言（`tw`、`ja`、`en`；null 代表英文；config 以 Step 3 確認後的值為準）。
- 使用者文件 `docs/configuration.md` 與 `docs/configuration.zh-TW.md` 的 `locale` 說明改寫：它是 AI 產出文件與報告的語言，review、verify 也用它呈現結果；其餘時候 Agent 用使用者的語言對話。
- ASSET_VERSION 由 v1.48.0 升為 v1.49.0，render golden 與 assets.lock 跟著再生。

## Capabilities

### New Capabilities

- `audit-skill`: /speclink-audit 技能的內容，本次規定獨立模式報告的語言綁定。現有規格沒有 audit 技能的逐技能規格：skill-authoring 只管技能文字的寫法、不管單一技能的流程行為；workspace-tools 與 archive-merge 只在檔案清單中提到 audit 的技能檔。

### Modified Capabilities

- `skill-authoring`: 新增「技能共用的對話語言段落」需求——段落的內容、單一來源、渲染位置與覆蓋面。
- `discuss-skill`: 新增討論記錄的撰寫語言依 `locale`（新建記錄）或沿用原語言（既有記錄）的需求。
- `improve-skill`: 新增改進討論記錄的撰寫語言依 `locale` 的需求。
- `config-skill`: 新增起草 `context`、`rules` 的撰寫語言依 `locale` 的需求。

## Impact

- 目標使用者與情境：透過 AI 代理跑 SDD 的開發者／PO／PM；共用段落涵蓋所有 skill 的對話，語言綁定涵蓋 discuss、improve（討論階段）、config（設定階段）與 audit（安全檢查）四個階段的產出。
- 影響的 crate：speclink-core（技能原稿、渲染函式、測試）。speclink-cli 與 speclink-node 經同一個渲染函式產出技能檔，行為跟著改變，但不改程式碼。
- 影響的技能與工具：19 個產品 skill 的 SKILL.md 都多出共用段落；discuss、improve、config、audit 四個另有語言綁定；claude、codex、copilot 與自訂描述子同步。`speclink instructions --skill` 輸出的原稿不含共用段落（apply 用它讀 tdd、audit 的紀律段，此時 apply 自己的技能檔已帶有段落）。
- 設定欄位：不新增、不變更任何欄位。`locale` 的語意不變（AI 產出文件與報告的語言），預設仍為 English；不讀 SPECLINK_LOCALE 環境變數的限制比照 manual、baseline 既有前例。
- 相容性影響：
  - CLI 指令、旗標、stdin、exit code 與人眼／`--json` 輸出都不變。
  - render golden 六份快照與 assets.lock 會變（升版號後再生）；既有專案要執行 `speclink update` 才會拿到新的技能檔。
  - 行為改變：沒設 `locale` 的專案，新建的討論記錄、config 起草的內容與 audit 報告改為英文（以前通常跟著對話語言），與 proposal 的現況一致；設了 `locale` 的專案依設定的語言。已存在的討論記錄不變，後續新增的輪次沿用原語言。
- Affected specs: audit-skill（新）、skill-authoring、discuss-skill、improve-skill、config-skill
- Affected code:
  - New: crates/engine/speclink-core/assets/skills/conversation-language.md
  - Modified: crates/engine/speclink-core/src/workspace/skills.rs、crates/engine/speclink-core/src/workspace/init.rs、crates/engine/speclink-core/assets/skills/discuss.md、crates/engine/speclink-core/assets/skills/improve.md、crates/engine/speclink-core/assets/skills/config.md、crates/engine/speclink-core/assets/skills/audit.md、crates/engine/speclink-core/tests/it/skill_authoring.rs、crates/engine/speclink-core/tests/it/render_golden.rs、crates/engine/speclink-core/tests/golden/claude.snapshot.md、crates/engine/speclink-core/tests/golden/claude-worktree.snapshot.md、crates/engine/speclink-core/tests/golden/codex.snapshot.md、crates/engine/speclink-core/tests/golden/copilot.snapshot.md、crates/engine/speclink-core/tests/golden/neutral-cli.snapshot.md、crates/engine/speclink-core/tests/golden/neutral-tool-call.snapshot.md、crates/engine/speclink-core/tests/golden/assets.lock、docs/configuration.md、docs/configuration.zh-TW.md
  - 另外：repo 自用的技能檔（claude 與 codex 兩個技能目錄下的 speclink 技能）由 speclink update 再生

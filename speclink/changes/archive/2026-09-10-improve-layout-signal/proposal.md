## Why

`/speclink-improve` 的目標使用者是透過 AI 代理跑 SDD 的開發者，使用情境是「請模型掃碼提改進」。技能目前只有一套判準：五條摩擦訊號＋刪除測試（刪掉抽象後複雜度是否集中）。資料夾與模組的分組整理不減少程式碼複雜度，照字面過不了刪除測試——2026-09-09 的 improve-repo-layout 掃描是掃描者在記錄的背景段自行改寫准入判準（「第一次打開的人能否不靠搜尋預測功能住哪、哪些檔是一組」）才做得下去，該改寫只活在那一份記錄裡；前例 cli-verb-family-modules（2026-08）也是用同型判準通過。使用者裁定（討論 improve-layout-signal）：任何專案的使用者跑 improve 都應拿得到這套標準，所以它要進技能字面，並列為其中一項訊號、不是獨立模式。

相關規格掃描：`improve-skill`（本案修改對象，其「掃描精髓段逐字保留」要求明列五條訊號與刪除測試）；`discuss-skill`、`skill-routing`、`workflow-schemas` 名字相近但分別規範討論技能、技能路由與 schema，不涵蓋 improve 的判準。

## What Changes

影響的 crate：speclink-core（技能正典 asset、`ASSET_VERSION`、render golden 與 assets.lock）。影響的技能與工具：speclink-improve 的 claude 與 codex 兩種渲染實例（init／update 再生）。不新增或變更 CLI 指令、旗標、stdin 與 exit code；不涉及設定欄位。

1. **第六條摩擦訊號**：技能正典 crates/speclink-core/assets/skills/improve.md 的 Step 3 由「五條」改「六條」，新增第六條「分組只靠檔名猜」——同一概念的檔案散在平鋪目錄裡，讀者要靠前綴或字母序重拼分組；或目錄與架構文件的分層對不上。
2. **第六訊號自己的准入測試**：寫在刪除測試段落之後——對第六訊號，刪除測試改問讀者預測測試（第一次打開這個目錄的人能否不靠搜尋預測功能住哪、哪些檔是一組）；分組須有客觀依據（引用方向、既有架構文件、對外連結）；且對呼叫者不可見（既有路徑經 re-export 或舊路徑守門斷言維持不變）。只換位置、呼叫端要跟著改路徑的不算。其餘五訊號與刪除測試字面不變。
3. **Step 5 深度四問的第六訊號對應版**：分組邊界依據什麼客觀來源；對外路徑靠什麼維持不變；搬移前盤點了哪些硬限制（公開網址、tag 觸發的 workflow、相對路徑 include、跨套件 path 相依）；回到平鋪讀者失去什麼。另補兩條做法：測試鏡射原始碼分組搬移（含內嵌測試的行數門檻）；切刀先動目錄本身、再動目錄裡的分組，不用 worktree 平行。
4. **Guardrails**：「Deletion test gates every candidate」一條補「the sixth signal uses the reader-prediction test instead」。
5. **版號三連動**：`ASSET_VERSION` bump（在途變更 discussion-hold-until-release 落地後為 v1.32.0，本案再 bump 為 v1.33.0）、四份 render golden 與 assets.lock 同批更新、repo 內 .claude 與 .agents 的 SKILL.md 以 `speclink update` 再生。

**相容性影響**：渲染產出的 speclink-improve 技能檔內容改變，render golden 更新屬刻意變更；已初始化的專案會因版號提升收到「技能過期」提示並經 `speclink update` 取得新版；人眼輸出與 `--json` 皆不變。

**排程限制**：在途變更 discussion-hold-until-release（已落雙章、待封存）正改動同一份 improve.md（扇出段落）、`ASSET_VERSION` 與四份 golden——本案 SHALL 於它封存並提交後才 apply，tasks 寫在它落地後的文字上（兩者改動段落不同，不會文字衝突，但版號行與 golden 必撞）。

## Non-Goals

- 不做獨立模式或旗標（使用者裁定列為一項）。
- 不動其餘五訊號、刪除測試、六步骨架、subagent 上限與防重提檢查的字面。
- 不把第六訊號加進 `/speclink-review` 的 Standards 軸（Deferred，另案）。
- 不改 `speclink update`／init 的渲染機制。

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `improve-skill`：「掃描精髓段逐字保留」要求由五條訊號改六條，並規定第六訊號的准入測試；「candidates 以討論記錄承載」要求補 interface depth check 對第六訊號的對應四問。

## Impact

- Affected specs：improve-skill（MODIFIED 兩條 requirement；scenario「五條 friction 訊號逐條在列」改名為「六條 friction 訊號逐條在列」並以 REMOVED-SCENARIO 宣告；新增兩條 scenario）。
- Affected code:
  - New: （無）
  - Modified: crates/speclink-core/assets/skills/improve.md、crates/speclink-core/src/init.rs（ASSET_VERSION）、crates/speclink-core/tests/golden/claude.snapshot.md、crates/speclink-core/tests/golden/claude-worktree.snapshot.md、crates/speclink-core/tests/golden/codex.snapshot.md、crates/speclink-core/tests/golden/neutral-cli.snapshot.md、crates/speclink-core/tests/golden/neutral-tool-call.snapshot.md、crates/speclink-core/tests/golden/assets.lock、.claude/skills/speclink-improve/SKILL.md、.agents/skills/speclink-improve/SKILL.md（後兩者與其餘 35 份 SKILL.md 由 speclink update 再生，只有版號行變動）
  - Removed: （無）

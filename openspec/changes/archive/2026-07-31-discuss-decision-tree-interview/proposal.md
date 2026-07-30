## Why

speclink-discuss 技能的 interview 模式現行「挑最重要的先問」，且無結構性查證要求——開場 scout 明文淺掃（數秒、至多 5 檔）、「Ground in reality」只是軟指示，實際體感是「簡單看過就回答」，未解決的設計問題容易滲進後續 proposal。討論 grill-mode-in-discuss 評估了 mattpocock 的 grilling skill，結論是其「決策樹遍歷」紀律可以拆出來全面採用（「relentless 不准停」則不採用），同時修掉查證深度不足的缺口。

目標使用者是透過 AI 代理跑 SDD 的開發者／PO／PM；使用情境是 workflow 的 discuss 階段（speclink-discuss 技能，工具面同時涵蓋 claude 與 codex 兩份技能實例與內嵌 assets）。

## What Changes

- speclink-discuss 技能 interview 模式的預設提問策略換成決策樹遍歷：開場先攤開決策空間（root 為「這題到底在決定什麼」，展開子決策與依賴邊），依依賴順序一次一題——上游決策先解，因為下游問題的形狀取決於上游答案。
- 每題必附建議答案，且建議必附 Evidence（檔案路徑或查證結果）——把 assumptions 模式既有的 Evidence 慣例升級為 interview 模式的硬規則。
- 事實／決策分診：每個節點解決前先判斷是事實（環境查得到）還是決策（使用者裁定）；事實類沿樹逐節點以 Grep／Read 自行查證，不問使用者、不得憑印象作答。查證深度跟著樹走：深讀花在確定會走到的分支上，被剪掉的分支不預讀。
- 記錄內容慣例：首輪 Position 攤開初始決策空間（可含 ASCII 樹），之後每輪解一個節點，中途發現的新分支記入該輪 Open——維持既有 append-only 規則，Open ledger 因此成為精確的「樹前緣」。
- 三處技能實例同步更新：內嵌 assets（`crates/speclink-core/assets/skills/discuss.md`）、claude 技能（`.claude/skills/speclink-discuss/SKILL.md`）、codex 技能（`.agents/skills/speclink-discuss/SKILL.md`）。
- render golden 同批再生：四份 snapshot 均內嵌 discuss 技能文字，於乾淨樹以 UPDATE_GOLDEN=1 執行 render_golden 測試再生並審視 diff。

相容性影響：引擎零改動，所有 CLI 指令的人眼輸出與 --json shape 不變；golden snapshot 的變更是本提案的刻意產出（技能文字更新），同批更新並在此記載，無使用者遷移需求；既有討論記錄無需遷移，新舊記錄格式一致。

## Non-Goals

- 不新增 grill 模式、第三種輪 mode 值或觸發詞——決策樹是預設紀律，不是選配。
- 不採用 relentless 停止條件（「不全解完不放行」）——one nudge maximum 與 Deferred 續留，停止條件維持使用者主導。
- 不改引擎行為：speclink-core 的 discuss 模組與 speclink-cli 的任何指令、旗標、輸出均不動。
- 不改討論文件骨架與輪模板：Context／Rounds／Conclusion 結構與 Focus／Position／Ruled out／Open 欄位沿用。
- 不加深開場 scout（它只負責選模式）；assumptions 模式保留不動（脈絡足夠時等於預填好答案的樹一次攤開）。
- 不整包照抄 grilling 原文——已於討論中裁定拆用。

## Capabilities

### New Capabilities

- `discuss-skill`: speclink-discuss 技能的 interview 提問紀律——決策樹遍歷、每題附建議答案與 Evidence、事實／決策分診。

### Modified Capabilities

(none)

## Impact

- Affected specs: 新增 `discuss-skill`
- Affected code:
  - Modified:
    - `crates/speclink-core/assets/skills/discuss.md`
    - `.claude/skills/speclink-discuss/SKILL.md`
    - `.agents/skills/speclink-discuss/SKILL.md`
    - `crates/speclink-core/tests/golden/claude.snapshot.md`
    - `crates/speclink-core/tests/golden/codex.snapshot.md`
    - `crates/speclink-core/tests/golden/neutral-cli.snapshot.md`
    - `crates/speclink-core/tests/golden/neutral-tool-call.snapshot.md`
  - New: (none)
  - Removed: (none)
- 影響的 crate／app：speclink-core（assets 與 golden 測試資料）；引擎程式碼與 CLI 不動。
- 影響的技能與工具：speclink-discuss 技能，claude（`.claude/skills/`）與 codex（`.agents/skills/`）兩者。

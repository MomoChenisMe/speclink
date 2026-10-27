## Why

speclink-archive 技能的步驟 5「Clean up tracking file」要求在步驟 6 執行 speclink archive 之前刪除 .speclink/touched/<change>.json，但引擎的 @trace 檔案清單來源是「evidence 記錄非空就用記錄、空了才退回掃描工作樹的髒檔」。先刪記錄等於強制走退路，把平行 session 或其他無關的未提交檔案灌進正典規格的 @trace code 清單——此事已實際發生：本專案 discuss-propose-from-docs 封存時，一份屬於他條工作線的 CLAUDE.md 被記進 touched，若照技能文字先刪記錄，該檔會被寫進兩份正典規格的追溯清單。

技能文字同時與正典 verify-evidence 的 Requirement「archive trace 由 evidence 建立」牴觸，也與引擎原始碼的既有註解（明示 touched 記錄應留在原地供 commit 技能使用）牴觸，且技能檔內部自相矛盾：bulk archive 段說刪除在封存「之後」，步驟 5 卻要求在封存之前。

目標使用者是透過 AI 代理跑 SDD 的開發者／PO／PM；使用情境是 workflow 的 archive 階段（speclink-archive 技能，工具面涵蓋 claude 與 codex 技能實例與內嵌 assets）。

## What Changes

- **刪除時機改到封存之後**：技能檔規定 .speclink/touched/<change>.json 的刪除 SHALL 排在 speclink archive 執行完成之後；封存前 SHALL NOT 刪除該檔，理由（evidence 記錄是 @trace 的來源、刪除會退回掃描髒檔）隨文字寫明。單一封存與 bulk 封存兩段採同一時序。
- **提交後才清理**：技能檔明定該檔同時是 commit 技能的檔案歸屬來源，刪除 SHALL 排在提交之後，避免提交尚未完成就失去檔案清單。
- **修正 @trace 來源敘述**：bulk archive 段目前寫「工作樹的髒檔集就是 @trace 來源」，改為與引擎一致的條件式敘述——有 evidence 記錄時以記錄為準、無記錄時才退回髒檔集；工作樹整潔的要求保留，但理由改述為避免無記錄時的退路污染。
- **落地面**：archive 技能檔三處實例同步（內嵌 assets、claude 與 codex 技能目錄），render golden 同批再生（四份 snapshot 均內嵌 archive 技能文字，於乾淨樹以 UPDATE_GOLDEN=1 執行 render_golden 測試再生並審視 diff）。

相容性影響：引擎零改動，所有 CLI 指令的人眼輸出與 --json shape 不變；golden snapshot 的變更是本提案的刻意產出（技能文字更新），同批更新並在此記載。既有封存流程的產物不受影響——本變更只改代理人執行順序的指示，不改任何指令語意。

## Non-Goals

- 不改引擎：archive 的 @trace 產生邏輯（evidence 記錄優先、缺席才退回髒檔）維持現狀，本變更只讓技能文字與該邏輯一致。
- 不改正典 verify-evidence：該規格對 trace 來源的規定本來就正確，是技能文字偏離了它。
- 不追加自動化守門（例如封存前拒絕髒樹）：單一封存目前不強制整潔工作樹，加上會改變引擎行為，超出技能文字修正的範圍。
- 不回頭修補既有已封存規格的 @trace 內容：歷史記錄按當時狀態保存，本變更只避免未來再次發生。
- 不改 commit 技能：它讀取 touched 記錄的方式不變，本變更只保證該檔在提交前仍然存在。

## Capabilities

### New Capabilities

- `archive-skill`: speclink-archive 技能的追溯完整性紀律——touched 記錄的刪除時機與 @trace 來源敘述。

### Modified Capabilities

(none)

## Impact

- Affected specs: 新增 `archive-skill`
- Affected code:
  - Modified:
    - `crates/speclink-core/assets/skills/archive.md`
    - `.claude/skills/speclink-archive/SKILL.md`
    - `.agents/skills/speclink-archive/SKILL.md`
    - `crates/speclink-core/tests/golden/claude.snapshot.md`
    - `crates/speclink-core/tests/golden/codex.snapshot.md`
    - `crates/speclink-core/tests/golden/neutral-cli.snapshot.md`
    - `crates/speclink-core/tests/golden/neutral-tool-call.snapshot.md`
  - New: (none)
  - Removed: (none)
- 影響的 crate／app：speclink-core（assets 與 golden 測試資料）；引擎程式碼與 CLI 不動。
- 影響的技能與工具：speclink-archive 技能，claude（`.claude/skills/`）與 codex（`.agents/skills/`）兩者。

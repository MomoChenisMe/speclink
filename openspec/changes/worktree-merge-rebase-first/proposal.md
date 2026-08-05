## Why

worktree 平行開發的合併收尾走一般 git merge，主分支在 worktree 開出後一旦前進（例如另一個 worktree 先合回），每次收尾都產生合併節點，線圖持續分岔、可讀性隨平行 change 數惡化。討論 worktree-flow-gaps 裁定改為 rebase-first：speclink/* 是本地拋棄式分支、從不推送，改寫其歷史零風險；審查章指紋是檔案內容雜湊、不綁 commit hash，rebase 不影響蓋章。目標使用者是以 worktree-merge 技能收尾平行開發的開發者（claude 與 codex 兩種工具的生成產物同源同批更新）。

## What Changes

- worktree-merge 技能正典模板（crates/speclink-core/assets/skills/worktree-merge.md）的合併步驟改為 rebase-first 階梯：先於 worktree 內把 speclink/<change名> rebase 到合併目標分支，成功後於主資料夾以 fast-forward 限定方式合併（線圖成直線）；rebase 衝突時中止 rebase（分支完整復原）並退回現行的一般 merge；merge 仍衝突時維持既有守則——中止、回報衝突檔案、絕不代解。最壞情況行為等於現狀。
- 釘住技能內文的一致性測試（crates/speclink-core/tests/it/render_golden.rs 的 worktree-merge 測試）同批補上 rebase-first 與 fallback 階梯的斷言；既有 merge --abort 斷言因 fallback 保留而繼續成立。
- 內嵌 asset 內容一變就要同批推進版本鎖：crates/speclink-core/src/init.rs 的 MARKER_VERSION 由 v1.13.0 推進為 v1.14.0（本 repo 慣例：內容變動即 minor +1），並重生 crates/speclink-core/tests/golden/ 下的快照與 assets.lock 指紋。
- 以引擎再生機制更新本 repo 的生成產物：.claude/skills/ 與 .agents/skills/ 的 speclink-worktree-merge/SKILL.md，以及版本戳波及的 CLAUDE.md、AGENTS.md 與兩處全部技能檔 frontmatter 版號（本 repo 的 tools 為 claude 與 codex，兩者模板同源同批更新）。

相容性影響：不涉及任何 CLI 指令的人眼或 --json 輸出、無設定欄位變更；變的是技能生成內文與指令檔的標記版號（skills 與指令檔屬注入產物，再生即更新）。既有專案的指令檔版號會落後於 v1.14.0，依既有過期偵測提示使用者重生。已存在的合併節點屬歷史，不回改。

## Non-Goals

- 不改引擎程式碼行為（僅技能 assets、版本戳常數與一致性測試）
- 不採 squash 合併（失去逐任務 conventional commits 粒度，討論已否決）
- 不回改既有 git 歷史的合併節點
- 不動 apply-with-worktree 技能與 worktree 觀察面（desktop 資料路徑歸同討論扇出的 worktree-data-routing）
- 不放寬「衝突絕不代解、不代 stash、不代提交」的既有紅線

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `worktree-merge-skill`: 「worktree-merge 技能的收尾流程指示」的合併步驟由一般 merge 改為 rebase-first 階梯（worktree 內 rebase 目標分支 → 主資料夾 fast-forward 限定合併；rebase 衝突中止後退回一般 merge；merge 衝突維持中止回報），其餘 preflight、清理與交棒指示不變。

## Impact

- Affected specs: worktree-merge-skill
- Affected code:
  - Modified: crates/speclink-core/assets/skills/worktree-merge.md、crates/speclink-core/tests/it/render_golden.rs、crates/speclink-core/src/init.rs（MARKER_VERSION）、crates/speclink-core/tests/golden/（快照與 assets.lock 重生）、.claude/skills/ 與 .agents/skills/ 的 speclink-worktree-merge/SKILL.md、兩處其餘 speclink-*/SKILL.md（frontmatter 版號）、CLAUDE.md、AGENTS.md（版本戳重生）
  - New: (none)
  - Removed: (none)

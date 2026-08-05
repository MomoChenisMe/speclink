## Why

worktree 平行開發的合併收尾走一般 git merge，主分支在 worktree 開出後一旦前進（例如另一個 worktree 先合回），每次收尾都產生合併節點，線圖持續分岔、可讀性隨平行 change 數惡化。討論 worktree-flow-gaps 裁定改為 rebase-first：speclink/* 是本地拋棄式分支、從不推送，改寫其歷史零風險；審查章指紋是檔案內容雜湊、不綁 commit hash，rebase 不影響蓋章。目標使用者是以 worktree-merge 技能收尾平行開發的開發者（claude 與 codex 兩種工具的生成產物同源同批更新）。

## What Changes

- worktree-merge 技能正典模板（crates/speclink-core/assets/skills/worktree-merge.md）的合併步驟改為 rebase-first 階梯：先於 worktree 內把 speclink/<change名> rebase 到合併目標分支，成功後於主資料夾以 fast-forward 限定方式合併（線圖成直線）；rebase 衝突時中止 rebase（分支完整復原）並退回現行的一般 merge；merge 仍衝突時維持既有守則——中止、回報衝突檔案、絕不代解。最壞情況行為等於現狀。
- 釘住技能內文的一致性測試（crates/speclink-core/tests/it/render_golden.rs 的 worktree-merge 測試）同批補上 rebase-first 與 fallback 階梯的斷言；既有 merge --abort 斷言因 fallback 保留而繼續成立。
- 以引擎再生機制更新本 repo 的生成產物 .claude/skills/speclink-worktree-merge/SKILL.md（本 repo 工具僅 claude；codex 模板同源，於啟用 codex 的專案再生時生效）。

相容性影響：不涉及任何 CLI 指令的人眼或 --json 輸出、無設定欄位變更；變的是技能生成內文（skills 屬注入產物，再生即更新）。已存在的合併節點屬歷史，不回改。

## Non-Goals

- 不改引擎程式碼行為（僅技能 assets 與其一致性測試）
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
  - Modified: crates/speclink-core/assets/skills/worktree-merge.md、crates/speclink-core/tests/it/render_golden.rs、.claude/skills/speclink-worktree-merge/SKILL.md
  - New: (none)
  - Removed: (none)

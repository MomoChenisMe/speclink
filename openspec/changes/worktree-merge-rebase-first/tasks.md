## 1. 測試先行——內文斷言紅燈

- [ ] 1.1 [測試先行] 更新 crates/speclink-core/tests/it/render_golden.rs 的 worktree_merge_skill_states_preflight_conflict_and_cleanup：補「worktree-merge 技能的收尾流程指示」rebase-first 階梯的斷言——內文含 rebase 目標分支指示、--ff-only 合併指示、rebase --abort 後退回一般 merge 的 fallback 指示；既有 merge --abort 與不代編衝突斷言保留。驗證：cargo test -p speclink-core --test it worktree_merge 紅燈（現行內文無 rebase 指示）。 <!-- speclink-task:tsk_01KZ92E7C9XK2QJTTR3Q3S2FNF -->

## 2. 正典模板改寫與再生

- [ ] 2.1 改寫 crates/speclink-core/assets/skills/worktree-merge.md 的合併步驟為 rebase-first 階梯：worktree 內 git rebase 合併目標分支 → 成功後主資料夾 git merge --ff-only speclink/<change名>；rebase 衝突時 git rebase --abort（分支完整復原）退回一般 merge；一般 merge 衝突維持中止回報、不代解；preflight、清理與交棒段落不動。行為：技能指示線圖直線化且最壞情況等於現狀。驗證：1.1 斷言轉綠、該測試檔其餘既有斷言不破。 <!-- speclink-task:tsk_01KZ92E7C9J106CAS2WJZX38MP -->
- [ ] 2.2 以引擎再生機制更新本 repo 生成產物 .claude/skills/speclink-worktree-merge/SKILL.md，內容與正典模板同步（含 rebase-first 階梯與 fallback 指示）。驗證：重生後讀取 SKILL.md 含「rebase」與「--ff-only」字樣，且 git diff 僅該產物與本 change 涉及檔案變動。 <!-- speclink-task:tsk_01KZ92E7C9MPTE8CJR2H6KXT40 -->

## 3. 回歸收尾

- [ ] 3.1 全量回歸：cargo test -p speclink-core --test it render_golden:: 全綠（其他技能 golden 零變動），確認無 CLI 人眼或 --json 輸出波及。驗證：指令通過且 git status 無非預期檔案變動。 <!-- speclink-task:tsk_01KZ92E7C9NA3VY1Q2CQ12Z52C -->

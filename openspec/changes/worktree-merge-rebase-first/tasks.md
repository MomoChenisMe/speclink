## 1. 測試先行——內文斷言紅燈

- [x] 1.1 [測試先行] 更新 crates/speclink-core/tests/it/render_golden.rs 的 worktree_merge_skill_states_preflight_conflict_and_cleanup：補「worktree-merge 技能的收尾流程指示」rebase-first 階梯的斷言——內文含 rebase 目標分支指示、--ff-only 合併指示、rebase --abort 後退回一般 merge 的 fallback 指示；既有 merge --abort 與不代編衝突斷言保留。驗證：cargo test -p speclink-core --test it worktree_merge 紅燈（現行內文無 rebase 指示）。 <!-- speclink-task:tsk_01KZ92E7C9XK2QJTTR3Q3S2FNF -->

## 2. 正典模板改寫與再生

- [x] 2.1 改寫 crates/speclink-core/assets/skills/worktree-merge.md 的合併步驟為 rebase-first 階梯：worktree 內 git rebase 合併目標分支 → 成功後主資料夾 git merge --ff-only speclink/<change名>；rebase 衝突時 git rebase --abort（分支完整復原）退回一般 merge；一般 merge 衝突維持中止回報、不代解；preflight、清理與交棒段落不動。行為：技能指示線圖直線化且最壞情況等於現狀。驗證：1.1 斷言轉綠、該測試檔其餘既有斷言不破。 <!-- speclink-task:tsk_01KZ92E7C9J106CAS2WJZX38MP -->
- [x] 2.2 內嵌 asset 內容變動須同批推進版本鎖：crates/speclink-core/src/init.rs 的 MARKER_VERSION 由 v1.13.0 推進為 v1.14.0，再依序重生 golden 與 asset 指紋鎖（UPDATE_GOLDEN=1 → UPDATE_ASSETS_LOCK=1，皆跑 cargo test -p speclink-core --test it render_golden::）。行為：版本戳前進，crates/speclink-core/tests/golden/ 下的快照與 assets.lock 與新內文同步。驗證：render_golden::embedded_assets_are_locked_to_the_product_version 與 claude_rendering_with_the_worktree_policy_on_is_bit_identical_to_golden 轉綠。 <!-- speclink-task:tsk_01KZ93MBFWYW1TX5MYY6JJ49AX -->
- [x] 2.3 以引擎再生機制（speclink update）更新本 repo 生成產物：.claude/skills/ 與 .agents/skills/ 下的 speclink-worktree-merge/SKILL.md 內容與正典模板同步（含 rebase-first 階梯與 fallback 指示）；版本戳波及的 CLAUDE.md、AGENTS.md 與兩處全部 speclink-*/SKILL.md 的 frontmatter 版號同批重生。驗證：重生後讀取 speclink-worktree-merge/SKILL.md 含「rebase」與「--ff-only」字樣、CLAUDE.md 與 AGENTS.md 的標記版號為 v1.14.0，且 git diff 中除 worktree-merge 技能內文外只有版號行變動。 <!-- speclink-task:tsk_01KZ92E7C9MPTE8CJR2H6KXT40 -->

## 3. 回歸收尾

- [x] 3.1 全量回歸：cargo test -p speclink-core --test it render_golden:: 全綠（golden 差異僅限 worktree-merge 內文與版本戳，其他技能內文零變動），確認無 CLI 人眼或 --json 輸出波及。驗證：指令通過且 git status 無非預期檔案變動。 <!-- speclink-task:tsk_01KZ92E7C9NA3VY1Q2CQ12Z52C -->

## 1. 資產字面與正典三連動

- [x] 1.1 更新 crates/speclink-core/assets/skills/apply-worktree-post.md 的 W3 逐字交棒句：在 /speclink-review（工藝品質）∥ /speclink-verify（規格符合度）之後補列 /speclink-quality（兩站合跑），同句其餘字面（Apply baseline 提醒、蓋章補提交、worktree-merge 收尾）不動——落實 worktree-apply-skill delta「apply-with-worktree 技能的收尾指示」的三入口要求。驗證：grep 該檔 W3 交棒句，同一句內同時出現 speclink-review、speclink-verify、speclink-quality 三個字樣。 <!-- speclink-task:tsk_01M11772P688HDTQ653W7CHEW7 -->
- [x] 1.2 版號與 lock 同步：crates/speclink-core/src/init.rs 的 ASSET_VERSION 常數遞增（v1.22.0 → v1.23.0），並以 UPDATE_ASSETS_LOCK=1 開關重生 crates/speclink-core/tests/golden/assets.lock。驗證：assets.lock 首行 version 與 init.rs 常數一致，且 cargo test -p speclink-core --test it 的 lock 守門測試通過。 <!-- speclink-task:tsk_01M11772P682BKB6AYBQBZR609 -->
- [x] 1.3 golden 刻意更新：以 UPDATE_GOLDEN=1 cargo test -p speclink-core --test it render_golden:: 重生 crates/speclink-core/tests/golden/claude-worktree.snapshot.md，檢視 diff 僅 W3 段新增 quality 入口與版號行變動；同一開關會把其餘 4 份 golden（claude、codex、neutral-cli、neutral-tool-call）的版號行一併重寫，盤點 diff 時以「入口名單字面＋版號行」為預期集合。驗證：不帶環境變數重跑 cargo test -p speclink-core --test it render_golden:: 全綠。 <!-- speclink-task:tsk_01M11772P68Q0P8RKAPV3SMYKG -->

## 2. 衍生技能檔再生與收尾驗證

- [x] 2.1 以 speclink update 再生技能檔，讓 claude 與 codex 兩個目標的 apply-with-worktree 技能取得新交棒句——落實 scenario「內文含停點與正典順序交棒指示」的 THEN。驗證：grep .claude/skills/speclink-apply-with-worktree/SKILL.md 與 .agents/skills/speclink-apply-with-worktree/SKILL.md 的 W3 交棒句皆明列 review、verify、quality 三入口；git status 盤點異動僅含預期檔案（apply-with-worktree、apply、worktree-merge 三技能的六份 SKILL.md 內文異動＋其餘技能檔的版號行再生）。 <!-- speclink-task:tsk_01M11772P6M1MMKPXQGCWH5T8G -->
- [x] 2.2 收尾回歸：cargo test -p speclink-core --test it 全綠（golden 與 lock 守門皆過），speclink validate worktree-handoff-quality-mention 通過。驗證：兩個指令的輸出無失敗項。 <!-- speclink-task:tsk_01M11772P66V5G3BXWGXD6PAYX -->

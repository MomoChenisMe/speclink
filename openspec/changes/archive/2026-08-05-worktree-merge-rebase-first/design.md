## Context

（本 design 於實作與合併完成後補齊，內容反映已落地的決策，非事前設計稿。）

worktree-merge 技能是 worktree 平行開發的收尾半場：`/speclink-apply-with-worktree` 在 worktree 內提交後即止步，合併是人為觸發的下一步。技能內文由 `crates/speclink-core/assets/skills/worktree-merge.md` 這份正典模板生成，claude 與 codex 兩種工具的產物同源同批更新；內文一致性由 `crates/speclink-core/tests/it/render_golden.rs` 的斷言與 `crates/speclink-core/tests/golden/` 的快照兩層釘住，內嵌 asset 的內容指紋另由 `assets.lock` 鎖定。

原內文的合併步驟是單一的一般 merge。speclink/* 分支從 worktree 建立時的主分支狀態開出，主分支只要在那之後前進（平行開發下的常態：另一個 worktree 先合回），收尾就必然產生合併節點，線圖隨平行 change 數持續分岔。討論 worktree-flow-gaps 裁定改走 rebase-first，理由有二：speclink/* 是本地拋棄式分支、從不推送，改寫其歷史零風險；審查章的指紋錨是檔案內容雜湊、不綁 commit hash，rebase 不會使已蓋的章失效。

## Goals / Non-Goals

**Goals:**

- 合併目標分支已前進時，收尾仍讓線圖保持直線（fast-forward 落地）
- 最壞情況（rebase 或 fast-forward 走不通）的可觀察行為與改動前的單一 merge 流程相同
- 「衝突絕不代解、不代 stash、不代提交」的既有紅線一條不放寬，並把 rebase 衝突納入同一條紅線
- 使用者看得出本次是以哪一種方式落地（直線或合併節點）

**Non-Goals:**

- 不改引擎程式碼行為——本次只動技能 asset、版本戳常數與一致性測試
- 不採 squash（會失去逐任務 conventional commits 的粒度，討論已否決）
- 不回改既有歷史裡的合併節點
- 不動 apply-with-worktree 技能與 worktree 觀察面
- 不改任何 CLI 指令的人眼或 `--json` 輸出，無設定欄位變更

## Decisions

### D1 合併走三階梯：rebase → fast-forward → 一般 merge

內文指示的順序固定為：(1) 於 **worktree** 內 `git rebase <目標分支>`；(2) 成功後於 **主資料夾** `git merge --ff-only speclink/<change名>`；(3) 走不通時退回主資料夾的一般 `git merge`。

階梯而非二選一，是因為兩個失敗點的成因不同但出口相同：rebase 可能撞衝突，fast-forward 可能因為「rebase 與合併之間目標分支又前進」（另一個 worktree 剛好先落地）而被拒。兩者都退回一般 merge，並明白告訴使用者本次留下合併節點——沉默地換一種落地方式，會讓使用者以為線圖是直線的。

替代方案：偵測目標分支未前進時才 fast-forward、否則直接 merge。否決——那要先問一次 `merge-base`，多一個判斷點卻換不到更好的結果；`--ff-only` 被拒本身就是最準確的偵測。

### D2 rebase 在 worktree、合併在主資料夾

rebase 的對象是 speclink 分支自己的歷史，必須在持有該分支的 worktree 內執行（`git -C <worktree-path> rebase`）；合併則在主資料夾。這使得技能前提敘述的原文「每一步都在主資料夾執行」不再成立，內文因此同批修正為「步驟由主資料夾驅動，rebase 以 `-C <worktree-path>` 指向 worktree」。spec 為此立了一條顯式約束：前提敘述 SHALL NOT 宣稱所有步驟都不在 worktree 內執行——這種「文件與步驟悄悄不一致」的錯，正是技能內文最容易累積的一種。

### D3 rebase 衝突與 merge 衝突同一條紅線

rebase 撞衝突時指示 `git rebase --abort`（分支完整復原，回到 rebase 前的狀態），然後退回一般 merge——不代解、不挑邊、不留半套。守則清單同批加入「不代解 rebase 衝突」，與既有的「不代解 merge 衝突」並列。兩條衝突處置疊起來之後，最壞路徑的可觀察行為與改動前一致：一般 merge 衝突 → 中止 → 回報衝突檔案清單 → 停下等使用者。

替代方案：rebase 衝突時就地停下、要使用者自己解。否決——rebase 是本次新增的最佳化，它失敗不該讓使用者比以前多做一件事；退回舊路徑才是「最壞情況等於現狀」的意思。

### D4 asset 內容一動，版本戳與兩道鎖同批推進

`crates/speclink-core/assets/skills/*.md` 是內嵌 asset，內容變動須連帶推進 `crates/speclink-core/src/init.rs` 的 `MARKER_VERSION`（本 repo 慣例：內容變動即 minor +1，v1.13.0 → v1.14.0），並重生 `crates/speclink-core/tests/golden/` 下的快照與 `assets.lock` 指紋。三者不同批就會紅燈，這是刻意的：版本戳是既有專案偵測「指令檔過期」的唯一依據，內文改了而版號沒動，等於讓所有既有專案錯過重生。

寫入順序固定：先改 asset 內文 → 推進 `MARKER_VERSION` → `UPDATE_GOLDEN=1` 重生快照 → `UPDATE_ASSETS_LOCK=1` 重生指紋鎖。順序顛倒會拿舊內文去生成新鎖，鎖住錯的東西。任一步失敗即停在該步，此時 golden 測試為紅——不會留下「鎖已更新而內文未更新」這種看起來全綠的半套狀態。

### D5 本 repo 的生成產物以引擎自己的再生機制更新

`.claude/skills/` 與 `.agents/skills/` 下的 SKILL.md、以及 `CLAUDE.md`／`AGENTS.md` 的注入區塊，都是生成產物，一律經 speclink 自己的再生路徑更新，不手改。版本戳前進會波及兩處全部 `speclink-*/SKILL.md` 的 frontmatter 版號——那是預期中的大面積 diff；審查時的判準是「除 worktree-merge 內文外只有版號行變動」。

## Implementation Contract

- **行為**：再生後的 `speclink-worktree-merge/SKILL.md` 內文含 (1) preflight 三條件與「不代 stash／不代提交」；(2) 合併目標分支確認（`branch --show-current`，停在 `speclink/*` 或 detached 即停、不代切換）；(3) worktree 內 rebase 目標分支、成功後主資料夾 `--ff-only` 合併；(4) rebase 衝突以 `rebase --abort` 復原後退回一般 merge；(5) fast-forward 被拒走同一出口並告知留下合併節點；(6) 一般 merge 衝突即停、回報檔案清單、`merge --abort`、不代編；(7) 合併成功後移除 worktree 並刪除分支；(8) 成功輸出標示本次以 fast-forward 或合併節點落地；(9) 提示續走品質站或封存。前提敘述不得宣稱所有步驟都不在 worktree 內執行。
- **介面／資料形狀**：無 CLI 指令、旗標、`--json` 欄位或設定欄位的增刪改。變的是技能生成內文與注入產物的標記版號（`v1.13.0` → `v1.14.0`）。
- **失敗模式**：技能內文描述的失敗處置即上列 (4)(5)(6)；建置面的失敗模式是 D4 的順序性紅燈（內文、版號、golden、assets.lock 任一不同步即測試失敗）。
- **驗收**：`cargo test -p speclink-core --test it render_golden::` 全綠，其中 `worktree_merge_skill_states_preflight_conflict_and_cleanup` 涵蓋 rebase-first 與 fallback 斷言、`embedded_assets_are_locked_to_the_product_version` 與各 golden 快照比對通過；`git diff` 中除 worktree-merge 技能內文外只有版號行變動。
- **範圍邊界**：in——`crates/speclink-core/assets/skills/worktree-merge.md`、`crates/speclink-core/src/init.rs`（僅 `MARKER_VERSION`）、`crates/speclink-core/tests/it/render_golden.rs`、`crates/speclink-core/tests/golden/`（重生）、兩處 `speclink-*/SKILL.md` 與 `CLAUDE.md`／`AGENTS.md`（再生）；out——引擎程式碼行為、CLI、desktop、remote／server、既有 git 歷史。

## Risks / Trade-offs

- [回歸對照] 版本戳波及兩處全部技能檔與兩份指令檔，diff 面積大，真正的內文變動容易被淹沒 → 以「除 worktree-merge 內文外只有版號行變動」為審查判準，golden 快照逐位元比對把關
- [跨平台] rebase 與 `merge --ff-only` 在 Windows／macOS／Linux 的 git 行為一致，技能內文不含平台分支；`-C <path>` 亦為跨平台寫法 → 無平台專屬路徑要維護
- [rebase 改寫歷史] 僅限本地、從不推送的 speclink/* 分支；審查章綁檔案內容雜湊而非 commit hash，rebase 後仍成立 → 不擴及任何其他分支
- [既有專案版號落後] v1.14.0 前的專案指令檔會被既有過期偵測標為落後，需重生 → 屬既有機制的正常提示，非本次新增負擔
- [fallback 路徑較少被走到] 平時多半 fast-forward 成功，退回一般 merge 的分支缺乏日常演練 → 由 golden 斷言釘住該段內文存在，避免日後被誤刪

## Migration Plan

無資料或設定遷移。既有專案下次執行技能再生即取得新內文；已存在的合併節點屬歷史，不回改。回滾即 revert 對應 commit（含版本戳與鎖）。

## Open Questions

（無——階梯順序、fallback 出口與紅線範圍已由討論 worktree-flow-gaps 裁決。）

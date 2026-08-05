---
topic: worktree 流程的三個缺口:抽屜任務未同步、審查章誤判 stale、merge 線圖分岔
slug: worktree-flow-gaps
status: promoted
promoted_to: worktree-data-routing, worktree-merge-rebase-first
created: 2026-08-05
created_by: MomoChen <momochenisme@gmail.com>
---

# Discussion: worktree 流程的三個缺口:抽屜任務未同步、審查章誤判 stale、merge 線圖分岔

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

使用者一次回報三個 worktree 流程問題:(1) 抽屜任務分頁計數正確但下方勾選全空、merge 後恢復;(2) worktree 內蓋章的 change 被誤判 reviewedStale(附完整根因分析);(3) merge 線圖分岔、詢問可否 rebase 化。模式:假設模式——scout 找到 apps/desktop/core/src/query.rs、manage.rs、packages/ui/src/components/RichDetailDrawer.tsx、crates/speclink-core/src/tasks.rs 等直接相關檔案。問題 2 使用者自帶逐行分析,以逐條查證方式處理。相關 change:config-station-canon-guard(已合併,誤標實例)、semantic-color-system、review-stamp-violet(worktree 流程進行中)。

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-08-05)

**Focus**: 三個回報的定位與查證——它們是不是同一件事?
**Position**: 問題 1 與 2 是同族缺陷(讀取路徑繞過 WorktreeOverlay),且順藤挖出未回報的寫入面雷;問題 3 技術上可行:
- 問題 2 使用者的根因分析逐條查證全數屬實:query.rs:67-68 的 read_file 走 ctx.workspace.root 繞過 overlay,任務數卻走 overlay 後的 store(query.rs:35,69);facts 早就帶著每個 change 的 worktree 路徑(query.rs:21-30);既有四態回歸測試(query.rs:584-624)無任何 worktree 案例。
- 問題 1 根因:抽屜分頁徽章來自 list_changes_at(有 overlay,正確),下方任務原文來自 document_at(query.rs:279,ctx.store 直讀主 checkout,錯誤)。同族入口還有 status_at(query.rs:260)與 change_capabilities_at(query.rs:297)。
- 寫入面雷(未回報):抽屜勾選框對 worktree change 未停用(RichDetailDrawer.tsx:218-232),後端 set_task_done_at/set_all_tasks_at(manage.rs:111,153)無 refuse_if_worktree_is_open 守門——點下去會寫進主 checkout 的 tasks.md,與 worktree 分支分岔,merge 時撞衝突。現行守門只覆蓋退回提案中(manage.rs:81)。
- 問題 3:worktree-merge 技能目前是一般 git merge(SKILL.md:66)。rebase-first(worktree 內 rebase 主分支→主 checkout --ff-only 快轉)可行:speclink/* 是本地拋棄式分支,改寫歷史無風險;審查章指紋是檔案內容 sha256、不綁 commit hash,rebase 不弄髒章。
**Open**: 寫入面守門還是路由進 worktree;問題 1+2 開一個 change 還是兩個;merge 技能要不要真的改 rebase-first。

### Round 2 — assumptions (2026-08-05)

**Focus**: 三個裁決:寫入面處理、change 切法、merge 線圖。
**Position**: 寫入路由進 worktree(使用者裁定,推翻助手原建議的守門方案);問題 1+2+寫入面合開一個 change;worktree-merge 技能改 rebase-first:
- 路由可行性錨點:tasks::complete 的全部側效只認兩個把手(crates/speclink-core/src/tasks.rs:274-327)——tasks.md 與開工章走 store,touched 記錄/git_changed_files/head_commit 走 ws。把 ProjectContext 整個重新定根到 facts 的 e.path(worktree 是完整 checkout,自帶 openspec/ 與 git),側效自然一致落在 worktree 內,touched 歸因掃 worktree 髒檔反而比現狀更正確,不需逐點改側效。
- 合一個 change 的理由:根因同源(desktop core 資料路徑對 overlay 涵蓋不完整)、改動集中同一支 query.rs/manage.rs、共用同一套 worktree 測試夾具。
- rebase-first 只改 SKILL.md,引擎零改動;遇衝突 abort 回報的守則沿用(rebase --abort);已亂掉的歷史線圖不回改。
**Ruled out**: 寫入守門擋下——顯示修正後與操作不一致,worktree change 淪為唯讀櫥窗,使用者裁定體驗一致優先;squash 合併——失去逐任務 conventional commits 粒度,與專案 commit 習慣相抵;維持一般 merge——線圖可讀性輸給 rebase-first 且改動成本極低。
**Open**: 無——全數進結論。

### Round 3 — assumptions (2026-08-05)

**Focus**: 全面掃雷——desktop core 還有哪些入口對 worktree change 讀錯／寫錯位置?rebase 失敗的 fallback 是什麼?
**Position**: 逐一盤點 desktop core 全部 per-change 入口,在原已知 6 個落點外新發現 8 個;rebase 失敗採「abort → 退回一般 merge → 再衝突才停」的階梯:
- 讀取面新增 4 個繞過 overlay:change_meta_at(manage.rs:43,抽屜 header 的 created/started 欄位——開工章蓋在 worktree,主 checkout meta 缺)、validate_at 與 analyze_at(verbs.rs:16,27,抽屜「分析」鈕分析的是主 checkout 舊 artifacts,報告整份失真)、search_workspace_at(search.rs,全文搜尋掃主 checkout,worktree 新內容搜不到)。
- 寫入面新增 3 個:move_task_at(manage.rs:245,任務拖排寫主 checkout tasks.md)、reorder_card_at(manage.rs:253,rank 寫主 checkout meta 但看板讀 overlay——worktree 卡拖排會彈回,且 merge 時 meta 兩側互撞)、discard_review_at(verbs.rs:50,UI 可達性低——封存對話框被守門擋在前面,但同屬路由範圍)。
- 守門缺口 1 個:delete_change_at(manage.rs:59-69)完全沒有 refuse_if_worktree_is_open——刪除會 discard 主 checkout 的 change 目錄,但 worktree 分支還在、facts 仍映射,overlay 會讓卡片以幽靈狀態復活。這個屬破壞性生命週期動詞,應比照封存/退回守門,不是路由。
- 確認安全不用動:list_specs_at 與 spec_document_at(正典 specs,worktree 不動正典——封存被擋)、archived_*/cache.rs(封存面)、discussions.rs(討論不進 worktree 流程)、watch_targets_at(已 worktree-aware)、remote/server(crates/speclink-server 是 TeamStore DB 後端,無 git worktree 概念)、CLI(list 已走 overlay,commands.rs:345-357;其他動詞維持既定非目標)。
- rebase fallback 階梯:worktree 內 git rebase 主分支 → 衝突則 git rebase --abort(分支完整復原、零風險)→ 退回現行一般 git merge(rebase 逐 commit 重放可能撞中間狀態衝突,merge 只看最終內容,可能乾淨過;此時合併節點是「真有分岔」的誠實記錄)→ merge 也衝突則 git merge --abort 回報使用者,絕不代解(現行守則)。最壞情況等於今日行為;--ff-only 在 rebase 後主分支又前進的競態下會拒絕而非亂合。
**Ruled out**: rebase 衝突即停不 fallback——rebase 特有的中間狀態衝突會白白擋掉一次本可乾淨完成的 merge;delete 走路由——刪除的語意是「主 checkout 除名」,路由進 worktree 沒有意義,worktree 掛著時就該擋。
**Open**: 無——併入既有結論的 change A(路由範圍擴大 +8 落點、delete 補守門)與 change B(rebase-first 含 fallback 階梯)。

## Conclusion

**Decision**: 扇出兩個 change。(A) desktop core 對 worktree change 的資料路徑一律解析到該 change 的 worktree(observed_facts 含該 change 時重新定根到 e.path;facts 為空時行為與現狀完全相同——既有紅線)。範圍經全面掃雷定案:讀取面 8 個(list_changes_at 的 freshness read_file、document_at、status_at、change_capabilities_at、change_meta_at、validate_at、analyze_at、search_workspace_at),寫入面 5 個(set_task_done_at、set_all_tasks_at、move_task_at、reorder_card_at、discard_review_at);另 delete_change_at 屬破壞性生命週期動詞,不路由、補 refuse_if_worktree_is_open 守門(比照封存/退回,D7 的遺漏)。確認安全不動:list_specs_at、spec_document_at、archived_*/cache.rs、discussions.rs、watch_targets_at、remote/server(TeamStore 無 worktree 概念)、CLI(list 已 overlay,其餘維持非目標)。(B) worktree-merge 技能改 rebase-first 含 fallback 階梯:worktree 內 rebase 主分支 → 衝突則 rebase --abort(分支完整復原)→ 退回現行一般 git merge → 仍衝突則 merge --abort 回報,絕不代解;成功則主 checkout --ff-only 快轉。最壞情況等於今日行為。
**Rationale**: 同一張卡與抽屜的資料源不得劈成兩半——使用者裁定「顯示與操作一致」優先於「守門實作簡單」;重新定根讓 tasks::complete 的全部側效(touched、開工章、git 髒檔歸因)自然落在 worktree 內,正確性反而提升。rebase 特有的中間狀態衝突不該擋掉本可乾淨完成的 merge,故 fallback 到 merge 而非即停。
**Rejected alternatives**: 寫入面守門擋下(顯示對了卻不能操作,體驗劈半);squash 合併(失去逐任務 commit 粒度);維持一般 merge(線圖可讀性差而修正只需改 SKILL.md);rebase 衝突即停不 fallback(白白擋掉可完成的合併);delete 走路由(刪除語意是主 checkout 除名,worktree 掛著時就該擋)。
**Deferred**: 實作形狀(逐 change 重新定根 ProjectContext vs 擴大 WorktreeOverlay 涵蓋)留給 propose/design 裁定;「worktree 資料夾已移除但 facts 未更新」的缺檔 fallback 維持現行「缺檔即 Stale」語意,design 記一筆即可;cache.rs:156 的 reviewStatus(封存清單既存值)在 propose 時確認一遍非第二落點。
**Capture to**: proposal
**Next**: /speclink-propose --from-discussion worktree-flow-gaps

---
topic: 任務勾選後才收合，沒勾的任務一律展開
slug: task-row-expand-by-done
status: promoted
created: 2026-10-13
created_by: MomoChen <momochenisme@gmail.com>
promoted_to: task-row-expand-by-done
---

# Discussion: 任務勾選後才收合，沒勾的任務一律展開

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

使用者看變更詳情頁的任務文件時，希望已勾的任務收合、沒勾的任務展開；目前所有任務列一律從收合開始（packages/ui/src/components/TaskList.tsx 的 useState(false)），沒勾任務的內文與驗證要手動點開才看得到。需求一開始就可驗證（勾選狀態決定展開與否），不需要 grill 階段。相關規格為 desktop-app 的「任務群組標題與章節標籤同款式」（現行寫明任務列預設只顯示標題層），在途變更 doc-tiles-fill-and-round-order: desktop-app, discussion-stage-and-unpromoted-cuts: desktop-app 都在改 desktop-app，但沒有碰這條需求。
Prior discussions: detail-content-redesign, desktop-reading-and-tasks-ux

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-10-13)

**Focus**: 任務列的展開與否要怎麼跟著勾選狀態走
**Position**: 使用者確認五條假設全對——展開與否改由勾選狀態決定，勾選狀態一變就跟著翻。
- 預設：沒勾的任務展開、已勾的收合；沒有內文也沒有驗證的任務沒有展開鈕，不受影響（TaskList.tsx 的 expandable 判斷）
- 勾選狀態一變就跟著變：勾下去收合、取消勾選展開；工具列「全部已完成」全部收合、「重置任務」全部展開
- 手動點箭頭照樣有效，效力持續到這條任務的勾選狀態下次改變；列以 taskKey 為 key，拖曳排序後手動選擇不跑到別列
- 已封存唯讀頁用同一條規則（現行規格本來就寫「展開收合照常」）
- 範圍只有 desktop 顯示：改 desktop-app「任務群組標題與章節標籤同款式」的預設句與「任務列三層與展開」情境，以及 taskList.test.tsx 現有的「預設收合」測試；不改 tasks.md 格式與引擎
- 重開 detail-content-redesign 的「任務列預設只顯示標題」：那次的理由「第一屏看得出層次」對已勾任務仍成立，對沒勾任務不成立——內文與驗證就是接下來要做的事
- 已知代價：剛開工的變更（例如 0/10）打開時會整片展開，頁面變長
**Ruled out**: 只在打開頁面時套用、勾選後不自動收合——使用者明說要「勾選後再收合」；手動選擇永遠優先——勾選後就不會自動收合，與需求矛盾；已封存頁一律收合——多一條例外，且規格本來就寫展開收合照常

## Conclusion

**Decision**: 任務列的展開與否改由勾選狀態決定——沒勾的展開、已勾的收合，勾選狀態一變就跟著翻；只改 desktop 顯示。
- 預設：沒勾且有內文或驗證的任務一開始就展開；已勾的任務一開始就收合；沒有內文也沒有驗證的任務沒有展開鈕，不受影響
- 勾選狀態一變就跟著變：勾下去當下收合、取消勾選當下展開；工具列「全部已完成」讓全部收合、「重置任務」讓全部展開
- 手動點列尾箭頭照樣可以展開或收合，效力持續到這條任務的勾選狀態下次改變為止；拖曳排序後手動選擇跟著原本那條任務走
- 已封存唯讀頁用同一條規則
- 拖曳浮起列（只畫標題層）不變
- 例：群組內 3.1 沒勾（有內文）、3.2 已勾（有內文）、3.3 沒勾（無內文）→ 打開頁面時 3.1 展開、3.2 收合、3.3 沒有箭頭；手動收起 3.1 → 維持收合；接著勾 3.1 → 收合；再取消勾 3.1 → 展開
- 規格：MODIFIED desktop-app「任務群組標題與章節標籤同款式」——「任務列預設 SHALL 只顯示勾選框、序號、標題、D 籤與手動籤」改為依勾選狀態決定預設展開，並改寫情境「任務列三層與展開」、補一個勾選後收合的情境
- 測試：packages/ui 的 taskList.test.tsx 現有「預設只有序號、標題與 D 籤」那條要改成以已勾任務驗收合、以沒勾任務驗展開
- 不改 tasks.md 格式、不改引擎、不改 server-web（TaskList 目前只有 desktop 使用）
- 已知代價：剛開工的變更（例如 0/10）打開時整片展開，頁面變長——使用者接受
**Rationale**: 已勾的任務是做完的事，看標題就夠；沒勾的任務，內文與驗證就是接下來要做的事，預設收合等於每條都要多點一下。上一場 detail-content-redesign 定下「預設只顯示標題」的理由是「第一屏看得出層次」，這個理由對已勾任務仍成立，對沒勾任務不成立，所以只在沒勾的任務上翻轉預設。
**Rejected alternatives**:
- 只在打開頁面時套用、勾選後不自動收合 — 使用者明說要「勾選後再收合」
- 手動選擇永遠優先於勾選狀態 — 勾選後就不會自動收合，與需求矛盾
- 已封存頁一律收合 — 多一條例外，且現行規格本來就寫唯讀時展開收合照常
**Deferred**: none
**Capture to**: spec | tasks
**Next**: /speclink-propose --from-discussion task-row-expand-by-done

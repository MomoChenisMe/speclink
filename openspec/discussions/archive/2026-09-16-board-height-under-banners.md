---
topic: 技能提示出現時看板被壓縮、底部被裁切
slug: board-height-under-banners
status: promoted
created: 2026-09-16
created_by: MomoChen <momochenisme@gmail.com>
promoted_to: fix-board-height-under-banners
---

# Discussion: 技能提示出現時看板被壓縮、底部被裁切

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

使用者回報：版本橫幅與技能檔提示同時出現時，看板被壓縮、底部內容看不到（截圖：欄位底部被切掉）。要求明確（裁切 bug、可驗證），未經 grill 階段直接列假設。
偵察：版本橫幅（UpdateBanner）在 h-screen flex 直欄內、shrink-0，不是元兇；技能檔提示（AssetUpdatePrompt）掛在 <main> 內，main 是一般區塊排版且 overflow-hidden，看板根節點 h-full 拿 main 的 100% 高度，被提示往下推的那截被裁掉。規格頁、手冊頁、已封存頁根節點同為 h-full，同樣受影響。
相關規格：desktop-app「指令檔過期提示」（分頁內容頂部、per 專案、非阻斷）、「指令檔過期提示捲動釘選」、「桌面自動更新」；正典未承諾提示存在時內容區的高度。相關變更：無。
Prior discussions: update-check-on-focus-and-cli, release-changelog-whats-new（皆只談橫幅何時出現與日誌不用橫幅，未碰版面）

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-09-16)

**Focus**: 看板底部被裁切的根因是哪一層，以及修法要不要動提示位置或合併兩條橫幅
**Position**: 純排版 bug，修在 <main> 一處，四個視圖都受惠；提示位置與版本橫幅維持現狀。
- 根因：`apps/desktop/src/App.tsx:687` 的 <main> 是一般區塊排版（非 flex 直欄）且 overflow-hidden；AssetUpdatePrompt 佔掉頂部後，KanbanBoard 根節點（`packages/ui/src/components/KanbanBoard.tsx:374`）仍以 h-full 拿 main 的 100% 高度，多出來的那截被裁掉
- 版本橫幅（`apps/desktop/src/components/UpdateBanner.tsx:30`）在 h-screen flex 直欄內 shrink-0，main 隨之縮小，無 bug；它只是讓畫面更矮、問題更明顯
- 規格頁（`SpecList.tsx:159`）、手冊頁（`ManualPage.tsx:134`）、已封存頁（`ArchivedList.tsx:307`）根節點同為 h-full，同一修法一併修好
- 各欄自己的捲軸（`KanbanBoard.tsx:152` 的 flex-1 min-h-0 overflow-y-auto）不動；修後欄高剛好等於 main 扣掉提示的剩餘高度，捲到底能看到最後一張卡
- 使用者確認四條假設全對：只修裁切
**Ruled out**: 把技能提示搬到 header 上方與版本橫幅並排——違反規格「分頁內容頂部、per 專案」語意，把一行修正變成設計決策；兩條橫幅同時出現時合併或縮小——使用者明示不要，畫面變矮是合理佈局不是裁切
**Open**: 無

## Conclusion

**Decision**: 只修裁切。把 `apps/desktop/src/App.tsx` 的 <main> 改成 flex 直欄：技能檔提示 shrink-0，下方內容區（看板／規格頁／手冊頁／已封存頁）改為填滿剩餘高度（flex-1 min-h-0）而不是 h-full。提示的位置、per 專案語意、捲動釘選與版本橫幅全部維持現狀。canon 補一個 scenario 到 desktop-app「指令檔過期提示」：提示存在時，看板每一欄仍完整可見、欄內捲軸捲到底能看到最後一張卡。例：技能提示與版本橫幅同時出現、提案中欄有 3 張卡，最後一張卡「add-change-plan-remote」的進度列在欄內捲到底時完整可見，不被視窗底緣切掉。
**Rationale**: 根因是 main 不是 flex 直欄、內容區 h-full 拿了整個 main 高度；修在容器一處，四個視圖都受惠，元件內各自的捲軸不動。版本橫幅在正確的 flex 層，無需改。
**Rejected alternatives**: 把技能提示搬到 header 上方與版本橫幅並排——違反規格「分頁內容頂部、per 專案、非阻斷」，一行修正變設計決策；兩條橫幅同時出現時合併或縮小——使用者明示不要，畫面變矮是合理佈局不是裁切；只修 KanbanBoard 根節點——規格頁、手冊頁、已封存頁同樣受影響，會漏。
**Deferred**: none
**Capture to**: spec（desktop-app 補 scenario）＋ tasks
**Next**: /speclink-propose --from-discussion board-height-under-banners

---
topic: detail 抽屜會疊加——同時應只能開著一個抽屜
slug: drawer-exclusivity
status: promoted
promoted_to: drawer-exclusivity
created: 2026-07-17
created_by: MomoChen <momochenisme@gmail.com>
---

# Discussion: detail 抽屜會疊加——同時應只能開著一個抽屜

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

使用者回報：從 tray 開啟討論（或已轉出的討論）抽屜後，再開啟變更的詳情抽屜，兩個抽屜疊加顯示；期望同時只能開著一個抽屜。模式選 assumptions——codebase scout 找到 store.ts、App.tsx、tray.ts 與四個 Drawer 元件（RichDetailDrawer、SpecDrawer、ArchivedDrawer、DiscussionDrawer），證據充分。相關規格：desktop-app（抽屜行為 requirements 所在）、tray-status-menu（tray 開啟入口）。介面深度檢查不觸發（純 UI 狀態不變量，無新模組/IPC/跨層流）。

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-07-17)

**Focus**: 抽屜疊加的根源在哪、修在哪一層才不會漏入口
**Position**: 疊加根源在 store 的四個獨立 detail 欄位，修法是把「同時只開一個抽屜」下沉為 store 層不變量——四項假設全數獲使用者確認：
- 根源：store.ts:335-371 的 openDetail/openDiscussion/openSpec/openArchived 各自只設自己的欄位（detailChange/detailDiscussion/detailSpec/detailArchived），從不清另外三個；App.tsx:420-488 四抽屜平行掛載，任兩者皆可疊加
- 既有補丁證明互斥本是預期語意：App.tsx:446-449、484-487 抽屜內跳轉已手動「先關再開」，但 tray 路徑（tray.ts:308、318、385-386）直呼 open* 繞過了
- 修法：每個 open* 動作同時清掉另外三個 detail 欄位，後開者取代先開者（不是拒開）
- 範圍：全部四種抽屜全域互斥，不只截圖中的「討論＋變更」組合
- openDetail 既有的 drawerVerb: null 清理（store.ts:337-338）保留不動，互斥清理疊加上去
- App.tsx 兩處手動先關再開順勢移除（不變量生效後成冗餘，留著會誤導新入口照抄）
**Ruled out**: 逐入口補 close（per-callsite 補丁已被證明會漏——現況正是漏了 tray，且未來新入口再漏）；只修討論＋變更組合（其他抽屜對同病因會再報一次）
**Open**: 無——直接收斂

## Conclusion

**Decision**: 把「同時只能開著一個 detail 抽屜」下沉為 store 層不變量——四個 open* 動作（openDetail/openDiscussion/openSpec/openArchived）各自在設定自己的欄位時同時清掉另外三個 detail 欄位，後開者取代先開者；openDetail 既有的 drawerVerb: null 清理保留；App.tsx:446-449、484-487 兩處手動「先關再開」順勢移除。
**Rationale**: 互斥是全域語意，該由狀態層一處保證。per-callsite 補丁已被現況證偽——抽屜內跳轉補了、tray 入口漏了；下沉到 store 後所有現有與未來入口（看板、tray 選單、tray 面板、抽屜內跳轉）自動正確。
**Rejected alternatives**: 逐入口補 close——會漏且不可維護，現況正是這種做法漏了 tray；只修「討論＋變更」組合——規格/封存抽屜病因相同，之後會再報一次。
**Deferred**: none
**Capture to**: proposal（spec delta 落在 desktop-app：抽屜互斥 requirement——任一 detail 抽屜開啟時開啟另一種抽屜，先開者關閉、同時僅一個抽屜可見）
**Next**: /speclink-propose --from-discussion drawer-exclusivity

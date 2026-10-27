---
topic: 看板「討論」欄如何分離已轉出討論以降低視覺複雜度
slug: promoted-discussion-board-ux
status: promoted
promoted_to: promoted-discussion-toggle
created: 2026-07-09
---

# Discussion: 看板「討論」欄如何分離已轉出討論以降低視覺複雜度

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

桌面看板「討論」欄目前把已轉出（promoted）討論以**預設展開**的欄底群組「已轉出變更的討論 (N)」呈現（Image #4），每列為 topic＋衍生變更樹，chip 用**統一灰底**（`DiscussionColumn.tsx:159` bg-muted）不分狀態。使用者目標：把「討論中」（open/concluded）與「已轉換成 changes」（promoted）的討論**分開顯示、降低視覺複雜度**；提議 (a) 把收合 toggle 移到泳道 header、(b) chip 用顏色區分提案中／已封存等狀態。

模式：assumptions（掃到 DiscussionColumn.tsx、KanbanBoard.tsx、stage.ts、i18n.tsx）。

配色語言盤點：看板全用 **teal 單色濃度階梯**（KanbanBoard.tsx:29 STAGE_STYLE：proposed primary/8 → in-progress primary/12 → ready primary），無任何多色語意輪；討論 badge（STATUS_BADGE）亦然。chip 狀態實為 5 種：proposed/in-progress/ready（changeStage 派生）＋已封存＋已刪除（discussionChipStage）。現行 showPromoted 預設 true（DiscussionColumn.tsx:186）；欄計數用 discussions.length 含 promoted（:221）。無新 IPC/儲存縫——純 packages/ui 元件的狀態與 className。

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-07-09)

**Focus**: 如何分離 active／promoted 討論、chip 要不要上色與上什麼色
**Position**: 以「降低 promoted 存在感」為主軸，四項——
- chip 上色沿用看板階段配色（chip 色＝該變更所在欄位的 teal 濃度），已封存維持中性灰、已刪除淡 destructive／刪除線；不引入紅綠琥珀新色輪（破壞看板單色克制）。
- chip 實為 5 態：proposed/in-progress/ready＋已封存＋已刪除，非 3 態。
- 真正問題是 showPromoted 預設 true（DiscussionColumn.tsx:186），最便宜高價值修正＝**預設收合**，收合後欄底僅一行。
- 欄計數用 discussions.length 含 promoted（:221），與「降級 promoted」矛盾 → 計數改只算 active。
- toggle 移 header 較不易發現、欄寬 250–360px 空間緊；欄底自標籤收合列（「已轉出變更的討論 (2)」）更好按、更自解釋，傾向不移。
**Ruled out**: 引入紅／綠／琥珀新色輪（破壞看板 teal 單色語言）；維持 showPromoted 預設展開
**Open**: 使用者選「預設收合欄底群組」還是「header 篩選器隱藏整組」；chip 是否採階段色方案（待 ASCII 對稿確認）

### Round 2 — assumptions (2026-07-09)

**Focus**: A（欄底收合列）vs B（header 開關）定案
**Position**: 走方案 B——header 開關，關閉時 promoted 零佔位，達成最徹底的分離：
- header 控制沿「轉出」語意 icon（ArrowUpRight ↗，與卡片「轉為變更」動詞同 icon）＋已轉出數字；0 個已轉出討論時整個開關不顯示。
- 預設關閉（隱藏）；開關狀態元件內 local（不跨 session 持久，比照現行 showPromoted）。
- 開啟時 promoted 於欄體底部呈現，上方細分隔＋輕標籤維持與「討論中」卡的分層；關閉時整組（含標籤）消失。
- 欄計數改只算 active（open＋concluded）；promoted 數量改由 header 開關上的數字承載。
- chip 沿看板階段色（提案中 primary/8、進行中 primary/12、已就緒 primary）、已封存中性灰、已刪除淡 destructive＋刪除線。
- 邊角：當 0 active 但有 promoted（使用者現況），欄體空狀態勿顯「尚無討論」誤導——留白或淡提示，由 header ↗N 表達「有 N 個已轉出討論被收起」。
**Ruled out**: 方案 A（欄底收合列）——使用者要更徹底分離、關閉時零佔位，A 仍留一行
**Open**: chip 階段色最終確認；header 開關的 icon/label 呈現；0-active＋有-promoted 空狀態文案

## Conclusion

**Decision**: 看板「討論」欄改採方案 B 分離已轉出討論——
- header 加「顯示已轉出」開關：icon `↗`（ArrowUpRight，呼應「轉為變更」動詞）＋已轉出數字，形如 `↗ 2`；0 個已轉出討論時整個開關不顯示。
- 預設關閉（隱藏）；開關狀態元件內 local，不跨 session 持久。
- 開啟時 promoted 群組於欄體底部呈現（細分隔＋輕標籤，維持與「討論中」卡的分層）；關閉時整組零佔位。
- 欄計數改只算 active（open＋concluded）；已轉出數量改由開關上的 `↗ N` 承載。
- chip 沿看板階段色：提案中 primary/8、進行中 primary/12、已就緒 primary、已封存中性灰、已刪除淡 destructive＋刪除線（chip 色＝該變更現處欄位色）。
- 空狀態邊角：0 active 但有 promoted 時欄體留白，不顯「尚無討論」，由 header `↗ N` 表達有 N 個已轉出討論收起。
**Rationale**: 使用者目標是把「討論中」與「已轉換成 changes」的討論分離、降低視覺複雜度；方案 B 關閉時 promoted 零佔位，分離最徹底。chip 沿階段色＝變更現處欄位色，掃一眼知進度且不引入看板沒有的顏色，維持既有 teal 單色濃度階梯的克制。
**Rejected alternatives**: 方案 A（欄底收合列、預設收合）——仍留一行，分離不如 B 徹底；引入紅／綠／琥珀多色語意輪——破壞看板 teal 單色語言；欄計數維持含 promoted——與降級 promoted 的意圖矛盾。
**Deferred**: none（實作細節如開關 hover tooltip 文案於 propose 決定）
**Capture to**: proposal（新變更：DiscussionColumn header 開關＋計數只算 active＋chip 階段色＋空狀態；i18n 補開關 aria/tooltip 字串）
**Next**: /speclink-propose --from-discussion promoted-discussion-board-ux

---
topic: 討論詳情頁：步驟列與狀態籤顏色對不上；結論裡還沒轉出的刀要更明顯並加提示
slug: discussion-stage-and-unpromoted-cuts
status: concluded
created: 2026-10-13
created_by: MomoChen <momochenisme@gmail.com>
---

# Discussion: 討論詳情頁：步驟列與狀態籤顏色對不上；結論裡還沒轉出的刀要更明顯並加提示

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

使用者看 wadpilot 的討論詳情頁截圖提出兩項：一、左欄身分區狀態籤「已轉出・保留中」（琥珀）與「狀態」區步驟列的現站「已轉出」（主色綠）看起來對不上；二、結論「分刀」區段裡還沒轉出的刀沒有任何標示，希望有特別的顏色或標示，並比照任務文件的手動任務提示框在上方加提示。兩項目標明確，不需要 grill 階段，直接以假設清單推進。相關正式規格為 desktop-app（「討論詳情頁檢視與轉出變更」的狀態籤與階段步驟列、「討論結論以欄位標籤呈現」的分刀卡與「結論文件 SHALL NOT 有提示框」、「討論於看板第 0 欄兩級呈現」的狀態籤判準）；剛建立的 doc-tiles-fill-and-round-order 改 DetailShell 與 RoundsDoc，與本題檔案不重疊；在途變更 tray-panel-content-priority: desktop-app 改「討論於看板第 0 欄兩級呈現」，clickable-cursor-hover-rule: desktop-app 只新增可點元素游標需求，皆不改本題的兩條需求。
Prior discussions: detail-content-redesign, multi-cut-discussion-hold, promoted-discussion-board-ux, discussion-auto-archive-before-conclusion

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-10-13)

**Focus**: 兩處狀態為什麼對不上、還沒轉出的刀怎麼標示、交付方式
**Position**: 使用者接受第 1 項與交付方式的假設，否決第 2 項的灰色虛線標示（不夠明顯），要比照任務文件的手動任務提示框在結論上方加提示。
- 文字不同屬設計：狀態籤＝狀態加提醒（已轉出・保留中），步驟列＝走到哪一站（已轉出），其下「保留在途」列解釋（desktop-app「討論詳情頁檢視與轉出變更」scenario 明寫）
- 真的對不上一：步驟列現站固定主色（`DiscussionStageRow` 寫死 `tone="primary"`），狀態籤保留中與尚無結論時為琥珀；改為現站共用 `statusPill()` 的色調，現站文字不變
- 真的對不上二：promoted 即 `stationIndex` 2，尚無結論的已轉出討論也把「已結論」畫成打勾；改為該站畫灰圈（未到站）
- 判斷刀是否轉出只比對變更名是否在 `promoted_to`（`rowStage` 查不到即未轉出），不推算還欠幾刀；同刀多變更時逐顆標示；刀列沒寫變更名時不標
- 「衍生變更」分頁不改（`PromotedDoc` 略過沒有成員的刀）
- 另開一個小變更，不併進 doc-tiles-fill-and-round-order（檔案不重疊、名稱不準）
**Ruled out**: 狀態籤改回主色綠以配合步驟列——看板討論卡用同一套琥珀判準（規格明文），改了與看板不一致；未轉出刀只用灰色籤加虛線外框——使用者認為不夠明顯；未轉出刀用琥珀色——琥珀在本 app 代表需要注意，保留中的系列有刀未轉出是正常狀態
**Open**: 未轉出刀的醒目畫法與結論上方提示框的內容（與正典「結論文件 SHALL NOT 有提示框」衝突）；已封存討論頁是否也出提示框

### Round 2 — interview (2026-10-13)

**Focus**: 未轉出刀的醒目畫法與結論上方提示框
**Position**: 使用者接受提示框與刀卡的畫法，顏色由藍色改為橄欖色（手動任務色 `--status-manual`）。
- 結論文件數字磚之下加提示框「N 刀尚未轉出」：每刀一列（刀編號籤、範圍句、「前往 →」捲到該刀卡），末行附轉出下一刀的指令；版型比照任務文件的手動任務提示框（`changeDocSummary.tsx` 的 manualCallout）
- 整刀未轉出的刀卡改橄欖色虛線外框＋淡橄欖底，刀編號旁加「尚未轉出」籤；同刀部分轉出時只在未轉出的變更名旁加籤
- 橄欖色語意吻合：手動任務色代表「要由你親自動手」，轉出下一刀同樣是使用者發起的動作
- 已封存討論頁不出提示框、只保留刀卡標示：封存記錄不能再轉出，指令按了無效
- 偏離正典「結論文件 SHALL NOT 有提示框」與 detail-content-redesign 的「提示框只用七處」：當時判準是「只放推得出的事實」，未轉出刀由結論分刀列比對 `promoted_to` 直接推出，與衍生變更頁「只差最後一刀」同類；七處是當時的數量，不是上限
**Ruled out**: 藍色（note 色）——使用者選橄欖色；已封存討論頁也出提示框——封存記錄不能轉出，指令無效
**Open**: 無——寫結論

## Conclusion

**Decision**: 一個小變更修改 desktop-app 兩條需求：討論詳情頁的階段步驟列與身分區狀態籤同色同義，結論文件以橄欖色提示框與刀卡標示還沒轉出的刀。
- 步驟列現站的色調共用身分區狀態籤的判準：已轉出・保留中與已轉出・尚無結論為琥珀，其餘維持主色；現站文字不變（「保留在途」列照舊解釋保留中）
- 已轉出但尚無結論的討論，步驟列的「已結論」站畫成未到站的灰圈，不打勾
- 刀是否轉出只比對刀列的變更名是否在 `promoted_to`；刀列沒寫變更名時不標示；不推算還欠幾刀
- 結論文件數字磚之下加橄欖色（手動任務色）提示框「N 刀尚未轉出」：每刀一列（刀編號籤、範圍句、「前往 →」捲到該刀卡），末行說明這幾刀還沒有變更並附可複製的轉出指令 `/speclink-propose --from-discussion <slug>`；版型比照任務文件的手動任務提示框
- 整刀未轉出的刀卡改為橄欖色虛線外框＋淡橄欖底，刀編號旁加「尚未轉出」籤；同刀部分轉出時只在未轉出的變更名旁加籤
- 「尚未轉出」籤以 StatusPill 新增的 manual 色調呈現，不以 className 覆蓋既有色調
- 已封存討論頁不出提示框，刀卡標示照常
- 例：wad-plugins-merge-or-split 結論有 cut 1、cut 2，`promoted_to` 只有 cut 1 的 add-team-plugin-library → 提示框標題「1 刀尚未轉出」、列出 cut 2 與「前往 →」；cut 2 卡為橄欖色虛線框、帶「尚未轉出」籤；cut 1 卡照舊實線框、變更名旁為「進行中」階段籤
- 規格：desktop-app「討論詳情頁檢視與轉出變更」補步驟列現站色調與尚無結論時的「已結論」站；「討論結論以欄位標籤呈現」移除「結論文件 SHALL NOT 有提示框」、加入未轉出提示框與刀卡標示，並改寫「分刀卡與變更籤」scenario 的第三張卡
- 「衍生變更」分頁不改；與 doc-tiles-fill-and-round-order 分開成另一個變更
**Rationale**: 狀態籤與步驟列講的是同一個狀態，顏色不同會讓人以為是兩件事，「已結論 ✓」更與「尚無結論」直接矛盾；未轉出的刀是使用者接下來要親自發起的工作，用手動任務的橄欖色與同款提示框，讀者一看就知道「這裡等你動手」，且資料全由結論分刀列比對 `promoted_to` 推出，不需新增任何格式。
**Rejected alternatives**:
- 狀態籤改回主色綠以配合步驟列 — 看板討論卡用同一套琥珀判準（規格明文），改了與看板不一致
- 未轉出刀只用灰色籤加虛線外框 — 使用者認為不夠明顯
- 未轉出刀用琥珀色 — 琥珀在本 app 代表需要注意，保留中的系列有刀未轉出是正常狀態
- 未轉出刀與提示框用藍色（note 色） — 使用者選橄欖色，且手動任務色的「要你親自動手」語意更貼切
- 已封存討論頁也出提示框 — 封存記錄不能再轉出，指令按了無效
- 併進 doc-tiles-fill-and-round-order — 檔案不重疊、變更名不準確
**Deferred**: none
**Capture to**: spec
**Next**: /speclink-propose --from-discussion discussion-stage-and-unpromoted-cuts

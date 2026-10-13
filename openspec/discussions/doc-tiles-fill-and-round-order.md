---
topic: 數字磚依格數平分整列；討論過程時間軸改成新的輪在上（只改 desktop 顯示）
slug: doc-tiles-fill-and-round-order
status: promoted
created: 2026-10-13
created_by: MomoChen <momochenisme@gmail.com>
promoted_to: doc-tiles-fill-and-round-order
---

# Discussion: 數字磚依格數平分整列；討論過程時間軸改成新的輪在上（只改 desktop 顯示）

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

使用者看截圖提出兩項桌面顯示調整：一、閱讀卡標題下的數字磚固定排一列四欄，只有三格時第四欄空著，希望依格數平分、填滿整列；二、討論過程時間軸由舊到新排列，希望最新的輪在最上面——只改 desktop 顯示，討論記錄的 markdown 不動。兩項需求都有可檢查的目標與範圍，不需要 grill 階段，直接以假設清單推進。相關正式規格為 desktop-app（「詳情頁的文件樹、閱讀卡與本頁目錄」的數字磚列容器、「討論輪以卡片呈現」、工單文件的輪次順序）；在途變更 clickable-cursor-hover-rule: desktop-app、tray-panel-content-priority: desktop-app 改的是其他需求（可點元素的游標與 hover 回饋、討論於看板第 0 欄兩級呈現），與本討論不重疊。
Prior discussions: detail-content-redesign

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-10-13)

**Focus**: 兩項顯示調整各怎麼做、範圍到哪裡、規格要不要補字
**Position**: 使用者確認六條假設全對：兩項都只改共用元件、補規格字句釘住，合成一個小變更。
- 數字磚：共用 `DocTiles`（packages/ui/src/components/DetailShell.tsx:222 寫死 grid-cols-4）改成「有幾格分幾欄、各格等寬填滿整列」；變更、已封存、討論、規格、手冊五種頁面一起生效
- 數字磚只有 1 格時也填滿整列（例：背景文件沒有關聯時只剩「先前討論 0」）
- 正式規格沒寫數字磚寬度（desktop-app「詳情頁的文件樹、閱讀卡與本頁目錄」只寫 0 到 4 格；detail-content-redesign 只定「缺資料就少一格」），補一句等寬填滿
- 討論過程：`RoundsDoc`（RoundsDoc.tsx:39 照檔案順序畫卡）改成依輪次由新到舊；討論詳情頁與已封存討論頁共用、一起改；右欄本頁目錄照畫面標題順序產生（DetailShell.tsx:44），自動跟著倒序；markdown 不動
- 預設展開維持第一輪與最後一輪（RoundsDoc.tsx:40、規格「第一輪與最後一輪 SHALL 預設展開」），倒序後即最上（最新）與最下（Round 1）展開
- 「討論輪以卡片呈現」只寫「逐輪呈現」沒寫方向，補「依輪次由新到舊」避免被改回
- 審查與驗證工單的輪次維持升序（規格「輪次依序號升序」、TicketView.tsx:171），不在本次範圍
**Ruled out**: 最少分 2 欄讓單格只佔半列——使用者要依格數填滿，單格長磚可接受；只改提案頁與任務頁——共用元件改一處即全頁一致，加開關是多餘彈性；已封存討論維持舊順序——兩頁共用元件，分開要多一個參數且兩頁閱讀方向不一致；倒序後只展開最新一輪——沿用既有首末輪展開，不改沒被要求的行為；工單輪次一併倒序——使用者只要求討論過程；拆成兩個變更——兩處都是 desktop-app 的小改動，一刀即可
**Open**: 無——寫結論

## Conclusion

**Decision**: 一個小變更修改 desktop-app 兩條需求：數字磚依格數平分整列、討論過程時間軸由新到舊排列，都只改畫面、不動任何 markdown。
- 數字磚：共用 `DocTiles` 由固定一列四欄改為「有幾格分幾欄、各格等寬填滿整列」，變更、已封存、討論、規格、手冊五種頁面一起生效
- 數字磚只有 1 格時同樣填滿整列
- 例：提案出 3 格時三格各佔 1/3 列，不留空欄；4 格時與現況相同
- 規格補字：desktop-app「詳情頁的文件樹、閱讀卡與本頁目錄」的數字磚列容器補「各格等寬、填滿整列」
- 討論過程：`RoundsDoc` 依輪次由新到舊排列；討論詳情頁與已封存討論頁一起改；右欄本頁目錄照畫面順序自動跟著倒序
- 預設展開維持第一輪與最後一輪，倒序後即最上（最新）與最下（Round 1）展開，其餘收合
- 例：9 輪的討論由上而下為 Round 9、Round 8 … Round 1，Round 9 與 Round 1 展開
- 規格補字：desktop-app「討論輪以卡片呈現」補「依輪次由新到舊排列」，並調整相關 scenario
- 審查與驗證工單的輪次維持由舊到新，不在範圍內
**Rationale**: 兩項都是單一共用元件的排版改動，改一處就讓所有用到的頁面一致；正式規格原本沒寫數字磚寬度與輪的排列方向，補字是為了讓測試釘住新行為、避免日後被改回。
**Rejected alternatives**:
- 數字磚最少分 2 欄讓單格只佔半列 — 使用者要依格數填滿，單格長磚可接受
- 只改提案頁與任務頁 — 共用元件改一處即全頁一致，加開關是多餘彈性
- 已封存討論維持舊順序 — 兩頁共用元件，分開要多一個參數且兩頁閱讀方向不一致
- 倒序後只展開最新一輪 — 沿用既有首末輪展開，不改沒被要求的行為
- 工單輪次一併倒序 — 使用者只要求討論過程
- 拆成兩個變更 — 兩處都是 desktop-app 的小改動，一刀即可
**Deferred**: none
**Capture to**: spec
**Next**: /speclink-propose --from-discussion doc-tiles-fill-and-round-order

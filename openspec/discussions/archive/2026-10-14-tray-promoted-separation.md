---
topic: 系統匣面板：已轉出討論與上方討論中的「還有 N 個…」收合列黏在一起，底色區分不明顯——怎麼調整設計
slug: tray-promoted-separation
status: promoted
created: 2026-10-14
created_by: MomoChen <momochenisme@gmail.com>
promoted_to: tray-promoted-separation
---

# Discussion: 系統匣面板：已轉出討論與上方討論中的「還有 N 個…」收合列黏在一起，底色區分不明顯——怎麼調整設計

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

使用者附截圖指出：系統匣面板討論卡展開「已轉出」後，已轉出列緊接在討論中清單的「還有 6 個…」溢出列下方，只靠 bg-muted/40 底色區分，看起來黏成一團。需求的目標已明確（讓已轉出與討論中一眼分開），不需 grill stage，直接以假設推進。相關規格為 tray-status-menu（「討論列表」「面板內容卡收合」「面板樣式」「分區溢出摺疊」）與 desktop-app（「介面狀態語意色分層」）；進行中的變更 quality-gate-speedup、workflow-config-write-feedback、server-web-detail-pages 的 delta 都不涉及這兩個 capability。現行收法來自已封存變更 tray-panel-restyle（出自討論 desktop-ui-redesign）。
Prior discussions: tray-panel-content-priority, tray-empty-stage-sections, tray-panel-card-design, tray-panel-scroll-and-vibrancy, promoted-discussion-board-ux, tray-rich-panel, desktop-ux-polish

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-10-14)

**Focus**: 已轉出在面板上要怎麼跟討論中分開——留在討論卡內調樣式，還是拆成獨立卡
**Position**: 採方案 A：已轉出拆成討論卡正下方的獨立卡，使用者確認五條假設全對。
- 黏在一起的三個成因（TrayPanel.tsx:870-913）：討論中的「還有 N 個…」溢出列正好壓在已轉出列上方、讀起來像它們的標題；兩邊同用 DiscussionRow，只差外層 bg-muted/40，淺色主題下幾乎看不出；組名「已轉出 N」在這組列的下方，展開時先見列、後見名
- 面板唯一明顯的分界是卡片間距（panel-scroll-content 的 gap-2）；拆卡後溢出列成為討論卡最後一列，不再貼著別的列
- 收合改用既有 SectionCard 標頭機制（整條標頭可點、箭頭展開朝下收合朝右），拿掉特製收合列與「展開時箭頭朝上」特例；預設收合、沿用 speclink.tray.promotedExpanded 記憶鍵
- 標頭為灰色的討論同款圖示（MessageSquareText）＋「已轉出」＋中性灰計數徽章——desktop-app「介面狀態語意色分層」要求已轉出分區與計數為中性；不用 ArrowUpRight，頁尾「開啟 Speclink」已佔用（TrayPanel.tsx:933）
- 已轉出列沿用 DiscussionRow，拿掉 bg-muted/40 灰底（卡已分開，灰底只會像不能按）
- 範圍只有面板（macOS 與 Windows 共用呈現層）；看板與 Linux 原生選單不動；討論卡收合時已轉出卡仍可見；討論 0＋已轉出 N 時討論卡為計數 0 的空狀態卡
- 與正典衝突：tray-status-menu「討論列表」的「macOS 面板 SHALL NOT 另立『已轉出』分區」與上方展開、箭頭朝上的規定；推翻 tray-panel-restyle 的收法——其理由（舊獨立卡常駐展開、與看板收合＋記憶不一致）新卡仍滿足，與看板只差卡內卡外；看板分得清是因為已轉出為精簡細列、討論中為整張白卡，面板兩邊同一種列，照搬看板收法分不開
**Ruled out**: 方案 B 留在卡內、收合列移到這組列上方並改往下展開——分隔仍只靠卡內細線，效果中等，且展開方向與看板相反；方案 C 只把底色加深——「還有 N 個…」像標題的主因不變；標頭用 ArrowUpRight——與頁尾「開啟 Speclink」同一圖示兩種意思
**Open**: 無——收尾細節（零筆、載入中、溢出摺疊、i18n 詞條）在結論補齊

## Conclusion

**Decision**: 系統匣面板（macOS 與 Windows 共用呈現層）把「已轉出」從討論卡底部的收合列拆成討論卡正下方的獨立卡，靠卡片間距與討論中分開。
- 中段順序：提案中→進行中→已就緒→討論→已轉出；已轉出卡有料才現，零筆、載入中、載入失敗時都不呈現
- 收合：已轉出卡沿用其他內容卡的卡片標頭收合（整條標頭可點、箭頭展開朝下收合朝右）；拿掉特製收合列與「展開時箭頭朝上」特例；預設收合，記憶沿用 `speclink.tray.promotedExpanded`（使用者現有展開狀態不遺失），不併入 `speclink.tray.collapsedSections`
- 標頭：灰色的討論同款圖示（MessageSquareText）＋「已轉出」（沿用原生選單已有的 `tray.promotedHeader` 詞條，不新增 i18n）＋中性灰計數徽章；不用 ArrowUpRight
- 列：沿用 DiscussionRow 與五筆溢出摺疊（前 5 筆＋「還有 N 個…」），拿掉 `bg-muted/40` 灰底
- 討論卡：計數與空狀態只看討論中；討論 0＋已轉出 N 時為計數 0 的空狀態卡，下方接已轉出卡；「還有 N 個…」成為討論卡最後一列
- 兩卡各自收合：收起討論卡時已轉出卡照常顯示
- 不動：看板討論欄（含 `discussion.promotedBar` 與看板記憶鍵）、Linux 原生選單（本來就有獨立「已轉出」分區）、分流判準（promoted＋concluded＋hold）、面板高度上限與量高邏輯
- 規格：MODIFIED tray-status-menu「討論列表」（刪「SHALL NOT 另立『已轉出』分區」與上方展開、箭頭朝上的規定，改寫為獨立卡）、「面板內容卡收合」（收合的討論卡不再提已轉出收合列；已轉出卡預設收合、獨立記憶）、「面板樣式」（區塊順序與「分區卡與列的外觀」情境）；改名的 scenario 要補 REMOVED-SCENARIO 宣告
- 例：wadpilot 討論中 11、已轉出 3 → 「討論 11」卡列前 5 筆＋最後一列「還有 6 個…」；8px 空隙後是只有一行的「已轉出 3 ›」卡；點開後 3 列出現在標頭下方、箭頭朝下；關掉面板再開仍展開；收起討論卡後「已轉出 3」卡照常在
**Rationale**: 面板上唯一一眼分得清的界線是卡與卡之間的間距；卡內只靠底色區分，在淺色主題下幾乎看不出，加上「還有 N 個…」正好壓在已轉出列上方像它們的標題，換色解決不了。tray-panel-restyle 把已轉出收進討論卡的理由是「舊獨立卡常駐展開、與看板的收合＋記憶不一致」，新卡預設收合並記憶，那條理由仍然滿足；看板分得清是因為已轉出為精簡細列、討論中為整張白卡，面板兩邊用同一種列，照搬看板收法才分不開。拆卡同時把兩套收合機制合成一套。
**Rejected alternatives**:
- 留在討論卡內、收合列移到這組列上方改往下展開 — 分隔仍只靠卡內細線，效果中等，且展開方向與看板相反
- 只把已轉出底色加深 — 「還有 N 個…」像標題的主因不變
- 標頭用 ArrowUpRight — 頁尾「開啟 Speclink」已用同一圖示，一個面板裡一圖兩義
- 已轉出改記在 collapsedSections — 該記憶預設展開，首次開面板已轉出會整組攤開，與現行預設收合相反
**Deferred**: none
**Capture to**: spec | design
**Next**: /speclink-propose --from-discussion tray-promoted-separation

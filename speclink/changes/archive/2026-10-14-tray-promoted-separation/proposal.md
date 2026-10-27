## Why

系統匣面板（macOS 與 Windows）把已轉出討論收在「討論」卡底部的「已轉出 N ›」收合列，展開後已轉出列直接接在討論中清單的「還有 N 個…」溢出列下方，只靠外層 `bg-muted/40` 底色區分——淺色主題下幾乎看不出，溢出列又正好壓在已轉出列上方、讀起來像它們的標題，組名「已轉出 N」則落在這組列的下方。使用者一眼分不出哪幾筆還在討論、哪幾筆已轉出。面板上唯一一眼分得清的界線是卡與卡之間的間距，所以把已轉出拆成自己的卡（討論 tray-promoted-separation 的結論）。

## What Changes

- 「已轉出」從討論卡底部的收合列改為討論卡正下方的獨立卡；中段順序變成提案中→進行中→已就緒→討論→已轉出。已轉出卡有料才現：零筆、載入中、載入失敗時都不呈現。
- 已轉出卡的收合改用其他內容卡同一套卡片標頭（整條標頭可點、箭頭展開朝下收合朝右、`aria-expanded`、不進 Tab 順序）；拿掉特製的收合列與「展開時箭頭朝上」特例。預設收合，展開狀態沿用既有記憶鍵 `speclink.tray.promotedExpanded`，使用者現有的展開狀態不遺失；不併入 `speclink.tray.collapsedSections`。
- 已轉出卡標頭：灰色的討論同款圖示（MessageSquareText）＋「已轉出」（沿用原生選單已有的 `tray.promotedHeader` 詞條）＋中性灰計數徽章；不用 ArrowUpRight（頁尾「開啟 Speclink」已用同一圖示）。
- 已轉出列沿用討論列（slug 為題、topic 為描述、列尾複製鈕）與五筆溢出摺疊，拿掉 `bg-muted/40` 灰底。
- 討論卡的計數與空狀態只看討論中：討論 0、已轉出 N 時討論卡為計數 0 的空狀態卡，下方接已轉出卡；「還有 N 個…」成為討論卡的最後一列。
- 兩張卡各自收合：收起討論卡時已轉出卡照常顯示。
- 不變：分流判準（promoted、concluded、hold）、看板討論欄（含 `discussion.promotedBar` 與看板記憶鍵）、Linux 與 Windows 右鍵的原生選單、面板高度上限與量高邏輯、溢出門檻。

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `tray-status-menu`: 「討論列表」的面板已轉出由討論卡底部收合列改為獨立卡（刪「SHALL NOT 另立『已轉出』分區」與上方展開、箭頭朝上的規定）；「面板內容卡收合」把已轉出卡納入可收合的內容卡，並寫明它預設收合、獨立記憶，收合的討論卡不再牽涉已轉出；「面板樣式」的區塊順序、同源描述與有料才現的規定改指已轉出卡。看板的 `desktop-app` 規格不受影響：「介面狀態語意色分層」已要求「已轉出」分區與計數為中性，新卡照此上色。

## Impact

- 目標使用者與情境：用 desktop 系統匣面板追蹤討論與變更的開發者／PO／PM，對應 discuss 轉出變更之後、在面板上一眼分辨「還在討論」與「已轉出」的時刻。
- Affected specs: `tray-status-menu`
- 影響的 app：`apps/desktop` 的系統匣面板前端；`packages/ui`、speclink-core、speclink-cli、speclink-host 與 server 不動。
- 相容性影響：沒有 CLI 指令、`--json` 輸出、設定欄位或技能的變更。localStorage 鍵與值格式不變（`speclink.tray.promotedExpanded` 照舊是 "true"／"false"），升級後展開狀態原樣保留。DOM 錨點：恢復 `panel-section-promoted`，移除 `panel-promoted-toggle` 與 `panel-promoted-rows`。
- Affected code:
  - New: (none)
  - Modified: apps/desktop/src/panel/TrayPanel.tsx, apps/desktop/src/__tests__/trayPanel.test.tsx
  - Removed: (none)

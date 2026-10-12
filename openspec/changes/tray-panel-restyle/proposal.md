## Summary

把 macOS 系統匣面板換成與主視窗同一套容器語言：側欄淡灰補光底、白底細框 12px 圓角卡（專案 tab 條、各分區、動作區各一張）、細線分隔的列與灰底 hover、「已轉出」收進討論卡底部的收合列並記憶展開狀態、專案 tab 的首字母方塊與主視窗圖示列同形；原生選單（非 macOS）與 Rust 側面板視窗不動。

## Motivation

- 討論 desktop-ui-redesign 已把看板、詳情頁、設定頁全部改成「容器＝白底細框卡、項目＝卡內細線列或小卡」的規則；面板目前仍是毛玻璃上的半透明灰卡、主色實心反白的 hover 列、三條分割線與獨立的「已轉出」分區，是桌面 app 最後一塊舊語言（討論結論 cut 5）。
- 面板的專案 tab 與主視窗圖示列的專案方塊代表同一個專案，形狀卻不同（面板是 24px 小方塊疊在實心主色卡上）；討論裁定兩者同形。
- 看板討論欄底的「已轉出 N ›」已改為收合列並記憶展開狀態（desktop-board-reskin），面板仍把已轉出列成常駐分區，兩處對同一資料的呈現不一致。
- 目標使用者：在 macOS 以系統匣一瞥專案進度、不想切回主視窗的開發者；對應 apps/desktop 的 panel 入口（`apps/desktop/src/panel/TrayPanel.tsx`），不涉及 CLI 指令、設定欄位與技能檔。

## Proposed Solution

- 面板底改為側欄面色（`--sidebar`）約九成不透明的補光層，拿掉主色漸層 wash；HudWindow 毛玻璃材質與 13px 圓角維持，由餘下透明度透出。
- 專案 tab 條、每個分區（討論、提案中、進行中、已就緒）、動作區塊各自是一張白底（`--card`）細框 12px 圓角卡；卡間以間距分隔，三條分割線移除。
- 分區卡的標頭為 12px 粗體＋色相圖示＋計數徽章，標頭下一條細線；列以細線分隔、整列 hover 灰底（半透明前景色），不再以主色實心反白；站章、波次數字、複製鈕在 hover 期間維持本色。
- 專案 tab 的首字母方塊沿用圖示列同一組尺寸與配色（作用中實心主色白字、其餘灰底），尾端「加入專案」為虛線方塊；方塊樣式抽成兩邊共用的常數模組。
- 「已轉出」不再是獨立分區：收進討論卡底部的「已轉出 N ›」收合列，預設收合，點按就地在列上方展開已轉出討論列（同樣五筆折疊）；展開狀態存 app 本機（與看板的記憶鍵分開），跨面板開閉與 app 重啟保留；零筆時無此列。原生選單維持獨立「已轉出」分區。
- 復原卡與 stale 狀態列改白底卡＋語意色邊框，動作鈕改用共用 Button 變體。
- tray-status-menu 規格五條需求 MODIFIED：面板樣式（macOS）、討論列表、面板變更列的品質站章、變更列的波次標示與順序同源，以及「面板交接主視窗後滑鼠互動完整」的「抽屜」字面對齊為詳情頁（純字面，無程式碼對應）。

## Non-Goals

- 原生選單（非 macOS 平台，及 macOS 面板建立失敗後備）的結構、順序與文案不變；`apps/desktop/src/tray.ts` 不動。
- Rust 側面板視窗（HudWindow 材質、13px 圓角、nonactivating、失焦收合、positioner 貼齊）不動；補光只在 CSS 層。
- 面板資料流不變：TraySnapshot 仍由主視窗 store 推送，不新建查詢路徑；三段式版面、高度自適應、捲動指示條、溢出門檻 5、skeleton 與載入失敗終態、復原卡動作集合皆維持。
- 看板討論欄的收合列與其記憶鍵不動；面板記憶另存一鍵，兩處各自記。
- 面板不放品牌標記；不加新的 i18n 詞條（收合列文案沿用共用元件庫的「已轉出 {n}」）。

## Alternatives Considered

- 維持毛玻璃半透明卡、只對齊 hover 與圓角——討論早輪的假設，後被「層次比照看板」的裁定取代；半透明卡在淺色底上分不出容器與項目兩層。
- 面板與看板共用同一個「已轉出」記憶鍵——兩個視窗各自讀寫 localStorage、看板的值走 store 同步；共用一鍵會讓面板的點按改動看板的收合狀態，使用者在兩處看到互相干擾的結果。
- 面板分區直接複用看板的 BoardColumn 元件——欄有 3px 色相頂條、卡片區鋪灰與欄內捲動，面板分區是不捲動的列式容器；複用會帶進用不到的結構。
- hover 維持主色實心反白、只換圓角——與「hover＝半透明黑、teal 只給選取與主要操作」的風格原則相反，且站章與波次數字要為反白另寫一套色。

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `tray-status-menu`: 「面板樣式（macOS）」補光層改側欄面色約九成、三類白底細框卡取代半透明卡與分割線、列式分區與灰底 hover、專案 tab 方塊與圖示列同形；「討論列表」面板的已轉出改為討論卡底部收合列並記憶展開狀態；「面板變更列的品質站章」與「變更列的波次標示與順序同源」移除 hover 反白改色；「面板交接主視窗後滑鼠互動完整」的抽屜字面改為詳情頁。

## Impact

- Affected specs: tray-status-menu（5 條 MODIFIED，其中 4 個 scenario 更名以 REMOVED-SCENARIO 宣告）
- Affected code:
  - New: `apps/desktop/src/components/projectTile.ts`、`apps/desktop/src/panel/promotedExpanded.ts`
  - Modified: `apps/desktop/src/panel/TrayPanel.tsx`、`apps/desktop/src/components/ProjectRail.tsx`、`apps/desktop/src/components/UpdateRailButton.tsx`、`packages/ui/src/components/CopyButton.tsx`、`packages/ui/src/__tests__/copyButton.test.tsx`、`apps/desktop/src/__tests__/trayPanel.test.tsx`、`apps/desktop/src/__tests__/updateRailButton.test.tsx`
  - Removed: (none)

## Why

semantic-color-system（2026-08-05）與 verify-station-parity（2026-08-06）落地後，使用者回饋一批桌面 UI 缺陷：變更詳情抽屜的狀態列把進度條、兩站章、蓋章日期與完整作者身分（含 email）塞在同一行且不可壓縮，超寬即被抽屜裁切看不見；看板卡片的主題化提示因元件庫預設 700ms hover 延遲被感知為「tooltip 被移除」（掃查證實程式碼未刪、僅延遲不一致——詳情抽屜 0ms、卡片 700ms）；系統匣變更列 hover 反白時站章維持紫色、對比不足；截斷收尾淡出與省略號並存被讀成破圖；指令檔過期提示一捲動就消失。討論 quality-skill-pause-and-ui-polish（2026-08-07 結論）裁定五項修法。

目標使用者：桌面 app 與 server-web console 的使用者——涉及看板卡片、變更詳情抽屜、系統匣面板與專案設定頁四個表面；對應 workflow 的品質站狀態檢視與專案設定情境。

## What Changes

- **主題化提示延遲統一**：packages/ui 的 tooltip 元件設單一共用預設延遲 300ms，移除各處 local delayDuration 覆寫（含詳情抽屜現行的 0ms）；desktop 與 server-web 皆消費 packages/ui、同源生效；系統匣面板刻意維持原生 title 提示、不在此範圍
- **詳情抽屜狀態列重做**：兩站章改為章籤（圖示＋狀態詞），蓋章日期與蓋章者完整識別收進主題化提示——與出身列既有「email 收進主題化提示」同構；狀態列於任何資料組合下維持單行、不再溢出被裁切
- **系統匣變更列 hover 章色**：列 hover 反白（主色底）時站章隨列改前景色，與同列其他元素一致；站別由圖示形狀承辨
- **看板卡片標題截斷改省略號收尾**——此為對 2026-08-04 card-name-single-line-fade 規格決定的刻意翻案：使用者裁定淡出被讀成破圖，且全系統其餘截斷皆省略號，統一為省略號並移除 CardNameRow 的漸層遮罩與寬度量測邏輯
- **指令檔過期提示捲動釘選**：提示於分頁內容捲動時釘選於可視區頂部、以不透明底呈現

## Non-Goals

- 不動兩站章的紫色配色本身——「品質站蓋章配色與主色分離」維持；hover 前景色僅適用系統匣列反白瞬間
- 不動系統匣的原生 title 提示機制（其刻意不用主題化提示的既有決定不變）
- 不動看板卡片其他解剖（描述列、meta 列、複製鈕行為、頭像與各行內符號的組成）
- 不含 quality skill 每輪暫停制——同討論轉出的另一變更 quality-skill-round-pause
- 無 CLI、協定或 i18n 詞條變更

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `desktop-app`: 「變更詳情抽屜標頭的四層結構」狀態列章籤化＋單行不溢出；「看板卡片統一解剖學」截斷收尾由漸層淡出改為省略號；另新增「主題化提示統一延遲」與「指令檔過期提示捲動釘選」兩條 requirement
- `tray-status-menu`: 「面板變更列的品質站章」補列 hover 反白時章色隨列改前景色

## Impact

- Affected specs: `desktop-app`、`tray-status-menu`
- Affected code:
  - Modified: packages/ui/src/components/ui/tooltip.tsx、packages/ui/src/components/RichDetailDrawer.tsx、packages/ui/src/components/CardNameRow.tsx、apps/desktop/src/panel/TrayPanel.tsx、apps/desktop/src/components/InstructionUpdatePrompt.tsx、packages/ui/src/__tests__/richDrawer.test.tsx、packages/ui/src/__tests__/cardNameRow.test.tsx、packages/ui/src/__tests__/kanban.test.tsx、apps/desktop/src/__tests__/trayPanel.test.tsx、apps/desktop/src/__tests__/instructionUpdatePrompt.test.tsx
  - New: packages/ui/src/__tests__/tooltipDelay.test.tsx
  - Removed: (none)
- 相容性影響：純前端呈現層變更；CLI 人眼輸出與 --json 皆不受影響，無協定欄位變動；i18n 詞條沿用既有 key，無新增
- 實機驗證備註：tauri dev 載入靜態 dist，實機確認 tooltip 與 sticky 行為前需先重建 desktop 前端

## 1. 主題化提示延遲統一

- [x] 1.1 撰寫延遲統一紅測（規格「主題化提示統一延遲」；design「D1 主題化提示延遲下沉共用元件」）：新增測試斷言共用 tooltip 元件預設延遲——fake timers 下 hover 未達 300ms 不顯示、達 300ms 顯示；並斷言 RichDetailDrawer 不再帶 local delayDuration 覆寫（與卡片同一延遲）。檔案 packages/ui/src/__tests__/tooltipDelay.test.tsx。驗證：vitest 新測試紅燈 <!-- speclink-task:tsk_01KZD1NW43D29AWWN5RY1GSKVS -->
- [x] 1.2 下沉共用延遲預設（同規格；design「D1 主題化提示延遲下沉共用元件」）：packages/ui/src/components/ui/tooltip.tsx 設 300ms 共用預設，packages/ui/src/components/RichDetailDrawer.tsx 移除 0ms 覆寫；行為——看板卡片與詳情抽屜的主題化提示停留 300ms 顯示、延遲一致，skipDelay 連續 hover 即時行為保留；系統匣原生 title 不動。驗證：1.1 轉綠；實機（先重建 desktop 前端，tauri dev 載入靜態 dist）hover 卡片審查章停留 0.3 秒提示顯示 <!-- speclink-task:tsk_01KZD1NW44MM3W7Z6ME06TDFBZ -->

## 2. 詳情抽屜狀態列章籤化

- [x] 2.1 撰寫狀態列紅測（規格「變更詳情抽屜標頭的四層結構」；design「D2 狀態列章籤化與提示同構」）：斷言狀態列顯示「已審查」「已驗證」章籤狀態詞、可視文字不含 email 與蓋章日期、蓋章日期與含 email 完整識別位於提示內容、狀態列容器帶單行不溢出樣式。檔案 packages/ui/src/__tests__/richDrawer.test.tsx。驗證：vitest 新斷言紅燈 <!-- speclink-task:tsk_01KZD1NW44QV13CCG4J77S0TZP -->
- [x] 2.2 狀態列重排實作（同規格；design「D2 狀態列章籤化與提示同構」）：packages/ui/src/components/RichDetailDrawer.tsx 狀態列改為進度條＋百分比＋兩站章籤（圖示＋狀態詞，i18n 沿用既有詞條），蓋章日期與蓋章者完整識別收進主題化提示（與出身列同構），狀態列補最小寬度壓縮與單行約束；行為——任何資料組合下狀態列單行、抽屜無水平裁切。驗證：2.1 轉綠；實機開啟已審查＋已驗證的變更詳情抽屜確認章籤與提示、無溢出 <!-- speclink-task:tsk_01KZD1NW44DVBWFYEDXN4QBWBJ -->

## 3. 系統匣 hover 章色

- [x] 3.1 撰寫章色紅測（規格「面板變更列的品質站章」；design「D3 系統匣章色隨列反白」）：斷言 StationBadges 兩章元素帶 hover 前景色覆寫 class、非 hover 色調表不變。檔案 apps/desktop/src/__tests__/trayPanel.test.tsx。驗證：vitest 新斷言紅燈 <!-- speclink-task:tsk_01KZD1NW446K6NFVD9HWPRS5G5 -->
- [x] 3.2 章色覆寫實作（同規格；design「D3 系統匣章色隨列反白」）：apps/desktop/src/panel/TrayPanel.tsx 的 StationBadges 補 hover 前景色覆寫（僅用既有 token 類別，不新增原生色階字面）；行為——列 hover 反白時兩站章與同列元素同為前景色、離開回復紫色調、站別由圖示形狀承辨。驗證：3.1 轉綠；主題守門測試維持綠燈；實機 hover 已就緒列確認章色 <!-- speclink-task:tsk_01KZD1NW44K2ZMWV8411PRH4TY -->

## 4. 截斷統一省略號

- [x] 4.1 改寫截斷紅測（規格「看板卡片統一解剖學」；design「D4 截斷統一省略號與 CardNameRow 簡化」）：cardNameRow 測試改斷言標題以省略號截斷樣式呈現、遮罩樣式與 data-fade 標記缺席、複製鈕仍同列尾隨；kanban 測試的相關斷言同步改寫。檔案 packages/ui/src/__tests__/cardNameRow.test.tsx、packages/ui/src/__tests__/kanban.test.tsx。驗證：vitest 改寫後紅燈 <!-- speclink-task:tsk_01KZD1NW44ZSF1AEG6FM8FXFXH -->
- [x] 4.2 CardNameRow 簡化實作（同規格；design「D4 截斷統一省略號與 CardNameRow 簡化」）：packages/ui/src/components/CardNameRow.tsx 移除漸層遮罩常數與 ResizeObserver 量測邏輯，標題改 CSS 省略號截斷；行為——變更卡與討論卡標題過長時省略號收尾、短標題完整顯示無省略號，複製鈕行為不變（共用元件一次生效）。驗證：4.1 轉綠 <!-- speclink-task:tsk_01KZD1NW448A5S09E977SA73AT -->

## 5. 過期提示捲動釘選

- [x] 5.1 撰寫釘選紅測（規格「指令檔過期提示捲動釘選」；design「D5 過期提示捲動釘選」）：斷言提示根元素帶頂部釘選與不透明底樣式。檔案 apps/desktop/src/__tests__/instructionUpdatePrompt.test.tsx。驗證：vitest 新斷言紅燈 <!-- speclink-task:tsk_01KZD1NW44AFCB47BF559NF1AW -->
- [x] 5.2 釘選實作（同規格；design「D5 過期提示捲動釘選」）：apps/desktop/src/components/InstructionUpdatePrompt.tsx 根元素改 sticky 頂部釘選＋不透明底＋高於內容的層級；行為——專案設定頁捲動時提示固定於可視區頂部持續可見、下層內容不透出，未捲動時版面與現行一致。驗證：5.1 轉綠；實機於專案設定頁捲動確認，深淺主題各查一次 <!-- speclink-task:tsk_01KZD1NW446FVGPHD1D2MFYG3X -->

## 6. 省略號字形統一

- [x] 6.1 撰寫省略號字形紅測（規格「截斷省略號的統一字形」；design「D6 省略號字形統一」）：於 packages/ui/src/__tests__/theme.test.ts 加斷言——theme.css 含 family 為 `EllipsisLatin`、`unicode-range: U+2026` 的 @font-face 宣告，其 src 以 local() 涵蓋三平台拉丁字型（Helvetica Neue／Arial、Segoe UI、DejaVu Sans），且 body 的 font-family 以 `EllipsisLatin` 為首、其後既有堆疊順序不變。驗證：vitest 新斷言紅燈 <!-- speclink-task:tsk_01KZDDDPA8W96S1CVEVP28TSAD -->
- [x] 6.2 省略號字形層實作（同規格；design「D6 省略號字形統一」）：packages/ui/src/theme.css 新增該 @font-face 並將 `EllipsisLatin` 插入 body 字型堆疊最前；行為——同一畫面上等寬把手與中文文字的截斷省略號為同一字形（拉丁半形貼基線），兩者文字本身字型不變，`font-mono` 元素不受影響。驗證：6.1 轉綠；packages/ui 與 apps/desktop 全套測試維持綠燈 <!-- speclink-task:tsk_01KZDDFFSF9JN9P94PNPQV32J7 -->

## 7. 全套驗證

- [x] 7.1 全套測試與實機走查（design「D1 主題化提示延遲下沉共用元件」至「D6 省略號字形統一」全項）：workspace 前端測試全綠；實機走查六項行為——卡片與抽屜提示延遲一致、抽屜狀態列無裁切、系統匣 hover 章色、卡片標題省略號收尾、設定頁提示釘選、等寬把手與中文文字的省略號同形（先重建 desktop 前端）。檔案 packages/ui/src/__tests__/tooltipDelay.test.tsx、packages/ui/src/__tests__/richDrawer.test.tsx、packages/ui/src/__tests__/cardNameRow.test.tsx、packages/ui/src/__tests__/theme.test.ts、apps/desktop/src/__tests__/trayPanel.test.tsx、apps/desktop/src/__tests__/instructionUpdatePrompt.test.tsx。驗證：npm test 全綠、六項走查通過 <!-- speclink-task:tsk_01KZD1NW44KR8C1ZTVV56DNRN0 -->

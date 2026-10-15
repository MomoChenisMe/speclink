## 1. 裝置外觀偏好

- [x] 1.1 Red：在 apps/desktop/src/__tests__/theme.test.ts、apps/desktop/src/__tests__/appSettingsView.test.tsx 與 apps/desktop/src/__tests__/App.test.tsx 先寫「裝置級外觀偏好」失敗測試，涵蓋規格配色矩陣、缺鍵／非法值／讀取失敗、保存失敗保留原值、系統改變、監聽清理、無專案三選、鍵盤及中英文選中狀態、重掛載恢復；以 npm test -w apps/desktop -- src/__tests__/theme.test.ts src/__tests__/appSettingsView.test.tsx src/__tests__/App.test.tsx 確認失敗來自缺少新行為。 <!-- speclink-task:tsk_01M4PH4AXZ9RHDVXZ5CVPY3V0A -->
- [x] 1.2 Green／Refactor：在 apps/desktop/src/theme.ts 實作「裝置外觀偏好」單一控制器，保存 speclink.uiTheme、解析有效配色、套用 data-theme 與訂閱清理；於 apps/desktop/src/main.tsx 首次 render 前初始化，apps/desktop/src/App.tsx 與 apps/desktop/src/views/AppSettingsView.tsx 接上即時三選及保存失敗提示，apps/desktop/src/i18n/messages.ts 補雙語；執行 1.1 同組測試轉綠，整理僅此次新增的重複邏輯後再跑該組確認。 <!-- speclink-task:tsk_01M4PH4AXZZXNDDQ1CQ1QE2TJE -->

## 2. 共用配色與通知

- [x] 2.1 Red：於 packages/ui/src/__tests__/theme.test.ts、packages/ui/src/__tests__/sonner.test.tsx 與 apps/desktop/src/__tests__/App.test.tsx 加入「全介面配色一致套用有效外觀」、「markdown 內容保留文件結構呈現」、「看板全域操作成功靜默、失敗以 toast 浮層呈現」的改動測試，覆蓋明示 light／dark 優先、無覆寫系統模式、Toaster 傳入及缺省 theme、Markdown 既有字級結構不變；以 npm test -w packages/ui -- src/__tests__/theme.test.ts src/__tests__/sonner.test.tsx 及 npm test -w apps/desktop -- src/__tests__/App.test.tsx 確認新增行為失敗。 <!-- speclink-task:tsk_01M4PH4AXZWJA95VT8AEQ05TJD -->
- [x] 2.2 Green／Refactor：完成「共用配色與通知」，在 packages/ui/src/theme.css 統一 token、dark 變體與 color-scheme 判定，保留單份深淺色票；apps/desktop/src/index.css 維持 Markdown token 映射；packages/ui/src/components/ui/sonner.tsx 接受呼叫端 theme 且缺省 system，apps/desktop/src/App.tsx 傳有效配色；重跑 2.1 測試轉綠，再以 npm run build -w apps/desktop 與 npm run build -w apps/server-web 確認變體可編譯且共享色票未重複分叉。 <!-- speclink-task:tsk_01M4PH4AXZ979PPZZH4RBDVXKZ -->

## 3. 系統匣面板配色同步

- [x] 3.1 Red：於 apps/desktop/src/__tests__/tray.test.ts 與 apps/desktop/src/__tests__/trayPanel.test.tsx 覆蓋「面板樣式（macOS）」的配色同步：主題改變而 workspace 不變時仍推有效值、延遲完成監聽後才 ready、首份快照套色後才呈內容、隱藏／重建後拿最新值、主視窗隱藏時 system 變更與取消訂閱；透過 mock Tauri 事件載入面板入口驗證接線，以 npm test -w apps/desktop -- src/__tests__/tray.test.ts src/__tests__/trayPanel.test.tsx 確認新情境先失敗。 <!-- speclink-task:tsk_01M4PH4AXZGTAJMMD55R1YX643 -->
- [x] 3.2 Green／Refactor：在 apps/desktop/src/tray.ts 的 TraySnapshot 與 buildTraySnapshot 接入 effectiveTheme，主題訂閱即時 pushSnapshot 並於清理時取消；apps/desktop/src/panel/main.tsx 等 listen 完成才 emit ready，收到快照先套色再呈現，處理掛載取消與首份快照前透明空容器，尺寸觀察器於首份內容節點存在後掛載量測；共用 apps/desktop/src/theme.ts 的 DOM 套用，完成「系統匣面板配色同步」且不依賴跨窗 localStorage；執行 3.1 同組測試轉綠並確認既有面板動作、布局與高度調整測試仍通過。 <!-- speclink-task:tsk_01M4PH4AXZEMC0PKPFS5CEYH2T -->

## 4. 專案目錄展示

- [x] 4.1 Red：在 apps/desktop/src/__tests__/projectSettingsView.test.tsx 與 apps/desktop/src/__tests__/App.test.tsx 加入「專案設定顯示完整本機目錄」測試，覆蓋範例 /Users/mosu/project/ai_dev/speclink、/work/project-a 切 /work/project-b、遠端 checkoutRoot／無副本、設定各頁籤共用、含中文空白與 Windows 分隔符、離線 session／revision 衝突不改目錄及無 session 維持恢復頁；以 npm test -w apps/desktop -- src/__tests__/projectSettingsView.test.tsx src/__tests__/App.test.tsx 確認新行為失敗。 <!-- speclink-task:tsk_01M4PH4AXZVWJ30R867R3JF3A7 -->
- [x] 4.2 Green／Refactor：完成「專案目錄展示」，apps/desktop/src/App.tsx 從作用中 locator 推導 projectDirectory 並傳入 apps/desktop/src/views/ProjectSettingsView.tsx，於頁籤上方渲染等寬、可選取、完整換行的唯讀目錄，null 顯示未連結提示；apps/desktop/src/i18n/messages.ts 提供中英文詞條且不影響工作流語言，不新增位置查詢或寫入；執行 4.1 同組測試轉綠並檢查無 stale 專案路徑。 <!-- speclink-task:tsk_01M4PH4AXZNKAPRJGM49PS6N9G -->

## 5. 邊界檢查與渲染驗收

- [x] 5.1 對 apps/desktop/src/theme.ts、apps/desktop/src/tray.ts、apps/desktop/src/panel/main.tsx、apps/desktop/src/views/ProjectSettingsView.tsx 執行 audit discipline 的三角色檢查，確認偏好僅接受固定選項、儲存錯誤不假裝成功、路徑以 React 文字輸出且無 HTML 注入、無 Server 路徑或認證修改；以對應失敗分支測試及差異檢查記錄結果，修正新發現的問題後重跑受影響組。 <!-- speclink-task:tsk_01M4PH4AXZR1EME30W23EYZ7M5 -->
- [x] [M] 5.2 桌面實機驗收：針對 packages/ui/src/theme.css、apps/desktop/src/index.css、apps/desktop/src/panel/main.tsx 與 apps/desktop/src/views/ProjectSettingsView.tsx 執行實際瀏覽器／WebView 渲染驗收：系統深＋手動淺、系統淺＋手動深、system 動態切換，檢查背景、狀態徽章、Markdown、已顯示 toast 的計算色一致與重開記憶；驗證長 Windows／POSIX 路徑換行及複製原文；以 macOS 實窗驗證面板首次、重開、主視窗隱藏時切換與補光可讀性，確認錯誤通知，將使用者操作與結果記錄於 openspec/changes/desktop-theme-and-project-directory/validation.md，不能以 jsdom class 斷言替代。 <!-- speclink-task:tsk_01M4PH4AXZ9MBHGYENDWDPMWR2 -->
- [x] 5.3 驗證 apps/desktop、packages/ui 與 apps/server-web 受影響面整體無回歸：執行 npm test -w apps/desktop、npm test -w packages/ui、npm test -w apps/server-web、npm run build -w apps/desktop、npm run build -w apps/server-web；確認變更未觸及 CLI／引擎輸出而無 golden 修改，最後執行 node --test scripts/*.test.mjs scripts/*/*.test.mjs，全部通過後才勾選。 <!-- speclink-task:tsk_01M4PH4AXZD3VVAK22SNQXRBX9 -->
- [x] [M] 5.4 Server Web 實際瀏覽器驗收（2026-10-15 使用者豁免，未測；勾選僅表示豁免結案）：針對 apps/server-web 與 packages/ui/src/theme.css，在未設定 data-theme 外觀覆寫的頁面切換系統淺色／深色，檢查背景、文字與狀態色的計算樣式皆跟隨系統，且不受桌面 app 儲存偏好影響；將操作環境與兩種配色結果記錄於 openspec/changes/desktop-theme-and-project-directory/validation.md，不能以自動化測試或建置取代實際畫面驗收；另確認 5.3 的 node --test scripts/*.test.mjs scripts/*/*.test.mjs 結果已通過。

## 1. Tauri 殼能力與圖示資產

- [x] 1.1 啟用 tray 能力：apps/desktop/src-tauri/Cargo.toml 的 tauri dependency 加上 tray-icon 與 image-png features，apps/desktop/src-tauri/capabilities/default.json 補齊 tray、menu、視窗 show/set-focus/unminimize 權限——交付「前端可建立 tray/menu 並呼叫開窗動作而不被權限系統拒絕」的能力面。驗證：cargo build --release -p speclink-desktop 編譯成功
- [x] 1.2 tray 圖示資產：以使用者提供的單色 Speclink 標記（apps/desktop/src-tauri/icons/speclink-tray-18@2x.png，36×36 深藍剪影）base64 編碼內嵌為 apps/desktop/src/trayIcon.ts 常數，附解碼函式供 Image.fromBytes 建圖示——交付「系統匣顯示 Speclink 標記」。app bundle 圖示（icon.icns 等）由使用者更新透明度、tauri.conf.json 既已引用。驗證：trayIconBytes 解碼守護測試（斷言 PNG magic bytes）綠

## 2. 選單模型（純函式，TDD）

- [x] 2.1 撰寫選單模型測試（紅），覆蓋需求「系統匣圖示與原生選單」「生命週期分區與變更進度」「變更子選單動作」「討論列表」「macOS 進行中數文字徽章」：於 apps/desktop/src/__tests__/tray.test.ts 斷言——buildTrayModel 自快照組出：專案項（作用中打勾）、依 proposed/in-progress/ready 序的分區標題、每張變更帶進度條標籤與「開啟此變更」子選單動作、討論項（slug＋topic）、空狀態、徽章＝進行中數；progressBar 依比例填 unicode 方塊、total 0 不畫。驗證：npm test -w apps/desktop 新測試失敗（紅）
- [x] 2.2 於 apps/desktop/src/tray.ts 實作 buildTrayModel 與 progressBar 純函式（快照 → 選單模型：project/header/change(含 actions)/discussion/empty/separator/open/quit 的 discriminated union），使 2.1 全部斷言通過；分區沿用 packages/ui 的 changeStage/STAGES。驗證：npm test -w apps/desktop 全綠

## 3. Tauri 接線（訂閱、去抖重建、點擊 handlers，TDD）

- [x] 3.1 撰寫接線測試（紅），覆蓋需求「選單內容與看板同源」「選單專案切換」「變更子選單動作」「討論列表」「開啟視窗與結束動作」：於 tray.test.ts 以 vi.mock 樁替 @tauri-apps/api 的 tray/menu/window/image，斷言——初始化建圖示並掛選單（含專案 check 項）；store 變動去抖後以新模型重建選單並更新 macOS 徽章；點非作用中專案呼叫 openProjectAt；變更子選單「開啟此變更」開主視窗＋openDetail；討論項開主視窗＋openDiscussion；「開啟 Speclink」show＋setFocus；「結束」映射原生 predefined Quit。驗證：npm test 新測試失敗（紅）
- [x] 3.2 於 tray.ts 實作接線層（Image.fromBytes 建圖示、buildTrayModel→Menu.new 的 toOptions 映射含 Submenu/header/discussion、store subscribe＋去抖重建、各點擊 handler、macOS setTitle 徽章、暴露初始化與銷毀入口），使 3.1 全部斷言通過。驗證：npm test -w apps/desktop 全綠
- [x] 3.3 於 apps/desktop/src/App.tsx 的 AppInner 啟動 useEffect 呼叫 tray 初始化入口、卸載時呼叫銷毀函式；apps/desktop/src/i18n/messages.ts 補 tray 文案鍵（tray.open/quit/discussionsHeader/openChange/noChanges，繁中＋英文）——交付「app 載入完成後系統匣圖示與選單存在」。驗證：npm test 全綠、npm run build -w apps/desktop 通過型別檢查

## 4. 建置與真實視窗驗證

- [x] 4.1 全量建置驗證：npm test -w apps/desktop 與 packages/ui 全綠、npm run build -w apps/desktop 產出 dist、cargo build --release -p speclink-desktop 編譯成功（重建前先關閉執行中的 app）
- [x] 4.2 macOS 真實視窗驗證（GUI 改動不得只靠 jsdom）：啟動 release app 逐項確認——(1) 選單列出現 Speclink 標記圖示（非佔位符），旁有進行中數徽章；(2) 展開選單依序見專案區（作用中打勾）、生命週期分區（提案中/進行中/已就緒，進行中變更帶文字進度條＋n/m）、討論區（各 active 討論）、動作區（開啟 Speclink、結束 ⌘Q）；(3) 點非作用中專案切換不奪焦；(4) 變更子選單「開啟此變更」與討論項開主視窗＋跳詳情；(5) 「開啟 Speclink」聚焦主視窗、「結束」關閉 app。已由使用者於 macOS 實機截圖確認選單呈現與資料正確（真原生 NSMenu、vibrancy、⌘Q）

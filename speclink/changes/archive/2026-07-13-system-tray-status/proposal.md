## Why

Speclink 的核心情境是 agent／CLI 在背景寫入 openspec/，而使用者（透過 AI 代理跑 SDD 的開發者）人在終端機或編輯器等其他視窗工作。看板雖會隨檔案監看自動刷新，但視窗不在前景時進度不可見；apply／verify 等長時間 workflow 階段需要「不切視窗的一眼狀態」——在系統選單列／系統匣看到變更進度與討論，並能直接切換專案、跳進某變更或討論。

## What Changes

- 新增系統匣呈現面（macOS 選單列、Windows 通知區域、Linux AppIndicator）：圖示＋原生下拉選單（menu-first，跨平台一致），選單由前端擁有、訂閱既有 store，資料與看板同源（本地監看刷新；未來遠端經 SpeclinkDataSource contract 免費繼承）
- 選單內容（原生 NSMenu 能力內）：
  - 專案區——已開啟分頁，點選即切換、作用中打勾、不搶焦點
  - 生命週期分區——提案中／進行中／已就緒各一分區標題，每個階段列出其變更；每張變更一個子選單，標籤帶 unicode 文字進度條與「名稱 n/m」，子選單含「開啟此變更」（開主視窗＋跳該變更詳情）
  - 討論區——列出 active 討論，點選開主視窗＋跳該討論抽屜
  - 動作區——「開啟 Speclink」（顯示並聚焦主視窗）、「結束」（原生 predefined Quit）
- macOS 於圖示旁顯示進行中變更數文字徽章（僅 macOS 生效，其他平台自動忽略）
- tray 圖示為使用者提供的單色 Speclink 標記（template 渲染，適應深淺色選單列）
- Rust 側零新模組：僅啟用 tauri 的 tray-icon 與 image-png features、補 capability 權限

## Non-Goals

- 關窗背景常駐（隱窗不關、macOS ActivationPolicy、Dock 圖示去留）——屬獨立的 app 生命週期改造；第一版維持關窗即退出
- 自訂 webview 彈出面板（CodexBar 式）——經 tray-rich-panel 討論探索後裁定放棄：CodexBar/ChatGPT 那種質感是原生 Swift/AppKit（NSPopover＋SwiftUI 視圖），Tauri webview 本質追不到；原生 NSMenu 才是 Tauri 裡最原生的呈現，故回歸原生選單並在其能力內做豐富
- Rust 側直呼 speclink-desktop-core 組選單——遠端模式下需在 Rust 重複 Client SDK 與事件訂閱，形成第二條資料路徑，違反 UI 單一 DataSource contract
- CLI 不受影響：無新子指令、無旗標、無輸出變更

## Capabilities

### New Capabilities

- `tray-status-menu`: 系統匣圖示、macOS 文字徽章與原生下拉選單——跨平台圖示與選單的內容組成（專案切換、生命週期分區與變更進度、變更子選單動作、討論列表、開窗/結束）、焦點行為、資料同源刷新、平台差異（macOS 徽章、menu-first 約束）

### Modified Capabilities

(none)

## Impact

- Affected specs: 新增 `tray-status-menu`；`desktop-app` 既有需求不變
- 影響 crate：speclink-core、speclink-cli 皆不受影響；改動集中在 apps/desktop 前端與 Tauri 殼設定。CLI 人眼與 `--json` 輸出不變，回歸對照不受波及
- Affected code:
  - New: apps/desktop/src/tray.ts（tray 模組：選單模型純函式＋Tauri 接線）、apps/desktop/src/trayIcon.ts（tray 圖示 base64 資產與解碼）、apps/desktop/src/__tests__/tray.test.ts（單元測試）
  - New（圖示資產，使用者提供）: apps/desktop/src-tauri/icons/speclink-tray-18.png（18×18）、speclink-tray-18@2x.png（36×36）
  - Modified: apps/desktop/src-tauri/Cargo.toml（tray-icon／image-png features）、apps/desktop/src-tauri/capabilities/default.json（tray／menu／window 權限）、apps/desktop/src/App.tsx（AppInner 啟動初始化 tray）、apps/desktop/src/i18n/messages.ts（tray 文案鍵）
  - Modified（app bundle 圖示，使用者更新透明度）: apps/desktop/src-tauri/icons/icon.icns、icon.png、32x32.png、64x64.png、128x128.png、128x128@2x.png（tauri.conf.json 既已引用，下次 build 自動採用）
  - Removed: (none)

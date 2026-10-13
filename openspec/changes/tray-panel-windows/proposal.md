## Why

在 Windows 上用 desktop 追蹤 SDD 流程的開發者（透過 AI 代理跑 propose／apply 的開發者、PO、PM），目前點系統匣圖示只會得到原生下拉選單：看不到進度條、站章、波次，也沒有卡片收合，與 macOS 面板的「一眼看全部狀態」差很多。面板前端本來就沒有任何平台分支，缺的只是 Rust 端為 Windows 開視窗的那一層；面板限 macOS 是已封存變更 tray-macos-panel-only 列的 Non-Goal，沒有技術上做不到的理由。來源討論 `tray-panel-windows` 已定案。

## What Changes

- Windows 系統匣改為「左鍵開閉面板、右鍵顯示現有原生選單」；面板建立失敗時左鍵也退回原生選單；Linux 維持原生選單不動。
- Windows 面板與 macOS 共用同一份畫面：專案 tab 條、四張內容卡（提案中→進行中→已就緒→討論）、頁尾單列、常駐複製鈕、內容卡收合與共用收合記憶、高度自適應與上限 640。
- Windows 外觀差異：面板根層用不透明側欄底色、不鋪半透明補光、不畫自己的圓角；視窗不透明、無系統外框、開陰影，Windows 11 由系統畫圓角與陰影，Windows 10 為直角細邊。
- Windows 位置：圖示上方置中（工作列在頂端放不下時改放下方），水平方向限制在螢幕範圍內；視窗大小每次改變就重新定位，讓面板底部一直貼著工作列。
- Windows 焦點：打開時取得焦點、失焦自動收起；收起後約 300ms 內的開啟要求一律忽略，避免點圖示關面板時閃一下又打開。
- 規格：tray-status-menu 的「面板樣式（macOS）」改為跨平台的「面板樣式」（內文分「共用」與「平台差異」），同步修改「系統匣圖示與原生選單」與「生命週期分區與變更進度」（兩者都點名舊需求或寫死「非 macOS」）；「面板內容卡收合」由 tray-panel-content-priority 寫成不綁平台的措辭，本變更不需再改。

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `tray-status-menu`: 「面板樣式（macOS）」以新需求「面板樣式」取代（跨平台、分共用與平台差異）；「系統匣圖示與原生選單」的平台分流改為 macOS 面板、Windows 面板＋右鍵原生選單、Linux 原生選單；「生命週期分區與變更進度」的原生選單適用範圍與面板引用跟著改。

## Impact

- 目標使用者與情境：用 Windows 版 desktop 追蹤變更生命週期的開發者／PO／PM，對應 propose 之後到 archive 之前「看狀態、決定下一個動詞」的時刻。
- Affected specs: `tray-status-menu`
- 影響的 app：只有 `apps/desktop`——Tauri 殼的面板視窗模組、command 委派、外掛註冊與能力宣告，speclink-desktop-core 新增兩個純函式（面板水平限制、失焦後的重開抑制），系統匣建立與點擊分流、面板前端的平台外觀分支；speclink-core、speclink-cli、speclink-host、server 不動。
- 相容性影響：沒有 CLI 指令、`--json` 輸出、設定欄位或技能的變更；不新增 IPC command（沿用 toggle_tray_panel／hide_tray_panel）。Windows 使用者升級後左鍵由原生選單改為面板，右鍵仍是原本的原生選單；macOS 與 Linux 行為不變。
- 依賴：建立在 tray-panel-content-priority 的面板版面之上，且兩者都改「面板樣式（macOS）」，須在其後實作與封存。
- Affected code:
  - New: apps/desktop/core/src/tray_panel.rs, apps/desktop/src-tauri/capabilities/tray-panel.json
  - Modified: apps/desktop/core/src/lib.rs, apps/desktop/src-tauri/Cargo.toml, apps/desktop/src-tauri/src/panel.rs, apps/desktop/src-tauri/src/lib.rs, apps/desktop/src/tray.ts, apps/desktop/src/store.ts, apps/desktop/src/panel/TrayPanel.tsx, apps/desktop/src/panel/main.tsx, apps/desktop/src/__tests__/tray.test.ts, apps/desktop/src/__tests__/store.test.ts, apps/desktop/src/__tests__/trayPanel.test.tsx, apps/desktop/src/__tests__/themeBoot.test.ts
  - Removed: apps/desktop/src-tauri/capabilities/macos.json

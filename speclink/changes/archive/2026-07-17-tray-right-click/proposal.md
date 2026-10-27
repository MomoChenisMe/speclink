## Why

macOS 面板樣式下，右鍵點擊系統匣圖示無任何反應——tray 點擊接線只放行左鍵，與 macOS 狀態列慣例（左右鍵同行為）不符，spec 也只寫「點擊」未分鍵，屬實作缺口。連帶發現：面板樣式下 tray 完全沒有「結束」入口（面板動作區僅「開啟 Speclink」）——翻查三個封存 panel 變更的設計文件均未討論過「結束」去向，屬無意遺落。目標使用者為桌面 app（系統匣常駐）的開發者使用情境：不進主視窗即可從系統匣完成開啟、進設定、結束 app 三個基本動作。

## What Changes

- 右鍵（次要鍵）點擊系統匣圖示與左鍵完全等價：於 macOS 面板樣式開閉面板（源自討論 tray-right-click 的 A 案裁決）。
- tray 動作區跨平台補齊為「開啟 Speclink、設定、結束」三項：macOS 面板動作區塊與非 macOS 原生選單動作區皆同（跨平台對稱）。
- 「設定」動作＝喚起主視窗、取得焦點並切換至設定頁（與「開啟此變更」同一喚起語意；設定頁為主視窗既有頁面）。
- 面板的「結束」需新增一個桌面 app 端 Tauri command 作能力橋接（webview 無法自行結束行程），跟隨桌面 app 既有薄包裝命令模式。

## Non-Goals

- 右鍵開原生小選單（B 案）：面板／原生選單雙路徑、複雜度高一階，討論已排除。
- 引入 process 類外掛（plugin-process）：整顆外掛只為一個 exit 呼叫，改以單行命令達成。
- 設定頁本身的行為與內容：本變更僅新增系統匣入口，設定頁不動。
- Speclink CLI 引擎（speclink-core / speclink-cli）：本變更純屬桌面 app（apps/desktop），不動任何 crate 的指令、輸出與 `--json` 契約——無相容性影響、無回歸對照變動。
- 非 macOS 平台的右鍵行為：原生選單樣式下右鍵由系統原生開選單，本已可用、不在本變更範圍。

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `tray-status-menu`: 三個 requirement 修改——「系統匣圖示與原生選單」（動作區定義擴為三項）、「開啟視窗與結束動作」（新增「設定」動作行為與面板端「結束」）、「面板樣式（macOS）」（點擊不分左右鍵、面板動作區塊擴為三項）。

## Impact

- Affected specs: `tray-status-menu`（上列三個 requirement 的 delta）
- Affected code:
  - New: (none)
  - Modified:
    - apps/desktop/src/tray.ts（點擊按鍵過濾放寬為左右鍵、選單模型動作區加「設定」項、面板動作事件新增設定與結束的分派）
    - apps/desktop/src/panel/TrayPanel.tsx（動作區塊加「設定」「結束」兩列）
    - apps/desktop/src/panel/main.tsx（面板動作 props 接線新增設定與結束兩個動作發送）
    - apps/desktop/src-tauri/src/lib.rs（新增結束 app 的 Tauri command 並註冊）
    - apps/desktop/src/i18n/messages.ts（新增系統匣「設定」文案 key）
    - apps/desktop/src/__tests__/tray.test.ts（右鍵觸發、動作區模型、動作分派的測試）
    - apps/desktop/src/__tests__/trayPanel.test.tsx（面板動作區塊兩列與動作發送的測試）
  - Removed: (none)

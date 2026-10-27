## Why

系統匣目前是純原生選單（menu-first）：變更名稱與討論識別無法從選單複製（原生選單項不可選字），使用者要取得名稱餵 CLI 動詞或貼到訊息，只能開主視窗繞到詳情抽屜；同時選單的資訊呈現受限於純文字（進度條以 Unicode 方塊模擬），無法做出更豐富的狀態呈現——原生選單項嵌自訂 view 在 Tauri 選單層（muda）未暴露，走不通。另外系統匣討論項現以 topic 為標籤，違反共用詞彙的識別錨點慣例（討論以 slug 直出、topic 降為描述）。

目標使用者：以桌面 app 監看 SDD 工作流的開發者。使用情境：從系統匣快速複製變更名稱／討論 slug，作為執行 speclink 動詞（如 propose 的 from-discussion 參數）或跨工具引用的把手；以及在不開主視窗的前提下獲得更豐富的狀態一覽。

本提案源自討論 tray-copy-title-and-native-status 的結論：複製功能入原生選單、討論項 slug 化，並以 webview 面板模式試驗豐富呈現——兩種樣式並存、設定切換、由使用者實測後裁決去留。

## What Changes

- 原生選單的變更子選單新增「複製名稱」動作：複製該變更的 name（不含進度條字元），與詳情抽屜複製鈕行為一致。
- 討論項由單層選單項改為子選單：內含「開啟此討論」與「複製 slug」；父項標籤由 topic 改為 slug 直出（識別錨點慣例對齊，topic 於子選單內的呈現由 design 定）。
- 剪貼簿寫入經 Rust 端 clipboard 外掛（tauri-plugin-clipboard-manager）——系統匣點擊時主視窗可能隱藏或無焦點，webview 的 navigator.clipboard 會拒寫。
- 新增「面板」樣式：點擊系統匣圖示彈出貼齊圖示的 webview 面板視窗（tauri-plugin-positioner 定位＋tauri-nspanel 不搶焦點＋vibrancy 毛玻璃），內容與看板同源（同一 store），變更／討論列列尾常駐複製鈕；失焦自動收合；高度自適應內容（達上限後內部捲動）。
- 系統匣討論區分流：「討論」分區列討論中、「已轉出」分區列已轉出變更的討論（原生選單與面板同構；無已轉出時不顯示該分區）——使用者實測後裁決補充（ingest 2026-07-16）。
- 分區溢出摺疊：每分區直列前 5 筆，第 6 筆起收進「還有 N 個…」——原生選單為子選單（項目保有完整動作、超高由 macOS 選單原生捲動）、面板為可展開列——使用者實測後補充（ingest 2026-07-16）。
- 設定頁新增「系統匣樣式」偏好（原生選單／面板），持久化於 app 本機，即時切換無需重啟。
- openspec/LANGUAGE.md 的討論識別錨點例外枚舉面擴充：納入系統匣討論項與其複製動作。
- 兩種樣式全程並存：原生選單程式不拆除；最後一項任務為使用者實測裁決保留哪一種（或兩者皆留），裁決若改變規格再走 ingest 收斂。

## Non-Goals

- 不 fork muda 以支援原生選單項內嵌自訂 view——工程量不成比例（討論已排除）。
- 不動 speclink-core／speclink-cli：無 CLI 子指令、旗標、輸出或 exit code 變更，無回歸對照影響。
- 不改 .speclink.yaml 與 openspec/config.yaml：系統匣樣式是 app 本機 UI 偏好，不是專案設定。
- 不擴充 macOS 選單列文字徽章的資訊量（討論列為 deferred，本變更不動）。
- 不在本變更內預設最終樣式的裁決結果：裁決是最後一項任務的產物，不預寫進規格。
- 不採 navigator.clipboard、不複製含進度條字元的標籤全文、不以左右鍵分流或 dev flag 作為並存機制（討論已裁定設定頁切換）。

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `tray-status-menu`: 變更子選單新增複製動作；討論項改為子選單（開啟＋複製 slug）且標籤 slug 化；新增面板樣式需求（與原生選單並存、依偏好切換、貼齊圖示、失焦收合、hover 複製鈕）；「系統匣圖示與原生選單」的 menu-first 敘述放寬為兩種樣式擇一。
- `desktop-config`: 設定頁新增「系統匣樣式」偏好需求（原生選單／面板二選、app 本機持久化、即時生效）。

## Impact

- Affected specs: tray-status-menu、desktop-config
- Affected code:
  - New: apps/desktop/src/panel/TrayPanel.tsx、apps/desktop/src/panel/main.tsx、apps/desktop/panel.html、apps/desktop/src/__tests__/trayPanel.test.tsx、apps/desktop/src/trayStyle.ts、apps/desktop/src/__tests__/trayStyle.test.ts
  - Modified: apps/desktop/src/tray.ts、apps/desktop/src/__tests__/tray.test.ts、apps/desktop/src/i18n/messages.ts、apps/desktop/src/App.tsx、apps/desktop/src/store.ts、apps/desktop/src/views/SettingsView.tsx、apps/desktop/src-tauri/src/lib.rs、apps/desktop/src-tauri/Cargo.toml、apps/desktop/src-tauri/capabilities/default.json、apps/desktop/src-tauri/tauri.conf.json、apps/desktop/package.json、apps/desktop/vite.config.ts、Cargo.lock、openspec/LANGUAGE.md
  - Removed: (none)
- 新增相依：Rust 端 tauri-plugin-clipboard-manager、tauri-plugin-positioner、tauri-nspanel（git 釘 commit）；vibrancy 用 tauri 內建 window effects（不另加 window-vibrancy——與 tauri 內建版本重複連結）；前端 JS 綁定 @tauri-apps/plugin-clipboard-manager、@tauri-apps/plugin-positioner。
- 相容性影響：CLI 人眼與 --json 輸出皆無變更（不動 crates/）；系統匣討論項標籤由 topic 改為 slug 屬桌面 UI 行為變更，與看板討論卡的 slug 為題慣例對齊，無遷移動作。

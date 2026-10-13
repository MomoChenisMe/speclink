# Review — tray-panel-windows

## Round 1

**Phase**: discovery
**Patch**: sha256:3a56ee0d8c6c4fa89e2335258e2d7741a82cecf7ea7ade1690f31784af981d3f
**Scope**: apps/desktop/core/src/lib.rs, apps/desktop/core/src/tray_panel.rs, apps/desktop/src-tauri/Cargo.toml, apps/desktop/src-tauri/capabilities/tray-panel.json, apps/desktop/src-tauri/src/lib.rs, apps/desktop/src-tauri/src/panel.rs, apps/desktop/src/__tests__/platform.test.ts, apps/desktop/src/__tests__/store.test.ts, apps/desktop/src/__tests__/themeBoot.test.ts, apps/desktop/src/__tests__/tray.test.ts, apps/desktop/src/__tests__/trayPanel.test.tsx, apps/desktop/src/panel/TrayPanel.tsx, apps/desktop/src/panel/main.tsx, apps/desktop/src/store.ts, apps/desktop/src/tray.ts
- [SUGGESTION] apps/desktop/src-tauri/src/panel.rs — Standards: possible 慣例分歧。`static LAST_HIDDEN: Mutex<Option<Instant>>` 是殼層唯一的全域 static 狀態；lib.rs 其他整個程式共用的狀態都走 `app.manage(...)`（WatcherState、ConnectionsState、EventManager）。
- [SUGGESTION] apps/desktop/src-tauri/src/panel.rs — Standards: possible Duplicated Code。macos::create 與 windows::create 的 builder 鏈前段（new(panel.html)／title／inner_size／decorations／resizable／skip_taskbar／always_on_top／visible）相同，兩個 hide 的寫法也幾乎一樣。design D6 刻意分成兩個平台區塊，可以接受。
- [SUGGESTION] apps/desktop/src-tauri/src/lib.rs — Standards: possible Shotgun Surgery。`cfg(any(target_os = "macos", target_os = "windows"))` 和它的否定式在 lib.rs 出現六次（mod panel、toggle×2、hide×2、positioner 註冊），Cargo.toml 和 capabilities 的 platforms 也各寫一份。以後增減面板平台要同步改很多處。
- [SUGGESTION] apps/desktop/core/src/tray_panel.rs — Standards: possible Speculative Generality。`pub const REOPEN_SUPPRESS` 設成公開，但殼層只用到兩個函式。測試模組的 `use std::time::{Duration, Instant};` 和 `use super::*` 重複。
- [SUGGESTION] apps/desktop/src/__tests__/trayPanel.test.tsx — Standards: 命名不一致。規格需求已改名為「面板樣式」，patch 只改了一部分出處標註（store.test、tray.ts、新的 describe）。這個檔案仍有 8 處、TrayPanel.tsx 有 3 處寫「面板樣式（macOS）」，新 describe 和舊名並排。
- [CRITICAL] apps/desktop/src-tauri/src/lib.rs — Correctness: `toggle_tray_panel` 在 Windows 仍是同步 `#[tauri::command] fn`，但它呼叫的 `panel::toggle` 第一次執行時會走 `create()` → `WebviewWindowBuilder::build()`，也就是在同步命令裡建視窗。Tauri 2.11.5 的 webview_window.rs 第 58 行和第 115 行文件寫明：在 Windows 上這樣做會因 WebView2 而死結（deadlock，兩邊互等、程式卡住）。結果是 Windows 使用者第一次左鍵點系統匣圖示，整個 app 就卡死，也不會回 Err，前端的 panelFallback 根本走不到。macOS 不受影響。
- [WARNING] apps/desktop/src-tauri/src/panel.rs — Correctness: Windows 的 `create()` 建出的面板視窗只會被 `hide()`、從不關閉，所以會一直留在 Tauri 的視窗表裡。Tauri 要等視窗表清空才發 ExitRequested（tauri-runtime-wry lib.rs 第 4313 行）。觸發路徑：開過一次面板後，按主視窗 X 關閉，行程不會結束；系統匣的 JS 處理函式跟著主 webview 一起銷毀，左鍵沒反應，只剩右鍵原生選單的「結束」能用。改動前 Windows 沒有面板視窗，關主視窗 app 就直接結束。
- [SUGGESTION] apps/desktop/src-tauri/src/panel.rs — Correctness: `toggle` 裡 `hide()` 失敗，或 `place` 的 positioner 暫時出錯（例如 "Tray position not set"），都會回 Err。前端收到後會走 panelFallback，整個執行期永久降為 native-menu。可以考慮把暫時性錯誤和建立失敗分開處理。

## Round 2

**Phase**: validation
**Patch**: sha256:6fda839d61e63e28e2a38545060c6112150aebe0b6502a48406582f65ba2be9f
**Scope**: apps/desktop/core/src/tray_panel.rs, apps/desktop/src-tauri/Cargo.toml, apps/desktop/src-tauri/capabilities/tray-panel-macos.json, apps/desktop/src-tauri/capabilities/tray-panel.json, apps/desktop/src-tauri/src/lib.rs, apps/desktop/src-tauri/src/panel.rs, apps/desktop/src/__tests__/tray.test.ts, apps/desktop/src/__tests__/trayPanel.test.tsx, apps/desktop/src/panel/TrayPanel.tsx, apps/desktop/src/tray.ts
- [SUGGESTION] apps/desktop/src-tauri/src/panel.rs — Standards: 模組說明原本寫「建立或定位失敗回 Err」，被改成「任一步建立失敗回 Err」，跟程式對不上：macOS 的 positioner 定位失敗仍回 Err（design 也寫 macOS 維持原樣），Windows 顯示失敗也回 Err。

## Round 3

**Phase**: validation
**Patch**: sha256:105b2dfbaf486d517523afbfa592b8ca6f61238a6271277c751ec79ce18cd14b
**Scope**: apps/desktop/src-tauri/src/panel.rs

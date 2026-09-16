## Problem

桌面 app（apps/desktop，macOS）的使用者——透過 AI 代理跑 SDD 的開發者、PO 與 PM——在看板與抽屜之間切換時，會遇到一種「點得到、但游標與 hover 全死」的狀態：滑鼠移到卡片或按鈕上，CSS hover 不亮、游標樣式不變、Radix Tooltip 不出現；點擊卻照常運作。再切換幾次 app、或把 app 放到背景一陣子再回來，就又恢復。

使用者已找到穩定重現路徑：

1. Speclink 主視窗在背景約 5～10 秒。
2. 點系統匣圖示開面板（tray-status-menu「面板樣式（macOS）」）。
3. 點面板中的某個變更列。
4. 主視窗自動顯示到前景並開啟該變更的詳情抽屜。此時抽屜內按鈕的 hover 已經不亮。
5. 點抽屜灰底關閉抽屜，看板卡片的 hover 也不亮。

同一份 app 走 Cmd-Tab 回前景不會觸發。本問題只影響面板→主視窗這條交接路徑，影響對象是 apps/desktop 的 Tauri 殼（src-tauri）與前端 tray 接線（src/tray.ts）；speclink-core、speclink-cli 不受影響。

## Root Cause

討論 desktop-hover-dead-after-foreground 以四次觀察把病灶鎖定在原生層的滑鼠追蹤：

- 點擊正常、滑鼠移動全無 → 不是前端（Radix／React）問題；CSS hover 與游標樣式由 WebKit 引擎處理，前端管不到。
- Cmd-Tab 路徑不觸發 → 觸發點在面板交接主視窗的啟用順序。
- 發作時按 Esc 關得掉抽屜 → 主視窗確為 key window、鍵盤事件正常送達；「主視窗沒拿到 key window」的假設被推翻。

剩下的解釋：主視窗有鍵盤，但 AppKit 對主視窗的滑鼠追蹤區（NSTrackingArea）沒有重算「游標已進入」的狀態，於是不送滑鼠移動事件；點擊不看這個狀態，照常送達。最合理的觸發機制：面板貼齊系統匣圖示下方、與主視窗上緣重疊，使用者點面板時游標其實已在主視窗框內；面板 orderOut 消失後游標「沒有移動」就落在主視窗上，AppKit 不補送「游標進入」。切換 app 或 resize 視窗會讓 AppKit 重算，於是恢復。

程式碼上的交接路徑：面板 emit tray-panel-action → src/tray.ts 的 openIn 同步呼叫 openMainWindow（unminimize→show→setFocus）與 store 動作；tao 0.35.3 的 set_focus 順序為先 makeKeyAndOrderFront、再 activateIgnoringOtherApps；面板（src-tauri/src/panel.rs）設為 nonactivating、失去 key 即 orderOut。這個機制未經實機證實，靜態讀碼到此已無法再縮小範圍。

## Proposed Solution

依序試三個修法，第一個讓重現路徑不再發作的就收，其餘不做：

- **A. 面板先收、再叫主視窗**：src/tray.ts 的 open-* 動作（open-change、open-discussion、open-app、open-recovery、open-server-settings、reauthenticate、add-project、open-project-settings、open-settings 等所有會喚起主視窗的動作）先把面板收合，再執行 openMainWindow。一個冪等的 Rust 收合命令（hide_tray_panel）加幾行 TypeScript。
- **B. Rust 端新命令取代 setFocus**：src-tauri/src/lib.rs 新增一個「顯示並聚焦主視窗」的 Tauri command，順序為先 activateIgnoringOtherApps、再 makeKeyAndOrderFront、再把主視窗的 WKWebView 設回 first responder（對齊 wry 建視窗時的做法）；src/tray.ts 的 openMainWindow 改呼叫此命令。
- **C. 交接後重設追蹤區**：主視窗成為前景後，對主視窗呼叫一次 AppKit 的追蹤區重算（invalidateCursorRectsForView 或等價的無感 nudge）。最貼病灶、但最像補丁。

每一步做完都以下方 Success Criteria 的重現路徑實機驗收；通過即停在該步。無 CLI 指令、設定欄位或技能檔異動，無相容性影響。

**實測結果（2026-09-16）**：步驟 A 通過——使用者以本機安裝的開發版走三條重現路徑，hover 皆正常；B 與 C 未執行。

## Non-Goals

- 不修「主視窗不是 key window」——Esc 測試已推翻。
- 不改前端 Radix／React 層。
- 不把 macOS 系統匣預設改回原生選單。
- 不追 AppKit 的確切機制；若 C 才是有效解，修好後另開上游 issue（同族：tauri#7884、tauri#11386、wry#175）。

## Success Criteria

- 依重現路徑操作（主視窗背景 5～10 秒→開面板→點變更列→主視窗前景、抽屜開啟）：抽屜開著時，滑鼠移到抽屜內按鈕上 hover 要亮、游標樣式要變。
- 關閉抽屜後，滑鼠移到看板卡片上 hover 要亮、tooltip 要出現。
- 走 open-app（「開啟 Speclink」）與 open-discussion 兩條交接路徑，主視窗到前景後 hover 同樣正常。
- Cmd-Tab 回前景的既有行為不變；面板「不搶焦點、失焦自動收合」的既有情境（tray-status-menu）不變。
- apps/desktop 的 vitest（src/__tests__/tray.test.ts）覆蓋「open-* 動作先收面板再喚起主視窗」（若 A 成立）或「openMainWindow 呼叫新命令」（若 B 成立）。

## Impact

- Affected specs: tray-status-menu（新增「面板交接主視窗後滑鼠互動完整」需求）
- Affected code:
  - Modified: apps/desktop/src/tray.ts、apps/desktop/src/__tests__/tray.test.ts、apps/desktop/src-tauri/src/lib.rs（A 的 hide_tray_panel 命令）、apps/desktop/src-tauri/src/panel.rs（A 的冪等 hide 函式）
  - New: 無
  - Removed: 無
  - 未建立（A 通過，B 與 C 不執行）：Tauri command focus_main_window 與其追蹤區重算段落

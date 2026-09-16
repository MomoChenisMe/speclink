## Context

macOS 桌面 app 的系統匣面板（tray-status-menu「面板樣式（macOS）」）是一個 nonactivating 的 NSPanel：開啟不奪前景 app 焦點、失去 key 即 orderOut 收合（apps/desktop/src-tauri/src/panel.rs）。使用者點面板中的變更列後，面板 emit tray-panel-action，主視窗端 apps/desktop/src/tray.ts 的 openIn 同步做兩件事：呼叫 openMainWindow（unminimize→show→setFocus）與執行 store 動作（開抽屜）。setFocus 落到 tao 0.35.3 的 set_focus，順序為先 makeKeyAndOrderFront、再 activateIgnoringOtherApps。

沿這條路徑回前景後，主視窗會進入「鍵盤正常（Esc 關得掉抽屜）、點擊正常、滑鼠移動全無（hover／游標樣式／tooltip 死）」的狀態；Cmd-Tab 回前景不會。討論 desktop-hover-dead-after-foreground 判定病灶在 AppKit 對主視窗的滑鼠追蹤區（NSTrackingArea）未重算「游標已進入」狀態，最合理的觸發是面板與主視窗上緣重疊、面板消失後游標未移動即落在主視窗上。此機制未經實機證實，因此本設計不押單一修法，而是以使用者的重現路徑為裁判，依序試三步。

相關人：apps/desktop 的使用者（跑 SDD 的開發者、PO、PM）；受影響邊界只有 Tauri 殼（src-tauri）與前端 tray 接線，speclink-desktop-core、speclink-core、CLI 不動。

## Goals / Non-Goals

**Goals:**

- 面板→主視窗的每一條交接路徑（open-change、open-discussion、open-app、open-recovery、open-server-settings、reauthenticate、add-project、open-project-settings、open-settings）回前景後，主視窗的 hover、游標樣式與 tooltip 立即可用。
- 修法最小：三步依序試，第一個讓重現路徑不再發作的就收，之後的步驟不做。
- 保留既有面板行為：不搶焦點、失焦自動收合、Cmd-Tab 回前景不受影響。

**Non-Goals:**

- 不修「主視窗不是 key window」（Esc 測試已推翻）。
- 不改前端 Radix／React 層。
- 不改 macOS 系統匣預設樣式。
- 不追 AppKit 確切機制、不在本變更內提上游 issue。
- 非 macOS 平台無面板，行為不變、不動。

## Decisions

### D1. 三步依序、以重現路徑為裁判

依 A→B→C 順序實作，每步做完由使用者依 Implementation Contract 的重現路徑實機驗收；通過即停在該步，未做的步驟在 tasks.md 以「上一步已通過、本步不執行」勾銷並註明。理由：靜態讀碼已無法縮小根因，繼續猜的成本高於實機試修；三步互不衝突、成本遞增、貼病灶程度遞增。

替代：三步一次全上——被否決，會留下不知道哪一步有效的補丁，日後升級 tao／wry 時無從精簡。

### D2. 步驟 A：面板先收再喚起主視窗，用冪等的隱藏命令

apps/desktop/src/tray.ts 所有會喚起主視窗的動作，在 openMainWindow 之前先呼叫新的 Tauri command `hide_tray_panel`（apps/desktop/src-tauri/src/lib.rs 單行委派到 panel.rs 的 hide；非 macOS 回 Ok 無事）。理由：交接當下不再有第二個視窗（面板）與主視窗同拍互踩；面板可能已因失焦先收合，所以命令必須冪等（已隱藏再 hide 無事）。

替代：重用既有 toggle_tray_panel——被否決，toggle 在面板已收合時會把它重新打開，交接時序不可靠。

### D3. 步驟 B：Rust 端「顯示並聚焦主視窗」命令取代前端 setFocus（未執行：A 已通過）

新增 Tauri command `focus_main_window`（lib.rs 單行委派；實作放 panel.rs 的 macOS 視窗交接區段，非 macOS 委派回既有 show＋set_focus），順序固定為：activateIgnoringOtherApps → makeKeyAndOrderFront → 把主視窗的 WKWebView 設回 first responder（對齊 wry 建視窗時的做法）。apps/desktop/src/tray.ts 的 openMainWindow 改為 unminimize → invoke focus_main_window。理由：tao 的順序在 app 未啟用時會讓 makeKey 被 AppKit 延後到啟用那一拍，與面板收合同拍；先啟用再 makeKey 讓交接有明確先後。

### D4. 步驟 C：交接後對主視窗重算追蹤區（未執行：A 已通過）

在 focus_main_window 尾端（或 A 通過後獨立加在 hide_tray_panel 之後）對主視窗 contentView 呼叫一次 invalidateCursorRectsForView，並對 WKWebView 呼叫 updateTrackingAreas；兩者皆為 AppKit 公開 API、無視覺副作用。理由：最貼「追蹤區未重算」的病灶；列為最後一步是因為它是補丁、不是修因。

替代：以微幅 setSize 觸發 resize 重算——被否決，會閃動且動到視窗尺寸持久化。

### D5. 驗證分兩層

- 自動：apps/desktop/src/__tests__/tray.test.ts 斷言交接動作的呼叫順序（A：hide_tray_panel 先於 openMainWindow；B：openMainWindow 呼叫 focus_main_window 而非 setFocus）。
- 手動：使用者在本機安裝的開發版上走重現路徑，這是唯一能裁定「修好了」的證據，以 [M] 任務承載。

## Implementation Contract

**行為**：使用者於 macOS 走以下路徑後，主視窗的滑鼠互動完整——

1. 主視窗在背景 5～10 秒。
2. 點系統匣圖示開面板。
3. 點面板中的一個變更列。
4. 主視窗顯示到前景、該變更的詳情抽屜開啟：此時滑鼠移到抽屜內按鈕上，hover 樣式亮、游標樣式隨元素改變。
5. 點抽屜灰底關閉抽屜：滑鼠移到看板卡片上，hover 亮、tooltip 出現。

同一契約適用於 open-app（「開啟 Speclink」）與 open-discussion 路徑。

**介面**：

- Tauri command `hide_tray_panel`：無參數、回 Result<(), String>；面板不存在或已隱藏時回 Ok；非 macOS 回 Ok 無事。
- Tauri command `focus_main_window`（僅步驟 B 以後存在；**未建立**，A 已通過）：無參數、回 Result<(), String>；找不到主視窗回 Err 單行訊息；非 macOS 委派既有 show＋set_focus。
- 前端 openMainWindow 的呼叫順序契約：A 通過時為 hide_tray_panel → unminimize → show → setFocus；B 通過時為 hide_tray_panel → unminimize → focus_main_window。**實際落地為 A 的順序。**

**失敗模式**：hide_tray_panel 失敗靜默（不阻斷喚起主視窗）；focus_main_window 失敗記 console.error、不彈窗，主視窗至少仍以既有 show＋setFocus 顯示。

**驗收**：

- `npm test -w apps/desktop` 通過，且 tray.test.ts 含順序斷言。
- 使用者以 scripts/desktop/desktop-install.mjs --install 安裝開發版後，走上述五步，第 4 與第 5 步 hover 皆正常；Cmd-Tab 回前景仍正常；面板「不搶焦點且失焦自動收合」情境仍成立。

**範圍**：只動 apps/desktop/src/tray.ts、apps/desktop/src/__tests__/tray.test.ts、apps/desktop/src-tauri/src/lib.rs、apps/desktop/src-tauri/src/panel.rs 與 tray-status-menu 的 delta spec；不動 packages/ui、apps/desktop/core、任何 crate。

## Risks / Trade-offs

- [三步都無效] → 記錄實測結果於 tasks.md，變更改走 discuss 重開，帶著「A／B／C 皆無效」的證據往 tao／wry 上游追；不硬塞第四種補丁。
- [自動測試只能證明順序、證明不了 hover] → 手動 [M] 驗收是硬條件，quality 站在非 [M] 任務完成後即可跑，archive 等 [M] 勾完。
- [跨平台] → 新命令在非 macOS 皆為無事或委派既有行為；Windows／Linux 無面板，tray.test.ts 以 macOS 與非 macOS 兩組 deps 斷言不變。
- [回歸對照] → 不動 CLI 與 golden；desktop 只跑 `npm test -w apps/desktop`；改 protocol struct 才需跑 cargo test -p speclink-desktop，本變更不改。
- [speclink-desktop 測試環境] → 動到 src-tauri 時 cargo build -p speclink-desktop 需 sidecar 與 server-web dist；tasks 只要求 cargo check 通過、由 desktop-install 腳本負責完整建置。

## Migration Plan

無資料或設定遷移。使用者更新到含此修正的版本即生效；無 rollback 需求，若要退回只需還原 tray.ts 的呼叫順序。

## Open Questions

- 若 A 就通過，B 與 C 的命令不建立——tasks.md 的對應任務以「不執行」勾銷並註明依據（第幾次實測）。
  **已解**：第一次實測（2026-09-16，使用者於本機安裝 worktree 建置的開發版、引擎 v1.37.0）A 通過——三條路徑（變更列、「開啟 Speclink」、討論列）回前景後 hover 皆正常。B（D3）與 C（D4）未執行，focus_main_window 未建立；tasks.md 的 2.x 與 3.x 以「A 通過，不執行」勾銷。

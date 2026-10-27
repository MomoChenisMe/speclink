## Why

macOS 桌面 app 的系統匣有兩個使用者可見的缺陷（源自討論 tray-badge-and-recovery-buttons 的兩張實機截圖）：

1. **徽章數字殘留不更新**：系統匣圖示旁的進行中變更數徽章卡在殘留值——作用中 workspace 的進行中數已為 0 仍顯示 1、切換 workspace 不隨之調整、作用中 workspace 處於連線失敗（error）狀態時本應整個隱藏卻仍顯示。此行為違反 tray-status-menu 正典規格「macOS 進行中數文字徽章」需求（作用中專案的進行中變更數、隨資料變動更新、0 時不顯示）。
2. **連線失敗恢復卡按鈕文案換行**：次要按鈕「在 Speclink 中查看問題」在 320px 面板的半欄寬按鈕內放不下，折成兩行，視覺擁擠。

目標使用者是透過 Speclink 桌面 app（macOS、面板樣式系統匣）管理多個 workspace 的開發者；情境為日常掛在選單列監看變更進度，以及遠端 workspace 連線失敗時經恢復卡自救。徽章是使用者不開面板就能看到的唯一狀態訊號，殘留錯值會直接誤導。

## What Changes

- **移除系統匣數字徽章**（方向變更，使用者裁定 2026-07-23）：原計畫修復徽章更新；實機確診找到孤兒 tray icon 凍結舊值的根因並以固定 id 清理修復後，使用者實機仍觀察到數字不隨 workspace 切換更新，裁定整個移除——系統匣圖示不再顯示任何數字，進行中數由面板分區計數承載。tray 仍以固定 id 建立並於初始化前移除同 id 孤兒（此修復保留，防 icon 殘留）。
- **縮短恢復卡按鈕文案**：繁中「在 Speclink 中查看問題」改為「查看問題」，英文 "View issue in Speclink" 改為 "View issue"；跳轉語意由按鈕既有的外開箭頭圖示承載。排版結構（主鈕全寬＋兩顆次鈕各半欄）不動；「需要重新登入」狀態共用同一按鈕區，一併受惠。
- **回歸測試**：既有 tray 測試補上「徽章隨資料變動與 workspace 切換更新」「error 態隱藏」的失敗案例先行（TDD），再修實作。
- **移除分頁「待收尾數」徽章**（同批裁定）：主視窗 workspace 分頁上的待收尾數字同樣不隨狀態更新，一併移除——分頁只顯示名稱與狀態圖示；啟動時的背景分頁查詢降為路徑有效性探測（保留失效轉錯誤態行為）。
- **規格對齊**：tray-status-menu「macOS 進行中數文字徽章」需求以 REMOVED delta 移除（Reason 記錄實機確診與裁定緣由）；desktop-config「專案分頁列存於 app 本機」以 MODIFIED delta 移除徽章語句與場景、改明分頁不顯示計數。

**相容性影響**：純 apps/desktop 前端變更；不動任何 CLI 指令、人眼輸出或 --json 輸出，不影響 parity／golden 回歸對照。文案變更僅涉桌面 UI 字串，無遷移成本。

## Non-Goals

- 不做徽章的替代呈現（例如 tooltip 帶數字、icon 變形、跨 workspace 加總）——數字資訊一律由面板分區計數與看板欄計數承載，系統匣與分頁保持無數字。
- 不改恢復卡排版結構——次要按鈕改直排（卡片變高）與縮小字級（可讀性差、治標）皆已於討論中否決。
- 不動系統匣 tooltip（現為固定 "Speclink"）——是否補上徽章說明文字已明確遞延，本次不做。
- 不動非 macOS 平台——徽章本為 macOS 限定行為，Windows／Linux 無此徽章的現狀不變。
- 不涉及任何 Rust crate（speclink-core、speclink-cli 等）——影響面僅 apps/desktop 前端（TypeScript／React）；Tauri Rust 層（apps/desktop/src-tauri）現階段預期不動，除非確診結果指向 Rust 側。

## Capabilities

### New Capabilities

（無——不新增能力）

### Modified Capabilities

- `tray-status-menu`: 移除「macOS 進行中數文字徽章」需求（REMOVED）——系統匣不再顯示數字文字；孤兒 tray icon 清理（固定 id）記入 Migration。按鈕文案縮短不改變恢復卡動作集合（正典以動作語意而非字面標籤描述），不在 delta 範圍。
- `desktop-config`: 「專案分頁列存於 app 本機」移除待收尾數徽章語句與場景（MODIFIED）——分頁不顯示計數徽章；背景分頁啟動查詢改述為路徑有效性探測（失效轉錯誤態行為保留）。

## Impact

- Affected specs: tray-status-menu（REMOVED：macOS 進行中數文字徽章）、desktop-config（MODIFIED：專案分頁列存於 app 本機）
- Affected code:
  - Modified: apps/desktop/src/tray.ts、apps/desktop/src/i18n/messages.ts、apps/desktop/src/tabs.ts、apps/desktop/src/store.ts、apps/desktop/src/components/ProjectTabs.tsx、apps/desktop/src/__tests__/tray.test.ts、apps/desktop/src/__tests__/trayPanel.test.tsx、apps/desktop/src/__tests__/tabs.test.ts、apps/desktop/src/__tests__/store.test.ts、apps/desktop/src/__tests__/App.test.tsx
  - New: 無
  - Removed: 無

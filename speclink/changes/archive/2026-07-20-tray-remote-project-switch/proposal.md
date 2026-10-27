## Why

macOS 系統匣面板的專案 tab 條上，點擊 remote 專案分頁完全沒有反應，無法切換過去；非 macOS 原生選單同根因（切換動作對 remote 分頁等於以空字串開專案，錯誤只吐在可能隱藏的主視窗）。目標使用者是同時開啟多個專案分頁（含 remote workspace）的桌面 app 開發者，情境為透過系統匣在專案間快速原地切換、不喚起主視窗——remote 分頁在此路徑上完全失能。

**根因**：tray 建置當時 remote 分頁僅存在於型別，快照層將 remote 分頁的 root 設為空字串（tray 接線註明「remote 本刀無建構路徑」）；面板動作 handler 以 root 非空作為執行前提，空字串被靜默吃掉。此為已知缺口如期引爆，非回歸。store 既有的 activateTab 動詞早已完整承載 remote 與 local 兩型分頁的切換語意（remote：有 session 直切、重啟後重走 handshake；local：目錄失效轉分頁錯誤態），只是 tray 未接上。workspace-session 規格本就要求 tray 一律經 locator key 識別、不得以裸 root 字串比對——切換動作沿用 root 是殘留違例。

## What Changes

- tray 的專案切換動作（macOS 面板 tab 條與非 macOS 原生選單皆然）改以 locator key 呼叫 store 既有的 activateTab，取代現行以 root 路徑呼叫 openProjectAt 的路徑；修復後點擊 remote 專案分頁即完成原地切換，local 分頁行為不變。
- tray 快照的 root 欄位退場：切換把手統一為 locator key，快照不再攜帶僅 local 有意義的 root。
- 面板動作事件的 open-project 載荷從 root 改為 locator key；面板 tab 點擊回呼同步改傳 key。
- tray 與面板的單元測試同步改寫斷言（切換呼叫 activateTab 帶 key），並補 remote 分頁切換案例。

## Non-Goals

- 不在 tray／面板內新增切換失敗的錯誤 UI：失敗靜默為使用者明確裁定——錯誤沿用看板既有的分頁錯誤態（tabErrors）呈現，面板維持薄渲染層定位。
- 不為 remote 單獨加分支、local 續走 openProjectAt（已否決）：兩條切換路徑會語意分歧，且 openProjectAt 的初始化對話框語意不屬於「切換既有分頁」。
- 不動 openProjectAt 本身：它仍服務「加入專案」資料夾選擇器路徑（面板 add-project 動作），該路徑不在本變更範圍。
- 不併入進行中的 server-scope-read-api 變更（其 artifacts 全文未涉 tray）。
- 不改 tray 的其他區段（生命週期分區、討論區、動作區）與樣式。

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `tray-status-menu`: 「選單專案切換」與「面板樣式（macOS）」的切換語意明確涵蓋 remote 專案分頁——切換把手為 locator key（非 root 路徑），點擊 remote 分頁與 local 分頁同樣完成原地切換；remote 切換失敗沿用看板既有分頁錯誤態、tray 不另設錯誤呈現。

## Impact

- 影響的 crate／套件：純桌面前端（apps/desktop 的 React/TypeScript 層）；speclink-core、speclink-cli 零改動，無 CLI 指令、輸出、設定欄位或技能影響，不觸及回歸對照基線。
- Affected specs: `tray-status-menu`（delta：MODIFIED 選單專案切換、面板樣式（macOS））
- Affected code:
  - Modified: apps/desktop/src/tray.ts、apps/desktop/src/panel/TrayPanel.tsx、apps/desktop/src/panel/main.tsx、apps/desktop/src/__tests__/tray.test.ts、apps/desktop/src/__tests__/trayPanel.test.tsx
  - New: （無）
  - Removed: （無）

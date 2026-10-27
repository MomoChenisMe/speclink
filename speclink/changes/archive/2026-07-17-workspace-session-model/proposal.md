## Summary

Desktop 分頁身分從「本機 filesystem root 字串」重構為 WorkspaceLocator／WorkspaceSession——每個 session 自帶 dataSource／settings／events，App 不再注入單一全域 DataSource，Rust 側的 current-root 單例（AppState 的 root Mutex 與隨切換重掛的 watcher）改為逐呼叫收 root；純重構，既有本地行為與 UI 逐位元不變。

## Motivation

架構 §14 Phase 3 第 1 項與 roadmap Phase 3 gate「Workspace tab 不再只以 root path 作 identity」的第一子刀：remote spec-only 與 remote + checkout 三形態（架構 §10.4）都掛在 WorkspaceSession 這面承重牆上，後續 connection-registry、RemoteDataSource、chooser onboarding 全部以它為插槽。現況還有一類潛在 race：分頁切換靠改寫 Rust 全域 current root，前一個分頁尚未完成的 invoke 會落在新 root 上結算——逐呼叫傳 root 直接消滅這一類。desktop-core 早已無狀態逐呼叫收 root（query/manage 皆為帶路徑函式），本刀只是把同一原則推到 Tauri command 與前端接線層。

## Proposed Solution

- 新增 session 模組（apps/desktop/src/session.ts）：依架構 §10.4 定義 WorkspaceLocator（local｜remote 兩變體；本刀 remote 僅型別宣告、無任何建構路徑）、locator key 函式（分頁去重與持久化的身分）、WorkspaceSession（id、locator、descriptor、dataSource、settings、events）與 createLocalSession(root) 工廠——把 root 綁進 dataSource／settings 的閉包。
- 分頁模型 locator 化：ProjectTab 的身分欄位由 root 字串改為 locator；localStorage 持久化升 v2（含 locator），舊 v1 格式（root＋name）靜默遷移為 local locator，壞 JSON 歸零分頁的既有行為保留。
- Tauri commands 逐支加 root 參數：刪除 AppState 的 root Mutex；open_project 退化為純探測（不再改寫全域）；watcher 改由顯式 watch_workspace(root) 命令重掛（仍單一 watcher、跟隨活躍 session——行為凍結），workspace-changed 事件 emit 帶上被監看的 root，session 的 events 來源以自身 locator 過濾。
- store／App 接線：createStore 不再收全域 dataSource，改收 session 工廠；資料載入一律走活躍 session 的 dataSource（單活躍載入語意不變）；tray 選單與 panel 的分頁項改以 locator key 識別。
- packages/ui 零改動：SpeclinkDataSource 介面與所有 UI 元件不動——session 模型屬 apps/desktop 宿主層。

## Non-Goals

- 不含任何 remote 能力：無 connection、無登入、無 RemoteDataSource——remote locator 變體只有型別，建構路徑不存在（後續刀）。
- 不做多 session 併發資料載入或背景分頁快取：維持「單一活躍 session 載入」的現行語意，多 session 併發屬 remote-data-source 之後的刀。
- 不動 packages/ui 的 DataSource 介面與元件。
- 不動 desktop-core（apps/desktop/core）——它已是逐呼叫收 root 的正確形狀。
- 不做 per-session 多 watcher：watcher 維持單一、跟隨活躍 session。

## Impact

- 相容性影響：Tauri command 簽名屬 app 內部介面（無外部消費者）；localStorage 格式升版含 v1 靜默遷移，使用者分頁列與活躍分頁跨升級保留。
- Affected specs: `workspace-session`（新增）
- Affected code:
  - New: apps/desktop/src/session.ts、apps/desktop/src/__tests__/session.test.ts
  - Modified: apps/desktop/src/tabs.ts、apps/desktop/src/store.ts、apps/desktop/src/App.tsx、apps/desktop/src/main.tsx、apps/desktop/src/adapter/tauriDataSource.ts、apps/desktop/src/adapter/workspace.ts、apps/desktop/src/tray.ts、apps/desktop/src/panel/TrayPanel.tsx、apps/desktop/src/components/ProjectTabs.tsx、apps/desktop/src-tauri/src/lib.rs、apps/desktop/src-tauri/src/watch.rs、apps/desktop/src/__tests__/tabs.test.ts、apps/desktop/src/__tests__/tauriDataSource.test.ts、apps/desktop/src/__tests__/store.test.ts、apps/desktop/src/__tests__/workspace.test.ts、apps/desktop/src/__tests__/App.test.tsx、apps/desktop/src/__tests__/tray.test.ts、apps/desktop/src/__tests__/trayPanel.test.tsx、apps/desktop/src/__tests__/projectTabs.test.tsx
  - Removed: 無

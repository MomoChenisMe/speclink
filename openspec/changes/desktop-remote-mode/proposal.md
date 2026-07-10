> **Roadmap**: 四情境預設 GUI 工具矩陣原第 ③ 刀重切為三（討論 sdk-storage-seam-and-remote-desktop ＋ server-auth-and-push-transport）：① speclink-sdk-and-store-seam（核心）、② web-server-postgres（範例 server）、**③ desktop-remote-mode（本刀＝消費者）**。
> **依賴**: ① 的 server 運算端點與推播宣告欄；② 的 server 為連線目標；復用 crates/speclink-remote（RemoteClient、auth）與 crates/speclink-core（resolve_mode、write_remote_section）。

## Why

三刀重切後本刀（③）是消費者：desktop app 新增遠端模式，連 ② 的 server（或任何整合者依 verb-contract 建的 server）成為遠端 GUI。使用者裝 desktop 連遠端即得看板／文件／討論，無需瀏覽器零安裝版。

遠端模式下 desktop 顯示的資料全來自遠端（卡片中的 spec 資料、config.yaml），只有 .speclink.yaml 本地——故 desktop 有一定量改動：瀏覽路徑對前端透明，但**設定面分叉**、部分操作降級或改走端點。同時 desktop 與 CLI 共讀 .speclink.yaml，設 remote 區段即同時翻兩者為遠端。

## What Changes

- **desktop-core 後端替換**：`apps/desktop/core` 的各 `*_at` 委派函式依 `Workspace::resolve_mode()` 分流——`Fs` 走今日 `FsStore`；`Remote` 走既有 `RemoteClient` 打端點、重塑為 Tauri 命令既回傳的 `serde_json::Value`。前端、`SpeclinkDataSource`、`tauriDataSource` 對瀏覽路徑無感。
- **設定面分叉**：config.yaml 來自遠端且唯讀（`GET /config`；契約 PUT config 屬 host-admin）——設定頁 config 分頁於遠端模式轉唯讀顯示遠端值；.speclink.yaml 本地 ＋新增遠端卡（server URL／repo →remote 區段經 `write_remote_section`；PAT →使用者層級憑證經 `save_token_at`，絕不入 repo、`SPECLINK_TOKEN` 覆蓋）。
- **遠端模式操作**：`validate`／`analyze`／`drift` 走 ① 的 server 運算端點（非本地算）；`archive` 走 archive 端點；`setTaskDone` 走 task-done 端點；`moveTask`／`setAllTasks` 以 `If-Match` 重寫 tasks.md；`deleteChange`（discard）遠端不支援並明白回報；`reorderCard`（board_rank）不寫遠端；封存頁於遠端停用（v1 無列舉端點）。
- **即時刷新**：遠端模式地基 ＝輪詢；讀 server 宣告欄 `events:{url,transport}` 發現推播通道，支援即連（本刀實作 SSE client），不支援／無宣告則退回輪詢；WebSocket client 遞延（發現機制傳輸無關已備）。

## Non-Goals

- 不含 server（② web-server-postgres）、不含 SDK／契約修訂（① speclink-sdk-and-store-seam）。
- 不含 desktop 的 WebSocket client（遞延——發現機制已備，未來純加法）。
- 不做瀏覽器 web GUI（遞延）。
- 不改動 fs 模式行為（設定頁寫 config.yaml 於 fs 模式不變）。

## Capabilities

### New Capabilities

- `desktop-remote-mode`: desktop app 遠端模式——後端替換複用 RemoteClient、設定面分叉（config 唯讀、.speclink.yaml 遠端卡 ＋PAT）、遠端操作（運算走端點、寫入帶 If-Match、降級明確）、即時刷新（輪詢地基 ＋SSE client ＋宣告發現）。

### Modified Capabilities

(none)

## Impact

- Affected specs: desktop-remote-mode（新）
- Affected code:
  - Modified: apps/desktop/core/src/query.rs、apps/desktop/core/src/manage.rs、apps/desktop/core/src/verbs.rs、apps/desktop/core/src/discussions.rs（各 `*_at` 依 resolve_mode 分流）、apps/desktop/core/Cargo.toml（加 speclink-remote 依賴）、apps/desktop/src-tauri/src/lib.rs（新命令：寫 remote 區段、存／清 PAT）、apps/desktop/src/views/SettingsView.tsx（遠端卡 ＋config 唯讀分叉）、apps/desktop/src/adapter/workspace.ts（遠端設定命令）
  - New: apps/desktop/src 的 SSE client 訂閱與輪詢地基、遠端設定 UI（自建表單元件）
- 依賴: ① 的 server 運算端點與推播宣告欄；② 為連線目標
- 復用: crates/speclink-remote（RemoteClient、auth）、crates/speclink-core（resolve_mode、write_remote_section、remove_remote_section）

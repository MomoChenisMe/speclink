## Context

現況三個「單一全域」互相纏繞：(1) 前端 App 啟動時注入一個全域 SpeclinkDataSource（createTauriDataSource 無參數，command 不帶 root）；(2) Rust 側 AppState 持 root Mutex，所有 command 讀它決定作用專案，分頁切換＝open_project 改寫全域；(3) watcher 單例隨切換重掛、emit 無 payload 的 workspace-changed。desktop-core 本身已是無狀態逐呼叫收 root（list_changes_at(&root) 等），單例只存在於 Tauri 接線層。分頁模型（tabs.ts 的 ProjectTab）與持久化（localStorage 的 speclink.projectTabs，v1：root＋name＋activeRoot）皆以 root 字串為身分。架構 §10.4 要求分頁升格為 WorkspaceSession（id、locator、descriptor、dataSource、settings、events），是 Phase 3 全部後續刀的插槽。

## Goals / Non-Goals

**Goals:**

- 分頁身分＝WorkspaceLocator（roadmap Phase 3 gate 第 2 條），持久化升 v2 並靜默遷移 v1。
- 每個 session 自帶 dataSource／settings／events；Rust 側 current-root 單例消滅，command 逐呼叫收 root。
- 行為凍結：UI 可觀察行為（分頁列、看板、設定、tray、外部變更即時反映、重啟恢復）與現狀一致；既有 vitest 全綠。

**Non-Goals:**

- 無任何 remote 能力（remote locator 僅型別、無建構路徑）；無多 session 併發載入與背景快取（維持單活躍載入）；無 per-session 多 watcher；packages/ui 與 apps/desktop/core 零改動。

## Decisions

### 決策 1：locator 與 key——分頁身分的單一來源

WorkspaceLocator 依 §10.4 逐字採用：local 變體 kind:"local" 帶 root；remote 變體 kind:"remote" 帶 connectionId／projectId／repoId／checkoutRoot?——本刀即宣告完整型別（含 remote），使持久化 schema 與 key 規則跨後續刀穩定，但 remote 無任何建構路徑。locatorKey(locator) 為分頁去重、持久化 activeKey、tray 選單識別的唯一身分函式：local 為 local:{root}、remote 為 remote:{connectionId}/{projectId}/{repoId}。替代案「本刀只宣告 local、remote 後補」被否：後補會再動一次持久化 schema 與 key 函式，白付一次遷移。

### 決策 2：persisted v2 與 v1 靜默遷移

localStorage 鍵名不變（speclink.projectTabs），payload 升 v2：{version:2, tabs:[{locator, name}], activeKey}。讀取規則：有 version:2 依 v2 解析；無 version 欄位且形如 v1（tabs 條目有 root 字串）則逐條映射為 local locator、activeRoot 映射為 local key——靜默遷移、下次寫入即 v2；壞 JSON 或不識別形狀歸零分頁（既有行為保留）。不做反向降級。

### 決策 3：session 物件與 createLocalSession 工廠

WorkspaceSession＝{id, locator, descriptor, dataSource, settings, events}（§10.4 全欄位）。descriptor 承接現有顯示資訊（name、badge）。createLocalSession(root, deps)：dataSource＝createTauriDataSource(root)（每支 invoke 帶 root）；settings＝現 WorkspaceAdapter 的設定面 root 綁定版（readSettings(root) 等，型別名 WorkspaceSettingsProvider）；events＝訂閱 workspace-changed 且以自身 root 過濾 payload 的事件來源。deps 允許測試注入假 invoke。id 以 locatorKey 衍生（本刀不需要獨立於 locator 的 session id——同 locator 同 session；獨立 id 留給多視窗需求出現時）。

### 決策 4：Rust command 逐呼叫收 root、單例消滅

所有讀寫 command（list_changes、status、document、search、settings 讀寫、動詞等）簽名加 root: PathBuf，直通 desktop-core 的 *_at 函式；AppState 的 root Mutex 刪除。open_project 退化為純探測（回 project｜uninitialized payload，不再改寫任何全域）；current_project 命令刪除——啟動時的活躍專案由前端持久化 v2 的 activeKey 決定（含首啟以 CLI 引數／預設目錄開啟的既有路徑，改為前端顯式 openProjectAt；前端據新增的純讀 startup_dir 命令取得啟動 cwd——無狀態、非可變全域，凍結「自專案目錄啟動即自動開啟」的既有行為）。分頁切換的 race 類型（切換後前一分頁 in-flight invoke 落在新 root）隨全域消失而消滅。

### 決策 5：watcher 顯式重掛、事件帶 root

watcher 維持單一實例、跟隨活躍 session（行為凍結），但重掛改由顯式 watch_workspace(root) command 觸發（open_project 不再有此副作用）；workspace-changed 事件 emit 帶 payload root 字串，session 的 events 來源以自身 root 過濾——非活躍 session 的訂閱者天然收不到（watcher 只掛在活躍 root），活躍者收到後觸發既有 reload 路徑。「監看不可用僅失去自動刷新、不崩潰」的既有語意不變。替代案「per-session 多 watcher」為後續多 session 併發刀的事，本刀不做。

### 決策 6：store 收 session 工廠、單活躍載入語意不變

createStore 的注入參數由 (dataSource, workspace?) 改為 session 工廠與 workspace 探測面；store 持 sessions（以 locatorKey 為鍵）與 activeKey，資料載入（reload、詳情、動詞、任務操作）一律經活躍 session 的 dataSource；openProjectAt＝純探測 → upsert session 與分頁 → 設 activeKey → watch_workspace → reload。tray.ts 與 TrayPanel 的分頁項識別由 root 改 locatorKey（顯示文字不變）。既有測試斷言語意不變——僅注入方式與身分欄位跟著改。

## Implementation Contract

- 行為凍結面（手動＋自動雙驗）：既有 vitest 套件（tabs、store、workspace、App、tray、trayPanel、settingsView）全綠，斷言語意不得弱化；packages/ui 零 diff；cargo build --release -p speclink-desktop 成功。
- 新增單元測試：locatorKey 去重與格式、v2 讀寫、v1→v2 遷移（含 activeRoot→activeKey）、壞 JSON 歸零、createLocalSession 以假 invoke 斷言每支呼叫皆帶正確 root、事件來源以 root 過濾。
- GUI 鐵律（真實視窗）：兩個專案開分頁互切、設定頁讀寫、外部改檔（CLI 動 openspec/）看板秒級反映、以 v1 格式預置 localStorage 後啟動驗證分頁與活躍分頁完整遷移、tray panel 切換專案、重啟恢復分頁。操作前確認使用者未在使用螢幕。
- 邊界：v1 遷移只認 root 為字串的條目，其餘丟棄；watch_workspace 對無 openspec/ 的 root 沿用既有「監看不可用僅失去刷新」語意；open_project 對同一路徑重複呼叫冪等（純探測）。

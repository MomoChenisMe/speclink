## Context

三刀重切後本刀（③）是消費者：desktop app 新增遠端模式，連 ② 的 server 成為遠端 GUI。

現況（決定取捨的既有事實）：
- **desktop 今日 fs-only**：前端 → Tauri IPC → `apps/desktop/core` 的 `*_at(&root,…)` → `FsStore`；`SpeclinkDataSource`（`packages/ui/src/adapter.ts`）純 TS 介面、無 HTTP 知識，唯一實作 `tauriDataSource.ts`。
- **模式解析與遠端設定基建全在 Rust 現成**：`Workspace::resolve_mode()`（以 `.speclink.yaml` `remote:` 區段判 fs/remote）、`init::write_remote_section`/`remove_remote_section`、PAT 憑證 `crates/speclink-remote/src/auth.rs`（origin→token、0600/ACL、`SPECLINK_TOKEN` 覆蓋、絕不入 repo）。
- **RemoteClient 既有**：`crates/speclink-remote` 覆蓋 verb-contract 端點＋`translate_status` 錯誤翻譯。① 另加 analyze/validate/drift 運算端點與 `events` 宣告欄。
- **設定頁現況**：`SettingsView.tsx` 經 `WorkspaceAdapter`（`workspace.ts`）寫 config.yaml（`writeWorkflowConfig`/`writeWorkflowContext`/`writeWorkflowRules`）與 .speclink.yaml（`writeAppTools`）。
- **推播分層既定**：client 地基＝輪詢；推播可選、可宣告、傳輸無關（server-auth-and-push-transport 討論）。

## Goals / Non-Goals

**Goals:**
- desktop-core 依 resolve_mode 後端替換：Remote 走 RemoteClient；瀏覽路徑對前端透明。
- 設定面分叉：config.yaml 遠端唯讀顯示；.speclink.yaml 本地＋遠端卡（URL/repo＋PAT）。
- 遠端操作：validate/analyze/drift 走 ① 的 server 運算端點；moveTask/setAllTasks 以 If-Match 重寫 tasks；deleteChange/reorderCard/封存頁降級明確。
- 即時刷新：輪詢地基＋讀宣告欄發現＋SSE client（WS 遞延）。

**Non-Goals:**
- 不含 server（②）、SDK/契約（①）、瀏覽器 GUI、desktop WS client。
- 不改 fs 模式行為。

## Decisions

### D1: 後端替換複用 RemoteClient

`apps/desktop/core` 的各 `*_at` 委派函式依 `Workspace::resolve_mode()` 分流：`Fs` 走今日 `FsStore`；`Remote(conn)` 以 `RemoteClient` 打端點、重塑為 Tauri 命令既回傳的 `serde_json::Value`。前端、`SpeclinkDataSource`、`tauriDataSource`、React 元件對瀏覽路徑一位元不動。`apps/desktop/core` 新增 `speclink-remote` 依賴。

替代方案：前端 TS httpDataSource（**駁回**——Tauri WebView 載不了 @speclink/engine 原生模組、會重寫 RemoteClient、繞過既有 IPC 架構；瀏覽器宿主是 Non-Goal，屆時才建）。

### D2: 設定面分叉（config 遠端唯讀、.speclink.yaml 遠端卡）

遠端模式下 config.yaml 資料來自 `GET /config` 且唯讀（契約 PUT config 屬 host-admin）——`SettingsView` 的 config 分頁轉唯讀顯示遠端值，`WorkspaceAdapter` 的 config 寫入方法於遠端模式停用（無寫入標的）。.speclink.yaml 頁籤新增遠端卡：URL/repo 經 `write_remote_section` 寫入 remote 區段（保留其他鍵）、PAT 經 `save_token_at` 存使用者層級憑證（絕不入 repo、`SPECLINK_TOKEN` 覆蓋）；新增對應 Tauri 命令。遠端卡表單一律用自建元件（禁裸原生）。

替代方案：遠端也讓 desktop 改 config（**駁回**——需契約外 write-config 端點＋權限，範圍爆炸且違契約 config 唯讀）。

### D3: 遠端操作降級與運算走端點

遠端模式各操作明確界定：`validate`/`analyze`/`drift` 呼叫 ① 的 server 運算端點（非本地算，team 一致性）；`archive` 走 archive 端點；`setTaskDone` 走 task-done 端點；`moveTask`/`setAllTasks` 以 `get_artifact`→重排/全改→`put_artifact` 帶 `If-Match` 寫回（stale 回報衝突不覆蓋）；`deleteChange`（discard）遠端不支援並回報；`reorderCard`（board_rank）不寫遠端（本地狀態或停用）；封存頁遠端停用（v1 無列舉端點）。

替代方案：遠端 validate/analyze 仍 desktop 本地算（**駁回**——team 版本歧異使結果分裂；且與 ① 的 server 運算決策衝突）。

### D4: 即時刷新輪詢地基加宣告發現加 SSE client

遠端模式地基＝輪詢（對任何 server 都同步、永不鎖死）；讀 server metadata 的 `events:{url,transport}` 宣告欄發現推播通道，`transport` 為 `sse`（本刀支援）即連 SSE client 收 invalidate 事件觸發 refresh，無宣告/不支援之 transport 則退回輪詢；WebSocket client 遞延（發現機制傳輸無關已備，未來純加法）。

替代方案：寫死連 SSE（**駁回**——鎖死非 SSE server，違傳輸無關發現）；只輪詢不做 SSE（**駁回**——使用者要即時刷新）。

## Implementation Contract

#### desktop-remote-mode（新 capability）

- **可觀察行為**：`.speclink.yaml` 含 `remote:` 區段時 desktop 以遠端模式啟動，看板/文件/spec/討論資料源自遠端 server，前端 UI 與 fs 模式一致；url 兩處（section／`SPECLINK_STORE_URL`）皆缺則明確錯誤不退回 fs。
- **設定面**：config 分頁遠端唯讀顯示遠端值、不可寫；.speclink.yaml 遠端卡存 URL/repo（入 remote 區段、保留其他鍵）與 PAT（使用者層級憑證、不入 repo、`SPECLINK_TOKEN` 覆蓋）。
- **遠端操作**：validate/analyze/drift 呼叫 server 運算端點、輸出形狀與 fs 一致；moveTask/setAllTasks 帶 If-Match 寫 tasks，stale 回報衝突不覆蓋；deleteChange/reorderCard/封存頁於遠端明確不可用或不寫遠端。
- **即時刷新**：地基輪詢；讀宣告欄，`transport:"sse"` 即連 SSE client、收 invalidate 即 refresh；無宣告/不支援退回輪詢。
- **介面/資料形狀**：`apps/desktop/core` 的 `*_at` 依 resolve_mode 分流、Remote 重塑為既有 Tauri 命令的 `serde_json::Value`；新增 Tauri 命令：寫 remote 區段、存/清 PAT；前端 `SpeclinkDataSource`/`tauriDataSource` 瀏覽路徑不變。
- **驗收**：(a) desktop-core 遠端分支單元測試（mock RemoteClient，回傳形狀與 fs 分支一致）、`cargo test -p speclink-desktop-core --lib`；(b) 設定測試（遠端卡寫 remote 區段保留其他鍵、PAT 不入 repo、config 遠端唯讀）；(c) **真實視窗驗證**（release exe＋computer-use，遵 GUI 紅線、操作前確認使用者未用螢幕）：設遠端後看板/文件/討論從遠端載入、遠端卡可存、SSE 事件觸發刷新、降級操作 UI 明確；(d) moveTask stale If-Match 回報衝突不覆蓋。
- **In scope**：後端替換、設定分叉、遠端操作/降級、輪詢＋SSE 刷新、PAT 憑證。**Out**：server（②）、SDK/契約（①）、瀏覽器 GUI、WS client。

## Risks / Trade-offs

- **[遠端分支重塑 JSON 形狀，易與 fs 分支漂移]** → 遠端分支單元測試斷言回傳形狀與 fs 分支一致；共用型別。
- **[GUI 改動 jsdom 測不出 pointer/拖曳/hover]** → 真實視窗驗證（release exe＋computer-use）為 GUI 紅線，遵既有機器備忘。
- **[跨平台憑證 Unix 0600 vs Windows ACL]** → 複用既有 `crates/speclink-remote/src/auth.rs`，不另造。
- **[依賴 ①②未先完成則無可連 server／運算端點]** → 依賴序 ①→②→③；③ 測試以 mock RemoteClient／假 server 對打。

## Migration Plan

- 接入：設定頁遠端卡填 URL/repo/PAT → 寫 remote 區段＋存憑證 → desktop 與 CLI 同翻遠端。
- 回滾：移除 remote 區段（`remove_remote_section`）即退回 fs 模式。

## Open Questions

- board_rank 遠端為純本地狀態或完全停用——apply 定案（不寫遠端為硬界）。
- SSE 重連/退避策略——apply 依體感定案。

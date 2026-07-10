## 1. desktop-core 後端替換

- [ ] 1.1 撰寫 desktop-core 遠端分支失敗測試（Red）：`Remote-section presence selects the remote data backend`——`resolve_mode` 為 Remote 時 `*_at` 走 `RemoteClient` 且回傳形狀與 fs 分支一致（mock RemoteClient）、url 兩處皆缺則明確錯誤不退回 fs；測試先紅
- [ ] 1.2 讓 `apps/desktop/core` 的 `*_at` 依 `Workspace::resolve_mode()` 分流（design D1: 後端替換複用 RemoteClient）：Remote 走 `RemoteClient` 並重塑為既有 Tauri 命令的 `serde_json::Value`、`apps/desktop/core/Cargo.toml` 加 `speclink-remote` 依賴，前端與 `SpeclinkDataSource` 瀏覽路徑不動；1.1 測試轉綠
- [ ] 1.3 驗證：`cargo test -p speclink-desktop-core --lib` 遠端分支測試全綠

## 2. 設定面分叉與遠端卡

- [ ] 2.1 撰寫設定失敗測試（Red）：`Settings page splits remote-read-only config from local connection`——遠端模式 config 分頁唯讀顯示遠端值且寫入停用、遠端卡存 `url`/`repo` 入 `.speclink.yaml` `remote:` 區段（保留其他鍵）、PAT 入使用者層級憑證且不入 repo、`SPECLINK_TOKEN` 覆蓋；測試先紅
- [ ] 2.2 實作設定分叉與遠端卡（design D2: 設定面分叉（config 遠端唯讀、.speclink.yaml 遠端卡））：`SettingsView.tsx` config 分頁遠端唯讀、`workspace.ts` config 寫入於遠端停用、遠端卡 UI（自建表單元件、禁裸原生）；新增 Tauri 命令寫 remote 區段（經 `write_remote_section`）與存/清 PAT（經 `save_token_at`）；2.1 測試轉綠
- [ ] 2.3 驗證：設定測試綠，且**真實視窗驗證**（release exe＋computer-use，遵 GUI 紅線、操作前確認使用者未用螢幕）遠端卡可存、config 分頁遠端唯讀

## 3. 遠端操作降級與運算走端點

- [ ] 3.1 撰寫遠端操作失敗測試（Red）：`Remote-mode operations route to endpoints or degrade explicitly`——`validate`/`analyze`/`drift` 呼叫 server 運算端點（非本地算）、`moveTask`/`setAllTasks` 以 `If-Match` 重寫 tasks、`deleteChange` 不支援回報、`reorderCard` 不寫遠端、封存頁停用；測試先紅
- [ ] 3.2 實作遠端操作路徑（design D3: 遠端操作降級與運算走端點）：validate/analyze/drift 打端點、archive/task-done 打端點、moveTask/setAllTasks 讀 tasks→重排→帶 If-Match 寫回、discard/reorderCard/封存頁於遠端明確不可用或不寫遠端；3.1 測試轉綠
- [ ] 3.3 驗證：遠端操作測試綠，且 moveTask 遇 stale `If-Match` 回報衝突而不覆蓋

## 4. 即時刷新輪詢地基與 SSE

- [ ] 4.1 撰寫刷新失敗測試（Red）：`Live refresh uses a polling baseline with advertised push discovery`——地基輪詢對任何 server 保持新鮮、宣告 `transport:"sse"` 即連 SSE client 收 invalidate 觸發 refresh、無宣告/不支援之 transport 退回輪詢不報錯；測試先紅
- [ ] 4.2 實作輪詢地基＋宣告發現＋SSE client（design D4: 即時刷新輪詢地基加宣告發現加 SSE client）：讀 `events` 宣告欄、`sse` 即連 SSE client 訂閱 invalidate→refresh、否則輪詢；4.1 測試轉綠
- [ ] 4.3 驗證：刷新測試綠，且真實視窗驗證他端變動本端自動反映（宣告 sse 時即時、否則輪詢後反映）

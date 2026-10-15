## 1. 重現（Red）

- [ ] 1.1 在 `crates/host/speclink-server/tests/it/web/assets.rs` 新增測試 `detail_pages_serve_the_spa_shell_on_direct_open`：直接 GET `/admin/users/usr_example` 與 `/admin/registry/demo` 時斷言 200 且內容含 `id="root"`；GET `/admin/users/a/b`、`/admin/users/` 與 `/admin/registry/` 時斷言 404 且內容不含 `id="root"`。驗證：執行 `cargo test -p speclink-server --test it web::assets`，新測試因詳情頁回 404 而失敗，其餘既有測試仍通過。 <!-- speclink-task:tsk_01M4Z38WZ01V3X6XPKVJ937CKJ -->

## 2. 修正（Green）

- [ ] 2.1 修改 `crates/host/speclink-server/src/web/assets.rs` 的 `is_browser_route`：路徑以 `/admin/users/` 或 `/admin/registry/` 開頭、且其後恰好一段非空且不含 `/` 的片段時回 true；其餘路徑的判定不變，維持「SPA 資產與 fallback 具可驗證的安全邊界」的允許清單語意。驗證：`cargo test -p speclink-server --test it web::assets` 全部通過，包含新測試、`every_browser_route_serves_the_spa_shell_after_the_switch`（`/admin/changes` 等仍為 404）與 `health_and_ready_are_not_swallowed_by_fallback`。 <!-- speclink-task:tsk_01M4Z38WZ07BE70N5V8RPYHNV3 -->
- [ ] 2.2 排查同類缺漏並做 sharp-edges 稽核：把 `apps/server-web/src/routes/AppRoutes.tsx` 與 `apps/server-web/src/pages/admin/AdminSection.tsx` 宣告的每一條路由（搜尋寫法：在這兩個檔案找 `path=`）與 `is_browser_route` 的允許清單逐條對照，列出每條路由由哪一條規則覆蓋；再依 `speclink instructions --skill audit` 的清單檢查新的路徑比對：多段、空片段、`..` 片段與百分號編碼片段只會得到 SPA 頁殼或 404，且不讀取任何檔案。驗證：在任務記錄中寫下對照表與稽核結論；對照表內每條前端路由都有對應規則。 <!-- speclink-task:tsk_01M4Z38WZ06RJQPZAVV8S9Z8YC -->

## 3. 收尾

- [ ] 3.1 跑 repo 腳本測試，確認文件與守門面沒有被波及：`node --test scripts/*.test.mjs scripts/*/*.test.mjs` 以 exit code 0 結束；再跑一次 `cargo test -p speclink-server --test it web::assets` 確認全綠。 <!-- speclink-task:tsk_01M4Z38WZ0F8W28BWGFYP03YHM -->

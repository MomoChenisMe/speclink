## 1. server import 端點（TDD）

- [x] 1.1 紅（規格「import 端點僅限 CreateNew 且原子」；design「決策 1：import 端點固定 CreateNew、不上 wire 暴露 Overwrite」）：新增 crates/speclink-server/tests/import_api.rs——空 scope 匯入成功逐文件 Created 且各讀取端點回 Bundle 一致內容、非空 scope 拒絕零寫入（內容與 revision 未變）、reader 403、未知 format_version 與缺欄位拒收、wire 無任何 Overwrite 可達路徑。cargo test -p speclink-server 確認全紅。 <!-- speclink-task:tsk_01KY174AKMAC2JQSG1ZX7HE8KT -->
- [x] 1.2 綠：crates/speclink-server/src/routes.rs 的 import handler（Binding 認證→role 檢查→Bundle 驗證→store import CreateNew 單一原子提交→ImportReport wire 回應）、crates/speclink-server/src/app.rs 掛路由、crates/speclink-protocol/src/query.rs 的 Bundle 與 ImportReport wire DTOs、crates/speclink-remote/src/client.rs 的 import 方法。1.1 全綠。 <!-- speclink-task:tsk_01KY174AKMV8S5KXA86VKAS2H6 -->

## 2. Bundle 組裝（TDD）

- [x] 2.1 紅（規格「Bundle 組裝涵蓋本地 workspace 全集」；design「決策 2：Bundle 組裝在 desktop Rust、以 core Store 讀取面為準」）：src-tauri 組裝純函式測試——對含 active changes（meta＋artifacts）、canonical specs、live 與 archived 討論、archived changes、workflow config、LANGUAGE.md 的夾具 workspace，斷言 Bundle 文件全集與內容；壞 meta 中止並點名檔案。確認全紅。 <!-- speclink-task:tsk_01KY174AKMCVSWRCX5075WWXMB -->
- [x] 2.2 綠：apps/desktop/src-tauri/src/remote.rs 的組裝純函式與 migrate 命令雛形（root 進、Bundle 出、經 client import 上傳）。2.1 全綠；追加全集往返整合測試（組裝→匯入 in-process server→逐端點讀回一致）。 <!-- speclink-task:tsk_01KY174AKMQQBXRD0D04N5ZJVX -->

## 3. 遷移流程與轉換

- [x] 3.1（規格「遷移成功後才轉換本地且一律備份」；design「決策 3：遷移流程與本地轉換順序」）：migrate 命令完成順序——import 成功後 openspec/ 改名 openspec.migrated-{日期}（同名加序號）→ write_remote_section 寫 marker → 回報轉換結果；import 失敗本地分毫不動；改名失敗時回報指引且不寫 marker。Rust 測試覆蓋成功順序、失敗零副作用、備份同名加序號。全綠。 <!-- speclink-task:tsk_01KY174AKMKF1SC4SNMY205594 -->
- [x] 3.2（design「決策 4：兩個入口同一流程」；規格「遷移入口雙路且不經並存合併」）：新增 apps/desktop/src/components/MigrationDialog.tsx——重用 chooser 的 scopes 選擇（提示僅可遷入空 scope）、破壞性確認（目標 Project/Repo＋備份說明）、上傳進度與結果、成功後原地轉 remote 分頁（checkoutRoot）；chooser 本機路徑含 openspec/ 專案時的「遷移到 server…」次要動作接入。新增 apps/desktop/src/__tests__/migrationDialog.test.tsx 假 adapter 覆蓋全流程與失敗呈現。全綠。 <!-- speclink-task:tsk_01KY174AKMHHJCBQSFNE4WHAD7 -->
- [x] 3.3（規格 workspace-chooser「remote marker 資料夾的探測分流」修訂的三出口；design「決策 5：並存對話的第三選項語意」）：並存衝突對話改三出口——繼續本地（既有）、以 server 為準（本地改名備份→轉 checkout 開 remote 分頁、不上傳）、遷移本地內容（開 MigrationDialog）；文案明說以 server 為準＝備份後棄用本地非合併。apps/desktop/src/__tests__/remoteOpen.test.ts 與 workspaceChooser.test.tsx 覆蓋三出口。全綠。 <!-- speclink-task:tsk_01KY174AKM5TA5NMBC4VJH1MVV -->

## 4. 驗收

- [x] 4.1 GUI 鐵律手動全鏈（design Implementation Contract；操作前確認使用者未在使用螢幕）：npm run dev → 以測試副本走完整遷移至空 scope（確認對話→上傳→資料夾現 openspec.migrated-*/ 與 marker→分頁原地轉 remote→看板完整）→ CLI 於該資料夾 remote 模式 speclink list 同形 → 對非空 scope 重試被拒且本地不動 → 並存資料夾三出口各走一次（以 server 為準的備份與不上傳斷言）。 <!-- speclink-task:tsk_01KY174AKM31QBKMBYZC1B2NWE -->
- [x] 4.2 回歸：cargo test --workspace、npm test -w apps/desktop、npm test -w packages/ui、cargo build --release -p speclink-desktop 全綠（重建前關閉執行中 exe）。 <!-- speclink-task:tsk_01KY174AKMS00CZ0V0HGFCKA86 -->

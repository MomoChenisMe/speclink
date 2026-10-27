## Why

架構 §14 Phase 3 第 3 項的最後一塊：local-to-remote migration UX。chooser 刀的並存衝突對話裡「使用 remote」還停用著、附「待正式遷移功能」說明——本刀就是那個功能。store 契約的原子原語全部就位（Bundle、ImportMode 的 CreateNew、ImportReport，且 CreateNew 的「目標 scope 必須為空」已由 conformance gate 釘死），缺的只是 server 的 HTTP import 端點、desktop 的 Bundle 組裝與遷移流程 UX。

## What Changes

- server 新增 POST /import（binding scope 認證）：接受 wire Bundle（文件全集＋format version），模式固定 CreateNew——wire 不暴露 Overwrite（遷移專用 fail-closed；Overwrite 屬既有離線 Restore 維運工具）；目標 scope 非空即拒（store 的 CreateNew gate 直通、映射明確 wire 錯誤）；reader 403；成功回 ImportReport（project revision 與逐文件結果），整包單一 UoW 原子提交。
- protocol 新增 Bundle／ImportReport wire DTOs；speclink-remote client 新增 import 方法。
- desktop 的 Bundle 組裝：以引擎的 core Store 讀本地 workspace 全集（changes 的 meta 與 artifacts、canonical specs、live 與 archived 討論、archived changes、workflow config、LANGUAGE.md）組裝 wire Bundle——DocumentId 全集與 store 契約對齊。
- 遷移流程 UX：並存衝突對話「使用 remote」啟用 → 選擇 connection 與空 scope（重用 chooser 的 scopes 步驟、提示僅可遷入空 scope）→ 破壞性確認（指出內容將上傳至該 Project/Repo、本地 openspec/ 將改名備份）→ 上傳 import → 成功後本地 openspec/ 改名為帶日期的備份目錄（保留不刪除）、寫入 remote marker、原地開啟 remote 分頁（checkoutRoot＝該資料夾）。任一步失敗（scope 非空、網路、驗證）即原樣呈現錯誤且本地分毫不動。
- 遷移入口同時提供於 chooser 的本機路徑（開到含 openspec/ 的專案時的「遷移到 server…」次要動作），非只有並存衝突一途。

## Capabilities

### New Capabilities

- `workspace-migration`: local→remote 正式遷移的行為保證——CreateNew 專用 import 端點、Bundle 全集組裝、原子上傳、本地備份改名與 marker 轉換、失敗零副作用。

### Modified Capabilities

- `workspace-chooser`: 並存衝突對話的「使用 remote」由停用附說明改為進入遷移流程。

## Impact

- 相容性影響：server 純新增端點；desktop 遷移為顯式流程、未觸發時零影響；本地 openspec/ 一律改名保留、無刪除路徑。
- Affected specs: `workspace-migration`（新增）、`workspace-chooser`（修改）
- Affected code:
  - New: crates/speclink-server/tests/import_api.rs、apps/desktop/src/components/MigrationDialog.tsx、apps/desktop/src/__tests__/migrationDialog.test.tsx
  - Modified: crates/speclink-server/src/routes.rs、crates/speclink-server/src/app.rs、crates/speclink-protocol/src/query.rs、crates/speclink-remote/src/client.rs、apps/desktop/src-tauri/src/remote.rs、apps/desktop/src-tauri/src/lib.rs、apps/desktop/src/store.ts、apps/desktop/src/App.tsx、apps/desktop/src/components/WorkspaceChooser.tsx、apps/desktop/src/i18n/messages.ts、apps/desktop/src/__tests__/workspaceChooser.test.tsx、apps/desktop/src/__tests__/remoteOpen.test.ts、Cargo.lock
  - Removed: 無

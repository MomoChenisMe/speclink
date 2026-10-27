## Why

remote workspace 的看板卡片拖排是 capability 停用清單的最後一項。不能照本地做法直上：本地拖排把 board_rank 寫進變更 meta／討論 frontmatter，remote 若沿用等於每次拖卡都 mutate 共享規格 revision——汙染 append-only 文件歷史、觸發全員 SSE invalidate，正是架構文件「不建議的做法」明文禁止的「把 board/card 個人呈現狀態默認混入共享規格 revision」；同句給出的正路是「定義獨立、具 CAS 的 board resource」。本刀走這條路，與本地「rank 進 git 即團隊共享」的順序語意對齊。

## What Changes

- store 契約：DocumentId 新增 board order 種類（scope 層級單文件）——三個 driver（server fs、SQLite、PostgreSQL）的編碼／解碼與 conformance suite 案例補齊；export／import 走既有泛用文件列舉自動涵蓋（migration 與 backup 隨行，無需另做）。
- server 兩個端點：GET /board-order（內容＋ETag；缺席為正常態）與 PUT /board-order（If-Match CAS、editor role 限定、commit 後發 invalidate）。內容對 server 為不透明文本（呈現資源、不經引擎解析），僅設大小上限。
- speclink-protocol DTO 與 speclink-remote client 兩方法。
- desktop remote 排序 overlay：remote 清單指令取 board resource 併入排序（rank 升冪、缺 rank 卡依 server 回傳序置頂、同值以名稱決斷——與本地排序語意同構），UI 元件零改動。
- desktop remote 拖排直達：reorderCard 由停用改實作——讀清單＋board resource → 欄內補章／中點鍵（重用桌面 core 的 rank 演算法）→ PUT 全文；409 衝突重讀重算重試一次，再敗以錯誤呈現並刷新（不留假象順序）。每次 PUT 重寫全圖時修剪已消失卡的孤兒條目。
- capability 翻正：reorderCard 依 role 翻真（editor 真、reader 假）；remote-workspace-data 的停用清單自此清空。
- 順序依賴：本刀對 remote-workspace-data 的規格修訂以 remote-verb-parity 修訂後文本為基準，apply 與 archive SHALL 排在該刀之後。

## Non-Goals

- 本地模式的順序真相不動：仍為卡片 meta 的 board_rank（進 git 共享）；不把本地遷移到 board resource。
- 不做個人（per-user）順序視圖——共享順序是與本地語意對齊的選擇；個人視圖偏好屬未來需求。
- 不做 board resource 的歷史瀏覽／還原 UI；文件歷史由 store immutable history 既有機制承載。
- server 不解析、不校驗 board resource 的語意內容（呈現資源非政策文件；損壞內容由桌面 fallback 排序容錯）。

## Capabilities

### New Capabilities

- `remote-board-order`: remote 看板順序的行為保證——board resource 文件契約（scope 單文件、不透明內容、CAS）、remote 排序 overlay 語意、拖排寫回與衝突收斂、孤兒條目修剪與損壞容錯。

### Modified Capabilities

- `board-card-order`: 順序真相 requirement 分模式改寫——本地 session 以卡片 meta 的 board_rank 為真相（不變）；remote session 以 board resource 為真相、排序語意同構。
- `remote-workspace-data`: capability 停用清單清空——看板拖排改直達 board resource（依 role）。
- `teamstore-contract`: DocumentId 封閉種類集合擴充 board order（並補記已出貨但漏列的 language 種類——既有規格債順帶修正）。

## Impact

- Affected specs: `remote-board-order`（新增）、`board-card-order`（修改)、`remote-workspace-data`（修改）、`teamstore-contract`（修改）
- Affected code:
  - New: `crates/speclink-server/tests/board_order.rs`（端點整合測試）
  - Modified: `crates/speclink-store/src/types.rs`（DocumentId 變體）、`crates/speclink-store/src/conformance.rs`（案例）、`crates/speclink-store/src/lib.rs`（teststore 涵蓋，如需）、`crates/speclink-store-fs/src/layout.rs`、`crates/speclink-store-sqlite/src/lib.rs`、`crates/speclink-store-postgres/src/lib.rs`（三 driver 編碼／解碼）、`crates/speclink-server/src/app.rs`、`crates/speclink-server/src/routes.rs`（兩 handlers）、`crates/speclink-protocol/src/query.rs`（DTO）、`crates/speclink-remote/src/client.rs`（兩方法）、`apps/desktop/src-tauri/src/remote.rs`（排序 overlay＋reorder 指令＋capability 翻真）、`apps/desktop/src-tauri/src/lib.rs`（指令註冊）、`apps/desktop/src/adapter/remoteDataSource.ts`（reorderCard 直達）、`apps/desktop/src/__tests__/remoteDataSource.test.ts`（更新）
  - Removed: （無）

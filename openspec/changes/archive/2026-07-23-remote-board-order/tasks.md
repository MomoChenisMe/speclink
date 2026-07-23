## 1. store 契約擴充

- [x] 1.1 紅：於 crates/speclink-store/src/conformance.rs 新增案例——board order 文件 UoW 寫入後重開讀取逐位元組一致、出現於同 scope export bundle、封閉集合恰為八種（規格「文件定址採 Project 與 Repo scope 的邏輯 locator」修訂後語意；design「決策 2：BoardOrder 為 scope 層級單文件、內容不透明、泛用列舉自動隨行」）——此時紅（變體尚不存在則以編譯失敗計）。 <!-- speclink-task:tsk_01KY6AJ55C4M57JABAMEP2GKJ9 -->
- [x] 1.2 綠：crates/speclink-store/src/types.rs 新增 board order 種類；crates/speclink-store-fs/src/layout.rs、crates/speclink-store-sqlite/src/lib.rs、crates/speclink-store-postgres/src/lib.rs 補編碼／解碼；teststore 如需同步。驗收：三 driver conformance 全綠（PostgreSQL 以本機 speclink_test 資料庫執行）。 <!-- speclink-task:tsk_01KY6AJ55CWKTCBWJACH7VCC1Y -->

## 2. server 端點與 client

- [x] 2.1 紅：新增 crates/speclink-server/tests/board_order.rs 整合測試（in-process server）：規格「board resource 為 scope 單文件且 server 不解析」——GET 缺席回 null 內容附 ETag、PUT 過期 If-Match 回 409 reason 機器可判、reader PUT 403、超大 payload 拒絕、PUT 成功後訂閱端收 invalidate、任意文本（含非法 JSON）server 照存不校驗——此時紅。 <!-- speclink-task:tsk_01KY6AJ55C7SR5AATY5DMTFWA1 -->
- [x] 2.2 綠：crates/speclink-protocol/src/query.rs 新增 DTO；crates/speclink-server/src/app.rs 掛 GET/PUT /board-order；crates/speclink-server/src/routes.rs 兩 handlers 沿 put_config 的直寫形狀不新增引擎 Command（design「決策 3：PUT 全文＋If-Match CAS 沿 put_config 先例」）；crates/speclink-remote/src/client.rs 新增讀寫兩方法。驗收：2.1 全綠。 <!-- speclink-task:tsk_01KY6AJ55CJZ14CBWEBSJN48YG -->

## 3. 桌面 Rust：排序 overlay 與拖排流程

- [x] 3.1 紅：apps/desktop/src-tauri/src/remote.rs 測試——remote 清單合併排序：具 rank 依字典序升冪、缺 rank 依 server 回傳序置頂、同值以名稱決斷、無 board resource 時順序與現行逐項一致（規格「remote 排序 overlay 與本地語意同構」）；board resource 為非法 JSON 時視為全缺 rank 照常渲染（規格「損壞容錯與孤兒條目修剪」的退回語意；design「決策 4：排序 overlay 在桌面 Rust 側，UI 零改動」「決策 6：損壞容錯與孤兒條目歸桌面」）——此時紅。 <!-- speclink-task:tsk_01KY6AJ55CX0D3KX1W1PK9YVWC -->
- [x] 3.2 紅：reorder 流程測試——欄內缺 rank 整欄補章只寫 board resource、落點鄰居中點鍵（消失鄰居視開放端、逆序棄上界）、PUT 全文修剪不在現行清單的條目、409 重讀重算重試恰一次、再敗回錯誤並刷新 server 現況、全程不觸碰卡片 meta／frontmatter（規格「拖排寫回以全文 CAS 與一次重試收斂」「看板卡片順序以 board_rank 欄位為真相」修訂後的 remote 不寫 meta 斷言；design「決策 5：拖排寫回＝讀清單＋board resource→補章／中點→PUT 全文，409 重試一次」「決策 1：共享順序＝獨立 CAS board resource，卡片 meta 不動」）——此時紅。 <!-- speclink-task:tsk_01KY6AJ55C5TWJF77VP1EQSSEZ -->
- [x] 3.3 綠：remote.rs 實作清單排序 overlay 與 reorder 指令（重用 apps/desktop/core rank 模組的 spread／midpoint 與 stage 同構欄推導）、apps/desktop/src-tauri/src/lib.rs 註冊。驗收：3.1 與 3.2 全綠。 <!-- speclink-task:tsk_01KY6AJ55C62731ZFBBG0DYE71 -->

## 4. 桌面 TS 與 capability 翻正

- [x] 4.1 紅：更新 apps/desktop/src/__tests__/remoteDataSource.test.ts——reorderCard 斷言改為對 remote reorder invoke 的參數映射（kind／id／prevId／nextId，不再回拒絕錯誤）；RemoteCapabilities 斷言：editor 的 reorderCard 真、reader 假（規格「capability 驅動停用且不偽造缺口」修訂後語意）——此時紅。 <!-- speclink-task:tsk_01KY6AJ55CW7T7JG0DDVH6A8C8 -->
- [x] 4.2 綠：apps/desktop/src/adapter/remoteDataSource.ts 的 reorderCard 由 unsupported 改 invoke 直達；remote.rs 的 RemoteCapabilities 依 role 翻真（design「決策 7：capability 依 role 翻真，停用清單清空」）；UI 元件零改動。驗收：4.1 全綠、npm test -w packages/ui 零修改全綠。 <!-- speclink-task:tsk_01KY6AJ55CSXVRJ0TYPSWHPCJX -->

## 5. 收尾驗證

- [x] 5.1 GUI 鐵律手動驗證（remote-dev-harness：npm run dev；操作前確認使用者未在使用螢幕）：editor 於 remote 分頁拖排卡片——落位即時、另開一個 client 數秒內同序、重啟 app 順序持久；reader 分頁拖排把手不渲染附繁中說明；本地分頁拖排行為與交付前完全一致（meta 寫回照舊）。 <!-- speclink-task:tsk_01KY6AJ55CE6WTP8WXJB43RT89 -->
- [x] 5.2 全量回歸：cargo test --workspace 與 npm test（workspaces）全綠；speclink validate remote-board-order 通過。apply 與 archive 順序守則：本刀 SHALL 排在 remote-verb-parity 之後（remote-workspace-data 修訂文本與 remote.rs／remoteDataSource.ts 共檔，平行時依提交衛生合流）。 <!-- speclink-task:tsk_01KY6AJ55CGCQA6M491FVTGWZF -->

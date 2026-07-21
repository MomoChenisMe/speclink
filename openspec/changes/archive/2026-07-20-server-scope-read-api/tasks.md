## 1. Store 列舉 seam（TDD）

- [x] 1.1 紅（規格「archived 列舉為 Store 契約的一部分」；design「決策 2：archived 列舉 seam 加在 core Store trait」）：對 core Store trait 的 list_archived_changes 寫測試——teststore 於 archive 後列舉可見且 dated name 降冪；host BridgeStore 對 TeamStore（memory driver）同斷言、列舉與 archived_change_exists 點讀一致。執行 cargo test -p speclink-core -p speclink-host 確認新案例全紅。 <!-- speclink-task:tsk_01KXXBM4PCN7T1HVMAKDHN3EYN -->
- [x] 1.2 綠：crates/speclink-core/src/store.rs 加 trait 方法；crates/speclink-core/src/teststore.rs 與 crates/speclink-host/src/bridge.rs 實作真值；crates/speclink-host/src/drift.rs 的唯讀最小 adapter 顯式回空並註明用途限定；CLI 側檔案系統 Store 實作以 archive 目錄列舉補齊。1.1 全綠、cargo test --workspace 無回歸。 <!-- speclink-task:tsk_01KXXBM4PCF49NN5CQ5XHE6R8Q -->

## 2. server 端點（TDD）

- [x] 2.1 紅（規格「scopes 清單依 membership 過濾且身分即可呼叫」；design「決策 1：/scopes 走「身分不綁定」抽取器」）：新增 crates/speclink-server/tests/read_api.rs——/scopes 雙使用者不同 membership 互不可見、無 membership 空清單 200、停用帳號 403、缺 Bearer 401、admin 無特權繞過。確認全紅。 <!-- speclink-task:tsk_01KXXBM4PC10ZXGJD7BZVBMT77 -->
- [x] 2.2 紅（規格「正典 spec 內文可讀」「archived 瀏覽三端點」「workspace 全文搜尋與桌面語意對齊」；design「決策 3：search 語意＝桌面 D6 逐字對齊」「決策 4：端點形狀與錯誤語意」）：同測試檔續寫——spec 內文對播種正典回真值、缺席 404；archive 動詞後清單欄位如實（任務計數、specCount、createdBy、fromDiscussions）、artifact 內文與 capabilities 回真值、跨 scope 不可見；search 大小寫不敏感、每卡首個命中、snippet 截斷補 …、空查詢空陣列、change 與 discussion 兩 kind 並現。確認全紅。 <!-- speclink-task:tsk_01KXXBM4PC5DVN6K9C5NM62Z2J -->
- [x] 2.3 綠：crates/speclink-server/src/auth.rs 加 IdentityOnly 抽取器；新增 crates/speclink-server/src/read_api.rs（五 handler 與清單欄位衍生、search 線性掃描）；crates/speclink-server/src/app.rs 掛五路由；crates/speclink-protocol/src/query.rs 加 ScopesResponse／SpecDocumentResponse／ArchivedListResponse 與 ArchivedItem／SearchResponse 與 SearchHit DTOs（schemars 照慣例）。2.1、2.2 全綠。 <!-- speclink-task:tsk_01KXXBM4PC3H6C8QJKMJA8X1BB -->

## 3. client 與桌面解鎖

- [x] 3.1 crates/speclink-remote/src/client.rs 加五方法（list_scopes、spec_document、archived_list、archived_artifact、archived_capabilities、search——scopes 供下一刀 chooser 消費，本刀不接桌面 UI），對 in-process server 的既有測試模式補方法級斷言。cargo test -p speclink-remote 全綠。 <!-- speclink-task:tsk_01KXXBM4PCYCKJSP0YRB34A0AG -->
- [x] 3.2（規格 remote-workspace-data 的「capability 驅動停用且不偽造缺口」修訂；design「決策 5：桌面解鎖的邊界」）：apps/desktop/src-tauri/src/remote.rs 補五個 remote_* 命令；apps/desktop/src/adapter/remoteDataSource.ts 的 listArchived／getArchivedDocument／archivedCapabilities／getSpecDocument／searchWorkspace 改直達；capability 描述翻正三項、validate/analyze 與刪除/拖排維持停用；remote 分頁的 archived 與 spec 內文提示卡移除。apps/desktop/src/__tests__/remoteDataSource.test.ts 與 capability 停用測試同步更新（可用三項＋維持停用斷言）。npm test -w apps/desktop 全綠。 <!-- speclink-task:tsk_01KXXBM4PC185JS89RBYHJ9C0F -->

## 4. 驗收

- [x] 4.1 手動驗證（remote-dev-harness；操作前確認使用者未在使用螢幕）：npm run dev → remote 分頁封存頁見清單與內文、規格卡開內文、看板搜尋可用且結果與本地同形；以無 membership 帳號的 PAT 呼叫 /scopes 得空清單、有 membership 帳號得其專案與 repos。 <!-- speclink-task:tsk_01KXXBM4PCV6ZFVSEFZNXN594G -->
- [x] 4.2 回歸：cargo test --workspace、npm test -w apps/desktop、npm test -w packages/ui、cargo build --release -p speclink-desktop 全綠；CLI 輸出凍結不受影響（本刀不動 CLI 命令面）。 <!-- speclink-task:tsk_01KXXBM4PCKHHJW9P9G6P81E8S -->

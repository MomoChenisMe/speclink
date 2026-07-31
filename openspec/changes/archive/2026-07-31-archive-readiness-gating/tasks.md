## 1. 引擎:單筆封存任務完成度守門

對應:spec change-lifecycle「單筆封存的任務完成度守門」;design D1 守門放 core 封存函式本體,拒絕用 typed Refusal。

- [x] 1.1 撰寫守門測試:crates/speclink-core/src/archive.rs 的 #[cfg(test)] 新增——任務 1/3 未完成且未帶 mark_tasks_complete 時 archive 回 Err(訊息含「1/3」與 --mark-tasks-complete 提示)且 store 零寫入、任務全完成放行、任務總數 0 放行、mark_tasks_complete=true 放行;crates/speclink-core/src/command/mod.rs 測試斷言 runtime 對未完成 change 的 Archive 命令回 refused 分類錯誤 <!-- speclink-task:tsk_01KYVFH9G80DZK3NJW0F10C58K -->
- [x] 1.2 實作守門:crates/speclink-core/src/archive.rs 的 archive 函式於 require_valid_meta 之後、dated-name 檢查之前,經 Store 讀 tasks.md 以 tasks::parse+progress 判定,未帶 mark_tasks_complete 且 total>0 且 complete<total 時回 command::Refusal 型錯誤(比照 discard.rs 既有模式),訊息載明 N/M 與兩條出路——落實 spec 需求「單筆封存的任務完成度守門」 <!-- speclink-task:tsk_01KYVFH9G8DBYV6J2P82AKH9W5 -->
- [x] 1.3 撰寫 CLI 整合測試:crates/speclink-cli/tests/ 新增——speclink archive <name> 對未完成 change 非零 exit、stderr 含 N/M 證據與出路且 changes/archive/ 無新目錄;--mark-tasks-complete 成功且封存後 tasks.md 全勾;既有單筆成功路徑與批次(--all)skip 測試不修改即通過(證明成功路徑逐位元不變) <!-- speclink-task:tsk_01KYVFH9G8H35W987SEZ8GHPPX -->
- [x] 1.4 驗證:cargo test -p speclink-core -p speclink-cli 全綠;手跑 speclink archive <未完成 change> 確認 stderr 訊息與 exit code <!-- speclink-task:tsk_01KYVFH9G8C1VWN6WDMZZCQ294 -->

## 2. desktop 刪除接 discard 與 remote force 翻案

對應:spec desktop-app「桌面刪除變更走 discard 全語意」;design D2 desktop 刪除改接 discard,remote 翻案 force=false。

- [x] 2.1 撰寫刪除測試:apps/desktop/core/src/manage.rs 的 #[cfg(test)] 新增——刪除由討論轉出且無開工痕跡的 change 後,來源討論 promoted_to 移除該名、清單空時狀態回復、touched 紀錄清除;meta 含 started_at 的 change 刪除回 Err 且 openspec/ 檔案不變(既有 delete_change_removes_active_change 測試同步改為無痕跡 fixture) <!-- speclink-task:tsk_01KYVFH9G8X9QGMFBA44K87NWJ -->
- [x] 2.2 實作:apps/desktop/core/src/manage.rs 的 delete_change_at 改為委派 speclink_core::discard::discard(force=false),簽名與 Result<(), String> 回傳不變,Tauri 殼 delete_change command 單行委派不動——落實 spec 需求「桌面刪除變更走 discard 全語意」 <!-- speclink-task:tsk_01KYVFH9G8AM7SX2KTBE7DEJY2 -->
- [x] 2.3 remote 刪除翻案:apps/desktop/src/adapter/remoteDataSource.ts 的 deleteChange 改帶 force: false 並更新決策註解;apps/desktop/src/__tests__/ 以 vitest 斷言 invoke("remote_delete_change") 收到 force: false,拒絕錯誤沿既有 store.deleteFailed toast 路徑 <!-- speclink-task:tsk_01KYVFH9G8YNSYFA495Q74B3CQ -->
- [x] 2.4 驗證:apps/desktop/core 下 cargo test 全綠;npm test -w apps/desktop 全綠 <!-- speclink-task:tsk_01KYVFH9G82RT1ZKVZ599AEMMP -->

## 3. 看板拖曳落點就緒守門

對應:spec desktop-app「拖曳封存落點以浮層呈現」、spec board-card-order「跨欄拖曳不改變變更階段」;design D3 落點浮現條件以就緒名單傳入純函式。

- [x] 3.1 撰寫拖曳測試:packages/ui/src/__tests__/kanban.test.tsx 與 boardDnd 純函式測試——archiveZoneVisible 對就緒變更卡 id 回 true、非就緒變更卡與討論卡回 false、null 回 false;拖曳進行中卡時畫面不出現封存落點元素;dragEnd 於 over=archived 且卡非就緒時不觸發 onArchive <!-- speclink-task:tsk_01KYVFH9G8SEJP1EC2HZD4K0YD -->
- [x] 3.2 實作:packages/ui/src/boardDnd.ts 的 archiveZoneVisible 擴充簽名收就緒名單(ReadonlySet);packages/ui/src/components/KanbanBoard.tsx 以 changeStage 派生就緒集合傳入,dragEnd 的 archived 分支加同一就緒判定——落實 spec 需求「拖曳封存落點以浮層呈現」與「跨欄拖曳不改變變更階段」 <!-- speclink-task:tsk_01KYVFH9G83229ZKD5K9ESFGQB -->
- [x] 3.3 驗證:npm test -w packages/ui 全綠,既有同欄拖排與跨欄彈回測試不修改即通過 <!-- speclink-task:tsk_01KYVFH9G89KR7SE5Z9GEQAC67 -->

## 4. 詳情抽屜階段守門

對應:spec desktop-app「詳情抽屜的封存與刪除依階段守門」;design D4 抽屜鈕守門原因與既有 UnavailableAction 合流。

- [x] 4.1 撰寫抽屜測試:packages/ui/src/__tests__/richDrawer.test.tsx——提案中與進行中 change 的封存鈕 disabled 且 tooltip 載任務進度與出路、已就緒可按;進行中與已就緒 change 的刪除鈕 disabled 且 tooltip 載開工痕跡與退回出路、提案中可按;宿主傳入 unavailable.archive/delete(remote 能力缺失)時該原因優先於階段原因呈現 <!-- speclink-task:tsk_01KYVFH9G8313WQX9N91FDZZ3M -->
- [x] 4.2 實作:packages/ui/src/components/RichDetailDrawer.tsx 以既有 changeStage 派生階段,封存鈕非 ready、刪除鈕非 proposed 時 disabled 並經 UnavailableAction 呈現原因(unavailable 原因優先);packages/ui/src/i18n.tsx 新增雙語守門原因文案(封存:任務進度+完成後才能封存;刪除:已有開工痕跡+先退回提案中)——落實 spec 需求「詳情抽屜的封存與刪除依階段守門」 <!-- speclink-task:tsk_01KYVFH9G89R150SSQ99QDEV97 -->
- [x] 4.3 驗證:npm test -w packages/ui 全綠;npm test -w apps/desktop 全綠(App 接線無回歸) <!-- speclink-task:tsk_01KYVFH9G84VEDXZAR4JH4HNPM -->

## 5. 收尾驗證

- [x] 5.1 全量回歸:cargo test --workspace 與根目錄 npm test 全綠,確認未動的 golden 與既有 CLI 測試無需更新 <!-- speclink-task:tsk_01KYVFH9G8163RM2KBX60WEFDT -->
- [x] 5.2 手動驗證與規格對齊:dev 環境啟動 desktop——拖曳進行中卡全程無封存落點、抽屜鈕 disabled 附原因、就緒卡拖曳封存與提案中刪除照常、刪除轉出 change 後討論卡回復;speclink validate archive-readiness-gating 通過 <!-- speclink-task:tsk_01KYVFH9G84GGE8GXQQC3G9VGP -->

## 1. 狀態機與事件廣播（TDD）

- [x] 1.1 紅（規格「離線狀態機單一真相且明確呈現」；design「決策 1：connection 狀態機在 Rust、單一真相、事件廣播」）：apps/desktop/src-tauri/tests/remote_runtime.rs 與 tests/event_manager.rs 新增案例（注入失敗序列與閾值）——連續失敗達閾值轉 offline 並廣播 remote-connection-state、單次成功歸零回 online、needs-reauth 置位時廣播且優先於 offline、worker 退避中 sync-state 失敗同計。cargo test -p speclink-desktop 確認全紅。 <!-- speclink-task:tsk_01KXZWJENQC11BH782G99RJKHZ -->
- [x] 1.2 綠：apps/desktop/src-tauri/src/remote.rs 的失敗計數與狀態轉換、apps/desktop/src-tauri/src/event_manager.rs 的收斂成功/失敗回報接線、apps/desktop/src-tauri/src/lib.rs 的事件 emit。1.1 全綠、好天氣路徑既有測試零改動。 <!-- speclink-task:tsk_01KXZWJENQ4HFPS5CHZ1E9N4GM -->

## 2. stale 唯讀與寫入即拒（TDD）

- [x] 2.1 紅（規格「最後 snapshot 唯讀且寫入即拒無佇列」；design「決策 2：stale 唯讀＝保留最後內容＋capability 疊加 offline mask」）：Rust——offline 期間全部寫入命令即拒（與 needs-reauth 同語意）、讀取放行；整合測試（in-process server 停起）：殺 server → 寫入被拒 → 重啟 → server 端查無離線期間寫入。TS（新增 apps/desktop/src/__tests__/remoteResilience.test.tsx，假事件源）——offline 事件後清單保留不清空、stale 橫幅與 cloud-off 呈現、寫入 affordance 全停用、本地分頁不受影響。確認全紅。 <!-- speclink-task:tsk_01KXZWJENQT5PYRB74305FAD9S -->
- [x] 2.2 綠：Rust 寫入命令的 offline 檢查；apps/desktop/src/store.ts 的 reload 失敗保留既有內容（修正可能清空的路徑）、connections 狀態訂閱翻 session 呈現；apps/desktop/src/session.ts 的有效 capability＝handshake capability 疊加 offline mask；apps/desktop/src/App.tsx 與 apps/desktop/src/components/ProjectTabs.tsx 的橫幅與圖示；apps/desktop/src/i18n/messages.ts 文案。2.1 全綠。 <!-- speclink-task:tsk_01KXZWJENQK8KFWYC9BNG1Y6Q2 -->

## 3. 恢復與重新認證

- [x] 3.1（規格「恢復自動收斂並清除 stale」；design「決策 3：恢復＝worker 收斂事件驅動，不新造機制」）：worker 收斂成功 → online 廣播＋全量失效通知 → store 全量重查清 stale 的接線；整合測試：殺 server 期間以另一 client 寫入、重啟後分頁自動含新內容且 stale 清除、無使用者操作。全綠。 <!-- speclink-task:tsk_01KXZWJENQA39PT2HTTA19GN7E -->
- [x] 3.2（規格「重新認證原地復活不退 local」；design「決策 4：重新認證入口與原地恢復編排」）：needs-reauth 橫幅的重新登入入口（開應用程式設定頁伺服器簽聚焦該連線）；登入成功後的編排——該 connection 全部 remote sessions 逐一 re-handshake、全量重查、event worker 重啟；vitest 斷言編排呼叫序與「分頁全程存在」；歷程中無任何退回 local mode 的路徑（斷言不存在該分支）。全綠。 <!-- speclink-task:tsk_01KXZWJENQW1V70MXW6J9V2BJS -->
- [x] 3.3（規格「remote 破壞性操作確認一致」；design「決策 5：destructive 一致化＝檢核與措辭，不改機制」）：remote 分頁 archive 確認描述補 scope（Project/Repo 名）；deleteChange 停用與 offline 遮罩下 archive 停用的既有斷言確認（apps/desktop/src/__tests__/remoteCapabilities.test.tsx 補案例）。全綠。 <!-- speclink-task:tsk_01KXZWJENQRBGTZ4XDVDFB9GB7 -->

## 4. 驗收

- [x] 4.1 GUI 鐵律手動全鏈（design Implementation Contract；操作前確認使用者未在使用螢幕）：npm run dev 開 remote 分頁 → 殺 server：橫幅與 cloud-off、看板可讀、勾任務被拒 → 重啟 server：自動恢復、stale 清除、server 端查無離線期間的寫入 → 於 /account 撤 device family：needs-reauth 橫幅 → 重新登入 → 分頁原地復活可讀寫 → 全程分頁不消失、本地分頁如常。 <!-- speclink-task:tsk_01KXZWJENQJ7CSBY8P70NEE19Y -->
- [x] 4.2 回歸：cargo test --workspace、npm test -w apps/desktop、npm test -w packages/ui、cargo build --release -p speclink-desktop 全綠（重建前關閉執行中 exe）。 <!-- speclink-task:tsk_01KXZWJENQHSM38V9QA9PKQKR5 -->

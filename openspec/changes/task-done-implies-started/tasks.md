## 1. D1 引擎任務完成協作函式（core::tasks 單點四合一）與 D5 歸屬參數縫維持開放

- [ ] 1.1 紅：crates/speclink-core/src/tasks.rs 新增協作函式（暫名 complete）單元測試——(a) 對 meta 無 started_* 的 change 完成一項任務後：tasks.md 該任務成 [x]、touched 記錄含該任務項（有未認領 dirty 檔時；無則不追加）、meta 新增 started_at 且既有欄位逐字元保留；(b) 已含 started_* 的 change 完成後續任務：started_* 三欄位值不變；(c) 任務已完成：回報 already、tasks.md 與 meta 與 touched 皆無寫入；(d) identity/agent 依 D5 缺席規則——identity None 時無 started_by、agent None 時無 started_with（本刀兩端皆傳 agent None）；(e) 任務序號越界回 Err。驗證：cargo test -p speclink-core 出現上述測試的預期紅燈。
- [ ] 1.2 綠：實作 complete——組合既有 core::tasks::mark_done、store 寫回、TouchedRecord 記錄與 core::inprogress::add（冪等首章）。行為契約以 specs/change-lifecycle/spec.md「任務完成蘊含開工標記」各 Scenario 為準。驗證：cargo test -p speclink-core 全綠。

## 2. D2 CLI task done 改薄呼叫端且輸出凍結

- [ ] 2.1 紅：crates/speclink-cli 整合測試——speclink task done 首次完成後 .openspec.yaml 含 started_at；對已完成任務重跑維持現行「already done」錯誤且無任何檔案變動；tasks.md 缺失錯誤訊息與現行一致。驗證：cargo test -p speclink-cli 預期紅燈。
- [ ] 2.2 綠：crates/speclink-cli/src/commands.rs 的 cmd_task done 分支改呼叫 1.2 的協作函式，人眼輸出、--json payload（change/status/taskDesc/taskId 欄位形狀）、錯誤訊息與順序（tasks.md 缺失先於 id 驗證、already 於勾章判定後）、exit code 全部不變。驗證：cargo test -p speclink-cli 全綠；改動前先保存基線 exe，自我基線雙沙盒對照確認 task done 輸出零差異、檔案樹差異僅 .openspec.yaml 新增 started_*（刻意分歧，於變更記錄註明基線更新）。

## 3. D3 桌面勾選走協作函式且冪等寬容

- [ ] 3.1 紅：apps/desktop/core/src/manage.rs 測試——(a) set_task_done_at done=true 於 meta 無 started_* 的 change：tasks.md 勾章＋meta 蓋章＋touched 記錄；(b) 對已完成任務 done=true：冪等成功（Ok）且無任何檔案寫入；(c) done=false 與 move_task_at：僅 tasks.md 變動，meta 與 touched 逐字元不變；(d) ordinal 對齊——以含群組標題、巢狀縮排與非 checkbox 行混排的同一 tasks.md fixture，desktop ordinal N 與引擎 task id N 指向同一任務。驗證：cargo test -p speclink-desktop 預期紅燈。
- [ ] 3.2 綠：set_task_done_at 的 done=true 路徑改呼叫協作函式（identity 沿 git 身分、agent 缺席），done=false 與 move 維持既有行編輯。行為契約以 specs/desktop-app/spec.md「GUI 勾任務與 CLI 完成語意一致」各 Scenario 為準。驗證：cargo test -p speclink-desktop 全綠。

## 4. D4 看板派生加入任務進度（顯示與歸屬分離）

- [ ] 4.1 [P] 紅：packages/ui/src/__tests__/stage.test.ts 更新判定矩陣——新增（無章、3/28 → in-progress）、（有章、0/28 → in-progress）、（無章、0/28 → proposed）、（無章、28/28 → ready）案例；packages/ui/src/__tests__/kanban.test.tsx 同步驗證無章有進度的卡片落於進行中欄。驗證：npm test -w packages/ui 預期紅燈。
- [ ] 4.2 [P] 綠：packages/ui/src/stage.ts 的 changeStage 優先序改為：全完成（總數>0）→ ready；startedAt 或 completedTasks>0 → in-progress；其餘 → proposed。行為契約以 specs/desktop-app/spec.md 修改後的需求「看板欄位由生命週期標記驅動」及其欄位判定矩陣為準；詳情抽屜開工列維持「meta 有 startedAt 才顯示」（RichDetailDrawer 無需變更，以既有 richDrawer 測試守住）。驗證：npm test -w packages/ui 全綠。

## 5. 整合建置與真實視窗驗證

- [ ] 5.1 建置整合：npm run build -w apps/desktop 後 cargo build --release -p speclink-desktop（重建前先關閉執行中的 speclink-desktop.exe）。驗證：兩者成功結束、npm test -w apps/desktop 全綠。
- [ ] 5.2 真實視窗驗證 specs/desktop-app/spec.md 各 Scenario（操作前先確認使用者沒在使用螢幕；前提：drawer-live-reload 已落地——抽屜內容隨刷新世代重載，否則 (a) 的開工列改以重開抽屜後驗證）：(a) 以拋棄式測試 change 於看板勾首任務 → 卡片移入進行中欄、抽屜顯示開工列、.openspec.yaml 含 started_at/started_by；(b) 以編輯器直接修改另一測試 change 的 tasks.md 勾一項 → 看板刷新後卡片列進行中欄、抽屜無開工列、meta 無變動；(c) 取消勾選與拖曳排序 → meta 與 touched 逐字元不變。驗證：每項截圖或觀察記錄留證，驗證後清理測試 change。

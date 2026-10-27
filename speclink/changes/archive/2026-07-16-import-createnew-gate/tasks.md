## 1. conformance gate（先紅）

- [x] 1.1 【紅】conformance suite 新增「全新建立模式拒絕非空 scope」gate（crates/speclink-store/src/conformance/mod.rs）：目標 scope 先 commit 一份 bundle 外的文件 X，import 只含 Y 的 bundle（CreateNew）——斷言整筆拒絕（backend 類別）、scope 仍只持有 X、project revision 未動、無 Y 痕跡；同場景以 Overwrite 模式匯入同名文件斷言照常成功（涵蓋「覆蓋模式不受空 scope 前置影響」情境）。驗收：gate 進 suite 的必跑清單；此時 memory 與 sqlite 的 conformance 轉紅、fs 與 postgres 維持綠。 <!-- speclink-task:tsk_01KXK5DD0K0GSZGDQ6QYBVMYRX -->

## 2. 修正兩支偏差實作（轉綠）

- [x] 2.1 【綠】memory 的 CreateNew 前置改為「目標 scope 持有任何文件即拒絕」（crates/speclink-store/src/memory.rs），錯誤類別與訊息維持現值；cargo test -p speclink-store 全綠（含新 gate）。 <!-- speclink-task:tsk_01KXK5DD0M05S6E4M5TE1C3QQ4 -->
- [x] 2.2 【綠】sqlite 的 CreateNew 前置同樣改為 scope 級檢查（crates/speclink-store-sqlite/src/lib.rs）；cargo test -p speclink-store-sqlite 全綠（含新 gate）。 <!-- speclink-task:tsk_01KXK5DD0M8YT8FD0DBXMZH2AX -->

## 3. 四支一致與回歸

- [x] 3.1 四支實作對更新後 suite 重跑：cargo test -p speclink-store、-p speclink-store-sqlite、-p speclink-store-fs 全綠；speclink-store-postgres 以環境變數指向的實例全綠（fs 與 postgres 應零改動通過——它們即正典行為）。驗收：四份 conformance 報告零 failure。 <!-- speclink-task:tsk_01KXK5DD0MEG9AAY6D8T3J6QB5 -->
- [x] 3.2 執行 npm run test:all 確認全 workspace 回歸：server 既有 backup/restore 與 e2e 不受影響（restore 只進空目標，行為不變）；parity 31 項、color 16 項、twin 8 情境凍結零 diff。驗收：全數通過。 <!-- speclink-task:tsk_01KXK5DD0MNBDQ3SYWWAFZW18K -->

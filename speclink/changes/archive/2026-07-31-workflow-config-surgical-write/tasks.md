## 1. 文字層手術改寫（決策 1、2、3）

- [x] 1.1 撰寫手術單元測試：crates/speclink-core/src/config.rs 的 #[cfg(test)]，覆蓋規格「workflow-config set 政策欄位寫入」修改後的新場景——缺鍵插於 schema 之下且與前後內容各恰一空行（相鄰處已有空行不重複補）、多缺鍵一次寫入成正典序連續區塊、註解與空行逐位元保留、檔尾既有鍵原位改值不搬家、schema 缺席插檔案最頂端、schema 底下使用者自加內容原樣後移、設 false 移除鍵行但保留其上方註解行、context 整塊替換不動其他區段、含多行 block scalar 未知頂層鍵的分段不誤判、結尾無換行與 CRLF 邊界、內部改寫驗證失敗拒寫且原檔逐位元不變。驗證：cargo test -p speclink-core 紅燈。 <!-- speclink-task:tsk_01KYV1QWSA4RYMHRSK0JBZ7Z4H -->
- [x] 1.2 重寫 update_workflow_config_text 為文字層手術：頂層鍵行分段（行首非空白且含冒號）、政策四鍵逐行原位改值／刪行（不刪上方註解行）、缺鍵按正典序插 schema 鍵行之後（空行規則見決策 2）、context／rules 以區塊代換、其餘區段原樣拼接；落檔前重新解析與目標狀態逐鍵等值比對、不等值回單行錯誤（決策 3 防呆）。函式簽名與參數語意不變。落實決策 1：行級分段的文字層手術，不用註解保留庫；決策 2：缺鍵插入位置與空行規則；決策 3：寫後解析等值驗證 fail-closed。驗證：1.1 測試綠燈。 <!-- speclink-task:tsk_01KYV1QWSAFTHN21VD23Q1T94H -->

## 2. 既有斷言翻新（決策 4）

- [x] 2.1 更新 crates/speclink-cli 既有測試：workflow-config set／context／rules 與 dry-run diff 的輸出斷言由「整檔重排」預期改為「僅目標行變動」預期（僅改斷言值，測試結構與呼叫端程式不動）。落實決策 4：測試斷言隨輸出更新的邊界。驗證：cargo test -p speclink-cli 綠燈。 <!-- speclink-task:tsk_01KYV1QWSA5FADJ01N21T2DX4H -->
- [x] 2.2 更新 apps/desktop/core settings 既有測試：設定頁寫入路徑的輸出斷言同步翻新（僅斷言值）。驗證：cargo test -p speclink-desktop-core 綠燈。 <!-- speclink-task:tsk_01KYV1QWSAJSB9M3FVYHCJWDSA -->

## 3. 收尾驗證

- [x] 3.1 全套測試：cargo test（workspace 全量，含 remote 寫回同函式的 serverfs／sqlite 相關套件）。驗證：全綠。 <!-- speclink-task:tsk_01KYV1QWSA6FCRW7ETA37VEHPC -->
- [x] 3.2 手動煙霧驗證：於本 repo 的 openspec/config.yaml 副本（scratchpad）執行 speclink workflow-config set 對「無政策鍵之檔」與「政策鍵在檔尾之檔」各一次，檢視插入位置、空行與註解保留符合規格場景。驗證：diff 逐行檢視符合「缺鍵插於 schema 之下且空行區隔」「檔尾既有鍵原位改值不搬家」兩場景。 <!-- speclink-task:tsk_01KYV1QWSAM0VT1E3N3FGT27CT -->
- [x] 3.3 speclink validate workflow-config-surgical-write 通過。驗證：無 Critical 與 Warning。 <!-- speclink-task:tsk_01KYV1QWSA2XQQFGAJPMF50FNG -->

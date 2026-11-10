## 1. 依賴與骨架

- [ ] 1.1 讓 speclink-host 依賴套件 speclink-store-port，並建立三個空模組 port_view、port_exec、teamstore_port：完成後 workspace 能建置，既有行為不變。涉及 crates/host/speclink-host/Cargo.toml、crates/host/speclink-host/src/lib.rs、crates/host/speclink-host/src/port_view.rs、crates/host/speclink-host/src/port_exec.rs、crates/host/speclink-host/src/teamstore_port.rs、Cargo.lock。驗證：`cargo build -p speclink-host` 通過；`cargo test -p speclink-host` 既有測試全部通過。 <!-- speclink-task:tsk_01M72TXM647XDSWQZSQQZDYJ9V -->

## 2. 引擎的儲存視圖

- [ ] 2.1 （Red）依 D7 新增雙路徑測試的讀取部分與覆蓋表測試：同樣的內容分別放進檔案實作與 `MemoryStorePort`，比對列出變更、單一變更的狀態、列出討論、讀詞彙表的結果與 not_found 的錯誤碼；覆蓋表列出引擎儲存介面的每個方法，每個方法至少一個經 `PortView` 的斷言（對應「engine 動詞經儲存接口執行」的「兩條路徑結果一致」）。涉及 crates/host/speclink-host/tests/port_dual_path.rs、crates/host/speclink-host/src/port_view.rs。驗證：`cargo test -p speclink-host --test port_dual_path` 失敗。 <!-- speclink-task:tsk_01M72TXM64W1R0J9QWMRKPKCCA -->
- [ ] 2.2 （Green）依 D1 實作 `PortView` 的讀取面：以固定前綴記憶列出的結果、每個鍵記憶讀取的結果、需要多份時一次讀取、把鍵解析回名稱（每一類鍵有「建立再解析得到原值」的測試）、變更的最後修改時間取各文件的最大值。對應「經儲存接口執行時只讀取用到的文件」。涉及 crates/host/speclink-host/src/port_view.rs。驗證：2.1 的讀取測試通過；單元測試用計數的儲存確認同一個前綴只列出一次、同一個鍵只讀取一次、三份 metadata 由一次讀取取得。 <!-- speclink-task:tsk_01M72TXM64NJGMT0Q1YAGE3W8V -->
- [ ] 2.3 （Red）為 D1 的寫入面寫測試：同一次執行讀得到剛寫的內容、列得到剛建立的文件、列不到剛刪除的文件；預期版本取自第一次看到的版本且同一個鍵寫兩次不變；沒看過就寫的鍵先讀取一次；刪除這次才建立的文件不留任何暫存；刪除整個變更、封存變更、封存討論的暫存內容正確。把覆蓋表裡寫入、刪除、搬移的方法補上斷言。涉及 crates/host/speclink-host/src/port_view.rs、crates/host/speclink-host/tests/port_dual_path.rs。驗證：`cargo test -p speclink-host port_view::` 的新測試失敗。 <!-- speclink-task:tsk_01M72TXM64ACWA6NJN310NR3ZE -->
- [ ] 2.4 （Green）實作 `PortView` 的寫入面（暫存、基準版本、搬移）。涉及 crates/host/speclink-host/src/port_view.rs。驗證：2.3 的測試通過；覆蓋表測試涵蓋引擎儲存介面的每一個方法並通過。有任何方法無法只用列出、讀取、提交完成時，停下來回報，不修改契約。 <!-- speclink-task:tsk_01M72TXM64FH96FGY15PFAG24V -->

## 3. 執行入口

- [ ] 3.1 （Red）為 D2 的執行入口寫測試，對應「engine 動詞經儲存接口執行」的七個情境：task done 是正好一次提交並帶 task-completed 事件；查詢不提交；封存是一次提交裡的搬移；別人先寫入時回報衝突且其他寫入與事件都沒有進入儲存；命令層拒絕時儲存不變；讀取途中儲存失敗時回報儲存的失敗且不提交（用一個在第 N 次呼叫失敗的儲存）；雙路徑的寫入、刪除變更、封存變更、封存討論結果一致。涉及 crates/host/speclink-host/src/port_exec.rs、crates/host/speclink-host/tests/port_dual_path.rs。驗證：`cargo test -p speclink-host` 的新測試失敗。 <!-- speclink-task:tsk_01M72TXM644T4V7FSCE2WBH9VN -->
- [ ] 3.2 （Green）實作 `port_exec::execute`、`PortExecution`、`ExecError`，以及收時間參數的內部版本；把 crates/host/speclink-host/src/commit.rs 的事件對映抽成不依賴 TeamStore 型別的共用函式（事件名稱與 camelCase 的內容）。涉及 crates/host/speclink-host/src/port_exec.rs、crates/host/speclink-host/src/commit.rs、crates/host/speclink-host/src/lib.rs。驗證：3.1 的測試全部通過；`cargo test -p speclink-host --test port_dual_path` 通過。 <!-- speclink-task:tsk_01M72TXM6428289ZPFB27XHX4C -->

## 4. TeamStore 轉接與橋接改接

- [ ] 4.1 （Red）為 D5 的 `TeamStorePort` 寫測試，對應「TeamStore 轉接成儲存接口」與「Host 承擔 TeamStore 的 UoW 與 event commit」：九種文件的鍵雙向對應（含規格的對應表）；一次執行只匯出一次；提交後 outbox 得到對應的 event record 且時間是提交的時間；兩次提交競寫同一文件時敗方得到衝突並帶鍵、expected、actual；對不到文件種類的鍵讀取是「沒有」、寫入是後端錯誤且 TeamStore 不變；不是綁定的範圍回報後端錯誤；最後修改時間是 0。涉及 crates/host/speclink-host/src/teamstore_port.rs。驗證：`cargo test -p speclink-host teamstore_port::` 失敗。 <!-- speclink-task:tsk_01M72TXM64BD6S95HZ2E6Y8SA1 -->
- [ ] 4.2 （Green）實作 `TeamStorePort`（列出與讀取從同一次匯出回答、自己提交後作廢記憶、提交轉成一個 unit of work）。涉及 crates/host/speclink-host/src/teamstore_port.rs。驗證：4.1 的測試全部通過。 <!-- speclink-task:tsk_01M72TXM641Q7YWTYK8ACSX21P -->
- [ ] 4.3 依 D3 把既有的橋接入口改成轉接加執行入口，對應「engine 動詞經橋接於 TeamStore 上執行」：`bridge::execute` 的簽名不變、內容改走 `TeamStorePort` 與 `port_exec::execute`；移除 `BridgeStore`、`materialize` 與 commit.rs 裡不再有呼叫端的 `begin_unit_of_work`、`commit_with_events`；`drift::spec_drift` 改用 `PortView`；`BridgeError::Store` 的內容改成 `PortError`。涉及 crates/host/speclink-host/src/bridge.rs、crates/host/speclink-host/src/commit.rs、crates/host/speclink-host/src/drift.rs、crates/host/speclink-host/tests/bridge_dual_path.rs。驗證：既有的五個雙路徑測試不改斷言就通過（只改因型別變動而必要的地方）；`cargo test -p speclink-host` 全部通過；在 crates/host 搜尋不到 `BridgeStore` 與對它的 `materialize` 的呼叫。 <!-- speclink-task:tsk_01M72TXM64YZ6CCW9MCQS0V273 -->
- [ ] 4.4 依 D4 完成 Server 的錯誤對映並盤點 driver：新增 `PortError` 到 `ApiError` 的對映（衝突是 409 與 revision_conflict、暫時無法使用是 503、後端錯誤是 500）；盤點 crates/store 底下三個 driver 在 snapshot、export、history、commit 回傳 NotFound 與 PermissionDenied 的位置，把結論寫進任務的完成說明，有的話在轉接帶出訊息並補 server 測試；搜尋 crates/host/speclink-server 與 crates/adapters/speclink-cli 的測試裡對 409 訊息全文的斷言並更新；補上「版本衝突經 Server 回報」的測試。涉及 crates/host/speclink-server/src/api/error.rs、crates/host/speclink-server/src/api/verb.rs、crates/host/speclink-host/src/teamstore_port.rs。驗證：`cargo test -p speclink-server` 全部通過（背景執行；既有的 review_api flaky 單獨重跑確認）。 <!-- speclink-task:tsk_01M72TXM6448BV5G2R81S22F1R -->

## 5. 量測

- [ ] 5.1 依 D6 新增量測測試：把本專案的 speclink 資料夾載入 `MemoryStorePort`（路徑換成鍵、封存依鍵對照表、Windows 的分隔符換成 `/`），外包計數的儲存，對列出變更、單一變更的狀態、執行計畫、列出討論、新增一份 artifact、勾選一個任務、封存一個變更、搜尋討論各跑一次並印出列出次數、讀取次數、讀取的鍵數與位元組數、提交次數。斷言三項：三個讀取類命令不讀 `archive/` 底下的鍵也不列出 `archive/changes/`；每個命令同一個前綴最多列出一次、同一個鍵最多讀取一次；寫入類命令提交正好一次。涉及 crates/host/speclink-host/tests/port_cost.rs。驗證：`cargo test -p speclink-host --test port_cost -- --nocapture` 通過並印出數字表。 <!-- speclink-task:tsk_01M72TXM64CGMMJ6Q328RGNC6E -->
- [ ] 5.2 把 5.1 印出的數字表與結論寫進這個變更的 design.md 新增的「量測結果」一節，依 D6 的判定規則下結論：三項斷言成立且三個讀取類命令對儲存的呼叫各不超過 10 次時寫「不加快取」；否則停下來向使用者回報，不自行加快取。同時列出「沒看過就寫而多出來的讀取」有幾次。涉及 speclink/changes/engine-on-store-port/design.md。驗證：design.md 有「量測結果」一節，內含八個命令的數字與一句結論；`speclink validate engine-on-store-port` 通過。 <!-- speclink-task:tsk_01M72TXM646HMDHQH45H17W992 -->

## 6. 收尾

- [ ] 6.1 對新增的公開介面（`port_exec::execute`、`PortExecution`、`ExecError`、`TeamStorePort`）套用 sharp-edges 稽核清單（用 `speclink instructions --skill audit` 取得）：確認 `PortView` 沒有被公開、儲存錯誤不會被當成「沒有」而繼續提交、錯誤訊息不帶出文件內容、轉接不會把別的範圍的資料回給呼叫端。找到的問題在這個任務修正並補測試。涉及 crates/host/speclink-host/src/port_exec.rs、crates/host/speclink-host/src/port_view.rs、crates/host/speclink-host/src/teamstore_port.rs。驗證：每一項的結論寫在任務的完成說明裡；`cargo test -p speclink-host` 通過。 <!-- speclink-task:tsk_01M72TXM64DX4XD3CSFK39VKWY -->
- [ ] 6.2 最後驗證 D1 到 D7 的驗收條件。涉及 crates/host/speclink-host、crates/host/speclink-server。驗證：`cargo test -p speclink-host` 通過；`cargo test -p speclink-server` 通過；`cargo build --workspace` 通過且沒有新的警告；`node --test scripts/*.test.mjs scripts/*/*.test.mjs` 通過。 <!-- speclink-task:tsk_01M72TXM64Y9YQQWYSGJ574EHV -->

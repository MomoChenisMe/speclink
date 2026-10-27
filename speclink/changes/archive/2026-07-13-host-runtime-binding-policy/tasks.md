## 1. host crate 與 ExecutionContext（design 決策一：speclink-host 為獨立 crate，依賴方向 host → core 與 host → store；決策五：binding 型別與 fail-closed 驗證，本地模式映射 default binding）

- [x] 1.1 建立 crate 並撰寫失敗測試，覆蓋「Project 與 Repo binding 驗證 fail closed」與 ExecutionContext 型別組成：SpeclinkExecutionContext 含 actor／project／repo／mode／resolved policy（帶 digest）；binding 多義時拒絕並列出候選、缺失與無權限各回對應原因；本地 fs 模式零設定映射 default project/repo（crates/speclink-host/src/context.rs、crates/speclink-host/src/binding.rs 的 #[cfg(test)]；根 Cargo.toml 追加 workspace 成員）。cargo test -p speclink-host 觀察紅燈。
- [x] 1.2 實作 ExecutionContext 與 binding 解析驗證，1.1 轉綠；驗證 cargo build --release 全 workspace 編譯通過。

## 2. policy 與模式的注入（design 決策四：policy 與模式的 env 層由 host 注入，core 保留純函式）

- [x] 2.1 撰寫失敗測試，覆蓋「Engine 規格面不讀 process env 與 git identity」的政策與模式面：以注入的環境覆寫集合（含 SPECLINK_TDD）解析政策時 process env 的相反值無效果；store 模式解析以注入值運作（crates/speclink-host/src/policy.rs、crates/speclink-core/src/config.rs 與 crates/speclink-core/src/workspace.rs 的注入形測試）。紅燈。
- [x] 2.2 實作 EffectiveWorkflowPolicy（包 core ResolvedPolicy 與政策文件 digest，不進任何現有輸出）；core 的 EnvOverrides 直讀 process env 便利建構移出非測試碼、SPECLINK_STORE_URL 讀取上移 host 邊界（core 保留注入形純函式），2.1 轉綠；驗證 cargo test -p speclink-core 與 cargo test -p speclink-host 全綠。

## 3. actor 注入與 execute 簽名（design 決策二：ExecutionContext 由 Host 邊界一次解析，command 參數不可覆寫 identity；決策三：git identity 搬遷至 host，core 流程改收明確 actor）

- [x] 3.1 撰寫失敗測試，覆蓋「ExecutionContext 由 Host 解析且不可覆寫」：Command 封閉 enum 不含 actor 與 policy 欄位；new change 的 created_by 章只隨 context actor 改變；無身分時沿用現行無章行為；in-progress 的 started_by 與 discuss 建立者章同斷言（crates/speclink-core/src/command/mod.rs、crates/speclink-core/src/newcmd.rs、crates/speclink-core/src/inprogress.rs、crates/speclink-core/src/discuss.rs 測試）。紅燈。
- [x] 3.2 實作：git identity 解析函式自 crates/speclink-core/src/util.rs 移入 crates/speclink-host；execute 簽名攜 ExecutionContext；newcmd、archive、demo、inprogress、discuss 五處呼叫點改收明確 actor，殘留呼叫以編譯錯誤窮舉跟進（crates/speclink-core/src/archive.rs、crates/speclink-core/src/demo.rs 一併），3.1 轉綠；驗證 cargo test -p speclink-core 全綠。
- [x] 3.3 撰寫並轉綠靜態盤點斷言，覆蓋「Engine 規格面不讀 process env 與 git identity」的零命中場景：speclink-core 非測試碼對 process env 讀取與 git config 身分呼叫零命中（以測試化的原始碼盤點或 CI grep 步驟落地，測試位置 crates/speclink-core/tests/）。

## 4. lifecycle gate（design 決策六：lifecycle gate 狀態機落在 host，本地為唯讀映射）

- [x] 4.1 撰寫失敗測試，覆蓋「lifecycle gate 是單一裁決點」：六站封閉 enum；合法路徑 drafting→review→ready→applying→verified→archived 逐步全允許；drafting 直跳 verified 拒絕並指出缺少的中間站；本地三態唯讀推導（未開工＝drafting、已標記開工＝applying、已封存＝archived）且不寫入檔案（crates/speclink-host/src/gate.rs 測試）。紅燈。
- [x] 4.2 實作 gate 狀態機、transition 裁決函式與本地站點推導，4.1 轉綠；驗證 cargo test -p speclink-host 全綠。

## 5. TeamStore commit 骨架（design 決策七：Host 對 TeamStore 的 commit 骨架以整合測試交付）

- [x] 5.1 撰寫失敗測試，覆蓋「Host 承擔 TeamStore 的 UoW 與 event commit」：以 ExecutionContext 對 speclink-store in-memory reference 開 UoW、寫一份文件帶一筆領域事件 commit 後，自 cursor 0 重讀 outbox 得恰一筆含 actor 與事件名的 record、文件與事件同 commit 可見；兩個 Host commit 以相同 expected revision 競寫時敗方錯誤保留 revision_conflict 與 expected/actual（crates/speclink-host/src/commit.rs 整合測試）。紅燈。
- [x] 5.2 實作 Host commit 組合路徑與「core typed event → store event record」單向映射（不接線任何現行 CLI 流程），5.1 轉綠；驗證 cargo test -p speclink-host 全綠。

## 6. 組裝點遷移與輸出凍結（design 決策八：輸出凍結以 baseline exe 雙沙盒對照驗證）

- [x] 6.1 建置並保存 baseline exe（cargo build --release -p speclink-cli）於非 scratchpad 位置，準備樣本 workspace（含設定 SPECLINK_TDD 與 git 身分的情境）並記錄對照步驟。
- [x] 6.2 CLI 組裝點改經 host：進入點解析一次 ExecutionContext 再呼叫 execute（crates/speclink-cli/src/commands.rs，clap 定義與渲染不動）；覆蓋「組裝點遷移輸出凍結」——對樣本 workspace 逐動詞（人眼與 --json）與 baseline diff 為空、exit code 相同；parity 31 項／color 16 項／twin 8 情境全綠。
- [x] 6.3 Node dispatch 組裝點改經 host（crates/speclink-node/src/lib.rs）：argv 詞彙、envelope 形狀與錯誤碼不變；驗證 crates/speclink-node 內 npm run build 與 npm test 全綠。

## 7. 全量收尾

- [x] 7.1 對新公開 API 跑 sharp-edges 稽核檢查表（speclink instructions --skill audit）並修正發現；驗證 cargo test --workspace 與 npm run test:all 全綠；git diff --stat 對照 proposal 的 Impact 清單檢查改動面無溢出。

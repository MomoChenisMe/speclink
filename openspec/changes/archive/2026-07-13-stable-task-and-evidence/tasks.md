## 1. 任務解析與 ID 形制（design 決策一：ID 形制為 tsk_ 前綴 ULID，內嵌於任務行尾註解）

- [x] 1.1 撰寫失敗測試：parse 認得任務行尾的 speclink-task 註解並剝離出顯示文字與 stable ID、無註解任務的 ID 為空、同檔重複 ID 可被偵測列出；ULID 產生器輸出 tsk_ 前綴 26 字元且時間排序（crates/speclink-core/src/tasks.rs 的 #[cfg(test)]；Cargo.toml 引入 ulid 輕量依賴）。cargo test -p speclink-core 觀察紅燈。
- [x] 1.2 實作註解解析、顯示剝離、重複偵測與 ID 產生，1.1 轉綠；驗證 cargo test -p speclink-core 全綠。

## 2. 蓋章時機（design 決策二：蓋章時機——Engine 產出全檔蓋章、task done 單行補章、不做全檔強制遷移）

- [x] 2.1 撰寫失敗測試，覆蓋「任務行內嵌不可變 stable ID」與「task done 對無 ID 目標行單行補章」：new artifact tasks 寫入後全檔任務各帶唯一 ID 且描述不變；重排兩任務後 ID 不變僅序數互換；對全檔無 ID 的 tasks.md 執行 task done 3 後僅第 3 行變更（勾選＋ID 註解）、其餘行逐位元不變；task undone 不補章（crates/speclink-core/src/command/mod.rs 與 crates/speclink-core/src/tasks.rs 測試）。紅燈。
- [x] 2.2 實作產出全檔蓋章與 task done 單行補章，2.1 轉綠；驗證 cargo test -p speclink-core 全綠。

## 3. 雙值域定址與事件（design 決策三：定址雙值域，stable ID 為第一級身分）

- [x] 3.1 撰寫失敗測試，覆蓋「定址接受 ordinal 與 stable ID 雙值域」「重複 stable ID 使 task 動詞拒絕」「任務事件載荷攜 stable ID」：tsk_ 定址在重排後仍命中原任務而 ordinal 命中新位置任務；查無此 ID 回與超界對稱的錯誤；重複 ID 時 done／undone 拒絕點名重複值且檔案不變；task-completed／task-uncompleted 事件任務識別為 stable ID（undone 對無 ID 任務為序數字串）（crates/speclink-core/src/command/mod.rs 測試）。紅燈。
- [x] 3.2 實作雙值域解析（純數字→ordinal、tsk_ 前綴→ID 查找、其餘沿現行非法錯誤）、重複拒絕與事件載荷，3.1 轉綠；驗證 cargo test -p speclink-core 全綠。
- [x] 3.3 撰寫並轉綠 CLI 整合測試，覆蓋「任務取消勾選動詞」的值域擴充：speclink task done 與 task undone 以 tsk_ ID 操作成功且 --json 形狀與數字值域一致、行尾註解原文保留；既非數字亦非 tsk_ 前綴的值回非法 task id 錯誤；數字值域的人眼與 --json 輸出和本變更前逐位元一致（crates/speclink-cli/tests/，渲染與參數處理落 crates/speclink-cli/src/commands.rs）。

## 4. evidence 記錄 v2（design 決策四：evidence 演進 TouchedRecord 為 v2 schema，同檔向下相容）

- [x] 4.1 撰寫失敗測試，覆蓋「task done 寫入逐任務 evidence」：完成任務後 touched 記錄含 version 與該任務 entry（taskId、actor、repo、headCommit、touchedFiles、basisDigests 三項、recordedAt UTC）；無版本標記的 v1 舊檔讀取正常且檔案清單語意不變；task undone 不寫不改任何記錄（crates/speclink-core/src/tasks.rs 測試，actor 與 repo 經 ExecutionContext 注入）。紅燈。
- [x] 4.2 實作 TouchedRecord v2 schema（寫入一律 v2、讀取相容 v1）與 basis digest 計算（spec／tasks／policy），4.1 轉綠；驗證 cargo test -p speclink-core 全綠。

## 5. VerifyBundle 與 stale 判定（design 決策五：VerifyBundle 與 stale 判定落在 speclink-host）

- [x] 5.1 撰寫失敗測試，覆蓋「VerifyBundle 固定驗證基準」與「evidence 的 stale 判定」：對同一 change 連續產生兩份 bundle 的三項 digest 逐項相同、修改 delta spec 後 spec digest 改變；tasks.md 修改後對新 bundle 判 stale 且僅列 tasks digest 不符；全符判有效（crates/speclink-host/src/evidence.rs 的 #[cfg(test)]）。紅燈。
- [x] 5.2 實作 produce_verify_bundle 與 judge_staleness（host 層錯誤型別、不動命令層五碼、不接線 CLI），5.1 轉綠；驗證 cargo test -p speclink-host 全綠。

## 6. archive trace 來源演進（design 決策六：archive trace 由 evidence 建立、輸出格式凍結，gate 檢查不強制）

- [x] 6.1 撰寫並轉綠測試，覆蓋「archive trace 由 evidence 建立」：v2 evidence 聚合出的 trace 檔案清單與相同內容的 v1 記錄產生者逐位元同構；v1 舊檔走現行路徑無錯誤；host 的 archive gate 檢查函式對 stale evidence 回帶原因拒絕、對齊全狀態通過，且本地 speclink archive 不受阻擋（crates/speclink-core/src/archive.rs 與 crates/speclink-host/src/evidence.rs 測試）。

## 7. 桌面與 UI stable ID 化（design 決策七：桌面與 UI 以 stable ID 呈現與操作）

- [x] 7.1 撰寫失敗測試，覆蓋「UI 剝離 ID 註解並以 stable ID 操作」：tasks 解析剝離註解後顯示文字與無註解時相同、清單項以 tsk_ ID 作 key（無 ID 退回 ordinal key）、勾選請求攜 stable ID（無 ID 任務走 ordinal 相容）、樂觀就地改寫保留行尾註解原文（packages/ui/src/tasks.ts 與 packages/ui/src/components/TaskList.tsx 的 vitest；apps/desktop/core/src/verbs.rs 雙值域測試）。紅燈。
- [x] 7.2 實作 UI 剝離與 stable ID 定址、desktop core 任務動詞雙值域跟進，7.1 轉綠；驗證 npm test -w packages/ui 與 npm test -w apps/desktop 全綠。

## 8. 全量回歸收尾（design 決策八：輸出凍結與刻意變更清單）

- [x] 8.1 對新公開 API 跑 sharp-edges 稽核檢查表（speclink instructions --skill audit）並修正發現；以 baseline exe 對照樣本 workspace 的數字 ordinal task 動詞情境逐位元一致；parity 31 項／color 16 項／twin 8 情境全綠；npm run test:all 全綠；git diff --stat 對照 proposal Impact 檢查改動面無溢出。

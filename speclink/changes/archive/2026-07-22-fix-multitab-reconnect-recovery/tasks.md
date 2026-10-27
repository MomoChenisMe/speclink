<!--
Each task description MUST state:
- the behavior or contract being delivered (what is observably true when the
  task is complete), and
- the verification target that proves completion (test, CLI invocation,
  analyzer check, manual assertion, or content review).

File paths are supporting context for locating the work, never the task
itself. "Edit file X" is not a valid task — it is missing both behavior and
verification.
-->

## 1. Rust 認證恢復 TDD

- [x] 1.1 RED：在 `apps/desktop/src-tauri/tests/remote_runtime.rs` 以同步屏障重現「恢復自動收斂並清除 stale」的同 connection 兩 caller 競態，分別覆蓋 bearer 尚未取得及兩個失效 bearer 請求交錯回 401，斷言修正前會重複消耗一次性 refresh credential、至少一方失敗或進入 `needs-reauth`；執行 `cargo test -p speclink-desktop --test remote_runtime <test-name> -- --exact` 確認因實際 bug 失敗。（CLI：無；`speclink-core`／`speclink-cli`：無變更） <!-- speclink-task:tsk_01KY4B09E2B392V66K1TMEBKVM -->
- [x] 1.2 GREEN：依「每個 connection 的 refresh singleflight 與雙重檢查」在 `apps/desktop/src-tauri/src/remote.rs` 做最小實作，使兩 caller 只輪替一次、共用新 bearer 且 family 維持有效；以 1.1 的測試轉綠並斷言沒有 `needs-reauth` state event。（CLI：無；`speclink-core`／`speclink-cli`：無變更） <!-- speclink-task:tsk_01KY4B09E29S8MVXGAXFS6Q1M6 -->
- [x] 1.3 REGRESSION：在 `apps/desktop/src-tauri/tests/remote_runtime.rs` 鎖定真正 revoked family、暫時性 refresh 失敗與 PAT 的既有安全語意；測試應先通過，再暫時破壞對應分類分支確認測試會失敗、還原後執行 `cargo test -p speclink-desktop --test remote_runtime` 全綠，證明 singleflight 未吞掉 reauth 或誤分類錯誤。（CLI：無；`speclink-core`／`speclink-cli`：無變更） <!-- speclink-task:tsk_01KY4B09E2579A81JD6YSFP7AW -->
- [x] 1.4 REFACTOR：只簡化本次新增的 rotation 協調與鎖定範圍，確保一般資料請求與不同 origin 不被序列化；以 `cargo test -p speclink-desktop --test remote_runtime` 與 `cargo clippy -p speclink-desktop --test remote_runtime --no-deps -- -D warnings -A clippy::unused-unit -A clippy::cloned-ref-to-slice-refs` 驗證行為不變且本 change 路徑無警告（兩項 allow 僅隔離既有 `panel.rs` 與既有 PAT assertion 警告）。（影響：`apps/desktop/src-tauri/src/remote.rs`、`apps/desktop/src-tauri/tests/remote_runtime.rs`；CLI：無） <!-- speclink-task:tsk_01KY4B09E231PG0WWBHD27019D -->

## 2. Workspace snapshot 所有權 TDD

- [x] 2.1 RED：在 `apps/desktop/src/__tests__/store.test.ts` 或 `apps/desktop/src/__tests__/session.test.ts` 以 A、B 兩個 WorkspaceSession 重現「每個 session 自帶 dataSource 且 Rust 側無 current-root 全域」的內容洩漏，覆蓋 B 有自己 snapshot、B 從未成功載入且 refresh 失敗、切回 A 三種行為；執行對應 Vitest 檔案確認測試因仍顯示 A 內容而失敗。（CLI：無；`speclink-core`／`speclink-cli`：無變更） <!-- speclink-task:tsk_01KY4B09E220159XG9M7K9Z6K5 -->
- [x] 2.2 GREEN：依「依 locator 隔離最後成功 snapshot 與可見投影」在 `apps/desktop/src/store.ts` 建立每 locator 的記憶體 snapshot，讓 activeKey 切換時同步顯示目標自己的內容或安全未載入狀態，且關閉分頁即清除；以 2.1 的測試轉綠並確認 localStorage shape 不變。（CLI：無；`speclink-core`／`speclink-cli`：無變更） <!-- speclink-task:tsk_01KY4B09E2G7TVPJED7AN8PEP1 -->
- [x] 2.3 RED：在 `apps/desktop/src/__tests__/store.test.ts` 與必要的 `apps/desktop/src/__tests__/remoteResilience.test.tsx` 加入可控制 Promise 完成順序的 A 晚回、同一 B 較舊 refresh 晚回、切至 `needs-reauth` 後搜尋命中／detail 不殘留案例；執行對應 Vitest 檔案確認至少一個案例因過期結果或衍生內容覆寫而失敗。（CLI：無；`speclink-core`／`speclink-cli`：無變更） <!-- speclink-task:tsk_01KY4B09E2DWGAH45VJTD6G46F -->
- [x] 2.4 GREEN：依「以來源世代守衛非同步結果而非取消請求」在 `apps/desktop/src/store.ts` 做每 locator latest-wins 結算與 activeKey 所有權檢查，並只在必要時於 `apps/desktop/src/App.tsx` 調整安全未載入呈現，使過期／背景結果不覆寫 active 分頁且搜尋、詳情、待確認狀態不跨 workspace；以 2.3 的測試全部轉綠。（CLI：無；`speclink-core`／`speclink-cli`：無變更） <!-- speclink-task:tsk_01KY4B09E2E0JHDY2ZZJ1W8AZD -->
- [x] 2.5 REFACTOR：只整理本次新增的 snapshot 投影與世代守衛輔助邏輯，不建立通用快取層；以 `npm test -w apps/desktop -- store.test.ts session.test.ts remoteResilience.test.tsx remoteWorkspaceRecovery.test.tsx` 驗證各 session snapshot、reauth 與 recovery 行為維持全綠。（影響：`apps/desktop/src/store.ts`、必要的 `apps/desktop/src/App.tsx` 與相關測試；CLI：無） <!-- speclink-task:tsk_01KY4B09E2Q830S1JKBC7AP82E -->

## 3. 整合回歸與真實 Desktop 驗收

- [x] 3.1 依「TDD 回歸矩陣與既有安全行為」在 `apps/desktop/src-tauri/tests/phase3_chain.rs` 鎖定同來源多 session 恢復，使自動鏈可觀察一次 rotation、兩 workspace 讀取成功且無 reauth；先以 mutation check 證明新增斷言能抓到破壞，再還原並執行 `cargo test -p speclink-desktop --test phase3_chain` 驗證通過，且不修改 `openspec/changes/phase3-e2e/` artifact。（CLI：無；`speclink-core`／`speclink-cli`：無變更） <!-- speclink-task:tsk_01KY4B09E26R3D48B9KF5AGK3Q -->
- [x] 3.2 執行 `cargo test -p speclink-desktop`、`npm test -w apps/desktop` 與 `npm run build -w apps/desktop`，驗證 Rust Desktop 與 React Desktop 全套測試／建置通過；若受保護 crates 在 apply preflight 時為乾淨，則要求 `git diff -- crates/speclink-core crates/speclink-cli crates/speclink-server crates/speclink-protocol` 為空，若原本已有其他 change 的 dirty diff，則要求測前／測後 `git diff --binary` 雜湊一致並檢查本 change 實際修改僅在 Desktop 路徑，以等價證明 scope boundary。（影響：驗證本 change 的 Desktop 路徑；CLI：無） <!-- speclink-task:tsk_01KY4B09E2SHZV7AYJKT6BZRK0 -->
- [x] 3.3 在使用者再次確認未使用螢幕後，以真實 Desktop 開啟同來源兩個 remote 分頁，重跑 Server 中斷／恢復、切換既有 snapshot 與無 snapshot 的 `needs-reauth` 分頁；人工斷言兩分頁自動恢復、credential family 未自撤銷、畫面從未顯示另一 workspace 的變更／規格／討論／搜尋／詳情，並記錄驗收結果。（影響：真實 Desktop 執行期；CLI：無；不修改其他 change artifact） <!-- speclink-task:tsk_01KY4B09E2C8C886YRR8NPBB9P -->
- [x] 3.4 執行 `target/debug/speclink analyze fix-multitab-reconnect-recovery --json` 與 `target/debug/speclink validate fix-multitab-reconnect-recovery`，驗證無 Critical／Warning、所有 task 可追蹤且 proposal、design、兩份 delta spec 與實作結果一致。（影響：`openspec/changes/fix-multitab-reconnect-recovery/`；CLI：僅驗證 Speclink change，不變更 CLI 行為） <!-- speclink-task:tsk_01KY4B09E2YCA12Q93BP4TMSV5 -->

## 1. 設定與守門（D1、D2）

- [ ] 1.1 RED 先行：在 scripts/release/delivery-gate.test.mjs 新增一條測試，守「開發建置不保留除錯中間檔」——逐行找出根 Cargo.toml 的 [profile.dev] 區段（標頭到下一個 [ 標頭之間），斷言區段存在且 debug 的值為 false 或 0；失敗訊息寫明 dev profile 必須關閉除錯資訊，原因是避免 .o 在 target/debug/deps 累積（D2，不引入 TOML 解析套件）。驗證：執行 node --test scripts/*.test.mjs scripts/*/*.test.mjs，這條新測試在 Cargo.toml 尚未設定前失敗，訊息含上述原因，其他測試維持綠 <!-- speclink-task:tsk_01M4YJP1R6T7XVZ1K90K8BMN49 -->
- [ ] 1.2 在根 Cargo.toml 新增 [profile.dev] 區段，只設 debug = false（D1：不用 macOS 限定的 packed、不用 line-tables-only），使 dev 與 test 建置不再產生 .o；不新增 [profile.release]、不新增 .cargo/config.toml，release 建置維持原狀。驗證：node --test scripts/*.test.mjs scripts/*/*.test.mjs 全綠（1.1 的守門轉綠）；git diff Cargo.toml 只有這兩行新增 <!-- speclink-task:tsk_01M4YJP1R6YWCNMX899TWDFRED -->

## 2. 清空與行為確認（D3）

- [ ] 2.1 一次清空主 checkout 的 target 目錄（D3）：先以 pgrep 檢查沒有其他 session 的 cargo 或 rustc 行程在跑，有就等它結束；再於背景執行 cargo clean（要刪兩百多萬個檔案，可能要數分鐘）。這是 repo 外的一次性動作，不進 commit。驗證：cargo clean 以 exit code 0 結束，target 目錄不存在或為空 <!-- speclink-task:tsk_01M4YJP1R6H0PSN8GR42XS9ZQ8 -->
- [ ] 2.2 確認「開發建置不保留除錯中間檔」的檔案系統效果：以 cargo test -p speclink-core --no-run 建置後，不排序列出 target/debug/deps 並計數 .o 為 0；再對 crates/engine/speclink-core 的一個原始檔做 touch 後重建，.o 仍為 0（重新編譯不累積）。另在 scratchpad 建一個同樣設定 debug = false 的小 crate，讓測試 assert 失敗，確認 panic 訊息帶檔名、行號與欄號。驗證：兩次計數皆為 0，panic 訊息含 src/lib.rs 的行號與欄號 <!-- speclink-task:tsk_01M4YJP1R6EA47Y1A0YWNGJMGA -->
- [ ] 2.3 回歸對照：關閉除錯資訊不改變 CLI 的人眼與 --json 輸出，golden 快照與 CLI 整合測試的期望值不變。驗證：cargo test -p speclink-core（含 crates/engine/speclink-core/tests/golden）與 cargo test -p speclink-cli --test it 全綠，且未更新任何 golden 檔 <!-- speclink-task:tsk_01M4YJP1R6H6Y24BNR31FZCYNW -->

## 3. 量測與收尾

- [ ] 3.1 量測本機完整測試耗時（這是討論 test-speed-and-wording 定下的驗收目標，不是一般 task 驗證步驟；其他變更的任務驗證仍不跑 test:all）：在清空後的主 checkout 背景執行 npm run test:all（不接管線，事後從輸出檔找失敗行），記錄總耗時與 cargo 段耗時，目標約 10 分鐘；packages/ui drawer 逾時 flaky 若出現，只單獨補跑該站。驗證：每一段 exit code 為 0，回報冷建置的總耗時；跑完後 target/debug/deps 的 .o 計數仍為 0 <!-- speclink-task:tsk_01M4YJP1R61G2DJ3QFAR2MF61V -->
- [ ] 3.2 收尾確認：本變更在 repo 內只動到 Cargo.toml、scripts/release/delivery-gate.test.mjs 與 openspec/changes/dev-profile-no-debuginfo/ 底下的 artifacts。驗證：node --test scripts/*.test.mjs scripts/*/*.test.mjs 全綠，git status 只列出上述檔案（平行 session 的檔案另行標出、不納入） <!-- speclink-task:tsk_01M4YJP1R6XMMXWEN7PDFATFNX -->

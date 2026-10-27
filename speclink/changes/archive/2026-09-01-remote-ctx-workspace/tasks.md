## 1. RemoteCtx 帶 workspace 與六處重取遷移

- [x] 1.1 在 crates/speclink-cli/src/remote_base.rs 為 RemoteCtx 加 pub(crate) ws: Workspace 欄位，remote_ctx() 把探索到的 Workspace 區域變數移入回傳值（design D1）；cargo build -p speclink-cli 編譯通過 <!-- speclink-task:tsk_01M1DZ3RS07VN6HG05K415ZGXJ -->
- [x] 1.2 遷移 crates/speclink-cli/src/verbs/station.rs 三處（review prepare、station scope、station stamp 指令）：require_workspace 與 discover 加自訂 bail 改為讀 ctx.ws，stamp 的缺席 bail 整段刪除（design D2）；station.rs 孤兒化的 require_workspace import 清除 <!-- speclink-task:tsk_01M1DZ3RS0EC9FCSE0KKJP5HQR -->
- [x] 1.3 遷移 crates/speclink-cli/src/verbs/checks.rs（drift 指令：探索改 Some(&ctx.ws)，git_available 過濾保留）、crates/speclink-cli/src/verbs/instructions.rs（instructions 指令的 context 投影：靜默略過分支刪除）、crates/speclink-cli/src/verbs/progress.rs（task done 指令：touched 與 head 直接自 ctx.ws 計算，空集合語意不變）（design D2） <!-- speclink-task:tsk_01M1DZ3RS0PY1VY9S4F5Q607TZ -->

## 2. remote_resolve_change 呼叫儀式收斂

- [x] 2.1 把 crates/speclink-cli/src/verbs/new.rs（new artifact）、crates/speclink-cli/src/verbs/progress.rs（task done、task undone）、crates/speclink-cli/src/verbs/documents.rs（artifact cat）四處 remote 臂的外層 match 改為 let-else 一行形（design D3），各臂提示字串逐字保留 <!-- speclink-task:tsk_01M1DZ3RS03PMW5B2VKMF20FQM -->
- [x] 2.2 遷移完成斷言：grep 確認 crates/speclink-cli/src/verbs/ 內 discover_cwd 只剩 toolchain.rs 與 connection.rs 的合法使用；cargo build -p speclink-cli 零 dead-code 與零 unused-import warning <!-- speclink-task:tsk_01M1DZ3RS0Q4NXFHS2F36ZJRJA -->

## 3. 凍結輸出驗證

- [x] 3.1 cargo test -p speclink-cli --test it 全綠——含 remote_verb_parity 凍結對照，兩模式輸出位元級一致即本刀「行為零變更」的驗收 <!-- speclink-task:tsk_01M1DZ3RS07J3PDRREM3H7J0HC -->

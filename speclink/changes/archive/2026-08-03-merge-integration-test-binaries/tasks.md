## 1. speclink-core 合併（最小 crate 先行）

- [x] 1.1 先以 cargo test -p speclink-core -- --list 記下整合測試總數；git mv 五個測試檔至 `crates/speclink-core/tests/it/`，新增 `crates/speclink-core/tests/it/main.rs` 以 mod 逐一宣告五個模組。行為：cargo test -p speclink-core --no-run 的整合測試執行檔由 5 支變 1 支（名為 it）。驗證：cargo test -p speclink-core 全綠，且 --test it -- --list 的測試總數與搬遷前記錄一致。 <!-- speclink-task:tsk_01KYY54KZWCM10WWQN5ARQERQG -->

## 2. speclink-store-fs 合併

- [x] 2.1 先記錄 cargo test -p speclink-store-fs -- --list 基準；git mv 五個測試檔至 `crates/speclink-store-fs/tests/it/`，新增 `crates/speclink-store-fs/tests/it/main.rs` 以 mod 逐一宣告。驗證：cargo test -p speclink-store-fs 全綠、測試總數與基準一致、整合測試執行檔 1 支。 <!-- speclink-task:tsk_01KYY54KZWE82ZGV0QRGF7SY6D -->

## 3. speclink-remote 合併

- [x] 3.1 先記錄 cargo test -p speclink-remote -- --list 基準；git mv 十一個測試檔至 `crates/speclink-remote/tests/it/`，新增 `crates/speclink-remote/tests/it/main.rs` 以 mod 逐一宣告。驗證：cargo test -p speclink-remote --tests 全綠、測試總數與基準一致、整合測試執行檔 1 支（既有 doctest 編譯錯誤屬平行 change，不在本 task 修復，故驗證範圍限 --tests）。 <!-- speclink-task:tsk_01KYY54KZWEC854ZE7Y0THB8GK -->

## 4. speclink-cli 合併

- [x] 4.1 先記錄 cargo test -p speclink-cli -- --list 基準；git mv 二十七個測試檔至 `crates/speclink-cli/tests/it/`，新增 `crates/speclink-cli/tests/it/main.rs` 以 mod 逐一宣告。驗證：cargo test -p speclink-cli 全綠、測試總數與基準一致、整合測試執行檔 1 支。 <!-- speclink-task:tsk_01KYY54KZWVE7F5E56S78Z5CXV -->

## 5. desktop（speclink-desktop）合併

- [x] 5.1 先記錄 cargo test -p speclink-desktop -- --list 基準；git mv 八個測試檔與 common 模組至 `apps/desktop/src-tauri/tests/it/`，新增 `apps/desktop/src-tauri/tests/it/main.rs`：宣告 mod common 一次並逐一宣告八個測試模組，八檔各自的 mod common 宣告刪除、引用改 crate::common 路徑。行為：HARNESS_GATE 由 common 原樣搬移，合併後單一 static Mutex 覆蓋全部 harness 測試，序列化語意不變。驗證：cargo test -p speclink-desktop 全綠、測試總數與基準一致、整合測試執行檔 1 支。 <!-- speclink-task:tsk_01KYY54KZW0MVPQ38FS4ESVH8C -->

## 6. speclink-server 合併（最大宗；postgres_store 原地不動）

- [x] 6.1 先以 cargo test -p speclink-server -- --list 記錄基準與現有失敗清單；git mv 四十七個 libtest 檔與 common 模組至 `crates/speclink-server/tests/it/`，新增 `crates/speclink-server/tests/it/main.rs`：宣告 mod common 一次並逐一宣告四十七個測試模組，各檔 mod common 宣告刪除、引用改 crate::common 路徑；`crates/speclink-server/tests/postgres_store.rs` 與 `crates/speclink-server/tests/pg/` 原地不動，其 Cargo.toml [[test]] 宣告不改。驗證：cargo test -p speclink-server 的整合測試執行檔恰為 2 支（it 與 postgres_store）、測試總數與基準一致、失敗清單不比搬遷前多（e2e_cli 既有的連線字樣斷言失敗屬平行 change，不因搬遷修改其內容）。 <!-- speclink-task:tsk_01KYY54KZWKQSW9TW3KP5ZESK3 -->

## 7. speclink-store-postgres 客製 harness 統一

- [x] 7.1 先讀 `crates/speclink-store-postgres/tests/support/mod.rs` 的 run 函式，確認 skip 回報與失敗匯總語意；六個測試檔搬遷至 `crates/speclink-store-postgres/tests/it/`，各檔 fn main 改為公開的套件執行函式（沿用 support 迷你 harness 的（名稱, 函式指標）清單形態），新增 `crates/speclink-store-postgres/tests/it/main.rs` 依序執行六個模組清單並沿用既有 skip 回報。行為：無 PostgreSQL 配置時整支回報 skipped 且 exit code 與現行一致。驗證：本機（無 PG）跑 cargo test -p speclink-store-postgres 輸出 skipped 語意與搬遷前逐字等價。 <!-- speclink-task:tsk_01KYY54KZWQQ6MQ5QVB1GQAC33 -->
- [x] 7.2 `crates/speclink-store-postgres/Cargo.toml` 六個 [[test]] 區塊收斂為一個（name 為 it、harness = false）。驗證：cargo test -p speclink-store-postgres --no-run 僅產出 1 支整合測試執行檔。 <!-- speclink-task:tsk_01KYY54KZWAC8AKK91NC08NHZG -->

## 8. napi build 的 target 隔離

- [x] 8.1 新增 `crates/speclink-node/.cargo/config.toml` 設定 build.target-dir 為相對路徑 target（即 crates/speclink-node/target/），並確認 `.gitignore` 涵蓋該目錄（root 的 target/ 規則若不涵蓋子目錄則補一行）。行為：napi build 的中間產物完全落在 crates/speclink-node/target/，不再寫 workspace target/。驗證：依序跑 cargo test --workspace --no-run、npm --prefix crates/speclink-node run build、再跑一次 cargo test --workspace --no-run——第二次輸出零 Compiling 行（指紋未被 napi 弄髒），且 crates/speclink-node/target/ 出現中間產物。 <!-- speclink-task:tsk_01KYY54KZWJNCQF4QW6P5F2QYP -->

## 9. 全量驗證

- [x] 9.1 cargo test --workspace --no-run 清點整合測試執行檔恰為 11 支（core、store-fs、remote、cli、desktop、store-postgres 各 1，server 2，單檔 crate speclink-fs／speclink-host／speclink-store-sqlite 各 1）。驗證：清單逐支對照上述分佈，無多無少。 <!-- speclink-task:tsk_01KYY54KZWPMT6TW2GXY0VTJNP -->
- [x] 9.2 npm run test:all 完整走一輪全綠（既有平行 change 的失敗除外，以 6.1 記錄的失敗清單為基準比對），並連跑 cargo test --workspace 三輪觀察 server 側是否出現 EINVAL 族 flaky——若出現，依 design Decisions 5 的備援（server 的 tests/it/common/ 加掛 poison-tolerant gate 或 --test-threads 降並發）另開 task 處理，不在本 task 內實作。驗證：三輪結果一致、無新 flaky（觀察輪已於 2026-08-03 執行：三輪一失——第三輪 admin_e2e，觸發備援 task 10.1；本 task 的驗證改以 10.1 完成後的三輪重跑為準）。 <!-- speclink-task:tsk_01KYY54KZWB4N20HHQCX7MSVM8 -->

## 10. server 重量級測試的 process gate（備援，由 9.2 觀察觸發）

- [x] 10.1 `crates/speclink-server/tests/it/common/mod.rs` 新增 poison-tolerant 的 PROCESS_GATE（照 desktop tests/it/common 的 HARNESS_GATE 樣式：static Mutex、取鎖用 PoisonError::into_inner 避免前一測試 panic 連坐），並在九個會 spawn 真實子程序的測試檔（admin_e2e、admin_three_entry、backup_e2e、cli_admin、e2e_cli、invite、phase2_chain、startup、serverfs_store）的每個測試函式開頭取 gate；serverfs_store 的 ignored 子程序 helper（serverfs_verb_flow_child）不取。行為：重量級子程序測試互斥執行，其餘 in-process 測試維持全並發，壓低 loopback 埠競爭、free_port TOCTOU 窗口與 healthz／setup-token 就緒時限（10s／5s）在高負載下被擠壓的風險。驗證：cargo test -p speclink-server 全綠並記錄 it 執行時間與 92–106 秒基準對照（若超過基準 3 倍，縮小 gate 射程至帶硬時限的 admin_e2e、e2e_cli、startup 三檔並重測）；連跑三輪 cargo test --workspace（完整輸出留檔）三輪一致全綠、無 FAILED。 <!-- speclink-task:tsk_01KZ315BWDPFM7W144DBEGADEH -->

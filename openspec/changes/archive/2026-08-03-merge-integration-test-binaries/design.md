## Context

實測診斷（openspec/discussions/slow-global-test-suite.md，五回合）定位全局測試一小時起跳的成因：整合測試 113 支 binary、每支 67-81MB debug 檔，macOS Gatekeeper 對每支新連結 binary 首次執行收約 60 秒掃描稅；napi build 與 cargo test 共用 target/ 互踩指紋，令每輪 test:all 都重連結、重繳稅。測試邏輯本身僅約 7 分鐘。Cargo 的測試目標自動發現規則是本設計的地基：tests/ 頂層每個 .rs 檔各成一支 test binary，tests/ 子目錄不被自動發現，但 tests/<dir>/main.rs 形態會以 <dir> 為名成為單一目標。

現況盤點（實作前已核實）：
- libtest 形態：speclink-server 47 檔（另有 harness = false 的 postgres_store）、speclink-cli 27 檔、speclink-remote 11 檔、apps/desktop/src-tauri 8 檔、speclink-store-fs 5 檔、speclink-core 5 檔。
- 共用模組：server 與 desktop 各有 tests/common/mod.rs（desktop 的含 HARNESS_GATE static Mutex）；server 的 postgres_store 另用 tests/pg/mod.rs；store-postgres 六檔共用 tests/support/mod.rs，其中 support 內建迷你 harness（run 函式吃測試名與函式指標清單）。
- 全部測試檔零筆 env 寫入與工作目錄切換（grep 核實），無程序級狀態污染，合併安全。
- 專案文件與腳本無「cargo test --test <檔名>」選跑引用（grep 核實），無需同步改文件。

## Goals / Non-Goals

**Goals**
- 整合測試 binary 113 → 11：六個 libtest crate 各收斂為一支 it 目標；store-postgres 六支客製 harness 收斂為一支；postgres_store 維持獨立。
- napi build 的中間產物與 workspace target/ 完全隔離，任何 napi 重建不再改變 cargo test 的指紋。
- 測試內容、斷言、skip 語意逐檔原樣保留；cargo test --workspace 行為等價。

**Non-Goals**（詳見 proposal）：不採 cargo-nextest；不動單檔 crate；不改測試邏輯（含 phase3_chain 等待瘦身）；不處理平行 change 的 doctest／斷言字樣問題；不處理機器層豁免設定。

## Decisions

1. **libtest 合併走 tests/it/main.rs 模式**。各檔 git mv 進 tests/it/ 後成為模組，main.rs 逐一以 mod 宣告；common（server、desktop）隨檔搬入 tests/it/common/，由 main.rs 宣告一次，各檔原有的 mod common 宣告刪除、引用改 crate::common 路徑。選 it 為目標名：短、慣例明確（integration tests）、選跑過濾寫作 cargo test -p speclink-server --test it admin_api:: 仍以原檔名為模組前綴。
   - 否決「保留頂層檔＋include 匯入」：include 拼接破壞模組邊界且 rustfmt／IDE 支援差。
   - 否決「每 crate 拆數支主題 binary」：稅基下降幅度打折，且主題分界主觀、日後檔案歸屬爭議不斷。
2. **speclink-server 的 postgres_store 維持獨立 binary**。它是 harness = false（需真 PostgreSQL、未配置時自報 skipped），與 libtest 的 it 目標混不進同一 harness；其 [[test]] 宣告與 tests/pg/ 模組原地不動。server 由 48 支收為 2 支。
3. **store-postgres 六支客製 harness 統一為一支 harness = false 的 it 目標**。六檔搬入 tests/it/ 成模組，各檔的 fn main 改為公開的套件執行函式（沿用既有 support 迷你 harness 的測試清單形態），新 main.rs 依序執行六個模組的清單並沿用 support 的 skip 回報；Cargo.toml 六個 [[test]] 區塊收斂為一個（name = it、harness = false）。實作第一步是先讀 support 的 run 函式，確認 skip 與失敗匯總的既有語意後照抄，不自創回報格式。
4. **napi build 隔離走 crates/speclink-node/.cargo/config.toml 的 build.target-dir**。設為相對路徑 target（即 crates/speclink-node/target/）：napi build 經 npm --prefix 於該目錄執行、就近讀取此 config；root 執行的 cargo test／cargo build 不受影響。
   - 否決「npm script 前綴環境變數」：跨平台不可攜（本 repo 有 Windows 開發機，cmd 不吃前綴環境變數語法），引入 cross-env 又多一個依賴。
   - 否決「調整 test:all 步驟順序」：污染仍在，只是換受害者；下輪 cargo test 照樣重連結。
   - 代價：napi 首次於隔離目錄全量編譯一次（一次性成本）、磁碟多一份中間產物；.gitignore 需確認涵蓋該目錄。
5. **並行語意變化的守備**。合併後跨檔測試首次於同 process 的 libtest thread pool 並行，loopback server 同時存活數上升。desktop 的 HARNESS_GATE 是 static Mutex，合併後單一實例覆蓋全部 harness 測試，序列化語意不變且更完整。server 側無 gate、有 EINVAL 前科（見討論 Deferred），驗收含連跑三輪整套觀察；若出現同族 flaky，備援為 server 的 tests/it/common/ 加掛同款 poison-tolerant gate，或以 --test-threads 降並發——備援不預先實作（YAGNI）。
   - **觀察結果（2026-08-03 實作階段）**：三輪觀察一失——第三輪 server it 的 admin_e2e（the_admin_manages_a_team_end_to_end_over_the_real_binaries）失敗，該測試單獨重跑 5/5 全過、補跑第四輪完整 workspace 亦全綠；失敗當輪輸出經過濾、錯誤內文未留存，無法確認是否 EINVAL 族。備援據此觸發（task 10.1），選 **poison-tolerant PROCESS_GATE**，射程限定九個 spawn 真實子程序的測試檔——三個嫌疑失敗模式（EINVAL 族埠競爭、free_port TOCTOU、healthz/setup-token 硬時限被高負載擠壓）共同根因都是重量級子程序測試並行，gate 一次全壓；其餘約 260 個 in-process 測試不取 gate、維持全並發。
   - 否決「--test-threads 降並發」：Cargo 無 per-target 的 test-threads 設定，改 test:all 腳本救不了裸跑 cargo test 的人，workspace 級 RUST_TEST_THREADS 則把輕測試也拖慢，殺傷面過大。
   - 誠實界線：gate 針對負載共因，無法回溯證明修到了該次失敗的真因；驗收三輪全綠語意是「未再現」而非「已根治」。

## Risks / Trade-offs

- **abort 級崩潰的爆炸半徑**：崩潰帶走同 binary 其餘測試，隔離粒度由 113 格降為 11 格。發生率極低（歷史無 segfault 紀錄），接受。
- **平行 session 檔案衝突**：113 檔搬遷與任何 in-flight change 的測試檔改動必然相撞。實作前確認無平行 session 正在動 tests/ 檔案，且 git mv 保留歷史；e2e_cli 現有的斷言失敗屬平行 change 半成品，不因搬遷而修改其內容。
- **選跑習慣改變**：cargo test --test <檔名> 失效，改為 --test it <模組>:: 過濾。無文件引用需改，屬開發者肌肉記憶層面，於 tasks 完成訊息與 commit body 中明示。

## Migration Plan

逐 crate 一個 commit（六個 libtest crate 由簡至繁：core → store-fs → remote → cli → desktop → server），store-postgres 與 napi 隔離各自獨立 commit；每個 commit 內該 crate 的 cargo test -p 全綠才進下一個。任何一步出包可獨立 revert，不影響其他 crate。

## Open Questions

（無——備援方案已列於 Decisions 5，觸發條件明確）

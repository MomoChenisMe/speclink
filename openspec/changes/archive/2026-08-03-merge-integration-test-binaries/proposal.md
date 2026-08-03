## Why

全局測試（npm run test:all）在 macOS 上一跑一小時起跳。實測診斷（openspec/discussions/slow-global-test-suite.md）證實測試邏輯本身僅約 7 分鐘，其餘全是 macOS Gatekeeper 對每支新連結測試 binary 首次執行的惡意軟體掃描（約 60 秒／支 × 112 支整合測試 binary），且 test:all 內 napi build 與 cargo test 共用 target/ 互踩快取指紋，使每輪都重連結、重繳整筆掃描稅。本機已用 Developer Tools 豁免消掉稅率，但稅基（binary 數 × 每輪重連結）仍在——CI、新開發機、Windows Defender 同類機制的環境都會全額繳稅。受影響的是所有在本 repo 跑全局測試的開發者與 AI 代理（apply／verify 階段的測試迴圈都吃這筆時間）。

## What Changes

- 七個多測試檔 crate 的整合測試由「tests/ 頂層一檔一 binary」重組為「一 crate 一進入點」，整合測試 binary 從 113 支降為 11 支。測試內容與斷言逐檔原樣搬移，不改任何測試邏輯：
  - libtest 形態（六個 crate）：speclink-server 47 檔、speclink-cli 27 檔、speclink-remote 11 檔、apps/desktop/src-tauri 8 檔、speclink-store-fs 5 檔、speclink-core 5 檔，各自搬入 tests/it/ 子目錄成模組、由 tests/it/main.rs 統一宣告。
  - 客製 harness 形態：speclink-store-postgres 六檔均為 harness = false（需真 PostgreSQL、未配置時回報 skipped），已共用 support 迷你 harness——統一為單一 harness = false 進入點，skipped 回報語意不變。
  - 例外：speclink-server 的 postgres_store（harness = false，同需真 PostgreSQL）維持獨立 binary，不與 libtest 的 it 合併。
- speclink-node 的 napi build 改用與 workspace 隔離的 build 目錄，杜絕它與 cargo test 互踩共用 target/ 的快取指紋（互踩會導致零改動也整批重連結）。
- server 的 tests/it/common/ 加掛 poison-tolerant PROCESS_GATE（design Decisions 5 的備援，由三輪觀察的 admin_e2e flaky 觸發）：九個 spawn 真實子程序的測試檔互斥執行，其餘 in-process 測試維持全並發。
- 開發者選跑單一測試檔的指令由「--test <檔名>」改為「--test it <模組路徑>」過濾（專案內文件與腳本經 grep 確認無舊形式「--test <檔名>」引用；實作時發現 docs/product-status 兩檔另有指向測試檔的 Markdown 路徑連結，由 scripts/remote-docs.test.mjs 連結檢查把關，已隨搬遷同步改為 tests/it/ 新路徑）。
- 相容性影響：speclink CLI 與各 app 的行為、輸出、artifacts 均不變——本 change 只動測試檔配置與 build 中間產物位置；test:all 指令本身不變。

## Non-Goals

- 不採 cargo-nextest 作為主解（不減 binary 數，掃描與 link 稅不變；日後可另案補充）。
- 不只合併四大戶——多檔 crate 一次做完（同型機械改動，分批徒增 churn）。
- 不改動測試邏輯與斷言：phase3_chain 的 75 秒級實時等待瘦身、speclink-remote doctest 編譯錯誤、e2e_cli 連線錯誤字樣斷言不一致（後兩者屬平行 in-flight change 的半成品）皆不在本 change 範圍。
- 不動單檔 crate（speclink-store-sqlite、speclink-host、speclink-fs）——已是一 crate 一 binary。
- 不處理 Developer Tools 豁免（機器層設定，已由使用者完成，無程式碼可載）。

## Capabilities

### New Capabilities

（無——測試基礎設施重組，不引入能力）

### Modified Capabilities

（無——無任何 spec 層級行為變更）

## Impact

- Affected specs:（無）
- Affected code:
  - New:
    - `crates/speclink-server/tests/it/main.rs`
    - `crates/speclink-cli/tests/it/main.rs`
    - `crates/speclink-remote/tests/it/main.rs`
    - `apps/desktop/src-tauri/tests/it/main.rs`
    - `crates/speclink-store-postgres/tests/it/main.rs`（harness = false 客製進入點）
    - `crates/speclink-store-fs/tests/it/main.rs`
    - `crates/speclink-core/tests/it/main.rs`
    - `crates/speclink-node/.cargo/config.toml`
  - Modified:
    - `crates/speclink-server/tests/`（47 個 libtest 檔與 common 模組搬遷至 `crates/speclink-server/tests/it/`，各檔移除重複的 common 模組宣告；postgres_store 與其 pg 模組原地不動）
    - `crates/speclink-cli/tests/`（27 檔搬遷至 `crates/speclink-cli/tests/it/`）
    - `crates/speclink-remote/tests/`（11 檔搬遷至 `crates/speclink-remote/tests/it/`）
    - `apps/desktop/src-tauri/tests/`（8 檔與 common 模組搬遷至 `apps/desktop/src-tauri/tests/it/`）
    - `crates/speclink-store-postgres/tests/`（6 檔與 support 模組搬遷至 `crates/speclink-store-postgres/tests/it/`，六個客製 main 收斂為單一進入點）
    - `crates/speclink-store-postgres/Cargo.toml`（六個顯式 test 目標宣告收斂為一個 harness = false 的 it 目標）
    - `crates/speclink-store-fs/tests/`（5 檔搬遷至 `crates/speclink-store-fs/tests/it/`）
    - `crates/speclink-core/tests/`（5 檔搬遷至 `crates/speclink-core/tests/it/`）
    - `docs/product-status.md`、`docs/product-status.zh-TW.md`（測試檔 Markdown 連結改指 tests/it/ 新路徑）
  - Removed:（無——檔案為搬遷非刪除）

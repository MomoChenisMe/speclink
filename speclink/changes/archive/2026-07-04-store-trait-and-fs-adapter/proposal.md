## Why

speclink 的引擎邏輯（speclink-core）目前直接呼叫 std::fs 存取 `openspec/` 目錄——佈局知識散落在 `crates/speclink-core/src/paths.rs` 與各模組的檔案操作中，引擎與「文件存放方式」緊耦合。依十六輪討論（sdd-engine-as-sdk-with-pluggable-document-storage-for-team-scenarios）的結論，speclink 要讓團隊系統自決文件的存放與呈現方式，第一步（也是後續設定體系、動詞契約、Node SDK 三個 change 的共同地基）就是把「引擎邏輯」與「文件儲存」之間的縫線切出來。

目標使用者：透過 AI 代理跑 SDD 的開發者（現況情境不變），以及後續要以 SDK 串接自家儲存後端的團隊系統開發者（本 change 只鋪地基，不交付 SDK）。

## What Changes

- 在 speclink-core 新增領域層級的 `Store` 介面：以 change／artifact／discussion／spec／workflow-config 為語彙（同步、非 async），引擎所有規格文件存取一律經由此介面，core 不再直接對 spec 目錄做檔案操作。
- 現行 `openspec/` 目錄佈局的全部知識（路徑組裝、目錄列舉、mtime 排序推導、archive 搬移機制、discussion 檔案骨架）搬入 `crates/speclink-fs`（現為空殼 crate），成為 `Store` 的預設實作，並加入 workspace members。
- 引擎宿主端的工作資料（`.speclink/` 的 touched 與 snapshots）與 workspace 檔案（CLAUDE.md、技能目錄）不屬於 Store——其路徑解析留在引擎宿主側，與規格文件儲存分離。
- 對外行為完全不變：所有 CLI 指令的人眼輸出、`--json` payload、exit code、檔案系統效果與現況位元級一致；`.speclink.yaml`、`openspec/config.yaml` 的讀取行為不變。
- 新增中英雙語文件骨架：架構說明（引擎—Store—呈現三層、縫線位置）與入門教學（純本地情境），README 增加 Documentation 章節引用，後續三個 change 各自增補對應章節。

## Non-Goals

（範圍排除與被否決方案記錄於 design.md 的 Goals / Non-Goals 章節。）

## Capabilities

### New Capabilities

- `store-abstraction`: 引擎經由 Store 介面存取規格文件；預設 fs 實作保持現行 openspec/ 佈局，所有既有 CLI 可觀察行為不變。

### Modified Capabilities

（無——本 change 不改變任何既有需求層級行為；openspec/specs/ 目前亦無既有規格。）

## Impact

- Affected specs: 新增 `store-abstraction`
- Affected crates: speclink-core（介面抽取與去 fs 化）、speclink-fs（新 adapter，加入 workspace）、speclink-cli（組裝點改為注入 fs adapter）
- 相容性影響: 人眼輸出與 `--json` 均無變更；回歸對照（parity_suite 31 項、color_suite 16 項、twin harness 8 情境）必須維持通過——本 change 以此為驗收條件之一
- Affected code:
  - New: `crates/speclink-core/src/store.rs`、`crates/speclink-fs/src/lib.rs`、`crates/speclink-fs/src/layout.rs`、`docs/architecture.md`、`docs/architecture.zh-TW.md`、`docs/getting-started.md`、`docs/getting-started.zh-TW.md`
  - Modified: `Cargo.toml`（workspace members 加入 speclink-fs）、`crates/speclink-fs/Cargo.toml`、`crates/speclink-core/src/lib.rs`、`crates/speclink-core/src/model.rs`、`crates/speclink-core/src/discuss.rs`、`crates/speclink-core/src/archive.rs`、`crates/speclink-core/src/status.rs`、`crates/speclink-core/src/validate.rs`、`crates/speclink-core/src/analyzer.rs`、`crates/speclink-core/src/drift.rs`、`crates/speclink-core/src/inprogress.rs`、`crates/speclink-core/src/tasks.rs`、`crates/speclink-core/src/newcmd.rs`、`crates/speclink-core/src/init.rs`、`crates/speclink-core/src/config.rs`、`crates/speclink-core/src/preflight.rs`、`crates/speclink-cli/src/commands.rs`、`crates/speclink-cli/src/main.rs`、`README.md`
  - Removed: `crates/speclink-core/src/paths.rs`（規格目錄佈局邏輯遷入 speclink-fs；宿主側工作資料路徑解析移至 core 內新歸屬，詳見 design）

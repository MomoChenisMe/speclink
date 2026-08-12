# store-abstraction Specification

## Purpose

規格儲存介面抽象化的相容性語意：把直接檔案存取換成儲存介面之後，既有指令的行為與 openspec/ 的預設檔案系統佈局皆保持不變，工作區建置預設即帶檔案系統實作。本 capability 保證這層抽象化是純內部重構——對使用者端零可觀察差異。

## Requirements

### Requirement: 儲存重構後既有指令行為保持不變

引擎改經儲存介面存取規格文件後，CLI 的所有既有指令 SHALL 維持與重構前完全一致的可觀察行為：人眼輸出（含色彩與 --no-color）、`--json` payload 的欄位（camelCase）與值、exit code、以及對檔案系統的效果。本需求為輸出凍結敏感：重構前的既有輸出基線 SHALL 維持位元級不變（驗證載體為 crates/speclink-cli/tests/ 的整合測試與 speclink-core 的 render_golden 測試）。

#### Scenario: 既有專案的清單查詢輸出一致

- **WHEN** 於既有 fs 專案根目錄執行 speclink list --json
- **THEN** 輸出 JSON 的欄位與值與重構前基線一致，exit code 為 0

#### Scenario: 無專案目錄時的錯誤輸出一致

- **WHEN** 於不含任何 speclink 專案標記的目錄執行 speclink list
- **THEN** stderr 的錯誤訊息文字與 exit code 與重構前基線一致

#### Scenario: 人眼輸出與 --no-color 一致

- **WHEN** 於既有專案分別執行 speclink status --change 某 change 與加上 --no-color 的同一指令
- **THEN** 兩種輸出均與重構前基線一致（含 ANSI 色彩序列與去色版本）


<!-- @trace
source: spectra-legacy-cleanup
updated: 2026-07-27
code:
  - README.en.md
  - README.md
  - apps/desktop/src/App.tsx
  - apps/desktop/src/components/ProjectTabs.tsx
  - apps/desktop/src/index.css
  - crates/speclink-cli/src/color.rs
  - crates/speclink-cli/src/commands.rs
  - crates/speclink-cli/src/main.rs
  - crates/speclink-cli/tests/discuss_promote_snapshot.rs
  - crates/speclink-cli/tests/task_done_stamps.rs
  - crates/speclink-core/assets/skills/archive.md
  - crates/speclink-core/src/analyzer.rs
  - crates/speclink-core/src/archive.rs
  - crates/speclink-core/src/command/mod.rs
  - crates/speclink-core/src/config.rs
  - crates/speclink-core/src/demo.rs
  - crates/speclink-core/src/discuss.rs
  - crates/speclink-core/src/drift.rs
  - crates/speclink-core/src/init.rs
  - crates/speclink-core/src/instructions.rs
  - crates/speclink-core/src/lib.rs
  - crates/speclink-core/src/listing.rs
  - crates/speclink-core/src/model.rs
  - crates/speclink-core/src/newcmd.rs
  - crates/speclink-core/src/preflight.rs
  - crates/speclink-core/src/schema.rs
  - crates/speclink-core/src/skills.rs
  - crates/speclink-core/src/status.rs
  - crates/speclink-core/src/tasks.rs
  - crates/speclink-core/src/validate.rs
  - crates/speclink-core/tests/golden/claude.snapshot.md
  - crates/speclink-core/tests/golden/codex.snapshot.md
  - crates/speclink-core/tests/golden/neutral-cli.snapshot.md
  - crates/speclink-core/tests/golden/neutral-tool-call.snapshot.md
  - crates/speclink-host/src/context.rs
  - docs/platform-architecture.zh-TW.md
  - packages/ui/src/__tests__/delta.test.ts
  - packages/ui/src/__tests__/taskList.test.tsx
  - packages/ui/src/components/ChangeList.tsx
  - packages/ui/src/components/DeltaBadges.tsx
  - packages/ui/src/components/RichDetailDrawer.tsx
  - packages/ui/src/delta.ts
  - packages/ui/src/index.ts
  - packages/ui/src/theme.css
-->

---
### Requirement: 預設檔案系統佈局不變
預設儲存實作 SHALL 維持現行 openspec/ 目錄佈局：specs/<capability>/spec.md、changes/<name>/、changes/archive/、discussions/、config.yaml。所有寫入類指令產生的檔案路徑、檔名與內容格式 SHALL 與重構前一致。

#### Scenario: 建立 change 的檔案系統效果一致
- **WHEN** 執行 speclink new change demo-change --agent claude
- **THEN** 於 openspec/changes/demo-change/ 建立 .openspec.yaml，其欄位（schema、created、created_by）與重構前格式一致，exit code 為 0

#### Scenario: 損壞 metadata 的容錯行為一致
- **WHEN** 某 change 目錄內的 .openspec.yaml 內容無法解析，執行 speclink list --json
- **THEN** 該 change 仍以預設 metadata 列出，指令不中斷、不輸出錯誤，與重構前行為一致

#### Scenario: 變更排序行為一致
- **WHEN** 存在多個 change 且其檔案修改時間不同，執行 speclink list
- **THEN** 排序依「change 內最近修改時間（截整秒）由新至舊」，與重構前一致


<!-- @trace
source: store-trait-and-fs-adapter
updated: 2026-07-04
code:
  - AGENTS.md
  - CLAUDE.md
  - Cargo.lock
  - Cargo.toml
  - README.md
  - crates/speclink-cli/Cargo.toml
  - crates/speclink-cli/src/commands.rs
  - crates/speclink-cli/src/main.rs
  - crates/speclink-core/assets/skills/discuss.md
  - crates/speclink-core/src/analyzer.rs
  - crates/speclink-core/src/archive.rs
  - crates/speclink-core/src/config.rs
  - crates/speclink-core/src/demo.rs
  - crates/speclink-core/src/discuss.rs
  - crates/speclink-core/src/drift.rs
  - crates/speclink-core/src/init.rs
  - crates/speclink-core/src/inprogress.rs
  - crates/speclink-core/src/instructions.rs
  - crates/speclink-core/src/lib.rs
  - crates/speclink-core/src/model.rs
  - crates/speclink-core/src/newcmd.rs
  - crates/speclink-core/src/paths.rs
  - crates/speclink-core/src/preflight.rs
  - crates/speclink-core/src/schema.rs
  - crates/speclink-core/src/status.rs
  - crates/speclink-core/src/store.rs
  - crates/speclink-core/src/tasks.rs
  - crates/speclink-core/src/util.rs
  - crates/speclink-core/src/validate.rs
  - crates/speclink-core/src/workspace.rs
  - crates/speclink-core/tests/no_direct_fs.rs
  - crates/speclink-fs/Cargo.toml
  - crates/speclink-fs/src/layout.rs
  - crates/speclink-fs/src/lib.rs
  - crates/speclink-fs/tests/store_fs.rs
  - docs/architecture.md
  - docs/architecture.zh-TW.md
  - docs/getting-started.md
  - docs/getting-started.zh-TW.md
-->

---
### Requirement: 工作區建置包含預設儲存實作
專案 workspace SHALL 包含預設檔案系統儲存實作套件，完整建置與測試 SHALL 成功。

#### Scenario: 全 workspace 建置與測試成功
- **WHEN** 依序執行 cargo test 與 cargo build --release
- **THEN** 兩者均以 exit code 0 結束，且產出的 speclink 執行檔可回應 speclink --version

<!-- @trace
source: store-trait-and-fs-adapter
updated: 2026-07-04
code:
  - AGENTS.md
  - CLAUDE.md
  - Cargo.lock
  - Cargo.toml
  - README.md
  - crates/speclink-cli/Cargo.toml
  - crates/speclink-cli/src/commands.rs
  - crates/speclink-cli/src/main.rs
  - crates/speclink-core/assets/skills/discuss.md
  - crates/speclink-core/src/analyzer.rs
  - crates/speclink-core/src/archive.rs
  - crates/speclink-core/src/config.rs
  - crates/speclink-core/src/demo.rs
  - crates/speclink-core/src/discuss.rs
  - crates/speclink-core/src/drift.rs
  - crates/speclink-core/src/init.rs
  - crates/speclink-core/src/inprogress.rs
  - crates/speclink-core/src/instructions.rs
  - crates/speclink-core/src/lib.rs
  - crates/speclink-core/src/model.rs
  - crates/speclink-core/src/newcmd.rs
  - crates/speclink-core/src/paths.rs
  - crates/speclink-core/src/preflight.rs
  - crates/speclink-core/src/schema.rs
  - crates/speclink-core/src/status.rs
  - crates/speclink-core/src/store.rs
  - crates/speclink-core/src/tasks.rs
  - crates/speclink-core/src/util.rs
  - crates/speclink-core/src/validate.rs
  - crates/speclink-core/src/workspace.rs
  - crates/speclink-core/tests/no_direct_fs.rs
  - crates/speclink-fs/Cargo.toml
  - crates/speclink-fs/src/layout.rs
  - crates/speclink-fs/src/lib.rs
  - crates/speclink-fs/tests/store_fs.rs
  - docs/architecture.md
  - docs/architecture.zh-TW.md
  - docs/getting-started.md
  - docs/getting-started.zh-TW.md
-->
# store-abstraction Specification

## Purpose

規格儲存介面抽象化的相容性語意：把直接檔案存取換成儲存介面之後，既有指令的行為與 speclink/ 的預設檔案系統佈局皆保持不變，工作區建置預設即帶檔案系統實作。本 capability 保證這層抽象化是純內部重構——對使用者端零可觀察差異。

## Requirements

### Requirement: 儲存重構後既有指令行為保持不變

引擎改經儲存介面存取規格文件後，CLI 的所有既有指令 SHALL 維持與重構前完全一致的可觀察行為：人眼輸出（含色彩與 --no-color）、`--json` payload 的欄位（camelCase）與值、exit code、以及對檔案系統的效果。本需求為輸出凍結敏感：重構前的既有輸出基線 SHALL 維持位元級不變（驗證載體為 crates/adapters/speclink-cli/tests/ 的整合測試與 speclink-core 的 render_golden 測試）。

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

---
### Requirement: 預設檔案系統佈局
預設儲存實作 SHALL 使用規格資料夾（預設 `speclink/`，決定方式見 spec-dir-migration）下的佈局：specs/<capability>/spec.md、changes/<name>/、changes/archive/、discussions/、config.yaml。每個變更的資料檔 SHALL 為 changes/<name>/.meta.yaml（已封存時為 changes/archive/<dated-name>/.meta.yaml），欄位與格式維持既有規則。讀取變更資料檔時 SHALL 先讀 `.meta.yaml`，不存在時才讀舊檔名 `.openspec.yaml`；兩者並存時以 `.meta.yaml` 為準。寫入 SHALL 只寫 `.meta.yaml`，該變更資料夾有 `.openspec.yaml` 時 SHALL 在同一次寫入中刪除它。封存移動變更資料夾時，資料檔隨資料夾移動、檔名不變。

#### Scenario: 建立 change 的檔案系統效果一致
- **WHEN** 在使用 `speclink/` 的專案執行 speclink new change demo-change --agent claude
- **THEN** 於 speclink/changes/demo-change/ 建立 .meta.yaml，其欄位（schema、created、created_by）與既有格式一致，沒有建立 .openspec.yaml，exit code 為 0

#### Scenario: 損壞 metadata 的容錯行為一致
- **WHEN** 某 change 目錄內的 .meta.yaml 內容無法解析，執行 speclink list --json
- **THEN** 該 change 仍以預設 metadata 列出，指令不中斷、不輸出錯誤，與既有行為一致

#### Scenario: 變更排序行為一致
- **WHEN** 存在多個 change 且其檔案修改時間不同，執行 speclink list
- **THEN** 排序依「change 內最近修改時間（截整秒）由新至舊」，與既有行為一致

#### Scenario: 只有舊檔名時照常讀取
- **WHEN** speclink/changes/old/ 只有 .openspec.yaml（含 `depends_on: base`），執行 speclink plan --json
- **THEN** old 的 dependsOn 含 base，exit code 為 0

#### Scenario: 寫入時移除舊檔名
- **WHEN** speclink/changes/old/ 只有 .openspec.yaml，執行 speclink change depends old --on base2
- **THEN** speclink/changes/old/ 只有 .meta.yaml，內容為原 .openspec.yaml 加上新的 depends_on 值，.openspec.yaml 已不存在

#### Scenario: 新舊檔名並存時以新檔名為準
- **WHEN** speclink/changes/both/ 同時有 .meta.yaml（schema: no-spec-change）與 .openspec.yaml（schema: spec-driven），執行 speclink status --change both --json
- **THEN** schemaName 為 no-spec-change

<!-- @trace
source: spec-dir-migration
updated: 2026-10-23T23:15:21+08:00
-->

---
### Requirement: 遠端封存變更資料檔的鍵

遠端 store 的已封存變更文件以相對檔名為鍵。系統讀取已封存變更的資料檔時 SHALL 先讀鍵 `.meta.yaml`，不存在時才讀舊鍵 `.openspec.yaml`；新封存的變更 SHALL 以 `.meta.yaml` 為鍵寫入。既有遠端資料 SHALL NOT 被搬移或改鍵。

#### Scenario: 舊鍵照常讀取
- **WHEN** 遠端 store 有一個以 `.openspec.yaml` 為鍵的已封存變更，client 讀取已封存清單
- **THEN** 該項的建立者與來源討論等欄位照常帶出，與舊鍵改名前相同

#### Scenario: 新封存寫新鍵
- **WHEN** 在遠端模式封存變更 demo
- **THEN** 遠端 store 中該已封存變更的資料檔鍵為 `.meta.yaml`，沒有 `.openspec.yaml` 鍵

<!-- @trace
source: spec-dir-migration
updated: 2026-10-23T23:15:21+08:00
-->
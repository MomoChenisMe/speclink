# store-abstraction Specification

## Purpose

規格儲存介面抽象化的相容性語意：把直接檔案存取換成儲存介面之後，既有指令的行為與 openspec/ 的預設檔案系統佈局皆保持不變，工作區建置預設即帶檔案系統實作。本 capability 保證這層抽象化是純內部重構——對使用者端零可觀察差異。

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

---
### Requirement: 檢查紀錄的缺席與讀取失敗可區分

規格儲存的完成檢查文件讀取 SHALL 區分文件存在、真正不存在與讀取失敗。索引存在但無法讀取、無效UTF-8、拒絕連結、超過大小限制與IO失敗 SHALL NOT 被轉成不存在或空歷史；單筆文件的相同錯誤 SHALL 由歷史查閱回應明示該筆讀取失敗，且不阻擋其他完整紀錄。只有實際不存在的索引且無既存歷史證據時 SHALL 沿用舊變更無紀錄的空清單語意，不掃描Git或目錄自動重建索引；若來源能明確判斷索引遭刪除 SHALL 顯示來源錯誤，不編造清單。

本機及Remote SHALL 使用同一索引與內容完整性判定；Remote權限、認證或快照取得失敗 SHALL 為來源錯誤，不宣稱單筆不存在。讀取 SHALL 只限所屬活動或完整dated封存名稱下允許的checks相對路徑，拒絕路徑越界、檔案與父目錄符號連結及未提交的封存／未復原交易。既有每份文件64 MiB及索引100000筆的限制 SHALL 不被取消。此窄介面 SHALL NOT 修改磁碟佈局、一般文件格式或原寫入／驗收的完整性守門；歷史查閱 SHALL 不建立、刪除或修改任何歷史文件、索引與章。

#### Scenario: 真正缺檔與無權限分別回報

- **WHEN** 索引可讀且分別有一筆文件不存在、一筆文件因IO或權限問題不可讀，以及一筆完整文件
- **THEN** 歷史清單分別以missing_check_record、check_record_read_error及無readError表示，完整單筆可讀，不將前兩項當作歷史從未存在，沒有檔案寫入

#### Scenario: 索引讀取失敗不變成空歷史

- **WHEN** 已存在索引因權限、無效編碼、格式錯誤或連結而讀取失敗
- **THEN** 呼叫端收到整體來源或索引錯誤，不回成功空清單、不建立替代索引，也不自動變更既有文件

#### Scenario: 舊變更從未保存索引

- **WHEN** 活動或已封存變更從未保存完成紀錄且索引真正不存在
- **THEN** 查詢回空清單，既有檔案系統佈局保持不變，不掃描Git、建立索引或製造紀錄

#### Scenario: 越界與連結不被當作可信文件

- **WHEN** 索引路徑越界、所列文件或父目錄為符號連結，或資料超過既有大小／筆數限制
- **THEN** 非法索引或來源界線錯誤整體拒絕；合法索引下單筆文件不可讀則明示該筆失敗，不讀變更外內容，不放寬限制，其他完整紀錄仍依合法索引查閱

#### Scenario: 封存尚未提交或交易待復原

- **WHEN** 封存資料仍處於未提交狀態，或活動檢查交易journal尚待復原
- **THEN** 歷史查閱不顯示半套紀錄，回明確來源錯誤或history_recovery_required，且查閱本身不刪除復原資料

#### Scenario: Remote來源失敗與讀取一致

- **WHEN** 同一份索引在Local或已取得授權的Remote快照中含一筆缺失文件，或Remote請求在快照取得前遭401／403／離線
- **THEN** 前者逐筆分類一致且完整文件可讀，後者整體失敗而不列文件缺失，Remote不變更revision

<!-- @trace
source: archived-quality-history-access
updated: 2026-10-23T22:41:06+08:00
-->
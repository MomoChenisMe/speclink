# change-lifecycle Specification

## Purpose

TBD - created by archiving change 'desktop-board-parity'. Update Purpose after archive.

## Requirements

### Requirement: in-progress 標記真相存於 change meta

in-progress 標記 SHALL 以 change 目錄的 .openspec.yaml 為唯一真相：執行 speclink in-progress add 某 change 後，該檔 SHALL 含 started_at（ISO 日期）；started_by 與 started_with SHALL 依 created_by／created_with 的同一身分機制寫入——呼叫端可歸屬者寫入、不可歸屬者缺席（CLI 以 git 身分供 started_by；CLI 現無 agent 識別來源，started_with 缺席，寫入縫留在引擎函式的 agent 參數，desktop／remote 通道屆時供給）。既有欄位（schema、created_*、from_discussion 等）SHALL 原樣保留。指令 SHALL NOT 建立 .git/speclink-app/ 目錄或其下任何檔案。重複執行 SHALL 冪等——已有 started_* 時欄位值不變。本需求為 parity 敏感：指令的 stdout、stderr 與 exit code SHALL 與遷移前的行為位元級一致（首次與重複執行皆然）；對不存在 change 的行為 SHALL 維持遷移前實測基線——靜默成功（無輸出、exit 0），且 SHALL NOT 寫入任何檔案。

#### Scenario: 標記後 meta 含三站中的開工欄位

- **WHEN** 對含 created_* 欄位的 change 執行 speclink in-progress add 該 change
- **THEN** 該 change 的 .openspec.yaml 新增 started_at 與 started_by（git 身分可得時），created_* 與 schema 欄位逐字元保留，且 .git/speclink-app/ 未被建立；經引擎函式帶 agent 識別呼叫時另含 started_with

#### Scenario: 重複標記冪等

- **WHEN** 對已含 started_* 的 change 再次執行 speclink in-progress add
- **THEN** 三欄位值與首次標記後完全相同（保留首次開工蓋章），stdout 與 exit code 與首次執行一致

#### Scenario: 不存在的 change 行為不變

- **WHEN** 對不存在的 change 名執行 speclink in-progress add
- **THEN** 與遷移前版本一致地靜默成功——無輸出、exit 0（遷移前實測基線：名稱不驗證），且無任何檔案被寫入（無 meta 變動、無 .git/speclink-app/）


<!-- @trace
source: desktop-board-parity
updated: 2026-07-06
code:
  - .spectra.yaml
  - AGENTS.md
  - CLAUDE.md
  - Cargo.lock
  - apps/desktop/core/src/cache.rs
  - apps/desktop/core/src/lib.rs
  - apps/desktop/core/src/manage.rs
  - apps/desktop/core/src/query.rs
  - apps/desktop/core/src/testfixture.rs
  - apps/desktop/core/src/verbs.rs
  - apps/desktop/src-tauri/Cargo.toml
  - apps/desktop/src-tauri/src/lib.rs
  - apps/desktop/src-tauri/src/watch.rs
  - apps/desktop/src/App.tsx
  - apps/desktop/src/__tests__/App.test.tsx
  - apps/desktop/src/__tests__/store.test.ts
  - apps/desktop/src/adapter/tauriDataSource.ts
  - crates/speclink-cli/src/commands.rs
  - crates/speclink-core/Cargo.toml
  - crates/speclink-core/src/archive.rs
  - crates/speclink-core/src/inprogress.rs
  - crates/speclink-core/src/lib.rs
  - crates/speclink-core/src/listing.rs
  - crates/speclink-core/src/model.rs
  - crates/speclink-core/src/store.rs
  - crates/speclink-core/src/teststore.rs
  - crates/speclink-core/tests/no_direct_fs.rs
  - crates/speclink-fs/src/lib.rs
  - crates/speclink-fs/tests/store_fs.rs
  - crates/speclink-node/src/store_bridge.rs
  - package-lock.json
  - packages/ui/package.json
  - packages/ui/src/__tests__/archivedList.test.tsx
  - packages/ui/src/__tests__/kanban.test.tsx
  - packages/ui/src/__tests__/richDrawer.test.tsx
  - packages/ui/src/__tests__/stage.test.ts
  - packages/ui/src/__tests__/taskList.test.tsx
  - packages/ui/src/adapter.ts
  - packages/ui/src/components/ArchivedList.tsx
  - packages/ui/src/components/RichDetailDrawer.tsx
  - packages/ui/src/components/TaskList.tsx
  - packages/ui/src/stage.ts
  - packages/ui/src/tasks.ts
-->

---
### Requirement: 歸檔保留完整生命週期歸屬

speclink archive 一個已標記開工的 change 後，封存目錄的 .openspec.yaml SHALL 同時含 created／created_by（建立站）、started_at／started_by／started_with（開工站）與 archived_at／archived_by（歸檔站）——started_* 欄位 SHALL NOT 於歸檔時被剝除或改寫。

#### Scenario: 歸檔後三站欄位並存

- **WHEN** 對 meta 含 created_* 與 started_* 的 change 執行 speclink archive
- **THEN** changes/archive/ 下該 change 的 .openspec.yaml 同時含三站全部欄位，started_* 的值與歸檔前逐字元一致


<!-- @trace
source: desktop-board-parity
updated: 2026-07-06
code:
  - .spectra.yaml
  - AGENTS.md
  - CLAUDE.md
  - Cargo.lock
  - apps/desktop/core/src/cache.rs
  - apps/desktop/core/src/lib.rs
  - apps/desktop/core/src/manage.rs
  - apps/desktop/core/src/query.rs
  - apps/desktop/core/src/testfixture.rs
  - apps/desktop/core/src/verbs.rs
  - apps/desktop/src-tauri/Cargo.toml
  - apps/desktop/src-tauri/src/lib.rs
  - apps/desktop/src-tauri/src/watch.rs
  - apps/desktop/src/App.tsx
  - apps/desktop/src/__tests__/App.test.tsx
  - apps/desktop/src/__tests__/store.test.ts
  - apps/desktop/src/adapter/tauriDataSource.ts
  - crates/speclink-cli/src/commands.rs
  - crates/speclink-core/Cargo.toml
  - crates/speclink-core/src/archive.rs
  - crates/speclink-core/src/inprogress.rs
  - crates/speclink-core/src/lib.rs
  - crates/speclink-core/src/listing.rs
  - crates/speclink-core/src/model.rs
  - crates/speclink-core/src/store.rs
  - crates/speclink-core/src/teststore.rs
  - crates/speclink-core/tests/no_direct_fs.rs
  - crates/speclink-fs/src/lib.rs
  - crates/speclink-fs/tests/store_fs.rs
  - crates/speclink-node/src/store_bridge.rs
  - package-lock.json
  - packages/ui/package.json
  - packages/ui/src/__tests__/archivedList.test.tsx
  - packages/ui/src/__tests__/kanban.test.tsx
  - packages/ui/src/__tests__/richDrawer.test.tsx
  - packages/ui/src/__tests__/stage.test.ts
  - packages/ui/src/__tests__/taskList.test.tsx
  - packages/ui/src/adapter.ts
  - packages/ui/src/components/ArchivedList.tsx
  - packages/ui/src/components/RichDetailDrawer.tsx
  - packages/ui/src/components/TaskList.tsx
  - packages/ui/src/stage.ts
  - packages/ui/src/tasks.ts
-->

---
### Requirement: meta 新欄位向後相容

change meta 的解析 SHALL 對缺少 started_* 欄位的既有檔案維持既有行為：所有讀取 meta 的指令與查詢 SHALL 正常運作、該 change 視為未開工，SHALL NOT 產生任何警告或錯誤。

#### Scenario: 舊 meta 檔正常解析且視為未開工

- **WHEN** 對 meta 僅含 schema 與 created_* 欄位（無 started_*）的 change 執行 speclink list --json 與 speclink status --change 該 change
- **THEN** 兩指令輸出與遷移前版本位元級一致，exit code 為 0，無警告

<!-- @trace
source: desktop-board-parity
updated: 2026-07-06
code:
  - .spectra.yaml
  - AGENTS.md
  - CLAUDE.md
  - Cargo.lock
  - apps/desktop/core/src/cache.rs
  - apps/desktop/core/src/lib.rs
  - apps/desktop/core/src/manage.rs
  - apps/desktop/core/src/query.rs
  - apps/desktop/core/src/testfixture.rs
  - apps/desktop/core/src/verbs.rs
  - apps/desktop/src-tauri/Cargo.toml
  - apps/desktop/src-tauri/src/lib.rs
  - apps/desktop/src-tauri/src/watch.rs
  - apps/desktop/src/App.tsx
  - apps/desktop/src/__tests__/App.test.tsx
  - apps/desktop/src/__tests__/store.test.ts
  - apps/desktop/src/adapter/tauriDataSource.ts
  - crates/speclink-cli/src/commands.rs
  - crates/speclink-core/Cargo.toml
  - crates/speclink-core/src/archive.rs
  - crates/speclink-core/src/inprogress.rs
  - crates/speclink-core/src/lib.rs
  - crates/speclink-core/src/listing.rs
  - crates/speclink-core/src/model.rs
  - crates/speclink-core/src/store.rs
  - crates/speclink-core/src/teststore.rs
  - crates/speclink-core/tests/no_direct_fs.rs
  - crates/speclink-fs/src/lib.rs
  - crates/speclink-fs/tests/store_fs.rs
  - crates/speclink-node/src/store_bridge.rs
  - package-lock.json
  - packages/ui/package.json
  - packages/ui/src/__tests__/archivedList.test.tsx
  - packages/ui/src/__tests__/kanban.test.tsx
  - packages/ui/src/__tests__/richDrawer.test.tsx
  - packages/ui/src/__tests__/stage.test.ts
  - packages/ui/src/__tests__/taskList.test.tsx
  - packages/ui/src/adapter.ts
  - packages/ui/src/components/ArchivedList.tsx
  - packages/ui/src/components/RichDetailDrawer.tsx
  - packages/ui/src/components/TaskList.tsx
  - packages/ui/src/stage.ts
  - packages/ui/src/tasks.ts
-->

---
### Requirement: 任務完成蘊含開工標記

speclink task done 成功完成一項任務時，若該 change 的 .openspec.yaml 尚無 started_* 欄位，SHALL 於同一操作內蓋開工章：started_at 為當日 ISO 日期；started_by 依 git 身分可得性寫入——可歸屬者寫入、不可歸屬者缺席；本指令無 agent 識別來源，started_with 缺席。meta 既有欄位 SHALL 逐字元保留。該 change 已有 started_* 時 SHALL 保留首章不變。touched-files 記錄（.speclink/ 下）行為 SHALL 維持現行語意。

本需求為 parity 敏感：指令的人眼輸出、--json payload（change、status、taskDesc、taskId 對應之既有欄位形狀）與 exit code SHALL 與現行位元級一致（對 Spectra 2.3.1 的輸出 parity 不變）；.openspec.yaml 的開工章為刻意檔案效果分歧——自我基線的檔案樹對照 SHALL 隨本需求更新並記載此差異。錯誤路徑（tasks.md 缺失、任務序號無效、任務已完成）SHALL 維持現行訊息與非零 exit code，且 SHALL NOT 寫入任何檔案。

#### Scenario: 首次完成任務蓋開工章

- **WHEN** 對 meta 含 created_* 而無 started_* 的 change 執行 speclink task done 完成一項未完成任務
- **THEN** tasks.md 該任務標記為 [x]，stdout 與 exit code 與現行一致，.openspec.yaml 新增 started_at（git 身分可得時另含 started_by），schema 與 created_* 欄位逐字元保留

##### Example: meta 前後對照

- **GIVEN** .openspec.yaml 內容為 schema、created、created_by、created_with 四欄，tasks.md 為 0/5
- **WHEN** speclink task done 1 --change demo 成功
- **THEN** .openspec.yaml 於既有四欄之後新增 started_at: <當日> 與 started_by: <git 身分>，無其他變動；tasks.md 成 1/5

#### Scenario: 已開工的 change 完成後續任務不改章

- **WHEN** 對已含 started_* 的 change 執行 speclink task done 完成另一項任務
- **THEN** started_at、started_by、started_with 值與執行前完全相同（首章保留），tasks.md 正常勾章

#### Scenario: 任務已完成時無任何檔案效果

- **WHEN** 對已標記 [x] 的任務再執行 speclink task done
- **THEN** 指令以現行「already done」錯誤訊息與非零 exit code 結束，tasks.md、.openspec.yaml 與 touched 記錄皆無變動

#### Scenario: tasks.md 缺失時不蓋章

- **WHEN** 對無 tasks.md 的 change 名執行 speclink task done
- **THEN** 指令以現行「tasks.md not found」錯誤結束，該 change 的 .openspec.yaml（若存在）無任何變動

<!-- @trace
source: task-done-implies-started
updated: 2026-07-07
code:
  - Cargo.lock
  - apps/desktop/core/src/manage.rs
  - crates/speclink-cli/Cargo.toml
  - crates/speclink-cli/src/commands.rs
  - crates/speclink-cli/tests/task_done_stamps.rs
  - crates/speclink-core/src/tasks.rs
  - crates/speclink-core/src/teststore.rs
  - crates/speclink-core/tests/no_direct_fs.rs
  - packages/ui/src/__tests__/kanban.test.tsx
  - packages/ui/src/__tests__/stage.test.ts
  - packages/ui/src/stage.ts
-->
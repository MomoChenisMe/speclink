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

本需求為輸出凍結敏感：指令的人眼輸出、--json payload（change、status、taskDesc、taskId 對應之既有欄位形狀）與 exit code SHALL 與現行位元級一致（既有輸出基線不變）；.openspec.yaml 的開工章為刻意檔案效果變更——自我基線的檔案樹對照 SHALL 隨本需求更新並記載此差異。錯誤路徑（tasks.md 缺失、任務序號無效、任務已完成）SHALL 維持現行訊息與非零 exit code，且 SHALL NOT 寫入任何檔案。

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
### Requirement: 變更以 discard 動詞廢棄

speclink discard SHALL 接受一個位置參數（變更名）與 --force、--json 旗標，廢棄一個尚未動工的變更：刪除 openspec/changes/<change>/ 目錄整棵，並刪除該變更的 touched 紀錄檔（若存在）。變更不存在時 SHALL 以非零 exit code 結束並於 stderr 說明。變更有動工痕跡——meta 含 started_at，或 tasks.md 有任何已勾任務——且未帶 --force 時 SHALL 拒絕：非零 exit code、stderr 提示動工痕跡與 --force，且 SHALL NOT 改動任何檔案；帶 --force 時 SHALL 照常執行。成功時 exit code 0，stdout 報告已刪除的變更名與每份解鏈討論的 slug 及回退後狀態（--no-color 下無 ANSI 色彩）；帶 --json 時 SHALL 輸出 camelCase payload：變更名與解鏈討論清單（各含 slug 與回退後狀態）。remote store 模式下 SHALL 以非零 exit code 於 stderr 報 discard 不支援。變更目錄刪除失敗時 SHALL 以非零 exit code 回報，已完成的討論解鏈不回滾且輸出 SHALL 明示已解鏈清單。本指令為 Speclink 自有延伸；既有指令的人眼與 --json 輸出 SHALL 逐位元不變。

#### Scenario: 未動工變更成功廢棄

- **WHEN** 對 meta 無 started_at 且 tasks.md 無已勾任務的變更執行 speclink discard
- **THEN** exit code 0；openspec/changes/<change>/ 目錄消失；stdout 報告已刪除的變更名與解鏈結果

#### Scenario: 動工痕跡守衛拒絕

- **WHEN** 對有動工痕跡的變更執行 speclink discard 且未帶 --force
- **THEN** 非零 exit code；stderr 提示動工痕跡與 --force；openspec/ 下任何檔案逐位元不變

##### Example: 動工痕跡判定

| meta 有 started_at | tasks.md 有已勾任務 | 未帶 --force 的結果 |
| ------------------ | ------------------- | ------------------- |
| 否                 | 否                  | 放行                |
| 是                 | 否                  | 拒絕                |
| 否                 | 是                  | 拒絕                |
| 是                 | 是                  | 拒絕                |

#### Scenario: --force 放行動過工的變更

- **WHEN** 對有動工痕跡的變更執行 speclink discard --force
- **THEN** exit code 0；變更目錄與其 touched 紀錄檔皆刪除；stdout 報告同成功路徑

#### Scenario: 變更不存在報錯

- **WHEN** 執行 speclink discard 給定不存在的變更名
- **THEN** 非零 exit code；stderr 說明變更不存在；無任何檔案變動

#### Scenario: remote store 模式不支援

- **WHEN** 於 remote store 綁定的專案執行 speclink discard
- **THEN** 非零 exit code；stderr 報 discard 不支援於 remote 模式；無任何檔案變動

#### Scenario: --json 輸出 payload

- **WHEN** 執行 speclink discard <change> --json 成功廢棄
- **THEN** stdout 為 JSON：含變更名欄位與解鏈討論陣列（每項含 slug 與回退後狀態），欄位名一律 camelCase


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
### Requirement: restale_from 記錄變更待重新反映的討論並經 CLI 觀測

變更 meta 檔（openspec/changes/<name>/.openspec.yaml）MAY 帶 restale_from 欄位——逗號分隔的討論 slug 清單，語意為「本變更曾反映（seal）這些討論，其後這些討論被重新結論，內容相對新結論過期、待 re-ingest」。ChangeMeta SHALL 提供 restale_from() accessor 回傳 Vec<String>：欄位缺席時回空、逗號值 SHALL 各段 trim 後分割，行為平行既有 from_discussion／from_discussions()。此欄位由 discuss conclude 寫入、discuss seal 清除（見 discussion-docs 正典），本需求規範其讀取與觀測。speclink show <change> --json SHALL 於變更 payload 恆曝 restaleFrom（camelCase 字串陣列，無旗標為空陣列），平行既有 fromDiscussions。speclink list --json SHALL 於 restale_from 非空的變更 payload 曝 restaleFrom 陣列、為空時省略該欄位——以維持 list --json 對無旗標變更的既有輸出逐位元不變。speclink analyze <change> 於某變更 restale_from 非空時 SHALL 出一條資訊性 finding，指明該變更反映的討論已重新結論、需重新 ingest 以同步新結論。此欄位讀取 SHALL 為零 per-load 掃描——僅讀既存 meta 欄位，不掃描討論記錄。

#### Scenario: restale_from() accessor 讀取

- **WHEN** 變更 meta 含 restale_from: alpha-search, beta-cache
- **THEN** ChangeMeta::restale_from() 回傳 ["alpha-search", "beta-cache"]；meta 無該欄位時回傳空 Vec

#### Scenario: show 恆曝 restaleFrom

- **WHEN** 對 restale_from 含 alpha-search 的變更、以及無該欄位的變更，各執行 speclink show <change> --json
- **THEN** 前者 payload 的 restaleFrom 為 ["alpha-search"]；後者 payload 的 restaleFrom 為空陣列（欄位恆存在）

#### Scenario: list 曝 restaleFrom 且對無旗標變更輸出不變

- **WHEN** 對含一個 restale_from 非空變更與一個無該欄位變更的專案執行 speclink list --json
- **THEN** 非空變更的 payload 含 restaleFrom 陣列（如 ["alpha-search"]）；無旗標變更的 payload 省略 restaleFrom 欄位，其 list --json 輸出與本變更前逐位元一致

#### Scenario: analyze 對過期變更出資訊性 finding

- **WHEN** 對 restale_from 非空的變更執行 speclink analyze <change>
- **THEN** 輸出含一條資訊性 finding，指明該變更反映的討論已重新結論、需 re-ingest；restale_from 為空時無此 finding，且 analyze 輸出與本變更前逐位元一致


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
### Requirement: 壞 metadata 使生命週期寫入 fail closed

change 的 `.openspec.yaml` 存在但 YAML 解析失敗時，讀寫該 change 生命週期狀態的動詞——in-progress add、claim、task done、task undone、new artifact、archive 與 discard——SHALL 以帶檔案位置與解析原因的錯誤拒絕，且 SHALL NOT 寫入、移動或刪除任何檔案。壞 metadata SHALL NOT 被解讀為未開工、預設 schema 或無來源討論。「檔案不存在」與「欄位缺席」SHALL 維持既有預設行為（「meta 新欄位向後相容」需求不變，其約束對象是缺欄位而非壞檔）。

#### Scenario: in-progress add 對壞 metadata 拒絕且不疊寫

- **WHEN** 對 `.openspec.yaml` 為壞 YAML 的 change 執行 speclink in-progress add 該 change
- **THEN** 以非零 exit code 結束；該 `.openspec.yaml` 逐位元不變（未被文字手術追加或代換 started_* 行）

#### Scenario: task done 因蘊含開工標記而拒絕

- **WHEN** 對壞 metadata 的 change 執行 task done 勾選任一任務
- **THEN** 以非零 exit code 結束；tasks.md 與 `.openspec.yaml` 皆逐位元不變

#### Scenario: discard 不得把壞 metadata 當未開工

- **WHEN** 對壞 metadata 的 change 執行 speclink discard 該 change（未帶 --force）
- **THEN** 以非零 exit code 結束且 change 目錄完整保留；stderr 指出 metadata 損壞，而非以「未開工」放行刪除

#### Scenario: discard 帶 --force 仍拒絕

- **WHEN** 對壞 metadata 的 change 執行 speclink discard 該 change --force
- **THEN** 以非零 exit code 結束且 change 目錄完整保留（使用者修復 metadata 後方可廢棄）

#### Scenario: archive 對壞 metadata 拒絕

- **WHEN** 對壞 metadata 的 change 執行 speclink archive 該 change
- **THEN** 以非零 exit code 結束；正典規格未被併入、change 目錄未被移動

---
### Requirement: in-progress 標記經 remote 通道寫入 server meta

remote 模式下 speclink in-progress add SHALL 路由至 server：started_at 與 started_by SHALL 以 server 端認證身分蓋進該 change 的 meta 文件（與 created_* 同一身分機制——可歸屬者寫入、不可歸屬者缺席）；started_with 維持缺席（CLI 現無 agent 識別來源，fs 與 remote 一致）。CLI 的 stdout、stderr 與 exit code SHALL 維持 parity 凍結形狀：首次蓋章、重複執行、change 不存在三種情形皆靜默成功（無輸出、exit 0）。change 不存在或已有 started_* 時 server SHALL NOT 寫入任何文件、SHALL NOT 發布事件、scope revision SHALL NOT 前進；實際蓋章時 SHALL 發布對應領域事件。變更清單摘要 SHALL 攜帶選填 startedAt 欄位（camelCase，None 缺席、缺席時反序列化為預設），值來自 change meta 的 started_at，供消費端做欄位推導。

#### Scenario: remote 蓋章帶認證歸屬

- **WHEN** 於 remote 模式以認證使用者 momo 對 server 上未開工的 change 執行 speclink in-progress add
- **THEN** CLI 靜默結束（無輸出、exit 0），server 端該 change 的 meta 含 started_at（ISO 日期）與 started_by（momo 的身分），既有欄位逐字元保留

#### Scenario: 不存在的 change 靜默成功且零寫入

- **WHEN** 於 remote 模式對 server 上不存在的 change 名稱執行 speclink in-progress add
- **THEN** CLI 靜默結束（無輸出、exit 0），server 端零文件寫入、零事件發布、scope revision 不前進

#### Scenario: startedAt 隨清單上 wire

- **WHEN** 已蓋開工章的 change 出現於 server 的變更清單回應（GET /changes）
- **THEN** 該清單項含 startedAt 欄位（camelCase）且值等於 meta 的 started_at；未開工的 change 清單項不含該欄位。CLI 的 speclink list --json 維持與 fs 模式同形（fs 清單項凍結不帶 started_*——verb-contract 的列舉形狀 parity 優先），欄位推導由桌面等消費端在 wire payload 上進行

##### Example: 清單項的 startedAt

| change meta | GET /changes 清單項 |
| ----------- | ------------------- |
| `started_at: 2026-07-30` | `{"name":"demo","completedTasks":0,"totalTasks":15,"startedAt":"2026-07-30"}` |
| （無 started_at） | `{"name":"demo","completedTasks":0,"totalTasks":15}`（無 startedAt 鍵） |

<!-- @trace
source: remote-cli-parity
updated: 2026-07-31
code:
  - apps/desktop/src-tauri/src/remote.rs
  - apps/desktop/src-tauri/tests/remote_data.rs
  - apps/desktop/src/__tests__/remoteDataSource.test.ts
  - crates/speclink-cli/src/commands.rs
  - crates/speclink-cli/src/remote_commands.rs
  - crates/speclink-cli/tests/discuss_slug.rs
  - crates/speclink-cli/tests/remote_verb_parity.rs
  - crates/speclink-cli/tests/remote_write_path.rs
  - crates/speclink-protocol/src/command.rs
  - crates/speclink-protocol/src/query.rs
  - crates/speclink-remote/src/client.rs
  - crates/speclink-remote/tests/typed_client.rs
  - crates/speclink-server/src/app.rs
  - crates/speclink-server/src/routes.rs
  - crates/speclink-server/tests/backup_e2e.rs
  - crates/speclink-server/tests/discussion_routes.rs
  - crates/speclink-server/tests/e2e_cli.rs
  - crates/speclink-server/tests/query_routes.rs
  - crates/speclink-server/tests/verb_api.rs
  - docs/verb-contract.md
  - docs/verb-contract.zh-TW.md
-->

---
### Requirement: in-progress 標記可自 change meta 移除(零工作痕跡守門)

執行 speclink in-progress remove 某 change 後,系統 SHALL 僅在該 change 為零工作痕跡——tasks.md 的已勾任務數為 0(tasks.md 不存在視為 0)且 touched 記錄的 v1 與 v2 兩清單皆空(記錄檔不存在視為空)——時,自該 change 的 .openspec.yaml 移除 started_at、started_by、started_with 三欄位;其餘欄位與內容 SHALL 逐字保留,SHALL NOT 重新序列化整份文件。子指令不接旗標、不讀 stdin。成功移除 SHALL exit 0 並於 stdout 印出移除確認;對未開工(無任何 started_* 欄位)的 change SHALL 冪等成功——exit 0、零檔案寫入。已勾任務數 > 0 或 touched 記錄非空時 SHALL 拒絕:exit 非 0,stderr SHALL 列出已勾任務數與 touched 記錄的檔案清單(兩清單聯集、去重)及出路說明(已勾任務可取消後重試;touched 需以人工或 agent 判斷處理),且 SHALL NOT 修改任何檔案。指名不存在的 change SHALL exit 非 0 並於 stderr 報找不到——此行為與 in-progress add 對未知名稱的靜默成功刻意不對稱(add 受遷移前 parity 凍結,remove 為新動詞、修正動作打錯名字必須明確報錯)。change meta 損毀無法解析時 SHALL fail-closed 報錯且不動任何檔案。本指令 SHALL NOT 提供任何強制旗標或機械清理已勾任務/touched 記錄的路徑,SHALL NOT 影響 speclink in-progress add 的既有輸出與行為。

#### Scenario: 零痕跡的進行中變更成功退回

- **WHEN** 一個 change 曾執行 in-progress add(meta 含 started_at 與 started_by),tasks.md 無任何已勾任務且無 touched 記錄,對其執行 speclink in-progress remove
- **THEN** exit 0,stdout 印出移除確認;meta 的 started_at、started_by、started_with 消失,schema、created_*、from_discussion 等其餘欄位逐字不變

#### Scenario: 已勾任務時拒絕退回

- **WHEN** 一個進行中的 change 其 tasks.md 有 2 個已勾任務,對其執行 speclink in-progress remove
- **THEN** exit 非 0,stderr 說明有 2 個已勾任務並提示取消勾選後可重試;meta 與 tasks.md 皆不變

#### Scenario: touched 記錄非空時拒絕退回並列出檔案

- **WHEN** 一個進行中的 change 已勾任務數為 0,但 touched 記錄含檔案,對其執行 speclink in-progress remove
- **THEN** exit 非 0,stderr 列出 touched 記錄的檔案清單並說明需以人工或 agent 判斷處理;meta 與 touched 記錄皆不變

##### Example: 證據清單為兩版記錄的聯集去重

- **GIVEN** touched 記錄 v1 清單含 src/a.rs 與 src/b.ts,v2 清單含 src/b.ts 與 src/c.rs
- **WHEN** 對該 change 執行 speclink in-progress remove
- **THEN** stderr 的檔案清單恰為 src/a.rs、src/b.ts、src/c.rs 三項,無重複

#### Scenario: 未開工的變更冪等成功

- **WHEN** 對一個 meta 無任何 started_* 欄位的 change 執行 speclink in-progress remove
- **THEN** exit 0,不寫入任何檔案

#### Scenario: 不存在的變更明確報錯

- **WHEN** 對不存在的 change 名稱執行 speclink in-progress remove
- **THEN** exit 非 0,stderr 報找不到該 change

#### Scenario: meta 損毀時 fail-closed

- **WHEN** 一個 change 的 .openspec.yaml 無法解析,對其執行 speclink in-progress remove
- **THEN** exit 非 0 報錯,不修改任何檔案

<!-- @trace
source: revert-in-progress-to-proposed
updated: 2026-07-31
code:
  - apps/desktop/core/src/manage.rs
  - apps/desktop/src-tauri/src/lib.rs
  - apps/desktop/src-tauri/src/remote.rs
  - apps/desktop/src-tauri/tests/remote_data.rs
  - apps/desktop/src-tauri/tests/remote_runtime.rs
  - apps/desktop/src/App.tsx
  - apps/desktop/src/__tests__/App.test.tsx
  - apps/desktop/src/__tests__/remoteDataSource.test.ts
  - apps/desktop/src/__tests__/tauriDataSource.test.ts
  - apps/desktop/src/adapter/remoteDataSource.ts
  - apps/desktop/src/adapter/tauriDataSource.ts
  - apps/desktop/src/i18n/messages.ts
  - apps/desktop/src/store.ts
  - crates/speclink-cli/src/commands.rs
  - crates/speclink-cli/src/main.rs
  - crates/speclink-cli/src/remote_commands.rs
  - crates/speclink-cli/tests/in_progress_remove.rs
  - crates/speclink-cli/tests/remote_write_path.rs
  - crates/speclink-core/assets/skills/apply.md
  - crates/speclink-core/src/command/mod.rs
  - crates/speclink-core/src/inprogress.rs
  - crates/speclink-core/tests/golden/claude.snapshot.md
  - crates/speclink-core/tests/golden/codex.snapshot.md
  - crates/speclink-core/tests/golden/neutral-cli.snapshot.md
  - crates/speclink-core/tests/golden/neutral-tool-call.snapshot.md
  - crates/speclink-host/src/bridge.rs
  - crates/speclink-host/src/commit.rs
  - crates/speclink-protocol/src/command.rs
  - crates/speclink-protocol/src/error.rs
  - crates/speclink-remote/src/client.rs
  - crates/speclink-remote/src/device.rs
  - crates/speclink-remote/src/events.rs
  - crates/speclink-remote/src/lib.rs
  - crates/speclink-remote/tests/reauth_retry.rs
  - crates/speclink-remote/tests/typed_client.rs
  - crates/speclink-server/src/app.rs
  - crates/speclink-server/src/error.rs
  - crates/speclink-server/src/events.rs
  - crates/speclink-server/src/routes.rs
  - crates/speclink-server/tests/verb_api.rs
  - packages/ui/src/__tests__/discussionColumn.test.tsx
  - packages/ui/src/__tests__/discussionDrawer.test.tsx
  - packages/ui/src/__tests__/kanban.test.tsx
  - packages/ui/src/__tests__/revertBlockedDialog.test.tsx
  - packages/ui/src/__tests__/richDrawer.test.tsx
  - packages/ui/src/adapter.ts
  - packages/ui/src/components/ChangeCard.tsx
  - packages/ui/src/components/DiscussionColumn.tsx
  - packages/ui/src/components/DiscussionDrawer.tsx
  - packages/ui/src/components/KanbanBoard.tsx
  - packages/ui/src/components/RevertBlockedDialog.tsx
  - packages/ui/src/components/RichDetailDrawer.tsx
  - packages/ui/src/i18n.tsx
  - packages/ui/src/index.ts
-->

---
### Requirement: 單筆封存的任務完成度守門

speclink archive <change>(單筆路徑)SHALL 於封存前檢查 tasks.md 的任務完成度:任務總數大於零且完成數小於總數、且未帶 --mark-tasks-complete 時 SHALL 拒絕——非零 exit code,stderr 列證據(完成數/總數)與兩條出路(完成任務後再封存、或帶 --mark-tasks-complete),且 SHALL NOT 改動任何檔案(change 目錄、正典 specs、快照與 touched 紀錄逐位元不變)。帶 --mark-tasks-complete 時 SHALL 維持既有語意:先將 tasks.md 全部勾選再封存。任務全數完成、或任務總數為零的 change,單筆封存的人眼與 --json 輸出 SHALL 與守門引入前逐位元一致。此守門 SHALL 於引擎封存流程本體生效,一體適用 CLI 單筆、桌面 app 封存動詞與 server 封存通道——桌面對任務未完成 change 觸發封存時 SHALL 收到引擎拒絕訊息(依既有失敗 toast 語意呈現),SHALL NOT 將該 change 標為已封存。批次封存(--all 或多變更名)的預過濾與跳過回報行為 SHALL 維持不變。本守門屬刻意行為變更:單筆封存對任務未完成 change 由成功改為拒絕。

#### Scenario: 任務未完成的單筆封存被拒

- **WHEN** 對 tasks.md 有 3 個任務、僅 1 個已勾的 change 執行 speclink archive <change>
- **THEN** 非零 exit code;stderr 載明完成數與總數(1/3)並提示完成任務或 --mark-tasks-complete;openspec/ 下任何檔案逐位元不變,changes/archive/ 無新目錄

#### Scenario: --mark-tasks-complete 放行並先全勾

- **WHEN** 對同一 change 執行 speclink archive <change> --mark-tasks-complete
- **THEN** exit code 0;封存後的 tasks.md 全部任務為已勾;change 移入 changes/archive/,stdout 報告與既有成功路徑一致

#### Scenario: 任務全完成的單筆封存逐位元不變

- **WHEN** 對任務全數完成的 change 執行 speclink archive <change>(人眼與 --json 各一次)
- **THEN** 兩種輸出與 exit code 皆與守門引入前完全一致,封存效果(specs 套用、快照、meta 蓋章)不變

#### Scenario: 桌面封存動詞收到引擎拒絕

- **WHEN** 桌面 app 對任務未完成的 change 觸發封存並確認
- **THEN** 封存不發生,app 依既有失敗 toast 語意呈現引擎拒絕訊息,該 change 仍在看板

##### Example: 守門判定

| 任務總數 | 完成數 | 帶 --mark-tasks-complete | 結果 |
| -------- | ------ | ------------------------ | ---- |
| 3        | 1      | 否                       | 拒絕 |
| 3        | 1      | 是                       | 先全勾再封存 |
| 3        | 3      | 否                       | 照常封存 |
| 0        | 0      | 否                       | 照常封存 |

<!-- @trace
source: archive-readiness-gating
updated: 2026-07-31
code:
  - apps/desktop/core/src/manage.rs
  - apps/desktop/src-tauri/tests/remote_data.rs
  - apps/desktop/src/__tests__/App.test.tsx
  - apps/desktop/src/__tests__/helpers/remoteFixtures.ts
  - apps/desktop/src/__tests__/remoteDataSource.test.ts
  - apps/desktop/src/__tests__/remoteResilience.test.tsx
  - apps/desktop/src/adapter/remoteDataSource.ts
  - crates/speclink-cli/tests/archive_readiness_gate.rs
  - crates/speclink-core/src/archive.rs
  - crates/speclink-core/src/command/mod.rs
  - packages/ui/src/__tests__/kanban.test.tsx
  - packages/ui/src/__tests__/richDrawer.test.tsx
  - packages/ui/src/boardDnd.ts
  - packages/ui/src/components/KanbanBoard.tsx
  - packages/ui/src/components/RichDetailDrawer.tsx
  - packages/ui/src/i18n.tsx
-->

---
### Requirement: 封存的 linked worktree 環境守門

封存動詞（單筆與 bulk）SHALL 於任何檔案效果之前判定執行環境：workspace root 的 .git 為檔案（linked worktree 特徵）且 git 回報的當前分支具 speclink/ 前綴時 SHALL 拒絕封存——非零 exit code，stderr 說明封存不得於 linked worktree 內執行、並指路先以 worktree-merge 合回主分支再封存；change 目錄、正典規格與解封存備份目錄 SHALL 維持零變動，且 --mark-tasks-complete 的前置全勾寫入 SHALL NOT 發生（tasks.md 逐位元不變）。.git 為目錄（主 checkout）時本守門 SHALL NOT spawn git 且封存行為不變——即使當前分支名恰具 speclink/ 前綴亦然（fs 短路先於分支判定）。git 不可用、指令失敗或分支輸出為空（detached HEAD）時 SHALL 放行（fail-open，沿 worktree discovery 的既有慣例：無 git 的環境不得因此無法封存）；分支無 speclink/ 前綴時 SHALL 放行。

#### Scenario: worktree 內封存被拒且零檔案效果

- **WHEN** 於分支 speclink/some-change 的 linked worktree 內對任一 change 執行封存
- **THEN** exit code 非零；stderr 含 worktree 事實與 worktree-merge 指路；該 change 目錄仍在原位，無正典規格寫入亦無備份目錄產生

#### Scenario: 拒絕時 --mark-tasks-complete 前置寫入零效果

- **WHEN** 於 speclink/ 分支的 linked worktree 內，對含未勾任務的 change 帶 --mark-tasks-complete 執行封存
- **THEN** exit code 非零，且該 change 的 tasks.md 逐位元不變（前置全勾寫入未發生）

#### Scenario: bulk 封存同受守門

- **WHEN** 於 speclink/ 分支的 linked worktree 內執行 bulk 封存（--all 或多個 change 名）
- **THEN** exit code 非零；輸出含 worktree 事實與 worktree-merge 指路（bulk 的中止報告走 stdout，stderr 為 bulk 失敗摘要）；所有 change 目錄原地不動

#### Scenario: 非 speclink 分支的 worktree 放行

- **WHEN** workspace root 的 .git 為檔案、當前分支為 feature/anything，執行封存
- **THEN** 封存行為與主 checkout 完全相同

#### Scenario: 主 checkout 零額外開銷

- **WHEN** workspace root 的 .git 為目錄，執行封存——含分支名恰為 speclink/demo 的情形
- **THEN** 本守門不 spawn git，封存行為與導入前完全相同

#### Scenario: git 不可用時 fail-open

- **WHEN** workspace root 的 .git 為檔案但 git 不可用，執行封存
- **THEN** 封存照常執行


<!-- @trace
source: archive-guard-test-hardening
updated: 2026-08-06
-->

---
### Requirement: 封存的章失效守門

單筆封存 SHALL 於任務完成度守門之後、任何封存檔案效果之前,對 change 的 review 章與 verify 章各執行一次失效判定(依各站「指紋錨與失效判定」條文):章欄位齊備且判為 stale 時 SHALL 拒絕封存——非零 exit code,stderr 點名過期的站別與破錨原因(內容錨列出首個不符的檔案;任務錨述明計數),並指路重跑該站技能後再封存;兩章皆 stale 時 SHALL 並列點名。無章、或章欄位不全(Unknown)的 change SHALL 放行,其封存行為與本守門引入前逐位元一致。任務未完成與章失效並存時,任務完成度守門 SHALL 先拒且其訊息不變。`--mark-tasks-complete` 路徑 SHALL 於前置全勾寫入之前判定章失效——stale 拒絕時 tasks.md 逐位元不變,未手測的 `[M]` 任務不得被代勾。任一站工單開立中時,該站之章 SHALL 不入失效判定——該站的封存處置(擋下或 `--carry-*` 帶走)由未結工單守門承載,已被重開工單取代的章不得攔路。本守門 SHALL 於引擎封存流程本體生效,一體適用 CLI 單筆、桌面封存動詞與 server 封存通道;批次封存經同一流程,stale 章的拒絕沿未結工單守門的既有 fail-fast 樣式中止批次並點名該 change,SHALL NOT 靜默跳過。remote 封存通道無工作樹可讀,SHALL 僅判任務錨、跳過內容錨——此非對稱屬已知限制。

#### Scenario: 蓋章後改碼的封存被拒

- **WHEN** review 章蓋成後修改任一 `reviewed_scope` 檔案內容,任務全數完成後執行單筆封存
- **THEN** 非零 exit code,stderr 點名 review 章已過期並列出首個內容不符的檔案,指路重跑審查站;openspec/ 下任何檔案逐位元不變

#### Scenario: 補勾手動任務後封存放行

- **WHEN** 寫碼任務全完成時兩章蓋成,之後勾選最後一個 `[M]` 任務且 scope 檔零改動,執行單筆封存
- **THEN** exit code 0,封存照常完成——補勾 `[M]` 任務不使章失效

#### Scenario: 無章與 Unknown 章放行

- **WHEN** 對無任何章、或章欄位不全的任務全完成 change 執行單筆封存
- **THEN** 封存行為(人眼與 --json 輸出、exit code、檔案效果)與本守門引入前逐位元一致

#### Scenario: 任務守門先於章失效守門

- **WHEN** change 同時有未完成的寫碼任務與已失效的 review 章,執行單筆封存(未帶 --mark-tasks-complete)
- **THEN** 拒絕訊息為既有任務完成度守門訊息(完成數/總數與兩條出路),不提及章失效

#### Scenario: 帶旗標封存的拒絕路徑零寫入

- **WHEN** 章已失效的 change 以 `--mark-tasks-complete` 執行單筆封存
- **THEN** 拒絕來自章失效守門且 tasks.md 逐位元不變——未勾的 `[M]` 任務不被代勾

#### Scenario: 工單開立中的站不入失效判定

- **WHEN** 兩站蓋章後重開兩張工單,scope 檔已改動,帶 `--carry-review --carry-verify` 執行單筆封存
- **THEN** 封存放行,工單隨目錄帶走——已被重開工單取代的章不擋 carry 處置

#### Scenario: remote 通道僅判任務錨

- **WHEN** 經 server 封存通道對「蓋章後改過 scope 檔、任務錨未破」的 change 觸發封存
- **THEN** 封存放行——remote 側無工作樹,內容錨不判定;任務錨破(如蓋章後新增任務)時仍拒絕

##### Example: 守門判定

| 章狀態 | 蓋章後變動 | 結果 |
| ------ | ---------- | ---- |
| review 章齊備 | scope 檔內容改變 | 拒絕,點名 review 站 |
| 兩章齊備 | 補勾 [M] 任務 | 放行 |
| 兩章齊備 | 新增一個任務 | 拒絕,任務錨破 |
| 無章 | 任意 | 放行(行為不變) |
| 章欄位不全 | 任意 | 放行(行為不變) |

<!-- @trace
source: manual-task-marker-gates
updated: 2026-08-11
-->
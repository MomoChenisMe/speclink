# verb-contract Specification

## Purpose

TBD - created by archiving change 'verb-contract-and-remote-client'. Update Purpose after archive.

## Requirements

### Requirement: 動詞契約的涵蓋面與 payload 形狀
動詞契約 SHALL 涵蓋 RD 本地全流程：changes 列舉／讀取／建立／認領／歸檔、artifacts 讀取／寫入、tasks 勾選／取消勾選、instructions 計算、discussions 全動詞（列舉、建立、context、add-round、conclude、archive、promote）、政策讀取（workflow-config 有效值）、詞彙讀取（LANGUAGE 內容）、正典規格讀取、身分查驗。remote 模式下對應 CLI 指令的 stdout 輸出（人眼與 --json）形狀 SHALL 與 fs 模式一致（欄位 camelCase 同名）。契約的端點、payload 與錯誤形狀 SHALL 以 docs/verb-contract.md 為正典參考文件。

#### Scenario: remote 列舉輸出形狀與 fs 一致
- **WHEN** 於 remote 模式執行 speclink list --json，server 回傳兩個 change
- **THEN** stdout 的 JSON 欄位名與 fs 模式的 speclink list --json 完全一致，exit code 為 0

#### Scenario: instructions 由 server 計算
- **WHEN** 於 remote 模式執行 speclink instructions proposal --change 某 change --json
- **THEN** payload 含 context、rules、template、locale 等欄位（值來自 server 端政策），欄位名與 fs 模式一致

---
### Requirement: 樂觀並行控制與 409 語意
artifact 寫入 SHALL 攜帶讀取時取得的版本（If-Match）；版本過期時 server 回 409 且 body SHALL 含機器可判的 reason 欄位。CLI 對每個 409 reason SHALL 輸出對應的建議動作訊息（version_conflict → 重新拉取後再寫；ownership_lost → 重新認領；change_busy → 等待進行中的變更完成；repo_mismatch → 於歸屬 repo 執行）。

#### Scenario: 版本衝突的可讀訊息
- **WHEN** 於 remote 模式寫入 artifact，而該 artifact 已被他人更新（server 回 409、reason 為 version_conflict）
- **THEN** exit code 非 0，stderr 單行訊息說明內容已被更新並建議重新拉取，不顯示裸狀態碼

#### Scenario: 認領被搶佔
- **WHEN** 執行 speclink claim 某 change，而 server 回 409、reason 為 ownership_lost（已被他人認領）
- **THEN** exit code 非 0，stderr 訊息含目前持有人資訊與建議動作


<!-- @trace
source: verb-contract-and-remote-client
updated: 2026-07-05
code:
  - .speclink.yaml
  - Cargo.lock
  - Cargo.toml
  - README.md
  - crates/speclink-cli/Cargo.toml
  - crates/speclink-cli/src/commands.rs
  - crates/speclink-cli/src/main.rs
  - crates/speclink-cli/src/remote_commands.rs
  - crates/speclink-cli/tests/doc_verbs.rs
  - crates/speclink-cli/tests/remote_connect.rs
  - crates/speclink-cli/tests/remote_read_path.rs
  - crates/speclink-cli/tests/remote_write_path.rs
  - crates/speclink-core/Cargo.toml
  - crates/speclink-core/assets/skills/archive.md
  - crates/speclink-core/assets/skills/commit.md
  - crates/speclink-core/assets/skills/discuss.md
  - crates/speclink-core/assets/skills/propose.md
  - crates/speclink-core/assets/skills/sync.md
  - crates/speclink-core/src/discuss.rs
  - crates/speclink-core/src/init.rs
  - crates/speclink-core/src/instructions.rs
  - crates/speclink-core/src/preflight.rs
  - crates/speclink-core/src/status.rs
  - crates/speclink-core/src/store.rs
  - crates/speclink-core/src/workspace.rs
  - crates/speclink-core/tests/golden/claude.snapshot.md
  - crates/speclink-core/tests/golden/codex.snapshot.md
  - crates/speclink-core/tests/golden/neutral-cli.snapshot.md
  - crates/speclink-core/tests/golden/neutral-tool-call.snapshot.md
  - crates/speclink-core/tests/golden/remote-claude.marker.md
  - crates/speclink-core/tests/mode_resolution.rs
  - crates/speclink-core/tests/render_golden.rs
  - crates/speclink-core/tests/skill_verbization.rs
  - crates/speclink-fs/src/layout.rs
  - crates/speclink-fs/src/lib.rs
  - crates/speclink-remote/Cargo.toml
  - crates/speclink-remote/src/auth.rs
  - crates/speclink-remote/src/client.rs
  - crates/speclink-remote/src/lib.rs
  - crates/speclink-remote/tests/auth_store.rs
  - crates/speclink-remote/tests/client_errors.rs
  - docs/team-mode.md
  - docs/team-mode.zh-TW.md
  - docs/verb-contract.md
  - docs/verb-contract.zh-TW.md
-->

---
### Requirement: API 版本協商與錯誤翻譯紅線
每個請求 SHALL 攜帶 API 版本 header；server 不支援該版本時 CLI SHALL 輸出明確的版本不符訊息與升級指引。任何非 2xx 回應 SHALL 翻譯為單行語義化訊息＋建議動作（401 → 執行 speclink auth login；404 → 指出不存在的資源名；連線失敗或 5xx → server 不可用與檢查連接 url 的提示）；CLI SHALL NOT 將裸 HTTP 狀態碼作為錯誤輸出的主體，SHALL NOT 於連線失敗時改用本地資料回應（不做快取 fallback）。

#### Scenario: 未登入的動詞提示登入
- **WHEN** 未登入（無憑證檔且無 SPECLINK_TOKEN）於 remote 模式執行 speclink list
- **THEN** exit code 非 0，stderr 單行訊息提示執行 speclink auth login

#### Scenario: 連線失敗明確失敗
- **WHEN** 連接 url 無法連線時執行 speclink list
- **THEN** exit code 非 0，stderr 訊息指出 server 不可用與檢查連接設定的建議；stdout 無任何資料輸出


<!-- @trace
source: verb-contract-and-remote-client
updated: 2026-07-05
code:
  - .speclink.yaml
  - Cargo.lock
  - Cargo.toml
  - README.md
  - crates/speclink-cli/Cargo.toml
  - crates/speclink-cli/src/commands.rs
  - crates/speclink-cli/src/main.rs
  - crates/speclink-cli/src/remote_commands.rs
  - crates/speclink-cli/tests/doc_verbs.rs
  - crates/speclink-cli/tests/remote_connect.rs
  - crates/speclink-cli/tests/remote_read_path.rs
  - crates/speclink-cli/tests/remote_write_path.rs
  - crates/speclink-core/Cargo.toml
  - crates/speclink-core/assets/skills/archive.md
  - crates/speclink-core/assets/skills/commit.md
  - crates/speclink-core/assets/skills/discuss.md
  - crates/speclink-core/assets/skills/propose.md
  - crates/speclink-core/assets/skills/sync.md
  - crates/speclink-core/src/discuss.rs
  - crates/speclink-core/src/init.rs
  - crates/speclink-core/src/instructions.rs
  - crates/speclink-core/src/preflight.rs
  - crates/speclink-core/src/status.rs
  - crates/speclink-core/src/store.rs
  - crates/speclink-core/src/workspace.rs
  - crates/speclink-core/tests/golden/claude.snapshot.md
  - crates/speclink-core/tests/golden/codex.snapshot.md
  - crates/speclink-core/tests/golden/neutral-cli.snapshot.md
  - crates/speclink-core/tests/golden/neutral-tool-call.snapshot.md
  - crates/speclink-core/tests/golden/remote-claude.marker.md
  - crates/speclink-core/tests/mode_resolution.rs
  - crates/speclink-core/tests/render_golden.rs
  - crates/speclink-core/tests/skill_verbization.rs
  - crates/speclink-fs/src/layout.rs
  - crates/speclink-fs/src/lib.rs
  - crates/speclink-remote/Cargo.toml
  - crates/speclink-remote/src/auth.rs
  - crates/speclink-remote/src/client.rs
  - crates/speclink-remote/src/lib.rs
  - crates/speclink-remote/tests/auth_store.rs
  - crates/speclink-remote/tests/client_errors.rs
  - docs/team-mode.md
  - docs/team-mode.zh-TW.md
  - docs/verb-contract.md
  - docs/verb-contract.zh-TW.md
-->

---
### Requirement: change 的 repo 歸屬規則
每個 change SHALL 恰歸屬一個 repo（v1 一 change 一 repo）：remote 模式下建立 change 時，歸屬 SHALL 取自請求攜帶的 repo 名（單 repo 專案自動預設）；change 列舉 SHALL 依請求的 repo 過濾；跨 repo 的需求 SHALL 以拆分為多個 change 處理（每個各歸屬一個 repo），契約 SHALL NOT 提供跨 repo 歸屬的 change 形狀。此規則與列舉過濾語意 SHALL 明載於 docs/verb-contract.md。

#### Scenario: 建立的 change 歸屬當前 repo
- **WHEN** 於 repo 欄位為 backend 的專案以 remote 模式執行 speclink new change demo --agent claude
- **THEN** 建立請求攜帶 backend 身分，後續於同 repo 執行 speclink list --json 的輸出含 demo

#### Scenario: 他 repo 的 change 不出現在清單
- **WHEN** 於同專案另一個 repo（frontend）的工作目錄執行 speclink list --json
- **THEN** 歸屬 backend 的 demo 不出現在輸出清單


<!-- @trace
source: verb-contract-and-remote-client
updated: 2026-07-05
code:
  - .speclink.yaml
  - Cargo.lock
  - Cargo.toml
  - README.md
  - crates/speclink-cli/Cargo.toml
  - crates/speclink-cli/src/commands.rs
  - crates/speclink-cli/src/main.rs
  - crates/speclink-cli/src/remote_commands.rs
  - crates/speclink-cli/tests/doc_verbs.rs
  - crates/speclink-cli/tests/remote_connect.rs
  - crates/speclink-cli/tests/remote_read_path.rs
  - crates/speclink-cli/tests/remote_write_path.rs
  - crates/speclink-core/Cargo.toml
  - crates/speclink-core/assets/skills/archive.md
  - crates/speclink-core/assets/skills/commit.md
  - crates/speclink-core/assets/skills/discuss.md
  - crates/speclink-core/assets/skills/propose.md
  - crates/speclink-core/assets/skills/sync.md
  - crates/speclink-core/src/discuss.rs
  - crates/speclink-core/src/init.rs
  - crates/speclink-core/src/instructions.rs
  - crates/speclink-core/src/preflight.rs
  - crates/speclink-core/src/status.rs
  - crates/speclink-core/src/store.rs
  - crates/speclink-core/src/workspace.rs
  - crates/speclink-core/tests/golden/claude.snapshot.md
  - crates/speclink-core/tests/golden/codex.snapshot.md
  - crates/speclink-core/tests/golden/neutral-cli.snapshot.md
  - crates/speclink-core/tests/golden/neutral-tool-call.snapshot.md
  - crates/speclink-core/tests/golden/remote-claude.marker.md
  - crates/speclink-core/tests/mode_resolution.rs
  - crates/speclink-core/tests/render_golden.rs
  - crates/speclink-core/tests/skill_verbization.rs
  - crates/speclink-fs/src/layout.rs
  - crates/speclink-fs/src/lib.rs
  - crates/speclink-remote/Cargo.toml
  - crates/speclink-remote/src/auth.rs
  - crates/speclink-remote/src/client.rs
  - crates/speclink-remote/src/lib.rs
  - crates/speclink-remote/tests/auth_store.rs
  - crates/speclink-remote/tests/client_errors.rs
  - docs/team-mode.md
  - docs/team-mode.zh-TW.md
  - docs/verb-contract.md
  - docs/verb-contract.zh-TW.md
-->

---
### Requirement: store 文件讀取動詞
CLI SHALL 提供雙模式的文件讀取動詞：speclink artifact cat <artifact> --change <name> 輸出該 artifact 的內容；speclink language show 輸出共用詞彙文件內容（fs 模式讀本地檔、remote 模式經契約端點）；文件不存在時 SHALL 以非 0 exit code 與語義化訊息結束。生成技能資產中的文件閱讀指示 SHALL 使用這些動詞，SHALL NOT 指示直接讀取規格目錄下的檔案路徑。

#### Scenario: artifact cat 於兩模式輸出一致
- **WHEN** 分別於 fs 模式與 remote 模式（server 存有相同內容）執行 speclink artifact cat proposal --change 某 change
- **THEN** 兩模式 stdout 輸出相同的 artifact 內容，exit code 為 0

#### Scenario: 詞彙文件缺失
- **WHEN** 專案無共用詞彙文件時執行 speclink language show
- **THEN** exit code 非 0，stderr 訊息說明詞彙文件不存在（技能據此靜默跳過詞彙載入）

#### Scenario: 技能資產不含直接讀檔指示
- **WHEN** 執行 speclink update 後掃描生成的全部 SKILL.md 內容
- **THEN** 不存在指示直接開啟規格目錄檔案路徑的語句，文件閱讀一律以 speclink 動詞表述

<!-- @trace
source: verb-contract-and-remote-client
updated: 2026-07-05
code:
  - .speclink.yaml
  - Cargo.lock
  - Cargo.toml
  - README.md
  - crates/speclink-cli/Cargo.toml
  - crates/speclink-cli/src/commands.rs
  - crates/speclink-cli/src/main.rs
  - crates/speclink-cli/src/remote_commands.rs
  - crates/speclink-cli/tests/doc_verbs.rs
  - crates/speclink-cli/tests/remote_connect.rs
  - crates/speclink-cli/tests/remote_read_path.rs
  - crates/speclink-cli/tests/remote_write_path.rs
  - crates/speclink-core/Cargo.toml
  - crates/speclink-core/assets/skills/archive.md
  - crates/speclink-core/assets/skills/commit.md
  - crates/speclink-core/assets/skills/discuss.md
  - crates/speclink-core/assets/skills/propose.md
  - crates/speclink-core/assets/skills/sync.md
  - crates/speclink-core/src/discuss.rs
  - crates/speclink-core/src/init.rs
  - crates/speclink-core/src/instructions.rs
  - crates/speclink-core/src/preflight.rs
  - crates/speclink-core/src/status.rs
  - crates/speclink-core/src/store.rs
  - crates/speclink-core/src/workspace.rs
  - crates/speclink-core/tests/golden/claude.snapshot.md
  - crates/speclink-core/tests/golden/codex.snapshot.md
  - crates/speclink-core/tests/golden/neutral-cli.snapshot.md
  - crates/speclink-core/tests/golden/neutral-tool-call.snapshot.md
  - crates/speclink-core/tests/golden/remote-claude.marker.md
  - crates/speclink-core/tests/mode_resolution.rs
  - crates/speclink-core/tests/render_golden.rs
  - crates/speclink-core/tests/skill_verbization.rs
  - crates/speclink-fs/src/layout.rs
  - crates/speclink-fs/src/lib.rs
  - crates/speclink-remote/Cargo.toml
  - crates/speclink-remote/src/auth.rs
  - crates/speclink-remote/src/client.rs
  - crates/speclink-remote/src/lib.rs
  - crates/speclink-remote/tests/auth_store.rs
  - crates/speclink-remote/tests/client_errors.rs
  - docs/team-mode.md
  - docs/team-mode.zh-TW.md
  - docs/verb-contract.md
  - docs/verb-contract.zh-TW.md
-->

---
### Requirement: 任務取消勾選動詞

CLI SHALL 提供 speclink task undone <task-id>（旗標 --change <name> 與 --json，無 stdin），把已勾選的任務翻回未勾選。task-id SHALL 接受兩種值域：純數字（ordinal 定址，行為與輸出與現行逐位元一致）與 tsk_ 前綴的 stable ID（依 task-identity 能力查找）。取消勾選 SHALL 為純狀態翻轉：SHALL NOT 寫入 touched 記錄、SHALL NOT 變更 change 的開工標記、SHALL NOT 補指派 stable ID。省略 --change 時的變更解析規則 SHALL 與 task done 相同。成功時 exit code 為 0；任務已是未勾選、task id 既非數字亦非 tsk_ 前綴、task id 超界或查無此 stable ID、tasks.md 不存在時 SHALL 以非 0 exit code 結束、stderr 輸出語義化訊息（形狀與 task done 的對應錯誤對稱）且無任何檔案效果。remote 模式下本動詞的人眼與 --json 輸出形狀 SHALL 與 fs 模式一致。本動詞為對 Spectra 2.3.1 的刻意延伸（Spectra 無此動詞），SHALL NOT 改變既有指令的 parity 基線。

#### Scenario: 取消已勾選的任務

- **WHEN** 對 tasks.md 中第 3 個任務已勾選的 change 執行 speclink task undone 3 --change demo
- **THEN** tasks.md 僅該任務由 [x] 變回 [ ]（縮排與 bullet 風格保留、其餘內容不變），stdout 顯示成功訊息（--no-color 下無 ANSI 序列），exit code 為 0，且 .speclink/ 下無新增 touched 記錄

#### Scenario: 以 stable ID 取消勾選

- **WHEN** 對帶 tsk_ ID 且已勾選的任務執行 speclink task undone 該 ID --change demo
- **THEN** tasks.md 僅該任務由 [x] 變回 [ ]（行尾 ID 註解原文保留），exit code 為 0；--json 形狀與數字值域一致

#### Scenario: --json 輸出形狀與 task done 對稱

- **WHEN** 執行 speclink task undone 3 --change demo --json
- **THEN** stdout 為 compact 單行 JSON，鍵依序為 change、status、task_desc、task_id，status 值為 undone，exit code 為 0

##### Example: 取消第 3 個任務的 payload

- **GIVEN** change demo 的 tasks.md 第 3 個任務為已勾選的「1.3 Third」
- **WHEN** 執行 speclink task undone 3 --change demo --json
- **THEN** stdout 為 {"change":"demo","status":"undone","task_desc":"1.3 Third","task_id":"3"}

#### Scenario: 任務已是未勾選

- **WHEN** 對未勾選的任務執行 speclink task undone
- **THEN** exit code 非 0，stderr 單行訊息說明該任務已是未完成狀態，tasks.md 與 .speclink/ 無任何變更

#### Scenario: tasks.md 不存在

- **WHEN** 對沒有 tasks.md 的 change 執行 speclink task undone 1
- **THEN** exit code 非 0，stderr 訊息指出該 change 的 tasks.md 不存在（與 task done 的同情境訊息一致）

#### Scenario: remote 模式輸出形狀與 fs 一致

- **WHEN** 於 remote 模式（server 回應成功）執行 speclink task undone 3 --change demo --json
- **THEN** stdout 的 JSON 鍵名與鍵序與 fs 模式完全一致，exit code 為 0

#### Scenario: 取消勾選不回滾開工標記

- **WHEN** 對已有開工標記的 change 執行 speclink task undone 取消其唯一已勾選的任務
- **THEN** 該 change 的開工標記維持原值，touched 記錄內容不減少


<!-- @trace
source: stable-task-and-evidence
updated: 2026-07-13
code:
  - Cargo.lock
  - Cargo.toml
  - apps/desktop/core/src/manage.rs
  - apps/desktop/src-tauri/src/lib.rs
  - apps/desktop/src/App.tsx
  - apps/desktop/src/adapter/tauriDataSource.ts
  - crates/speclink-cli/src/commands.rs
  - crates/speclink-cli/tests/task_stable_id.rs
  - crates/speclink-cli/tests/task_undone.rs
  - crates/speclink-core/Cargo.toml
  - crates/speclink-core/src/archive.rs
  - crates/speclink-core/src/command/mod.rs
  - crates/speclink-core/src/newcmd.rs
  - crates/speclink-core/src/tasks.rs
  - crates/speclink-host/Cargo.toml
  - crates/speclink-host/src/evidence.rs
  - crates/speclink-host/src/lib.rs
  - crates/speclink-node/src/lib.rs
  - packages/ui/src/__tests__/taskList.test.tsx
  - packages/ui/src/adapter.ts
  - packages/ui/src/components/RichDetailDrawer.tsx
  - packages/ui/src/components/TaskList.tsx
  - packages/ui/src/tasks.ts
-->
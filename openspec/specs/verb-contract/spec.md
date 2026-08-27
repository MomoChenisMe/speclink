# verb-contract Specification

## Purpose

動詞在 remote 通道上的對外契約：涵蓋面與 payload 形狀、樂觀並行控制與 409 語意、API 版本協商與錯誤翻譯紅線、change 的 repo 歸屬規則，以及 store 文件讀取與任務取消勾選等動詞。本 capability 保證同一動詞在本機與 remote 兩模式下人眼輸出同形、--json 輸出形狀凍結，且兩模式之間的行為分岔只在單一處宣告，不散落到各動詞實作。

## Requirements

### Requirement: 動詞契約的涵蓋面與 payload 形狀

動詞契約 SHALL 涵蓋 RD 本地全流程：changes 列舉／讀取／建立／認領／歸檔／捨棄（discard，含 force 語意）／開工標記（in-progress add）、artifacts 讀取／寫入、tasks 勾選／取消勾選／搬移（1-based ordinal 定址與 before 側別）、validate 與 analyze 衍生查詢、instructions 計算、discussions 全動詞（列舉、建立含 slug 覆寫、context、add-round、conclude、archive、promote、discard 含 force 語意、link、seal）、政策讀取（workflow-config 有效值）、詞彙讀取（LANGUAGE 內容）、正典規格讀取、身分查驗。remote 模式下對應 CLI 指令的 stdout 輸出（人眼與 --json）形狀 SHALL 與 fs 模式一致（欄位 camelCase 同名）；validate 的聚合語意（無參數／--all／--changes）SHALL 由 client 以逐 change 端點呼叫組合，聚合輸出形狀與 fs 模式一致；show SHALL 由 client 以讀取端點組合出與 fs 模式同形輸出，SHALL NOT 於 remote 模式讀取本機 store。本質本機動詞（demo、trace）於 remote 模式 SHALL 明確拒絕（非零 exit code、stderr 說明僅限本機模式），SHALL NOT 靜默作用於本機 store，SHALL NOT 發出任何 server 請求；trace 的拒絕訊息 SHALL 說明溯源鏈自本地 openspec 樹組裝。契約的端點、payload 與錯誤形狀 SHALL 以 docs/verb-contract.md 為正典參考文件。

list --json 的 change 條目得含本機觀察面的可空欄位 worktree（物件：path 字串與 branch 字串）——該欄位僅於 fs 模式的主 checkout、worktree 政策開啟且映射成立時出現，缺席時 SHALL NOT 序列化；remote 模式的 list 條目 SHALL 恆缺席此欄位。形狀一致性 SHALL 以「可空且缺席不序列化」維持：無 worktree 情境下兩模式的 list 輸出逐欄位一致。

#### Scenario: remote 列舉輸出形狀與 fs 一致

- **WHEN** 於 remote 模式執行 speclink list --json，server 回傳兩個 change
- **THEN** stdout 的 JSON 欄位名與 fs 模式的 speclink list --json 完全一致，exit code 為 0

#### Scenario: instructions 由 server 計算

- **WHEN** 於 remote 模式執行 speclink instructions proposal --change 某 change --json
- **THEN** payload 含 context、rules、template、locale 等欄位（值來自 server 端政策），欄位名與 fs 模式一致

#### Scenario: remote validate 輸出形狀與 fs 一致

- **WHEN** 於 remote 模式執行 speclink validate --json（無參數聚合）
- **THEN** stdout 為與 fs 模式同形的逐 change results JSON（欄位名一致），有任一 invalid 時 exit code 非 0

#### Scenario: remote discard 的 guard 語意與本地一致

- **WHEN** 於 remote 模式對已勾選任務的 change 執行 speclink discard（無 --force）
- **THEN** exit code 非 0，stderr 語義化訊息說明已開工需 --force（與 fs 模式同語意），server 上該 change 完整保留

#### Scenario: remote 建立討論帶 slug 覆寫

- **WHEN** 於 remote 模式以中文主題執行 speclink discuss new 並帶 --slug board-search-bar
- **THEN** server 端以 board-search-bar 建立記錄，stdout 與 --json 的 slug 欄位形狀與 fs 模式一致；非法 slug 值時 exit code 非 0、stderr 說明原因、server 端不落檔

#### Scenario: remote show 輸出與 fs 一致

- **WHEN** 同一份 change 內容分別存在於 fs 專案與 remote server，兩模式各執行 speclink show 該 change
- **THEN** 兩者的人眼輸出與 --json 逐欄位一致；remote 模式的結果來自 server 資料，未讀取本機 store

#### Scenario: remote in-progress add 蓋章於 server

- **WHEN** 於 remote 模式對 server 上存在的 change 執行 speclink in-progress add
- **THEN** CLI 靜默結束（無輸出、exit 0），server 端該 change 的 meta 含 started_at 與 started_by（server 認證身分）

#### Scenario: demo 於 remote 明確拒絕

- **WHEN** 於 remote 模式執行 speclink demo
- **THEN** exit code 非 0，stderr 說明 demo 僅限本機模式，本機與 server 均未新增任何 change

#### Scenario: trace 於 remote 明確拒絕

- **WHEN** 於 remote 模式執行 speclink trace 某 capability
- **THEN** exit code 非 0，stderr 說明 trace 僅限本機模式且溯源鏈自本地 openspec 樹組裝，未發出任何 server 請求

#### Scenario: remote list 恆無 worktree 欄位

- **WHEN** 於 remote 模式執行 speclink list --json，而本機恰有合乎慣例的 linked worktree
- **THEN** 所有條目均無 worktree 欄位，欄位名與 fs 模式無 worktree 情境完全一致


<!-- @trace
source: stamp-contract-trace-docs
updated: 2026-08-27
-->

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

CLI SHALL 提供 speclink task undone <task-id>（旗標 --change <name> 與 --json，無 stdin），把已勾選的任務翻回未勾選。task-id SHALL 接受兩種值域：純數字（ordinal 定址，行為與輸出與現行逐位元一致）與 tsk_ 前綴的 stable ID（依 task-identity 能力查找）。取消勾選 SHALL 為純狀態翻轉：SHALL NOT 寫入 touched 記錄、SHALL NOT 變更 change 的開工標記、SHALL NOT 補指派 stable ID。省略 --change 時的變更解析規則 SHALL 與 task done 相同。成功時 exit code 為 0；任務已是未勾選、task id 既非數字亦非 tsk_ 前綴、task id 超界或查無此 stable ID、tasks.md 不存在時 SHALL 以非 0 exit code 結束、stderr 輸出語義化訊息（形狀與 task done 的對應錯誤對稱）且無任何檔案效果。remote 模式下本動詞的人眼與 --json 輸出形狀 SHALL 與 fs 模式一致。本動詞為 speclink 自有延伸，SHALL NOT 改變既有指令的輸出基線。

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
### Requirement: 動詞人眼輸出的兩模式同形

CLI 動詞的人眼輸出（stdout 文本，含 --no-color 模式）在本機與 remote 兩模式 SHALL 逐位元一致，僅下列明文分歧清單除外，清單外的任何輸出差異 SHALL 視為缺陷：

1. new change 的 Path 行——本機印、remote 不印（server 端路徑對本機使用者無意義）
2. list 的 worktree 標示——remote 恆缺席（worktree 是本機主 checkout 的觀察面）
3. status 的 schema 覆寫旗標——remote 以固定訊息明確拒絕（server 的 workflow config 決定 schema）
4. workflow-config 的文件標籤——remote 以 config.yaml 為標籤（server 端無本機路徑可印）
5. discuss promote 的 Path 行與其後的 propose 提示行——本機印、remote 不印（新變更目錄是 store 端的檔案系統位置，同第 1 項的裁定；兩行綁在一起去留）

同形範圍涵蓋 list、discuss 全部子指令、task done 與 task undone、in-progress remove、discard、archive、review 與 verify 的 add-round／stamp／discard／show。模式差異 SHALL 只存在於資料取得與守門拒絕，SHALL NOT 存在於輸出文本的組版。

#### Scenario: list 的 invalid 標記兩模式同形

- **WHEN** 專案含一筆 metadata 損壞的變更，分別於本機與 remote 模式執行 list
- **THEN** 兩模式 stdout 均在該變更行尾渲染 invalid 標記，整段文本逐位元一致，exit code 均為 0

#### Scenario: discuss 動詞成功訊息兩模式同形

- **WHEN** 分別於本機與 remote 模式執行 discuss 子指令（例如 new、add-round、conclude、archive）
- **THEN** 兩模式的成功訊息文本逐位元一致，exit code 一致

#### Scenario: 封存與工單閱讀對新 server 同形

- **WHEN** remote server 為新版，於 remote 模式執行 archive 與 review show
- **THEN** archive 的 stdout 含封存目的地（dated 名稱）、規格計數行、封存討論行，與本機同文本；review show 的 stdout 印出工單文件原文全文，與本機同文本

#### Scenario: 對舊 server 整體退化

- **WHEN** remote server 為舊版（回應缺新欄位），於 remote 模式執行 archive 與 review show
- **THEN** 兩指令輸出整體退回既有 remote 輸出（簡短封存行、結構化工單摘要），exit code 0，SHALL NOT 出現新舊欄位混合的部分渲染

<!-- @trace
source: cli-render-unification
updated: 2026-08-08
-->

---
### Requirement: 動詞 --json 輸出形狀凍結

動詞的 --json 輸出欄位集合與 camelCase 命名 SHALL 維持既有契約不變;工單原文 SHALL NOT 出現在任何 --json 輸出。manual-task-marker 系列的兩次刻意契約更新:instructions apply 的任務項增列 `manual` 欄位、progress 增列 `codeTotal`/`codeComplete`/`codeRemaining`;隨 `[P]` 語意移除,任務項的 `parallel` 欄位 SHALL 自基線移除——任務項欄位集合為 `id`/`description`/`done`/`manual`。除上述刻意更新外 SHALL NOT 改名或移除既有欄位;更新後的欄位集合即凍結契約的新基線。

#### Scenario: 工單 --json 兩模式同形且無原文欄位

- **WHEN** 分別於本機與 remote 模式執行 review show --json
- **THEN** 兩模式 payload 欄位集合一致——change、rounds、lastRound,rounds 各項含 index、phase、patchHash、scope、findings——且不存在攜帶工單原文的欄位

#### Scenario: instructions apply 兩模式同形且無 parallel 欄位

- **WHEN** 分別於本機與 remote 模式對同一 change 執行 instructions apply --json
- **THEN** 兩模式任務項欄位集合皆為 id/description/done/manual(無 parallel),progress 皆含 total/complete/remaining 與 codeTotal/codeComplete/codeRemaining,欄位集合一致


<!-- @trace
source: task-marker-ui-and-parallel-removal
updated: 2026-08-11
-->

---
### Requirement: 模式分岔的單點宣告

CLI 頂層動詞 SHALL 逐一歸屬四種模式形狀之一，本機（fs）/remote 的分岔決策 SHALL 集中於 dispatch 的宣告層，SHALL NOT 散佈於各動詞函式內：

- **ModeFree**（init、update、link、unlink、auth、schemas、templates、feedback、schema、config、completion）：執行 SHALL NOT 觸發 store 模式解析，dispatch SHALL NOT 因宣告層而對其引入 .speclink.yaml 的解析失敗——不讀取專案設定的動詞（completion、config）SHALL 不受壞的 .speclink.yaml 影響；部分動詞（如 schemas、templates、update）的 workspace 探索本就讀取 .speclink.yaml 以解析 spec_dir，其於壞檔下的既有失敗行為維持不變；連線管理動詞（link、unlink、auth）的連線解析由動詞自理。
- **Dual**（list、show、validate、analyze、drift、archive、discard、artifact、language、status、instructions、new、workflow-config、task、in-progress、discuss、review、verify）：fs 模式 SHALL 作用於本機 store，remote 模式 SHALL 作用於 remote store，SHALL NOT 於 remote 模式靜默作用於本機 store；宣告 SHALL 同時載明本機臂與 remote 臂，缺任一臂 SHALL 構成建置失敗而非執行期靜默回退；不消費 store 的前置步驟（instructions 的 --skill 分流、workflow-config 的 argv／stdin 正規化）SHALL 先於模式解析執行，維持既有可觀察順序。
- **FsOnly**（demo）：remote 模式 SHALL 以非零 exit code 明確拒絕，拒絕判定 SHALL 僅解析模式而不建立連線——SHALL NOT 發出任何 server 請求，離線環境同樣拒絕。
- **RemoteOnly**（claim）：fs 模式 SHALL 以非零 exit code 明確拒絕並於 stderr 說明需要 remote store。

模式判定 SHALL 惰性執行：僅於宣告形狀需要時解析模式，僅於 remote 臂將執行時建立連線。

#### Scenario: ModeFree 動詞不受壞連線設定影響

- **WHEN** 於 .speclink.yaml 內容無法解析的專案目錄下執行 speclink completion generate zsh 與 speclink config list
- **THEN** 兩者正常輸出且 exit code 為 0，stderr 不含 .speclink.yaml 的解析錯誤

#### Scenario: FsOnly 動詞於 remote 模式零請求拒絕

- **WHEN** 於 remote 模式設定且 server 不可達的環境下執行 speclink demo
- **THEN** exit code 非零，stderr 說明該動詞僅限本機模式，且過程未發出任何 server 請求

#### Scenario: RemoteOnly 動詞於 fs 模式明確拒絕

- **WHEN** 於 fs 模式專案執行 speclink claim 指定 change
- **THEN** exit code 非零，stderr 說明該動詞需要 remote store

<!-- @trace
source: cli-mode-dispatch-convergence
updated: 2026-08-10
-->
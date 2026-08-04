# worktree-overlay Specification

## Purpose

TBD - created by archiving change 'worktree-parallel-apply'. Update Purpose after archive.

## Requirements

### Requirement: worktree discovery 與映射慣例

於 local workspace 的主 checkout（workspace 根目錄的 .git 為目錄）執行 speclink list，且有效 worktree 政策為 true 時，引擎 SHALL 以 git 的 worktree 名冊（git worktree list --porcelain 的輸出語意）探索 linked worktrees，並依分支命名慣例建立 change 與 worktree 的映射。映射 SHALL 於三條件同時成立時建立：(a) 該 worktree 的分支名為 speclink/<change名>；(b) 同名 change 存在於主 workspace 且未封存；(c) 該 worktree 路徑下 openspec/changes/<change名>/ 目錄可讀。任一條件不成立時該條目 SHALL 被靜默略過並回讀主副本；detached HEAD 與資料夾已不存在的名冊條目 SHALL 一律略過。git 指令不可用或執行失敗時，discovery SHALL 回傳空映射且 list SHALL 照常輸出，SHALL NOT 因此以非零 exit code 失敗。映射 SHALL NOT 依賴 git 名冊以外的任何持久化儲存。

有效 worktree 政策為 false 或未設定時，list SHALL NOT 執行 discovery，其人眼與 --json 輸出 SHALL 與本能力導入前位元級一致。於 linked worktree 內（workspace 根目錄的 .git 為檔案）執行 list SHALL NOT 套用映射；remote workspace SHALL NOT 套用映射。

discovery 為此觀察面讀取政策時 SHALL fail-open：政策文件或應用設定不可解析時視同政策關閉——list SHALL 照常以 exit code 0 輸出且不執行 discovery，SHALL NOT 因此失敗。此為對「讀取政策的指令一律 fail-closed」的明文例外，僅限本觀察面：觀察面不得把原本會成功的指令變成失敗，而 workflow-config 等以政策為輸出主體的指令維持既有 fail-closed 行為。

#### Scenario: 壞政策文件不使 list 失敗

- **WHEN** openspec/config.yaml 含 YAML 語法錯誤，於主 checkout 執行 speclink list --json
- **THEN** exit code 為 0，所有條目均無 worktree 欄位

#### Scenario: 映射成立

- **WHEN** 主 checkout 啟用 worktree 政策，存在分支 speclink/add-dark-mode 的 linked worktree，其內 openspec/changes/add-dark-mode/ 可讀，且主 workspace 有未封存的 change add-dark-mode，執行 speclink list --json
- **THEN** add-dark-mode 條目含 worktree 欄位，exit code 為 0

#### Scenario: 分支不合慣例即略過

- **WHEN** 存在分支 feature/add-dark-mode（無 speclink/ 前綴）的 linked worktree，執行 speclink list --json
- **THEN** 所有條目均無 worktree 欄位，輸出與無 worktree 時位元級一致

#### Scenario: 同名 change 不存在即略過

- **WHEN** 存在分支 speclink/ghost-change 的 linked worktree，但主 workspace 無名為 ghost-change 的未封存 change，執行 speclink list --json
- **THEN** 輸出與無該 worktree 時位元級一致，stderr 無警告

#### Scenario: worktree 內 spec 目錄不可讀即回讀主副本

- **WHEN** 存在分支 speclink/add-dark-mode 的 linked worktree，但其內 openspec/changes/add-dark-mode/ 不存在，執行 speclink list --json
- **THEN** add-dark-mode 條目無 worktree 欄位，任務計數與狀態來自主副本

#### Scenario: 政策關閉時零介入

- **WHEN** 有效 worktree 政策為 false，存在合乎慣例的 linked worktree，執行 speclink list 與 speclink list --json
- **THEN** 兩種輸出均與本能力導入前位元級一致

#### Scenario: linked worktree 內執行不套用映射

- **WHEN** 於某 linked worktree 資料夾內執行 speclink list --json
- **THEN** 所有條目均無 worktree 欄位

#### Scenario: git 失敗時 fail-open

- **WHEN** 主 checkout 啟用 worktree 政策，但 git 執行檔不可用，執行 speclink list --json
- **THEN** exit code 為 0，輸出與無 worktree 時位元級一致


<!-- @trace
source: worktree-parallel-apply
updated: 2026-08-04
-->

---
### Requirement: list 的 worktree 觀察面輸出

映射成立的 change，speclink list SHALL 呈現 worktree 觀察面：--json 條目 SHALL 增加 worktree 物件欄位（camelCase），含 path（字串，worktree 絕對路徑）與 branch（字串，分支全名）兩欄；人眼輸出 SHALL 於該 change 既有行尾追加固定字面「 [worktree]」，--no-color 下字面相同。無映射的 change 條目 SHALL NOT 出現 worktree 欄位（缺席不序列化）。此為刻意變更：既有無 worktree 情境的輸出維持位元級不變，新增情境的 golden 與 CLI 測試同批更新。

映射成立的 change，其任務計數、狀態與開工戳記等既有欄位的「值」SHALL 來自該 worktree 副本內的 change 目錄（openspec/changes/<change名>/），欄位名與型別 SHALL 維持既有契約不變；該 change 於主副本的檔案 SHALL NOT 被 list 讀取或寫入變更。worktree 移除後重跑 list，SHALL 回讀主副本且 worktree 觀察面消失，SHALL NOT 留下任何殘餘標示或欄位。worktree 副本內 change 中介資料損壞時，SHALL 沿用既有的損壞診斷欄位行為（metaError）如實呈現。

#### Scenario: 任務計數即時反映 worktree 副本

- **WHEN** 主副本的 add-dark-mode 完成 0 個任務，其 worktree 副本內勾完 3 個任務（共 5 個），於主 checkout 執行 speclink list --json
- **THEN** add-dark-mode 條目的 completedTasks 為 3、totalTasks 為 5，且含 worktree 欄位

##### Example: 計數與欄位形狀

- **GIVEN** worktree 位於 /repos/speclink.worktrees/add-dark-mode、分支 speclink/add-dark-mode，副本內 tasks.md 勾 3／5
- **WHEN** 於主 checkout 執行 speclink list --json
- **THEN** 該條目含 "completedTasks": 3、"totalTasks": 5、"worktree": { "path": "/repos/speclink.worktrees/add-dark-mode", "branch": "speclink/add-dark-mode" }

#### Scenario: 人眼輸出的標示

- **WHEN** 映射成立，執行 speclink list 與 speclink list --no-color
- **THEN** 該 change 行尾均出現「 [worktree]」字面，兩者字面一致

#### Scenario: worktree 移除後還原

- **WHEN** 先前映射成立的 worktree 以 git worktree remove 移除後，於主 checkout 執行 speclink list --json
- **THEN** 該條目無 worktree 欄位，任務計數回讀主副本的值

#### Scenario: worktree 副本中介資料損壞如實診斷

- **WHEN** 映射成立，但 worktree 副本內該 change 的 .openspec.yaml 為無法解析的 YAML，執行 speclink list --json
- **THEN** 該條目呈現 metaError 欄位（與主副本損壞時同款行為）

<!-- @trace
source: worktree-parallel-apply
updated: 2026-08-04
-->
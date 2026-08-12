# worktree-overlay Specification

## Purpose

平行 worktree 疊在主 checkout 上的觀察語意：worktree 的探測與 change 對應慣例、list 的 worktree 觀察面輸出，以及桌面看板對 worktree 卡片的呈現。本 capability 保證使用者能在主資料夾一處看到各 worktree 的即時進度，並在 worktree 仍掛著時擋下會踩到它的桌面動詞。

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

---
### Requirement: desktop 看板的 worktree 呈現

local workspace 的 desktop 看板 SHALL 經與 CLI list 同一觀察面組裝取得 worktree facts：有 worktree 映射的 change，卡片 SHALL 帶 worktree 標示，變更抽屜 SHALL 顯示分支名（speclink/<change名>）與 worktree 路徑（OS 原生路徑形式）。文案於 zh-TW 與 en 介面語言下均直出「worktree」一詞。worktree 的增減、或其內 openspec/changes/<change名>/ 目錄的變動，SHALL 使看板自動更新（無需手動重整）；worktree 移除（merge 收尾）後標示與抽屜資訊 SHALL 退場。git 不可用時看板照常呈現且無任何 worktree 標示（fail-open，沿用 discovery 慣例）。remote 工作區不適用本需求。

對有 worktree 映射的 change，desktop 的全部 per-change 讀取面 SHALL 解析到該 worktree 副本：變更抽屜各分頁的文件原文（提案／設計／任務／規格）、metadata 欄位（建立與開工資訊）、狀態報告、驗證與分析報告，以及看板全文搜尋的 artifact 掃描。同一 change 於同一畫面的計數與內容 SHALL 同源（SHALL NOT 出現計數來自 worktree、內容來自主 checkout 的劈半）。observed_facts 為空（政策關閉、非主 checkout、git 不可用）時，上述讀取面行為 SHALL 與本需求擴充前完全相同；worktree 內 change 目錄不可讀時該條目依 discovery 慣例靜默回讀主副本。

#### Scenario: 卡片標示與抽屜資訊

- **WHEN** 存在分支 speclink/add-auth 的活躍 worktree 且 add-auth 為未封存 change 時開啟 desktop 看板
- **THEN** add-auth 卡片帶 worktree 標示，開啟其變更抽屜可見分支 speclink/add-auth 與該 worktree 的路徑

#### Scenario: worktree 內進度即時反映

- **WHEN** 於 worktree 副本內將一個任務勾為完成
- **THEN** 主看板該 change 卡片的任務計數自動更新，無需手動重整

#### Scenario: 抽屜任務內容與計數同源

- **WHEN** worktree 副本內的 tasks.md 已勾 9/10，主 checkout 的同名檔全未勾，開啟該 change 的變更抽屜任務分頁
- **THEN** 分頁徽章顯示 9/10 且下方任務清單有 9 項呈已勾狀態，兩者同源於 worktree 副本

#### Scenario: 分析報告反映 worktree 現值

- **WHEN** worktree 副本內的 proposal.md 已更新而主 checkout 仍為舊內容，於抽屜觸發分析
- **THEN** 報告依 worktree 副本的 artifact 內容產出

#### Scenario: 全文搜尋命中 worktree 內容

- **WHEN** 某字串僅存在於 worktree 副本的 design.md、不存在於主 checkout，於看板全文搜尋該字串
- **THEN** 該 change 卡片出現於命中清單，snippet 取自 worktree 副本內容

#### Scenario: 收尾後標示退場

- **WHEN** worktree 經 merge 收尾流程移除後看板刷新
- **THEN** 卡片不再帶 worktree 標示，抽屜不再顯示分支與路徑

#### Scenario: git 不可用時看板照常

- **WHEN** git 於環境中不可用時開啟 desktop 看板
- **THEN** 看板照常列出 changes 且無任何 worktree 標示，不顯示錯誤

---


<!-- @trace
source: worktree-data-routing
updated: 2026-08-05
-->

---
### Requirement: worktree 掛著時的 desktop 動詞防護

對有 worktree 映射的 change，desktop 動詞 SHALL 分兩級處理。破壞性生命週期動詞——「封存」「退回提案中」「刪除」——SHALL 拒絕執行，並提示先執行 worktree-merge 收尾；其語意為主 checkout 的 change 目錄存廢，SHALL NOT 路由至 worktree。粒度寫入動詞——任務勾選、全部勾選、任務拖排、卡片拖排、放棄審查工單——SHALL 路由至該 worktree 副本執行：檔案效果（tasks.md、change metadata、審查工單）與其側效（touched 記錄、首次完成的開工章、git 髒檔歸因）SHALL 全數落在 worktree 副本內，主 checkout 的對應檔案 SHALL NOT 被寫入。唯讀呈現（抽屜、diff 檢視）不受防護影響。observed_facts 為空時兩級動詞行為 SHALL 與無 worktree 時完全相同。此防護僅及 desktop 動詞層；CLI 對應動詞不在此限。

#### Scenario: 封存被擋

- **WHEN** 對有活躍 worktree 映射的 change 於 desktop 觸發封存
- **THEN** 動詞拒絕執行，訊息含先收尾的指引，change 目錄與看板狀態不變

#### Scenario: 刪除被擋

- **WHEN** 對有活躍 worktree 映射的 change 於 desktop 觸發刪除
- **THEN** 動詞拒絕執行，訊息含 change 名與先收尾的指引，主 checkout 與 worktree 的 change 目錄均不變

#### Scenario: 任務勾選寫入 worktree

- **WHEN** 對有活躍 worktree 映射的 change 於抽屜勾選一個任務
- **THEN** worktree 副本的 tasks.md 該任務變為已勾且 touched 記錄落在 worktree 副本，主 checkout 的 tasks.md 位元級不變

#### Scenario: 卡片拖排位置保持

- **WHEN** 於看板拖動有 worktree 映射的 change 卡片至新位置後看板重新整理
- **THEN** 卡片停留於新位置（rank 寫入 worktree 副本的 change metadata，與看板讀取同源）

#### Scenario: 收尾後解禁

- **WHEN** 該 change 的 worktree 經 merge 收尾移除後，於 desktop 再次觸發封存
- **THEN** 封存照常執行


<!-- @trace
source: worktree-data-routing
updated: 2026-08-05
-->
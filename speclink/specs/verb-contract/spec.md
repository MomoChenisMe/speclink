# verb-contract Specification

## Purpose

動詞在 remote 通道上的對外契約：涵蓋面與 payload 形狀、樂觀並行控制與 409 語意、API 版本協商與錯誤翻譯紅線、change 的 repo 歸屬規則，以及 store 文件讀取與任務取消勾選等動詞。本 capability 保證同一動詞在本機與 remote 兩模式下人眼輸出同形、--json 輸出形狀凍結，且兩模式之間的行為分岔只在單一處宣告，不散落到各動詞實作。

## Requirements

### Requirement: 動詞契約的涵蓋面與 payload 形狀

動詞契約 SHALL 涵蓋 RD 本地全流程：changes 列舉／讀取／建立／認領／歸檔／捨棄（discard，含 force 語意）／開工標記（in-progress add）、artifacts 讀取／寫入、tasks 勾選／取消勾選／搬移（1-based ordinal 定址與 before 側別）、validate 與 analyze 衍生查詢、instructions 計算、discussions 全動詞（列舉、建立含 slug 覆寫、context、add-round、conclude、archive、promote、discard 含 force 語意、link、seal）、政策讀取（workflow-config 有效值）、詞彙讀取（LANGUAGE 內容）、正式規格讀取、evidence 讀取（evidence show）、溯源（trace）、執行計畫（plan，含 --strict-overlap）、身分查驗。remote 模式下對應 CLI 指令的 stdout 輸出（人眼與 --json）形狀 SHALL 與 fs 模式一致（欄位 camelCase 同名）；validate 的聚合語意（無參數／--all／--changes）SHALL 由 client 以逐 change 端點呼叫組合，聚合輸出形狀與 fs 模式一致；show SHALL 由 client 以讀取端點組合出與 fs 模式同形輸出，SHALL NOT 於 remote 模式讀取本機 store。本質本機動詞（demo）於 remote 模式 SHALL 明確拒絕（非零 exit code、stderr 說明僅限本機模式），SHALL NOT 靜默作用於本機 store，SHALL NOT 發出任何 server 請求。trace 於 remote 模式 SHALL 由 server 以同一份引擎實作組裝溯源鏈，CLI 的 stdout（人眼與 --json）SHALL 與 fs 模式對同一內容一致，SHALL NOT 讀取本機 store。契約的端點、payload 與錯誤形狀 SHALL 以 docs/verb-contract.md 為正典參考文件。

list --json 的 change 條目得含本機觀察面的可空欄位 worktree（物件：path 字串與 branch 字串）——該欄位僅於 fs 模式的主 checkout、worktree 政策開啟且映射成立時出現，缺席時 SHALL NOT 序列化；remote 模式的 list 條目 SHALL 恆缺席此欄位。形狀一致性 SHALL 以「可空且缺席不序列化」維持：無 worktree 情境下兩模式的 list 輸出逐欄位一致。

list --json 的 change 條目 SHALL 含 `created`（字串，`YYYY-MM-DD` 的建立日期；metadata 沒有或無法解析時缺席；metadata 記的值不是日期時照原樣輸出，與 speclink show 相同，SHALL NOT 改寫或捨棄）與 `modifiedAt`（字串，RFC 3339 的 UTC 時間，該 change 底下的文件最後一次被修改的時間；儲存答不出時缺席，SHALL NOT 以 0 或空字串代替）。兩個欄位在兩種模式 SHALL 同名同型；人眼輸出 SHALL NOT 因這兩個欄位而改變。

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

#### Scenario: remote list 恆無 worktree 欄位

- **WHEN** 於 remote 模式執行 speclink list --json，而本機恰有合乎慣例的 linked worktree
- **THEN** 所有條目均無 worktree 欄位，欄位名與 fs 模式無 worktree 情境完全一致

#### Scenario: trace 於 remote 走 server

- **WHEN** server 的某個 capability 有兩個已封存的變更動過它，於 remote 模式執行 speclink trace 該 capability，再執行同一指令加 --json
- **THEN** 兩次 exit code 皆為 0；人眼輸出與 --json 和 fs 模式對相同內容的輸出逐位元相同；未讀取本機 store

#### Scenario: remote trace 找不到 capability

- **WHEN** 於 remote 模式執行 speclink trace 一個不存在的 capability
- **THEN** exit code 非 0，stderr 是與 fs 模式相同的找不到訊息與近似建議

#### Scenario: list --json 帶出時間欄位

- **WHEN** 於 fs 模式對一個 2026-11-01 建立的變更執行 speclink list --json
- **THEN** 該條目的 created 為 `2026-11-01`，modifiedAt 是一個 RFC 3339 的 UTC 時間字串；speclink list 的人眼輸出與新增欄位前逐位元相同

#### Scenario: 勾選任務後 modifiedAt 不倒退

- **WHEN** 記下某個變更的 modifiedAt，對它執行 speclink task done，再執行 speclink list --json
- **THEN** 該條目新的 modifiedAt 不早於先前的值

#### Scenario: 儲存答不出最後修改時間

- **WHEN** 於 remote 模式連到一個不回報文件最後修改時間的 server，執行 speclink list --json
- **THEN** 各條目沒有 modifiedAt 欄位，created 照常存在，exit code 為 0


<!-- @trace
source: remote-read-verbs
updated: 2026-11-12T15:18:56+08:00
-->

---
### Requirement: 樂觀並行控制與 409 語意
artifact 寫入 SHALL 攜帶讀取時取得的版本（If-Match）；版本過期時 server 回 409 且 body SHALL 含機器可判的 reason 欄位。reason 的值 SHALL 取自 Protocol 的錯誤 reason 註冊表（409 用到 `revision_conflict` 與 `refused`），SHALL NOT 使用註冊表以外的值。CLI 對 409 SHALL 輸出可讀的建議動作訊息，SHALL NOT 顯示裸狀態碼：reason 為 `revision_conflict` 時 SHALL 說明內容在讀取後已被更新、建議重新讀取後再套用修改；reason 為 `refused` 時 SHALL 轉述 server 的訊息（認領衝突時訊息含目前持有人與建議動作）。

#### Scenario: 版本衝突的可讀訊息
- **WHEN** 於 remote 模式寫入 artifact，而該 artifact 已被他人更新（server 回 409、reason 為 revision_conflict）
- **THEN** exit code 非 0，stderr 單行訊息含 `content changed since you read it — re-read it and re-apply your edit`，不顯示裸狀態碼

#### Scenario: 認領被搶佔
- **WHEN** 執行 speclink claim 某 change，而 server 回 409、reason 為 refused（已被他人認領）
- **THEN** exit code 非 0，stderr 訊息含目前持有人資訊與建議動作，例如 `change 'add-auth' is already claimed by Alice <alice@example.com> — coordinate with them, or ask them to release it`


<!-- @trace
source: sync-specs-after-docs-cleanup
updated: 2026-10-08T10:00:10+08:00
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

CLI SHALL 提供雙模式的文件讀取動詞：speclink artifact cat <artifact> --change <name> 輸出該 artifact 的內容；speclink language show 輸出共用詞彙文件內容（fs 模式讀本地檔、remote 模式經契約端點）；文件不存在時 SHALL 以非 0 exit code 與語義化訊息結束。生成技能資產中的文件閱讀指示 SHALL 使用這些動詞，SHALL NOT 指示直接讀取規格目錄下的檔案路徑。讀取正式規格的指示 SHALL 以 speclink show <capability> --item-type spec 表述（不指定類型時，同名的作用中變更優先），SHALL NOT 指示以 speclink list --specs --json 回傳的 path 欄位開啟檔案——該欄位在 remote 模式是儲存的鍵，不是本機路徑。全部產生的技能中改寫既有 artifact 的指示 SHALL 以「speclink artifact cat 讀出全文、修改後以 speclink new artifact 帶 --stdin --force 整份寫回」表述，SHALL NOT 留下未指名動詞的讀取或改寫步驟。讀取 evidence 的指示 SHALL 以 speclink evidence show 表述；判斷變更有哪些 delta 的指示 SHALL 以 speclink list --json 的 deltaCapabilities 表述；判斷變更閒置多久的指示 SHALL 以 speclink list --json 的 created 與 modifiedAt 表述，SHALL NOT 以 git 歷史判斷。以「規格資料夾不在這個 checkout」產生的技能 SHALL NOT 含規格資料夾的路徑，SHALL NOT 含 `.evidence.json` 或 `.meta.yaml`，SHALL NOT 含對規格檔的 git 步驟。以「規格資料夾在這個 checkout」產生的技能中，含規格資料夾路徑的句子 SHALL 只用於 git 步驟與路徑的顯示，SHALL NOT 是讀取或改寫的指示。直接寫入規格資料夾的技能（baseline 與 manual）不受這兩句限制，豁免 SHALL 明列於守門測試；這兩支技能以「規格資料夾不在這個 checkout」產生時 SHALL 在動手寫入之前先說明不支援並結束（見 baseline-skill 與 manual-skill）。

#### Scenario: artifact cat 於兩模式輸出一致
- **WHEN** 分別於 fs 模式與 remote 模式（server 存有相同內容）執行 speclink artifact cat proposal --change 某 change
- **THEN** 兩模式 stdout 輸出相同的 artifact 內容，exit code 為 0

#### Scenario: 詞彙文件缺失
- **WHEN** 專案無共用詞彙文件時執行 speclink language show
- **THEN** exit code 非 0，stderr 訊息說明詞彙文件不存在（技能據此靜默跳過詞彙載入）

#### Scenario: 技能資產不含直接讀檔指示
- **WHEN** 執行 speclink update 後掃描生成的全部 SKILL.md 內容
- **THEN** 不存在指示直接開啟規格目錄檔案路徑的語句，文件閱讀一律以 speclink 動詞表述

#### Scenario: discuss 以 show 讀正式規格

- **WHEN** 檢視生成的 discuss 技能中讀取候選規格 Purpose 的步驟
- **THEN** 該步驟以 speclink show <capability> --item-type spec 表述；全文不存在把 path 欄位與 spec.md 寫在同一句的指示

#### Scenario: 三支技能的改寫經動詞

- **WHEN** 檢視生成的 propose 與 ingest 技能中修正或更新既有 artifact 的步驟
- **THEN** 每個這樣的步驟都指向「speclink artifact cat 讀出、speclink new artifact 帶 --stdin --force 寫回」的做法；兩支技能的全文不含 config.yaml

#### Scenario: remote 模式下 discuss 讀得到正式規格

- **WHEN** 於 remote 模式（checkout 內沒有規格資料夾）依 discuss 技能的步驟讀取某個 capability 的 Purpose
- **THEN** 執行的是 speclink show <capability> --item-type spec，stdout 含該規格的 Purpose，exit code 為 0

#### Scenario: 同名的作用中變更不會被誤讀成正式規格

- **WHEN** 專案有一個與某 capability 同名的作用中變更，依 discuss 技能的步驟讀那個 capability 的 Purpose
- **THEN** 執行的指令帶 --item-type spec，stdout 是正式規格的內容，不是那個變更的內容

#### Scenario: remote 專案的技能檔不含規格路徑

- **WHEN** 以規格資料夾名稱 openspec、「規格資料夾不在這個 checkout」渲染 baseline 與 manual 以外的全部技能（Claude 與中性 tool-call 兩種目標）
- **THEN** 結果不含 `openspec/`，不含 `.evidence.json`，不含 `.meta.yaml`

#### Scenario: 本機專案的技能檔不以路徑讀寫

- **WHEN** 以「規格資料夾在這個 checkout」渲染同一批技能
- **THEN** 含 `openspec/` 的每一段（到下一個空白行為止的連續幾行；以冒號結尾的引導句和緊接在後的那一段算同一段）都不含 read、open、glob、grep、cat、check for、check whether、compare … against、edit、write、rewrite 這些字，也不含它們加了字首或字尾的寫法（例如 reads、re-read、editing、rewritten、overwrite）

#### Scenario: commit 與 worktree 開工前以動詞讀 evidence

- **WHEN** 檢視產生的 commit 與 apply-with-worktree 技能中取得「這個變更改了哪些檔」的步驟
- **THEN** 該步驟以 speclink evidence show 表述

#### Scenario: 閒置的判斷不用 git 歷史

- **WHEN** 檢視產生的 apply 與 drift 技能中判斷變更閒置多久的步驟
- **THEN** 該步驟以 speclink list --json 的 created 與 modifiedAt 表述，並寫明沒有 modifiedAt 時只看建立時間；apply 在把變更標成進行中之前讀取這兩個欄位，apply-with-worktree 在建立 worktree 之前讀取（這兩個動作都會讓 modifiedAt 變成現在）；全文不含對規格資料夾的 git log

#### Scenario: 守門抓到新加入的路徑讀取

- **WHEN** 在一份未豁免的原稿加入一句以路徑讀取規格檔的指示
- **THEN** 守門測試失敗並指名該技能


<!-- @trace
source: skills-verbs-second-wave
updated: 2026-11-12T16:54:51+08:00
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

同形範圍涵蓋 list、plan（含 --strict-overlap）、trace、evidence show、discuss 全部子指令、task done 與 task undone、in-progress remove、discard、archive、review 與 verify 的 add-round／stamp／discard／show。模式差異 SHALL 只存在於資料取得與守門拒絕，SHALL NOT 存在於輸出文本的組版。

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

#### Scenario: plan 的 --strict-overlap 於 remote 同形

- **WHEN** server 與本機 fs 專案含相同的兩個變更（皆含同一 capability 的 delta、動到不同 requirement），分別於兩模式執行 speclink plan --strict-overlap 與加 --json 的同一指令
- **THEN** 兩模式的 stdout 逐位元相同，兩個變更被排進不同的波次；remote 模式未讀取本機 store

#### Scenario: evidence show 兩模式同形

- **WHEN** server 與本機 fs 專案含相同的變更與相同的 evidence 記錄，分別於兩模式執行 speclink evidence show 該變更 --no-color
- **THEN** 兩模式的 stdout 逐位元相同


<!-- @trace
source: remote-read-verbs
updated: 2026-11-12T15:18:56+08:00
-->

---
### Requirement: 動詞 --json 輸出形狀凍結

動詞的 --json 輸出欄位集合與 camelCase 命名 SHALL 維持既有契約不變;工單原文 SHALL NOT 出現在不帶 `--detail` 的任何 --json 輸出；只有 `review show --json --detail` 與 `verify show --json --detail` SHALL 以 `content` 鍵帶出工單原文。manual-task-marker 系列的兩次刻意契約更新:instructions apply 的任務項增列 `manual` 欄位、progress 增列 `codeTotal`/`codeComplete`/`codeRemaining`;隨 `[P]` 語意移除,任務項的 `parallel` 欄位 SHALL 自基線移除——任務項欄位集合為 `id`/`description`/`done`/`manual`。另一次刻意契約更新:list 的 change 條目增列 `created` 與 `modifiedAt` 兩個可缺席的欄位。再一次刻意契約更新：`discuss list --json` 的每一項、`discuss show --json` 的 `info` 與 `discuss search --json` 的每一筆命中增列可缺席的 `promotedTo`，只在該討論有轉出或併入的變更時出現，依字母順序排在 `path` 與 `rounds` 之間。除上述刻意更新外 SHALL NOT 改名或移除既有欄位;更新後的欄位集合即凍結契約的新基線。

#### Scenario: 工單 --json 兩模式同形且無原文欄位

- **WHEN** 分別於本機與 remote 模式執行 review show --json
- **THEN** 兩模式 payload 欄位集合一致——change、rounds、lastRound,rounds 各項含 index、phase、patchHash、scope、findings——且不存在攜帶工單原文的欄位

#### Scenario: instructions apply 兩模式同形且無 parallel 欄位

- **WHEN** 分別於本機與 remote 模式對同一 change 執行 instructions apply --json
- **THEN** 兩模式任務項欄位集合皆為 id/description/done/manual(無 parallel),progress 皆含 total/complete/remaining 與 codeTotal/codeComplete/codeRemaining,欄位集合一致

#### Scenario: list 新增欄位不影響既有欄位

- **WHEN** 比對新增欄位前後 speclink list --json 對同一專案的輸出
- **THEN** 既有欄位的名稱、型別與值不變，差異只有每個 change 條目多出的 created 與 modifiedAt

#### Scenario: 帶 --detail 的工單 --json 才有原文

- **WHEN** 對同一張工單分別執行 review show --json 與 review show --json --detail
- **THEN** 前者沒有 `content` 鍵，輸出與引入 `--detail` 之前逐位元相同；後者多一個 `content` 鍵，其餘鍵與值和前者相同

#### Scenario: 討論的 promotedTo 不影響既有欄位

- **WHEN** 比對新增 `promotedTo` 前後 speclink discuss list --json 對同一個專案的輸出，專案裡有一份已轉出的討論與一份沒有轉出的討論
- **THEN** 沒有轉出的那一項逐位元相同；已轉出的那一項既有鍵的名稱、相對順序與值不變，差異只有多出的 `promotedTo`；對 speclink discuss search --json 的命中項作同樣的比對，結果相同


<!-- @trace
source: archived-list-and-detail-reads
updated: 2026-11-23T13:33:38+08:00
-->

---
### Requirement: 模式分岔的單點宣告

CLI 頂層動詞 SHALL 逐一歸屬四種模式形狀之一，本機（fs）/remote 的分岔決策 SHALL 集中於 dispatch 的宣告層，SHALL NOT 散佈於各動詞函式內：

- **ModeFree**（init、update、link、unlink、auth、schemas、templates、feedback、schema、config、completion）：執行 SHALL NOT 觸發 store 模式解析，dispatch SHALL NOT 因宣告層而對其引入 .speclink.yaml 的解析失敗——不讀取專案設定的動詞（completion、config）SHALL 不受壞的 .speclink.yaml 影響；部分動詞（如 schemas、templates、update）的 workspace 探索本就讀取 .speclink.yaml 以解析 spec_dir，其於壞檔下的既有失敗行為維持不變；連線管理動詞（link、unlink、auth）的連線解析由動詞自理。
- **Dual**（list、show、validate、analyze、drift、archive、discard、artifact、language、status、instructions、new、workflow-config、task、in-progress、discuss、review、verify、plan、change、trace、evidence）：fs 模式 SHALL 作用於本機 store，remote 模式 SHALL 作用於 remote store，SHALL NOT 於 remote 模式靜默作用於本機 store；宣告 SHALL 同時載明本機臂與 remote 臂，缺任一臂 SHALL 構成建置失敗而非執行期靜默回退；不消費 store 的前置步驟（instructions 的 --skill 分流、workflow-config 的 argv／stdin 正規化）SHALL 先於模式解析執行，維持既有可觀察順序。
- **FsOnly**（demo）：remote 模式 SHALL 以非零 exit code 明確拒絕，拒絕判定 SHALL 僅解析模式而不建立連線——SHALL NOT 發出任何 server 請求，離線環境同樣拒絕。Dual 家族內 SHALL NOT 有只限本機的子指令或旗標。
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

#### Scenario: plan 與 change depends 於 remote 模式走 server

- **WHEN** 於 remote 模式執行 speclink plan --json 與 speclink change depends add-b --on add-a
- **THEN** 兩者分別對 server 發出 GET /plan 與 POST /changes/add-b/depends，stdout 形狀與 fs 模式一致，未觸碰本機 store

#### Scenario: change rank 於 remote 模式走 server

- **WHEN** 於 remote 模式執行 speclink change rank c --before a
- **THEN** 對 server 發出 POST /changes/c/rank，stdout 形狀與 fs 模式一致，未觸碰本機 store

#### Scenario: trace 與 evidence show 於 remote 模式走 server

- **WHEN** 於 remote 模式執行 speclink trace 某 capability 與 speclink evidence show 某 change
- **THEN** 兩者分別對 server 發出 GET /trace/{capability} 與 GET /changes/{name}/evidence，stdout 形狀與 fs 模式一致，未觸碰本機 store

#### Scenario: plan --strict-overlap 於 remote 模式走 server

- **WHEN** 於 remote 模式執行 speclink plan --strict-overlap --json
- **THEN** 對 server 發出帶 strictOverlap=true 的 GET /plan，stdout 形狀與 fs 模式一致


<!-- @trace
source: remote-read-verbs
updated: 2026-11-12T15:18:56+08:00
-->

---
### Requirement: artifact cat 讀取已封存變更的文件

speclink artifact cat 的 --change 名稱沒有對應的進行中變更時，SHALL 改讀已封存的變更：名稱等於封存名稱、或等於封存名稱去掉開頭日期後的名稱者，有多筆時取最新的一筆——與 evidence show 動詞同一份解析。artifact 的識別沿用既有詞彙（proposal、design、tasks、specs/<capability>）。同名的進行中變更存在時 SHALL 讀進行中的那一份。兩者皆無、或該變更沒有這份文件時 SHALL 以非 0 exit code 與既有的語義化訊息結束。--change 的名稱不是單一路徑段（含 `/` 或 `\`，或為 `.`、`..`、`archive`）時 SHALL 視為找不到，SHALL NOT 查詢任何儲存。對進行中的變更，輸出 SHALL 維持既有的位元級輸出。remote 模式 SHALL 經既有的 artifact 端點得到相同的結果，SHALL NOT 讀取本機 store。

#### Scenario: 以原本的名稱讀已封存變更的提案

- **WHEN** 變更 demo 已封存為 2026-11-12-demo，執行 speclink artifact cat proposal --change demo
- **THEN** stdout 是該封存變更的提案全文，exit code 為 0

#### Scenario: 以封存名稱讀 delta spec

- **WHEN** 執行 speclink artifact cat specs/auth --change 2026-11-12-demo
- **THEN** stdout 是該封存變更對 auth 的 delta spec 全文

#### Scenario: 進行中的變更優先且輸出不變

- **WHEN** 已封存 2026-11-01-demo，另有進行中的變更 demo，執行 speclink artifact cat proposal --change demo
- **THEN** stdout 是進行中那一份提案，與新增這條需求前的輸出逐位元相同

#### Scenario: 兩邊都沒有

- **WHEN** 進行中與已封存都沒有名為 ghost 的變更，執行 speclink artifact cat proposal --change ghost
- **THEN** exit code 非 0，stderr 是既有的找不到訊息

#### Scenario: 名稱不是單一路徑段

- **WHEN** 執行 speclink artifact cat proposal --change archive/2026-11-12-demo
- **THEN** exit code 非 0，stderr 是找不到該變更的訊息，stdout 為空

#### Scenario: remote 模式讀已封存變更

- **WHEN** 於 remote 模式執行 speclink artifact cat proposal --change demo，demo 在 server 上已封存
- **THEN** stdout 與 fs 模式對相同內容的輸出相同

<!-- @trace
source: remote-read-verbs
updated: 2026-11-12T15:18:56+08:00
-->

---
### Requirement: 清單的詳細輸出

`speclink list --json` SHALL 接受旗標 `--detail`。帶上它時，`changes` 的每一項 SHALL 在既有欄位之外增列下列選填的鍵，每個鍵 SHALL 只在該變更有對應的記錄時出現：

- `review`、`verify`：物件。形狀 SHALL 與 `speclink show <change> --json --detail` 的同名鍵相同，但 SHALL NOT 含 `scope`（`state` 是 `inProgress` 或 `stamped`，蓋章後另有蓋章的時間、輪數、任務數等既有欄位）。`state` 是 `inProgress` 時 SHALL 另有 `rounds`（整數）：工單目前的輪數；工單無法解析時 SHALL 不輸出 `rounds`，清單 SHALL NOT 因此失敗。該關卡沒有工單也沒有章時 SHALL 不輸出這個鍵。
- `claimed`：物件，`by`（字串）、有記錄才出現的 `byId`（字串）與 `at`（字串）。形狀 SHALL 與 `show --json` 的 `claimed` 相同。沒有被認領時 SHALL 不輸出。
- `branch`：字串，開工時記下的分支。沒有記錄時 SHALL 不輸出。
- `landed`：物件，只有 `commit`（字串）。沒有落地記錄時 SHALL 不輸出。
- `createdBy`：字串，metadata 記下的建立者。沒有記錄時 SHALL 不輸出。
- `fromDiscussions`：字串陣列，這個變更的來源討論的 slug，順序 SHALL 與 `show --json` 的 `fromDiscussions` 相同。沒有來源討論時 SHALL 不輸出。

鍵名一律 camelCase。清單的既有欄位、項目的順序與 `specs` 區段 SHALL 不受 `--detail` 影響；metadata 損壞的變更 SHALL 照既有行為列出（帶 `metaError`），且 SHALL NOT 帶上述任何一個鍵。

不帶 `--detail` 的 `speclink list --json` SHALL 維持既有的位元級輸出。不帶 `--json` 的 `--detail`、以及 `--detail` 與 `--specs` 並用，SHALL 以用法錯誤拒絕：CLI 以非零 exit code 結束、訊息走 stderr、stdout 沒有輸出；經宿主套件執行時以 `invalid_argv` 拒絕。`--detail` SHALL NOT 改變人眼輸出，因為它不能脫離 `--json` 使用。輸出 SHALL 不隨 `locale`（`tw`、`ja`、`en` 或未設定）改變。

三種執行方式對相同的資料 SHALL 輸出相同的鍵與值：本機專案、連到遠端的 CLI、宿主套件。連到遠端的 CLI SHALL 以既有的變更清單回應組出這些鍵，SHALL NOT 對每個變更另外發一次請求。連到不提供這些欄位的舊版 Server 時，對應的鍵 SHALL 不出現，指令 SHALL NOT 失敗。離線或認證失效時 SHALL 與不帶 `--detail` 的 `list` 相同地失敗。本機專案沒有認領，所以 `claimed` 在本機 SHALL 不出現（除非 metadata 本來就帶著認領欄位）。

蓋章之後檔案是否又被改過，SHALL NOT 由清單判斷：`review` 與 `verify` 只反映記錄下來的狀態，與 `show --json` 相同。

`--detail` 與 `--archived` 並用 SHALL 以用法錯誤拒絕（見「已封存變更的清單」）。

#### Scenario: 有記錄的變更帶出四類狀態

- **WHEN** 變更 demo 的審查已蓋章、驗證有未結的工單、由 Alice（識別 u_1）認領、開工時記下分支 `speclink/demo`、落地的 commit 是 `0123456789abcdef0123456789abcdef01234567`，執行 `speclink list --json --detail`
- **THEN** demo 這一項含 `review`（`state` 是 `stamped`）、`verify`（`state` 是 `inProgress`）、`claimed`（`by` 是 `Alice`、`byId` 是 `u_1`、有 `at`）、`branch`（`speclink/demo`）、`landed`（`{ "commit": "0123456789abcdef0123456789abcdef01234567" }`），既有的欄位與不帶 `--detail` 時相同

##### Example: 各種記錄對應的鍵

| 變更的記錄 | 多出的鍵 |
| --- | --- |
| 什麼記錄都沒有 | 沒有任何新鍵；這一項與不帶 `--detail` 時相同 |
| 只有審查章 | `review` |
| 審查工單未結（兩輪）、沒有章 | `review`（`state` 是 `inProgress`、`rounds` 是 2） |
| metadata 記下建立者 `Ann <ann@example.com>` | `createdBy` |
| 由討論 `two` 轉出 | `fromDiscussions`（`["two"]`） |
| 被認領、沒有識別 | `claimed`（只有 `by` 與 `at`） |
| 記下分支、還沒落地 | `branch` |
| 已落地 | `branch`（有記錄時）、`landed` |

#### Scenario: 不帶 --detail 時輸出不變

- **WHEN** 對同一個專案執行 `speclink list --json`，專案裡有已蓋章、被認領、已落地的變更
- **THEN** 輸出與引入 `--detail` 之前逐位元相同，任何一項都沒有 `review`、`verify`、`claimed`、`branch`、`landed`、`createdBy`、`fromDiscussions`

#### Scenario: --detail 不帶 --json 被拒絕

- **WHEN** 執行 `speclink list --detail`
- **THEN** 指令以非零 exit code 結束，stderr 指出 `--detail` 要和 `--json` 一起用，stdout 沒有清單

#### Scenario: --detail 與 --specs 並用被拒絕

- **WHEN** 執行 `speclink list --specs --json --detail`
- **THEN** 指令以非零 exit code 結束，stderr 指出兩個旗標不能並用

#### Scenario: 三種執行方式輸出相同

- **WHEN** 本機專案、一台 Server 與一個宿主套件的儲存含相同的變更與相同的記錄（不含認領），分別執行 `speclink list --json --detail`、連到該 Server 執行同一個指令、以及經宿主套件執行同一個 argv
- **THEN** 三個結果的 `changes` 深度相等（本機專案才有的 `worktree` 鍵除外）

#### Scenario: 連到舊版 Server 時不失敗

- **WHEN** CLI 連到一台變更清單回應沒有關卡、分支與落地欄位的 Server，執行 `speclink list --json --detail`
- **THEN** 指令成功，每一項都沒有新鍵

#### Scenario: metadata 損壞的變更不帶新鍵

- **WHEN** 專案含一筆 metadata 損壞的變更，執行 `speclink list --json --detail`
- **THEN** 該項照既有行為帶 `metaError`，沒有 `review`、`verify`、`claimed`、`branch`、`landed`、`createdBy`、`fromDiscussions`；指令成功

#### Scenario: 帶出建立的人與來源討論

- **WHEN** 變更 add-two 的 metadata 記下建立者 `Ann <ann@example.com>`，它由討論 two 轉出；變更 solo 沒有建立者記錄也沒有來源討論；執行 `speclink list --json --detail`
- **THEN** add-two 這一項含 `"createdBy": "Ann <ann@example.com>"` 與 `"fromDiscussions": ["two"]`；solo 這一項沒有這兩個鍵

#### Scenario: 進行中的關卡帶輪數

- **WHEN** 變更 demo 的審查工單有兩輪、沒有章，執行 `speclink list --json --detail`
- **THEN** demo 這一項的 `review` 是 `{ "state": "inProgress", "rounds": 2 }`

#### Scenario: 舊版 Server 沒有回報進行中的輪數

- **WHEN** CLI 連到一台關卡物件在 `inProgress` 時沒有 `rounds` 的 Server，demo 的審查工單未結，執行 `speclink list --json --detail`
- **THEN** 指令成功，demo 這一項的 `review` 是 `{ "state": "inProgress" }`


<!-- @trace
source: archived-list-and-detail-reads
updated: 2026-11-23T13:33:38+08:00
-->

---
### Requirement: 內嵌宿主的人眼輸出不印路徑行

經宿主套件執行動詞而得到的人眼輸出，SHALL NOT 含下列路徑行，因為文件存在宿主的儲存裡，沒有對應的本機路徑：

1. `new change` 的 `Path:` 行。
2. `discuss promote` 的 `Path:` 行與緊接其後的 propose 提示行（兩行一起去留，與 remote 模式的裁定相同）。
3. `discuss new` 的 `Path:` 行。

這三個動詞的其餘輸出行 SHALL 與本機模式逐位元相同。它們的 `--json` 輸出 SHALL 不變，`path` 鍵照舊出現。其他動詞經宿主套件的人眼輸出 SHALL 不受本需求影響。CLI 在本機與 remote 兩模式的輸出、以及 Node SDK 的 createEngine（本機資料夾）的輸出 SHALL 維持既有的位元級輸出；「動詞人眼輸出的兩模式同形」的分歧清單 SHALL 不變。

#### Scenario: 經宿主套件建立變更不印路徑

- **WHEN** 經宿主套件執行 `new change demo`（人眼形式）
- **THEN** 輸出的第一行是 `✓ Created change: demo`，有 `Schema:` 行，沒有以 `Path:` 開頭的行

##### Example: 三個動詞在兩種執行方式的輸出

| 動詞 | 本機 CLI 的輸出行 | 經宿主套件的輸出行 |
| --- | --- | --- |
| `new change demo` | `✓ Created change: demo`、`Path: …`、`Schema: spec-driven` | `✓ Created change: demo`、`Schema: spec-driven` |
| `discuss new "Topic" --slug topic` | `✓ Created discussion: topic`、`Topic: Topic`、`Path: …` | `✓ Created discussion: topic`、`Topic: Topic` |
| `discuss promote topic` | `✓ Promoted discussion 'topic' → change 'topic'`、`Path: …`、propose 提示行 | `✓ Promoted discussion 'topic' → change 'topic'` |

#### Scenario: JSON 的 path 鍵保留

- **WHEN** 經宿主套件執行 `discuss new "Topic" --slug topic --json`
- **THEN** 回傳的物件含 `path` 鍵，與引入本需求之前相同

#### Scenario: 本機資料夾的引擎照常印路徑

- **WHEN** 以 Node SDK 的 createEngine 對一個本機專案資料夾執行 `new change demo`
- **THEN** 輸出含 `Path:` 行，與引入本需求之前逐位元相同

<!-- @trace
source: verb-output-for-embedded
updated: 2026-11-18T21:38:05+08:00
-->

---
### Requirement: 詞彙文件的寫入動詞

CLI SHALL 提供 `speclink language replace --stdin`：自 stdin 讀入內容，把它整份寫成專案的共用詞彙文件（LANGUAGE 文件）。文件不存在時 SHALL 建立；存在時 SHALL 以 stdin 的內容整份取代，不做合併。stdin 的內容是空的或只有空白時 SHALL 拒絕且零寫入。沒有帶 `--stdin` 時 SHALL 以用法錯誤拒絕。寫入之後，`speclink language show` SHALL 輸出剛寫入的內容。

動詞 SHALL 是雙模式的，並在內嵌宿主可用：本機專案寫規格資料夾裡的 `LANGUAGE.md`；連到遠端的 CLI 經契約端點寫入；宿主套件經引擎命令（穩定名稱 `language-replace`）寫入，寫入之前 SHALL 先問執行前掛鉤。成功的寫入 SHALL 隨一次提交落盤並發布事件 `language-replaced`。這個動詞 SHALL 只需要一般的寫入權限，SHALL NOT 被當成流程設定的政策鍵。

輸出：成功時 stdout SHALL 是 `✓ Replaced the LANGUAGE document`；這次呼叫建立了原本不存在的文件時 SHALL 是 `✓ Created the LANGUAGE document`。exit code SHALL 是 0。`--no-color` 下文字相同、不含色碼。`--json` SHALL 輸出一個物件，含 `created`（布林：這次呼叫是否建立了原本不存在的文件）。失敗時訊息走 stderr、exit code 非零、stdout 沒有輸出。人眼輸出在本機、remote 與宿主套件三種執行方式 SHALL 逐位元一致，且 SHALL 不隨 `locale`（`tw`、`ja`、`en` 或未設定）改變。

遠端路徑：撞上 revision 衝突時 SHALL 沿既有的寫入動詞行為失敗並請使用者重跑，文件維持衝突前的內容；離線或認證失效時 SHALL 直接失敗、不排隊；沒有寫入權限的身分 SHALL 被拒絕且文件不變。

生成技能資產中改寫詞彙文件的指示 SHALL 以「`speclink language show` 讀出全文、修改後以 `speclink language replace --stdin` 整份寫回」表述，SHALL NOT 指示直接編輯規格資料夾裡的檔案。

#### Scenario: 取代既有的詞彙文件

- **WHEN** 專案已有詞彙文件，使用者把修改後的全文經 stdin 交給 `speclink language replace --stdin`，接著執行 `speclink language show`
- **THEN** 第一個指令 exit code 0、stdout 是 `✓ Replaced the LANGUAGE document`；第二個指令輸出的內容與 stdin 的內容相同

#### Scenario: 文件不存在時建立

- **WHEN** 專案沒有詞彙文件（`speclink language show` 會失敗），使用者執行 `speclink language replace --stdin --json` 並從 stdin 給內容
- **THEN** exit code 0，stdout 是 `{"created":true}`；之後 `speclink language show` 成功並輸出該內容

#### Scenario: 空內容被拒絕

- **WHEN** stdin 是空的或只有空白與換行，執行 `speclink language replace --stdin`
- **THEN** 指令以非零 exit code 結束，stderr 指出內容是空的，既有的詞彙文件逐位元不變

##### Example: 輸入與結果

| 既有文件 | stdin | 結果 | `created` |
| --- | --- | --- | --- |
| 有 | `# Language\n\n## 詞彙\n` | 整份取代 | false |
| 沒有 | `# Language\n` | 建立 | true |
| 有 | （空字串） | 拒絕，文件不變 | 不輸出 |
| 有 | `   \n\n` | 拒絕，文件不變 | 不輸出 |

#### Scenario: 三種執行方式結果相同

- **WHEN** 以相同的內容分別在本機專案執行、連到 Server 執行、以及經宿主套件執行 `language replace --stdin`
- **THEN** 三者的人眼輸出逐位元相同；三邊之後的 `language show` 都輸出該內容

#### Scenario: 宿主的掛鉤擋下寫入

- **WHEN** 宿主的執行前掛鉤對命令 `language-replace` 回答拒絕、原因「只有維護者可以改詞彙表」，使用者經宿主套件執行 `language replace --stdin`
- **THEN** 呼叫以 `refused` 失敗，訊息是「只有維護者可以改詞彙表」，詞彙文件不變

#### Scenario: 沒有寫入權限時被拒絕

- **WHEN** 連到遠端專案、只有讀取權限的使用者執行 `speclink language replace --stdin`
- **THEN** 指令以非零 exit code 結束，詞彙文件不變

<!-- @trace
source: language-replace-and-discuss-restore
updated: 2026-11-18T23:16:44+08:00
-->

---
### Requirement: 單筆變更的詳細輸出

`speclink show <change> --json` SHALL 接受旗標 `--detail`，不讀 stdin。帶上它時，輸出的 `review` 與 `verify` 物件在 `state` 為 `stamped` 時 SHALL 在既有欄位之外增列 `scope`：陣列，每一項是 `{ "path": 字串, "hash": 字串 }`，`path` 是蓋章時記下的 repo 相對路徑（以 `/` 分隔），`hash` 是蓋章時記下的內容指紋（小寫十六進位），順序 SHALL 與蓋章時記錄的順序相同。已蓋章但沒有記下任何指紋的關卡，`scope` SHALL 是空陣列。`scope` 的鍵名與每一項的鍵名 SHALL 與線上格式單筆變更回應的 `scope` 相同。`state` 為 `inProgress` 的關卡 SHALL 在 `state` 之外增列 `rounds`（整數）：工單目前的輪數；工單無法解析時 SHALL 不輸出 `rounds`，指令 SHALL NOT 因此失敗。沒有工單也沒有章而不輸出的關卡 SHALL 與不帶 `--detail` 時相同。`show` 的其他欄位 SHALL 不受 `--detail` 影響；`show` 的對象是正式規格時（`--item-type spec`，或名稱解析成正式規格），輸出 SHALL 與不帶 `--detail` 時相同。

不帶 `--detail` 的 `speclink show <change> --json` SHALL 維持既有的位元級輸出，SHALL NOT 含 `scope`，進行中的關卡 SHALL NOT 含 `rounds`。`speclink list --json --detail` SHALL NOT 含 `scope`。不帶 `--json` 的 `--detail` SHALL 以用法錯誤拒絕：CLI 以非零 exit code 結束、訊息走 stderr 並指出 `--json`、stdout 沒有輸出；經宿主套件執行（`host.dispatch`、掛鉤的 `read`、Node 的 `engine.dispatch`）時以 `invalid_argv` 拒絕。成功時 exit code SHALL 是 0。`--detail` SHALL NOT 改變人眼輸出，因為它不能脫離 `--json` 使用；`--no-color` 對 JSON 輸出沒有作用。輸出 SHALL 不隨 `locale`（`tw`、`ja`、`en` 或未設定）改變。這個旗標 SHALL NOT 寫入任何檔案，`.meta.yaml` 與 `.speclink/` 底下的檔案 SHALL 不變。

三種執行方式對相同的資料 SHALL 輸出相同的 `review` 與 `verify`：本機專案、連到遠端的 CLI、宿主套件。連到遠端的 CLI SHALL 以既有的單筆變更回應組出 `scope` 與進行中的 `rounds`，SHALL NOT 另外發請求。連到不回報關卡欄位的舊版 Server 時，`review` 與 `verify` SHALL 不出現，指令 SHALL NOT 失敗；Server 的進行中關卡沒有 `rounds` 時，輸出的進行中關卡 SHALL 沒有 `rounds`。離線或認證失效時 SHALL 與不帶 `--detail` 的 `show` 相同地失敗。這個動詞只讀取，沒有 revision 衝突的情況。

蓋章之後檔案是否又被改過，SHALL NOT 由這個輸出判斷：`scope` 只是蓋章時記下的指紋，比對由呼叫端自己做。

#### Scenario: 已蓋章的關卡帶出指紋

- **WHEN** 變更 demo 的審查已蓋章，章記下 `src/a.rs` 與 `src/b.rs` 兩個檔的指紋；驗證有未結的工單，工單有兩輪；執行 `speclink show demo --json --detail`
- **THEN** exit code 是 0；`review` 含 `state` `stamped`、`stampedAt`、`rounds`、`tasksTotal` 與 `scope`，`scope` 依序是 `src/a.rs` 與 `src/b.rs` 兩項，各有 `path` 與 `hash`；`verify` 是 `{ "state": "inProgress", "rounds": 2 }`，沒有 `scope`

##### Example: 各種關卡記錄對應的輸出

| 關卡的記錄 | 不帶 `--detail` | 帶 `--detail` |
| --- | --- | --- |
| 沒有工單也沒有章 | 不輸出這個關卡 | 不輸出這個關卡 |
| 工單未結（兩輪） | `{ "state": "inProgress" }` | `{ "state": "inProgress", "rounds": 2 }` |
| 已蓋章，記下兩個檔 | 既有欄位，沒有 `scope` | 既有欄位加 `scope`（兩項） |
| 已蓋章，沒有記下任何指紋 | 既有欄位，沒有 `scope` | 既有欄位加 `"scope": []` |

#### Scenario: 不帶 --detail 時輸出不變

- **WHEN** 對同一個已蓋章的 demo 執行 `speclink show demo --json`
- **THEN** 輸出與引入 `--detail` 之前逐位元相同，`review` 與 `verify` 都沒有 `scope`

#### Scenario: 清單的詳細輸出不帶指紋

- **WHEN** 對含已蓋章的 demo 的專案執行 `speclink list --json --detail`
- **THEN** demo 這一項的 `review` 沒有 `scope`

#### Scenario: --detail 不帶 --json 被拒絕

- **WHEN** 執行 `speclink show demo --detail`
- **THEN** 指令以非零 exit code 結束，stderr 指出 `--detail` 要和 `--json` 一起用，stdout 沒有輸出

#### Scenario: 宿主套件以 invalid_argv 拒絕

- **WHEN** 經 `host.dispatch` 執行 `['show', 'demo', '--detail']`，以及在 `beforeCommand` 裡執行 `read(['show', 'demo', '--detail'])`
- **THEN** 兩者都以 Error 拒絕，`code` 是 `invalid_argv`，儲存沒有收到寫入

#### Scenario: 正式規格不受影響

- **WHEN** 對正式規格 auth 執行 `speclink show auth --item-type spec --json --detail`
- **THEN** 輸出與 `speclink show auth --item-type spec --json` 相同

#### Scenario: 三種執行方式輸出相同

- **WHEN** 本機專案、一台 Server 與一個宿主套件的儲存含相同的變更 demo 與相同的審查章，分別執行 `speclink show demo --json --detail`、連到該 Server 執行同一個指令、以及經 `host.dispatch` 執行同一個 argv
- **THEN** 三個結果的 `review` 深度相等，都含相同的 `scope`

#### Scenario: 連到舊版 Server 時不失敗

- **WHEN** CLI 連到一台單筆變更回應沒有關卡欄位的 Server，執行 `speclink show demo --json --detail`
- **THEN** 指令成功，輸出沒有 `review` 與 `verify`

#### Scenario: 離線時與不帶 --detail 相同地失敗

- **WHEN** CLI 連到的 Server 無法連線，執行 `speclink show demo --json --detail`
- **THEN** exit code 與 stderr 訊息與執行 `speclink show demo --json` 時相同，stdout 沒有輸出

#### Scenario: 不帶 --detail 時進行中的關卡沒有輪數

- **WHEN** 變更 demo 的驗證工單有兩輪、沒有章，執行 `speclink show demo --json`
- **THEN** `verify` 是 `{ "state": "inProgress" }`，輸出與引入進行中的 `rounds` 之前逐位元相同


<!-- @trace
source: archived-list-and-detail-reads
updated: 2026-11-23T13:33:38+08:00
-->

---
### Requirement: --json 輸出裡既有的 snake_case 欄位是凍結契約

動詞的 `--json` 輸出裡，下表的欄位名是 snake_case。它們 SHALL 維持現有的名稱，SHALL NOT 改成 camelCase，也 SHALL NOT 消失；它們和 camelCase 的欄位一樣屬於「動詞 --json 輸出形狀凍結」的契約。本需求不改變任何輸出：它把既有的例外明文列出。

| 動詞 | snake_case 的欄位 |
| --- | --- |
| `task done`、`task undone` | `task_id`、`task_desc` |
| `discuss archive` | `archived_to` |
| `analyze` | `change_id`、`artifacts_analyzed`、`artifacts_missing`、`finding_count`，以及每一條發現的 `summary_msg`、`recommendation_msg` |
| `drift` | `change_id`、`last_commit`、`commits_since_created`、`broken_anchors`、`tasks_maybe_resolved`、`tasks_blocked_external`、`spec_assumptions`、`total_score`、`primary_recommendation`、`contributes_to_total` |

之後新增的 `--json` 欄位 SHALL 是 camelCase。使用文件 SHALL 說明這件事：`docs/verb-contract.md` 與 `docs/verb-contract.zh-TW.md` SHALL 列出上表的動詞與欄位，並寫明它們是凍結的；`docs/sdk-node.md` 與 `docs/sdk-node.zh-TW.md` 說明 dispatch 結果的欄位命名時 SHALL NOT 寫成全部都是 camelCase，SHALL 指向那份清單。

#### Scenario: task done 的欄位名不變

- **WHEN** 變更 demo 有任務 `tsk_01`，執行 `speclink task done tsk_01 --change demo --json`
- **THEN** exit code 是 0；stdout 的 JSON 物件含 `task_id` 與 `task_desc` 兩個鍵，沒有 `taskId` 或 `taskDesc`

#### Scenario: discuss archive 的欄位名不變

- **WHEN** 討論 demo-topic 已有結論，執行 `speclink discuss archive demo-topic --json`
- **THEN** exit code 是 0；stdout 的 JSON 物件含 `archived_to`，沒有 `archivedTo`

#### Scenario: analyze 的欄位名不變

- **WHEN** 執行 `speclink analyze demo --json`
- **THEN** exit code 是 0；stdout 的 JSON 物件含 `change_id`、`artifacts_analyzed`、`artifacts_missing`，每個 dimension 含 `finding_count`；有發現時，每一條發現含 `summary_msg` 與 `recommendation_msg`

#### Scenario: 文件列出全部例外

- **WHEN** 檢視 `docs/verb-contract.md` 與 `docs/verb-contract.zh-TW.md` 說明 `--json` 欄位命名的段落
- **THEN** 兩份文件都列出上表的四組動詞與各自的欄位名，並寫明這些欄位不會改名

#### Scenario: SDK 文件不再寫全部是 camelCase

- **WHEN** 檢視 `docs/sdk-node.md` 與 `docs/sdk-node.zh-TW.md` 說明 dispatch 結果的段落
- **THEN** 兩份文件都寫明結果與 CLI 的 `--json` 輸出相同、欄位大多是 camelCase、少數既有欄位是 snake_case，並連到 verb-contract 文件的清單

<!-- @trace
source: sdk-command-names-and-transfer-reasons
updated: 2026-11-20T15:39:51+08:00
-->

---
### Requirement: 已封存變更的清單

`speclink list` SHALL 接受旗標 `--archived`，不讀 stdin。帶上它時，stdout SHALL 是一個 JSON 物件 `{ "archived": [ … ] }`，SHALL NOT 含 `changes` 與 `specs` 鍵；成功時 exit code SHALL 是 0。`archived` 的每一項代表一個已封存的變更，SHALL 含下列鍵（鍵名一律 camelCase）：

- `datedName`：字串，封存後帶日期的名稱（例如 `2026-11-22-add-two`）。SHALL 一定出現。
- `date`：字串，`datedName` 開頭的日期 `YYYY-MM-DD`；名稱沒有日期前綴時 SHALL 是空字串。SHALL 一定出現。
- `name`：字串，去掉日期前綴的名稱；名稱沒有日期前綴時 SHALL 等於 `datedName`。SHALL 一定出現。
- `reviewOutcome`、`verifyOutcome`：字串，`none`、`stamped` 或 `notPassed`，判定 SHALL 與線上格式已封存清單的同名欄位相同（封存的文件含該關卡的工單是 `notPassed`；否則 metadata 有章是 `stamped`；否則 `none`）。SHALL 一定出現。
- `tasksTotal`、`tasksDone`：整數。封存的文件含任務清單時兩者 SHALL 都出現，否則 SHALL 都不出現。
- `createdBy`：字串。metadata 沒有建立者時 SHALL 不出現。
- `fromDiscussions`：字串陣列。沒有來源討論時 SHALL 不出現。
- `deltaCapabilities`：字串陣列，封存的文件裡有 delta spec 的 capability 名稱，依名稱遞增。沒有時 SHALL 不出現。
- `fields`：物件，自由欄位。沒有時 SHALL 不出現。

項目 SHALL 依 `datedName` 由大到小排列；`--sort` SHALL NOT 影響這個順序。沒有已封存的變更時輸出 SHALL 是 `{ "archived": [] }`。metadata 無法解析的已封存變更 SHALL 照常列出：`createdBy`、`fromDiscussions`、`fields` 不出現，兩個結局以「沒有章」判定，指令 SHALL NOT 失敗。

`--archived` 不帶 `--json`，以及 `--archived` 與 `--specs`、`--changes` 或 `--detail` 並用，SHALL 以用法錯誤拒絕：CLI 以非零 exit code 結束、訊息走 stderr、stdout 沒有輸出；經宿主套件執行時以 `invalid_argv` 拒絕。`--archived` 沒有人眼輸出，因為它不能脫離 `--json` 使用；`--no-color` 對 JSON 輸出沒有作用。輸出 SHALL 不隨 `locale`（`tw`、`ja`、`en` 或未設定）改變。這個旗標只讀取：SHALL NOT 寫入任何檔案，`.meta.yaml` 與 `.speclink/` 底下的檔案 SHALL 不變。不帶 `--archived` 的 `speclink list` 的人眼輸出與 `--json` 輸出 SHALL 維持既有的位元級輸出。

三種執行方式對相同的資料 SHALL 輸出相同的鍵與值：本機專案、連到遠端的 CLI、宿主套件。連到遠端的 CLI SHALL 以既有的已封存清單端點組出這份輸出，SHALL NOT 對每個已封存的變更另外發一次請求，SHALL NOT 讀取本機的規格資料夾。連到已封存清單的項目沒有 `deltaCapabilities` 的舊版 Server 時，該鍵 SHALL 不出現，指令 SHALL NOT 失敗。離線或認證失效時 SHALL 與不帶 `--archived` 的 `list` 相同地失敗。這個動詞只讀取，沒有 revision 衝突的情況。經宿主套件執行時，執行前掛鉤 SHALL 收到命令名稱 `list`，參數裡 `archived` 是 true。

已封存變更的文件 SHALL 以既有的 `speclink artifact cat <artifact> --change <datedName>` 讀取；這條需求 SHALL NOT 新增讀文件的動詞。

#### Scenario: 列出已封存的變更

- **WHEN** 專案有兩個已封存的變更：`2026-11-22-add-two`（任務 3 個全部完成、建立者 `Ann <ann@example.com>`、由討論 two 轉出、對 auth 有 delta spec、兩個關卡都蓋了章）與 `2026-11-10-old-one`（沒有任務清單、沒有建立者、沒有章也沒有工單），執行 `speclink list --archived --json`
- **THEN** exit code 是 0；`archived` 依序是 `2026-11-22-add-two` 與 `2026-11-10-old-one`；輸出沒有 `changes` 與 `specs` 鍵

##### Example: 兩個項目的內容

| 鍵 | `2026-11-22-add-two` | `2026-11-10-old-one` |
| --- | --- | --- |
| `datedName` | `2026-11-22-add-two` | `2026-11-10-old-one` |
| `date` | `2026-11-22` | `2026-11-10` |
| `name` | `add-two` | `old-one` |
| `tasksTotal`、`tasksDone` | 3、3 | 兩個鍵都不出現 |
| `createdBy` | `Ann <ann@example.com>` | 不出現 |
| `fromDiscussions` | `["two"]` | 不出現 |
| `deltaCapabilities` | `["auth"]` | 不出現 |
| `reviewOutcome` | `stamped` | `none` |
| `verifyOutcome` | `stamped` | `none` |
| `fields` | 不出現 | 不出現 |

#### Scenario: 帶著未結的工單封存

- **WHEN** 一個變更帶著未結的驗證工單被封存，執行 `speclink list --archived --json`
- **THEN** 它的 `verifyOutcome` 是 `notPassed`

#### Scenario: 沒有已封存的變更

- **WHEN** 專案沒有任何已封存的變更，執行 `speclink list --archived --json`
- **THEN** exit code 是 0，stdout 是 `{ "archived": [] }`

#### Scenario: 用清單的名稱讀文件

- **WHEN** 清單有一項 `datedName` 是 `2026-11-22-add-two`、`deltaCapabilities` 是 `["auth"]`，執行 `speclink artifact cat specs/auth --change 2026-11-22-add-two`
- **THEN** stdout 是該封存變更對 auth 的 delta spec 全文

#### Scenario: 不帶 --json 被拒絕

- **WHEN** 執行 `speclink list --archived`
- **THEN** 指令以非零 exit code 結束，stderr 指出 `--archived` 要和 `--json` 一起用，stdout 沒有輸出

#### Scenario: 與其他旗標並用被拒絕

- **WHEN** 分別執行 `speclink list --archived --json --specs`、`speclink list --archived --json --changes` 與 `speclink list --archived --json --detail`
- **THEN** 三次都以非零 exit code 結束，stderr 指出兩個旗標不能並用，stdout 沒有輸出

#### Scenario: 宿主套件以 invalid_argv 拒絕

- **WHEN** 經 `host.dispatch` 執行 `['list', '--archived']`
- **THEN** 呼叫以 Error 拒絕，`code` 是 `invalid_argv`，儲存沒有收到寫入

#### Scenario: 三種執行方式輸出相同

- **WHEN** 本機專案、一台 Server 與一個宿主套件的儲存含相同的已封存變更，分別執行 `speclink list --archived --json`、連到該 Server 執行同一個指令、以及經 `host.dispatch` 執行同一個 argv
- **THEN** 三個結果的 `archived` 深度相等

#### Scenario: 連到舊版 Server 時不失敗

- **WHEN** CLI 連到一台已封存清單的項目沒有 `deltaCapabilities` 的 Server，執行 `speclink list --archived --json`
- **THEN** 指令成功，每一項都沒有 `deltaCapabilities`，其餘的鍵照常輸出

#### Scenario: metadata 無法解析的已封存變更

- **WHEN** 已封存變更 `2026-11-01-broken` 的 metadata 無法解析，它有任務清單（2 個任務、1 個完成），執行 `speclink list --archived --json`
- **THEN** 指令成功；這一項有 `datedName`、`date`、`name`、`tasksTotal` 2、`tasksDone` 1，`reviewOutcome` 與 `verifyOutcome` 都是 `none`，沒有 `createdBy`、`fromDiscussions`、`fields`

#### Scenario: 不帶 --archived 時輸出不變

- **WHEN** 對同一個有已封存變更的專案執行 `speclink list` 與 `speclink list --json`
- **THEN** 兩個輸出與引入 `--archived` 之前逐位元相同

#### Scenario: 離線時與不帶 --archived 相同地失敗

- **WHEN** CLI 連到的 Server 無法連線，執行 `speclink list --archived --json`
- **THEN** exit code 與 stderr 訊息與執行 `speclink list --json` 時相同，stdout 沒有輸出

<!-- @trace
source: archived-list-and-detail-reads
updated: 2026-11-23T13:33:38+08:00
-->

---
### Requirement: 關卡工單的詳細輸出

`speclink review show <change> --json` 與 `speclink verify show <change> --json` SHALL 接受旗標 `--detail`，不讀 stdin。帶上它時，輸出 SHALL 在既有的 `change`、`rounds`、`lastRound` 之外增列 `content`：字串，工單文件的全文，SHALL 與同一個動詞不帶 `--json` 時印到 stdout 的原文逐字相同。既有三個鍵的值 SHALL 與不帶 `--detail` 時相同。成功時 exit code SHALL 是 0。

不帶 `--detail` 的 `review show --json` 與 `verify show --json` SHALL 維持既有的位元級輸出，SHALL NOT 含 `content`。不帶 `--json` 的 `--detail` SHALL 以用法錯誤拒絕：CLI 以非零 exit code 結束、訊息走 stderr 並指出 `--json`、stdout 沒有輸出；經宿主套件執行時以 `invalid_argv` 拒絕。`--detail` SHALL NOT 改變人眼輸出，因為它不能脫離 `--json` 使用；`--no-color` 對 JSON 輸出沒有作用。輸出 SHALL 不隨 `locale`（`tw`、`ja`、`en` 或未設定）改變。這個旗標只讀取：SHALL NOT 寫入任何檔案。該變更沒有工單、或變更不存在時，指令 SHALL 與不帶 `--detail` 時相同地失敗（非零 exit code、既有的 stderr 訊息）。

三種執行方式對相同的工單 SHALL 輸出相同的鍵與值：本機專案、連到遠端的 CLI、宿主套件。連到遠端的 CLI SHALL 以既有的工單回應的 `content` 組出這個鍵，SHALL NOT 另外發請求。連到工單回應沒有 `content` 的舊版 Server 時，輸出 SHALL 沒有 `content` 鍵，指令 SHALL NOT 失敗。離線或認證失效時 SHALL 與不帶 `--detail` 的同一個動詞相同地失敗。這個動詞只讀取，沒有 revision 衝突的情況。

#### Scenario: 帶出審查工單的全文

- **WHEN** 變更 demo 的審查工單有兩輪，執行 `speclink review show demo --json --detail`
- **THEN** exit code 是 0；輸出有 `change`、`rounds`（長度 2）、`lastRound` 與 `content`；`content` 與 `speclink review show demo --no-color` 印出的原文逐字相同

#### Scenario: 驗證工單同樣帶出全文

- **WHEN** 變更 demo 的驗證工單有一輪，執行 `speclink verify show demo --json --detail`
- **THEN** 輸出的 `content` 是驗證工單的全文，`rounds` 長度是 1

#### Scenario: 不帶 --detail 時輸出不變

- **WHEN** 對同一張工單執行 `speclink review show demo --json`
- **THEN** 輸出與引入 `--detail` 之前逐位元相同，沒有 `content` 鍵

#### Scenario: --detail 不帶 --json 被拒絕

- **WHEN** 執行 `speclink review show demo --detail`
- **THEN** 指令以非零 exit code 結束，stderr 指出 `--detail` 要和 `--json` 一起用，stdout 沒有輸出

#### Scenario: 宿主套件讀得到工單全文

- **WHEN** 經 `host.dispatch` 執行 `['review', 'show', 'demo', '--json', '--detail']`
- **THEN** 回傳的物件有 `content`，值是 demo 的審查工單全文

#### Scenario: 沒有工單

- **WHEN** 變更 demo 沒有審查工單，執行 `speclink review show demo --json --detail`
- **THEN** exit code 非零，stderr 與執行 `speclink review show demo --json` 時相同，stdout 沒有輸出

#### Scenario: 三種執行方式輸出相同

- **WHEN** 本機專案、一台 Server 與一個宿主套件的儲存含相同的變更 demo 與相同的審查工單，分別執行 `speclink review show demo --json --detail`、連到該 Server 執行同一個指令、以及經 `host.dispatch` 執行同一個 argv
- **THEN** 三個結果深度相等

#### Scenario: 連到舊版 Server 時不失敗

- **WHEN** CLI 連到一台工單回應沒有 `content` 的 Server，執行 `speclink review show demo --json --detail`
- **THEN** 指令成功，輸出有 `change`、`rounds`、`lastRound`，沒有 `content`

<!-- @trace
source: archived-list-and-detail-reads
updated: 2026-11-23T13:33:38+08:00
-->

---
### Requirement: 清單的詳細輸出與 show 帶說明、階段與時間

`speclink list --json --detail` 的 `changes` 每一項，與 `speclink show <change> --json` 的輸出，SHALL 在既有的鍵之外增列下列鍵（鍵名一律 camelCase）：

- `stage`：字串，`proposed`、`in-progress` 或 `ready`。SHALL 與 `speclink plan --json` 對同一個變更回報的 `stage` 相同，且 SHALL NOT 依賴 plan 的計算：宣告的依賴成環、plan 失敗時，這個鍵 SHALL 照樣有值。metadata 讀得出來的變更 SHALL 一定有這個鍵。
- `whyExcerpt`：字串，提案第一個 `## Why` 段落裡第一個有內容的行的原文（去掉前後空白），SHALL NOT 截斷。空行、`#` 開頭的行（例如 `### 子標題`）與 HTML 註解（從 `<!--` 開頭的行起，到第一個含 `-->` 的行為止）SHALL NOT 算內容；`## ` 開頭的行結束段落。提案不存在、沒有 `## Why` 段落或第一個 Why 段落裡沒有內容時 SHALL 不輸出；後面的 `## Why` 段落 SHALL NOT 遞補。
- `startedAt`：字串，開工的日期（`YYYY-MM-DD`）。metadata 沒有開工標記時 SHALL 不輸出。
- `readyAt`：字串，任務全部完成的日期（`YYYY-MM-DD`）。SHALL 只在 metadata 記有就緒時間、而且這個變更目前至少有一個任務並且全部勾完時輸出（見 change-lifecycle「任務全部完成時記下就緒時間」）。

`show --json` SHALL 另外輸出 `createdBy`（字串，metadata 記下的建立者），沒有記錄時 SHALL 不輸出。

既有的 `summary`（含 30 個字元的上限）、`status` 與 `created` SHALL 維持既有的值與格式。`status` 的值 SHALL 只有兩個：至少有一個任務而且全部勾完時是 `done`，其他情況是 `in-progress`；`status` 是 `done` 時 `stage` SHALL 是 `ready`。

不帶 `--detail` 的 `speclink list --json` SHALL 維持既有的位元級輸出，SHALL NOT 含上述任何一個鍵。metadata 損壞的變更在清單裡 SHALL NOT 帶上述任何一個鍵。`speclink status --json`、`speclink plan --json` 與全部的人眼輸出 SHALL NOT 因這條需求而改變。輸出 SHALL 不隨 `locale` 改變。

三種執行方式對相同的資料 SHALL 輸出相同的鍵與值：本機專案、連到遠端的 CLI、宿主套件。連到遠端的 CLI SHALL 以既有的變更清單回應與單一變更回應組出這些鍵，SHALL NOT 對每個變更另外發一次請求。連到不提供這些欄位的舊版 Server 時，清單裡對應的鍵 SHALL 不出現，指令 SHALL NOT 失敗。

#### Scenario: 清單的詳細輸出帶出新的鍵

- **WHEN** 變更 demo 的提案 Why 段落第一行是「這個變更讓 Desktop Client Credentials 登入可以在沒有瀏覽器的機器上完成。」，metadata 有 `started_at: 2026-11-24T17:03:18+08:00` 與 `ready_at: 2026-11-25T09:00:00+08:00`，3 個任務全部勾完，執行 `speclink list --json --detail`
- **THEN** exit code 為 0；demo 這一項的 `stage` 是 `ready`、`whyExcerpt` 是那一行的全文、`startedAt` 是 `2026-11-24`、`readyAt` 是 `2026-11-25`；`summary` 仍是截到 30 個字元的「這個變更讓 Desktop Client Credentia…」，`status` 是 `done`

##### Example: 階段與任務數、開工標記的對應

| 任務（完成／總數） | 有開工標記 | `stage` | `status` |
| --- | --- | --- | --- |
| 0／3 | 否 | `proposed` | `in-progress` |
| 0／3 | 是 | `in-progress` | `in-progress` |
| 1／3 | 否 | `in-progress` | `in-progress` |
| 3／3 | 否 | `ready` | `done` |
| 0／0 | 否 | `proposed` | `in-progress` |

#### Scenario: Why 段落開頭是子標題或註解

- **WHEN** 變更 demo 的提案是 `## Why` 之後依序有空行、`### 背景`、`<!-- 佔位`、`還沒寫 -->`，接著是「這次要補上說明。」，執行 `speclink list --json --detail` 與 `speclink show demo --json`
- **THEN** 兩個輸出的 `whyExcerpt` 都是「這次要補上說明。」

#### Scenario: 第一個 Why 段落沒有內容

- **WHEN** 變更 demo 的提案是 `## Why` 之後只有一個註解行，接著 `## Impact`，再接著第二個 `## Why` 與「後面的說明。」，執行 `speclink list --json --detail`
- **THEN** exit code 為 0；demo 這一項沒有 `whyExcerpt`

#### Scenario: show 帶出同樣的鍵與建立者

- **WHEN** 對同一個變更 demo（metadata 另有 `created_by: Ann <ann@example.com>`）執行 `speclink show demo --json`
- **THEN** exit code 為 0；輸出含 `stage`（`ready`）、`whyExcerpt`、`startedAt`（`2026-11-24`）、`readyAt`（`2026-11-25`）與 `createdBy`（`Ann <ann@example.com>`），值與清單的詳細輸出相同；既有的 `proposal`、`tasks` 等鍵不變

#### Scenario: 依賴成環時清單照樣有階段

- **WHEN** 變更 a 與 b 的 metadata 互相宣告為前置，`speclink plan --json` 因成環而失敗，執行 `speclink list --json --detail`
- **THEN** exit code 為 0；a 與 b 兩項都有 `stage`

#### Scenario: 取消勾選後就緒日不出現

- **WHEN** 變更 demo 的 3 個任務全部勾完、metadata 記有就緒時間，之後取消勾選其中一個任務，執行 `speclink list --json --detail` 與 `speclink show demo --json`
- **THEN** 兩個輸出都沒有 `readyAt`，`stage` 都是 `in-progress`

#### Scenario: 不帶 detail 的清單不變

- **WHEN** 對同一個專案執行不帶 `--detail` 的 `speclink list --json`
- **THEN** 輸出與這條需求加入之前逐位元相同，沒有 `stage`、`whyExcerpt`、`startedAt`、`readyAt`

#### Scenario: 連到舊版 Server 時鍵缺席

- **WHEN** CLI 連到一台回應裡沒有 `stage`、`whyExcerpt` 與 `readyAt` 的舊版 Server，執行 `speclink list --json --detail`
- **THEN** exit code 為 0；每一項都沒有這三個鍵，其他鍵照常輸出

<!-- @trace
source: change-and-discussion-list-fields
updated: 2026-11-26T12:03:40+08:00
-->

---
### Requirement: 建立者識別的輸出

metadata 或討論記錄記有建立者的識別時，下列輸出 SHALL 以 `createdById`（字串，原樣）帶出它：`speclink list --json --detail` 的每個變更、`speclink show <change> --json`、`speclink discuss list --json`（含 `--archived`）的每一筆、`speclink discuss show <slug> --json` 的 `info`，以及 `speclink list --archived --json` 的每一筆；`speclink discuss search --json` 的命中項是討論清單的項目加上 `matches`，所以每一筆命中 SHALL 帶出與清單相同的 `createdById`。沒有記錄時 SHALL 不輸出這個鍵，SHALL NOT 輸出 null 或空字串。`createdBy` 既有的輸出 SHALL 不變。三種執行方式（本機專案、連到遠端的 CLI、宿主套件）對相同的資料 SHALL 輸出相同的值；連到不提供這個欄位的舊版 Server 時鍵 SHALL 不出現，指令 SHALL NOT 失敗。

#### Scenario: 五個輸出都帶出識別

- **WHEN** 變更 demo 與討論 two 的記錄各有建立者的識別 `u_1`，另有一個已封存的變更 old 的 metadata 也記有 `u_1`，依序執行 `speclink list --json --detail`、`speclink show demo --json`、`speclink discuss list --json`、`speclink discuss show two --json`、`speclink list --archived --json`
- **THEN** 五個指令的 exit code 都是 0；demo、two 與 old 對應的項目（`discuss show` 是 `info`）都有 `"createdById": "u_1"`

#### Scenario: 沒有識別時不輸出

- **WHEN** 變更 plain 的 metadata 只有 `created_by`、沒有識別，執行 `speclink list --json --detail` 與 `speclink show plain --json`
- **THEN** 兩個輸出都有 `createdBy`、都沒有 `createdById` 這個鍵

<!-- @trace
source: change-and-discussion-list-fields
updated: 2026-11-26T12:03:40+08:00
-->
# host-runtime Specification

## Purpose

應用層 Host 作為引擎唯一邊界的語意：ExecutionContext 由 Host 解析且不可被呼叫端覆寫、Project 與 Repo binding 的驗證、lifecycle gate 作為單一裁決點，以及 TeamStore 的 unit of work 與事件提交歸屬。本 capability 保證引擎規格面不讀 process 環境變數、不讀 git 身分——這些一律由 Host 注入，且組裝點遷移不改變任何對外輸出。

## Requirements

### Requirement: ExecutionContext 由 Host 解析且不可覆寫

SpeclinkExecutionContext（actor、project、repo、mode、resolved workflow policy 含 digest）SHALL 由 Host 在進入點解析一次；Engine 的 command 輸入 SHALL NOT 含 actor 或 policy 欄位，呼叫端與模型 SHALL NOT 能經 command 參數傳入或覆寫 identity。本地 fs 模式的 actor SHALL 沿用現行 git config 身分語意：無 git 或未設 user.name 時為無章匿名，行為與現行一致。

#### Scenario: command 無從攜帶 identity

- **WHEN** 檢視命令層的 Command 輸入型別並嘗試以任意 command 參數影響蓋章身分
- **THEN** Command 封閉 enum 不存在 actor 或 policy 欄位；蓋章內容只隨 ExecutionContext 的 actor 改變

#### Scenario: 本地 actor 語意不變

- **WHEN** 在設有 git user.name 與 user.email 的 workspace 執行 new change，與移除 git 身分後再執行一次
- **THEN** 前者的 created_by 章與現行版本逐位元一致；後者沿用現行無章行為


<!-- @trace
source: host-runtime-binding-policy
updated: 2026-07-13
code:
  - Cargo.lock
  - Cargo.toml
  - apps/desktop/core/Cargo.toml
  - apps/desktop/core/src/discussions.rs
  - apps/desktop/core/src/manage.rs
  - apps/desktop/core/src/query.rs
  - apps/desktop/core/src/settings.rs
  - apps/desktop/core/src/verbs.rs
  - crates/speclink-cli/Cargo.toml
  - crates/speclink-cli/src/commands.rs
  - crates/speclink-cli/src/remote_commands.rs
  - crates/speclink-core/src/archive.rs
  - crates/speclink-core/src/command/mod.rs
  - crates/speclink-core/src/config.rs
  - crates/speclink-core/src/demo.rs
  - crates/speclink-core/src/discuss.rs
  - crates/speclink-core/src/instructions.rs
  - crates/speclink-core/src/newcmd.rs
  - crates/speclink-core/src/schema.rs
  - crates/speclink-core/src/util.rs
  - crates/speclink-core/src/workspace.rs
  - crates/speclink-core/tests/no_process_env.rs
  - crates/speclink-host/Cargo.toml
  - crates/speclink-host/src/binding.rs
  - crates/speclink-host/src/commit.rs
  - crates/speclink-host/src/context.rs
  - crates/speclink-host/src/gate.rs
  - crates/speclink-host/src/lib.rs
  - crates/speclink-host/src/policy.rs
  - crates/speclink-node/Cargo.toml
  - crates/speclink-node/src/lib.rs
  - crates/speclink-remote/Cargo.toml
  - crates/speclink-remote/src/auth.rs
-->

---
### Requirement: Engine 規格面不讀 process env 與 git identity

speclink-core 的非測試碼 SHALL NOT 直接讀取 process 環境變數或執行 git 身分查詢：工作流政策的環境變數層與 store 模式解析 SHALL 以 Host 注入的查找值運作，git 身分 SHALL 以 ExecutionContext 的 actor 供給。政策與模式的解析效果 SHALL 與現行完全一致（僅讀取位置改變）。

#### Scenario: 注入值決定政策而非 process env

- **WHEN** 以注入的環境覆寫集合（含 SPECLINK_TDD）呼叫政策解析，同時 process env 設定相反的值
- **THEN** 解析結果只反映注入集合；process env 的相反值無效果

#### Scenario: core 無殘留直讀

- **WHEN** 對 speclink-core 非測試碼執行 process env 讀取與 git config 呼叫的靜態盤點
- **THEN** 盤點結果為零命中（測試模組與 host crate 除外）


<!-- @trace
source: host-runtime-binding-policy
updated: 2026-07-13
code:
  - Cargo.lock
  - Cargo.toml
  - apps/desktop/core/Cargo.toml
  - apps/desktop/core/src/discussions.rs
  - apps/desktop/core/src/manage.rs
  - apps/desktop/core/src/query.rs
  - apps/desktop/core/src/settings.rs
  - apps/desktop/core/src/verbs.rs
  - crates/speclink-cli/Cargo.toml
  - crates/speclink-cli/src/commands.rs
  - crates/speclink-cli/src/remote_commands.rs
  - crates/speclink-core/src/archive.rs
  - crates/speclink-core/src/command/mod.rs
  - crates/speclink-core/src/config.rs
  - crates/speclink-core/src/demo.rs
  - crates/speclink-core/src/discuss.rs
  - crates/speclink-core/src/instructions.rs
  - crates/speclink-core/src/newcmd.rs
  - crates/speclink-core/src/schema.rs
  - crates/speclink-core/src/util.rs
  - crates/speclink-core/src/workspace.rs
  - crates/speclink-core/tests/no_process_env.rs
  - crates/speclink-host/Cargo.toml
  - crates/speclink-host/src/binding.rs
  - crates/speclink-host/src/commit.rs
  - crates/speclink-host/src/context.rs
  - crates/speclink-host/src/gate.rs
  - crates/speclink-host/src/lib.rs
  - crates/speclink-host/src/policy.rs
  - crates/speclink-node/Cargo.toml
  - crates/speclink-node/src/lib.rs
  - crates/speclink-remote/Cargo.toml
  - crates/speclink-remote/src/auth.rs
-->

---
### Requirement: Project 與 Repo binding 驗證 fail closed

binding 解析 SHALL 在缺失、無權限或存在多個候選時以帶原因的錯誤拒絕，SHALL NOT 自動選擇第一個候選。本地 fs 模式 SHALL 以 workspace root 映射固定 default project 與 repo，無需任何新設定且行為不變。ProjectId 與 RepoId SHALL 為不可變身分，key 僅為可讀名稱。

#### Scenario: 多義 binding 拒絕

- **WHEN** binding 解析面對兩個同時合格的候選 repo
- **THEN** 回帶「多個候選」原因的拒絕錯誤並列出候選，不自動選擇

#### Scenario: 本地 default binding 零設定

- **WHEN** 於無任何 remote 設定的本地 workspace 建立 ExecutionContext
- **THEN** binding 解析成功映射 default project/repo；全部現行本地動詞行為不變


<!-- @trace
source: host-runtime-binding-policy
updated: 2026-07-13
code:
  - Cargo.lock
  - Cargo.toml
  - apps/desktop/core/Cargo.toml
  - apps/desktop/core/src/discussions.rs
  - apps/desktop/core/src/manage.rs
  - apps/desktop/core/src/query.rs
  - apps/desktop/core/src/settings.rs
  - apps/desktop/core/src/verbs.rs
  - crates/speclink-cli/Cargo.toml
  - crates/speclink-cli/src/commands.rs
  - crates/speclink-cli/src/remote_commands.rs
  - crates/speclink-core/src/archive.rs
  - crates/speclink-core/src/command/mod.rs
  - crates/speclink-core/src/config.rs
  - crates/speclink-core/src/demo.rs
  - crates/speclink-core/src/discuss.rs
  - crates/speclink-core/src/instructions.rs
  - crates/speclink-core/src/newcmd.rs
  - crates/speclink-core/src/schema.rs
  - crates/speclink-core/src/util.rs
  - crates/speclink-core/src/workspace.rs
  - crates/speclink-core/tests/no_process_env.rs
  - crates/speclink-host/Cargo.toml
  - crates/speclink-host/src/binding.rs
  - crates/speclink-host/src/commit.rs
  - crates/speclink-host/src/context.rs
  - crates/speclink-host/src/gate.rs
  - crates/speclink-host/src/lib.rs
  - crates/speclink-host/src/policy.rs
  - crates/speclink-node/Cargo.toml
  - crates/speclink-node/src/lib.rs
  - crates/speclink-remote/Cargo.toml
  - crates/speclink-remote/src/auth.rs
-->

---
### Requirement: lifecycle gate 是單一裁決點

生命週期 SHALL 以封閉的六站狀態機表達：drafting、review、ready、applying、verified、archived；站間 transition SHALL 經 Host 的單一裁決函式判定，非法 transition SHALL 回帶原因的拒絕。本地模式 SHALL 提供唯讀站點推導（未開工＝drafting、已標記開工＝applying、已封存＝archived），且本刀 SHALL NOT 以 gate 裁決改變任何現行動詞的行為。

#### Scenario: 非法 transition 拒絕

- **WHEN** 對處於 drafting 的變更請求直接 transition 到 verified
- **THEN** 裁決函式回拒絕並指出缺少的中間站；合法路徑（drafting→review→ready→applying→verified→archived）逐步請求則全數允許

#### Scenario: 本地站點唯讀推導

- **WHEN** 對「未開工」「已標記開工」「已封存」三種本地變更狀態執行站點推導
- **THEN** 分別得到 drafting、applying、archived；推導不寫入任何檔案


<!-- @trace
source: host-runtime-binding-policy
updated: 2026-07-13
code:
  - Cargo.lock
  - Cargo.toml
  - apps/desktop/core/Cargo.toml
  - apps/desktop/core/src/discussions.rs
  - apps/desktop/core/src/manage.rs
  - apps/desktop/core/src/query.rs
  - apps/desktop/core/src/settings.rs
  - apps/desktop/core/src/verbs.rs
  - crates/speclink-cli/Cargo.toml
  - crates/speclink-cli/src/commands.rs
  - crates/speclink-cli/src/remote_commands.rs
  - crates/speclink-core/src/archive.rs
  - crates/speclink-core/src/command/mod.rs
  - crates/speclink-core/src/config.rs
  - crates/speclink-core/src/demo.rs
  - crates/speclink-core/src/discuss.rs
  - crates/speclink-core/src/instructions.rs
  - crates/speclink-core/src/newcmd.rs
  - crates/speclink-core/src/schema.rs
  - crates/speclink-core/src/util.rs
  - crates/speclink-core/src/workspace.rs
  - crates/speclink-core/tests/no_process_env.rs
  - crates/speclink-host/Cargo.toml
  - crates/speclink-host/src/binding.rs
  - crates/speclink-host/src/commit.rs
  - crates/speclink-host/src/context.rs
  - crates/speclink-host/src/gate.rs
  - crates/speclink-host/src/lib.rs
  - crates/speclink-host/src/policy.rs
  - crates/speclink-node/Cargo.toml
  - crates/speclink-node/src/lib.rs
  - crates/speclink-remote/Cargo.toml
  - crates/speclink-remote/src/auth.rs
-->

---
### Requirement: Host 承擔 TeamStore 的 UoW 與 event commit

Host SHALL 經 TeamStore 的轉接提供 commit 組合路徑：儲存接口的一次提交轉成一個 unit of work，提交裡的全部寫入與刪除放進同一個 unit of work，領域事件單向映射為 event records，經 commit 原子落入文件與 outbox；TeamStore 的 revision_conflict SHALL 轉成儲存接口的衝突並保留文件、expected 與 actual 的詳情。event record 的時間 SHALL 是該次提交的時間。這條 commit 路徑 SHALL NOT 接線任何本機模式的 CLI 流程。

#### Scenario: commit 後 outbox 含對應事件

- **WHEN** Host 經 TeamStore 的轉接對 in-memory reference store 提交一份文件的寫入並帶一筆領域事件
- **THEN** 提交成功後自 cursor 0 重讀 outbox 得到恰一筆對應 event record（含 actor 與事件名）；文件與事件同 commit 可見

#### Scenario: CAS 衝突原樣傳遞

- **WHEN** 兩次提交以相同的預期版本競寫同一文件
- **THEN** 敗方收到的 Host 錯誤是衝突，保留 revision_conflict 分類與 expected/actual 詳情，並指出是哪一份文件


<!-- @trace
source: engine-on-store-port
updated: 2026-11-11T12:52:06+08:00
-->

---
### Requirement: 組裝點遷移輸出凍結

CLI 與 Node dispatch 改經 Host 組裝後，全部現行動詞的人眼輸出、--json 輸出、exit code 與錯誤訊息 SHALL 與遷移前逐位元一致；政策環境變數與 git 身分的可觀測效果 SHALL 不變。

#### Scenario: baseline 對照逐位元一致

- **WHEN** 對同一樣本 workspace 於遷移前後執行覆蓋表動詞（人眼與 --json 兩形式，含設定 SPECLINK_TDD 與 git 身分的情境）
- **THEN** stdout、stderr 與 exit code 逐位元一致；`crates/adapters/speclink-cli/tests/` 的整合測試（含 `--no-color` 人眼輸出斷言與 fs／remote 對照）與 `crates/engine/speclink-core/tests/render_golden.rs` 全綠


<!-- @trace
source: stale-verification-vehicles
updated: 2026-07-27
code:
  - docs/implementation-refactor-roadmap.zh-TW.md
-->

---
### Requirement: engine 動詞經橋接於 TeamStore 上執行

Host SHALL 提供 engine-over-TeamStore 執行橋接：橋接 SHALL 把 TeamStore 的範圍經轉接包成儲存接口，再走「engine 動詞經儲存接口執行」的同一條執行路徑；Host SHALL NOT 另外保留一套把整個範圍載入記憶體的讀取視圖。同一動詞對語意相同的內容分別經本地 fs seam 與經橋接執行，typed outcome、錯誤分類與領域事件 SHALL 一致；TeamStore 的 revision_conflict SHALL 映射為命令層錯誤且保留 expected/actual 詳情。橋接 SHALL NOT 分叉 engine 命令層的動詞語意，發現的檔案系統暗依賴 SHALL 修在引擎的儲存視圖。

#### Scenario: 雙路徑 outcome 一致

- **WHEN** 對含相同 change 內容的 fs workspace 與 TeamStore scope 分別執行同一查詢動詞與同一變更型動詞
- **THEN** 兩路徑的 typed outcome 結構相等、變更型動詞回報相同種類的領域事件；失敗情境（如 not_found）的錯誤碼相同

#### Scenario: 橋接寫入原子落店

- **WHEN** 經橋接執行 task done 成功
- **THEN** 任務勾選後的文件內容、revision 遞增與 task-completed 事件記錄在同一 commit 內可見；commit 前 store 無任何中間狀態

#### Scenario: 版本衝突經 Server 回報

- **WHEN** 經橋接執行的變更型動詞在提交時遇到別人先寫入的同一份文件
- **THEN** Server 回應 HTTP 409，reason 是 revision_conflict，message 含該文件的鍵、expected 與 actual


<!-- @trace
source: engine-on-store-port
updated: 2026-11-11T12:52:06+08:00
-->

---
### Requirement: engine 動詞經儲存接口執行

Host SHALL 提供在任何儲存接口實作與一個範圍上執行一個 engine 命令的入口，回傳 typed outcome、領域事件，以及有提交時的新範圍版本號。engine 儲存介面的每一個操作——列出、是否存在、讀取、寫入、刪除、搬移——SHALL 只用儲存接口的列出、讀取、提交三個操作完成。一個動詞的全部寫入與刪除 SHALL 連同它的領域事件收成正好一次提交；同一次執行裡，動詞 SHALL 讀得到自己剛寫入的內容。每筆寫入與刪除的預期版本 SHALL 取自這次執行第一次看到該文件時的版本，文件當時不存在時預期是「必須不存在」。查詢，以及沒有任何寫入也沒有領域事件的動詞，SHALL NOT 提交。engine 命令層回報錯誤時，或讀取途中儲存回報失敗時，Host SHALL NOT 提交，儲存 SHALL 沒有任何變動；這兩種失敗 SHALL 以各自的分類回報，不合併。同一動詞對語意相同的內容分別經本地 fs seam 與經記憶體版的儲存接口執行，typed outcome、錯誤分類與領域事件種類 SHALL 一致。

#### Scenario: 寫入的動詞一次提交

- **WHEN** 經儲存接口執行 task done 成功
- **THEN** 儲存收到正好一次提交，內含任務勾選後的文件與 task-completed 事件；回傳的範圍版本號大於執行前的值

#### Scenario: 查詢不提交

- **WHEN** 經儲存接口執行列出變更
- **THEN** 儲存沒有收到任何提交，範圍版本號不變，回傳的範圍版本號是「沒有」

#### Scenario: 封存是一次提交裡的搬移

- **WHEN** 經儲存接口封存一個變更
- **THEN** 儲存收到正好一次提交：該變更的每份文件寫到已封存的鍵、進行中的鍵全部刪除、受影響的正式規格寫入；提交之後以 `changes/` 為前綴列出時不含該變更

#### Scenario: 別人先寫入時回報衝突且不留半套

- **WHEN** 動詞讀過一份文件之後、提交之前，另一方改寫了同一份文件
- **THEN** 執行回報儲存的衝突，指出鍵、預期與實際；這個動詞的其他寫入與領域事件都沒有進入儲存

#### Scenario: 命令層拒絕時儲存不變

- **WHEN** 經儲存接口對不存在的變更執行 task done
- **THEN** 執行回報命令層的 not_found，儲存沒有收到提交

#### Scenario: 讀取途中儲存失敗時不提交

- **WHEN** 儲存在動詞執行途中的一次讀取回報「暫時無法使用」
- **THEN** 執行回報儲存的失敗，不回報成命令層的錯誤，儲存沒有收到提交

#### Scenario: 兩條路徑結果一致

- **WHEN** 對含相同內容的 fs workspace 與記憶體版的儲存接口分別執行同一查詢動詞、同一寫入動詞、刪除變更、封存變更與封存討論
- **THEN** 兩路徑的 typed outcome 結構相等、領域事件種類相同；not_found 這類失敗的錯誤碼相同

<!-- @trace
source: engine-on-store-port
updated: 2026-11-11T12:52:06+08:00
-->

---
### Requirement: 經儲存接口執行時只讀取用到的文件

經儲存接口執行一個動詞時，Host SHALL 只向儲存要動詞實際用到的文件。同一次執行裡，同一個前綴 SHALL 最多列出一次，同一個鍵 SHALL 最多讀取一次。engine 儲存介面的單一操作需要多份文件時 SHALL 以一次讀取取得，SHALL NOT 逐份讀取。列出變更、查詢單一變更的狀態、列出進行中的討論（不帶封存旗標）這三類動詞 SHALL NOT 讀取任何鍵在 `archive/` 底下的文件內容，也 SHALL NOT 列出 `archive/changes/` 前綴。動詞覆寫一份這次執行沒看過的文件時，Host SHALL 先讀取該鍵以取得預期版本。

#### Scenario: 列出變更不碰封存資料

- **WHEN** 範圍裡有 3 個進行中的變更與 165 個已封存的變更，經儲存接口執行列出變更
- **THEN** 儲存收到的列出不含 `archive/changes/` 前綴，讀取的鍵沒有任何一個在 `archive/` 底下

#### Scenario: 多份 metadata 一次讀取

- **WHEN** 經儲存接口執行列出變更，範圍裡有 3 個進行中的變更
- **THEN** 3 份 metadata 由同一次讀取取得

#### Scenario: 同一個鍵不重複讀取

- **WHEN** 一個動詞在執行途中兩次需要同一份文件
- **THEN** 儲存只收到一次對該鍵的讀取

<!-- @trace
source: engine-on-store-port
updated: 2026-11-11T12:52:06+08:00
-->

---
### Requirement: TeamStore 轉接成儲存接口

Host SHALL 提供把 TeamStore 包成儲存接口的轉接，一個轉接 SHALL 能長期持有並服務多個範圍，讓示範用的 Server 繼續使用既有的 driver。轉接 SHALL 讓鍵與 TeamStore 的九種文件雙向對應，對應 SHALL 依儲存接口的鍵命名規則。範圍識別 SHALL 是 `<project>/<repo>` 的形式；project 裡的 `%` 與 `/` SHALL 寫成 `%25` 與 `%2F`，使一個範圍識別只對應一個範圍。轉接 SHALL 為每個用過的範圍保留最近一次載入的內容：下一次列出或讀取時，該範圍所屬專案的 revision 與載入當時相同 SHALL 重用它而 SHALL NOT 再次匯出；revision 不同 SHALL 重新載入。因此轉接在一次動詞執行裡 SHALL 最多匯出該範圍一次，而連續只查詢範圍版本號 SHALL NOT 造成任何匯出。轉接自己的提交成功後 SHALL 丟棄該範圍已載入的內容。同一個範圍的載入 SHALL 一次一個：等待中的呼叫在 revision 相同時 SHALL 重用剛完成的載入，SHALL NOT 另行匯出。直接經轉接（不經視圖）的每次呼叫，回傳的內容 SHALL NOT 比該次呼叫開始時 TeamStore 的 revision 舊。轉接 SHALL 另外提供一個請求用的視圖：視圖的第一次列出或讀取取得該範圍當下的載入，同一個請求之後的列出與讀取 SHALL 由這份載入回答；經視圖提交之後，視圖 SHALL 重新載入，因此讀得到自己寫入的內容。示範用 Server 的每個請求 SHALL 經這樣的視圖讀寫，使一個請求只載入一次範圍，命令內的讀取也來自同一個 revision。文件版本 SHALL 是 TeamStore 的文件 revision，範圍版本號 SHALL 是專案 revision。轉接回報的最後修改時間 SHALL 是 0。對不到任何一種文件的鍵：讀取 SHALL 回傳「沒有」，寫入 SHALL 回報後端錯誤並指出該鍵。範圍識別的形式不合時 SHALL 回報後端錯誤。

#### Scenario: 鍵與文件種類雙向對應

- **WHEN** 對九種文件各取一份，由文件種類換成鍵再換回
- **THEN** 得到原本的文件種類與名稱

##### Example: 對應

| TeamStore 的文件 | 鍵 |
| ---------------- | -- |
| 變更 `add-x` 的 metadata | `changes/add-x/.meta.yaml` |
| 變更 `add-x` 的 artifact `specs/auth/spec.md` | `changes/add-x/specs/auth/spec.md` |
| 變更 `add-x` 的 evidence | `changes/add-x/.evidence.json` |
| 已封存變更 `2026-11-10-add-x` 的 `proposal.md` | `archive/changes/2026-11-10-add-x/proposal.md` |
| 正式規格 `auth` | `specs/auth/spec.md` |
| 流程設定 | `config.yaml` |
| 詞彙表 | `LANGUAGE.md` |
| 看板順序 | `board/order` |

#### Scenario: 一次執行只匯出一次

- **WHEN** 經轉接執行一個會列出兩個前綴並讀取多份文件的動詞
- **THEN** TeamStore 的匯出只被呼叫一次

#### Scenario: 對不到文件種類的鍵

- **WHEN** 提交裡有一筆寫入的鍵不屬於九種文件的任何一種
- **THEN** 提交回報後端錯誤並指出該鍵，TeamStore 沒有任何變動

#### Scenario: 一個轉接服務兩個範圍

- **WHEN** 同一個轉接先後對範圍 `p1/main` 與 `p2/main` 各列出一次變更
- **THEN** 兩次各得到自己範圍的內容，互不混用

#### Scenario: revision 沒變時不再匯出

- **WHEN** 對同一個範圍連續兩次經轉接取得範圍版本號，中間沒有任何提交
- **THEN** 兩次得到相同的版本號；TeamStore 的匯出在第二次沒有被呼叫

#### Scenario: 別人提交後讀到新內容

- **WHEN** 轉接載入某個範圍之後，另一方直接對 TeamStore 提交了一份文件，之後再經轉接讀取該文件
- **THEN** 讀到的是新內容與新的文件版本

#### Scenario: 自己提交後讀到新內容

- **WHEN** 經轉接提交一筆寫入後，再經同一個轉接讀取該文件
- **THEN** 讀到剛寫入的內容，文件版本等於提交回傳的範圍版本號

#### Scenario: 一個請求只載入一次

- **WHEN** 一個請求經視圖列出兩個前綴並讀取一份文件，之後另一方直接對 TeamStore 提交了該文件，同一個請求再讀取它
- **THEN** TeamStore 的快照只被取一次；第二次讀取得到的仍是原本的內容與文件版本

#### Scenario: 經視圖提交後讀到自己寫的內容

- **WHEN** 經視圖提交一筆寫入後，再經同一個視圖讀取該文件
- **THEN** 讀到剛寫入的內容，文件版本等於提交回傳的範圍版本號

#### Scenario: 同時錯過同一份載入的兩個請求

- **WHEN** 轉接沒有該範圍的載入，兩個請求同時列出同一個範圍
- **THEN** TeamStore 的匯出只被呼叫一次，兩個請求都得到同樣的內容

#### Scenario: 含 `/` 的專案名稱

- **WHEN** 專案 `team/alpha` 的 repo `main` 與專案 `team` 的 repo `alpha/main` 各有一份不同的文件，經同一個轉接各列出一次
- **THEN** 範圍識別分別是 `team%2Falpha/main` 與 `team/alpha/main`，兩次各得到自己範圍的內容

#### Scenario: 範圍識別的形式不合

- **WHEN** 以不含 `/` 的範圍識別經轉接列出
- **THEN** 回報後端錯誤並指出範圍識別；TeamStore 沒有被呼叫


<!-- @trace
source: server-routes-on-store-port
updated: 2026-11-11T22:20:41+08:00
-->

---
### Requirement: 不經引擎命令的文件讀寫只用儲存接口

Host SHALL 為遠端用戶端需要、但不是引擎命令的文件讀寫提供函式，每個函式 SHALL 只用儲存接口的列出、讀取、提交三個操作，SHALL NOT 依賴任何 HTTP 框架，並 SHALL 能在任何儲存接口實作上執行。涵蓋：範圍版本號；流程設定的原文與 schema 名稱；看板順序的讀取與整份寫入；某個變更的 evidence；單份正式規格；已封存的清單、單份已封存文件與已封存變更的 capability 清單；搜尋；Context 快照；變更清單與單筆變更回應補充的欄位；spec drift。需要回報範圍版本號的函式 SHALL 先取版本再讀內容，使回報的版本 SHALL NOT 比內容新。讀取一批文件 SHALL 以一次帶多個鍵的讀取完成。這些函式 SHALL NOT 匯出整個範圍來取得其中一部分：只需要某個前綴底下的鍵時 SHALL 只列出該前綴。示範用 Server 的對應路由 SHALL 呼叫這些函式；它的專案路由中，直接呼叫 TeamStore 的 SHALL 只剩匯入與推送事件的串流。改用這些函式後，各路由的狀態碼、回應本文與 ETag 標頭 SHALL 維持既有的位元級輸出，例外只在儲存失敗時：兩個請求幾乎同時、比對之後提交之前別人改了同一份文件時，409 的訊息文字與儲存回報後端錯誤時 500 的訊息文字改用儲存接口的格式；TeamStore 回報找不到或無權限時改回 500，因為儲存接口只有衝突、暫時無法使用與後端錯誤三種失敗。

看板順序的整份寫入 SHALL：內容超過 1 MiB 時拒絕；請求帶的預期範圍版本與當下不同時以版本衝突拒絕；以一次提交寫入，文件存在時預期它的文件版本、不存在時預期「必須不存在」，並在同一次提交帶一筆 board-order-updated 事件。任何一種拒絕發生時儲存 SHALL 沒有變動。

#### Scenario: 流程設定的讀取只讀一份文件

- **WHEN** 在一個有 165 個已封存變更的範圍上讀取流程設定
- **THEN** 回傳原文、schema 名稱與範圍版本號；儲存收到的讀取只含 `config.yaml` 一個鍵，沒有列出 `archive/` 底下的任何前綴

#### Scenario: 單筆變更的 delta capability 只列出該變更的前綴

- **WHEN** 取得變更 add-x 的 delta capability 清單
- **THEN** 儲存收到一次前綴為 `changes/add-x/specs/` 的列出，沒有讀取任何文件內容

#### Scenario: 已封存清單一次讀取

- **WHEN** 在有三個已封存變更的範圍上取得已封存清單
- **THEN** 儲存收到一次前綴為 `archive/changes/` 的列出與一次讀取（含各變更的任務清單與 metadata）；清單依名稱由新到舊，各項含任務總數、已完成數與規格數

#### Scenario: 不存在的正式規格

- **WHEN** 讀取不存在的 capability 的正式規格
- **THEN** 回報找不到並指出該 capability

#### Scenario: 壞的流程設定

- **WHEN** 範圍的流程設定無法解析，讀取流程設定
- **THEN** 回報無效的設定，不回傳部分內容

#### Scenario: 壞的 evidence

- **WHEN** 某個變更的 evidence 文件不是合法的內容，讀取它
- **THEN** 回報內容損壞；不回傳空的清單

#### Scenario: 看板順序寫入成功

- **WHEN** 範圍版本號是 7、看板順序文件不存在，以預期版本 7 寫入一份 200 位元組的內容
- **THEN** 儲存收到正好一次提交：一筆對 `board/order` 的寫入（預期是「必須不存在」）與一筆 board-order-updated 事件；回傳的範圍版本號大於 7

#### Scenario: 看板順序的預期版本過期

- **WHEN** 範圍版本號是 8，以預期版本 7 寫入看板順序
- **THEN** 回報版本衝突；儲存沒有收到提交

#### Scenario: 看板順序內容過大

- **WHEN** 寫入超過 1 MiB 的看板順序內容
- **THEN** 回報內容過大；儲存沒有收到任何呼叫

#### Scenario: Context 快照沒有變時不讀內容

- **WHEN** 取得 Context 快照時帶上等於當下範圍版本號的識別
- **THEN** 回報「沒有變」；儲存只收到一次列出，沒有收到讀取

#### Scenario: Context 快照指定不存在的變更

- **WHEN** 取得 Context 快照並指定不存在的變更
- **THEN** 回報找不到

#### Scenario: 回報的版本不比內容新

- **WHEN** 讀取流程設定的過程中，在取得版本之後、讀取內容之前，另一方提交了新的流程設定
- **THEN** 回傳的內容是新的，回傳的範圍版本號是舊的；用戶端以該版本號輪詢時得知範圍已變動

#### Scenario: 改寫前後線上回應相同

- **WHEN** 對同一批資料，分別在改寫前與改寫後的示範用 Server 請求流程設定、看板順序、evidence、單份正式規格、已封存清單、單份已封存文件、搜尋、Context 快照與 drift
- **THEN** 每一對回應的狀態碼、本文與 ETag 標頭逐位元相同

#### Scenario: 儲存暫時無法使用

- **WHEN** 儲存在讀取流程設定時回報暫時無法使用
- **THEN** 函式回報儲存的「暫時無法使用」；示範用 Server 回應既有的 unavailable

<!-- @trace
source: server-routes-on-store-port
updated: 2026-11-11T22:20:41+08:00
-->

---
### Requirement: 沒有 workspace 時封存不寫入本機檔案

引擎的封存在執行情境沒有 workspace 時 SHALL NOT 讀寫程序的檔案系統：SHALL NOT 寫入正式規格的本機備份與其清單檔，SHALL NOT 移除本機的開工標記檔；封存結果的「已建立備份」SHALL 為 false。封存的其餘效果——驗證、合併 delta、寫入正式規格、搬移變更的文件、一併封存來源討論——SHALL 與有 workspace 時相同，並 SHALL 全部經儲存介面完成。有 workspace 的封存（本機專案與 Node SDK 的 fs 形式）SHALL 維持既有行為：備份照寫，既有的位元級輸出不變。

#### Scenario: 經 Host 封存不在程序目錄寫檔

- **WHEN** 程序的工作目錄是一個空的暫存資料夾，經儲存接口封存一個會改寫既有正式規格的變更
- **THEN** 封存成功，結果的「已建立備份」是 false；該暫存資料夾仍然是空的；儲存收到正好一次提交

#### Scenario: 遠端封存的回應

- **WHEN** 對示範用 Server 送出封存請求
- **THEN** 回應的 snapshotCreated 是 false，其餘欄位（datedName、各 capability 的計數、archivedDiscussions、evidenceRecorded）與既有行為相同

#### Scenario: 本機封存照舊寫備份

- **WHEN** 在本機專案執行 speclink archive，該變更改寫一份既有的正式規格
- **THEN** `.speclink/snapshots/<封存名>/specs/<capability>/spec.md` 含改寫前的內容；stdout 與既有行為逐位元相同

<!-- @trace
source: server-routes-on-store-port
updated: 2026-11-11T22:20:41+08:00
-->
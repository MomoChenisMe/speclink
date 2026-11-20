# host-runtime Specification

## Purpose

應用層 Host 作為引擎唯一邊界的語意：ExecutionContext 由 Host 解析且不可被呼叫端覆寫、Project 與 Repo binding 的驗證、lifecycle gate 作為單一裁決點，以及引擎命令經儲存接口執行、一個動詞收成一次提交的歸屬。本 capability 保證引擎規格面不讀 process 環境變數、不讀 git 身分——這些一律由 Host 注入，且組裝點遷移不改變任何對外輸出。

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
### Requirement: 不經引擎命令的文件讀寫只用儲存接口

Host SHALL 為遠端用戶端需要、但不是引擎命令的文件讀寫提供函式，每個函式 SHALL 只用儲存接口的列出、讀取、提交三個操作，SHALL NOT 依賴任何 HTTP 框架，並 SHALL 能在任何儲存接口實作上執行。涵蓋：範圍版本號；流程設定的原文與 schema 名稱；看板順序的讀取與整份寫入；某個變更的 evidence；單份正式規格；已封存的清單、單份已封存文件與已封存變更的 capability 清單；搜尋；Context 快照；變更清單與單筆變更回應補充的欄位；spec drift。需要回報範圍版本號的函式 SHALL 先取版本再讀內容，使回報的版本 SHALL NOT 比內容新。讀取一批文件 SHALL 以一次帶多個鍵的讀取完成。這些函式 SHALL NOT 匯出整個範圍來取得其中一部分：只需要某個前綴底下的鍵時 SHALL 只列出該前綴。示範用 Server 的對應路由 SHALL 呼叫這些函式；它的專案路由 SHALL NOT 經儲存接口以外的途徑讀寫文件；匯入 SHALL 由宿主套件以儲存接口完成。改用這些函式後，各路由的狀態碼、回應本文與 ETag 標頭 SHALL 維持既有的位元級輸出，例外只在儲存失敗時：兩個請求幾乎同時、比對之後提交之前別人改了同一份文件時，409 的訊息文字與儲存回報後端錯誤時 500 的訊息文字改用儲存接口的格式。儲存接口只有衝突、暫時無法使用與後端錯誤三種失敗；儲存回報後端錯誤時示範用 Server SHALL 回 500。

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

#### Scenario: 示範用 Server 的匯入經儲存接口的一次提交

- **WHEN** 對示範用 Server 送出匯入請求並成功
- **THEN** 全部文件經儲存接口的一次提交寫入；儲存沒有收到其他寫入的呼叫


<!-- @trace
source: retire-teamstore-and-drivers
updated: 2026-11-17T22:53:25+08:00
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

---
### Requirement: 專案與儲存庫對映成範圍識別字

Host SHALL 提供把一組專案與儲存庫對映成儲存接口範圍識別字的函式，以及它的反向。範圍識別字 SHALL 是 `<project>/<repo>` 的形式；project 裡的 `%` 與 `/` SHALL 寫成 `%25` 與 `%2F`，使一個範圍識別字只對應一組專案與儲存庫。專案與儲存庫的名稱組不成合格的範圍識別字（例如超過範圍識別字的長度上限）時，函式 SHALL 回報錯誤。反向的函式遇到形式不合的範圍識別字（沒有 `/`、任一段是空的、project 含 `%25` 與 `%2F` 以外的跳脫）SHALL 回報錯誤並指出該範圍識別字，SHALL NOT 猜測。示範用 Server SHALL 用對映的函式（由專案與儲存庫得到範圍識別字）決定每個請求讀寫的範圍。

#### Scenario: 含斜線的專案名稱

- **WHEN** 把專案 `team/alpha` 的儲存庫 `main` 與專案 `team` 的儲存庫 `alpha/main` 各自對映成範圍識別字
- **THEN** 得到 `team%2Falpha/main` 與 `team/alpha/main`，兩者不同；各自經反向的函式得回原本的專案與儲存庫

#### Scenario: 含百分號的專案名稱

- **WHEN** 把專案 `50%` 的儲存庫 `main` 對映成範圍識別字再反向
- **THEN** 範圍識別字是 `50%25/main`；反向得回專案 `50%` 與儲存庫 `main`

#### Scenario: 範圍識別字的形式不合

- **WHEN** 以不含 `/` 的範圍識別字呼叫反向的函式
- **THEN** 回報錯誤並指出該範圍識別字

#### Scenario: 兩個儲存庫各自的範圍

- **WHEN** 經示範用 Server 對同一個專案的兩個儲存庫各寫入一份文件
- **THEN** 兩份文件落在兩個不同的範圍；讀其中一個儲存庫看不到另一個的文件

<!-- @trace
source: retire-teamstore-and-drivers
updated: 2026-11-17T22:53:25+08:00
-->

---
### Requirement: 文件種類與鍵的對應

Host SHALL 提供把第一版的包用的九種文件種類換成儲存接口的鍵的對應，以及它的反向，對應 SHALL 依儲存接口的鍵命名規則。這份對應 SHALL 是第一版匯入包與第一版備份檔唯一的轉換來源。換不成鍵的文件種類 SHALL 回報「沒有對應」，呼叫端據此拒絕整個包，SHALL NOT 略過該文件。

#### Scenario: 文件種類換成鍵再換回

- **WHEN** 對九種文件各取一份，由文件種類換成鍵再換回
- **THEN** 得到原本的文件種類與名稱

##### Example: 對應

| 文件種類 | 鍵 |
| -------- | -- |
| 變更 `add-x` 的 metadata | `changes/add-x/.meta.yaml` |
| 變更 `add-x` 的 artifact `specs/auth/spec.md` | `changes/add-x/specs/auth/spec.md` |
| 變更 `add-x` 的 evidence | `changes/add-x/.evidence.json` |
| 已封存變更 `2026-11-10-add-x` 的 `proposal.md` | `archive/changes/2026-11-10-add-x/proposal.md` |
| 正式規格 `auth` | `specs/auth/spec.md` |
| 流程設定 | `config.yaml` |
| 詞彙表 | `LANGUAGE.md` |
| 看板順序 | `board/order` |

#### Scenario: 不認得的文件種類

- **WHEN** 第一版的包裡有一份文件換不成鍵（名稱不合鍵的命名規則，或種類不在九種之內），匯入這個包
- **THEN** 匯入的呼叫以「不合格的包」結束並指出該文件，範圍沒有被寫入；名稱不合命名規則的文件換成鍵時回報沒有對應

<!-- @trace
source: retire-teamstore-and-drivers
updated: 2026-11-17T22:53:25+08:00
-->

---
### Requirement: 經儲存接口寫入時回報儲存的鍵

引擎經儲存接口執行 `new artifact` 時，結果的 `path` SHALL 是那份文件在儲存裡的鍵：`changes/<變更名稱>/<artifact 的路徑>`，以 `/` 分隔，開頭沒有規格資料夾的名稱。這個值 SHALL 與儲存的 commit 函式在同一次呼叫收到的那一筆寫入的鍵逐字相同，也 SHALL 與同一個宿主裡 `discuss new` 回報的 `path` 同一種形式（鍵）。這是刻意的變更：變更前這個值是 `speclink/changes/<變更名稱>/<artifact 的路徑>`。

引擎經儲存接口執行 `discuss promote --json` 時，結果的 `path` SHALL 是新變更的鍵的共同開頭 `changes/<變更名稱>`，以 `/` 分隔，開頭沒有規格資料夾的名稱。這也是刻意的變更：變更前這個值是 `speclink/changes/<變更名稱>`。

本需求 SHALL NOT 改變下列輸出，它們維持既有的位元級輸出：CLI 在本機模式的 `new artifact`（`path` 是本機檔案的路徑）、CLI 在 remote 模式的 `new artifact`、Node SDK 的 createEngine（本機資料夾）的 `new artifact`、經儲存接口執行時「Artifact already exists: …」這則拒絕訊息的文字、內嵌宿主的人眼輸出（依「內嵌宿主的人眼輸出不印路徑行」不印路徑），以及經儲存接口執行時 `instructions --json` 的 `changeDir` 與 `contextFiles`（它們由每個變更的資料夾組出，仍以 `speclink/changes/` 開頭）。

#### Scenario: 內嵌宿主的 new artifact 回報鍵

- **WHEN** 宿主經 host.dispatch 執行 `['new', 'artifact', 'proposal', '--change', 'demo', '--stdin', '--json']`，stdin 是一份合法的 proposal
- **THEN** 回傳物件的 `path` 是 `changes/demo/proposal.md`；儲存的 commit 函式收到一筆寫入，它的鍵也是 `changes/demo/proposal.md`

#### Scenario: delta spec 的路徑也是鍵

- **WHEN** 宿主經 host.dispatch 執行 `['new', 'artifact', 'spec', 'user-auth', '--change', 'demo', '--stdin', '--json']`，`user-auth` 是正式規格已有的 capability
- **THEN** 回傳物件的 `path` 是 `changes/demo/specs/user-auth/spec.md`

#### Scenario: 內嵌宿主的 discuss promote 回報鍵的開頭

- **WHEN** 討論 topic-one 已有結論，宿主經 host.dispatch 執行 `['discuss', 'promote', 'topic-one', '--name', 'next-one', '--json']`
- **THEN** 回傳物件的 `path` 是 `changes/next-one`；儲存的 commit 函式收到的寫入裡有鍵 `changes/next-one/.meta.yaml`

#### Scenario: 本機資料夾的輸出不變

- **WHEN** 以 createEngine 對本機資料夾執行同一個 `new artifact … --json`
- **THEN** 回傳物件的 `path` 與引入本需求之前相同，是本機檔案的路徑，結尾是 `speclink/changes/demo/proposal.md`

#### Scenario: 已存在時的拒絕訊息不變

- **WHEN** 宿主經 host.dispatch 對已經有 proposal 的變更 demo 再執行一次 `new artifact proposal --change demo --stdin --json`，沒有 `--force`
- **THEN** 呼叫 reject，Error 的 `message` 與引入本需求之前逐字相同，仍以 `Artifact already exists:` 開頭；儲存的 commit 函式沒有被呼叫

<!-- @trace
source: sdk-command-names-and-transfer-reasons
updated: 2026-11-20T15:39:51+08:00
-->
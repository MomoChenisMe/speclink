## Context

在這個變更開始時（前兩個變更已完成），舊的儲存層剩下這些地方還在用：

| 位置 | 用到什麼 | 用途 |
| --- | --- | --- |
| 宿主套件 `teamstore_port.rs` | 整個 TeamStore 介面 | 把 TeamStore 包成儲存接口的轉接（正式程式約 500 行，測試 30 個）。沒有正式的呼叫端 |
| 宿主套件 `bridge.rs` | TeamStore、範圍、版本 | 在 TeamStore 上執行一個引擎命令。只有測試呼叫 |
| 宿主套件 `binding.rs`、`context.rs`、`http/mod.rs`、`scope.rs` | 專案與儲存庫的識別型別 | 公開結構的欄位型別 |
| 宿主套件 `projection.rs`、`documents.rs`、`transfer.rs` | 內容摘要的函式 | `projection` 另外把它轉出，CLI 的一個測試從那裡引用 |
| 宿主套件 `drift.rs`、`projection.rs` 的測試 | 舊的記憶體儲存 | 經轉接塞測試資料 |
| 宿主套件 tests/bridge_dual_path.rs | 舊的記憶體儲存與多個型別 | 「本地檔案與 TeamStore 兩條路徑結果相同」的 6 個測試 |
| Speclink Desktop `remote.rs` | 內容摘要的函式、第一版包的版本常數 | 組匯入包 |
| `speclink-remote` 與 Speclink Desktop 的測試 | 少數殘留的引用 | 前一個變更改過建構方式之後留下的 |

另外有四個 crate 本身、四份正式規格、流程設定的專案說明，以及散在引擎、協定與 server 的幾處註解與套件說明。

引擎的封存沒有「刪除一整個 capability」的操作：delta 把一份規格的需求全部移除之後，會留下一份只有 Purpose 的空規格檔。

## Goals / Non-Goals

**Goals:**

- workspace 裡沒有 `speclink-store` 與三個舊 driver，也沒有任何程式引用它們。
- 仍然需要的三樣東西（識別型別、內容摘要、第一版的版本常數）由宿主套件提供，對外的名稱與行為不變。
- 被刪除的測試所保護的行為，只要還存在，就有對應的測試留著。
- 流程設定的專案說明與正式規格描述現在的架構。

**Non-Goals:**

- 不改任何對外的行為。
- 不改寫使用者文件（只修會讓守門失敗的連結）。
- 不動 LANGUAGE.md、手冊、已封存的變更與討論記錄。

範圍內：四個 crate 的刪除、`speclink-host`、`speclink-remote`、Speclink Desktop 的 Tauri 殼、示範用 server 呼叫範圍識別字函式的五處寫法、幾處註解、CI、`speclink/config.yaml`、正式規格、docs/product-status 兩個檔的連結。範圍外：其餘全部。

## Decisions

### D1: 還在用的型別與函式搬進宿主套件

- 專案與儲存庫的識別型別（`ProjectId`、`RepoId`）：定義搬到宿主套件的 `scope.rs`（前一個變更建立，放範圍識別字的函式）。建構方式與方法不變。前一個變更加的轉出改成直接匯出定義。
- TeamStore 的範圍型別（專案加儲存庫的一對）不搬：`scope.rs` 的兩個函式改成直接收專案與儲存庫，回傳範圍識別字；反向回傳專案與儲存庫的一對。反向的函式今天在轉接的檔案裡，搬到 `scope.rs`；轉接刪除之後它沒有正式的呼叫端（示範用 server 只用到對映的那一個方向），依規格保留給手上只有範圍識別字、要還原專案與儲存庫的宿主。示範用 server 有五個檔案用這個範圍型別組範圍識別字（`state.rs`、`admin.rs`、`admin/backup.rs`、`api/routes.rs`、測試的 `common/mod.rs`），改成直接傳專案與儲存庫。
- 內容摘要的函式：定義搬到宿主套件的 `projection.rs`——它今天已經從那裡轉出，CLI 的測試與宿主套件自己的測試都從這個位置引用，所以對外的路徑不變。演算法不變：SHA-256，小寫十六進位，前綴 `sha256:`。宿主套件因此直接相依 `sha2`。
- 第一版包的版本常數：宿主套件 `transfer` 裡既有的常數改成公開，Speclink Desktop 改用它。
- 「文件種類與鍵的對應」反向的函式今天只在套件內可見，改成公開，讓套件外的測試與宿主都能用同一對函式。
- 宿主套件的 Cargo.toml 移除 `speclink-store`。

替代方案：把這幾樣放進儲存契約的 crate。不採用——儲存契約不知道「專案與儲存庫」這個概念，它只認範圍識別字；內容摘要是匯入匯出與投影用的，也不是契約的一部分。

### D2: 刪除轉接、橋接與它們的測試

刪除 `teamstore_port.rs`、`bridge.rs` 與 tests/bridge_dual_path.rs。刪之前先處理測試：

- **轉接的 30 個測試**：測範圍識別字的（跳脫、反向、形式不合）搬到 `scope.rs`；測「文件種類與鍵的對應」的，`transfer` 已有同樣的測試就刪，沒有就搬過去；其餘都是轉接特有的行為（重用載入、請求用的視圖、版本取 TeamStore 的 revision），刪除。
- **「兩條路徑」的 6 個測試**：逐一對照宿主套件既有的「本地檔案與儲存接口兩條路徑結果一致」測試（規格 host-runtime「engine 動詞經儲存接口執行」的情境「兩條路徑結果一致」）。已經有同樣斷言的刪除；沒有的改寫成在記憶體版的儲存接口上跑，放進既有的那個測試檔。
- **`drift.rs` 與 `projection.rs` 裡經轉接塞資料的測試**：改成直接對記憶體版的儲存接口以鍵寫入。斷言不變。

任務的完成說明要列出這 36 個測試各自的去向：搬到哪裡、由哪個既有測試涵蓋、或因為是轉接特有的行為而刪除。

#### 測試去向表（任務 2.1 的完成說明）

轉接的 30 個測試（`teamstore_port.rs`）：

| # | 測試 | 去向 |
| --- | --- | --- |
| 1 | `the_nine_document_kinds_map_to_keys_and_back` | 搬到 tests/transfer.rs 的 `each_of_the_nine_document_kinds_goes_to_its_key_and_back`（同樣的 11 列） |
| 2 | `a_document_whose_name_cannot_be_a_key_has_no_key` | 併進 tests/transfer.rs 既有的 `a_first_version_document_that_has_no_key_is_refused_by_name`（補上空名稱、`archive`、capability `a/b` 三列）；「範圍沒有被寫入」由 tests/http_door.rs 的 `a_bundle_that_is_not_valid_is_400_and_names_its_key`（名稱換不成鍵）與 `a_body_that_is_not_a_bundle_keeps_the_statuses_the_import_always_gave`（種類不在九種之內）經 HTTP 入口證明 |
| 3 | `the_scope_id_is_the_project_and_the_repo` | 搬到 `scope.rs` 的 `a_project_and_a_repo_make_one_scope_id_and_it_reads_back`（`default/main` 那一列） |
| 4 | `listing_gives_document_revisions_the_project_revision_and_time_zero` | 刪除：轉接特有（時間回報 0、版本取 TeamStore 的 revision） |
| 5 | `one_execution_exports_the_scope_once` | 刪除：轉接特有（重用載入） |
| 6 | `one_adapter_serves_two_scopes_without_mixing_them` | 刪除：轉接特有（每個範圍各自的載入）；範圍互不相見由儲存契約的合格測試涵蓋 |
| 7 | `an_unchanged_revision_answers_without_another_export` | 刪除：轉接特有（重用載入） |
| 8 | `a_commit_made_straight_to_the_teamstore_is_read_back` | 刪除：轉接特有（載入過期的判斷） |
| 9 | `its_own_commit_is_read_back` | 刪除：轉接特有（載入過期的判斷） |
| 10 | `a_scope_id_without_a_slash_is_a_backend_error_before_the_teamstore_is_called` | 形式不合的部分搬到 `scope.rs` 的 `a_scope_id_of_the_wrong_form_is_an_error_that_names_it`；「TeamStore 沒有被呼叫」是轉接特有，刪除 |
| 11 | `a_scope_lists_only_its_own_documents` | 刪除：轉接特有；範圍互不相見由儲存契約的合格測試涵蓋 |
| 12 | `a_scope_id_names_exactly_one_teamstore_scope` | 搬到 `scope.rs` 的 `a_project_and_a_repo_make_one_scope_id_and_it_reads_back` 與 `a_scope_id_of_the_wrong_form_is_an_error_that_names_it`（每一列都在） |
| 13 | `two_scopes_that_join_to_the_same_text_stay_apart` | 搬到 `scope.rs` 的 `a_project_and_a_repo_make_one_scope_id_and_it_reads_back`（最後一個斷言：兩個範圍識別字不同）；經 server 的證明由示範用 server 既有的測試涵蓋（規格情境「兩個儲存庫各自的範圍」） |
| 14 | `a_pinned_view_answers_one_request_from_one_load` | 刪除：轉接特有（請求用的視圖） |
| 15 | `a_pinned_view_keeps_its_fixed_point_while_another_writer_commits` | 刪除：轉接特有（請求用的視圖） |
| 16 | `a_pinned_view_reads_back_its_own_commit` | 刪除：轉接特有（請求用的視圖） |
| 17 | `a_thread_pin_answers_the_threads_reads_from_one_load_per_scope` | 刪除：轉接特有（請求用的視圖） |
| 18 | `a_thread_pin_reads_back_its_own_commit` | 刪除：轉接特有（請求用的視圖） |
| 19 | `a_thread_pin_leaves_other_threads_alone` | 刪除：轉接特有（請求用的視圖） |
| 20 | `a_pinned_view_serves_its_own_scope_only` | 刪除：轉接特有（請求用的視圖） |
| 21 | `an_older_load_never_replaces_a_newer_one` | 刪除：轉接特有（重用載入） |
| 22 | `two_requests_that_miss_the_same_load_export_once` | 刪除：轉接特有（重用載入） |
| 23 | `the_adapter_is_send_and_sync_and_can_be_shared` | 刪除：轉接特有 |
| 24 | `a_commit_lands_the_document_and_its_event_together` | 刪除：轉成 TeamStore 的 unit of work 是轉接特有；「事件與寫入同一次提交」由儲存契約的合格測試與 `port_exec.rs` 的 `task_done_is_exactly_one_commit_with_its_event` 涵蓋 |
| 25 | `a_commit_with_events_only_lands_the_events` | 刪除：同上；只有事件的提交由儲存契約的合格測試涵蓋 |
| 26 | `a_verb_run_through_the_bridge_exports_the_scope_once` | 刪除：轉接與橋接特有 |
| 27 | `a_commit_is_one_unit_of_work_with_writes_and_deletes` | 刪除：轉成 unit of work 是轉接特有；整筆提交由儲存契約的合格測試（`conflict-rejects-whole-commit`）涵蓋 |
| 28 | `the_loser_of_two_commits_on_one_document_gets_the_conflict` | 刪除：錯誤的轉換是轉接特有；衝突由儲存契約的合格測試與 `port_exec.rs` 的 `a_concurrent_write_is_a_conflict_and_nothing_lands` 涵蓋 |
| 29 | `a_key_of_no_document_kind_reads_as_nothing_and_cannot_be_written` | 「鍵對不到文件種類」搬到 tests/transfer.rs 的 `a_key_that_follows_no_naming_rule_names_no_document_kind`；「讀不到、寫不進」是轉接特有（真正的儲存接口任何鍵都能存），刪除 |
| 30 | `an_unavailable_store_stays_a_temporary_failure` | 刪除：錯誤的轉換是轉接特有；「暫時無法使用」照原樣回報由 `port_exec.rs` 的 `a_store_failure_at_any_call_is_a_store_error_and_writes_nothing` 涵蓋 |

「兩條路徑」的 6 個測試（tests/bridge_dual_path.rs）。對照的既有測試都在 tests/port_dual_path.rs 與 `port_exec.rs`，每一個都已經有同樣的斷言，所以沒有需要改寫的：

| # | 測試 | 去向 |
| --- | --- | --- |
| 31 | `query_verbs_yield_the_same_outcome_on_both_paths` | 刪除：由 `list_changes_matches_and_commits_nothing` 與 `status_of_one_change_matches` 涵蓋 |
| 32 | `mutating_verb_yields_the_same_outcome_and_event_kind_on_both_paths` | 刪除：由 `task_done_matches_and_records_the_same_evidence` 涵蓋 |
| 33 | `bridged_task_done_lands_document_revision_and_event_in_one_commit` | 刪除：由 `port_exec.rs` 的 `task_done_is_exactly_one_commit_with_its_event` 涵蓋（勾選後的任務清單、evidence、事件都在同一次提交，版本號等於提交後的範圍版本） |
| 34 | `bridged_workflow_config_replace_lands_the_document_at_the_commit_revision` | 刪除：由 `port_exec.rs` 的 `a_workflow_config_replace_creates_a_missing_document` 與 `a_workflow_config_write_that_changes_nothing_commits_nothing` 涵蓋 |
| 35 | `not_found_failure_carries_the_same_error_code_on_both_paths` | 刪除：由 `not_found_carries_the_same_error_code` 涵蓋 |
| 36 | `language_show_reads_the_shared_vocabulary_on_both_paths` | 刪除：由 `shared_vocabulary_matches` 涵蓋 |

`drift.rs` 經轉接塞資料的兩個測試（不算在 36 個裡）：`spec_drift_reports_a_store_failure_as_a_store_failure` 改成在記憶體版的儲存接口上跑，並對每一次儲存呼叫各失敗一次；`spec_drift_gives_the_same_answer_on_a_memory_store_and_on_the_teamstore_adapter` 改名為 `spec_drift_reads_the_change_and_the_canonical_spec_of_a_store_port`，留下對記憶體版的兩個斷言，「和轉接的結果相同」那一個隨轉接消失。`projection.rs` 沒有經轉接塞資料的測試。

### D3: 刪除四個 crate

順序：先做完 D1 與 D2，確認 workspace 裡沒有任何 Cargo.toml 還列著這四個 crate，再刪除四個資料夾並從根目錄 Cargo.toml 的成員清單移除。

Cargo.lock 只會減少：rusqlite、postgres、fs4 仍然被三個範例使用，`sha2` 與 `chrono` 仍然被其他 crate 使用。

`speclink-remote` 的測試用相依 `speclink-store` 與 Speclink Desktop 測試裡的引用，前一個變更（server-on-port-examples）已經移除；實作時確認沒有殘留，這幾個檔不用改。Speclink Desktop 的相依 `speclink-store` 移除，兩處引用改成宿主套件的 `projection::content_digest` 與 `transfer` 的第一版常數。

### D4: 正式規格

**直接刪除四份**：teamstore-contract、sqlite-team-store、serverfs-team-store、postgres-team-store 的資料夾。不寫 delta：把需求全部移除的 delta 在封存後會留下四份空的規格檔，而引擎沒有刪除 capability 的操作。刪除的理由是：這四份規定的介面與實作已經不存在；仍然成立的保證由 store-port-contract（契約）與 store-port-examples（三個範例）承接。

刪除前先搜尋其他正式規格、scripts、crates 與 apps 裡提到這四個名稱的地方。目前已知兩處：示範用 server 兩個測試檔開頭的註解（`postgres_store.rs`、`serverfs_store.rs`），改成指向 store-port-examples。

**host-runtime 的 Purpose**：delta 不更新既有 capability 的 Purpose，所以直接改正式規格檔的那一段（只改這一段），把「以及 TeamStore 的 unit of work 與事件提交歸屬」換成「以及引擎命令經儲存接口執行、一個動詞收成一次提交的歸屬」。

**其餘用 delta**：host-runtime、server-read-api、server-identity、change-diff-scope，內容見各自的 delta。

### D5: 流程設定的專案說明

用流程設定的動詞改 `speclink/config.yaml`，不手改檔案。

context 改三段，其餘原樣保留：

- 第一段的最後一句改成：「同時支援本機專案（`speclink/` ＋ Git、可完全離線）與遠端專案（規格由一台 Server 持有：內嵌了引擎的宿主，或官方的示範用 server）兩條路徑。」
- 技術棧那一段的「store driver 為 rusqlite／postgres／檔案系統」改成「儲存接口的三個範例用 rusqlite／postgres／檔案系統」。
- Cargo workspace 那一段裡 `speclink-store` 與三個 driver 的說明改成：「`speclink-store-port`（儲存契約：列出、讀取、提交三個操作，合格測試與記憶體版參考實作）＋ `speclink-store-port-{sqlite,fs,postgres}`（三個範例，也是示範用 server 的儲存）」；`speclink-host` 的說明改成「引擎唯一的應用層邊界與宿主套件：execution context、命令經儲存接口執行、掛鉤、HTTP 入口、匯入匯出」；`speclink-server` 的說明加上「示範用」。

`rules.design` 第一條的「TeamStore contract 與 driver 歸 store crates」改成「儲存契約與範例歸 store crates」，其餘文字不變。

寫回時的注意事項：`workflow-config show --json` 取得的 context 少結尾的換行，原樣寫回會讓 YAML 的區塊標記改變、整段被重寫。寫回前補上結尾的換行，先用 `--dry-run` 看差異只有上面列的幾行，再真的寫入。`rules design` 是整個區段取代，要把全部規則一起送回。

### D6: CI、文件連結與註解

- `.github/workflows/ci.yml` 的 PostgreSQL job：移除執行 `speclink-store-postgres` 測試的那一行。
- docs/product-status.md 與 docs/product-status.zh-TW.md：四列的證據連結改指到存在的檔案——三個儲存的列各指向對應範例的整合測試檔；「Command Runtime、Host 與 Protocol」那一列原本指向 bridge_dual_path.rs 的連結，改指到保有「兩條路徑結果一致」測試的檔案。連結的文字、列的其他欄位、日期都不動，兩個語言版本改同樣的地方（文件的對照守門要求兩版的列與日期一致）。
- 註解與套件說明：引擎 `lifecycle/tasks.rs` 的一句、協定 `query.rs` 的一句、宿主套件與示範用 server 的 Cargo.toml 的 description、示範用 server 的 Dockerfile 的一句註解、宿主套件 `lib.rs`、`change_diff.rs`、`context.rs`、`binding.rs` 的註解、示範用 server 測試腳手架 `tests/pg/mod.rs` 的註解、deploy/docker-compose.postgres.yml 的一句註解。改成不提 TeamStore 與舊 driver 的 crate 名稱。示範用 server 的 Cargo.toml 的 description 與 Dockerfile 在前一個變更之後已經不提 TeamStore，不用改。
- 完成的判斷：在 crates、apps、packages、scripts、deploy、.github 底下搜尋 `TeamStore`、`speclink_store::`、`speclink-store-sqlite`、`speclink-store-fs`、`speclink-store-postgres`，沒有命中。允許留下的只有：更新日誌與發版說明的歷史內容、測試裡當作詞彙表內容的字串（Speclink Desktop 的測試、宿主套件的 tests/transfer.rs、Node SDK 的 `__test__/transfer.spec.ts`——都是「TeamStore: 團隊儲存」這一行假的詞彙表內容）。docs、README 與 LANGUAGE.md 不在這次搜尋的範圍（留給最後一個變更）。

#### 搜尋結果（任務 3.2 的完成說明）

在 crates、apps、packages、scripts、deploy、.github 底下搜尋 `TeamStore`、`speclink_store::`、`speclink-store-sqlite`、`speclink-store-fs`、`speclink-store-postgres`，剩下 7 行，全部是允許清單裡「TeamStore: 團隊儲存」這一行假的詞彙表內容：

- crates/host/speclink-host/tests/transfer.rs（1 行）
- crates/adapters/speclink-node/__test__/transfer.spec.ts（2 行）
- apps/desktop/src-tauri/tests/it/migration.rs（2 行）
- apps/desktop/src-tauri/src/remote.rs 的測試模組（2 行）

沒有列進搜尋字串、所以留著的一處：crates/protocol/speclink-protocol/src/lib.rs 開頭的註解說這個 crate 不得相依 `speclink-store`。這個 crate 已經不存在，那一句的意思仍然對（協定不相依儲存層），留給最後一個變更和文件的用詞一起整理。

## Implementation Contract

**行為**

- 對外的行為沒有改變。CLI、示範用 server、Speclink Desktop、Node SDK 的全部既有測試不改斷言就通過。
- `speclink_host::projection::content_digest` 仍然存在，輸出相同。
- 從宿主套件引用的 `ProjectId` 與 `RepoId` 仍然存在，建構方式與方法相同。

**介面**

- 宿主套件 `scope`：專案與儲存庫 ↔ 範圍識別字的一對函式（規格 host-runtime「專案與儲存庫對映成範圍識別字」）。
- 宿主套件 `transfer`：公開第一版的版本常數；「文件種類與鍵的對應」的一對函式都公開（規格 host-runtime「文件種類與鍵的對應」）。

**失敗的樣子**

- 沒有新的失敗情況。範圍識別字形式不合的錯誤文字維持原樣（示範用 server 的測試有斷言）。

**驗收**

- `cargo build --workspace` 成功；`cargo metadata` 的 workspace 成員沒有四個舊 crate。
- `cargo test -p speclink-host`、`cargo test -p speclink-server`、`cargo test -p speclink-remote`、`cargo test -p speclink-desktop`、`cargo test -p speclink-cli` 全部通過。
- D6 的搜尋沒有命中（允許清單以外）。
- D2 的 36 個測試去向表完整。
- `speclink workflow-config show` 顯示新的 context 與規則；`speclink validate retire-teamstore-and-drivers` 通過。
- `node --test scripts/*.test.mjs scripts/*/*.test.mjs` 全部通過（含文件的連結檢查與中英對照）。

**範圍邊界**

- 範圍內：D1 到 D6。
- 範圍外：docs 與 README 的文字、LANGUAGE.md、手冊、三個範例與儲存契約的程式。

## 稽核結論

對象：搬進宿主套件的 `ProjectId`、`RepoId`、`scope_id`、`project_and_repo`、`content_digest`，以及改成公開的 `transfer::FIRST_VERSION` 與 `transfer::document_of_key`。用三個角色（刻意濫用的人、照抄範例的人、搞混用法的人）逐一檢查，並和搬家前的行為比對。搬家前的行為是在舊函式還在的時候，用一次性的測試對同一批輸入跑出來記下的。

**搬家前後相同（都有測試）**

| 輸入 | 結果 | 測試 |
| --- | --- | --- |
| 專案或儲存庫的名稱是空字串 | 組得出範圍識別字（`/main`、`team/`、`/`）；反向的函式拒絕這些識別字 | `scope.rs` 的 `an_empty_name_makes_an_id_that_does_not_read_back` |
| 名稱含換行或 NUL | 回報錯誤（範圍識別字不得含控制字元） | `a_name_with_a_control_character_makes_no_id` |
| 名稱超長（專案或儲存庫任一個） | 回報錯誤（超過 256 位元組）；跳脫之後才超長的也算 | `names_that_make_an_id_longer_than_a_scope_id_may_be_are_an_error` |
| 反向：`%` 結尾、`%2`、`%zz`、小寫的 `%2f`、`%` 後面接多位元組字元 | 回報錯誤並指出該識別字，不 panic | `the_reverse_refuses_a_cut_or_unknown_escape_and_does_not_panic` |
| 反向：開頭或結尾是 `/`（`/main`、`team/`） | 回報錯誤 | `a_scope_id_of_the_wrong_form_is_an_error_that_names_it` |
| 反向：`a//b`、`a/b/` | 只有第一個 `/` 分開專案與儲存庫，得到（`a`、`/b`）與（`a`、`b/`） | `only_the_first_slash_ends_the_project` |
| 內容摘要：空字串、一段中文、含 NUL、含 4 位元組 UTF-8 字元 | 與搬家前的值相同（也和 `shasum -a 256` 算出來的相同） | tests/documents.rs 的 `the_content_digest_is_sha256_in_lowercase_hex_with_its_prefix` |

**三個角色的檢查**

- 刻意濫用的人：專案名稱裡的 `%` 與 `/` 都跳脫，儲存庫名稱不跳脫但第一個 `/` 一定是專案的結尾，所以兩組不同的專案與儲存庫組不出同一個範圍識別字。控制字元與超長的名稱在組的時候就被拒絕。內容摘要的演算法寫死，沒有可以選的參數。
- 照抄範例的人：這幾個函式沒有預設值、沒有開關。`document_of_key` 對不合命名規則的鍵回傳「沒有」，呼叫端不會拿到猜出來的文件種類。
- 搞混用法的人：`scope_id` 收的是兩個不同的型別（`ProjectId`、`RepoId`），傳反了編譯不過。示範用 server 的 `scope_backlog` 在這次改動中一度變成收兩個字串，傳反了不會有錯誤訊息——已改成收範圍識別字，由呼叫端用 `scope_id` 組好再傳。`FIRST_VERSION`（1）與 `FORMAT_VERSION`（2）是兩個公開的常數，各自的說明寫明哪一個是依文件種類命名的舊格式。

**留著沒改的一點**

空的專案或儲存庫名稱組得出範圍識別字，但反向的函式不認它。這是搬家前就有的行為，這個變更不改對外的行為，所以照舊並用測試釘住。示範用 server 的專案與儲存庫名稱由它自己的註冊資料管理，不會是空的；自己帶名稱進來的宿主要自己檢查。

## Risks / Trade-offs

- [刪測試時丟掉仍然存在的行為] → D2 要求 36 個測試逐一列出去向；「兩條路徑結果一致」在刪除前先確認新的位置有同樣的斷言。
- [回歸對照] → 引擎的 golden 與 CLI 的整合測試不改；CLI 的 `remote_read_path` 測試從宿主套件引用內容摘要的路徑不變。驗收條件要求 CLI 的測試照常通過。
- [跨平台] → 這個變更沒有新的檔案或路徑處理。刪除 crate 之後在三個平台的建置由 CI 確認。
- [流程設定的 context 被整段重寫] → D5 規定補結尾換行並先 `--dry-run`。
- [文件的守門在刪除檔案之後變紅] → D6 在同一個變更修連結；最後一個任務跑全部的 scripts 測試。守門紅的時候 CI 會跳過其他四組測試，所以這一步不能漏。
- [直接刪除規格資料夾繞過了 delta] → 這是刻意的（D4）。理由寫在提案與這份設計；仍然成立的兩個規則已經用 delta 在 host-runtime 另立需求。
- [本機的 CLI 產物被 workspace 測試換成舊的] → 跑完 `cargo test -p speclink-desktop` 之後先 `cargo build -p speclink-cli` 再執行 speclink 的動詞。

## Migration Plan

沒有資料要遷移。回退方式是還原這個變更的 commit。

合併之後，開發者本機的 `target` 資料夾裡會留著四個舊 crate 的編譯產物；它們不影響建置。

## Open Questions

（無）

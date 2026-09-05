# command-runtime Specification

## Purpose

動詞執行期的跨入口共通語意：動詞覆蓋範圍與 CLI／server／node／desktop 之間的行為一致性、穩定錯誤碼註冊表、變更型動詞發出的領域事件，以及壞 change metadata 的統一處置。本 capability 保證同一動詞不論由哪個入口呼叫都得到相同結果與錯誤碼，且本地檔案寫入一律原子落盤。

## Requirements

### Requirement: 動詞覆蓋與跨入口一致性

引擎 SHALL 提供唯一的命令執行層，覆蓋讀寫規格儲存的領域動詞——查詢：list、show、status、instructions、validate、analyze、drift、artifact cat、language show、discuss list、discuss show 與 discuss search；變更：new change、new artifact、task done、task undone、claim、in-progress add、archive、discard、discuss new／context／add-round／conclude／promote／link／seal／archive／discard。CLI 與 Node SDK dispatch SHALL 經此層執行覆蓋表動詞；對相同 workspace 狀態執行同一動詞，各入口 SHALL 得到相同的語意結果與錯誤分類，且既有人眼輸出與 --json 形狀 SHALL 維持位元級一致（既有輸出基線不變）。discuss search 為唯讀查詢動詞，SHALL NOT 發出領域事件。workspace bootstrap 與周邊工具動詞（init、update、config、schema、completion、templates、feedback、demo）及 remote 連線管理動詞（link、unlink、auth）SHALL NOT 進入命令層。

#### Scenario: CLI 與 dispatch 的成功結果語意一致

- **WHEN** 同一 workspace 內分別執行 speclink list --json 與 engine.dispatch(['list'])
- **THEN** 兩者回傳的 changes 清單語意相同（同名稱集合、同排序），dispatch 結果為與 CLI --json 對齊的結構化物件

#### Scenario: CLI 與 dispatch 的錯誤分類一致

- **WHEN** 對不存在的 change 分別執行 speclink status --change ghost 與 engine.dispatch(['status', '--change', 'ghost'])
- **THEN** CLI 以非零 exit code 結束且 stderr 為現行訊息；dispatch 以 Error 拒絕、code 為 not_found、message 與 CLI 訊息文字相同

#### Scenario: 覆蓋動詞輸出凍結

- **WHEN** 對同一 workspace 於命令層導入前後執行覆蓋表內任一動詞（人眼與 --json 兩形式）
- **THEN** stdout 與 stderr 逐位元一致、exit code 相同（壞設定檔情境除外，該情境見 workflow-config 與 remote-connection 規格）

#### Scenario: discuss search 本機與 server 同語意

- **WHEN** 對同一組討論記錄分別以本機 speclink discuss search drawer --json 與 server 的 GET /discussions/search?q=drawer 查詢
- **THEN** 兩者回傳的 hits 順序、每筆的 slug 與 matches 陣列相同；本機執行 SHALL NOT 產生任何領域事件


<!-- @trace
source: discuss-search-recall
updated: 2026-09-05
-->

---
### Requirement: 穩定錯誤碼註冊表

命令層 SHALL 以封閉的錯誤碼集合分類失敗：invalid_argv（參數不合法）、not_found（主體不存在）、invalid_config（設定檔或 change metadata 檔存在但無法解析）、refused（前置條件拒絕，須 --force 或先完成前置動作）、error（其餘失敗）。同一失敗情境的錯誤碼 SHALL NOT 因入口而異；錯誤的語意訊息文字 SHALL 沿用現行 CLI 訊息。

#### Scenario: 需 --force 的拒絕

- **WHEN** 對已記錄開工的 change 執行 speclink discard（未帶 --force）
- **THEN** 指令以非零 exit code 拒絕、不刪除任何檔案，stderr 為現行拒絕訊息（此情境在命令層歸類為 refused）

#### Scenario: 錯誤碼跨入口穩定

- **WHEN** 以相同的非法參數組合分別經 CLI 與 dispatch 執行同一動詞
- **THEN** dispatch 錯誤碼為 invalid_argv，CLI 以非零 exit code 輸出同語意訊息

##### Example: 失敗情境對應錯誤碼

| 情境 | 錯誤碼 |
| --- | --- |
| status 指到不存在的 change | not_found |
| discard 已開工的 change 未帶 --force | refused |
| discuss discard 已有 rounds 未帶 --force | refused |
| .speclink.yaml 存在但 YAML 解析失敗 | invalid_config |
| openspec/config.yaml 存在但 YAML 解析失敗 | invalid_config |
| 某 change 的 .openspec.yaml 存在但 YAML 解析失敗 | invalid_config |
| dispatch 收到未支援的動詞 | invalid_argv |

---
### Requirement: 變更型動詞的領域事件

覆蓋表內每個變更型動詞成功時，命令層 SHALL 隨執行結果回報一至多筆領域事件，每筆事件 SHALL 含種類名、主體識別（change 名或 discussion slug）與 UTC 時間戳；查詢型動詞與失敗的執行 SHALL NOT 產生事件。本能力 SHALL NOT 含事件持久化與訂閱；事件契約標示為 experimental，於事件持久化能力落地前 SHALL 允許不相容調整。

#### Scenario: 建立變更回報 change-created

- **WHEN** 經引擎命令層成功建立名為 add-auth 的 change
- **THEN** 執行結果附帶恰一筆 change-created 事件，主體為 add-auth 且含 UTC 時間戳

#### Scenario: 失敗的命令不產生事件

- **WHEN** 以已存在的名稱再次建立 change 而失敗
- **THEN** 執行結果為錯誤且不附帶任何事件

#### Scenario: 複合動詞回報多筆事件

- **WHEN** 經引擎命令層將已結論的討論 promote 成新 change
- **THEN** 執行結果附帶 discussion-promoted（主體為討論 slug）與 change-created（主體為新 change 名）兩筆事件

##### Example: 變更型動詞與事件種類對應

| 動詞 | 事件種類 |
| --- | --- |
| new change | change-created |
| new artifact | artifact-created |
| task done | task-completed |
| task undone | task-uncompleted |
| task move | task-moved |
| claim | change-claimed |
| in-progress add | change-marked-in-progress |
| archive | change-archived |
| discard | change-discarded |
| discuss new | discussion-created |
| discuss context | discussion-context-set |
| discuss add-round | discussion-round-added |
| discuss conclude | discussion-concluded |
| discuss promote | discussion-promoted 與 change-created |
| discuss link | discussion-linked |
| discuss seal | discussion-sealed |
| discuss archive | discussion-archived |
| discuss discard | discussion-discarded |


<!-- @trace
source: remote-verb-parity
updated: 2026-07-23
code:
  - apps/desktop/core/src/manage.rs
  - apps/desktop/src-tauri/src/lib.rs
  - apps/desktop/src-tauri/src/remote.rs
  - apps/desktop/src-tauri/tests/remote_data.rs
  - apps/desktop/src/__tests__/remoteDataSource.test.ts
  - apps/desktop/src/adapter/remoteDataSource.ts
  - crates/speclink-cli/src/commands.rs
  - crates/speclink-cli/src/remote_commands.rs
  - crates/speclink-cli/tests/remote_verb_parity.rs
  - crates/speclink-core/src/command/mod.rs
  - crates/speclink-core/src/tasks.rs
  - crates/speclink-host/src/bridge.rs
  - crates/speclink-host/src/commit.rs
  - crates/speclink-protocol/src/binding.rs
  - crates/speclink-protocol/src/command.rs
  - crates/speclink-protocol/src/query.rs
  - crates/speclink-remote/src/client.rs
  - crates/speclink-remote/src/convert.rs
  - crates/speclink-remote/src/lib.rs
  - crates/speclink-server/src/app.rs
  - crates/speclink-server/src/auth.rs
  - crates/speclink-server/src/events.rs
  - crates/speclink-server/src/routes.rs
  - crates/speclink-server/tests/verb_api.rs
  - docs/verb-contract.md
  - docs/verb-contract.zh-TW.md
-->

---
### Requirement: change metadata 損壞的跨入口處置

`.openspec.yaml` 存在但 YAML 解析失敗的 change SHALL 標為 invalid：list SHALL 照常列出全部 change，該 change 的 `--json` 項目 SHALL 附選填欄位 metaError（值為解析原因）、人眼輸出 SHALL 於該行附 invalid 標記，其餘 change 的輸出 SHALL NOT 受影響。需要 metadata 語意的單一 change 動詞（查詢：status、instructions、validate、analyze、drift、artifact cat；與全部變更型動詞）SHALL 拒絕並停止，錯誤訊息 SHALL 指出該 metadata 檔的 workspace 相對路徑與解析原因，錯誤碼 SHALL 為 invalid_config 且 SHALL NOT 因入口而異。檔案不存在或欄位缺席 SHALL 維持既有預設行為。

#### Scenario: list 對壞 metadata 標 invalid 而不失效

- **WHEN** workspace 含兩個 change 且其一 `.openspec.yaml` 為壞 YAML，執行 speclink list --json
- **THEN** exit code 0；清單含全部兩個 change；壞檔項目帶 metaError 欄位；有效項目無 metaError 欄位且內容與無壞檔時一致

#### Scenario: 單一 change 查詢對壞 metadata fail closed

- **WHEN** 對 `.openspec.yaml` 為壞 YAML 的 change 執行 speclink status --change 該 change
- **THEN** 以非零 exit code 結束；stderr 指出該檔的 workspace 相對路徑與解析原因；此情境於命令層歸類為 invalid_config

#### Scenario: dispatch 與 CLI 對壞 metadata 的錯誤分類一致

- **WHEN** 同一壞 metadata 的 workspace 分別經 CLI 與 engine.dispatch(['status', '--change', 該 change]) 執行
- **THEN** dispatch 以 Error 拒絕、code 為 invalid_config，message 與 CLI 訊息文字相同

---
### Requirement: 本地檔案寫入原子落盤

引擎對 workspace 共享真相檔案（openspec/ 樹、.speclink.yaml、openspec/config.yaml）的寫入 SHALL 經單一寫檔入口以原子方式落盤：先寫同目錄暫存檔、再 rename 至目的路徑，使並行讀者於任一時點讀到的都是舊全文或新全文，SHALL NOT 觀察到空檔或部分內容。rename 因平台限制失敗（如 Windows sharing violation）時 SHALL 退回直接寫入並清理暫存檔——行為不劣於原子化前；暫存檔因權限限制建不出來（如父目錄不可寫而目的檔可寫）時 SHALL 同樣退回直接寫入，其他原因的暫存檔寫入失敗（如磁碟已滿）SHALL 浮出錯誤且目的檔內容不變。成功路徑 SHALL NOT 於目的目錄殘留暫存檔。原子保證於 unix SHALL 於暫存檔可建立時全額成立，Windows 為 best-effort（退回路徑存在即可）。

#### Scenario: 並行讀者不見半份內容

- **WHEN** 一個 process 正經引擎寫入 workspace 檔案，另一 process（或執行緒）同時反覆讀取同一路徑
- **THEN** 每次讀取得到的都是舊全文或新全文之一，絕不出現空檔、截斷或新舊混合內容（unix 全額保證）

#### Scenario: 寫入完成不殘留暫存檔

- **WHEN** 引擎寫檔成功完成
- **THEN** 目的目錄中不存在該次寫入使用的暫存檔

#### Scenario: 設定寫入走同一原子入口

- **WHEN** CLI 的設定編輯動詞或 desktop 設定頁寫入 openspec/config.yaml
- **THEN** 寫入經同一原子入口落盤，觀察面與引擎其他寫入一致（無暫存殘留、內容為完整全文）

<!-- @trace
source: atomic-file-writes
updated: 2026-08-11
-->
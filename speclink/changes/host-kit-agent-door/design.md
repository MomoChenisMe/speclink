## Context

前五個變更完成後，現況是這樣：

- `speclink-store-port` 定義三個函式的儲存接口（`StorePort`：列出、讀取、提交）、記憶體版的參考實作、16 項檢查的合格測試。只有 Rust 的介面。
- `speclink-host` 的 `port_exec::execute` 能在任何 `StorePort` 與一個範圍上執行一個引擎命令：用到才讀、一個動詞的寫入與事件收成一次提交。它從執行情境取操作者的顯示字串，同時當成提交記錄的識別與顯示名稱；提交記錄的命令標籤是從結果反推的，驗證的三個命令與搬移任務沒有專屬標籤，記成 `command`。
- `speclink-verbs` 是動詞文字的唯一解析與輸出：`parse`、`to_command`、`render`。
- 流程設定的讀寫是引擎命令（變更 workflow-config-commands），寫入的結果帶變動的鍵。
- `speclink-node` 的 `createEngine` 仍然走舊路：宿主的 JS 儲存有 31 個必要方法與 7 個選填方法，引擎直接對它執行，不經 `port_exec`。沒有交易、沒有版本比對，事件在回傳後被丟掉。操作者在建立引擎時綁定一次。
- 示範用 server 經 `bridge::execute` 走 `port_exec`，權限由它自己的路由層決定。引擎與 Host 的執行情境都沒有權限的概念。

這個變更要讓 Node 的宿主拿到新路徑，並給宿主兩個介入點。

限制：

- 引擎核心維持同步。Node 的 JS 函式可以回傳 Promise，所以跨界的等待在 `speclink-node` 處理。
- 原版的 CLI 與 Speclink Desktop 之後會經另一個門連到同一個宿主套件，所以宿主套件的規則不能依賴動詞文字：HTTP 的門收到的是請求，不是 argv。
- 這個系列全部完成才發版，所以這次新增的 Node 介面在之後的變更還能調整。

## Goals / Non-Goals

**Goals:**

- 一個由全部使用者共用的宿主套件：建立時給儲存與掛鉤，每次呼叫給身分與範圍。
- 宿主能在任何寫入發生前拒絕一個命令，並給原因。
- 宿主能在提交後收到事件做通知。
- 流程設定的寫入，宿主在決定前看得到會變動的鍵。
- Node 的宿主只實作三個 JS 函式，並能自己跑合格測試證明實作正確。
- Agent 工具的門涵蓋全部有引擎命令的動詞，結果與 `engine.dispatch` 相同。

**Non-Goals:**

- HTTP 的門、握手、輪詢、範圍清單、登入方式的宣告、能力開關（變更 host-kit-client-door）。
- 匯入匯出與大小上限（變更 import-export-on-port）。
- 示範用 server 改接宿主套件（變更 host-kit-client-door）。
- 移除 `createEngine` 的 JS 儲存形式（變更 retire-teamstore-and-drivers）。
- 替宿主收集本機事實（改了哪些檔、commit）。
- Rust 的動詞文字入口。

## Decisions

### D1: 宿主套件放在 speclink-host 的 kit 模組

`speclink-host` 是引擎唯一的應用層邊界，執行入口 `port_exec` 已經在這裡。宿主套件是它上面的一層薄殼：呼叫掛鉤、組身分、呼叫 `port_exec`。新增 `kit.rs`，公開：

- `HostKit::new(store: Arc<dyn StorePort>, hooks: Hooks) -> HostKit`
- `HostKit::run(&self, caller: &Caller, cmd: Command) -> Result<PortExecution, ExecError>`
- `Caller { identity: Identity, scope: ScopeId }`
- `Identity::new(id, display_name) -> Result<Identity, InvalidIdentity>`：兩個欄位都不可以是空字串或只有空白。`InvalidIdentity` 指出是哪一個欄位；儲存契約的 `InvalidValue` 只能由 store-port 自己建立，所以宿主套件用自己的錯誤型別。
- `Hooks { before_command, after_commit }`，兩個欄位都是 `Option`，`Hooks::default()` 是兩個都沒有。
- `BeforeCommand`、`Decision`、`AfterCommit`（D4、D5）。

`HostKit` 是 `Send + Sync`，內部沒有可變狀態，可以放進 `Arc` 給全部請求共用。

`speclink-host` 不相依 `speclink-verbs`：宿主套件收的是 `Command`。動詞文字的門由 `speclink-node` 組裝（D6）。之後 HTTP 的門也是把請求轉成 `Command` 再呼叫 `HostKit::run`，所以兩個門共用同一條處理路徑。

- **否決：新開一個 crate** — 只有一個檔案的內容，而且它需要 `port_exec` 的非公開部分。
- **否決：把掛鉤做進 `port_exec::execute` 的參數** — 示範用 server 與既有測試直接呼叫 `execute`，它們不需要掛鉤；分兩層後既有呼叫端不用改。

### D2: 身分怎麼進入引擎與提交

宿主給的身分有兩個欄位，去處不同：

| 欄位 | 去處 |
|---|---|
| 識別（`id`） | 提交記錄的 `actor.id` |
| 顯示名稱（`display_name`） | 提交記錄的 `actor.display_name`；引擎執行情境的操作者字串，也就是文件裡 `created_by`、`reviewed_by`、`verified_by` 與討論記錄建立者的值 |

文件裡的蓋章欄位維持一個字串，不加識別：這些欄位是人讀的，而且本機專案沒有識別可填。宿主想讓蓋章帶 email 時，把顯示名稱給成「名稱 <email>」。

`port_exec` 新增 `execute_with`，比 `execute` 多收兩樣：提交用的身分（識別與顯示名稱），以及一個選填的「提交前檢查」函式（D4 用）。它收的執行情境是引擎的執行情境，不是 Host 的執行情境：宿主套件沒有專案與 repo 的綁定，下面的「repo 留空」要由呼叫端決定。既有的 `execute` 改成呼叫 `execute_with`：身分照舊由執行情境的顯示字串組成、沒有提交前檢查，行為不變。

引擎執行情境的其他欄位，宿主套件這樣填：沒有 workspace、沒有環境變數的覆寫、沒有使用者設定資料夾、repo 留空。也就是引擎不讀程序的環境、不讀任何 checkout。

- **否決：把識別也寫進文件的蓋章欄位** — 會改變既有文件的格式，本機與遠端的同一份文件會長得不一樣。
- **否決：身分放進動詞文字或 stdin** — 動詞文字由模型產生，模型能偽造身分。

### D3: 命令的名稱與參數

`speclink-core` 新增 `command/describe.rs`，給 `Command` 兩個方法：

- `name(&self) -> &'static str`：每個變體一個固定的 kebab-case 名稱，等於變體名稱轉成 kebab-case（`NewChange` → `new-change`、`DiscussAddRound` → `discuss-add-round`、`WorkflowConfigEdit` → `workflow-config-edit`）。用手寫的 `match`，不加萬用分支：引擎新增命令而沒有命名時編譯不過。
- `params(&self) -> serde_json::Value`：一個 JSON 物件，鍵是該變體的欄位名稱轉成 camelCase，值是欄位的值。沒有欄位的變體回傳空物件。`Command` 與它欄位用到的型別加上 `Serialize`。

這兩個方法給掛鉤用，也給提交記錄用：`port_exec` 的命令標籤改成 `cmd.name()`，原本從結果反推的對照表移除。今天有專屬標籤的 23 個寫入命令，標籤都等於變體名稱的 kebab-case，所以不變；驗證的三個命令與搬移任務從 `command` 變成各自的名稱。

參數包含文字內容（例如 artifact 的全文）。宿主可以看，不能改。

名稱與參數的欄位名稱從此是對宿主的契約。用一份 golden 釘住：每個變體一筆，內容是名稱與參數的 JSON。改名或改欄位會讓這個測試紅。

- **否決：掛鉤收動詞文字（argv）** — HTTP 的門沒有 argv，兩個門的掛鉤會收到不同的東西。
- **否決：只給挑選過的幾個欄位（變更名稱、討論 slug）** — 宿主的用途包含「不准帶 `--carry-review` 封存」這種看旗標的規則，挑選過的欄位不夠。
- **否決：名稱沿用 CLI 的動詞文字（例如 `new change`）** — 一個動詞能對到不同的命令（`plan` 與 `plan --strict-overlap`、`workflow-config` 的 11 個動詞對到三種命令），而掛鉤要的是命令。

### D4: 執行前掛鉤

型別：`before_command: Option<Box<dyn Fn(&BeforeCommand) -> Decision + Send + Sync>>`。

- `BeforeCommand { identity, scope, command, params, changed_keys }`：`command` 是 D3 的名稱，`params` 是 D3 的參數，`changed_keys` 只有流程設定的寫入命令有值。
- `Decision::Allow` 或 `Decision::Deny { reason }`。

`HostKit::run` 對每次呼叫正好問一次，時間點分兩種：

1. **一般命令**：在引擎執行之前。拒絕時引擎沒有執行，儲存沒有收到任何呼叫。這樣沒有權限的人無法從「找不到」與「被拒絕」的差別探知資料是否存在。
2. **流程設定的寫入命令**（`workflow-config-edit` 與 `workflow-config-replace`）：在引擎算出結果之後、提交之前，經 D2 的提交前檢查。這時引擎的寫入只暫存在記憶體，儲存只被讀過。掛鉤收到的 `changed_keys` 取自這次執行的結果，和即將提交的內容是同一份；提交時的版本比對保證它所根據的內容沒有被別人改過。dry-run 與內容沒有變動時不提交，掛鉤照樣被問一次。

拒絕的結果：`ExecError::Command`，錯誤碼 `refused`，訊息是宿主給的原因；原因是空字串時用固定的文字「refused by host」。儲存沒有任何變動，提交後掛鉤不會被呼叫。

沒有提供掛鉤時一律放行。

- **否決：流程設定的寫入也在引擎之前問，先用 dry-run 算一次變動的鍵** — 兩次執行之間別人改了設定時，掛鉤核准的鍵和實際寫入的鍵會不同。
- **否決：全部命令都在引擎之後、提交之前問** — 被拒絕的人會先看到引擎的錯誤，能分辨「找不到」與「被拒絕」；也白做一次引擎的工作。

### D5: 提交後掛鉤

型別：`after_commit: Option<Box<dyn Fn(&AfterCommit) + Send + Sync>>`。

- `AfterCommit { identity, scope, command, scope_version, events }`：`events` 是這次提交的事件，每筆有名稱與 payload，和交給儲存提交函式的那一份相同。

只有真的提交了才呼叫：查詢、沒有寫入也沒有事件的命令、被拒絕或失敗的命令都不呼叫。呼叫發生在儲存的提交函式成功回傳之後。

它沒有回傳值，不能影響這次呼叫的結果。程序在提交後、呼叫前當掉時，這次通知就沒有了；這是它和提交函式的差別。需要保證的宿主在提交函式裡處理事件。

### D6: Node 的 createHost 與 host.dispatch

`speclink-node` 新增：

```ts
createHost(options: { store: StorePort; hooks?: HostHooks }): Host
interface Host {
  dispatch(call: { identity: { id: string; displayName: string }; scope: string; argv: string[]; stdin?: string }): Promise<unknown>
}
```

`host.dispatch` 的步驟：

1. 檢查 `identity` 與 `scope` 的格式。不合格時以 `invalid_argv` 拒絕。
2. 用 `speclink_verbs::parse` 解析 `argv`，檢查 stdin 的規則，用 `to_command` 轉成命令。這三步和 `engine.dispatch` 是同一段程式：把今天 `run_dispatch` 裡「執行」以外的部分抽成共用的函式，兩個入口只有「怎麼執行命令」不同。
3. 需要 checkout 的動詞照舊以 `invalid_argv` 拒絕。
4. 呼叫 `HostKit::run`。
5. 用 `speclink_verbs::render` 產生結果：有 `--json` 內容時回傳它，否則回傳 `{ output }`（有 stderr 的提示時多一個 `stderr`）。

和 `engine.dispatch` 的三個差別：

- **身分每次呼叫給**。`createHost` 沒有 `actor` 欄位。
- **蓋章不需要鎖，也不需要選填的儲存方法**。提交時的版本比對取代了實例內的鎖；刪除工單與改寫 metadata 都是同一次提交裡的操作。
- **儲存失敗有自己的錯誤碼**，沿用線上協定的分類名稱：版本衝突是 `revision_conflict`、暫時無法使用是 `unavailable`、其他是 `internal`。引擎的錯誤碼（`invalid_argv`、`not_found`、`invalid_config`、`refused`、`error`）不變。

`status` 不帶名稱且範圍裡沒有任何變更時，結果和 `engine.dispatch` 相同：一段提示文字，不是錯誤。

`dispatch` 在背景執行緒執行，不阻塞 JS 的事件迴圈；多個呼叫可以同時進行。

掛鉤的 JS 形狀：

```ts
interface HostHooks {
  beforeCommand?(info: { identity; scope: string; command: string; params: object; changedKeys?: string[] }):
    MaybePromise<void | { allow: true } | { allow: false; reason: string }>
  afterCommit?(info: { identity; scope: string; command: string; scopeVersion: number; events: { name: string; payload: unknown }[] }): MaybePromise<void>
}
```

- `beforeCommand` 回傳 `undefined` 或 `{ allow: true }` 是放行。丟出例外、Promise 被拒絕、或回傳其他形狀時，這次呼叫以 `internal` 結束並指出是哪個掛鉤，儲存沒有變動。也就是掛鉤壞掉時不放行。
- `afterCommit` 不等待：`dispatch` 的 Promise 不因它而延後。它丟出的例外與被拒絕的 Promise 都由橋接吃掉，不會變成未處理的 rejection。

### D7: JS 版的儲存接口

`port_bridge.rs` 在一個 JS 物件上實作 `StorePort`。JS 的形狀是 Rust 契約的對映，欄位用 camelCase：

```ts
interface StorePort {
  list(scope: string, prefix: string): MaybePromise<{ version: number; entries: { key: string; version: number; modifiedAt: number }[] }>
  read(scope: string, keys: string[]): MaybePromise<({ content: string; version: number; modifiedAt: number } | null)[]>
  commit(scope: string, commit: {
    actor: { id: string; displayName: string }
    command: string
    at: number
    ops: ({ op: 'put'; key: string; content: string; expected: number | null } | { op: 'delete'; key: string; expected: number })[]
    events: { name: string; payload: unknown }[]
  }): MaybePromise<number>
}
```

- `expected: null` 是「必須不存在」；數字是預期的文件版本。
- 版本與時間都是 0 到 9007199254740991 的整數，JS 的 number 能精確表示。
- `createHost` 在建立時檢查三個函式都存在，缺少時同步丟出錯誤並列出缺少的名稱。

回傳值的檢查（任何一項不合格都是儲存失敗，分類 `internal`，訊息指出是哪個函式與哪個欄位）：`read` 回傳的陣列長度等於鍵的數量；版本與時間是範圍內的整數；`list` 的鍵符合鍵的格式。

JS 丟出的錯誤怎麼分類：

| 丟出的東西 | 分類 |
|---|---|
| 物件的 `code` 是 `'conflict'`，而且是 `commit` 丟的，`key` 是這次提交裡的一個鍵 | 版本衝突；預期值取自這次提交的那筆操作，實際值取自錯誤的 `actual`（數字或 `null`；沒有給時當成 `null`。儲存契約的衝突沒有「未知」這個值，而合格測試會判定不給 `actual` 的儲存失敗） |
| 物件的 `code` 是 `'unavailable'` | 暫時無法使用 |
| 其他（含 `code` 是 `'conflict'` 但 `key` 不在這次提交裡） | 其他失敗 |

套件匯出兩個小類別 `StoreConflictError(key, actual)` 與 `StoreUnavailableError(message)`，它們只是設好 `code` 的 `Error`。分類只看 `code`，所以宿主不用這兩個類別也可以。

跨界的做法沿用 `store_bridge.rs` 既有的機制：背景執行緒經 threadsafe function 呼叫 JS 函式，回傳 Promise 時等它完成。

- **否決：`expected` 用字串 `'absent'`** — 一個欄位兩種型別，宿主寫 SQL 時 `null` 更直接。
- **否決：用錯誤類別的 `instanceof` 分類** — 宿主的打包工具可能載入兩份套件，`instanceof` 會失效。

### D8: 合格測試隨 npm 套件提供

```ts
runStoreConformance(harness: {
  store: StorePort
  newScope(): MaybePromise<string>
  recordedEvents?(scope: string): MaybePromise<{ name: string; payload: unknown; version: number }[]>
}): Promise<{ passed: boolean; checks: { name: string; outcome: 'passed' | 'failed' | 'skipped'; detail?: string }[] }>
```

它在背景執行緒呼叫 Rust 的 `speclink_store_port::conformance::run`，JS 的物件經 D7 的橋接成為受測的儲存。所以 Rust 實作與 JS 實作跑的是同一組 16 項檢查，沒有第二份。

- `newScope` 每次回傳一個沒用過的範圍。
- `recordedEvents` 是選填。沒有提供時，`events-with-commit` 的結果是 `skipped` 並說明原因；`events-only-commit` 照樣檢查只有事件的提交會讓範圍版本前進，只略過讀回事件的那一段。這和正式規格 store-port-contract 對不保存事件的實作的規定相同。`passed` 只看沒有 `failed`。
- 並行的兩項檢查（`concurrent-single-winner`、`batch-read-atomic`）從多個 Rust 執行緒同時呼叫；JS 這邊是同一個事件迴圈上交錯的非同步呼叫，所以它們驗到的是宿主的資料庫交易，而不是 JS 的執行緒。

SDK 自己的測試用一個測試專用的 JS 記憶體儲存（`__test__/memory-port.ts`）跑這組檢查，證明 JS 實作能通過；再用一個故意不比對版本的儲存，確認它被判定失敗。這個記憶體儲存不對外匯出。

- **否決：把合格測試改寫成 JS** — 會有兩份檢查，Rust 那份改了 JS 這份不會跟著改。
- **否決：對外匯出記憶體儲存** — 沒有需求；宿主要試用時十幾行就能寫出來，而且合格測試會告訴它寫得對不對。

### D9: Agent 工具的定義

`speclink-verbs` 新增 `tool.rs`：`tool_definition() -> ToolDefinition { name, description, input_schema }`。

- `name` 固定是 `speclink`。
- `input_schema` 是 JSON Schema：物件，`argv`（字串陣列，必填）與 `stdin`（字串，選填），不接受其他屬性。
- `description` 是英文，給模型看。開頭是固定的一段用法說明（argv 是不含 `speclink` 的動詞文字、帶 `--stdin` 的動詞把內容放在 `stdin`、加 `--json` 取得結構化的結果），後面是動詞清單。清單從 clap 的定義走訪產生：每個有引擎命令的末端動詞一行，內容是動詞文字與它的一行說明；需要 checkout 的動詞不列。

Node 把它匯出成常數 `agentTool`：`{ name, description, inputSchema }`。身分與範圍不在輸入格式裡，由宿主的程式在呼叫 `host.dispatch` 時給。

工具說明不含技能的文字。技能怎麼交給宿主的 Agent 是既有的渲染 API 的事。

- **否決：手寫動詞清單** — 動詞增加時會落後。
- **否決：每個動詞一顆工具** — 討論 agent-tool-sdk-layering 已否決型別化的方法。

### D10: 標示 31 方法的 JS 儲存即將移除

`index.d.ts` 的 `Store` 型別與 `CreateEngineOptions` 的 JS 儲存形式加上 `@deprecated` 的註解，指向 `createHost`。執行期的行為不變、不印警告。

## Implementation Contract

**行為**

- `HostKit::run` 對同一個儲存、同一個範圍、同一個命令，結果（outcome、事件、範圍版本號）和直接呼叫 `port_exec::execute` 相同；差別只有提交記錄的身分取自 `Caller`，以及掛鉤被呼叫。
- 執行前掛鉤每次呼叫正好被問一次。拒絕時：錯誤碼 `refused`、訊息是宿主的原因、儲存沒有收到提交、提交後掛鉤沒有被呼叫。一般命令被拒絕時儲存連讀取都沒有收到。
- 流程設定的寫入命令，掛鉤收到的 `changed_keys` 等於該命令結果裡的 `changed_keys`。
- 提交後掛鉤只在有提交時被呼叫一次，收到的事件與範圍版本號等於提交的內容與回傳值。
- 提交記錄的 `command` 等於 `Command::name()`；`actor.id` 與 `actor.display_name` 等於宿主給的身分。
- 文件裡的蓋章欄位等於顯示名稱。

**介面與資料形狀**

- Rust：`speclink_host::kit::{HostKit, Caller, Identity, Hooks, BeforeCommand, Decision, AfterCommit}`；`speclink_host::port_exec::execute_with`；`speclink_core::command::Command::{name, params}`；`speclink_verbs::tool::{tool_definition, ToolDefinition}`。
- Node：`createHost`、`Host.dispatch`、`runStoreConformance`、`agentTool`、`StoreConflictError`、`StoreUnavailableError`，以及型別 `StorePort`、`HostHooks`、`HostIdentity`。形狀見 D6、D7、D8、D9。
- 命令名稱與參數的 golden：`crates/engine/speclink-core/tests/golden` 底下一份，每個命令變體一筆。

**失敗的樣子**

| 情況 | 結果 |
|---|---|
| 掛鉤拒絕 | `refused`，訊息是原因 |
| JS 的執行前掛鉤丟出例外或回傳不認得的形狀 | `internal`，訊息指出掛鉤名稱；儲存不變 |
| JS 的提交後掛鉤丟出例外 | 沒有影響 |
| 儲存回報版本衝突 | `revision_conflict`，訊息含鍵、預期與實際 |
| 儲存回報暫時無法使用 | `unavailable` |
| 儲存的其他失敗、回傳值不合格 | `internal` |
| 身分或範圍格式不合 | `invalid_argv` |
| 需要 checkout 的動詞 | `invalid_argv`（既有行為） |

**驗收**

- `cargo test -p speclink-core` 含命令名稱與參數的 golden：變體的數量等於 `Command` 的變體數。
- `cargo test -p speclink-host --test kit`：用記憶體版儲存涵蓋上面「行為」的每一條。
- `cargo test -p speclink-host` 既有的測試不改斷言就通過（除了提交標籤從 `command` 變成專屬名稱的那幾個，如果有測試斷言它）。
- `cargo test -p speclink-verbs`：工具說明列出的動詞集合等於「有引擎命令的末端動詞」集合。
- Node：`host.spec.ts`、`port-conformance.spec.ts`、`agent-tool.spec.ts` 通過；既有的 Node 測試不改就通過。
- `cargo test -p speclink-cli` 與 `cargo test -p speclink-server` 不改就通過。

**範圍**

- 在範圍內：上面列的介面、提交標籤的來源改成命令名稱、型別宣告的 `@deprecated`。
- 不在範圍內：`speclink-cli`、`speclink-server`、store crates、桌面 app、技能的文字、使用者文件。

## Risks / Trade-offs

- [回歸對照] `Command` 加上 `Serialize` 與兩個方法不改任何輸出；CLI 的 golden 與 `remote_verb_parity` 不受影響。→ 驗收條件要求 `speclink-cli` 與 `speclink-server` 的測試不改就通過。
- [提交標籤改變] 驗證的三個命令與搬移任務的標籤從 `command` 變成專屬名稱。→ 標籤只進儲存的提交記錄；先搜尋 `speclink-host`、`speclink-server` 與三個 driver 的測試有沒有斷言 `"command"` 這個值，有的話同批更新並在任務裡列出。
- [參數成為契約] 之後改 `Command` 的欄位名稱會影響宿主的掛鉤。→ golden 會紅，改名的人看得到；這個系列發版前還能調整。
- [跨平台] 新增的程式沒有檔案路徑與程序呼叫；threadsafe function 的機制三個平台相同，既有的 JS 儲存橋接已在 Windows、macOS、Linux 的 CI 跑過。→ 並行的合格測試在 Windows 的 CI 也要通過；逾時的餘裕沿用既有測試的設定。
- [JS 掛鉤的延遲] 每個命令多一次跨界呼叫。→ 沒有提供掛鉤時不跨界。
- [死結] 背景執行緒等 JS 的 Promise，而 JS 的函式若同步等待同一個 `dispatch` 的結果會卡住。→ 這是既有 JS 儲存橋接已有的限制，型別宣告的註解寫明掛鉤與儲存函式裡不要等待同一個宿主的 `dispatch`。
- [儲存解耦] 這個變更讓 Node 的宿主第一次走上「三個函式」的路徑，舊的 31 方法路徑只剩下等待移除；方向和「storage 解耦的規格驅動引擎」一致。

## 稽核結論

依 sharp-edges 稽核清單（`speclink instructions --skill audit`）逐項檢查新增的介面，結論如下。

| 檢查項目 | 結論 | 證據 |
|---|---|---|
| 執行前掛鉤拒絕後仍然提交 | 沒有這條路徑。一般命令在引擎執行前被問；流程設定的寫入在 `execute_with` 的提交前檢查被問，檢查失敗就不呼叫儲存的提交。引擎或儲存在檢查前就失敗時，宿主套件補問一次，拒絕優先於原本的錯誤。 | `tests/kit.rs`：`refusing_an_archive_gives_the_reason_and_touches_nothing`、`a_dry_run_and_a_failed_write_are_asked_once_too`；`port_exec` 的 `a_failing_check_stops_the_commit_and_returns_its_error` |
| 掛鉤壞掉時不放行 | Node 的 `beforeCommand` 丟出例外、Promise 被拒絕、回傳不認得的形狀，都當成拒絕並以 `internal` 結束。Rust 的掛鉤 panic 會往上傳，提交前就中斷。 | `host.spec.ts`：`does not allow when beforeCommand throws`、`does not allow an answer it does not know` |
| 掛鉤名稱打錯而默默全部放行 | 找到並修正：`createHost` 遇到不認得的掛鉤名稱（例如 `beforecommand`）同步拋出錯誤。 | `host.spec.ts`：`refuses a hook name it does not know` |
| argv 與 stdin 影響身分 | 不會。身分只來自 `host.dispatch` 的 `identity`；`--agent` 只寫進 `created_with`。 | `host.spec.ts`：`keeps argv and stdin out of the identity` |
| 被拒絕的一般命令對儲存發出呼叫 | 不會。列出、讀取、提交都是零次。 | `tests/kit.rs`：`refusing_an_archive_gives_the_reason_and_touches_nothing`、`a_query_is_asked_too`；`host.spec.ts`：`rejects with refused when beforeCommand denies` |
| JS 回傳的版本與時間被靜默截斷 | 不會。小數、負數、大於 2^53 − 1 的值都以 `internal` 拒絕，訊息指出函式與欄位。 | `host.spec.ts`：`fails when a version is not an integer`、`refuses a version past 2^53 − 1 instead of rounding it` |
| 錯誤訊息帶出文件全文 | 不會。橋接的錯誤訊息只含函式名稱、欄位名稱、鍵與版本號；JS 丟出的訊息照原樣轉交，那是宿主自己的文字。 | `port_bridge.rs` 的 `malformed` 與 `settle_call` |
| 提交後掛鉤的例外讓程序當掉 | 不會。Rust 端以 `catch_unwind` 吃掉 panic；Node 端在 invoker 的 Promise 鏈裡吃掉同步例外與被拒絕的 Promise，`dispatch` 不等它。 | `tests/kit.rs`：`a_failing_hook_changes_nothing`；`host.spec.ts`：`calls afterCommit with the events, and its failure changes nothing`、`does not wait for afterCommit, and swallows its rejected Promise` |
| 讀取程序所在的 checkout | 找到並修正（見 Open Questions 第一題）：`task done` 沒有給檔案清單時，引擎原本會探測程序的工作目錄。 | `tests/kit.rs`：`task_done_without_touched_files_is_a_clear_result` |

沒有提供執行前掛鉤時一律放行，這是規格的預設（host-kit「執行前掛鉤可以拒絕命令」），不是稽核發現。

## Migration Plan

純新增，加上一處內部的標籤來源改動。既有的 `createEngine` 使用者不用改任何東西。要退回時還原這個變更的 commit 即可；沒有資料格式的變動。

## Open Questions

兩個問題都在實作時回答了，結論如下。

- 引擎的 `task done` 在沒有 workspace、也沒有給「改了哪些檔」時的行為，在宿主套件的路徑上是什麼（今天它會嘗試探測 workspace）。
  - **結論：明確的成功，不記任何檔案。** 實作時發現：沒有 workspace 時，引擎用空的路徑當專案根目錄，探測的是程序的工作目錄。宿主的程序若在一個 git checkout 的根目錄執行，evidence 會記下那份 checkout 的髒檔。這違反「引擎不讀任何 checkout」。
  - 所以宿主套件在 `task done` 沒有給檔案清單時，交給引擎一份空的清單：任務照常勾選，沒有可歸屬的檔案所以不寫 evidence。執行前掛鉤收到的參數仍是呼叫端送來的原樣（`touchedFiles` 是 null）。
  - 這只改宿主套件的路徑；`createEngine` 的 JS 儲存路徑行為不變。
  - 測試：`crates/host/speclink-host/tests/kit.rs` 的 `task_done_without_touched_files_is_a_clear_result`，以及 `kit.rs` 的單元測試 `a_completion_without_files_names_none_instead_of_probing`。
- `repo` 留空對 evidence 的讀取端有沒有影響。
  - **結論：沒有影響。** evidence 的 `repo` 是選填欄位，留空時不寫進記錄。讀取端只有兩處：CLI 的 `remote_evidence_files` 逐欄搬運、不解讀；`TouchedRecord::all_files` 只讀 `touchedFiles`。示範用 server 的路徑照舊填 repo，不受影響。
  - 測試：`tests/kit.rs` 的 `the_evidence_entry_of_the_kit_has_no_repo`。

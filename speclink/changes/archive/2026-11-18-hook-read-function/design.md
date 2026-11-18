## Context

宿主套件（`speclink-host` 的 `kit` 模組）在每次呼叫時問一次執行前掛鉤。掛鉤的型別是 `Fn(&BeforeCommand) -> Decision`，`BeforeCommand` 帶著呼叫者（身分與範圍）、命令名稱、參數，以及流程設定寫入時的變動鍵。掛鉤被問的時機有兩種：

- 一般命令：在引擎執行之前。拒絕時儲存連讀取都沒有收到。
- 流程設定的寫入：引擎算完要寫什麼之後、提交之前。這時儲存已經收到讀取。

Node 的轉接在 `speclink-node`：`index.js` 的 `createHost` 把使用者的 `hooks.beforeCommand` 包成一個回傳 JSON 的函式，原生端（`src/host.rs` 的 `decide`）把 `BeforeCommand` 轉成 JSON 丟給它，再把答案轉回 `Decision`。每個 `dispatch`、`handleRequest`、`exportScope`、`importScope` 都在自己新開的執行緒上跑（`src/lib.rs` 的 `on_worker`），原生端每次呼叫都用同一份儲存與掛鉤組一個新的 `HostKit`（`with_kit`）。

因為每個 dispatch 有自己的執行緒，掛鉤裡等一個內層 `host.dispatch` 其實會完成。但內層呼叫會再問一次掛鉤，而且 `index.d.ts` 與 SDK 文件目前寫的是「永遠不會完成」。

引擎的命令沒有「讀取類」與「寫入類」的標記。最接近的是 HTTP 入口的對照表（`http/coverage.rs` 的 `wire_coverage`）：它對每一種命令做沒有萬用分支的 match，新增命令時不處理就編譯不過。

## Goals / Non-Goals

**Goals:**

- 掛鉤能讀到判斷需要的狀態，而且讀取本身不會再觸發掛鉤。
- 讀取的能力在結構上只能讀：不靠文件約束。
- Rust 與 Node 的宿主用同一份實作，`host.dispatch` 與 `host.handleRequest` 行為相同。
- SDK 文件對「掛鉤裡呼叫 dispatch」的描述符合實際。

**Non-Goals:**

- 不在掛鉤的參數附上變更的狀態摘要（來源討論已否決）。
- 不正式支援在掛鉤裡呼叫 `host.dispatch`（來源討論已否決）；也不去阻擋它，行為維持現狀。
- 不保證讀取器讀到的內容與外層命令讀到的是同一份。
- 提交後掛鉤 `afterCommit` 不給讀取器：它不被等待，也不能影響結果。
- 儲存函式（`list`、`read`、`commit`）說明裡同樣的那句警告不改：在 `commit` 裡等另一個會提交的 dispatch，宿主的資料庫交易確實可能互相等待。
- 不改引擎（`speclink-core`）、CLI、示範用 server 與 Speclink Desktop。

## Decisions

### D1 讀取的唯一實作是宿主套件的 HostKit::read

`HostKit` 新增公開方法 `read(caller, cmd)`：先用 D2 的判定確認命令是讀取類，不是就回傳 `invalid_argv`；是的話直接交給既有的執行路徑（`port_exec::execute_with`），用呼叫者的身分與範圍組 `ExecutionContext`，並和 `run` 一樣先去掉本機事實。它不呼叫 `ask`，也不呼叫 `watch`。讀取類的命令不會提交，所以不需要另外包一層擋提交的儲存。

Rust 的掛鉤與 Node 的掛鉤都經過這個方法（D3、D4），不各寫一份。

替代方案：在 `run_with` 加一個「略過掛鉤」的選項。否決：`run_with` 還負責自由欄位、版本比對與流程設定的特例，讀取用不到；多一個旗標會讓「哪些路徑不問掛鉤」變得不容易看清楚。

### D2 讀取類命令的判定是一張沒有萬用分支的對照

`speclink-host` 新增模組 `read_only`，提供 `is_read(&Command) -> bool`。寫法比照 `wire_coverage`：對每一種命令列出結果，不寫萬用分支，所以引擎新增命令而沒有在這裡歸類時編譯不過。

讀取類：`List`（變更與正式規格）、`Show`、`Status`、`Instructions`、`Validate`、`Analyze`、`Trace`、`Plan`（含 `PlanStrict`，也就是帶旗標的 `plan`）、`ArtifactCat`、`EvidenceShow`、`LanguageShow`、`ReviewShow`、`VerifyShow`、`DiscussList`、`DiscussShow`、`DiscussSearch`、`WorkflowConfigShow`。其餘全部是寫入類，包含「這一次剛好沒有東西要寫」的情況（例如勾一個已經勾過的任務）：判定看的是命令的種類，不是這一次的結果，這樣掛鉤的作者能事先知道哪些動詞可以用。

用一個測試固定判定與實際行為一致：對每一種歸為讀取類的命令，在有資料的記憶體儲存上執行，斷言儲存的提交次數是零。命令的樣本沿用 `speclink-host` 的 HTTP 入口測試既有的「每一種命令各一個」的清單（`tests/http_door.rs` 的 `every_command`）；把它搬到測試共用的位置讓兩個測試檔都能用，不另外維護第二份清單。

替代方案：不做判定，改用一個擋掉提交的儲存包裝，讓任何「想寫」的命令在提交時失敗。否決：拒絕會發生在命令已經讀了一輪之後，而且「勾已經勾過的任務」這種命令會通過，可用的動詞變成看資料而定。

### D3 Rust 的掛鉤多一個讀取器參數

`BeforeCommandHook` 的型別改成 `Fn(&BeforeCommand, &Reader<'_>) -> Decision`。`Reader` 借用這個 `HostKit` 與這次的呼叫者，只有一個方法 `read(&self, cmd: Command) -> Result<PortExecution, ExecError>`，內容就是呼叫 `HostKit::read`。`Reader` 沒有公開的建構方式，所以掛鉤拿不到別人的身分或範圍。

`BeforeCommand` 這個結構不變（測試會把它複製進記錄裡，它需要維持可複製、可印出）。`ask` 內部建立 `Reader` 交給掛鉤；流程設定寫入的那條路徑同樣經過 `ask`，所以自動涵蓋。

repo 內設定執行前掛鉤的地方只有測試（`speclink-host` 的五個測試檔）與 Node 的轉接；示範用 server 沒有設定執行前掛鉤。它們的閉包各加一個不使用的參數。

替代方案：把讀取器放進 `BeforeCommand` 的欄位。否決：它帶著借用與閉包，`BeforeCommand` 就不能再複製與印出，既有測試的記錄方式全部要改。

### D4 Node 的 read 由 index.js 組出來，原生端多一個不對外的方法

函式不能跟著 JSON 過橋，所以 `read` 在 JS 這一側組：

- 原生的 `Host` 類別新增方法 `read(id, displayName, scope, argv)`。它和 `dispatch` 一樣在背景執行緒上解析 argv、輸出結果，差別是經 `HostKit::read` 執行。`index.js` 的 `Host` 包裝不把它放上對外的物件，`index.d.ts` 的 `Host` 介面也不宣告它。
- `index.js` 的 `createHost` 在呼叫使用者的 `beforeCommand` 時，傳入第二個參數 `{ read }`。`read` 是一個閉包：身分與範圍取自這次的 `info`，呼叫原生的 `read`。
- 閉包帶一個開關。使用者的 `beforeCommand` 回傳的值或 Promise 完成（成功或失敗）之後，開關關上；之後再呼叫 `read` 會直接回傳被拒絕的 Promise（`invalid_argv`），不進原生端。

`read` 的回傳值與錯誤沿用 `dispatch` 既有的信封與解包函式，所以值的形狀與 Error 的 `code` 自然相同。

原生端解析 argv 之後交給 `HostKit::read`；判定是否讀取類（D2）的只有 `HostKit::read` 一處，不在原生端再判一次。它拒絕時回 `invalid_argv`，原生端靠錯誤的來源型別（`NotARead`）認出這個拒絕，不比對訊息文字，再把訊息換成 `read accepts only verbs that write nothing; '<動詞>' is not one of them`，動詞用解析出的動詞文字（例如 `new change`）。`read` 沒有內容可給，所以吃內容的動詞（有 `--stdin` 旗標的寫入，例如 `new artifact`、兩站的 `stamp`）在轉成命令之前就由原生端以同一句拒絕，而不是被要求補上內容。需要 checkout 的動詞維持既有的 `invalid_argv`。

替代方案：把 `read` 做成對外的 `host.read(...)`。否決：它等於一個不問掛鉤的 dispatch，放上對外介面會讓「每次呼叫正好問一次掛鉤」多一個例外；掛鉤以外的地方要讀資料，用 `host.dispatch` 就可以。

### D5 讀取器讀到的不是外層命令的那一份

讀取器的每次讀取都是對儲存的獨立呼叫。掛鉤讀完之後、外層命令開始讀之前，別人可能提交。這個變更不處理它，理由：

- 引擎自己的守門仍然以外層命令讀到的內容判斷。以封存為例：掛鉤看到「已蓋章」之後有人重開工單，引擎的未結工單守門會擋；內容被改，章失效判定會擋。
- 宿主自己的條件（例如「有落地記錄」）一旦成立通常不會反轉：沒有移除落地記錄的動詞。

需要嚴格一致的宿主，可以在自己的 `read` 與 `commit` 儲存函式裡處理。這一點寫進 SDK 文件。

### D6 改正 SDK 文件對掛鉤裡呼叫 dispatch 的說法

`index.d.ts` 的 `HostHooks` 說明與 `docs/sdk-node` 中英兩版「一個動詞的四個步驟」一節：

- 移除「在掛鉤裡等 dispatch 永遠不會完成」的說法。
- 改寫成：要在 `beforeCommand` 裡讀資料，用第二個參數的 `read`。在掛鉤裡等同一個宿主套件的 `dispatch` 會再問一次 `beforeCommand`；宿主的儲存自己持有鎖時會卡住。
- 封存的範例改成用 `read(['show', params.change, '--json'])` 讀出 `review`、`verify` 與 `landed` 三個欄位後決定。
- 寫明 `read` 可以用的動詞、它不再問掛鉤、它在掛鉤結束後失效，以及 D5 的限制。

`StorePort` 說明裡的同一句警告不動（見 Non-Goals）。

## Implementation Contract

**行為**

- 有執行前掛鉤的宿主：掛鉤能執行讀取類的命令，拿到和直接執行相同的結果；這些讀取不問掛鉤、不呼叫提交後掛鉤、不提交。
- 掛鉤沒有使用讀取器時，一切和今天相同，包含「拒絕時儲存連讀取都沒有收到」。

**介面與資料形狀**

- Rust：`BeforeCommandHook = Box<dyn Fn(&BeforeCommand, &Reader<'_>) -> Decision + Send + Sync>`；`Reader::read(&self, cmd: Command) -> Result<PortExecution, ExecError>`；`HostKit::read(&self, caller: &Caller, cmd: Command) -> Result<PortExecution, ExecError>`；`read_only::is_read(&Command) -> bool`；`HostKit::read` 拒絕寫入類命令時，錯誤的來源型別是 `kit::NotARead`。
- Node：`beforeCommand?(info: BeforeCommandInfo, tools: BeforeCommandTools): MaybePromise<BeforeCommandAnswer>`，`BeforeCommandTools` 是 `{ read(argv: string[]): Promise<unknown> }`。`read` 的值與 `host.dispatch` 對同一個 argv 的值相同；`read` 不接受 stdin 與自由欄位。
- `BeforeCommandInfo`、`BeforeCommandAnswer`、`AfterCommitInfo`、`Host` 介面都不變。

**失敗模式**

- 非讀取類的命令：`invalid_argv`，訊息點名動詞；儲存沒有收到任何呼叫。
- 讀取的對象不存在等命令層的失敗：和直接執行相同的錯誤碼與訊息，回給掛鉤；外層呼叫不因此結束。
- Node 的掛鉤沒有接住 `read` 的拒絕：依既有規則，外層呼叫以 `internal` 結束，訊息指出 `beforeCommand`。
- 掛鉤結束後呼叫 `read`：`invalid_argv`，不進原生端。
- 儲存層的失敗（`unavailable`、`internal`）：和 `dispatch` 相同的錯誤碼，回給掛鉤。

**驗收**

- `cargo test -p speclink-host`：新增的測試涵蓋 host-kit 規格 delta 的七個情境，以及 D2「讀取類的命令提交次數是零」的測試；既有測試全部通過。
- `npm --prefix crates/adapters/speclink-node run build` 之後 `npm --prefix crates/adapters/speclink-node test`：新增的測試涵蓋 node-sdk 規格 delta 的七個情境；既有測試全部通過。
- `node --test scripts/*.test.mjs scripts/*/*.test.mjs` 通過。
- `index.d.ts` 與 `docs/sdk-node` 兩版不再出現「never finish」與「永遠不會完成」用在掛鉤上的說法。

**範圍**

- 範圍內：`speclink-host` 的 `kit` 與新的 `read_only` 模組、該 crate 的測試；`speclink-node` 的 `src/host.rs`、`index.js`、`index.d.ts` 與測試；`docs/sdk-node` 中英兩版。
- 範圍外：`speclink-core` 的命令與輸出；`show --json` 的欄位（認領的人由另一個變更加）；`afterCommit`；儲存函式的警告；CLI、示範用 server、Speclink Desktop。

## Risks / Trade-offs

- [Rust 的掛鉤型別多一個參數，是原始碼層級的不相容] → repo 內只有測試與 Node 的轉接受影響，同批更新；套件尚未發版，沒有外部使用者要遷移。
- [回歸對照] → 引擎的 golden 與 CLI 測試不涉及掛鉤，不需要更新；`speclink-host` 既有的掛鉤測試（正好問一次、拒絕時儲存沒有收到讀取）保留不改斷言，只改閉包的參數，用來證明沒有使用讀取器時行為不變。
- [引擎新增讀取類的命令時忘了在 D2 歸類] → 沒有萬用分支，編譯不過；歸錯類（把會寫的當成讀取類）由「提交次數是零」的測試抓到。
- [掛鉤每次都讀，會讓每個命令多幾次對儲存的讀取] → 這是宿主自己的選擇；文件的範例只在命令是 `archive` 時才讀。
- [跨平台] → 沒有檔案系統或路徑的行為；Node 的測試在 CI 的三個平台都跑。
- [掛鉤把 `read` 存起來之後再用] → D4 的開關讓它在掛鉤結束後失效，並有測試。

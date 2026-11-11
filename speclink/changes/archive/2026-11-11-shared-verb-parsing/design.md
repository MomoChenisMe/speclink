## Context

引擎的命令（`speclink-core` 的 `Command`）有 43 種。把動詞文字轉成命令的程式有兩份：

- **CLI**（`speclink-cli`）：`main.rs` 的 `Cli` 與 `Commands`，加上 `src/verbs/` 底下各家族的 clap 型別。共 34 個頂層子指令、82 個末端動詞。其中會建出命令的動詞涵蓋 33 種命令。這個 crate 只有執行檔，型別都是 `pub(crate)`。解析、開啟儲存、讀 stdin、建立命令、印出結果寫在同一批 `cmd_*` 函式裡。
- **Node SDK**（`speclink-node`）：`run_dispatch` 依 argv 的第一個字分流到 `verb_list`、`verb_status`、`verb_new`、`verb_claim`、`verb_station`，各自用手寫的 `parse_argv` 取旗標。只認得 9 個動詞。

兩份的行為不同：`parse_argv` 把不認得的 `--x` 當成旗標收下、忽略多出來的位置參數、不接受 `--a=b` 與短旗標。

CLI 有些動詞不建立命令：`drift`、`change rank`、`review prepare`、`review scope`、`verify scope` 需要 checkout 裡的事實，直接呼叫引擎或 Host 的函式；`review`／`verify` 的 add-round、show、stamp、discard 在本機模式直接呼叫 `core::station`（對應的命令只有 Node 與 server 在用）；`claim` 在本機模式被拒絕。

依賴方向：`speclink-node` 不依賴 `speclink-cli`，也不依賴 clap。`speclink-core` 與 `speclink-host` 沒有 clap。

限制：

- CLI 的 `--help` 文字、人眼輸出、`--json`、exit code 是既成契約，這個變更不能改動它們。
- 引擎不讀 stdin、不呼叫 git、不解讀終端機。這些留在呼叫端。
- 這個變更不搬輸出（下一個變更 shared-verb-output 處理），也不把流程設定變成引擎命令（變更 workflow-config-commands 處理）。

## Goals / Non-Goals

**Goals:**

- 動詞文字的詞彙（子指令、旗標、預設值、說明文字）只有一份定義。
- 「解析結果轉成引擎命令」只有一個函式，CLI 與 SDK 都呼叫它。
- 解析能在任何程序裡安全呼叫：不結束程序、不印出、不讀環境。
- SDK 對 argv 的接受與拒絕和 CLI 相同。

**Non-Goals:**

- 不增加 SDK 能用的動詞。這個變更結束時仍是 9 個。
- 不搬 render 函式，不改任何輸出。
- 不改 CLI 本機模式執行審查與驗證動詞的路徑（它仍直接呼叫 `core::station`）。把它收進命令層不在這次的範圍。
- 不把 workflow-config、config、init、auth、schema、completion 這些家族搬進共用的 crate。
- 不為搬任務（`TaskMove`）新增動詞文字。它只有線上的請求，沒有 CLI 的寫法。
- 不改 CLI 遠端模式送出的請求。

## Decisions

### D1: 新增函式庫 crate speclink-verbs

位置 `crates/adapters/speclink-verbs`，套件名稱 `speclink-verbs`，依賴 `speclink-core`、clap（derive）、`serde` 與 `serde_json`。`speclink-cli` 與 `speclink-node` 新增對它的依賴。

它屬於 adapter 層的共用函式庫：內容是「CLI 的動詞詞彙」這個對外契約的唯一實作落點。領域邏輯仍在 `speclink-core`，它只做文字到命令的轉換。

- **否決：放進 `speclink-core`** — 引擎不帶 clap，也不知道 CLI 的詞彙。
- **否決：把 `speclink-cli` 改成同時有函式庫** — Node 的綁定會連 CLI 的遠端用戶端、終端機顏色、設定檔處理一起依賴進來。
- **否決：不用 clap，手寫一份表格式的解析給兩邊用** — `--help`、completion、錯誤訊息都要重做，CLI 的既有輸出守不住。

### D2: 十一個家族的 clap 型別原樣搬移並公開

從 `speclink-cli` 的 `src/verbs/` 搬到 `speclink-verbs` 的 `src/families/`：query、checks、documents、trace、instructions、new、progress、lifecycle、plan、discuss、station。型別與欄位改成 `pub`，clap 的屬性（名稱、說明、預設值、互斥關係）一個字都不改。說明文字用到的靜態字串跟著搬。

`speclink-verbs` 另外定義 `SpecVerb`：一個 clap 的 `Subcommand` 列舉，每個變體是一個家族的頂層子指令。CLI 的 `Commands` 用 `#[command(flatten)]` 把 `SpecVerb` 併進去，其餘的頂層子指令（init、update、link、unlink、auth、schemas、templates、feedback、schema、config、workflow-config、completion、demo）照舊留在 CLI 自己的變體。

flatten 會把 `SpecVerb` 的子指令排在一起，原本它們和 CLI 自己的子指令交錯。`--help` 的順序由 CLI 的一張頂層順序表（`HELP_ORDER`）還原：CLI 建出指令樹時，依這張表替每個頂層子指令設定 `display_order`；`SpecVerb` 不帶任何版面資訊（品質關卡定案）。補全腳本依子指令加入的順序產生，這個順序 display_order 改不到，所以 `completion generate` 的腳本內部順序會變；補全的功能不變（apply 期定案，記在 proposal 的相容性影響）。

家族整個搬，不依「有沒有引擎命令」拆開。理由：`review` 底下同時有建立命令的末端（add-round）與不建立的（prepare），clap 的樹拆不開；而且詞彙只該有一份。

### D3: 解析入口 parse 不結束程序

`speclink_verbs::parse(argv)` 用 clap 的 `try_parse_from`，回傳 `ParsedVerb` 或 `ParseError`。

- 它不印出、不呼叫 `exit`、不讀環境變數。
- `ParseError` 帶：分類（詞彙錯誤，或使用者要求說明文字）、不含 ANSI 色碼的訊息、clap 原本要用的 exit code。
- `ParsedVerb` 包住 `SpecVerb`，並回答兩件事：`stdin_policy()` 與這個動詞的穩定名稱（例如 `new artifact`、`review stamp`）。蓋章的 `--stdin` 不在 `SpecVerb` 裡：`parse` 建出指令樹時替兩個 stamp 加上隱藏的 `--stdin`，有沒有帶由 `ParsedVerb` 記住（見 D6）。

CLI 不呼叫這個入口來解析整行指令——它仍用自己的 clap 指令樹 `Cli`（依 `HELP_ORDER` 排好順序後解析，印出與結束的行為和 `Cli::parse()` 相同），因為它還有十三個不屬於 `SpecVerb` 的家族，而且 `--help` 與錯誤時的行為要維持 clap 原本的樣子。CLI 拿到 `Commands` 裡的 `SpecVerb` 之後，和 SDK 走同一個 D4 的轉換函式。兩邊共用的是同一份 clap 定義，所以接受與拒絕的結果相同。共用的範圍是讀寫規格的動詞（`SpecVerb`）：頂層的 `--version`、CLI 自己的十三個家族、頂層 `--help` 的文字與拼字建議不在共用範圍內，兩個入口的結果可以不同（品質關卡定案）。

### D4: 轉成引擎命令的單一函式 into_command

`ParsedVerb::into_command(inputs)` 回傳三種結果之一：

- `Ok(Command)`：這個動詞對應的引擎命令。
- `LocalOnly { verb, reason }`：這個動詞沒有引擎命令，只能在 CLI 那一端執行，`reason` 說明原因。固定的清單：`drift`、`change rank`、`review prepare`、`review scope`、`verify scope`（需要 checkout，`NeedsCheckout`），以及帶 `--skill` 的 `instructions`（印的是內建在 CLI 的技能文字，`CliSkillText`）。
- `Err(CommandError)`，分類 `invalid_argv`：旗標的組合不合法，或需要的 stdin 沒有給。

函式裡有一個不帶萬用分支的 `match`，涵蓋 `SpecVerb` 的每個末端。新增末端而沒有處理時編譯不過。`SpecVerb` 與每個家族的參數型別都實作 `ToCommand`：`SpecVerb` 的實作就是這個函式，各家族的實作是它的一個分支；CLI 對不需要呼叫端資料的動詞，用同一個輔助函式取得命令（品質關卡定案）。

CLI 今天建出命令的 33 種之外，這個函式另外涵蓋 `claim` 與審查、驗證的八種（add-round、show、stamp、discard 各兩站），所以它涵蓋 43 種命令裡的 42 種。`TaskMove` 沒有動詞文字。

CLI 的 `cmd_*` 裡原本就地寫的 `Command::… { … }` 全部改成呼叫這個函式。CLI 本機模式的審查與驗證動詞今天不經命令，維持不變：它們用解析後的參數走原本的 `station_fs`。

### D5: 呼叫端提供的資料 VerbInputs

`VerbInputs` 是一個全部欄位都選填的結構，預設值是「沒有提供」：

| 欄位 | 用在哪個命令 | CLI 今天從哪裡取得 |
|---|---|---|
| `stdin` | new artifact、discuss context／add-round／conclude、review／verify add-round 與 stamp | `common::read_stdin`、`common::read_stdin_content` |
| `worktrees` | `List` 的 `worktrees` | `common::worktree_facts` 經 `speclink_host::worktree::payload_objects` |
| `touched_files`、`head_commit` | `TaskDone` | 遠端模式的 `core::tasks::git_changed_files` 與 `head_commit`；本機模式不給，由引擎探測 |
| `ranks` | `Plan` 的 `ranks` | 遠端模式由 server 的回應；本機模式不給 |

實作時逐一對照 CLI 的 `cmd_*`：命令的欄位值若不是來自 argv，就一定來自 `VerbInputs` 的某個欄位。對照的結果列在 `inputs.rs` 的文件註解。上表之外發現新的來源時，加一個欄位，不在 `into_command` 裡讀取。

`into_command` 不讀 stdin、不讀檔、不呼叫 git、不讀環境變數。蓋章用的 JSON（scope 與 missing）由 `stdin` 欄位帶入，在這個函式裡解析，格式不對時回傳 `invalid_argv`。

執行情境（操作者、repo、政策覆寫）不是命令的一部分，不在這裡。

### D6: stdin 的規則由解析結果回報

`ParsedVerb::stdin_policy()` 回傳三種之一：

- `Never`：這個動詞不讀 stdin。
- `WhenFlagged`：帶 `--stdin` 才讀（new artifact、蓋章）。

CLI 的蓋章沒有 `--stdin`（CLI 自己從 checkout 算範圍），SDK 的蓋章要它帶 JSON。`SpecVerb`——CLI flatten 進去的那份——不帶這個旗標。`parse` 只有 SDK 呼叫，它建出指令樹時替 `review stamp`／`verify stamp` 加上隱藏的 `--stdin`（`mut_subcommands`，不改子指令順序），並由 `ParsedVerb` 記住有沒有帶：`stdin_policy()` 對蓋章回報 `WhenFlagged`，`into_command` 依它解析蓋章 JSON、填進命令的 scope 與 missing。CLI 收到 `--stdin` 時照舊是 clap 的解析錯誤（exit 2），說明文字與補全都不受影響，CLI 不需要另外拒絕或重建補全用的指令樹（品質關卡定案，取代 apply 期「共用定義加隱藏旗標、CLI 拒絕並從補全拿掉」的做法）。
- `FlagOrPiped`：帶 `--stdin`，或 stdin 不是終端機時讀（discuss 的 context、add-round、conclude，以及審查與驗證的 add-round）。

CLI 依這個結果決定要不要讀 stdin，再把內容放進 `VerbInputs`。「是不是終端機」由 CLI 判斷，不在共用的 crate 裡。SDK 沒有終端機：呼叫端有給 `stdin` 就當成管線輸入。`WhenFlagged` 的動詞給了 `stdin` 卻沒帶 `--stdin` 時，以 `invalid_argv` 拒絕，不靜默丟掉內容（品質關卡定案，記在 proposal 的相容性影響）。

### D7: Node SDK 的 dispatch 改用共用的解析

`run_dispatch` 改成：`parse(argv)` → 蓋章先檢查宿主 Store 有沒有蓋章需要的方法 → 依 stdin 的規則決定要不要放進 stdin → `into_command(inputs)` → `LocalOnly` 回報只能在 CLI 執行 → 依命令執行並組結果。允許的 9 個動詞就是組結果那個 `match` 的分支，沒有另一份清單（品質關卡定案）。`LocalOnly` 要排在「不支援」之前，`drift` 這類動詞才拿得到「需要 checkout」的訊息。

- 移除 `parse_argv` 與 `Argv`。
- 9 個動詞組結果的程式不動。
- 不在這 9 個裡的命令：回傳 `invalid_argv`。轉換本身就失敗的（`archive` 帶多個 change 或 `--all`）回報轉換的錯誤。`review`／`verify` 底下不支援的子動詞維持今天的訊息（列出 add-round 與 stamp），因為正式規格與測試釘住它。
- `LocalOnly` 的動詞：回傳 `invalid_argv`，訊息依 `LocalOnly` 帶的原因說明：需要 checkout，或（`instructions --skill`）技能文字內建在 CLI。
- `ParseError` 轉成 `invalid_argv`，訊息用 clap 的文字（不含 ANSI）。使用者要求說明文字（`--help`）也回傳 `invalid_argv`，訊息是說明文字。
- `claim` 今天繞過命令、直接呼叫 JS 儲存的方法，這段不動；解析改用共用的。
- `status` 在沒有任何變更時的特殊輸出不動。

### D8: 搬移前後逐一比對 --help

沒有既有的測試列出每個動詞的說明文字。搬移是否原樣，用比對確認：動手前，用 main 建出的 CLI 對十一個家族的每個子指令與末端動詞各跑一次 `--help`，把輸出存成基準（放在暫存位置，不進 repo）；每搬完一批，用新建出的 CLI 再跑一次並比對，必須逐位元相同。頂層的 `speclink --help` 一併比對，確認子指令的順序沒變。

- **否決：新增一個永久的說明文字快照測試** — 之後每次改說明文字都要更新快照；這次要守的只是「搬移沒有改到字」。

## Implementation Contract

**行為**

- CLI：每個指令的接受與拒絕、`--help`、stdout、stderr、exit code 與變更前逐位元相同。
- SDK：9 個動詞成功時的回傳值不變。不認得的旗標與多出來的位置參數回傳 `invalid_argv`，訊息與 CLI 的解析錯誤相同。`--name=value` 被接受。
- 新增動詞時，在 `speclink-verbs` 加 clap 定義與 `into_command` 的一個分支，CLI 與 SDK 同時得到解析。

**介面**

- `speclink_verbs::{parse, ParsedVerb, ParseError, SpecVerb, ToCommand, VerbInputs, StdinPolicy, CommandTarget, LocalOnlyReason}`，其中 `CommandTarget` 是 `Command` 或 `LocalOnly`（帶 `LocalOnlyReason`）。
- `speclink_verbs::families::*`：各家族公開的 clap 型別。
- `speclink-node` 的 JS 介面（`dispatch(argv, options)`）不變。

**失敗的樣子**

- 詞彙錯誤：`ParseError`，訊息是 clap 的錯誤文字去掉 ANSI。SDK 轉成 `invalid_argv`。
- 旗標組合不合法、蓋章的 JSON 格式不對、需要 stdin 卻沒有：`CommandError`，分類 `invalid_argv`。
- 沒有引擎命令的動詞：`CommandTarget::LocalOnly`，不是錯誤；由呼叫端決定怎麼回應。

**驗收**

- `cargo test -p speclink-verbs`：表格測試涵蓋每個末端動詞——給 argv 與輸入，得到預期的命令或 `LocalOnly`；每個動詞的 stdin 規則；`parse` 對錯誤的 argv 不 panic、不結束程序。
- `cargo test -p speclink-cli` 全部通過，含 `remote_verb_parity`、`mode_dispatch`、`no_raw_wire_json`、說明文字的既有測試。
- D8 的比對：十一個家族與頂層的 `--help` 逐位元相同。
- `npm --prefix crates/adapters/speclink-node run build` 後 `npm --prefix crates/adapters/speclink-node test` 全部通過，含新增的案例（不認得的旗標被拒絕、`--name=value` 被接受、錯誤訊息與 CLI 相同）。
- 在 `speclink-cli` 的 `src/verbs/` 搜尋不到就地建立的 `Command::` 變體（遠端分支與 `station_fs` 用到的引擎型別不算）；在 `speclink-node` 搜尋不到 `parse_argv`。

**範圍**

- 範圍內：新 crate；CLI 十一個家族改用共用的型別與轉換；SDK 的解析。
- 範圍外：任何 render 函式與輸出；流程設定；CLI 遠端分支的請求；`speclink-core` 的命令；server；使用者文件。

## Risks / Trade-offs

- [回歸對照] 搬動約十一個檔的 clap 定義，一個屬性改到就會改變說明文字或預設值 → D8 的逐位元比對，加上 CLI 既有的整合測試（`remote_verb_parity` 比對本機與遠端的輸出與請求）。golden（`render_golden`）在引擎裡，這個變更不碰。
- [跨平台] 解析不碰路徑與換行；`--help` 的比對在同一台機器上做前後對照，不跨平台比。Windows 的 CI 會跑同一批 CLI 測試。
- [守門的掃描面] `no_raw_wire_json` 用逐行的允許清單掃 `src/verbs/`，搬走的行會讓清單對不上 → 同批更新清單，並確認它掃描的檔案集合仍涵蓋留在 CLI 的遠端分支。
- [SDK 變嚴格] 原本被忽略的旗標現在被拒絕，既有的宿主會壞 → 列為相容性影響；錯誤訊息指出是哪個旗標。
- [Node 的二進位變大] clap 進到 napi 的產物 → 接受。換來的是詞彙只有一份。
- [flatten 改變子指令順序] `#[command(flatten)]` 併進去的子指令在說明文字裡的位置會和原本不同 → D8 比對頂層說明文字；順序用 CLI 的頂層順序表還原。
- [CLI 的審查與驗證仍走舊路徑] 本機模式這八個動詞不經命令，`into_command` 對它們的分支只有 SDK 在用 → 表格測試涵蓋它們；收進命令層留待之後。

## Open Questions

- `VerbInputs` 的欄位是否只有 D5 的四類，要等實作時逐一對照 `cmd_*` 才確定。
- 正式規格 command-runtime 的覆蓋表列了 `drift`，但引擎沒有對應的命令。這是既有的落差，不在這次處理；`into_command` 把它列為 `LocalOnly`。

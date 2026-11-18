## Context

**清單。** 引擎的 `List` 命令（`speclink-core`）回傳變更與正式規格兩個區段。變更的每一項由 `lifecycle/listing.rs` 的 `changes_json_with` 組出：名稱、狀態、任務數、摘要、delta 的 capability、建立與修改時間、自由欄位等。它刻意不帶關卡狀態；`listing.rs` 有兩個測試固定「metadata 有章時輸出不變」，對應 review-station 與 verify-station 各一條正式需求。

關卡狀態、分支、落地與認領目前的讀取面：

- `show --json`（單一變更）：`review`、`verify`（引擎的 `station` 模組算出，物件由 `StationStatus::to_json` 產生）、`branch`、`landed`；變更 claim-release-and-output 會再加 `claimed`。
- 線上的 `GET /changes`：每一項有 `review`、`verify`、`branch`、`landedCommit`、`claimedBy`（claim-release-and-output 之後另有 `claimedById`、`claimedAt`）。這些是宿主套件的路由在 `List` 命令的結果之外補上的。
- CLI 連到遠端專案時，`list` 由 `speclink-cli` 的 `verbs/query.rs` 把 `GET /changes` 的每一項轉成引擎的清單項（`to_list_change_json`），再交給同一支輸出函式。

**路徑行。** 三個動詞的人眼輸出有 `Path:` 行：`new change`、`discuss promote`（外加一行 propose 提示）、`discuss new`。行的內容來自命令結果裡的路徑。CLI 的遠端分支自己組輸出的資料，對前兩個動詞不給路徑，所以不印。Node SDK 的 `dispatch` 則是把命令結果交給 `speclink-verbs` 的統一入口 `render(parsed, outcome, options)`；這個入口不知道呼叫端有沒有本機檔案，所以 `createHost` 的 `dispatch` 會印出儲存顯示用的路徑（例如 `speclink/changes/demo`）。

`RenderOptions`（`json`、`color`）在 `speclink-verbs` 與 CLI 裡有二十幾處手寫的字面值，大多在測試。

Agent 工具的說明由 `speclink-verbs` 的 `tool.rs` 產生。

## Goals / Non-Goals

**Goals:**

- 一次呼叫就能拿到全部變更的關卡、認領、分支與落地狀態，三種執行方式輸出相同。
- 不帶新旗標時，`list --json` 逐位元不變，呼叫儲存的次數也不變。
- 內嵌宿主的 Agent 不再看到不存在的路徑行，也被告知 JSON 的 `path` 不是檔案。

**Non-Goals:**

- 不改 `list --json` 的預設輸出，不改它的人眼輸出。
- 不改線上格式，不新增端點。
- 清單不判斷「蓋章後檔案是否改過」：那需要 checkout。
- 清單的 `landed` 不帶時間與回報的人：線上的清單沒有這兩個欄位，要看請用 `show --json`。
- 不從 JSON 輸出移除 `path`；不改 `new artifact` 的成功行（它印的是寫入的文件，遠端的 CLI 也印）。
- 不改 CLI 的遠端輸出；`discuss new` 在遠端照舊印線上路徑。
- 已封存的清單、討論的清單不在這次範圍。

## Decisions

### D1 清單的詳細資料由引擎的 List 命令在 detail 為真時算出

`Command::List` 新增 `detail: bool`。為真時，`changes_json_with` 對每一個 metadata 正常的變更多填五個選填欄位；為假時完全不碰它們，也不多讀任何文件。清單項的型別 `ListChangeJson` 新增這五個欄位，全部是「沒有值就不序列化」，所以 `detail` 為假時序列化的結果與現在逐位元相同。

各欄位的來源和 `show` 相同，不另寫一份判斷：

- `review`、`verify`：呼叫 `show` 用的同一個關卡狀態函式；序列化用 `StationStatus::to_json`；狀態是「沒有記錄」時不填。
- `claimed`：用 claim-release-and-output 引入的認領記錄型別與同一個建立函式。
- `branch`、`landed`：讀 `show` 用的同一份落地記錄；清單只輸出 `commit`。

`--detail` 的檢查（要有 `--json`、不能和 `--specs` 並用）放在共用的動詞解析（`speclink-verbs`），所以 CLI 與宿主套件給出同一個用法錯誤。

替代方案：讓宿主套件在 `List` 的結果之外補欄位（像 HTTP 路由那樣）。否決：本機專案與 `createEngine` 不經過宿主套件，會變成兩份實作。

### D2 連到遠端的 CLI 用既有的清單回應組出相同的鍵

`to_list_change_json` 在 `--detail` 時多轉五個欄位：

- `review`、`verify`：線上的關卡物件轉成引擎的關卡狀態，用遠端 `show` 已經在用的同一個轉換。
- `claimed`：`claimedBy`、`claimedById`、`claimedAt`。
- `branch`：`branch`。`landed`：`landedCommit`。

不帶 `--detail` 時這五個欄位不轉，輸出不變。回應缺欄位（舊版 Server）時對應的鍵不出現。不新增請求。

`speclink-cli` 的 `remote_verb_parity` 測試加一組對照：同樣的資料在本機與遠端的 `list --json --detail` 輸出相同。

### D3 呼叫儲存的次數：不帶 detail 時不變

`speclink-host` 的 `port_cost` 測試固定各命令對儲存的呼叫數。這次的要求：

- 不帶 `--detail`：既有的斷言一個都不改。
- 帶 `--detail`：關卡狀態需要知道兩張工單在不在。實作時把工單的讀取併進每個變更既有的讀取批次（一次讀取帶多個鍵），並在 `port_cost` 新增一個斷言記下帶 `--detail` 時的呼叫數，讓之後的改動看得到成本的變化。帶 `--detail` 的呼叫數比不帶時多是可以接受的；這是使用者明確要求更多資料的代價。

### D4 內嵌用的輸出入口不給路徑行

`speclink-verbs` 的統一入口拆成一個內部函式加兩個公開的入口：

- 既有的 `render(parsed, outcome, options)`：行為不變。
- 新的 `render_embedded(parsed, outcome, options)`：三個動詞的人眼輸出不帶路徑。作法是在組輸出資料時把路徑設成「沒有」：`new change` 沿用既有的 `NewChangeLines { path: None, .. }`；`discuss promote` 沿用既有的「沒有路徑就不印 Path 與提示行」；`discuss new` 的輸出資料把路徑改成選填，沒有時不印那一行。

`speclink-node` 的 `createHost` 路徑（`dispatch` 與變更 hook-read-function 引入的讀取）改呼叫 `render_embedded`；`createEngine` 的路徑繼續呼叫 `render`。JSON 形式兩個入口相同。

不在 `RenderOptions` 加欄位，因為它有二十幾處手寫的字面值，加欄位要全部改一遍，而需要這個差別的只有一個呼叫端。

替代方案：讓宿主套件在執行命令之後把結果裡的路徑清掉。否決：結果裡的路徑同時是 JSON 輸出的 `path`，清掉會連 JSON 一起改。

### D5 Agent 工具的說明加一句 path 的解釋

`tool.rs` 產生的說明在動詞清單之前加一句英文：`A "path" in a result is the document's key in the store, not a file you can open. To read a document, use "artifact cat", "show" or "discuss show".`。`inputSchema` 與動詞清單的產生方式不變。

### D6 使用者文件

- 動詞契約文件（中英）：`list` 的旗標加 `--detail`，並用一張表列出五個鍵、出現條件與來源；註明形狀與 `show --json` 相同、清單的 `landed` 只有 `commit`。
- Node SDK 文件（中英）的 Agent 工具一節：說明經 `host.dispatch` 的人眼輸出不印三個動詞的路徑行、JSON 的 `path` 是儲存的鍵；總覽畫面的範例改用 `list --json --detail`。
- 專案能力狀態（中英）：對應的能力列提到 `--detail`。

## Implementation Contract

**行為**

- `speclink list --json --detail`：每一項多出有記錄才出現的 `review`、`verify`、`claimed`、`branch`、`landed`。三種執行方式結果相同。
- `speclink list --json`：逐位元不變，呼叫儲存的次數不變。
- `host.dispatch` 的人眼輸出：`new change`、`discuss promote`、`discuss new` 不含 `Path:` 行（`discuss promote` 連同提示行）。CLI 與 `createEngine` 不變。
- `agentTool.description` 含 D5 的那一句。

**介面與資料形狀**

- CLI：`speclink list [--specs] [--changes] [--sort <order>] [--json [--detail]]`。
- 清單項的新鍵：`review?: object`、`verify?: object`（同 `show --json`）、`claimed?: { by: string, byId?: string, at?: string }`、`branch?: string`、`landed?: { commit: string }`。
- 引擎：`Command::List` 的 `detail: bool`；`speclink-verbs` 的 `render_embedded`，簽名與 `render` 相同。

**失敗模式**

- `--detail` 不帶 `--json`，或與 `--specs` 並用：用法錯誤。CLI 非零結束、stderr 有訊息、stdout 沒有輸出；宿主套件回 `invalid_argv`。
- metadata 損壞的變更：照既有行為列出，不帶新鍵，不讓指令失敗。
- 舊版 Server 的回應缺欄位：對應的鍵不出現，不失敗。
- 離線、認證失效：與不帶 `--detail` 的 `list` 相同。

**驗收**

- `cargo test -p speclink-core`：`detail` 為真時五個欄位的測試；`detail` 為假時 `listing.rs` 既有的兩個固定測試不改斷言並通過；命令名稱與參數的 golden 刻意更新。
- `cargo test -p speclink-verbs`：解析的三個情境（接受、缺 `--json`、與 `--specs` 並用）；清單 JSON 的輸出；`render_embedded` 對三個動詞的輸出與 `render` 的輸出逐位元不變；Agent 工具說明的測試。
- `cargo test -p speclink-host --test port_cost`：既有斷言不改；新增帶 `--detail` 的呼叫數斷言。
- `cargo test -p speclink-cli --test it remote_verb_parity`：本機與遠端的對照。
- `npm --prefix crates/adapters/speclink-node run build` 後 `npm --prefix crates/adapters/speclink-node test`：`host.dispatch` 的三個動詞不含路徑行、`createEngine` 照常含路徑行、`list --json --detail` 的結果、`agentTool.description`。
- `node --test scripts/*.test.mjs scripts/*/*.test.mjs` 通過。

**範圍**

- 範圍內：上面列的命令參數、清單項、解析、輸出入口、Node 的呼叫端、遠端 CLI 的轉換、Agent 工具說明、文件。
- 範圍外：線上格式、宿主套件的路由、Speclink Desktop、技能的內容、`show` 的輸出、已封存與討論的清單。

## Risks / Trade-offs

- [`list --json` 的回歸] → 新欄位全部是沒有值就不序列化；`listing.rs` 既有的兩個固定測試與 `speclink-verbs` 的清單輸出測試不改斷言；CLI 的 golden 與整合測試照跑。
- [這個變更用到 claim-release-and-output 引入的認領記錄型別與線上的 `claimedById`、`claimedAt`] → 排在它之後實作；用 `speclink change depends` 記下相依。
- [hook-read-function 讓 `createHost` 多一條讀取路徑] → 那條路徑同樣改用 `render_embedded`；兩個變更的先後由相依記錄保證（經 claim-release-and-output 間接排在它之後）。
- [帶 `--detail` 時對儲存的呼叫數增加] → D3：併進既有的讀取批次，並用 `port_cost` 的新斷言記下數字。
- [宿主的 Agent 可能已經依賴路徑行] → 套件尚未發版；路徑行指向不存在的位置，本來就不能用。文件寫明改用 JSON 的鍵與讀取動詞。
- [跨平台] → 路徑行在 Windows 上原本含反斜線，移除後這三個動詞在內嵌時的輸出不再有平台差異；其他輸出沒有路徑相關的變動。
- [說明文字多一句會讓每次工具定義多幾十個 token] → 一句話；換來模型不再嘗試開啟不存在的檔案。

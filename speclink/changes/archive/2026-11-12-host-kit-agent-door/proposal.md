## Why

把 speclink 內嵌進自己後端的宿主，今天只有一條路：用 Node SDK 的 `createEngine` 加一個 31 個方法的 JS 儲存。這條路有四個缺口。

- **身分只能在建立時給一次。** 一個引擎實例綁一個操作者。宿主有很多使用者，就得為每個人建一個實例。
- **寫入沒有交易，也沒有版本比對。** 一個動詞的多筆寫入分開送到 JS 儲存，中途失敗會留下半套；兩個人同時寫會互相覆蓋。
- **事件被丟掉。** 引擎產生的領域事件在這條路徑上不會交給宿主，宿主做不了即時通知。
- **宿主沒有地方把關。** 沒有任何一個點能讓宿主說「這個人現在不能做這件事」。

前五個變更已經把地基鋪好：宿主實作三個函式的儲存接口（列出、讀取、提交），引擎經它讀寫並把一個動詞的寫入與事件收成一次提交；動詞文字的解析與輸出各只有一份；流程設定的讀寫是引擎命令。這個變更把它們組成宿主能用的東西：**宿主套件**，以及它的第一個門——給宿主自己的 Agent 工具用的門。

- **目標使用者**：把 speclink 內嵌進自己後端的整合者。間接受益的是在宿主的 AI Agent 裡做討論與提案的 PO、PM 與顧問。
- **使用情境**：宿主的 Agent 呼叫一顆工具，工具把動詞文字交給同一個程序裡的宿主套件。對應 discuss、propose、ingest、config 這幾支技能在「沒有本機規格檔」的形態下的執行。
- **既有規格的對照**：node-sdk 規定 `createEngine` 與 `dispatch`，其中「一個實例一個身分」只談 `createEngine`；host-runtime 規定引擎經儲存接口執行，沒有規定誰在執行前把關；store-port-contract 規定三個函式與合格測試，只有 Rust 的介面；command-runtime 沒有規定命令的名稱。沒有任何正式規格涵蓋掛鉤、每次呼叫的身分、JS 版的儲存接口。

Source discussion: embedded-host-needs（cut 6）。

## What Changes

- `speclink-host` 新增**宿主套件**：建立時收儲存接口與選填的掛鉤；之後每次呼叫收「誰」「哪個範圍」「哪個命令」。一個宿主套件由全部使用者共用。
- 每次呼叫的身分有兩個欄位：改名後也不變的識別，以及顯示名稱。識別進入提交的記錄；顯示名稱進入文件裡的蓋章欄位（建立者、審查者、驗證者）。
- **claim 的擁有者以識別判斷**：宿主套件路徑上，引擎的執行情境多帶識別（`actor_id`）。`claim` 除了寫入顯示名稱（`claimed_by`），也寫入識別（`claimed_by_id`）；判斷是不是同一個擁有者時，兩邊都有識別就只比識別——識別不同、顯示名稱相同的兩個人是不同的擁有者，改了顯示名稱的人仍認得自己的 claim。沒有識別的入口（CLI、示範用 server、`createEngine`）照舊比顯示名稱。
- 兩個掛鉤，都是選填，用一個具名的集合傳入：
  - **執行前**：收到身分、範圍、命令名稱、命令的參數。可以放行或拒絕。拒絕時這次呼叫以 `refused` 結束，訊息是宿主給的原因，儲存沒有任何變動。沒有提供時一律放行。
  - **提交後**：提交成功後收到身分、範圍、命令名稱、新的範圍版本號、這次提交的事件。它只能旁觀：它的失敗不影響呼叫的結果，也不保證送達。
- 流程設定的寫入命令，執行前掛鉤多收到一個欄位：這次會變動的鍵（例如 `tdd`、`rules.proposal`）。
- 第三個介入點是儲存接口的提交函式本身。它已經存在，這個變更不加新介面：宿主在提交裡看得到全部寫入與事件，需要「至少送達一次」的宿主在同一個資料庫交易裡把事件寫進自己的表。
- `speclink-core`：每個引擎命令有一個穩定的名稱（例如 `new-change`、`archive`、`workflow-config-edit`），以及一份能轉成 JSON 的參數。提交記錄裡的命令標籤改用這個名稱。
- `speclink-node` 新增四樣對外的東西：
  - `createHost({ store, hooks })`：建立宿主套件。`store` 是實作三個函式的 JS 物件，函式可以回傳值或 Promise。
  - `host.dispatch({ identity, scope, argv, stdin })`：Agent 工具的門。動詞文字的解析與輸出用共用的那一份；回傳值與錯誤的形狀和 `engine.dispatch` 相同。
  - `runStoreConformance(harness)`：對宿主的 JS 儲存跑合格測試，和 Rust 實作跑的是同一組檢查。
  - `agentTool`：工具的名稱、給模型看的說明、輸入格式。宿主把它註冊進自己的 Agent 框架，不用自己寫工具說明。
- `speclink-verbs` 新增產生工具說明的函式：從共用的動詞定義列出可用的動詞，所以動詞增加時說明不會落後。
- 既有的 31 方法 JS 儲存與 `createEngine` 的行為不變；型別宣告上標示它即將移除（在最後階段的變更 retire-teamstore-and-drivers 移除）。
- **CLI 指令**：沒有新增或變更。子指令、旗標、stdin、exit code 都不變。
- **設定欄位**：`speclink/config.yaml` 與 `.speclink.yaml` 沒有新增或變更。
- **相容性影響**：
  - CLI 的人眼輸出與 `--json`：不變。既有的 golden 與 CLI 測試不用更新。
  - `engine.dispatch`：不變。
  - 提交記錄的命令標籤：審查以外原本沒有專屬標籤的寫入命令（驗證的三個、搬移任務）今天記成 `command`，之後記成各自的名稱（`verify-add-round`、`verify-stamp`、`verify-discard`、`task-move`）。其餘寫入命令的標籤不變。這個標籤只出現在儲存的提交記錄裡，線上格式與 CLI 輸出都沒有它。
  - 示範用 server：這個變更不接宿主套件，行為不變（在變更 host-kit-client-door 改接）。
  - change 的 metadata：經宿主套件認領的 change 多一個選填欄位 `claimed_by_id`。缺席時照舊比顯示名稱；舊版引擎讀到它時忽略；CLI 的輸出、`--json` 與線上格式都沒有它。
- **影響的技能**：沒有。技能的文字不變，claude 與 codex 都不用重新產生技能檔。

## Non-Goals

- **給 CLI 與 Speclink Desktop 的門**（收 HTTP 請求、握手、輪詢、範圍清單、登入方式的宣告）：在變更 host-kit-client-door 做。所以建立宿主套件時的輸入這次只有儲存與掛鉤；「列出這個人可以用哪些範圍」的函式與登入方式的設定在那個變更加入，匯入大小的上限在變更 import-export-on-port 加入。掛鉤與設定都是具名的選填欄位，之後加入不會破壞既有的呼叫端。
- **每次呼叫的能力開關**：它的用途是握手回應，這個門沒有握手，所以在 host-kit-client-door 加入。這個門的權限由執行前掛鉤決定。
- **能改寫命令或結果的掛鉤**：原版的 CLI 與 Speclink Desktop 必須看得懂每個回應。
- **每個動詞各一個掛鉤**：掛鉤收到命令名稱，由宿主自己分辨。
- **保證送達的事件通道**：提交後掛鉤是盡力而為。保證由宿主在提交的交易裡自己做。
- **Rust 的動詞文字入口**：Agent 工具的門做在 Node SDK，因為內嵌的宿主是 Node 程序。Rust 的宿主套件收的是引擎命令。
- **需要 checkout 的動詞**（`drift`、`review prepare`、`review scope`、`verify scope`、`instructions --skill`）：這個門照舊以 `invalid_argv` 拒絕。`change rank` 已由變更 skills-verbs-first-wave 改成引擎命令，這個門可以執行它。
- **匯入匯出、自由欄位、遠端提供技能**：各自有後面的變更。
- **移除 31 方法的 JS 儲存**：這次只標示，不移除。
- **使用者文件**：這個變更不動 README 與 docs。

## Capabilities

### New Capabilities

- `host-kit`: 宿主套件的契約——建立時的輸入、每次呼叫的身分與範圍、兩個掛鉤能做與不能做的事、提交記錄帶的身分與命令名稱。既有的 host-runtime 規定的是「引擎怎麼經儲存接口執行」，沒有談宿主怎麼介入；node-sdk 規定的是 Node 的對外介面，而宿主套件的規則對 Rust 與 Node 都成立，所以另立一個 capability。

### Modified Capabilities

- `command-runtime`: 新增一條——每個引擎命令有穩定的名稱與可序列化的參數。
- `node-sdk`: 新增四條——`createHost` 與 `host.dispatch`、JS 版的儲存接口、隨套件提供的合格測試、Agent 工具定義的匯出。
- `change-lifecycle`: 修改「認領標記欄位」——執行情境帶識別時，認領多寫 `claimed_by_id`，並以識別判斷是否同一身分。

## Impact

- Affected specs: host-kit（新增）、command-runtime（新增一條）、node-sdk（新增四條）、change-lifecycle（修改一條）
- Affected crates: `speclink-core`、`speclink-host`、`speclink-verbs`、`speclink-node`；`speclink-cli` 只改建立執行情境的一行（`actor_id: None`）。`speclink-server`、store crates 與桌面 app 不改。
- Affected code:
  - New:
    - crates/engine/speclink-core/src/command/describe.rs
    - crates/engine/speclink-core/tests/it/command_describe.rs
    - crates/engine/speclink-core/tests/golden/command-describe.json
    - crates/host/speclink-host/src/kit.rs
    - crates/host/speclink-host/tests/kit.rs
    - crates/adapters/speclink-verbs/src/tool.rs
    - crates/adapters/speclink-node/src/port_bridge.rs
    - crates/adapters/speclink-node/src/host.rs
    - crates/adapters/speclink-node/__test__/memory-port.ts
    - crates/adapters/speclink-node/__test__/host.spec.ts
    - crates/adapters/speclink-node/__test__/port-conformance.spec.ts
    - crates/adapters/speclink-node/__test__/agent-tool.spec.ts
  - Modified:
    - crates/engine/speclink-core/src/command/mod.rs
    - crates/engine/speclink-core/src/command/tests.rs
    - crates/engine/speclink-core/src/lifecycle/model.rs
    - crates/engine/speclink-core/src/lifecycle/plan.rs
    - crates/engine/speclink-core/src/workspace/config.rs
    - crates/engine/speclink-core/tests/it/main.rs
    - crates/engine/speclink-core/tests/it/render_golden.rs
    - crates/host/speclink-host/src/port_exec.rs
    - crates/host/speclink-host/src/lib.rs
    - crates/host/speclink-host/tests/common/mod.rs
    - crates/adapters/speclink-verbs/src/lib.rs
    - crates/adapters/speclink-verbs/src/parse.rs
    - crates/adapters/speclink-verbs/src/render/mod.rs
    - crates/adapters/speclink-cli/src/common.rs
    - crates/adapters/speclink-node/src/lib.rs
    - crates/adapters/speclink-node/src/store_bridge.rs
    - crates/adapters/speclink-node/Cargo.toml
    - crates/adapters/speclink-node/index.js
    - crates/adapters/speclink-node/index.d.ts
    - Cargo.lock
  - Removed: (none)
- 建置產生的檔案：`crates/adapters/speclink-node/binding.js` 與 `binding.d.ts` 在 `.gitignore` 裡，由 `npm run build` 重新產生，所以不在上面的清單。跑 Node 測試前要先重新建置。
- 相依：排在變更 workflow-config-commands 之後（流程設定的寫入要先成為引擎命令，執行前掛鉤才收得到變動的鍵）。
- 三種形態下的結果：
  - 本機 repo：不適用。本機沒有宿主，CLI 直接對檔案執行；`createEngine` 的 fs 形式不變。
  - 連到遠端的 checkout：這次不做。CLI 與 Speclink Desktop 連的是 HTTP 的門，在變更 host-kit-client-door 做；在那之前示範用 server 的行為不變。
  - 沒有本機規格檔的內嵌宿主：可用。宿主以 `createHost` 建立宿主套件，Agent 工具經 `host.dispatch` 執行全部有引擎命令的動詞。已知缺口：需要 checkout 的五個動詞不可用；`task done` 這類需要「改了哪些檔」的動詞，這個門不替宿主收集本機事實——沒有給檔案清單時任務照常勾選，但不記任何檔案，evidence 也不帶 repo 與 head commit。
- 文件影響（留給最後一個變更 docs-embedded-host，這個變更不動使用者文件）：
  - docs/sdk-node.zh-TW.md 與 docs/sdk-node.md：新增宿主套件、兩個掛鉤、JS 版儲存接口、合格測試、Agent 工具定義；把 31 方法的 JS 儲存標成即將移除。
  - README.md 與 README.zh-TW.md：「把 Speclink 內嵌進你的系統」一節要引用 `createHost`。
  - docs/product-status.zh-TW.md 與 docs/product-status.md：列出這個門的已知缺口。

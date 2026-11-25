## Context

Herdr 的 Speclink 面板靠終端機標籤認 agent：`speclink.change`、`speclink.discussion`、`speclink.skill`。標籤現在有兩個來源：

1. 開分頁的人（面板、技能跳窗、`herdr-speclink` 技能）在 `hc run` 時一起貼上。
2. 使用者在 agent 裡直接打字時，選用的 hook 貼上：Claude Code 與 Codex 用 `integrations/herdr/mods/speclink/label-skill.mjs`，Pi 用 `pi-labels.ts`。規則在 `integrations/herdr/mods/speclink/model.js` 的 `commandLabels`（`CHANGE_COMMANDS`、`DISCUSSION_COMMANDS`、`FROM_DISCUSSION`）。

hook 要手動裝，Codex 只測過 `UserPromptSubmit`，Cursor 沒有任何機制，所以常常漏。所有 agent 的 speclink 技能最後都會呼叫 speclink CLI，而且帶著名稱，所以 CLI 是唯一「每種 agent 都會經過」的地方。

已查證的 herdr-core 事實（hc 0.1.22，`hc schema` 的 protocol generation 1）：

- hc 終端機裡的每個程序都有 `HC_TERMINAL_ID`（例如 `t2p`）與 `HC_TERMINAL_SOCKET`（core 的 socket 絕對路徑）。不在 hc 裡就沒有。
- socket 講 JSON-RPC 2.0，一行一則；`hello` 是選用的，不送也能直接呼叫方法。實測在本機，唯讀的 `terminal.get` 一來一回約 0.1 毫秒，會寫入的 `terminal.label` 約 5 毫秒（回覆是整個終端機物件，約 7 KB）；socket 不存在時，連線約 0.01 毫秒就失敗。
- 方法 `terminal.label`，參數 `terminal`（`t7`、名稱或 `machine/t7`）與 `labels`（物件，值是空字串代表移除）。錯誤的 `data.kind` 有 `not_found`、`machine_unknown`、`invalid_params`、`unavailable`。
- 對不存在的 socket 跑 `hc` 指令時，hc 會自己啟動一個新的 core。所以 CLI 絕不能呼叫 `hc`。

CLI 的進入點在 `crates/adapters/speclink-cli/src/main.rs`：`main` 解析參數後呼叫 `dispatch`，`dispatch` 依動詞形狀（Dual、FsOnly、RemoteOnly）分到本機或 remote 的實作，成功時回傳 `Ok(())`。`crates/adapters/speclink-cli/src/skill_notice.rs` 是現成的前例：一個不改變 stdout、`--json` 與 exit code 的旁支行為，邏輯在同檔用函式層測試。

## Goals / Non-Goals

**Goals:**

- 任何 agent 在 Herdr 終端機裡跑做變更或討論的 speclink 指令時，那個終端機的 `speclink.change`／`speclink.discussion` 標籤跟著更新，不用裝 hook。
- 不在 Herdr 裡的使用者完全看不到差別：不連線、不開程序、不多輸出、不多延遲。
- 回報怎麼失敗都不影響指令：輸出、結束碼不變，最多多等 100 毫秒，絕不啟動 core，不留下程序。
- 有一個關掉的開關。

**Non-Goals:**

- 不拿掉 hook（`label-skill.mjs`、`pi-labels.ts`），也不改面板、技能跳窗、`herdr-speclink` 技能貼標籤的做法。上線並實測後另外決定要不要簡化。
- 不改 `integrations/herdr/mods/` 與 `integrations/herdr/skills/` 的程式。Herdr mod 不走 speclink 規格；這個變更只動 CLI 與兩種語言的 `integrations/herdr/README`。
- CLI 不貼 `speclink.skill`，也不在使用者送出訊息的當下貼標籤（那時還沒有任何 speclink 指令在跑）；這兩件事留給 hook 與開分頁的人。
- 不處理指令沒帶名稱、由引擎自己選名稱的情況（沒帶 `--change` 的 `task done`、沒帶 `--slug` 的 `discuss new`）。
- 不送 `hello` 握手、不用 framed 模式、不訂閱事件；只送一則 `terminal.label`。
- 不支援 Herdr 以外的終端機工具，不做通用的回報出口（見 D1）。
- 不在 Windows 上回報。

## Decisions

### D1：CLI 直接認 Herdr，不提供通用的活動回報出口

比較兩種做法：

| 面向 | A：CLI 直接認 Herdr（採用） | B：CLI 提供通用出口，由 Herdr 整合來接 |
| --- | --- | --- |
| 做法 | CLI 看到 `HC_TERMINAL_ID`／`HC_TERMINAL_SOCKET`，直接對 socket 送 `terminal.label` | CLI 在指令成功後執行使用者設定的一個指令（或寫一個活動檔），把「做什麼」交給它；Herdr 整合提供那支接收程式 |
| 非 Herdr 使用者 | 只多讀兩個環境變數，沒有檔案讀取、程序或輸出 | 沒設定時一樣沒事，但每個指令都要先讀設定才知道有沒有設定 |
| Herdr 使用者 | 什麼都不用裝 | 要另外設定出口，等於把「hook 要手動裝所以常漏」原樣搬過去，失去這次變更的目的 |
| 速度與程序 | 一次 socket 來回，`terminal.label` 約 5 毫秒 | 每個指令多開一個程序（node 啟動就要數十毫秒）；為了守住「不留背景程序」，CLI 還得等它結束、逾時時殺掉它，並把它的輸出導走 |
| 其他終端機工具（tmux、WezTerm、Zellij…） | 不受影響，CLI 什麼都不做；以後真有需要，再加一個同樣小的模組，共用「哪些指令算數」的規則 | 理論上可以自己接，但現在沒有任何工具要用；CLAUDE.md 要求不加沒被要求的彈性 |
| 測試 | 「哪些指令算數」是純函式；送標籤的函式收「終端機代號、socket 路徑、時間上限」三個值，在函式層用暫存資料夾裡的 Unix socket 測；binary 測試用 Herdr 本來就有的環境變數接假的 core，不用新增測試專用的變數 | 要在測試裡真的開程序、量逾時、殺程序；binary 測試要設定一個出口，等於多一個設定或環境變數接縫 |
| 代價 | CLI 認得一個外部協定（herdr-core generation 1 的一個方法）。Herdr 改了方法時回報會靜靜失效，不會壞掉指令 | CLI 的協定是自己定的，但多了一份要維護的「出口」契約 |

活動檔的變體（CLI 把「這個終端機在做什麼」寫進某個檔，Herdr 監看）也排除：檔案要用 `HC_TERMINAL_ID` 分開，所以 CLI 一樣要認得 Herdr；還多出檔案位置、殘留檔與監看延遲的問題。

### D2：哪些指令算「正在做」——照解析好的參數判斷，和 commandLabels 一致

CLI 已經把參數解析成 `SpecVerb`，所以直接看解析結果，不用像 hook 那樣比對指令文字。規則和 `commandLabels` 一致（見 spec 的表），差別有三個：

- hook 看指令文字，不管成功與否；CLI 只在指令以 exit code 0 結束時回報。
- hook 的規則有 `verify prepare`，但這個指令不存在（驗證站沒有 prepare），CLI 的規則不列它。
- `archive` 可以一次帶好幾個名稱，hook 取第一個；CLI 只在恰好帶一個名稱時回報，帶多個名稱、沒帶名稱或帶 `--all` 時不回報，因為終端機一次只標一個變更，挑哪一個都是猜。

只在成功時回報的理由是：失敗的指令可能帶著打錯的名稱，貼錯的標籤比沒貼更糟，而前一個指令貼的標籤會留著。

名稱原樣取自參數，不再檢查格式：表內的指令在成功時已經由引擎檢查過名稱。

這個判斷放在 speclink-cli，不放 speclink-verbs：只有 CLI 會用它，Node SDK 的 dispatch 不回報（見 D6）。

### D3：直接對 HC_TERMINAL_SOCKET 講 JSON-RPC，絕不執行 hc

CLI 用 Rust 標準函式庫的 Unix socket 連到 `HC_TERMINAL_SOCKET`，送一行：

```json
{"jsonrpc":"2.0","id":1,"method":"terminal.label","params":{"terminal":"t2p","labels":{"speclink.change":"add-auth","speclink.discussion":""}}}
```

再讀一行回覆，內容不看（成功、錯誤、不是 JSON 都一樣），然後關閉連線。等回覆是為了確定 core 處理完請求才斷線。

- 不執行 `hc`：對不存在的 socket 跑 hc 會啟動新的 core，直接連 socket 的話，socket 不在就只是連線失敗。
- 不猜 socket：只有 `HC_TERMINAL_ID` 沒有 `HC_TERMINAL_SOCKET` 時不回報，不去找 hc 的預設路徑。
- 請求用 `#[derive(Serialize)]` 的結構組成，不手拼字串（標籤值可能有中文或引號），也不用 `serde_json::Value`（CLI 的 `no_raw_wire_json` 守門）。
- `terminal` 放 `HC_TERMINAL_ID` 的原值，`machine/t7` 這種形式也照送。

不送 `hello`：協定說明寫明一行一則的模式不用握手，實測可用。少一次來回，也少一個會因版本變動而失敗的步驟。

### D4：100 毫秒的總上限用一條工作執行緒做

連線、送出、等回覆都在一條新開的執行緒裡做，主執行緒用 channel 的 `recv_timeout` 最多等 100 毫秒。到了上限就不再等，`main` 照常回傳結束碼；程序結束時，那條執行緒跟著消失，不會留下任何程序。

不靠 socket 的讀寫逾時，因為標準函式庫的 Unix socket 沒有連線逾時：Linux 上 core 的連線佇列滿了時，連線會一直卡住。用執行緒包起來，才能保證整段有上限。`recv_timeout` 是唯一的上限，工作執行緒自己不設逾時；這樣「core 不回覆」的測試也能直接證明主執行緒到時限就不等。

100 毫秒的理由：正常情況下 `terminal.label` 約 5 毫秒（實作後量 `in-progress add` 十二次，中位數比不在 Herdr 時多約 9 毫秒），留了十倍以上的餘裕給忙碌中的 core；就算 core 卡住，多等十分之一秒也幾乎感覺不出來。

### D5：關掉的開關是環境變數 SPECLINK_TERMINAL_REPORT

值是 `false`（去掉前後空白、不分大小寫）時不回報；不設或其他值都照常回報。解讀方式和 `SPECLINK_TDD`、`SPECLINK_WORKTREE` 相同（`crates/engine/speclink-core/src/workspace/config.rs` 的 `parse_env_bool`：只認 `true`／`false`，其他當成沒設）。

選環境變數，不選設定檔的理由：

- 觸發條件本來就是環境變數（`HC_TERMINAL_ID`），開關和它放在同一層，範圍也一樣：一個人、一個終端機環境。可以寫在 shell 設定檔關掉全部，也可以只對一個分頁設。
- 專案的 `speclink/config.yaml` 會提交、會跟隊友共用；「我的終端機要不要貼標籤」不是專案的決定。
- 使用者層級設定檔（`speclink config set`）要每個指令在 Herdr 裡多讀一次 YAML，還要在一個目前放工作流預設值的檔案裡加一個終端機整合用的鍵；環境變數不用讀檔。
- 這個 repo 的測試環境也用同一個開關（見 D7），不用另外做測試專用的機制。

開關在確定「有算數的指令」且「兩個 HC 變數都有值」之後才讀，所以不在 Herdr 裡時連這個變數都不讀。

### D6：只在 CLI 的進入點回報，本機與 remote 模式都回報

回報放在 `main`：解析參數後先算出要貼的標籤（借用 `cli.command`，不消耗它），`dispatch` 回傳 `Ok(())` 之後才送，回傳錯誤時不送。

- **本機與 remote 都回報**：標籤描述的是「這個終端機在做什麼」，和文件放在哪裡無關。放在 `dispatch` 之外、不分模式，也守住 verb-contract「兩模式的分岔只在單一處宣告」——這裡沒有新增分岔。remote 模式下，`dispatch` 回傳 `Ok` 代表 Server 已經接受；回報不送任何請求給 Server。
- **只有 CLI**：Node SDK、server、desktop 都不經過 CLI 的 `main`，也不是跑在 agent 的終端機裡，所以不回報。命令執行層「輸出定義只有一份」的規定不受影響，因為回報不是輸出。
- **放在 speclink-cli，不放 speclink-core 或 speclink-host**：core 不做 I/O；host 是 CLI、server、desktop 共用的應用層，放在那裡會讓 server 與 desktop 也可能回報。這件事和規格儲存的抽象無關，不影響「storage 解耦的規格驅動引擎」的方向。

### D7：測試在函式層測邏輯，binary 測試用 Herdr 本來就有的環境變數，repo 的 cargo 環境預設關掉

- **函式層**（`crates/adapters/speclink-cli/src/terminal_report.rs` 同檔的 `#[cfg(test)] mod tests`）：
  - 「哪些指令算數」：用 CLI 真的參數解析器解析一串指令，逐一比對要貼的標籤或「不貼」。
  - 讀環境：收一個查詢函式（和 `EnvOverrides::from_lookup` 同樣的做法），測兩個 HC 變數缺一個、空字串、開關的各種值。
  - 送標籤：收「終端機代號、socket 路徑、標籤、時間上限」，用暫存資料夾裡的 `UnixListener` 當假的 core，測收到的那一行、不回覆時在上限內返回、socket 不存在或路徑是一般檔案時安靜返回。
- **binary 層**（`crates/adapters/speclink-cli/tests/it/terminal_report.rs`，只在 Unix 編譯）：設 `HC_TERMINAL_ID`、`HC_TERMINAL_SOCKET`（指向測試裡的假 core），清掉 `SPECLINK_TERMINAL_REPORT`，跑真的 speclink。這些都是 Herdr 與這個功能本來就有的變數，不是為了測試新增的接縫（專案記憶「CLI binary 測試不加環境變數接縫」指的是測試專用的變數）。
- **repo 的 `.cargo/config.toml`**：新增 `[env]` 設 `SPECLINK_TERMINAL_REPORT = "false"`。開發者或 agent 在 Herdr 裡跑 `cargo test` 時，數十個 binary 測試會在暫存專案裡跑 `new change demo`、`task done --change …`；沒有這一行，這些假名稱會貼到開發者自己的終端機上，面板會把正在 apply 的 agent 從它的變更列上拿掉。已在暫存專案實測：cargo 的 `[env]` 會傳到 `cargo test` 開的 binary 子程序，而且使用者自己設的值會蓋過它（沒有 `force`）。`npm run cli` 是 `cargo build` 後由 node 直接開 binary，不受影響；`cargo run` 會受影響，實測時要用建好的 `target/debug/speclink` 或 `npm run cli`。

### D8：Windows 不回報

Rust 標準函式庫的 `std::os::unix::net` 只在 Unix 有。回報的實作用 `#[cfg(unix)]`，其他平台是一個什麼都不做的同名函式，讓 `main` 不用分平台。

## Implementation Contract

**行為：** 在 Herdr 終端機裡（`HC_TERMINAL_ID`、`HC_TERMINAL_SOCKET` 都有值、`SPECLINK_TERMINAL_REPORT` 不是 `false`、平台是 Unix），spec 表內的指令以 exit code 0 結束前，CLI 對那個 socket 送恰好一則 `terminal.label` 請求，`labels` 只有 `speclink.change` 與 `speclink.discussion`，值照 spec 的表。其他情況什麼都不做。

**介面與資料形狀：**

- 請求：`{"jsonrpc":"2.0","id":1,"method":"terminal.label","params":{"terminal":"<HC_TERMINAL_ID>","labels":{"speclink.change":"<名稱或空字串>","speclink.discussion":"<slug 或空字串>"}}}` 加一個換行。
- 新模組 `terminal_report`（speclink-cli 內部）對 `main` 提供兩件事：從解析好的指令算出要貼的標籤（沒有就是「不貼」），以及在指令成功後送出。讀環境的部分收一個查詢函式，送出的部分收終端機代號、socket 路徑與時間上限，這兩部分都不直接讀程序的環境變數。
- 新環境變數 `SPECLINK_TERMINAL_REPORT`；沒有新的子指令、旗標或設定鍵。

**失敗模式（全部安靜）：** socket 不存在、拒絕連線、不是 socket、路徑太長、寫入失敗、沒有回覆、回覆是錯誤或不是 JSON、超過 100 毫秒，都不印任何東西，不改 stdout、stderr 與 exit code。CLI 從不執行 `hc` 或其他程式。

**驗收：**

- `cargo test -p speclink-cli` 通過，包含 `terminal_report` 的函式層測試與 `tests/it/terminal_report.rs` 的 binary 測試；binary 測試涵蓋：本機 `task done --change` 成功時收到一則請求且 stdout、stderr、exit code 與不設 HC 變數時相同（含 `--json`）；`new change … --from-discussion`；`discuss add-round`；只讀指令不連線；失敗的指令不連線；socket 不存在時 `PATH` 上的假 `hc` 沒被執行；不回覆的假 core 讓指令在寬鬆上限內結束；`SPECLINK_TERMINAL_REPORT=false` 不連線；remote 模式下 Server 接受時回報、Server 回錯誤時不回報。
- 既有的 golden 與 CLI 測試（含 `crates/adapters/speclink-cli/tests/it/remote_verb_parity.rs`）不用改就通過。
- Windows CI 能編譯（非 Unix 的空函式）。
- 實測：在 Herdr 裡，沒裝 hook 的 Codex、Pi、Cursor 跑技能後，面板在對的列上看到 agent。

**範圍：**

- 範圍內：speclink-cli 的 `main` 與新模組、CLI 的新測試、repo 根目錄的 `.cargo/config.toml`、`integrations/herdr/README.md` 與 `README.en.md`、`docs/configuration.md` 與 `docs/configuration.zh-TW.md` 的環境變數表。
- 範圍外：speclink-core、speclink-verbs、speclink-host、Node SDK、server、desktop；`integrations/herdr/mods/` 與 `integrations/herdr/skills/` 的程式；hook 的去留。

## Risks / Trade-offs

- [在 Herdr 裡跑 `cargo test` 時，binary 測試把假名稱貼到開發者的終端機] → `.cargo/config.toml` 的 `[env]` 預設關掉（D7）；`[M]` 實測時在 Herdr 裡跑一次 `cargo test -p speclink-cli`，前後用 `hc get` 比對標籤沒變。
- [既有回歸對照被影響] → CI 沒有 HC 變數，本機的 cargo 測試有 `[env]` 關掉，golden 與 `remote_verb_parity.rs` 的輸出都不會變；新增的 binary 測試直接比對「有 HC 變數」與「沒有」兩次的 stdout、stderr、exit code。
- [跨平台] → macOS 與 Linux 用 Unix socket；Windows 編成空函式，Windows CI 會抓到編譯問題。macOS 的 socket 路徑上限是 104 位元組，測試的暫存 socket 要用短檔名；超過上限時連線失敗，屬於安靜失敗。
- [Linux 上連線可能卡住] → 執行緒加 100 毫秒總上限（D4）。
- [herdr-core 改了 `terminal.label` 或協定世代] → 回報安靜失效，指令不受影響；README 寫明測過的 hc 版本，hook 也還在，標籤不會完全消失。
- [hook 和 CLI 都貼同一個標籤] → 兩邊的值相同，重複貼沒有副作用。差別只在失敗的指令：hook 照貼，CLI 不貼。
- [同一個終端機裡的子 agent 跑別的變更的指令] → 標籤會改成那個變更。這和 hook 現在的行為相同：一個終端機一次只代表一件事，最後成功的指令說了算。
- [Codex 預設的沙盒擋 Unix socket] → 7.1 實測（Codex CLI 0.171）：Codex 跑的指令連 socket 得到 `Operation not permitted`，回報安靜失敗、指令照常完成，符合「回報不改變指令的結果也不啟動任何程序」。用 `-c sandbox_workspace_write.network_access=true` 打開沙盒網路後，變更與討論的標籤都正確。CLI 不繞過沙盒；README 寫明限制與打開網路的做法和代價，Codex 照舊靠 hook。
- [hook 看指令文字會誤判] → 7.1 實測時，Claude Code 的 hook 從一個 `hc send` 指令的文字裡讀到 `| speclink discuss add-round hello-talk`，把沒有跑這個指令的終端機標成那場討論。CLI 回報只發生在真的跑了指令的終端機，沒有這個問題；hook 要不要簡化，照 Non-Goals 另外決定。
- [agent 在專案 A 的終端機裡 cd 到專案 B 跑指令] → 標籤換成 B 的變更名稱，A 的面板找不到那一列。和 hook 的行為相同，不在這次處理。
- [每個成功的指令最多多 100 毫秒] → 只在 Herdr 裡、只在表內的指令、只在 core 卡住時才會碰到；正常約多 5 到 10 毫秒。

## Migration Plan

不用遷移。新版 CLI 安裝後自動生效，hook 照舊運作。要退回，設 `SPECLINK_TERMINAL_REPORT=false` 或裝回舊版 CLI。

## Open Questions

沒有。hook 與面板貼標籤的做法要不要簡化，等這個變更上線並實測後另外討論。

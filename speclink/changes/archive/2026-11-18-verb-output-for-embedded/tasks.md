## 1. 引擎：清單的詳細資料（D1、D3）

- [x] 1.1 先寫失敗的測試，涵蓋需求「清單的詳細輸出」的引擎部分，寫在 crates/engine/speclink-core/src/lifecycle/listing.rs 的測試區塊：`detail` 為真時，審查已蓋章的變更有 `review`（state 是 stamped）、有未結驗證工單的變更有 `verify`（state 是 inProgress）、被認領的變更有 `claimed`（by、有記錄才有的 byId 與 at）、記下分支的變更有 `branch`、已落地的變更有 `landed`（只有 commit）；什麼記錄都沒有的變更序列化結果與 `detail` 為假時相同；metadata 損壞的變更帶 metaError 而沒有五個新鍵。既有的「CLI 清單輸出的相容性釘住」與「CLI 清單輸出的驗證欄位釘住」兩個測試不改斷言。驗證：`cargo test -p speclink-core lifecycle::listing` 這時因 `detail` 參數不存在而編譯失敗。 <!-- speclink-task:tsk_01M7PNK22CQDRMKEZXEJ0JDWG6 -->
- [x] 1.2 實作 D1 清單的詳細資料由引擎的 List 命令在 detail 為真時算出：crates/engine/speclink-core/src/command/mod.rs 的 `Command::List` 新增 `detail`，crates/engine/speclink-core/src/lifecycle/listing.rs 的清單項新增五個沒有值就不序列化的欄位，來源沿用 `show` 的關卡狀態函式、認領記錄與落地記錄；crates/engine/speclink-core/src/command/describe.rs 的參數加上 `detail`，刻意更新 crates/engine/speclink-core/tests/golden/command-describe.json。驗證：1.1 的測試通過；`cargo test -p speclink-core` 全部通過，golden 的差異只有 list 的新參數。 <!-- speclink-task:tsk_01M7PNK22CXFZWB4Q0Q3SHB02Y -->
- [x] 1.3 依 D3 呼叫儲存的次數：不帶 detail 時不變，在 crates/host/speclink-host/tests/port_cost.rs 新增一個斷言記下帶 detail 的 List 對儲存的呼叫數，並確認兩張工單的讀取併在每個變更既有的讀取批次裡。驗證：`cargo test -p speclink-host --test port_cost` 通過，既有斷言一個都沒改。 <!-- speclink-task:tsk_01M7PNK22C5475MB0T18D3RMXC -->

## 2. 動詞的解析與輸出（D1、D4、D5）

- [x] 2.1 先寫失敗的測試，寫在 crates/adapters/speclink-verbs/src/parse.rs 與 crates/adapters/speclink-verbs/src/render/query.rs 的測試區塊：`list --json --detail` 解析成 `detail` 為真；`list --detail`（沒有 --json）與 `list --specs --json --detail` 以用法錯誤拒絕，訊息分別指出要和 --json 一起用、不能和 --specs 並用；清單的 JSON 輸出在有記錄時含五個鍵（camelCase），沒有記錄時與現在逐位元相同。驗證：`cargo test -p speclink-verbs` 這時失敗。 <!-- speclink-task:tsk_01M7PNK22CS42M9BBD6HHHS9ZN -->
- [x] 2.2 實作清單的解析與輸出：crates/adapters/speclink-verbs/src/parse.rs 與 inputs.rs 為 `list` 加上 `--detail` 與兩條檢查；crates/adapters/speclink-verbs/src/render/query.rs 的清單 JSON 直接序列化清單項的新欄位。驗證：2.1 的測試通過。 <!-- speclink-task:tsk_01M7PNK22C445Y23F33N0HQZE0 -->
- [x] 2.3 先寫失敗的測試，涵蓋需求「內嵌宿主的人眼輸出不印路徑行」，寫在 crates/adapters/speclink-verbs/src/render/mod.rs 的測試區塊：`render_embedded` 對 `new change` 的輸出沒有 Path 行而保留 Schema 與 From discussion 行；對 `discuss promote` 沒有 Path 行與 propose 提示行；對 `discuss new` 沒有 Path 行而保留 Topic 行；三個動詞的 JSON 形式與 `render` 相同、`path` 鍵仍在；其他動詞（取 `list` 與 `archive` 各一個樣本）兩個入口的輸出逐位元相同；`render` 對這三個動詞的輸出與現在逐位元相同。驗證：`cargo test -p speclink-verbs render::` 這時因 `render_embedded` 不存在而編譯失敗。 <!-- speclink-task:tsk_01M7PNK22C66PRD8MNFNXVHH8F -->
- [x] 2.4 實作 D4 內嵌用的輸出入口不給路徑行：crates/adapters/speclink-verbs/src/render/mod.rs 把統一入口拆成內部函式加 `render` 與 `render_embedded`；crates/adapters/speclink-verbs/src/render/new.rs 沿用既有的沒有路徑的資料形狀；crates/adapters/speclink-verbs/src/render/discuss.rs 的 `discuss new` 輸出資料把路徑改成選填。不在 `RenderOptions` 加欄位。驗證：2.3 的測試通過；`cargo test -p speclink-verbs` 全部通過。 <!-- speclink-task:tsk_01M7PNK22C2WF34PM99PF4W0DS -->
- [x] 2.5 先寫測試再實作 D5 Agent 工具的說明加一句 path 的解釋（需求「Agent 工具的說明指出 path 是儲存的鍵」）：crates/adapters/speclink-verbs/src/tool.rs 的說明加上設計 D5 的那一句英文，測試斷言說明含 `path`、`not a file`、`artifact cat`、`show`、`discuss show`，而且動詞清單與 inputSchema 不變。驗證：`cargo test -p speclink-verbs tool::` 通過。 <!-- speclink-task:tsk_01M7PNK22CD4HZSB6E299KVHPG -->

## 3. 連到遠端的 CLI（D2）

- [x] 3.1 先寫失敗的測試，寫在 crates/adapters/speclink-cli/tests/it/remote_verb_parity.rs：同樣的變更與記錄（已蓋章的審查、未結的驗證工單、分支、落地）在本機專案與測試用 server 上，`speclink list --json --detail` 的 `changes` 深度相等（本機才有的 worktree 鍵除外）；被認領的變更在遠端輸出 `claimed`；`speclink list --json` 在兩模式的輸出與現在相同；回應缺少關卡、分支與落地欄位時 `--detail` 不失敗且沒有新鍵；`speclink list --detail` 非零結束且 stdout 是空的。驗證：`cargo test -p speclink-cli --test it remote_verb_parity` 這時失敗。 <!-- speclink-task:tsk_01M7PNK22C7HE0DXX1A1AT456T -->
- [x] 3.2 實作 D2 連到遠端的 CLI 用既有的清單回應組出相同的鍵：crates/adapters/speclink-cli/src/verbs/query.rs 的 `to_list_change_json` 在 `--detail` 時轉換 review、verify（沿用遠端 show 的關卡轉換）、claimedBy／claimedById／claimedAt、branch、landedCommit，不新增請求。驗證：3.1 的測試通過；`cargo test -p speclink-cli --test it` 全部通過。 <!-- speclink-task:tsk_01M7PNK22CQP1PQ5V48B32096K -->

## 4. Node SDK 的宿主路徑（D4）

- [x] 4.1 先寫失敗的測試，寫在 crates/adapters/speclink-node/__test__/host.spec.ts 與 crates/adapters/speclink-node/__test__/agent-tool.spec.ts：`host.dispatch(['new', 'change', 'demo'])` 的 output 沒有以 `Path:` 開頭的行；`discuss new` 與 `discuss promote` 同樣沒有，promote 也沒有 propose 提示行；`discuss new --json` 的結果仍有 `path`；`createEngine` 對本機資料夾的 `new change` 仍有 `Path:` 行；`host.dispatch(['list', '--json', '--detail'])` 的項目含有記錄的鍵，`['list', '--detail']` 以 `invalid_argv` 拒絕；`agentTool.description` 含 path 的那一句。驗證：`npm --prefix crates/adapters/speclink-node run build` 後 `npm --prefix crates/adapters/speclink-node test`，新增的測試失敗。 <!-- speclink-task:tsk_01M7PNK22C5VQKX7WNZBRHYXTK -->
- [x] 4.2 實作：crates/adapters/speclink-node/src/lib.rs 與 crates/adapters/speclink-node/src/host.rs 讓 `createHost` 的輸出（dispatch，以及已存在時的掛鉤讀取路徑）改呼叫 `render_embedded`，`createEngine` 繼續呼叫 `render`；crates/adapters/speclink-node/index.d.ts 在 `Host.dispatch` 的說明寫明人眼輸出不含三個動詞的路徑行、JSON 的 path 是儲存的鍵。驗證：build 後 `npm --prefix crates/adapters/speclink-node test` 全部通過。 <!-- speclink-task:tsk_01M7PNK22C9VJ5EYGCDSSCH1BT -->

## 5. 稽核

- [x] 5.1 對新增的旗標與輸出入口套用 sharp-edges 稽核清單（用 `speclink instructions --skill audit` 取得），把結論寫在 crates/adapters/speclink-verbs/src/render/mod.rs 兩個入口的說明註解：`--detail` 的兩條檢查在共用的解析，CLI 與宿主套件給出同一個錯誤；`render_embedded` 只影響三個動詞的人眼輸出，JSON 兩個入口相同；清單的 `claimed.byId` 是宿主給的識別，讀得到這個範圍的人都看得到，文件已寫明；metadata 損壞的變更不會讓 `--detail` 失敗。驗證：`cargo test -p speclink-verbs` 與 `cargo test -p speclink-core` 全部通過。 <!-- speclink-task:tsk_01M7PNK22CZX0961SH18C5S9FN -->

## 6. 文件與收尾（D6）

- [x] 6.1 依 D6 使用者文件更新，中英兩版同步：docs/verb-contract.md 與 docs/verb-contract.zh-TW.md 的 `list` 加上 `--detail` 與五個鍵的表（出現條件、與 `show --json` 同形、清單的 landed 只有 commit）；docs/sdk-node.md 與 docs/sdk-node.zh-TW.md 的 Agent 工具一節說明路徑行與 path 鍵，並把總覽的範例改用 `list --json --detail`；docs/product-status.md 與 docs/product-status.zh-TW.md 的能力列提到 `--detail`。驗證：文件的鍵名與 1.1、2.1 的測試斷言一致；`node --test scripts/*.test.mjs scripts/*/*.test.mjs` 全部通過。 <!-- speclink-task:tsk_01M7PNK22CHAD7RHV7753YNYQ3 -->

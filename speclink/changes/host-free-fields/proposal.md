## Why

把 speclink 內嵌進自己系統的宿主，常常要在變更或討論上記下屬於自己的資料：這個變更來自哪裡、對應哪一張工單、誰負責、封存時的備註。今天沒有地方放：

- **引擎的 metadata 只有固定的欄位。** 沒有通用的「設一個鍵」的動詞。
- **線上的回應不帶未知的鍵。** 宿主就算自己把鍵寫進 metadata，用戶端與宿主的其他程式也讀不回來。
- **建立變更或討論時沒辦法順便帶資料。** 宿主只能在建立之後再寫一次，中間有一段時間這個變更沒有來源，而且第二次寫入可能失敗。

查證時確認了一件有利的事：metadata 與討論記錄開頭的區塊裡，引擎不認得的鍵在讀取時會被忽略、在改寫時會原樣保留。所以多一個專屬的區塊不會被既有的動詞弄丟。

討論 embedded-host-needs 的裁定：提供自由欄位。它是宿主自訂的鍵值，引擎只存、不解讀、不依它做任何判斷。寫入有通用的動詞，原版的 CLI 也能用；宿主的程式可以在呼叫入口時附上，和建立變更或討論在同一次寫入，不經過模型。

- **目標使用者**：把 speclink 內嵌進自己後端的整合者；用 CLI 把變更對到工單的開發者。
- **使用情境**：propose（建立變更）、discuss（建立討論）、archive 之前；宿主的看板或工單系統讀出這些欄位。
- **既有規格的對照**：沒有任何正式規格涵蓋宿主自訂的欄位。discussion-docs 規定討論記錄的固定欄位；client-protocol 規定清單與單筆回應的欄位，都是 speclink 自己的；host-kit 規定每次呼叫給身分與範圍，沒有其他附帶的資料。

Source discussion: embedded-host-needs（cut 15）。

## What Changes

**資料**

- 每個變更與每個討論可以有一組自由欄位：鍵對字串值。
- 鍵：小寫英文字母開頭，之後是小寫字母、數字、`_`、`-`、`.`，最長 64 個字元。
- 值：單行的字串，最長 1024 個位元組，不含換行與控制字元。
- 每份文件最多 32 個鍵。
- 存在既有的文件裡：變更是 metadata 的 `fields:` 區塊，討論是記錄開頭區塊裡的 `fields:` 區塊。儲存不用認得新的文件種類。封存時跟著文件一起進封存。
- 引擎只存。它不解讀任何鍵，不依欄位改變任何動詞的行為。

**動詞**

- `speclink change field set <name> <key>=<value>...`：設定一個或多個欄位。全部成功或全部不寫。
- `speclink change field unset <name> <key>...`：移除欄位。不存在的鍵不算錯誤。
- `speclink discuss field set <slug> <key>=<value>...` 與 `speclink discuss field unset <slug> <key>...`：討論的對應動詞。
- 四個動詞都有 `--json` 與 `--no-color`，沒有 stdin。stdout：一行確認，列出動到的鍵；`--json` 是 `{ "change" 或 "discussion", "fields": { 寫入後的全部欄位 } }`。
- exit code：成功 0；變更或討論不存在、鍵或值不合規則、超過 32 個鍵都是非 0，而且沒有任何寫入。
- `speclink new change` 與 `speclink discuss new` 新增可重複的旗標 `--field <key>=<value>`：建立時就帶上欄位，和建立在同一次寫入。
- 各產生一種事件：`change-fields-changed` 與 `discussion-fields-changed`，payload 有名稱與動到的鍵。

**讀取**

- `speclink list --json` 的變更項目、`speclink show <change> --json`、`speclink discuss list --json` 與 `speclink discuss show --json` 的討論資訊，新增選填的 `fields` 物件。沒有欄位時不輸出這個鍵，所以沒有用到自由欄位的專案，輸出逐位元不變。
- 線上的變更清單項、單筆變更、已封存清單項與討論資訊，新增選填的 `fields`。
- 人眼輸出：`speclink show <change>` 與 `speclink discuss show` 不變；自由欄位只在 `--json` 出現。

**宿主在呼叫入口時附上**

- Node 的 `host.dispatch` 的呼叫物件新增選填的 `fields`；`host.handleRequest` 新增選填的第三個參數，裡面有 `fields`。
- 只在這次呼叫會建立變更或討論時生效（建立變更、建立討論、從討論轉出變更）：欄位和建立在同一次寫入。其他命令忽略它，所以宿主可以每次呼叫都帶。
- 宿主附上的欄位蓋過請求自己帶的同名欄位：它來自宿主的程式，不經過模型，也不經過用戶端。
- 執行前掛鉤收到的參數含最後要寫入的欄位，宿主可以拒絕。

**線上**

- 建立變更、建立討論、轉出變更的請求新增選填的 `fields`。
- 新增兩條寫入的路由：設定與移除變更的欄位、設定與移除討論的欄位。

**相容性影響**

- 沒有自由欄位的變更與討論：全部既有的輸出逐位元不變，既有的 golden 不用更新。
- 有自由欄位時：上面列的 `--json` 輸出多一個 `fields` 鍵。
- 舊版的引擎讀到有 `fields:` 區塊的 metadata：忽略它，改寫時原樣保留（既有行為，測試釘住）。
- 舊版用戶端連到新版 Server：回應多一個選填欄位，照常運作。
- 新版 CLI 連到舊版 Server：設定欄位的路由不存在，回 404；`--field` 帶的欄位被舊版 Server 忽略，建立仍然成功——CLI 在這種情況印一行警告。
- **設定欄位**：`speclink/config.yaml` 與 `.speclink.yaml` 沒有新增或變更。
- **影響的技能**：沒有。

## Non-Goals

- **引擎依自由欄位做任何事**：不篩選、不排序、不擋動詞。要依欄位擋動詞的宿主用執行前掛鉤。
- **非字串的值**（數字、布林、巢狀結構）：宿主自己把它們編成字串。
- **已封存的變更與討論上修改欄位**：封存後是唯讀的。要在封存時留備註的宿主，先設欄位再封存。
- **任務、artifact、正式規格上的自由欄位**：只做變更與討論。
- **依欄位搜尋或篩選的動詞**：宿主在自己的資料庫裡做。
- **Speclink Desktop 顯示或編輯自由欄位**：這次不做。
- **把分支與落地的 commit 放進自由欄位**：它們是 speclink 自己的欄位（變更 station-status-and-landing）。
- **經已標示即將移除的 31 方法 JS 儲存使用自由欄位**：那條路徑不帶未知的鍵；自由欄位只保證在儲存接口與本機檔案上可用。
- **使用者文件**：這個變更不動 README 與 docs。

## Capabilities

### New Capabilities

- `free-fields`: 變更與討論上的自由欄位——鍵與值的規則、存放的位置、寫入與移除的動詞、建立時附帶、讀取的輸出，以及「引擎只存不解讀」的界線。既有的 discussion-docs 與 change-lifecycle 規定的是 speclink 自己的欄位；沒有規格涵蓋宿主自訂的資料。

### Modified Capabilities

- `client-protocol`: 新增「自由欄位的線上欄位與請求」。
- `server-verb-api`: 新增「自由欄位的寫入端點」。
- `host-kit`: 新增「呼叫入口時附上自由欄位」。
- `node-sdk`: 新增「dispatch 與 handleRequest 的 fields」。

## Impact

- Affected specs: free-fields（新增）、client-protocol、server-verb-api、host-kit、node-sdk（各新增一條）
- Affected crates: `speclink-core`、`speclink-verbs`、`speclink-host`、`speclink-protocol`、`speclink-remote`、`speclink-cli`、`speclink-node`。`speclink-server`、store crates、桌面 app 不改。
- Affected code:
  - New:
    - crates/engine/speclink-core/src/lifecycle/fields.rs
    - crates/engine/speclink-core/src/command/fields.rs
    - crates/adapters/speclink-verbs/src/families/fields.rs
    - crates/adapters/speclink-verbs/src/render/fields.rs
    - crates/adapters/speclink-cli/tests/it/free_fields.rs
    - crates/host/speclink-host/tests/free_fields.rs
    - crates/adapters/speclink-node/__test__/free-fields.spec.ts
  - Modified:
    - crates/engine/speclink-core/src/command/mod.rs
    - crates/engine/speclink-core/src/command/describe.rs
    - crates/engine/speclink-core/src/lifecycle/model.rs
    - crates/engine/speclink-core/src/lifecycle/newcmd.rs
    - crates/engine/speclink-core/src/lifecycle/discuss.rs
    - crates/engine/speclink-core/src/lifecycle/listing.rs
    - crates/engine/speclink-core/tests/golden/command-describe.json
    - crates/adapters/speclink-verbs/src/families/mod.rs
    - crates/adapters/speclink-verbs/src/families/new.rs
    - crates/adapters/speclink-verbs/src/families/plan.rs
    - crates/adapters/speclink-verbs/src/families/discuss.rs
    - crates/adapters/speclink-verbs/src/render/mod.rs
    - crates/adapters/speclink-verbs/src/render/query.rs
    - crates/adapters/speclink-verbs/src/render/discuss.rs
    - crates/host/speclink-host/src/kit.rs
    - crates/host/speclink-host/src/commit.rs
    - crates/host/speclink-host/src/documents.rs
    - crates/host/speclink-host/src/http/mod.rs
    - crates/host/speclink-host/src/http/router.rs
    - crates/host/speclink-host/src/http/routes.rs
    - crates/host/speclink-host/src/http/wire.rs
    - crates/host/speclink-host/src/http/coverage.rs
    - crates/protocol/speclink-protocol/src/command.rs
    - crates/protocol/speclink-protocol/src/query.rs
    - crates/protocol/speclink-remote/src/client.rs
    - crates/adapters/speclink-cli/src/main.rs
    - crates/adapters/speclink-cli/src/verbs/new.rs
    - crates/adapters/speclink-cli/src/verbs/plan.rs
    - crates/adapters/speclink-cli/src/verbs/discuss.rs
    - crates/adapters/speclink-cli/src/verbs/query.rs
    - crates/adapters/speclink-cli/tests/it/main.rs
    - crates/adapters/speclink-node/src/host.rs
    - crates/adapters/speclink-node/src/lib.rs
    - crates/adapters/speclink-node/index.js
    - crates/adapters/speclink-node/index.d.ts
  - Removed: (none)
- 相依：排在變更 station-status-and-landing 之後（兩者改同一批回應型別與同一個 `change` 動詞家族，依序做避免衝突）。
- 三種形態下的結果：
  - 本機 repo：四個動詞與 `--field` 可用，欄位存在本機的 metadata 與討論記錄。
  - 連到遠端的 checkout：同樣可用，欄位存在 Server。
  - 沒有本機規格檔的內嵌宿主：可用。宿主的程式另外能在 `host.dispatch` 與 `host.handleRequest` 附上欄位。已知缺口：已封存的文件不能改欄位；沒有依欄位查詢的動詞。
- 文件影響（留給最後一個變更 docs-embedded-host，這個變更不動使用者文件）：
  - 動詞與旗標的參考文件：四個動詞、`--field`、新的線上欄位與路由。
  - 「把 Speclink 內嵌進你的系統」與 Node SDK 文件：在呼叫入口時附上欄位、用執行前掛鉤檢查欄位。
  - 設定說明或工作流程文件：鍵與值的規則。

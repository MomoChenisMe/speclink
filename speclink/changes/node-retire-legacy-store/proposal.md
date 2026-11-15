## Why

Node SDK（npm 套件 `@speclink/engine`）今天有兩條讓宿主接上自己儲存的路：

- **舊的路**：`createEngine({ store: <物件> })`。宿主實作一個有 31 個必要方法、8 個選填方法的 JS 物件，引擎一個方法一個方法地呼叫它。
- **新的路**：`createHost({ store: { list, read, commit } })`。宿主只實作三個函式，引擎把一個動詞的全部寫入與事件收成一次提交。

舊的路有三個新的路已經解決的問題：一個動詞的多筆寫入沒有交易邊界，可能留下半套；沒有版本比對，兩個人會互相覆蓋；身分在建立引擎時綁定，一個實例只能代表一個人。它在型別宣告裡已經標成即將移除。之後新增的功能也不再支援它：自由欄位的動詞在舊的路上直接被拒絕。

繼續保留它的成本是：`dispatch` 的每一條規則都要寫兩份（本機資料夾與宿主物件），原生層有四個只為舊物件存在的「缺方法就拒絕」守門，測試有 68 個案例只測舊物件。

- **目標使用者**：把 speclink 內嵌進自己 Node 後端的整合者。
- **使用情境**：整合者選擇怎麼把引擎接上自己的資料庫。這個變更之後只有一條路：三個函式加 `createHost`。
- **既有規格的對照**：node-sdk 的「createEngine 的雙形式儲存建構」規定兩種形式；「dispatch 的輸入輸出契約」「createEngine 的建構期 actor 注入」「dispatch 的蓋章動詞」「dispatch 的流程設定動詞」「dispatch 與 handleRequest 的 fields」各有只對舊物件成立的條款與情境。change-plan 的「change rank 動詞寫入看板順序鍵」有一句舊物件的例外。host-kit 與 store-port-contract 不受影響。

查證的結果：舊物件接的是引擎自己的儲存介面，不是示範用 server 的舊儲存層（TeamStore）。所以這個變更和移除 TeamStore 沒有先後關係。

Source discussion: embedded-host-needs（cut 17）。

## What Changes

**`createEngine` 只接受本機資料夾**（**BREAKING**）

- `createEngine({ store: { type: 'fs', root, specDir? }, actor? })` 不變。
- 傳入其他形狀的 `store`（包含舊的 Store 物件）時，`createEngine` 同步拋出錯誤。訊息說明只接受本機資料夾，並指向 `createHost`。
- 型別宣告移除 `Store`、`ChangeMeta`、`ChangeInfo`、`DiscussionDoc` 與 `createEngine` 接受 Store 物件的那個多載。

**`engine.dispatch` 的規則只剩一份**

- 操作者身分：有給 `actor` 就用它；沒有給就用專案的 git 身分。「沒有給就不蓋章」的分支移除。
- `claim`：和 CLI 在本機專案一樣被拒絕（訊息相同）。由宿主物件的 `claim` 方法裁決的路移除。
- 蓋章動詞、流程設定動詞、`change rank`、自由欄位的動詞：只為舊物件存在的「缺某個選填方法就拒絕」全部移除。這些動詞在本機資料夾上本來就都能用。

**原生層與測試**

- 移除把 JS 物件接成引擎儲存的橋接（約 310 行）與四個只為它存在的守門。
- 呼叫 JS 函式的共用機制留下（三個函式的儲存與兩個掛鉤都在用），搬到名稱不帶「store」的檔案。
- 只測舊物件的 68 個測試案例：行為仍然存在的改用本機資料夾測；行為已經移除的刪掉。

**相容性影響**

- **破壞性**：用 Store 物件呼叫 `createEngine` 的程式會在建立引擎時得到錯誤。遷移方式：把儲存改寫成三個函式（`list`、`read`、`commit`），改用 `createHost`，每次呼叫給身分與範圍。套件裡的 `runStoreConformance` 能檢查三個函式的實作。
- 本機資料夾形式：`dispatch` 的輸入、輸出與錯誤碼都不變。
- `createHost`、`host.dispatch`、`host.handleRequest`、匯出匯入、`skills`、`agentTool`、`runStoreConformance`：都不變。
- **CLI 指令、設定欄位、技能、線上協定**：都沒有變更。
- 這個變更在整個系列完成之後才發版，發版說明要列出這一項破壞性變更。

## Non-Goals

- **移除 `createEngine` 本身**：本機資料夾形式留著，它是在 Node 裡操作本機專案的路。
- **替舊物件寫轉接成三個函式的相容層**：舊物件沒有交易邊界與版本比對，轉接出來的東西過不了合格測試。
- **示範用 server 的舊儲存層（TeamStore）與三個 driver**：變更 retire-teamstore-and-drivers。
- **使用者文件**：這個變更不動 README 與 docs。

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `node-sdk`: 移除「createEngine 的雙形式儲存建構」，新增「createEngine 只接受本機資料夾」；修改「dispatch 的輸入輸出契約」「createEngine 的建構期 actor 注入」「dispatch 的蓋章動詞」「dispatch 的流程設定動詞」「dispatch 與 handleRequest 的 fields」，拿掉只對舊物件成立的條款與情境。
- `change-plan`: 修改「change rank 動詞寫入看板順序鍵」，拿掉舊物件的例外。

## Impact

- Affected specs: node-sdk（移除一條、新增一條、修改五條）、change-plan（修改一條）
- Affected crates 與 app: `speclink-node`。`speclink-core`、`speclink-host`、`speclink-cli`、store crates、apps/desktop 不改。
- Affected code:
  - New:
    - crates/adapters/speclink-node/src/js_bridge.rs
  - Modified:
    - crates/adapters/speclink-node/src/lib.rs
    - crates/adapters/speclink-node/src/port_bridge.rs
    - crates/adapters/speclink-node/src/host.rs
    - crates/adapters/speclink-node/index.js
    - crates/adapters/speclink-node/index.d.ts
    - crates/adapters/speclink-node/__test__/helpers.ts
    - crates/adapters/speclink-node/__test__/stress.spec.ts
    - crates/adapters/speclink-node/__test__/review-verify-verbs.spec.ts
    - crates/adapters/speclink-node/__test__/workflow-config-verbs.spec.ts
    - crates/adapters/speclink-node/__test__/dispatch-contract.spec.ts
    - crates/adapters/speclink-node/__test__/write-path.spec.ts
    - crates/adapters/speclink-node/__test__/actor.spec.ts
    - crates/adapters/speclink-node/__test__/free-fields.spec.ts
    - speclink/specs/node-sdk/spec.md（只改 Purpose 那一段的一句；需求由 delta 處理）
  - Removed:
    - crates/adapters/speclink-node/src/store_bridge.rs
    - crates/adapters/speclink-node/__test__/store-bridge.spec.ts
- 相依：沒有。它只改 `speclink-node`，可以和 store-port-examples 同時做。
- 三種形態下的結果：
  - 本機 repo：可用，行為不變。`createEngine` 的本機資料夾形式照舊。
  - 連到遠端的 checkout：不適用。Node SDK 不是遠端的用戶端；連到 Server 的是 CLI 與 Speclink Desktop。
  - 沒有本機規格檔的內嵌宿主：可用，只剩一條路。宿主用 `createHost` 與三個函式；用 Store 物件的舊寫法不能再用（破壞性變更）。
- 文件影響（留給最後一個變更 docs-embedded-host，這個變更不動使用者文件）：
  - docs/sdk-node.zh-TW.md 與 docs/sdk-node.md：移除「自己實作 Store」「`claim`（選配）」兩節、31 個方法的表、錯誤碼表裡三列只屬於 Store 物件的說明、品質關卡一節裡「蓋章需要三個方法」的句子；「建立引擎」改成只有本機資料夾；「完整範例」改用 `createHost`；「目前的限制」跟著改。
  - README.md 與 README.zh-TW.md：文件一節裡「Store 介面」的說法。
  - docs/roadmap.zh-TW.md 與 docs/roadmap.md：提到「Store bridge」的兩處。
  - 發版說明：列出這一項破壞性變更與遷移方式。

## Context

Node SDK 的原生層（`crates/adapters/speclink-node`）今天有兩種後端：

- **本機資料夾**：`createEngine({ store: { type: 'fs', root } })`。它直接用 `speclink-fs` 的檔案儲存，經引擎的命令層執行。
- **宿主的 Store 物件**：`createEngine({ store: <物件> })`。原生層的 `store_bridge.rs` 把 JS 物件的 39 個方法（31 個必要、8 個選填）接成引擎的儲存介面。

另外有第三條、較新的路：`createHost` 加三個儲存函式。它用 `port_bridge.rs` 與 `host.rs`，不經 `createEngine`。

三條路共用一段「從原生執行緒呼叫 JS 函式並等結果」的機制，它今天寫在 `store_bridge.rs` 裡。所以這個檔案不能整個刪，要先把共用的部分分出來。

查證過的事實：

- 舊物件的橋接實作的是 `speclink-core` 的儲存介面，不是 TeamStore。`speclink-node` 沒有直接相依任何舊的 store crate。
- repo 裡沒有任何程式用舊物件：apps、scripts、integrations 都沒有。用它的只有 `speclink-node` 自己的 8 個測試檔（68 個案例）與一份共用的測試輔助（記憶體版的 Store 物件，約 168 行）。
- 原生層有四個只為舊物件存在的守門：缺蓋章方法、缺排序方法、自由欄位、缺流程設定寫入方法。
- `speclink-node` 沒有 Rust 測試；全部測試是 `__test__` 底下的 vitest。

## Goals / Non-Goals

**Goals:**

- `createEngine` 只接受本機資料夾；其他形狀在建構時得到指向 `createHost` 的錯誤。
- 原生層只剩一種後端，四個守門與舊物件的橋接移除。
- 仍然存在的行為都有本機資料夾形式的測試；只屬於舊物件的行為連測試一起移除，並逐項列出。

**Non-Goals:**

- 不改 `createHost`、`host.dispatch`、`host.handleRequest`、匯出匯入、`skills`、`agentTool`、`runStoreConformance` 的任何行為。
- 不改引擎（`speclink-core`）與宿主套件（`speclink-host`）。
- 不替舊物件做相容層。
- 不動使用者文件。

範圍內：`crates/adapters/speclink-node` 的原生層、JS 包裝、型別宣告與測試；正式規格 node-sdk 的 Purpose 一句。範圍外：其他 crate、apps、docs、README。

## Decisions

### D1: createEngine 的輸入檢查留在 JS 包裝

`index.js` 的 `createEngine` 今天已經做三項同步檢查（`store` 必須是物件、`actor` 必須是字串、fs 形式的 `root` 必須是字串）。新增一項：`store.type` 不是 `'fs'` 時拋出 `TypeError`，訊息是

`createEngine: only { type: 'fs', root } is accepted — use createHost for a custom store (list, read, commit)`

`store` 不存在或不是物件時的既有訊息跟著改，不再提「a Store object」。

檢查在讀 `store.type` 之後立刻做，不碰物件的其他屬性，所以傳入的物件沒有任何方法被呼叫。

替代方案：在原生層檢查。不採用——原生層的 `engineFromStore` 整個移除後，JS 包裝是唯一知道使用者傳了什麼的地方。

### D2: 呼叫 JS 函式的共用機制搬到 js_bridge.rs

`store_bridge.rs` 裡這幾樣是三條路共用的，搬到新檔 `src/js_bridge.rs`：失敗的型別（`BridgeFailure`）、呼叫的請求（`CallRequest`）、持有 JS 函式的結構與它的 `create_bridge`、`call`、`notify`。

結構從 `JsStoreBridge` 改名為 `JsBridge`：它現在也承載兩個掛鉤，名稱裡的「Store」不對。`port_bridge.rs` 與 `host.rs` 的引用跟著改。行為不變。

`store_bridge.rs` 其餘的部分（必要方法的清單、缺方法的檢查、把 JS 值轉成引擎型別的函式、約 310 行的儲存介面實作、`claim`）刪除，檔案刪除。

`index.js` 的 `makeInvoker` 有一個「方法不存在就回特殊標記」的分支。先確認 `createHost` 的路（三個函式與兩個掛鉤）用不用這個標記：不用就連同原生層讀它的地方一起移除；用就留下，並把註解改成說明誰在用。判斷的方式：移除後 `host.spec.ts`、`handle-request.spec.ts`、`port-conformance.spec.ts` 全部通過就是不用。

### D3: 原生層只剩本機資料夾一種後端

`lib.rs` 的後端列舉只剩一個成員，所以拿掉列舉，引擎的結構直接持有專案根、規格資料夾名稱與 actor。

移除：`engineFromStore` 這個匯出、四個守門（蓋章、排序、自由欄位、流程設定寫入）、`claim` 的宿主物件分支、`dispatch` 裡攔截 `BridgeFailure` 的部分中只為舊物件存在的那一段。

`claim` 改成直接交給引擎的命令層。命令層對本機儲存回的是固定的拒絕文字 `claim requires a remote store — this project uses the local fs store`，和 CLI 相同。

操作者身分的解析只剩一條：有 `actor` 用它，沒有就用專案的 git 身分。「沒有就不蓋章」的分支隨舊物件移除。

`BridgeFailure` 轉成 `dispatch` 錯誤的對照留著：`createHost` 的路還在用。

### D4: 測試的處置

原則：一個行為在這個變更之後還存在，它的測試就改用本機資料夾形式並保留斷言的行為；一個行為只屬於舊物件，它的測試刪除。改寫時，「Store 收到某個寫入呼叫」這類斷言改成「專案資料夾裡的檔案內容」。

逐檔：

| 檔案 | 處置 |
| --- | --- |
| `store-bridge.spec.ts`（6 個） | 整檔刪除 |
| `stress.spec.ts`（3 個） | 改成兩組：本機資料夾形式的並發 `dispatch`；`createHost` 加非同步的記憶體版三函式的並發 `host.dispatch`。兩組都斷言在時限內全部完成、事件迴圈期間計時器照常觸發 |
| `review-verify-verbs.spec.ts`（16 個） | 改用本機資料夾；刪「缺 deleteArtifact 時蓋章失敗」 |
| `workflow-config-verbs.spec.ts`（12 個） | 改用本機資料夾；刪「沒有寫入方法時拒絕」「讀取與 dry-run 不需要寫入方法」，新增「dry-run 不寫入」 |
| `dispatch-contract.spec.ts`（23 個） | 改用本機資料夾；刪 Store 的 claim 裁決、Store 自訂的衝突碼、Store 方法丟錯對應的錯誤碼；新增「claim 在本機專案被拒絕」 |
| `write-path.spec.ts`（4 個） | 改用本機資料夾 |
| `actor.spec.ts`（3 個） | 刪「宿主 Store 形式帶 actor」「宿主 Store 形式未給 actor 不蓋章」；本機資料夾的三個留著 |
| `free-fields.spec.ts`（1 個） | 刪「舊的 JS 儲存形式拒絕」 |
| `helpers.ts` | 刪記憶體版的 Store 物件與只有它用的型別、函式；保留本機 fixture 與 CLI 的輔助 |

刪除的行為清單（實作時逐項對照，不在清單上的不得刪）：缺必要方法時建構失敗；Store 物件的 `claim` 裁決與它的衝突碼；沒有 `actor` 時不蓋章；缺 `deleteArtifact`、`readChangeMeta`、`writeChangeMeta` 時蓋章被拒絕；缺 `writeWorkflowConfig` 時寫入被拒絕；缺 metadata 方法時 `change rank` 被拒絕；經 Store 物件的 `change rank` 把順序鍵寫進 metadata；舊物件拒絕自由欄位；Store 方法丟出的錯誤與自訂錯誤碼怎麼傳到 `dispatch`；從 Store 方法回呼引擎。

新增的測試：`createEngine` 收到帶方法的物件時同步拋出錯誤、訊息含 `createHost`、物件的方法沒有被呼叫（用每個方法都會記錄呼叫的物件驗證）。

### D5: 正式規格 node-sdk 的 Purpose

delta 只在新增 capability 時帶入 Purpose；既有 capability 的 Purpose 不由封存更新。node-sdk 的 Purpose 寫著「createEngine 的兩種儲存建構形式」。這個變更直接改正式規格檔的 Purpose 那一段，改成：

「Node 綁定對外的 SDK 契約：createEngine 的本機資料夾形式與 dispatch 的輸入輸出契約、createHost 與三個儲存函式、合格測試、Agent 工具定義，以及把引擎人眼輸出取回 JS 端的渲染 API（含 worktree 軸）。本 capability 保證 JS 端不必自行拼裝引擎內部結構，且渲染結果與 CLI 同源、不另立一套。」

只改這一段，`## Requirements` 以下不碰（需求由 delta 在封存時處理）。

## Implementation Contract

**行為**

- `createEngine({ store: { type: 'fs', root, specDir? }, actor? })`：和今天相同。
- `createEngine({ store: <任何其他物件> })`：同步拋出 `TypeError`，訊息含 `createHost`。不產生引擎實例；傳入物件的方法與屬性（`type` 除外）沒有被讀取或呼叫。
- `engine.dispatch(['claim', name])`：以 Error 拒絕，message 含 `claim requires a remote store`，專案沒有被寫入。
- 沒有給 `actor` 的引擎：蓋章的身分是專案的 git 身分。
- 蓋章、流程設定、`change rank`、自由欄位的動詞：在本機資料夾上的行為與今天相同，與 CLI 在同一個專案的結果相同。

**介面**

- `index.d.ts`：移除 `Store`、`ChangeMeta`、`ChangeInfo`、`DiscussionDoc`；`createEngine` 只剩接受 `FsStoreOptions` 的宣告；`CreateEngineOptions.store` 的型別是 `FsStoreOptions`。提到舊物件的註解（從 Store 方法回呼會死結、宿主 store 不蓋章、`change rank` 需要兩個方法、宿主 store 的 409 碼）移除或改寫。`MaybePromise` 留著。
- 原生匯出：`engineFromStore` 移除。產生的 `binding.js` 與 `binding.d.ts` 由建置重新產生，不進版控。
- `module.exports` 的七個名稱不變。

**失敗的樣子**

- 建構時的錯誤是同步的 `TypeError`，不是被拒絕的 Promise。
- `dispatch` 的錯誤碼值域不變。只由舊物件產生的錯誤碼在 `createHost` 的路上還有沒有來源，實作時確認；沒有來源的從型別宣告的註解移除，註冊表本身不動（它在 `speclink-core`）。

**驗收**

- `npm --prefix crates/adapters/speclink-node run build` 成功，`npm --prefix crates/adapters/speclink-node test` 全部通過。
- 在 `crates/adapters/speclink-node` 底下搜尋 `engineFromStore`、`JsStoreBridge`、`memoryStore`、`REQUIRED_METHODS`、`writeWorkflowConfig`、`readChangeMeta`：`src`、`index.js`、`index.d.ts`、`__test__` 都沒有命中。
- D4 的刪除清單與實際刪掉的測試逐項對得上；改寫的測試案例數與改寫前「行為仍存在」的案例數相同或更多。
- `speclink validate node-retire-legacy-store` 通過。

**範圍邊界**

- 只改 `crates/adapters/speclink-node` 與 node-sdk 正式規格的 Purpose 一段。
- 不改 `docs/`、README、CHANGELOG。

## Risks / Trade-offs

- [改寫測試時不小心丟掉仍然存在的行為] → D4 的刪除清單是封閉的；任務要求列出每個被刪案例屬於清單上的哪一項，並比對改寫前後的案例數。
- [回歸對照] → 本機資料夾形式的 `dispatch` 有與 CLI `--json` 逐欄位比對的測試（`engine.spec.ts`、`dispatch-contract.spec.ts` 的 fs 案例），這個變更不改它們的斷言。引擎的 golden 與 CLI 的整合測試不受影響：這個變更不改 `speclink-core` 與 `speclink-cli`。
- [跨平台] → 改用本機資料夾的測試會多寫暫存目錄。沿用 `helpers.ts` 既有的 fixture 函式（它已經處理 macOS 暫存目錄的符號連結）；路徑一律用 `path.join` 組；Windows 上刪暫存目錄沿用既有測試的寫法。
- [外部使用者的程式壞掉] → 這是刻意的破壞性變更。錯誤在建構時立刻出現，訊息指向 `createHost`；發版說明列出遷移方式。
- [`__missing__` 標記其實還有人用] → D2 規定先用 `createHost` 的三個測試檔驗證再移除。

## Migration Plan

- 套件的使用者：把 31 個方法的 Store 物件改寫成 `list`、`read`、`commit` 三個函式；用 `runStoreConformance` 檢查；把 `createEngine({ store })` 換成 `createHost({ store, … })`，每次呼叫給身分與範圍。
- 這個變更隨整個系列一起發版。回退方式是還原這個變更的 commit；它不改任何儲存的資料。

## Open Questions

（無）

## Why

每一次提交都帶領域事件。宿主在 `afterCommit` 或自己儲存的 `commit` 函式收到事件，換成對用戶端的推送：「哪個變更或哪份討論變了」，用戶端收到就重新讀取那個資源。現在有兩個問題：

- **有些提交改了兩種資源，卻只送其中一種的事件。** `new change --from-discussion` 把討論標成已轉出，只送 `change-created`；`archive` 連帶封存來源討論，只送 `change-archived`；`discard` 把討論從已轉出解開，只送 `change-discarded`；對已轉出的討論再次 conclude 會在變更上記下「待重新反映」，只送 `discussion-concluded`；`discuss link` 與 `discuss seal` 會改變更的 metadata，事件卻只對到討論；conclude 順手封存討論時沒有封存的事件。開著那個資源的畫面收不到通知，一直顯示舊的狀態。
- **SDK 沒有事件名稱的清單，也沒有「事件對到哪個資源」的函式。** 這張對照表在 Rust 已經有唯一的一份（宿主套件的 `event_resource`，示範用 Server 的推送用它），但 Node SDK 沒有匯出。宿主只能照文件手寫一張表；引擎新增事件時沒有任何東西提醒宿主。

目標使用者是內嵌 `@speclink/engine` 並自己做推送的宿主開發者；連帶受益的是任何依資源分開處理推送的用戶端。Speclink Desktop 收到任何事件都整批重讀，所以它現在沒有過期的畫面，這次也不改它。對應的 workflow 階段是每一個會寫入的動詞之後的畫面更新。

這是討論 embedded-host-needs-7 五個變更裡的最後一個（G1 加 G2）。

## What Changes

- 一次提交改到幾個變更與討論，就送幾個資源的事件，維持「一個事件只指一個資源」：
  - `new change --from-discussion` 在 `change-created` 之外多送 `discussion-promoted`（payload 有 `slug` 與 `change`），和 `discuss promote` 相同。
  - `archive` 每連帶封存一份討論，多送一個 `discussion-archived`（`slug`）。
  - `discuss conclude` 順手封存討論時，多送一個 `discussion-archived`（`slug`）。
  - `discard` 每解開一份討論，多送一個新事件 `discussion-unlinked`（`slug` 與 `change`）。
  - `discuss conclude` 在變更上新記下「待重新反映」時，每個變更多送一個新事件 `change-provenance-changed`（`change`）。
  - `discuss link` 真的把討論寫進變更的來源時、`discuss seal` 真的清掉變更的「待重新反映」時，各多送一個 `change-provenance-changed`（`change`）。
- 事件只在真的寫入時送：已經標記過的變更、已經連結過的討論、沒有標記可清的 seal，都不多送。
- 兩個新事件進事件對照表：`discussion-unlinked` 對到討論（`slug`），`change-provenance-changed` 對到變更（`change`）。示範用 Server 的推送自動跟著。
- 新增守門測試：對每一種會寫入的命令，檢查提交裡每個被寫到的進行中變更與討論，都有一個指到它的事件。新增命令而沒有補測試情境時，測試會失敗。
- Node SDK 新增兩個匯出：
  - `eventNames`：凍結的字串陣列，引擎的全部事件名稱加宿主套件自己的兩個（`scope-imported`、`board-order-updated`），共 39 個。
  - `eventResource(name, payload)`：回傳 `{ scope, resourceId }`，`scope` 是 `change`、`discussion`、`spec` 或 `unknown`。結果和示範用 Server 的推送用的對照相同。
- 引擎新增全部事件名稱的清單（Rust），供宿主套件與 SDK 使用，並以測試釘住它和事件種類一致。
- 不新增 CLI 指令、旗標、設定欄位；不涉及生成的技能與 Agent 指令；人眼輸出與 `--json` 輸出不變。

三種形態下的結果：

| 形態 | 結果 |
| --- | --- |
| 本機 repo | 不適用：本機專案的儲存不保存也不推送事件，Speclink Desktop 靠監看檔案更新 |
| 連到遠端的 checkout（示範用 Server） | 可用：上述提交會多推幾筆 `invalidate`；Speclink Desktop 照舊整批重讀，畫面結果不變 |
| 沒有本機規格檔的內嵌宿主 | 可用：`afterCommit` 與儲存的 `commit` 收到補齊的事件；`eventNames` 與 `eventResource` 可用 |

相容性影響：

- 上述幾種提交的事件數變多，並有兩個新的事件名稱。照文件把不認得的事件當成「整個範圍變了」的宿主，升級後行為正確，只是在加進對照之前會多讀幾次。用嚴格比對事件名稱或事件個數的宿主測試要更新。
- 既有事件的名稱、payload 與在提交裡的相對順序不變；新增的事件排在原本的事件之後。
- `eventResource` 對既有事件的結果等於 `docs/verb-contract` 的 Event stream 表格。
- 線上格式（`invalidate` 的資料形狀）不變。golden 不受影響。

掃描過的相關規格（沒有列進下面清單的）：host-kit「提交後掛鉤只能旁觀」規定事件「和交給儲存的那一份相同」，這次不改；discussion-docs 與 change-lifecycle 規定 link、seal、conclude、discard 寫入哪些文件，沒有規定事件，這次不改它們的寫入；remote-workspace-data 規定用戶端收到推送後重讀，這次不變。

## Capabilities

### New Capabilities

（無）

### Modified Capabilities

- `command-runtime`: 修改「變更型動詞的領域事件」（對照表補上跨資源的事件）；新增「一次提交的事件涵蓋它寫到的每個變更與討論」與「引擎提供全部事件名稱的清單」
- `server-event-stream`: 新增「跨資源事件的歸屬」
- `node-sdk`: 新增「SDK 匯出事件名稱清單與資源對照」

## Impact

- Affected specs: `command-runtime`、`server-event-stream`、`node-sdk`
- Affected code:
  - Modified:
    - crates/engine/speclink-core/src/command/mod.rs
    - crates/engine/speclink-core/src/command/describe.rs
    - crates/engine/speclink-core/src/command/tests.rs
    - crates/engine/speclink-core/src/lifecycle/discuss.rs
    - crates/engine/speclink-core/tests/it/command_describe.rs
    - crates/host/speclink-host/src/commit.rs
    - crates/host/speclink-host/src/kit.rs
    - crates/host/speclink-server/src/api/events.rs
    - crates/adapters/speclink-node/src/host.rs
    - crates/adapters/speclink-node/index.js
    - crates/adapters/speclink-node/index.d.ts
    - crates/adapters/speclink-node/binding.js
    - crates/adapters/speclink-node/binding.d.ts
    - crates/adapters/speclink-node/__test__/host.spec.ts
    - crates/adapters/speclink-node/__test__/engine.spec.ts
    - docs/verb-contract.md
    - docs/verb-contract.zh-TW.md
    - docs/sdk-node.md
    - docs/sdk-node.zh-TW.md
  - New:
    - crates/host/speclink-host/tests/event_coverage.rs
  - Removed: （無）

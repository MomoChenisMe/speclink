## Why

內嵌引擎的宿主在建立變更或討論時，用「這次呼叫附上的自由欄位」記下自己的資料，例如「這次呼叫是從宿主的哪個專案發起的」。宿主附上之前會先檢查呼叫者能不能從那個專案發起。但同一個鍵也能從請求寫入：`new change --field`、`discuss new --field`、`discuss promote --field`、`change field set|unset`、`discuss field set|unset`，以及入口的兩條 `PUT …/fields`。宿主這次沒有附上那個鍵時，Agent 或 CLI 寫的值照樣生效；宿主附上了，之後也能被 `field set` 改掉或被 `field unset` 刪掉。所以宿主的檢查可以被繞過，這個欄位只能當成不可信的標記。SDK 的型別說明寫自由欄位「from your own code, never from the model」，行為卻做不到。

目標使用者是內嵌 `@speclink/engine` 的宿主開發者；他們的 Agent（代表 PO、PM）與工程師的 CLI、Speclink Desktop 都會寫入同一個範圍。對應的 workflow 階段是建立變更與討論（discuss、propose 技能），以及之後任何編輯自由欄位的時候。

這是討論 embedded-host-needs-7 五個變更裡的第 4 個（G5）。

## What Changes

- `createHost` 新增選項 `reservedFieldPrefixes`（字串陣列，預設空）。鍵以其中任何一個前綴開頭的自由欄位是「宿主保留的鍵」。
- 請求要寫入或刪除宿主保留的鍵時，宿主套件拒絕這次呼叫：錯誤碼 `refused`、訊息指出那個鍵；經 HTTP 入口時回應是 409、reason 為 `refused`。拒絕發生在問 `beforeCommand` 之前，儲存沒有收到任何呼叫。
- 涵蓋每一條從請求寫欄位的路：三個建立動詞的 `--field`（`new change`、`discuss new`、`discuss promote`）、`change field set|unset`、`discuss field set|unset`，以及入口的建立變更、建立討論、轉出變更的請求本文與 `PUT /changes/{name}/fields`、`PUT /discussions/{slug}/fields`。
- 宿主保留的鍵只有宿主自己寫得了：`host.dispatch` 的 `fields` 與 `host.handleRequest` 第三個參數的 `fields`，行為和現在相同（只在建立變更或討論時寫入）。
- 請求寫了保留的鍵就拒絕，即使宿主這次也附上了同名的鍵。現在的行為是宿主的值默默蓋過；改成拒絕後，呼叫者會看到原因。
- 前綴要符合自由欄位的鍵的規則（小寫字母開頭，之後是小寫字母、數字、`_`、`.`、`-`，最長 64 個字元）；不符合時 `createHost` 直接拋出錯誤。
- 不改的部分：讀取（`list`、`show`、`discuss list` 等照常輸出保留的鍵）、封存時欄位隨文件進入封存、匯入與匯出、引擎核心對自由欄位的規則。不新增 CLI 指令、旗標、設定檔欄位；不涉及生成的技能與 Agent 指令。

三種形態下的結果：

| 形態 | 結果 |
| --- | --- |
| 本機 repo | 不適用：本機專案沒有宿主套件，自由欄位照常可寫 |
| 連到遠端的 checkout（示範用 Server） | 行為不變：示範用 Server 不設定保留的前綴 |
| 連到遠端的 checkout（內嵌宿主的入口） | 可用：CLI 與 Speclink Desktop 寫或刪保留的鍵時收到 409 與指出那個鍵的訊息；其他鍵照常 |
| 沒有本機規格檔的內嵌宿主 | 可用：Agent 經 `host.dispatch` 寫或刪保留的鍵時以 `refused` 失敗 |

相容性影響：

- 沒有設定 `reservedFieldPrefixes` 的宿主與示範用 Server：行為完全不變。
- 設定之後，原本成功的寫入（請求寫保留的鍵）會被拒絕；原本「宿主的值蓋過請求的值」在保留的鍵上改成拒絕。這是這個選項的目的。非保留的鍵仍然是宿主的值蓋過請求的值。
- 人眼輸出與 `--json` 輸出不變；線上格式不變；golden 不受影響。
- 已知缺口（這次不做）：保留的鍵建立之後連宿主也不能改值，因為宿主附上的欄位只在建立時寫入；`host.importScope` 與入口的 `POST /import` 寫入的是整份文件，不檢查裡面的保留鍵。

掃描過的相關規格（沒有列進下面清單的）：free-fields「自由欄位的規則與存放」規定引擎只存放、不解讀任何鍵，這次不改它——規則由宿主宣告、由宿主套件執行，引擎核心仍然不認得任何鍵；free-fields「自由欄位的寫入動詞」的本機行為不變；client-protocol 的錯誤 reason 清單已有 `refused`。

## Capabilities

### New Capabilities

（無）

### Modified Capabilities

- `host-kit`: 新增「宿主保留的自由欄位鍵」；修改「呼叫入口時附上自由欄位」（同名時宿主的值蓋過請求的值，保留的鍵例外）
- `node-sdk`: 新增「createHost 的保留欄位前綴」

## Impact

- Affected specs: `host-kit`、`node-sdk`
- Affected code:
  - Modified:
    - crates/engine/speclink-core/src/command/fields.rs
    - crates/host/speclink-host/src/kit.rs
    - crates/host/speclink-host/tests/free_fields.rs
    - crates/host/speclink-host/tests/http_door.rs
    - crates/adapters/speclink-node/src/host.rs
    - crates/adapters/speclink-node/index.js
    - crates/adapters/speclink-node/index.d.ts
    - crates/adapters/speclink-node/__test__/free-fields.spec.ts
    - docs/sdk-node.md
    - docs/sdk-node.zh-TW.md
  - New: （無）
  - Removed: （無）

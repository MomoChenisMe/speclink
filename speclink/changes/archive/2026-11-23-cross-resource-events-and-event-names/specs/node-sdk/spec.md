## ADDED Requirements

### Requirement: SDK 匯出事件名稱清單與資源對照

`@speclink/engine` SHALL 匯出 `eventNames`：一個凍結的字串陣列，含引擎每一種領域事件的名稱，以及宿主套件自己的兩個事件名稱 `scope-imported`（host.importScope 與入口的匯入）與 `board-order-updated`（host.handleRequest 寫看板排序），每個名稱正好一次。宿主的 afterCommit 掛鉤與儲存的 commit 函式收到的每一筆事件的 `name` SHALL 一定是這個陣列裡的一個名稱。陣列裡名稱的順序 SHALL NOT 是契約的一部分。型別定義檔 SHALL 把它宣告成唯讀的字串陣列。

`@speclink/engine` SHALL 另外匯出函式 `eventResource(name, payload)`，回傳 `{ scope, resourceId }`：這筆事件指的資源。`scope` SHALL 是 `change`、`discussion`、`spec` 或 `unknown` 其中之一；`resourceId` SHALL 是字串。結果 SHALL 與宿主套件唯一的那張對照相同，也就是示範用 Server 推送 invalidation 時用的 scope 與 resourceId：變更的事件、兩個品質關卡的事件與落地回報歸到 `change`，resourceId 是 payload 的 `change`；`change-archived` 歸到 `spec`，resourceId 是 payload 的 `change`；討論的事件歸到 `discussion`，resourceId 是 payload 的 `slug`；其餘的名稱（包含 `scope-imported`、`board-order-updated`、`language-replaced`、`workflow-config-changed` 與不在 `eventNames` 裡的名稱）SHALL 回 `{ scope: 'unknown', resourceId: '' }`。payload 沒有對應的字串鍵時，`resourceId` SHALL 是空字串，`scope` SHALL 維持該事件的類別。`name` 不是字串時 SHALL 同步拋出錯誤；`payload` SHALL 接受任何 JSON 值。這個函式 SHALL NOT 讀取或寫入儲存。型別定義檔 SHALL 宣告它的參數與回傳型別。

#### Scenario: 匯入得到全部事件名稱

- **WHEN** 以 `const { eventNames } = require('@speclink/engine')` 取得清單
- **THEN** 它是 39 個字串的陣列，沒有重複；含 `change-created`、`discussion-promoted`、`discussion-unlinked`、`change-provenance-changed`、`workflow-config-changed`、`scope-imported` 與 `board-order-updated`；`Object.isFrozen(eventNames)` 是 `true`

#### Scenario: 掛鉤收到的事件名稱都在清單裡

- **WHEN** 宿主的 afterCommit 記下每次收到的事件名稱，之後依序經 host.dispatch 執行 `new change add-two --from-discussion two`、`task done`、`discard add-two`，再呼叫 host.importScope 匯入另一個空的範圍，並經 host.handleRequest 送一個合法的 `PUT /board-order`
- **THEN** 記下的每一個名稱都在 `eventNames` 裡，其中有 `discussion-promoted`、`discussion-unlinked`、`scope-imported` 與 `board-order-updated`

#### Scenario: 事件對到資源

- **WHEN** 以下表的參數呼叫 eventResource
- **THEN** 回傳值如表

##### Example: 各種事件的資源

| `name` | `payload` | 回傳值 |
| --- | --- | --- |
| `change-created` | `{ change: 'add-two' }` | `{ scope: 'change', resourceId: 'add-two' }` |
| `review-round-added` | `{ change: 'add-two', round: 2 }` | `{ scope: 'change', resourceId: 'add-two' }` |
| `change-archived` | `{ change: 'add-two', datedName: '2026-11-22-add-two' }` | `{ scope: 'spec', resourceId: 'add-two' }` |
| `discussion-promoted` | `{ slug: 'two', change: 'add-two' }` | `{ scope: 'discussion', resourceId: 'two' }` |
| `discussion-unlinked` | `{ slug: 'two', change: 'add-two' }` | `{ scope: 'discussion', resourceId: 'two' }` |
| `change-provenance-changed` | `{ change: 'add-two' }` | `{ scope: 'change', resourceId: 'add-two' }` |
| `workflow-config-changed` | `{ keys: ['tdd'] }` | `{ scope: 'unknown', resourceId: '' }` |
| `scope-imported` | `{ documents: 3 }` | `{ scope: 'unknown', resourceId: '' }` |
| `not-an-event` | `{}` | `{ scope: 'unknown', resourceId: '' }` |
| `change-created` | `{}` | `{ scope: 'change', resourceId: '' }` |

#### Scenario: 用掛鉤收到的事件直接查資源

- **WHEN** 宿主在 afterCommit 對收到的每一筆事件呼叫 `eventResource(event.name, event.payload)`，之後經 host.dispatch 執行 `new change add-two --from-discussion two`
- **THEN** 得到的資源依序是 `{ scope: 'change', resourceId: 'add-two' }` 與 `{ scope: 'discussion', resourceId: 'two' }`

#### Scenario: name 不是字串

- **WHEN** 執行 eventResource(42, {}) 與 eventResource(undefined, {})
- **THEN** 兩次都同步拋出錯誤

#### Scenario: 型別定義檔宣告了兩個匯出

- **WHEN** 一支 TypeScript 檔匯入 `eventNames` 與 `eventResource`，把 `eventNames` 指派給型別為 `readonly string[]` 的變數，並把 `eventResource('change-created', { change: 'x' }).scope` 指派給型別為 `'change' | 'discussion' | 'spec' | 'unknown'` 的變數，以 `strict`、不開 `skipLibCheck` 編譯
- **THEN** 編譯沒有錯誤

## MODIFIED Requirements

### Requirement: SDK 匯出命令名稱清單

<!-- BEFORE: 宿主套件自己的命令名稱有三個，清單共 57 個 -->

`@speclink/engine` SHALL 匯出 `commandNames`：一個凍結的字串陣列，含引擎每一種命令的名稱，以及宿主套件自己的七個命令名稱：`scope-export`、`scope-import`、`put-board-order`（host.handleRequest 寫看板排序時用的名稱），以及 host.handleRequest 的四條讀取用的 `get-board-order`（`GET /board-order`）、`workspace-search`（`GET /search`）、`context-snapshot`（`POST /context`）與 `spec-drift`（`GET /changes/{name}/drift`），每個名稱正好一次。宿主的 beforeCommand 掛鉤與儲存的 commit 函式收到的 `command` SHALL 一定是這個陣列裡的一個名稱。陣列裡名稱的順序 SHALL NOT 是契約的一部分。`commandNames` SHALL NOT 帶命令的其他性質（例如會不會寫入）。型別定義檔 SHALL 把它宣告成唯讀的字串陣列。

#### Scenario: 匯入得到全部名稱

- **WHEN** 以 `const { commandNames } = require('@speclink/engine')` 取得清單
- **THEN** 它是 61 個字串的陣列，沒有重複；含 `new-change`、`claim-release`、`workflow-config-edit`、`scope-export`、`scope-import`、`put-board-order`、`get-board-order`、`workspace-search`、`context-snapshot` 與 `spec-drift`

#### Scenario: 清單不能被改動

- **WHEN** 檢查 `Object.isFrozen(commandNames)`
- **THEN** 結果是 `true`

#### Scenario: 掛鉤收到的名稱都在清單裡

- **WHEN** 宿主的 beforeCommand 記下每次收到的 `command`，之後依序經 host.dispatch 執行 `new change demo`、`claim demo`、`claim demo --release`，再呼叫 host.exportScope 與 host.importScope
- **THEN** 記下的名稱依序是 `new-change`、`claim`、`claim-release`、`scope-export`、`scope-import`，每一個都在 `commandNames` 裡

#### Scenario: 看板排序的寫入也用清單裡的名稱

- **WHEN** 宿主的 beforeCommand 記下收到的 `command`，之後經 host.handleRequest 送一個合法的 `PUT /board-order`
- **THEN** 記下的名稱是 `put-board-order`，它在 `commandNames` 裡；儲存的 commit 函式收到的 `command` 也是 `put-board-order`

#### Scenario: 型別定義檔宣告了這個匯出

- **WHEN** 一支 TypeScript 檔以 `import { commandNames } from '@speclink/engine'` 匯入，並把 `commandNames` 指派給型別為 `readonly string[]` 的變數，以 `strict`、不開 `skipLibCheck` 編譯
- **THEN** 編譯沒有錯誤

#### Scenario: 入口的讀取也用清單裡的名稱

- **WHEN** 宿主的 beforeCommand 記下收到的 `command` 並一律放行，之後經 host.handleRequest 依序送 `GET /board-order`、`GET /search?q=auth`、`POST /context`、`GET /config`、`GET /archived`
- **THEN** 記下的名稱依序是 `get-board-order`、`workspace-search`、`context-snapshot`、`workflow-config-show`、`list`，每一個都在 `commandNames` 裡；儲存的 commit 函式沒有被呼叫

#### Scenario: 只放行舊名稱的掛鉤擋下新的讀取

- **WHEN** 宿主的 beforeCommand 只放行升級前的 57 個名稱、其餘回答拒絕並給原因「這個命令還沒有開放」，經 host.handleRequest 送 `GET /search?q=auth`
- **THEN** 回應是 409、reason 為 refused、訊息是「這個命令還沒有開放」

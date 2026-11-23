## ADDED Requirements

### Requirement: createHost 的保留欄位前綴

createHost SHALL 接受選填的 `reservedFieldPrefixes`（字串陣列）。它的行為 SHALL 等同宿主套件「宿主保留的自由欄位鍵」：鍵以其中任何一個前綴開頭的自由欄位，只有 host.dispatch 的 `fields` 與 host.handleRequest 第三個參數的 `fields` 寫得了。argv 或 HTTP 請求要寫入或刪除這種鍵時，host.dispatch 的 Promise SHALL 以 `code` 為 `refused` 的 Error 拒絕，`message` SHALL 是 `field '<key>' is reserved by the host`；host.handleRequest SHALL 回狀態碼 409、reason 為 refused 的回應。被拒絕時儲存的 `list`、`read`、`commit` SHALL 都沒有被呼叫，beforeCommand 與 afterCommit SHALL 都沒有被呼叫。

`reservedFieldPrefixes` 不是陣列、陣列裡有不是字串的項目、或有不合自由欄位鍵規則的前綴（空字串、大寫字母、空白等）時，createHost SHALL 同步拋出錯誤，訊息 SHALL 指出 `reservedFieldPrefixes`。省略這個選項或給空陣列時，createHost、host.dispatch 與 host.handleRequest 的行為 SHALL 與沒有這條需求時相同。`reservedFieldPrefixes` SHALL 是 createHost 認得的選項名稱；型別定義檔 SHALL 把它宣告成選填的字串陣列。

#### Scenario: dispatch 拒絕請求寫保留的鍵

- **WHEN** 以 `createHost({ store, reservedFieldPrefixes: ['host.'] })` 建立 host，執行 `host.dispatch({ identity, scope, argv: ['new', 'change', 'spoof', '--field', 'host.project=p2'] })`
- **THEN** Promise 以 Error 拒絕，`code` 是 `refused`，`message` 是 `field 'host.project' is reserved by the host`；儲存的 `commit` 沒有被呼叫

#### Scenario: 宿主的 fields 寫得了保留的鍵

- **WHEN** 同一個 host 執行 `host.dispatch({ identity, scope, argv: ['new', 'change', 'ok', '--json'], fields: { 'host.project': 'p1' } })`，之後執行 `['list', '--json']`
- **THEN** 建立成功；清單裡 ok 的 `fields` 是 `{ 'host.project': 'p1' }`

#### Scenario: 設定與移除都被拒絕

- **WHEN** 同一個 host 依序執行 argv `['change', 'field', 'set', 'ok', 'host.project=p2']` 與 `['change', 'field', 'unset', 'ok', 'host.project']`
- **THEN** 兩個 Promise 都以 `code` 為 `refused` 的 Error 拒絕；ok 的 `host.project` 仍是 `p1`

#### Scenario: handleRequest 回 409

- **WHEN** 同一個 host 經 handleRequest 收到 `PUT /changes/ok/fields`，本文要把 `host.project` 設成 `p2`
- **THEN** 回應的狀態碼是 409，本文的 reason 是 refused，訊息指出 `host.project`

#### Scenario: beforeCommand 沒有被問

- **WHEN** host 另外設了記錄呼叫的 beforeCommand，執行 argv `['new', 'change', 'spoof', '--field', 'host.project=p2']`
- **THEN** Promise 以 `refused` 拒絕；beforeCommand 的呼叫次數是 0

#### Scenario: 選項的值不合法

- **WHEN** 分別以 `reservedFieldPrefixes: 'host.'`、`['host.', 7]`、`['']` 與 `['Host.']` 呼叫 createHost
- **THEN** 四次都同步拋出錯誤，訊息含 `reservedFieldPrefixes`，沒有建立 host

#### Scenario: 省略時行為不變

- **WHEN** 以沒有 `reservedFieldPrefixes` 的選項建立 host，執行 argv `['new', 'change', 'demo', '--field', 'host.project=p2', '--json']`
- **THEN** 建立成功，demo 的 `fields` 有 `host.project`

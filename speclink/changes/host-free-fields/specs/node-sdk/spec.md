## ADDED Requirements

### Requirement: dispatch 與 handleRequest 的 fields

host.dispatch 的呼叫物件 SHALL 接受選填的 `fields`（物件，字串對字串）；host.handleRequest SHALL 接受選填的第三個參數，其 `fields` 欄位同型。兩者的行為 SHALL 等同宿主套件「呼叫入口時附上自由欄位」：只在建立變更或討論時生效、蓋過同名的值、其他命令忽略。`fields` 的任一值不是字串時，呼叫 SHALL 以 code 為 invalid_argv 的 Error（dispatch）或狀態碼 400 的回應（handleRequest）結束，儲存 SHALL 沒有變動。host.dispatch SHALL 接受自由欄位的四個動詞與兩個建立動詞的 `--field` 旗標，回傳值與 CLI 的 `--json` 相同。engine.dispatch 的 fs 形式 SHALL 同樣接受這些動詞；以即將移除的 JS 儲存形式建立的引擎 SHALL 以 code 為 refused 的 Error 在動手前拒絕它們，訊息 SHALL 指向 createHost。

#### Scenario: dispatch 附上欄位建立變更

- **WHEN** 以 `host.dispatch({ identity, scope, argv: ['new', 'change', 'demo'], fields: { source: 'portal' } })` 執行
- **THEN** 儲存的 commit 被呼叫正好一次，寫入的 metadata 含 source；beforeCommand 收到的 params.fields 含 source

#### Scenario: 每次呼叫都帶也不影響其他動詞

- **WHEN** 以同樣的 fields 執行 `['list', '--json']`
- **THEN** 回傳清單，commit 沒有被呼叫

#### Scenario: handleRequest 的第三個參數

- **WHEN** 以 `handleRequest(建立變更的請求, identity, { fields: { source: 'portal' } })` 執行
- **THEN** 回應是成功；建立的變更有 source=portal

#### Scenario: 值不是字串

- **WHEN** 以 `fields: { n: 1 }` 呼叫 host.dispatch
- **THEN** Promise 以 code 為 invalid_argv 的 Error 拒絕；commit 沒有被呼叫

#### Scenario: 欄位動詞經 dispatch

- **WHEN** 執行 `host.dispatch({ …, argv: ['change', 'field', 'set', 'demo', 'ticket=PROJ-1', '--json'] })`
- **THEN** 回傳 `{ change: 'demo', fields: { ticket: 'PROJ-1' } }`

#### Scenario: 舊的 JS 儲存形式拒絕

- **WHEN** 以 JS 儲存形式建立的 engine 執行 `['change', 'field', 'set', 'demo', 'a=1']`
- **THEN** Promise 以 code 為 refused 的 Error 拒絕，訊息提到 createHost；儲存的任何寫入方法都沒有被呼叫

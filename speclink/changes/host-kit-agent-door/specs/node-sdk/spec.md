## ADDED Requirements

### Requirement: createHost 與 host.dispatch

Node SDK SHALL 匯出 createHost，它接受一個實作儲存接口三個函式的 JS 物件與選填的掛鉤集合，回傳宿主套件。儲存物件缺少任何一個函式時，createHost SHALL 同步拋出錯誤並列出缺少的函式名稱。host.dispatch SHALL 接受一個物件：身分（識別與顯示名稱）、範圍、argv 字串陣列、選填的 stdin 內容；回傳 Promise。argv 的詞彙、stdin 的規則、成功時的回傳值與失敗時的 Error 形狀（message 加 code 欄位）SHALL 和 engine.dispatch 相同，涵蓋全部有引擎命令的動詞；需要 checkout 的動詞 SHALL 以 `invalid_argv` 拒絕。身分 SHALL 由每次呼叫給，createHost SHALL NOT 接受建立期的操作者；argv 與 stdin SHALL NOT 能影響身分。身分或範圍的格式不合時 SHALL 以 `invalid_argv` 拒絕且不呼叫儲存。儲存層的失敗 SHALL 以這三個 code 呈現：版本衝突是 `revision_conflict`、暫時無法使用是 `unavailable`、其他是 `internal`。host.dispatch SHALL 於背景工作執行，SHALL NOT 阻塞 JS 事件迴圈，多個呼叫 SHALL 能同時進行。

掛鉤的 JS 形狀：beforeCommand 收到 identity、scope、command、params，以及流程設定寫入時的 changedKeys；回傳 undefined 或 `{ allow: true }` 是放行，回傳 `{ allow: false, reason }` 是拒絕；回傳值或 Promise 皆可。beforeCommand 拋出例外、Promise 被拒絕或回傳其他形狀時，這次呼叫 SHALL 以 `internal` 結束並在訊息指出是 beforeCommand，儲存 SHALL 沒有變動。afterCommit 收到 identity、scope、command、scopeVersion、events；host.dispatch 的 Promise SHALL NOT 等待它，它拋出的例外與被拒絕的 Promise SHALL NOT 影響呼叫的結果，也 SHALL NOT 成為未處理的 rejection。

#### Scenario: 經宿主套件建立與列出變更

- **WHEN** 以 JS 的記憶體儲存 createHost，以身分 `{ id: 'u_1', displayName: 'Alice' }` 在範圍 `s1` dispatch `['new', 'change', 'demo']`，再 dispatch `['list', '--json']`
- **THEN** 第二次的回傳物件含 demo，欄位名（camelCase）與 CLI 的 `speclink list --json` 一致；儲存的 commit 函式正好被呼叫一次，收到的 actor 是 `{ id: 'u_1', displayName: 'Alice' }`

#### Scenario: 結果與 engine.dispatch 相同

- **WHEN** 對內容相同的 fs 專案與 JS 記憶體儲存，分別以 engine.dispatch 與 host.dispatch 執行 `['status', '--change', 'demo', '--json']`
- **THEN** 兩個回傳物件深度相等

#### Scenario: 同一個宿主套件服務兩個身分

- **WHEN** 同一個 host 先以 Alice、再以 Bob 的身分各 dispatch 一次 `new change`
- **THEN** 兩個變更的 created_by 分別是 Alice 與 Bob 的顯示名稱

#### Scenario: 缺少函式時建立即失敗

- **WHEN** 以沒有 commit 函式的物件呼叫 createHost
- **THEN** 同步拋出錯誤，訊息列出 commit，不產生宿主套件

#### Scenario: 掛鉤拒絕時以 refused 失敗

- **WHEN** beforeCommand 對 `archive` 回傳 `{ allow: false, reason: '程式碼還沒合併' }`，dispatch `['archive', 'demo']`
- **THEN** Promise 以 Error 拒絕，code 是 `refused`，message 是「程式碼還沒合併」；儲存的 commit 沒有被呼叫

#### Scenario: 非同步的掛鉤

- **WHEN** beforeCommand 是回傳 Promise 的 async 函式，稍後才解析為放行
- **THEN** dispatch 等它解析後照常執行並成功

#### Scenario: 掛鉤拋出例外時不放行

- **WHEN** beforeCommand 拋出例外，dispatch `['new', 'change', 'demo']`
- **THEN** Promise 以 Error 拒絕，code 是 `internal`，message 含 beforeCommand；儲存的 commit 沒有被呼叫

#### Scenario: 流程設定的寫入帶 changedKeys

- **WHEN** dispatch `['workflow-config', 'set', 'tdd', 'true']`
- **THEN** beforeCommand 收到 command `workflow-config-edit` 與 changedKeys `['tdd']`

#### Scenario: afterCommit 收到事件且失敗不影響結果

- **WHEN** afterCommit 是會拋出例外的函式，dispatch `['new', 'change', 'demo']`
- **THEN** afterCommit 被呼叫一次，收到 command `new-change`、數字的 scopeVersion 與一筆 change-created 事件；dispatch 的 Promise 照常解析為成功；程序沒有出現未處理的 rejection

#### Scenario: 儲存的版本衝突

- **WHEN** 儲存的 commit 拋出 code 為 `conflict`、key 為這次提交其中一個鍵的錯誤
- **THEN** dispatch 的 Promise 以 Error 拒絕，code 是 `revision_conflict`，message 含該鍵

#### Scenario: 儲存暫時無法使用

- **WHEN** 儲存的 read 拋出 code 為 `unavailable` 的錯誤
- **THEN** dispatch 的 Promise 以 Error 拒絕，code 是 `unavailable`；commit 沒有被呼叫

#### Scenario: 需要 checkout 的動詞被拒絕

- **WHEN** dispatch `['drift', 'demo']`
- **THEN** Promise 以 Error 拒絕，code 是 `invalid_argv`；儲存沒有被呼叫

#### Scenario: 空的識別被拒絕

- **WHEN** 以 `{ id: '', displayName: 'Alice' }` 的身分 dispatch 任何動詞
- **THEN** Promise 以 Error 拒絕，code 是 `invalid_argv`；儲存沒有被呼叫

#### Scenario: 蓋章不需要額外的儲存方法

- **WHEN** 儲存只有 list、read、commit 三個函式，對一個任務全部完成、末輪沒有必修發現的變更 dispatch `['review', 'stamp', 'demo', '--stdin']` 並以 stdin 帶入 scope 的 JSON
- **THEN** 蓋章成功；commit 正好被呼叫一次，同一次提交裡含改寫後的 metadata 與工單文件的刪除

### Requirement: JS 版的儲存接口

宿主的 JS 儲存 SHALL 是三個函式，回傳值或 Promise 皆可，欄位一律 camelCase：

- `list(scope, prefix)` 回傳 `{ version, entries }`，entries 的每一筆是 `{ key, version, modifiedAt }`。
- `read(scope, keys)` 回傳和 keys 等長、順序相同的陣列，每一格是 `{ content, version, modifiedAt }` 或 null（文件不存在）。
- `commit(scope, commit)` 回傳提交後的範圍版本號（數字）。commit 物件含 `actor`（`{ id, displayName }`）、`command`（字串）、`at`（數字，毫秒）、`ops` 與 `events`（每筆 `{ name, payload }`）。ops 的每一筆是 `{ op: 'put', key, content, expected }` 或 `{ op: 'delete', key, expected }`；put 的 expected 是數字（預期的文件版本）或 null（必須不存在），delete 的 expected 是數字。

版本與時間 SHALL 是 0 到 9007199254740991 的整數。這三個函式的語意 SHALL 等同 store-port-contract 規定的列出、讀取、提交；領域事件 SHALL 只經 commit 的 events 交給宿主。

宿主函式拋出的錯誤 SHALL 依它的 `code` 屬性分類：commit 拋出 `code` 為 `conflict` 且 `key` 是這次提交其中一個鍵時是版本衝突，實際版本取自錯誤的 `actual`（數字或 null）；`code` 為 `unavailable` 時是暫時無法使用；其餘一律是其他失敗。SDK SHALL 匯出 StoreConflictError 與 StoreUnavailableError 兩個設好 `code` 的 Error 類別，分類 SHALL NOT 依賴 instanceof。宿主函式的回傳值不合格時——read 的陣列長度不等於鍵的數量、版本或時間不是範圍內的整數、list 回傳的鍵不合鍵的格式——SHALL 視為其他失敗，訊息 SHALL 指出是哪個函式與哪個欄位，這次命令 SHALL NOT 提交。

#### Scenario: 寫入的動詞只呼叫一次 commit 並帶事件

- **WHEN** 經 host.dispatch 執行 `['task', 'done', '1.1', '--change', 'demo']` 成功
- **THEN** 儲存的 commit 被呼叫正好一次；commit 物件的 command 是 `task-done`，ops 含 tasks 文件的 put，events 含一筆 name 為 task-completed 的事件

#### Scenario: 新文件的 expected 是 null

- **WHEN** 經 host.dispatch 建立一個新的變更
- **THEN** commit 物件裡該變更 metadata 的 put，expected 是 null

#### Scenario: 既有文件的 expected 是讀到的版本

- **WHEN** 儲存裡 tasks 文件的版本是 7，經 host.dispatch 勾選該文件的一個任務
- **THEN** commit 物件裡該文件的 put，expected 是 7

#### Scenario: 查詢不呼叫 commit

- **WHEN** 經 host.dispatch 執行 `['list', '--json']`
- **THEN** 儲存的 commit 沒有被呼叫

#### Scenario: read 回傳的長度不對

- **WHEN** 儲存的 read 對兩個鍵只回傳一格
- **THEN** dispatch 以 code `internal` 失敗，message 指出 read 與陣列長度；commit 沒有被呼叫

#### Scenario: 版本不是整數

- **WHEN** 儲存的 list 回傳的 version 是 1.5
- **THEN** dispatch 以 code `internal` 失敗，message 指出 list 與 version

#### Scenario: 不用錯誤類別也能回報衝突

- **WHEN** 儲存的 commit 拋出一個普通物件 `{ code: 'conflict', key, actual: 9 }`，key 是這次提交的一個鍵
- **THEN** dispatch 以 code `revision_conflict` 失敗，message 含該鍵與實際版本 9

#### Scenario: 衝突的鍵不在這次提交裡

- **WHEN** 儲存的 commit 拋出 code 為 `conflict`、key 不屬於這次提交的錯誤
- **THEN** dispatch 以 code `internal` 失敗

### Requirement: 合格測試隨套件提供

Node SDK SHALL 匯出 runStoreConformance，讓宿主對自己的 JS 儲存執行儲存契約的合格測試。它接受一個物件：受測的 store、每次回傳一個沒用過的範圍的 newScope、選填的 recordedEvents（回傳某個範圍已經收到的事件，每筆含 name、payload 與所屬提交的 version）；回傳 Promise，解析為 `{ passed, checks }`，checks 的每一筆是 `{ name, outcome, detail }`，outcome 是 `passed`、`failed` 或 `skipped`。檢查的項目與名稱 SHALL 和 store-port-contract 規定的合格測試相同，同一份檢查 SHALL 同時用於 Rust 實作與 JS 實作。沒有提供 recordedEvents 時，檢查事件的項目 SHALL 是 `skipped` 並在 detail 說明原因。passed SHALL 在沒有任何一項是 `failed` 時為 true。runStoreConformance SHALL NOT 阻塞 JS 事件迴圈。

#### Scenario: 正確的 JS 儲存通過全部檢查

- **WHEN** 對一個正確實作三個函式並提供 recordedEvents 的 JS 記憶體儲存執行 runStoreConformance
- **THEN** passed 是 true；checks 有 16 筆，名稱與 Rust 的合格測試相同，全部是 `passed`

#### Scenario: 不比對版本的儲存被判定失敗

- **WHEN** 對一個 commit 忽略 expected、一律寫入的 JS 儲存執行 runStoreConformance
- **THEN** passed 是 false；`conflict-stale-version` 這一項是 `failed`，detail 說明預期被拒絕卻寫入成功

#### Scenario: 沒有 recordedEvents 時事件的檢查略過

- **WHEN** 對一個正確的 JS 儲存執行 runStoreConformance，不提供 recordedEvents
- **THEN** `events-with-commit` 與 `events-only-commit` 是 `skipped` 且 detail 說明沒有提供 recordedEvents；其餘是 `passed`；passed 是 true

#### Scenario: 非同步的儲存

- **WHEN** 受測儲存的三個函式都回傳稍後才解析的 Promise
- **THEN** runStoreConformance 照常完成，並行的兩項檢查也有結果

### Requirement: Agent 工具定義的匯出

Node SDK SHALL 匯出常數 agentTool，含三個欄位：`name`（固定是 `speclink`）、`description`（英文，給模型看的說明）、`inputSchema`（JSON Schema）。inputSchema SHALL 描述一個物件：必填的 `argv`（字串陣列）與選填的 `stdin`（字串），不接受其他屬性；身分與範圍 SHALL NOT 出現在 inputSchema。description SHALL 說明 argv 是不含程式名稱的動詞文字、帶 `--stdin` 的動詞把內容放在 stdin、加 `--json` 取得結構化的結果，並 SHALL 列出每一個有引擎命令的末端動詞與它的一行說明。動詞清單 SHALL 由和 host.dispatch 相同的動詞定義產生：清單裡的每個動詞 SHALL 能被 host.dispatch 解析；需要 checkout 的動詞 SHALL NOT 出現在清單。

#### Scenario: 輸入格式只有 argv 與 stdin

- **WHEN** 檢視 agentTool.inputSchema
- **THEN** 它是型別為 object 的 JSON Schema，properties 只有 argv 與 stdin，required 是 `['argv']`，additionalProperties 是 false

#### Scenario: 說明列出的動詞都能執行

- **WHEN** 取出 agentTool.description 的動詞清單，對每一個動詞以它的動詞文字加 `--help` 以外的最少參數交給共用的解析
- **THEN** 清單的動詞集合等於「有引擎命令的末端動詞」集合；需要 checkout 的動詞（例如 `drift`、`review prepare`、`review scope`、`verify scope`）不在清單

#### Scenario: 工具的輸入直接交給 host.dispatch

- **WHEN** 宿主收到模型依 inputSchema 產生的輸入 `{ argv: ['discuss', 'list', '--json'] }`，加上自己的身分與範圍呼叫 host.dispatch
- **THEN** 回傳討論清單的結構化物件

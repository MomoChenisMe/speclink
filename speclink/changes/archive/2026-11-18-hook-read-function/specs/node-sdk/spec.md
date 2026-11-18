## ADDED Requirements

### Requirement: beforeCommand 的讀取函式

beforeCommand SHALL 收到第二個參數：一個物件，含函式 `read`。`read(argv)` 接受 argv 字串陣列（詞彙與 host.dispatch 相同），回傳 Promise；成功時的值 SHALL 與以同一個身分、同一個範圍呼叫 host.dispatch 執行同一個 argv 的值深度相等，失敗時 SHALL 以 Error 拒絕，Error 的形狀（message 加 code 欄位）與 host.dispatch 相同。第一個參數（identity、scope、command、params，以及流程設定寫入時的 changedKeys）SHALL 不變；只宣告一個參數的 beforeCommand SHALL 照常運作。

`read` SHALL 遵守宿主套件讀取器的規則（見 host-kit「執行前掛鉤可以用讀取器讀取」）：

- argv 不是讀取類的動詞時 SHALL 以 code `invalid_argv` 拒絕，message SHALL 指出 read 只接受讀取類的動詞並點名該動詞；儲存的三個函式 SHALL NOT 因為這次呼叫被呼叫。
- 經 `read` 執行的動詞 SHALL NOT 讓 beforeCommand 再被呼叫，SHALL NOT 呼叫 afterCommit，儲存的 commit SHALL NOT 被呼叫。
- `read` 沒有身分與範圍的參數；它 SHALL 使用這次呼叫的身分與範圍。
- `read` SHALL 於背景工作執行，SHALL NOT 阻塞 JS 事件迴圈；同一次 beforeCommand 裡 SHALL 能呼叫多次，也能同時進行。
- `read` SHALL 只在這次 beforeCommand 執行期間有效：beforeCommand 回傳的值或 Promise 完成之後再呼叫 `read`，SHALL 以 code `invalid_argv` 拒絕，儲存 SHALL NOT 被呼叫。

beforeCommand 沒有接住 `read` 的拒絕而讓它往外拋時，這次呼叫 SHALL 依既有規則以 `internal` 結束並在訊息指出是 beforeCommand。host.dispatch 與 host.handleRequest 進來的呼叫 SHALL 都給 beforeCommand 這個函式。

index.d.ts SHALL 宣告第二個參數的型別，並 SHALL 在 beforeCommand 的說明寫明：要在掛鉤裡讀資料時使用 `read`；在掛鉤裡等同一個宿主套件的 dispatch 會再問一次 beforeCommand，而且宿主的儲存自己持有鎖時會卡住。index.d.ts SHALL NOT 再宣稱在掛鉤裡等 dispatch 永遠不會完成。

#### Scenario: 用 read 讀出狀態後拒絕封存

- **WHEN** beforeCommand 是 `async (info, { read }) => { if (info.command !== 'archive') return; const shown = await read(['show', info.params.change, '--json']); if (shown.review?.state !== 'stamped') return { allow: false, reason: '還沒通過審查' } }`，變更 demo 的任務全部完成、沒有審查章，dispatch `['archive', 'demo']`
- **THEN** Promise 以 Error 拒絕，code 是 `refused`，message 是「還沒通過審查」；儲存的 commit 沒有被呼叫；beforeCommand 正好被呼叫一次

#### Scenario: read 的值與 host.dispatch 相同

- **WHEN** beforeCommand 把 `await read(['show', 'demo', '--json'])` 的值存起來並放行；之後以同一個身分與範圍直接 dispatch `['show', 'demo', '--json']`
- **THEN** 兩個物件深度相等

#### Scenario: read 拒絕會寫入的動詞

- **WHEN** beforeCommand 裡執行 `await read(['new', 'change', 'other'])` 並接住錯誤
- **THEN** 接住的 Error 的 code 是 `invalid_argv`，message 含 `new change`；儲存的 list、read、commit 都沒有因為這次呼叫被呼叫

#### Scenario: 沒有接住 read 的錯誤時以 internal 結束

- **WHEN** beforeCommand 裡執行 `await read(['show', 'ghost', '--json'])` 而沒有接住錯誤，範圍裡沒有 ghost，dispatch `['list', '--json']`
- **THEN** dispatch 的 Promise 以 Error 拒絕，code 是 `internal`，message 指出是 beforeCommand；儲存的 commit 沒有被呼叫

#### Scenario: 掛鉤結束後 read 失效

- **WHEN** beforeCommand 把 `read` 存到外面的變數並放行；dispatch 完成之後，程式呼叫那個存下來的 `read(['list', '--json'])`
- **THEN** 回傳的 Promise 以 Error 拒絕，code 是 `invalid_argv`；儲存的函式沒有因為這次呼叫被呼叫

#### Scenario: 只宣告一個參數的掛鉤照常運作

- **WHEN** beforeCommand 是 `(info) => undefined`，dispatch `['new', 'change', 'demo']`
- **THEN** 變更建立成功，結果與引入第二個參數之前相同

#### Scenario: handleRequest 進來的呼叫也有 read

- **WHEN** beforeCommand 在命令是 `archive` 時用 `read` 讀出變更沒有落地記錄並拒絕、原因「程式碼還沒落地」；用戶端經 host.handleRequest 送出封存 demo 的請求
- **THEN** 回應是拒絕，訊息是「程式碼還沒落地」；儲存的 commit 沒有被呼叫

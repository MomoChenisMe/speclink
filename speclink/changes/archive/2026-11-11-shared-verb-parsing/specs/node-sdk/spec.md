## MODIFIED Requirements

### Requirement: dispatch 的輸入輸出契約
<!-- BEFORE: dispatch 的 argv 由 SDK 自己的解析處理，不認得的旗標與多出來的位置參數被靜默忽略 -->
engine.dispatch SHALL 接受 argv 字串陣列（與 CLI 動詞詞彙一對一）與選填第二參數（stdin 內容），回傳 Promise；成功時解析為與 CLI --json 對齊的結構化物件（無 --json 形式的動詞回傳含 output 字串的物件）；失敗時以 Error 拒絕——message 為與 CLI 相同的語義化訊息並附 code 欄位。dispatch SHALL 於背景工作執行，SHALL NOT 阻塞 JS 事件迴圈。

dispatch SHALL 由與 CLI 共用的引擎命令層執行：argv 詞彙、回傳形狀與既有錯誤碼值域維持不變；對相同 workspace 狀態，dispatch 的成功結果與錯誤 SHALL 與 CLI 對應動詞語意一致，錯誤碼 SHALL 出自命令層的封閉註冊表（含 invalid_config 與 refused）。宿主 Store 提供的工作流設定文字存在但無法解析時，讀取政策的 dispatch 呼叫 SHALL 以 Error 拒絕且 code 為 invalid_config，SHALL NOT 以預設政策繼續執行。

dispatch 的 argv SHALL 由與 CLI 共用的同一份動詞解析處理：CLI 接受的寫法（含 `--name=value` 與 CLI 有的短旗標）dispatch SHALL 同樣接受；CLI 不接受的旗標與多出來的位置參數，dispatch SHALL 以 Error 拒絕、code 為 invalid_argv、message 為與 CLI 相同的解析錯誤文字（不含 ANSI 色碼），SHALL NOT 靜默忽略。argv 帶 `--json` 時，回傳的結構化物件與不帶時相同。argv 是 `--help` 時 SHALL 以 Error 拒絕、code 為 invalid_argv、message 為該動詞的說明文字。唯一的例外是 review stamp／verify stamp 的 `--stdin`：dispatch 接受它，並從 stdin 參數讀蓋章 JSON（見「dispatch 的蓋章動詞」），CLI 不接受。只在帶 `--stdin` 時才讀 stdin 的動詞（new artifact、review stamp、verify stamp）收到 stdin 參數、argv 卻沒帶 `--stdin` 時，dispatch SHALL 以 Error 拒絕、code 為 invalid_argv，SHALL NOT 靜默丟掉 stdin 的內容，也 SHALL NOT 寫入。

#### Scenario: 寫入動詞經 stdin 參數
- **WHEN** 執行 await engine.dispatch(['new', 'artifact', 'proposal', '--change', 'demo', '--stdin'], { stdin: 內容字串 })
- **THEN** 宿主 Store 收到該 artifact 的寫入呼叫，dispatch 解析為成功結果

#### Scenario: 錯誤以語義化例外傳遞
- **WHEN** Store 於認領時回報該 change 已被他人持有，執行 dispatch(['claim', 'x'])
- **THEN** Promise 以 Error 拒絕，message 為語義化訊息（含持有情境與建議動作）、code 反映衝突類別，宿主可將 message 直接回給 agent

#### Scenario: 並發 dispatch 不死結
- **WHEN** 對同一引擎並發發出多個 dispatch 呼叫（宿主 Store 方法為 async）
- **THEN** 全部呼叫在有限時間內完成（無互等死結），事件迴圈期間可持續處理其他工作

#### Scenario: 壞工作流設定經 dispatch 拒絕
- **WHEN** 宿主 Store 的工作流設定讀取方法回傳無法解析的 YAML 文字，執行 dispatch(['new', 'change', 'demo'])
- **THEN** Promise 以 Error 拒絕，code 為 invalid_config，message 指出工作流設定無法解析與原因

#### Scenario: 不認得的旗標被拒絕

- **WHEN** 執行 dispatch(['list', '--no-such-flag'])
- **THEN** Promise 以 Error 拒絕，code 為 invalid_argv，message 與執行 speclink list --no-such-flag 時 stderr 的錯誤文字相同（不含 ANSI 色碼）

#### Scenario: 多出來的位置參數被拒絕

- **WHEN** 執行 dispatch(['status', 'extra-one', 'extra-two'])，而 CLI 對同一串 argv 回報解析錯誤
- **THEN** Promise 以 Error 拒絕，code 為 invalid_argv

#### Scenario: 等號寫法與 CLI 相同

- **WHEN** 執行 dispatch(['list', '--sort=name'])
- **THEN** 結果與 dispatch(['list', '--sort', 'name']) 相同

#### Scenario: 帶 --json 時回傳形狀不變

- **WHEN** 分別執行 dispatch(['list']) 與 dispatch(['list', '--json'])
- **THEN** 兩者解析為相同的結構化物件

#### Scenario: 蓋章的 --stdin 只有 dispatch 接受

- **WHEN** 對任務全完成、末輪沒有必修 finding 的 change beta 執行 dispatch(['review', 'stamp', 'beta', '--stdin'], { stdin: 蓋章 JSON })，並在本機 repo 執行 speclink review stamp beta --stdin
- **THEN** dispatch 解析為 { change: 'beta' }；CLI 以 exit code 2 結束、沒有寫入，stderr 是解析錯誤

#### Scenario: 給了 stdin 卻沒帶 --stdin

- **WHEN** 執行 dispatch(['new', 'artifact', 'proposal', '--change', 'beta'], { stdin: 內容字串 })
- **THEN** Promise 以 Error 拒絕，code 為 invalid_argv，message 指出要帶 --stdin，宿主 Store 沒有收到寫入呼叫

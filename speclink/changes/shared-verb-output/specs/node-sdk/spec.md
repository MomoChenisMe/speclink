## MODIFIED Requirements

### Requirement: dispatch 的輸入輸出契約
<!-- BEFORE: dispatch 只接受 9 個動詞，其餘以 invalid_argv 拒絕；各動詞的回傳內容由 SDK 自己組出 -->
engine.dispatch SHALL 接受 argv 字串陣列（與 CLI 動詞詞彙一對一）與選填第二參數（stdin 內容），回傳 Promise；成功時解析為與 CLI --json 對齊的結構化物件（無 --json 形式的動詞回傳含 output 字串的物件）；失敗時以 Error 拒絕——message 為與 CLI 相同的語義化訊息並附 code 欄位。dispatch SHALL 於背景工作執行，SHALL NOT 阻塞 JS 事件迴圈。

dispatch SHALL 由與 CLI 共用的引擎命令層執行：argv 詞彙、回傳形狀與既有錯誤碼值域維持不變；對相同 workspace 狀態，dispatch 的成功結果與錯誤 SHALL 與 CLI 對應動詞語意一致，錯誤碼 SHALL 出自命令層的封閉註冊表（含 invalid_config 與 refused）。宿主 Store 提供的工作流設定文字存在但無法解析時，讀取政策的 dispatch 呼叫 SHALL 以 Error 拒絕且 code 為 invalid_config，SHALL NOT 以預設政策繼續執行。

dispatch 的 argv SHALL 由與 CLI 共用的同一份動詞解析處理：CLI 接受的寫法（含 `--name=value` 與 CLI 有的短旗標）dispatch SHALL 同樣接受；CLI 不接受的旗標與多出來的位置參數，dispatch SHALL 以 Error 拒絕、code 為 invalid_argv、message 為與 CLI 相同的解析錯誤文字（不含 ANSI 色碼），SHALL NOT 靜默忽略。argv 帶 `--json` 時，回傳的結構化物件與不帶時相同。argv 是 `--help` 時 SHALL 以 Error 拒絕、code 為 invalid_argv、message 為該動詞的說明文字。

dispatch SHALL 接受全部有引擎命令的動詞。只能在有 checkout 的一端執行的動詞（drift、change rank、review prepare、review scope、verify scope、帶 --skill 的 instructions）SHALL 以 Error 拒絕、code 為 invalid_argv、message 說明原因。dispatch 的回傳內容 SHALL 由與 CLI 共用的同一份輸出定義產生：有 --json 形式的動詞回傳與 CLI --json 內容相同的結構化物件；沒有 --json 形式的動詞回傳 { output }，output 與 CLI 在 --no-color 下的 stdout 相同。

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

#### Scenario: 原本不支援的動詞現在能用

- **WHEN** 對含變更 demo 的 workspace 執行 dispatch(['show', 'demo'])
- **THEN** Promise 解析為與 speclink show demo --json 內容相同的結構化物件

#### Scenario: 沒有 --json 形式的動詞回傳 output

- **WHEN** 執行 dispatch(['artifact', 'cat', 'proposal', '--change', 'demo'])
- **THEN** Promise 解析為 { output }，output 與 speclink artifact cat proposal --change demo --no-color 的 stdout 相同

#### Scenario: 需要 checkout 的動詞仍被拒絕

- **WHEN** 執行 dispatch(['change', 'rank', 'demo', '--before', 'other'])
- **THEN** Promise 以 Error 拒絕，code 為 invalid_argv，message 說明這個動詞只能在有 checkout 的一端執行

### Requirement: dispatch 的蓋章動詞
<!-- BEFORE: dispatch 只認得 review 與 verify 的 add-round 與 stamp 四個動詞，其他子動詞以 invalid_argv 拒絕 -->

dispatch SHALL 認得 `review` 與 `verify` 的 `add-round`、`show`、`stamp`、`discard` 八個動詞，argv 沿用 CLI 詞彙（`--accept`、`--agent <tool>`、`--stdin`）。add-round 的輪次內容 SHALL 由 dispatch 的 stdin 參數帶入；stamp 的 scope 指紋與 missing 清單 argv 承載不了，SHALL 由 stdin 參數以 JSON 帶入（`{ "scope": [{ "path", "hash" }], "missing": [] }`，兩欄缺席讀作空清單）。蓋章落下的 `reviewed_by`／`verified_by` SHALL 為該 engine 建構期綁定的 actor（未給時依儲存形式回退，與 created_by 同一條解析）。引擎既有的守門（任務未全完成、末輪未解必修 findings、scope ∪ missing 與工單聯集的分割）SHALL 原封傳遞，拒絕時以語義化例外呈現。蓋章會刪除工單文件並改寫 change 的 metadata 原文，宿主 Store 因此 SHALL 可提供三個選填的前置方法：`deleteArtifact(change, artifact)`、`readChangeMeta(name)`、`writeChangeMeta(name, content)`；缺任何一個時蓋章 SHALL 在動手前以語義化訊息拒絕（工單與 metadata 皆不動），其餘動詞不受影響。

#### Scenario: review 蓋章鏈落 actor

- **WHEN** 以 actor: 'Rev <rev@example.com>' 建構引擎，對一個任務全完成的 change 依序 dispatch(['review', 'add-round', 'beta', '--stdin'], { stdin: 只含 SUGGESTION 的輪次內容 }) 與 dispatch(['review', 'stamp', 'beta', '--stdin'], { stdin: JSON.stringify({ missing: [輪次 Scope 的檔路徑] }) })
- **THEN** add-round 解析為 { change: 'beta', round: 1 }；stamp 解析為 { change: 'beta' }，且 change 的 metadata 落下 reviewed_by: Rev <rev@example.com>

#### Scenario: verify 蓋章鏈落 actor

- **WHEN** 同引擎（同一 actor）對同一 change 走 verify add-round 與 verify stamp
- **THEN** metadata 落下 verified_by 為同一個 actor 值

#### Scenario: 蓋章守門的拒絕原封傳遞

- **WHEN** 對末輪帶 CRITICAL finding 的工單 dispatch(['review', 'stamp', ...]) 而不帶 --accept
- **THEN** dispatch 以 Error 拒絕，message 為引擎的語義化守門訊息，未落任何 reviewed_* 欄位

#### Scenario: 宿主 Store 未實作 deleteArtifact 時蓋章失敗

- **WHEN** 以未實作 deleteArtifact 的 JS Store 建構引擎，走完 add-round 後 dispatch(['review', 'stamp', ...])
- **THEN** 以 Error 拒絕，訊息指名 deleteArtifact 是蓋章所需的方法；同一個 store 的 list／status／new 動詞不受影響

#### Scenario: 未支援的子動詞明確拒絕

- **WHEN** dispatch(['review', 'prepare', 'beta'])
- **THEN** 以 code 為 invalid_argv 的 Error 拒絕，訊息說明 review prepare 只能在有 checkout 的一端執行

#### Scenario: 讀取與丟棄工單

- **WHEN** 對一個已有一輪審查記錄的 change 依序 dispatch(['review', 'show', 'beta']) 與 dispatch(['review', 'discard', 'beta'])
- **THEN** show 解析為與 speclink review show beta --json 內容相同的結構化物件；discard 成功後再 show，結果是沒有工單

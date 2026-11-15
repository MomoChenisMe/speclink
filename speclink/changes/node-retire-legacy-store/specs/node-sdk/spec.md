## REMOVED Requirements

### Requirement: createEngine 的雙形式儲存建構

**Reason**: 宿主實作的 Store 物件（31 個必要方法）這一種形式移除。它沒有交易邊界與版本比對，已經由 createHost 與三個儲存函式取代；需求名稱裡的「雙形式」不再成立

**Migration**: 由「createEngine 只接受本機資料夾」承接本機資料夾形式與它的情境「fs 形式與 CLI 行為對等」。用 Store 物件的宿主把儲存改寫成 list、read、commit 三個函式並改用 createHost（node-sdk「createHost 與 host.dispatch」「JS 版的儲存接口」），用套件裡的 runStoreConformance 檢查實作

## ADDED Requirements

### Requirement: createEngine 只接受本機資料夾

Node SDK 的 createEngine SHALL 只接受內建的本機資料夾形式：`store` 是 `{ type: 'fs', root, specDir? }`，root 是專案根目錄，specDir 是選填的規格資料夾名稱。`store` 是其他形狀（包含帶方法的物件）時，createEngine SHALL 在建構當下同步拋出錯誤：訊息 SHALL 說明只接受本機資料夾形式並含 `createHost` 字樣，SHALL NOT 產生引擎實例，SHALL NOT 呼叫傳入物件的任何方法。自訂的儲存 SHALL 只能經 createHost 與三個儲存函式提供。

#### Scenario: fs 形式與 CLI 行為對等

- **WHEN** 以 createEngine 的 fs 形式指向某 fixture 專案，執行 await engine.dispatch(['list', '--json'])
- **THEN** 回傳物件與在同一專案執行 speclink list --json 的輸出 JSON 完全一致

#### Scenario: 傳入帶方法的物件時建構失敗

- **WHEN** 以一個帶 listChanges、readArtifact 等方法的 JS 物件當 store 呼叫 createEngine
- **THEN** 同步拋出錯誤，訊息含 `createHost`；不產生引擎實例；該物件的任何方法都沒有被呼叫

#### Scenario: fs 形式缺 root 時建構失敗

- **WHEN** 執行 createEngine({ store: { type: 'fs' } })
- **THEN** 同步拋出錯誤，訊息指出 root 必須是字串；不產生引擎實例

## MODIFIED Requirements

### Requirement: dispatch 的輸入輸出契約
<!-- BEFORE: 內文與五個情境以宿主 Store 物件為對象；claim 由宿主 Store 的 claim 方法裁決、成功時回傳該方法的回傳值；壞的流程設定來自宿主 Store 的讀取方法 -->

engine.dispatch SHALL 接受 argv 字串陣列（與 CLI 動詞詞彙一對一）與選填第二參數（stdin 內容），回傳 Promise；成功時解析為與 CLI --json 對齊的結構化物件（無 --json 形式的動詞回傳含 output 字串的物件）；失敗時以 Error 拒絕——message 為與 CLI 相同的語義化訊息並附 code 欄位。dispatch SHALL 於背景工作執行，SHALL NOT 阻塞 JS 事件迴圈。

dispatch SHALL 由與 CLI 共用的引擎命令層執行：argv 詞彙、回傳形狀與既有錯誤碼值域維持不變；對相同 workspace 狀態，dispatch 的成功結果與錯誤 SHALL 與 CLI 對應動詞語意一致，錯誤碼 SHALL 出自命令層的封閉註冊表（含 invalid_config 與 refused）。規格資料夾的流程設定檔存在但無法解析時，讀取政策的 dispatch 呼叫 SHALL 以 Error 拒絕且 code 為 invalid_config，SHALL NOT 以預設政策繼續執行。

dispatch 的 argv SHALL 由與 CLI 共用的同一份動詞解析處理：CLI 接受的寫法（含 `--name=value` 與 CLI 有的短旗標）dispatch SHALL 同樣接受；CLI 不接受的旗標與多出來的位置參數，dispatch SHALL 以 Error 拒絕、code 為 invalid_argv、message 為與 CLI 相同的解析錯誤文字（不含 ANSI 色碼），SHALL NOT 靜默忽略。argv 帶 `--json` 時，回傳的結構化物件與不帶時相同。argv 是 `--help` 時 SHALL 以 Error 拒絕、code 為 invalid_argv、message 為該動詞的說明文字。

dispatch SHALL 接受全部有引擎命令的動詞。只能在有 checkout 的一端執行的動詞（drift、review prepare、review scope、verify scope、帶 --skill 的 instructions）SHALL 以 Error 拒絕、code 為 invalid_argv、message 說明原因。dispatch 的回傳內容 SHALL 由與 CLI 共用的同一份輸出定義產生：有 --json 形式的動詞回傳與 CLI --json 內容相同的結構化物件；沒有 --json 形式的動詞回傳 { output }，output 與 CLI 在 --no-color 下的 stdout 相同。有一種情況不經引擎命令：status 沒有給變更名稱、而且專案沒有任何變更時，與 CLI 一樣只有一行提示，回傳 { output }，output 與 CLI 的 stdout 相同。claim SHALL 與 CLI 在本機專案的行為相同：以 Error 拒絕，message 含 CLI 對本機專案的拒絕文字 `claim requires a remote store`。

沒有 --json 形式的動詞，CLI 另有提示文字印到 stderr 時（例如 archive 在變更沒有任務證據時的提示），dispatch 回傳的物件 SHALL 另帶 stderr 欄位，內容與 CLI 為該動詞印到 stderr 的提示文字相同；沒有這種文字時 SHALL NOT 帶 stderr 欄位。discuss conclude 的結論已寫入、而收尾的封存步驟失敗時，dispatch SHALL 解析為 conclude 的結構化物件並另帶 closingError 欄位（失敗原因的文字），SHALL NOT 以 Error 拒絕：結論已經寫入，拒絕會讓呼叫端以為沒有寫入而重試。

#### Scenario: 寫入動詞經 stdin 參數
- **WHEN** 對含變更 demo 的專案執行 await engine.dispatch(['new', 'artifact', 'proposal', '--change', 'demo', '--stdin'], { stdin: 內容字串 })
- **THEN** 專案的 speclink/changes/demo/proposal.md 的內容是傳入的字串，dispatch 解析為成功結果

#### Scenario: 錯誤以語義化例外傳遞
- **WHEN** 對一個本機專案執行 dispatch(['claim', 'x'])
- **THEN** Promise 以 Error 拒絕，message 為語義化訊息（含 `claim requires a remote store`）並附 code 欄位，宿主可將 message 直接回給 agent；專案裡沒有任何檔案被寫入

#### Scenario: 並發 dispatch 不死結
- **WHEN** 對同一引擎並發發出多個 dispatch 呼叫（讀取與寫入的動詞都有）
- **THEN** 全部呼叫在有限時間內完成（無互等死結），事件迴圈期間可持續處理其他工作

#### Scenario: 壞工作流設定經 dispatch 拒絕
- **WHEN** 專案的 speclink/config.yaml 是無法解析的 YAML 文字，執行 dispatch(['new', 'change', 'demo'])
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
- **THEN** Promise 以 Error 拒絕，code 為 invalid_argv，message 指出要帶 --stdin，專案裡沒有任何檔案被寫入

#### Scenario: 原本不支援的動詞現在能用

- **WHEN** 對含變更 demo 的 workspace 執行 dispatch(['show', 'demo'])
- **THEN** Promise 解析為與 speclink show demo --json 內容相同的結構化物件

#### Scenario: 沒有 --json 形式的動詞回傳 output

- **WHEN** 執行 dispatch(['artifact', 'cat', 'proposal', '--change', 'demo'])
- **THEN** Promise 解析為 { output }，output 與 speclink artifact cat proposal --change demo --no-color 的 stdout 相同

#### Scenario: 需要 checkout 的動詞仍被拒絕

- **WHEN** 執行 dispatch(['drift', 'demo'])
- **THEN** Promise 以 Error 拒絕，code 為 invalid_argv，message 說明這個動詞只能在有 checkout 的一端執行

#### Scenario: stderr 的提示經 stderr 欄位回傳

- **WHEN** 對任務全完成、沒有任務證據的變更 beta 執行 dispatch(['archive', 'beta', '-y'])
- **THEN** Promise 解析為 { output, stderr }，output 與 speclink archive beta -y --no-color 的 stdout 相同，stderr 與它的 stderr 相同（一行零證據提示）

#### Scenario: 沒有 stderr 文字時不帶 stderr 欄位

- **WHEN** 執行 dispatch(['new', 'change', 'demo'])
- **THEN** Promise 解析的物件只有 output 一個欄位

#### Scenario: 結論已寫入但收尾封存失敗

- **WHEN** 一份討論記錄已轉出的變更都已不在進行中、而收尾的封存步驟會失敗，執行 dispatch(['discuss', 'conclude', 該記錄的 slug, '--stdin'], { stdin: 結論內容 })
- **THEN** Promise 解析為含 slug、status 為 concluded 與 closingError 的物件，closingError 是失敗原因的文字；記錄的結論已寫入；Promise 不以 Error 拒絕

#### Scenario: dispatch 執行 change rank

- **WHEN** 以 fs 形式的引擎對含 a（有順序鍵）與 c（無順序鍵）兩個提案中變更的專案執行 dispatch(['change', 'rank', 'c', '--before', 'a', '--json'])
- **THEN** 回傳 `{ change: 'c', anchor: 'a', position: 'before' }`；隨後 dispatch(['plan', '--json']) 的 changes 順序 c 在 a 前

### Requirement: createEngine 的建構期 actor 注入
<!-- BEFORE: actor 在兩種儲存形式都收下；宿主 Store 形式未給 actor 時不蓋章；多身分的宿主以多個 engine 實例表達 -->
<!-- REMOVED-SCENARIO: 宿主 Store 形式帶 actor 落章 -->
<!-- REMOVED-SCENARIO: 宿主 Store 形式未給 actor 維持無章 -->

createEngine SHALL 接受選填 actor 欄位（"Name <email>" 格式字串），於建構期收下並綁定於該 engine 實例；dispatch 的操作者身分解析 SHALL 為：建構期 actor 有給值（trim 後非空）一律優先，未給時回退 workspace 的 git identity。trim 後為空字串的 actor SHALL 視同未給。actor SHALL NOT 能經 dispatch 的 argv 或其他呼叫期參數傳入或覆寫——一個實例一個身分。每次呼叫帶不同身分的需求由 createHost 承擔：身分由每次呼叫給。

#### Scenario: fs 形式明給 actor 優先於 git identity

- **WHEN** 在設有 git user.name 與 user.email 的 fixture 專案，以 createEngine({ store: { type: 'fs', root }, actor: 'Alice <alice@example.com>' }) 建構並 dispatch(['new', 'change', 'demo'])
- **THEN** demo 的 metadata created_by 為 Alice <alice@example.com>，而非 git identity

#### Scenario: fs 形式未給 actor 維持現行回退

- **WHEN** 同一 fixture 專案以不帶 actor 的 createEngine fs 形式建構並 dispatch(['new', 'change', 'demo2'])
- **THEN** demo2 的 created_by 與在該專案執行 CLI speclink new change 的蓋章逐位元一致（git identity 回退不變）

#### Scenario: 呼叫期無從覆寫身分

- **WHEN** 檢視 dispatch 的輸入契約並嘗試以任意 argv 影響蓋章身分
- **THEN** dispatch 不存在 actor 參數，蓋章內容只隨建構期 actor（或其回退）改變

### Requirement: dispatch 的蓋章動詞
<!-- BEFORE: 蓋章的身分未給時依儲存形式回退；宿主 Store 可提供 deleteArtifact、readChangeMeta、writeChangeMeta 三個選填方法，缺任何一個時蓋章在動手前拒絕 -->
<!-- REMOVED-SCENARIO: 宿主 Store 未實作 deleteArtifact 時蓋章失敗 -->

dispatch SHALL 認得 `review` 與 `verify` 的 `add-round`、`show`、`stamp`、`discard` 八個動詞，argv 沿用 CLI 詞彙（`--accept`、`--agent <tool>`、`--stdin`）。add-round 的輪次內容 SHALL 由 dispatch 的 stdin 參數帶入；stamp 的 scope 指紋與 missing 清單 argv 承載不了，SHALL 由 stdin 參數以 JSON 帶入（`{ "scope": [{ "path", "hash" }], "missing": [] }`，兩欄缺席讀作空清單）。蓋章落下的 `reviewed_by`／`verified_by` SHALL 為該 engine 建構期綁定的 actor（未給時回退 git identity，與 created_by 同一條解析）。引擎既有的守門（任務未全完成、末輪未解必修 findings、scope ∪ missing 與工單聯集的分割）SHALL 原封傳遞，拒絕時以語義化例外呈現。

#### Scenario: review 蓋章鏈落 actor

- **WHEN** 以 actor: 'Rev <rev@example.com>' 建構引擎，對一個任務全完成的 change 依序 dispatch(['review', 'add-round', 'beta', '--stdin'], { stdin: 只含 SUGGESTION 的輪次內容 }) 與 dispatch(['review', 'stamp', 'beta', '--stdin'], { stdin: JSON.stringify({ missing: [輪次 Scope 的檔路徑] }) })
- **THEN** add-round 解析為 { change: 'beta', round: 1 }；stamp 解析為 { change: 'beta' }，且 change 的 metadata 落下 reviewed_by: Rev <rev@example.com>

#### Scenario: verify 蓋章鏈落 actor

- **WHEN** 同引擎（同一 actor）對同一 change 走 verify add-round 與 verify stamp
- **THEN** metadata 落下 verified_by 為同一個 actor 值

#### Scenario: 蓋章守門的拒絕原封傳遞

- **WHEN** 對末輪帶 CRITICAL finding 的工單 dispatch(['review', 'stamp', ...]) 而不帶 --accept
- **THEN** dispatch 以 Error 拒絕，message 為引擎的語義化守門訊息，未落任何 reviewed_* 欄位

#### Scenario: 未支援的子動詞明確拒絕

- **WHEN** dispatch(['review', 'prepare', 'beta'])
- **THEN** 以 code 為 invalid_argv 的 Error 拒絕，訊息說明 review prepare 只能在有 checkout 的一端執行

#### Scenario: 讀取與丟棄工單

- **WHEN** 對一個已有一輪審查記錄的 change 依序 dispatch(['review', 'show', 'beta']) 與 dispatch(['review', 'discard', 'beta'])
- **THEN** show 解析為與 speclink review show beta --json 內容相同的結構化物件；discard 成功後再 show，結果是沒有工單

### Requirement: dispatch 的流程設定動詞
<!-- BEFORE: 宿主 Store 物件可提供選填方法 writeWorkflowConfig(content)；沒有這個方法時會寫入的動詞以 refused 拒絕；五個情境都以宿主 Store 為對象 -->
<!-- REMOVED-SCENARIO: 沒有寫入方法時拒絕 -->
<!-- REMOVED-SCENARIO: 讀取與 dry-run 不需要寫入方法 -->

dispatch SHALL 認得 workflow-config 的全部動詞（show、set、context、rules、review 的四個、subagent-model 的三個），argv 與 stdin 沿用 CLI 的詞彙，回傳內容與 CLI 同一動詞的輸出同源。會寫入的動詞 SHALL 直接寫入規格資料夾的 config.yaml，寫入後的檔案內容 SHALL 與 CLI 在同一個專案執行同一個動詞的結果相同；帶 --dry-run 的動詞 SHALL NOT 寫入。argv 帶 --stdin 而 dispatch 沒有收到 stdin 內容時，workflow-config 的動詞 SHALL 以 code invalid_argv 拒絕，config.yaml SHALL NOT 被改動；空字串是有效的內容（清掉該值）。

#### Scenario: 經 dispatch 設定政策欄位

- **WHEN** 專案的 speclink/config.yaml 含 tdd: false，執行 dispatch(['workflow-config', 'set', 'tdd', 'true'])
- **THEN** config.yaml 裡 tdd 是 true；Promise 解析為成功結果

#### Scenario: dry-run 不寫入

- **WHEN** 專案的 speclink/config.yaml 含 tdd: false，執行 dispatch(['workflow-config', 'set', 'tdd', 'true', '--dry-run'])
- **THEN** Promise 解析為成功結果；config.yaml 逐位元不變

#### Scenario: 帶 --stdin 卻沒有 stdin 內容時拒絕

- **WHEN** 專案的流程設定含 rules.tasks 一條規則，執行 dispatch(['workflow-config', 'rules', 'tasks', '--stdin']) 而沒有傳 stdin
- **THEN** Promise 以 Error 拒絕，code 為 invalid_argv；流程設定沒有被改動。改傳 stdin 為空字串時成功，rules.tasks 被移除

#### Scenario: 壞的流程設定經 dispatch 拒絕

- **WHEN** 專案的 speclink/config.yaml 是無法解析的 YAML，執行 dispatch(['workflow-config', 'show'])
- **THEN** Promise 以 Error 拒絕，code 為 invalid_config

### Requirement: dispatch 與 handleRequest 的 fields
<!-- BEFORE: engine.dispatch 只有 fs 形式接受自由欄位的動詞；以 JS 儲存形式建立的引擎以 refused 拒絕並指向 createHost -->
<!-- REMOVED-SCENARIO: 舊的 JS 儲存形式拒絕 -->

host.dispatch 的呼叫物件 SHALL 接受選填的 `fields`（物件，字串對字串）；host.handleRequest SHALL 接受選填的第三個參數，其 `fields` 欄位同型。兩者的行為 SHALL 等同宿主套件「呼叫入口時附上自由欄位」：只在建立變更或討論時生效、蓋過同名的值、其他命令忽略。`fields` 的任一值不是字串時，呼叫 SHALL 以 code 為 invalid_argv 的 Error（dispatch）或狀態碼 400 的回應（handleRequest）結束，儲存 SHALL 沒有變動。host.dispatch SHALL 接受自由欄位的四個動詞與兩個建立動詞的 `--field` 旗標，回傳值與 CLI 的 `--json` 相同。engine.dispatch SHALL 同樣接受這些動詞。

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

# node-sdk Specification

## Purpose

Node 綁定對外的 SDK 契約：createEngine 的本機資料夾形式與 dispatch 的輸入輸出契約、createHost 與三個儲存函式、合格測試、Agent 工具定義，以及把引擎人眼輸出取回 JS 端的渲染 API（含 worktree 軸）。本 capability 保證 JS 端不必自行拼裝引擎內部結構，且渲染結果與 CLI 同源、不另立一套。

## Requirements

### Requirement: dispatch 的輸入輸出契約

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


<!-- @trace
source: node-retire-legacy-store
updated: 2026-11-15T22:20:32+08:00
-->

---
### Requirement: 渲染 API

SDK SHALL 提供 skills.list()（回傳技能名與描述清單）與 skills.render(name, options)——options 涵蓋渲染矩陣：target（claude｜codex｜copilot｜pi｜cursor｜neutral）、invocation（cli｜tool-call，只對 neutral 有作用）、specInCheckout（布林，預設 true：規格資料夾是否位於使用這份技能的 checkout；不是布林值時——包含 null——SHALL 拋出錯誤，SHALL NOT 靜默當成預設值）、specDir 與 toolName；內嵌引擎而沒有本機規格檔的宿主 SHALL 以 specInCheckout 為 false 渲染給自己 Agent 的技能；回傳字串內容 SHALL 與 CLI 以對等參數生成的內容一致。target 不在值域內時 SHALL 拋出錯誤，訊息 SHALL 列出 claude、codex、copilot、pi、cursor、neutral 六個合法值。

#### Scenario: 中性 tool-call 渲染
- **WHEN** 執行 skills.render('propose', { target: 'neutral', invocation: 'tool-call', specInCheckout: false })
- **THEN** 回傳字串以「呼叫 speclink 工具」措辭表述動詞、不含 /speclink- 前綴與本地規格路徑句

#### Scenario: 與 CLI 生成一致
- **WHEN** 以 target claude（不傳 specInCheckout）呼叫 skills.render('apply', …)，並與 speclink init 於本機專案生成的 .claude/skills/speclink-apply/SKILL.md 比對
- **THEN** 兩者內容一致

#### Scenario: copilot target 與 CLI 生成一致
- **WHEN** 以 target copilot（不傳 specInCheckout）呼叫 skills.render('apply', …)，並與 speclink init --tools copilot 於本機專案生成的 .github/skills/speclink-apply/SKILL.md 比對
- **THEN** 兩者內容一致，且回傳字串的技能引用為 `/speclink-` 形式、不含 `$speclink-`

#### Scenario: pi 與 cursor target 與 CLI 生成一致
- **WHEN** 以 target pi 與 target cursor（不傳 specInCheckout）呼叫 skills.render('apply', …)，並分別與 speclink init --tools pi,cursor 於本機專案生成的 .pi/skills/speclink-apply/SKILL.md、.cursor/skills/speclink-apply/SKILL.md 比對
- **THEN** 兩組內容各自一致；pi 的技能引用為 `/skill:speclink-` 形式，cursor 的技能引用為 `/speclink-` 形式，兩者都不含 `$speclink-`

#### Scenario: 未知 target 被拒
- **WHEN** 執行 skills.render('apply', { target: 'github-copilot' })
- **THEN** 呼叫拋出錯誤，訊息含 `github-copilot` 與 claude、codex、copilot、pi、cursor、neutral 六個合法值

#### Scenario: specInCheckout 為 false 的渲染

- **WHEN** 以 skills.render('archive', { target: 'neutral', invocation: 'tool-call', specInCheckout: false, specDir: 'openspec' }) 渲染
- **THEN** 回傳的字串不含 `openspec/`，交棒以 `speclink-<名稱>` 表述

#### Scenario: 省略 specInCheckout 等於本機專案

- **WHEN** 分別以省略 specInCheckout 與 specInCheckout 為 true 渲染同一支技能
- **THEN** 兩個回傳字串相同，且與 CLI 在本機專案產生的該技能內容一致

#### Scenario: 非布林的 specInCheckout 被拒

- **WHEN** 以 specInCheckout 為字串 'false'、數字 0、物件或 null 呼叫 skills.render('commit', { target: 'claude', … })
- **THEN** 每一種都拋出錯誤，沒有任何一種被當成 true 渲染


<!-- @trace
source: skills-verbs-second-wave
updated: 2026-11-12T16:54:51+08:00
-->

---
### Requirement: 渲染 API 的 worktree 軸

instructions.render(options) 的 options SHALL 涵蓋 worktree 布林軸（未給定時視為 false），用以選擇 marker 區塊是否含兩行 worktree 技能指引；回傳內容 SHALL 與 CLI 於對等 worktree 政策下生成的 marker 一致。skills.render 不受此軸影響——技能檔內容與政策無關，政策只決定該技能是否被生成。

#### Scenario: worktree 軸切換 marker 內容

- **WHEN** 以同組 target 與 store 分別呼叫 instructions.render({ worktree: true }) 與 instructions.render({ worktree: false })
- **THEN** 前者的 marker 含 apply-with-worktree 與 worktree-merge 兩行，後者不含，其餘內容逐字相同

#### Scenario: 未給定 worktree 時取預設

- **WHEN** 呼叫 instructions.render 而未給定 worktree 選項
- **THEN** 回傳內容與 worktree: false 的結果相同

<!-- @trace
source: worktree-toggle-and-guards
updated: 2026-08-05
-->

---
### Requirement: createEngine 的建構期 actor 注入

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


<!-- @trace
source: node-retire-legacy-store
updated: 2026-11-15T22:20:32+08:00
-->

---
### Requirement: dispatch 的蓋章動詞

dispatch SHALL 認得 `review` 與 `verify` 的 `add-round`、`show`、`stamp`、`discard` 八個動詞，argv 沿用 CLI 詞彙（`--accept`、`--agent <tool>`、`--stdin`）。add-round 的輪次內容 SHALL 由 dispatch 的 stdin 參數帶入；stamp 的 scope 指紋與 missing 清單 argv 承載不了，SHALL 由 stdin 參數以 JSON 帶入（`{ "scope": [{ "path", "hash" }], "missing": [] }`，兩欄缺席讀作空清單）。蓋章落下的 `reviewed_by`／`verified_by` SHALL 為該 engine 建構期綁定的 actor（未給時回退 git identity，與 created_by 同一條解析）。引擎既有的守門（任務未全完成、末輪未解必修 findings、scope ∪ missing 與工單聯集的分割）SHALL 原封傳遞，拒絕時以語義化例外呈現。

#### Scenario: review 蓋章鏈落 actor

- **WHEN** 以 actor: 'Rev <rev@example.com>' 建構引擎，對一個任務全完成的 change 依序 dispatch(['review', 'add-round', 'beta', '--stdin'], { stdin: 只含 SUGGESTION 的輪次內容 }) 與 dispatch(['review', 'stamp', 'beta', '--stdin'], { stdin: JSON.stringify({ scope: [{ path: 輪次 Scope 的檔路徑, hash: 該檔的指紋 }] }) })
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


<!-- @trace
source: node-retire-legacy-store
updated: 2026-11-15T22:20:32+08:00
-->

---
### Requirement: dispatch 的流程設定動詞

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


<!-- @trace
source: node-retire-legacy-store
updated: 2026-11-15T22:20:32+08:00
-->

---
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

<!-- @trace
source: host-kit-agent-door
updated: 2026-11-12T09:48:03+08:00
-->

---
### Requirement: JS 版的儲存接口

宿主的 JS 儲存 SHALL 是三個函式，回傳值或 Promise 皆可，欄位一律 camelCase：

- `list(scope, prefix)` 回傳 `{ version, entries }`，entries 的每一筆是 `{ key, version, modifiedAt }`。
- `read(scope, keys)` 回傳和 keys 等長、順序相同的陣列，每一格是 `{ content, version, modifiedAt }` 或 null（文件不存在）。
- `commit(scope, commit)` 回傳提交後的範圍版本號（數字）。commit 物件含 `actor`（`{ id, displayName }`）、`command`（字串）、`at`（數字，毫秒）、`ops` 與 `events`（每筆 `{ name, payload }`）。ops 的每一筆是 `{ op: 'put', key, content, expected }` 或 `{ op: 'delete', key, expected }`；put 的 expected 是數字（預期的文件版本）或 null（必須不存在），delete 的 expected 是數字。

版本與時間 SHALL 是 0 到 9007199254740991 的整數。這三個函式的語意 SHALL 等同 store-port-contract 規定的列出、讀取、提交；領域事件 SHALL 只經 commit 的 events 交給宿主。

宿主函式拋出的錯誤 SHALL 依它的 `code` 屬性分類：commit 拋出 `code` 為 `conflict` 且 `key` 是這次提交其中一個鍵時是版本衝突，實際版本取自錯誤的 `actual`（數字或 null）；`code` 為 `unavailable` 時是暫時無法使用；其餘一律是其他失敗。SDK SHALL 匯出 StoreConflictError 與 StoreUnavailableError 兩個設好 `code` 的 Error 類別，分類 SHALL NOT 依賴 instanceof。宿主函式的回傳值不合格時——read 的陣列長度不等於鍵的數量、版本或時間不是範圍內的整數、list 回傳的鍵不合鍵的格式——SHALL 視為其他失敗，訊息 SHALL 指出是哪個函式與哪個欄位，這次命令 SHALL NOT 提交。

#### Scenario: 寫入的動詞只呼叫一次 commit 並帶事件

- **WHEN** 經 host.dispatch 執行 `['task', 'done', '1', '--change', 'demo']` 成功
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

<!-- @trace
source: host-kit-agent-door
updated: 2026-11-12T09:48:03+08:00
-->

---
### Requirement: 合格測試隨套件提供

Node SDK SHALL 匯出 runStoreConformance，讓宿主對自己的 JS 儲存執行儲存契約的合格測試。它接受一個物件：受測的 store、每次回傳一個沒用過的範圍的 newScope、選填的 recordedEvents（回傳某個範圍已經收到的事件，每筆含 name、payload 與所屬提交的 version）；回傳 Promise，解析為 `{ passed, checks }`，checks 的每一筆是 `{ name, outcome, detail }`，outcome 是 `passed`、`failed` 或 `skipped`。檢查的項目與名稱 SHALL 和 store-port-contract 規定的合格測試相同，同一份檢查 SHALL 同時用於 Rust 實作與 JS 實作。沒有提供 recordedEvents 時，只能靠讀回事件判定的檢查（`events-with-commit`）SHALL 是 `skipped` 並在 detail 說明原因；其餘檢查 SHALL 照常執行，和 store-port-contract 對不保存事件的實作的規定相同。passed SHALL 在沒有任何一項是 `failed` 時為 true。runStoreConformance SHALL NOT 阻塞 JS 事件迴圈。

#### Scenario: 正確的 JS 儲存通過全部檢查

- **WHEN** 對一個正確實作三個函式並提供 recordedEvents 的 JS 記憶體儲存執行 runStoreConformance
- **THEN** passed 是 true；checks 有 16 筆，名稱與 Rust 的合格測試相同，全部是 `passed`

#### Scenario: 不比對版本的儲存被判定失敗

- **WHEN** 對一個 commit 忽略 expected、一律寫入的 JS 儲存執行 runStoreConformance
- **THEN** passed 是 false；`conflict-stale-version` 這一項是 `failed`，detail 說明預期被拒絕卻寫入成功

#### Scenario: 沒有 recordedEvents 時事件的檢查略過

- **WHEN** 對一個正確的 JS 儲存執行 runStoreConformance，不提供 recordedEvents
- **THEN** `events-with-commit` 是 `skipped` 且 detail 說明沒有提供 recordedEvents；其餘是 `passed`（`events-only-commit` 照樣檢查只有事件的提交會讓範圍版本前進）；passed 是 true

#### Scenario: 非同步的儲存

- **WHEN** 受測儲存的三個函式都回傳稍後才解析的 Promise
- **THEN** runStoreConformance 照常完成，並行的兩項檢查也有結果

<!-- @trace
source: host-kit-agent-door
updated: 2026-11-12T09:48:03+08:00
-->

---
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

<!-- @trace
source: host-kit-agent-door
updated: 2026-11-12T09:48:03+08:00
-->

---
### Requirement: host.handleRequest 與 createHost 的入口設定

createHost SHALL 另外接受三個選填的欄位：`scopes`（函式，收身分並回傳專案清單，回傳值或 Promise 皆可）、`authMethods`（登入方式的宣告）、`pushTransports`（推送事件的宣告）。`scopes` 回傳的每個專案是 `{ key, name, repos }`，每個 repo 是 `{ key, name, scope, canWrite, canWritePolicy }`，欄位一律 camelCase。Node SDK SHALL 提供 host.handleRequest：第一個參數是 `{ method, path, query, headers, body }`（headers 是字串對字串的物件，body 是位元組陣列，query 與 body 可省略），第二個參數是身分或 null；回傳 Promise，解析為 `{ status, headers, body }`，或在路徑不屬於入口時解析為 null。它的行為 SHALL 等同宿主套件的 HTTP 入口，並與 host.dispatch 共用同一個儲存與同一組掛鉤。handleRequest SHALL 於背景工作執行，SHALL NOT 阻塞 JS 事件迴圈。`scopes` 拋出例外或 Promise 被拒絕時回應 SHALL 是 503。方法不是 GET、HEAD、POST、PUT、DELETE 時 SHALL 解析為狀態碼 405 的回應。handleRequest SHALL NOT 讀取 Authorization 標頭來決定身分。

#### Scenario: 用 JS 儲存服務握手與寫入

- **WHEN** 以 JS 的記憶體儲存與回傳單一專案單一 repo 的 scopes 建立 host，依序以同一個身分對 handleRequest 送出握手、建立變更 demo、列出變更
- **THEN** 三個回應的 status 都是 200，body 解析後是 Client Protocol 的對應 payload；儲存的 commit 被呼叫正好一次

#### Scenario: 身分為 null

- **WHEN** 以 null 當身分送出變更清單的請求
- **THEN** 回應的 status 是 401

#### Scenario: 不屬於入口的路徑

- **WHEN** 送出路徑為 `/healthz` 的請求
- **THEN** Promise 解析為 null

#### Scenario: 兩個門共用掛鉤

- **WHEN** beforeCommand 對 archive 回傳拒絕，分別經 host.dispatch 與 host.handleRequest 封存同一個變更
- **THEN** 前者以 code 為 refused 的 Error 拒絕，後者的回應 status 是 409 且 body 的 reason 是 refused；兩次的訊息相同

#### Scenario: scopes 是非同步函式

- **WHEN** scopes 是稍後才解析的 async 函式
- **THEN** handleRequest 等它解析後照常回應

#### Scenario: scopes 拋出例外

- **WHEN** scopes 拋出例外，送出變更清單的請求
- **THEN** 回應的 status 是 503

#### Scenario: 登入方式的宣告

- **WHEN** createHost 帶 `authMethods: { methods: ['browser'], browser: { issuer, clientId } }`，以 null 當身分請求 `/api/speclink/v1/auth-methods`
- **THEN** 回應的 status 是 200，body 含 methods 與 browser 的 issuer、clientId

#### Scenario: 不支援的方法

- **WHEN** 以 method 為 PATCH 送出請求
- **THEN** 回應的 status 是 405；儲存沒有被呼叫

<!-- @trace
source: host-kit-client-door
updated: 2026-11-12T13:00:47+08:00
-->

---
### Requirement: createHost 的技能清單

createHost SHALL 接受選填的 `skills` 欄位（字串陣列：這個宿主提供給連進來的用戶端的技能名稱）。省略時 SHALL 提供引擎的全部技能。陣列含不認得的技能名稱時 createHost SHALL 同步拋出錯誤並列出這些名稱，不產生宿主套件；空陣列時 createHost SHALL 同樣同步拋出錯誤（要提供全部技能就省略這個欄位）。host.handleRequest SHALL 依這個清單回答技能清單的請求，並在握手宣告技能版號。這個欄位 SHALL NOT 提供改寫技能文字的途徑。它 SHALL NOT 影響 skills.render 與 host.dispatch。

#### Scenario: 只提供兩支技能

- **WHEN** 以 `skills: ['discuss', 'propose']` 建立 host，經 handleRequest 送出 claude 的技能清單請求
- **THEN** 回應的 skills 只有 discuss 與 propose

#### Scenario: 省略時提供全部

- **WHEN** 不帶 skills 建立 host，送出 claude 的技能清單請求
- **THEN** 回應含引擎給 Claude 的每一支技能（worktree 的兩支依範圍的流程設定）

#### Scenario: 不認得的名稱

- **WHEN** 以 `skills: ['discuss', 'no-such-skill']` 呼叫 createHost
- **THEN** 同步拋出錯誤，訊息列出 no-such-skill

#### Scenario: 空陣列

- **WHEN** 以 `skills: []` 呼叫 createHost
- **THEN** 同步拋出錯誤，訊息說明清單是空的

#### Scenario: skills.render 不受影響

- **WHEN** 以 `skills: ['discuss']` 建立 host 之後呼叫 skills.render('archive', …)
- **THEN** 照常回傳 archive 的渲染結果

<!-- @trace
source: remote-skill-delivery
updated: 2026-11-13T14:35:20+08:00
-->

---
### Requirement: dispatch 與 handleRequest 的 fields

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


<!-- @trace
source: node-retire-legacy-store
updated: 2026-11-15T22:20:32+08:00
-->

---
### Requirement: 宿主的匯出與匯入方法

Node SDK 的宿主套件 SHALL 提供 `host.exportScope({ identity, scope })`，回傳 Promise，解析為第二版的包（`formatVersion`、`scopeVersion`、`documents`，每項 `key`、`content`、`digest`）；以及 `host.importScope({ identity, scope, bundle })`，回傳 Promise，解析為 `{ scopeVersion, documents }`（documents 為寫入的份數）。`bundle` SHALL 可以是第一版或第二版。兩者 SHALL 經過與 host.dispatch 相同的掛鉤，並 SHALL 於背景工作執行，SHALL NOT 阻塞 JS 事件迴圈。失敗時 SHALL 以帶 code 的 Error 拒絕：包不合格為 `invalid_argv`；目標範圍不是空的、掛鉤拒絕為 `refused`；儲存層的失敗沿用既有的三個 code。createHost SHALL 接受選填的 `limits.importBytes`（正整數）；不是正整數時 SHALL 同步拋出錯誤。host.handleRequest SHALL 回答匯入與匯出的路由，並套用同一個上限。

#### Scenario: 匯出再匯入

- **WHEN** 以 JS 的記憶體儲存建立 host，在範圍 s1 建立一個變更後 exportScope，再把結果 importScope 到範圍 s2
- **THEN** 對 s2 執行 `['list', '--json']` 得到同一個變更；s2 的 commit 被呼叫正好一次

#### Scenario: 匯入非空的範圍

- **WHEN** 對已有變更的範圍呼叫 importScope
- **THEN** Promise 以 code 為 refused 的 Error 拒絕；commit 沒有被呼叫

#### Scenario: 包不合格

- **WHEN** importScope 的 bundle 有一份文件的 digest 與內容不符
- **THEN** Promise 以 code 為 invalid_argv 的 Error 拒絕，message 含該文件的鍵

#### Scenario: 掛鉤拒絕

- **WHEN** beforeCommand 對 scope-import 回傳拒絕，呼叫 importScope
- **THEN** Promise 以 code 為 refused 的 Error 拒絕，message 是宿主的原因

#### Scenario: 上限經 handleRequest 生效

- **WHEN** 以 `limits: { importBytes: 1024 }` 建立 host，經 handleRequest 送出本文為 4096 位元組的匯入請求
- **THEN** 回應的 status 是 413

#### Scenario: 上限不是正整數

- **WHEN** 以 `limits: { importBytes: 0 }` 呼叫 createHost
- **THEN** 同步拋出錯誤

<!-- @trace
source: import-export-on-port
updated: 2026-11-14T22:04:44+08:00
-->

---
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

<!-- @trace
source: node-retire-legacy-store
updated: 2026-11-15T22:20:32+08:00
-->

---
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

<!-- @trace
source: hook-read-function
updated: 2026-11-18T15:33:01+08:00
-->

---
### Requirement: Agent 工具的說明指出 path 是儲存的鍵

agentTool.description SHALL 含一段給模型看的說明：動詞結果裡的 `path` 是文件在儲存裡的鍵，不是可以打開的檔案；讀文件的內容 SHALL 改用讀取的動詞，說明 SHALL 點名 `artifact cat`、`show` 與 `discuss show`。這段說明 SHALL 是英文，與 description 的其餘部分相同。agentTool 的 `name`、`inputSchema` 與動詞清單 SHALL 不因此改變。

#### Scenario: 說明提到 path 不是檔案

- **WHEN** 檢視 agentTool.description
- **THEN** 它含一句說明 `path` 是儲存的鍵而不是檔案，並點名 `artifact cat`、`show`、`discuss show` 三個讀取的動詞

#### Scenario: 其他欄位不變

- **WHEN** 比較引入本需求前後的 agentTool
- **THEN** `name` 與 `inputSchema` 深度相等；description 的動詞清單所列的動詞集合相同

<!-- @trace
source: verb-output-for-embedded
updated: 2026-11-18T21:38:05+08:00
-->

---
### Requirement: 本機引擎的 exportScope 方法

createEngine 回傳的引擎 SHALL 有方法 exportScope()：不接受參數，回傳 Promise。成功時的值 SHALL 是第 2 版的匯入包——`formatVersion` 是 2、`documents` 是陣列，每一項有 `key`（文件在儲存裡的鍵）、`content`（全文）與 `digest`（`sha256:` 加上 content 的 SHA-256 十六進位）；`documents` SHALL 依鍵的位元組順序排列，鍵 SHALL NOT 重複。值 SHALL NOT 含 `scopeVersion`：本機專案沒有範圍版本號。這個值 SHALL 能原樣交給 host.importScope 寫進一個空的範圍。

包的內容 SHALL 是這個本機專案的全部規格文件（見 workspace-migration「Bundle 組裝涵蓋本地 workspace 全集」）：進行中變更的 metadata、文件與 evidence，正式規格，進行中與已封存的討論，已封存變更的全部文件，流程設定，詞彙表，以及看板順序（一個順序鍵都沒有時不含）。把這個包匯入宿主之後，經 host.dispatch 讀到的變更、正式規格與討論 SHALL 與在原本的本機專案經 engine.dispatch 讀到的相同。

exportScope SHALL 只讀取、不寫入：本機專案的檔案 SHALL 逐位元不變。它 SHALL 於背景工作執行，SHALL NOT 阻塞 JS 事件迴圈。規格資料夾存在、但裡面沒有任何規格文件時 SHALL 成功並回傳 `documents` 為空陣列的包。

失敗時 SHALL 以 Error 拒絕 Promise，SHALL NOT 回傳不完整的包：任何一個變更（進行中或已封存）的 metadata 無法解析時，message SHALL 指出是哪個檔案；變更含有引擎的文件命名規則對不上的文件時，message SHALL 指出是哪份文件；通往規格資料夾路上的資料夾、規格資料夾本身或它底下任何一個檔案或資料夾是符號連結時（舊位置的 evidence 資料夾 `.speclink/touched/` 亦同），exportScope SHALL NOT 經連結讀檔，message SHALL 以相對專案根目錄、使用正斜線的路徑指出那個項目，連結指向的內容 SHALL NOT 出現在 message 裡（見 workspace-migration「本機專案的匯出與遷移產生相同的文件集合」）。

exportScope SHALL 只讀 createEngine 的 root 底下的規格資料夾：規格資料夾名稱（來自本機專案 `.speclink.yaml` 的 `spec_dir`，或 createEngine 的 `specDir`）是絕對路徑、含 `..` 或 `.` 路徑段、或是空的時，exportScope SHALL 在讀取任何檔案之前以 Error 拒絕，`code` 是 `error`，message SHALL 以雙引號原樣指出那個名稱，SHALL NOT 含任何檔案內容。專案根目錄裡面的多層名稱（例如 `docs/specs`）SHALL 照常匯出。engine.dispatch 讀規格資料夾的方式不受這條影響。Error 的形狀（message 加 code 欄位）SHALL 與 engine.dispatch 的失敗相同。createHost 回傳的宿主 SHALL 維持既有的 exportScope（接受身分與範圍），兩者互不影響。

規格資料夾名稱的檢查與符號連結的檢查都通過之後，root 底下以規格資料夾名稱指到的位置不是規格資料夾時——不存在、不是資料夾，或資料夾裡沒有 `changes/`、`specs/`、`discussions/` 子資料夾也沒有 `config.yaml`——exportScope SHALL 以 Error 拒絕，`code` 是 `error`，message SHALL 以 `no spec folder:` 開頭並以雙引號原樣指出規格資料夾名稱，SHALL NOT 含任何檔案內容或絕對路徑，SHALL NOT 回傳任何包；本機專案的檔案 SHALL 逐位元不變。規格資料夾名稱本身不合規則時（絕對路徑、含 `..` 或 `.` 段、空的），SHALL 維持上一段的拒絕，SHALL NOT 改報這一條。這條只管 createEngine 的 exportScope：Speclink Desktop 的「遷移到 Server」SHALL 維持既有行為，沒有規格資料夾的本機資料夾仍得到空的文件集合。

#### Scenario: 匯出本機專案並匯入宿主

- **WHEN** 以 createEngine 指向一個含一個進行中變更 demo（有提案與任務）、一份正式規格 auth、一份進行中的討論 topic 的本機專案，呼叫 `await engine.exportScope()`，再把結果交給以 JS 記憶體儲存建立的宿主的 `host.importScope({ identity, scope: 's1', bundle })`
- **THEN** 匯入成功；之後 `host.dispatch` 在範圍 s1 執行 `['list', '--json']`、`['show', 'auth', '--item-type', 'spec']`、`['discuss', 'list', '--json']`，結果與在本機專案以 engine.dispatch 執行同樣的 argv 相同（清單裡只有本機才有的欄位除外）

##### Example: 匯出的包的形狀

本機專案只有一個變更 demo，它只有提案與 metadata。

| 鍵（依順序） | 內容 |
| --- | --- |
| `changes/demo/.meta.yaml` | 變更的 metadata 原文 |
| `changes/demo/proposal.md` | 提案原文 |

回傳值是 `{ "formatVersion": 2, "documents": [ … 上面兩項，各有 key、content、digest … ] }`，沒有 `scopeVersion`。

#### Scenario: 匯出不改動本機專案

- **WHEN** 記下本機專案規格資料夾每個檔案的內容與修改時間，呼叫 `await engine.exportScope()`
- **THEN** 每個檔案的內容與修改時間都沒有改變，沒有新增或刪除的檔案

#### Scenario: 壞掉的 metadata 讓整個匯出失敗

- **WHEN** 本機專案有兩個變更，其中 broken 的 metadata 無法解析，呼叫 `engine.exportScope()`
- **THEN** Promise 以 Error 拒絕，message 指出 broken 的 metadata 檔；沒有回傳任何包

#### Scenario: 規格資料夾裡的符號連結讓匯出失敗

- **WHEN** 本機專案的 `speclink/specs/leak/spec.md` 是一個連到專案外檔案的符號連結，呼叫 `engine.exportScope()`
- **THEN** Promise 以 Error 拒絕，`code` 是 `error`，message 含 `speclink/specs/leak/spec.md`、不含那個外部檔案的內容；沒有回傳任何包

#### Scenario: 規格資料夾指到專案外讓匯出失敗

- **WHEN** 本機專案的 `.speclink.yaml` 寫 `spec_dir: ../other/speclink`，那個資料夾裡有一份正式規格，呼叫 `engine.exportScope()`
- **THEN** Promise 以 Error 拒絕，`code` 是 `error`，message 含 `"../other/speclink"`、不含那份正式規格的內容；沒有回傳任何包

##### Example: 規格資料夾名稱

| 規格資料夾名稱 | 結果 |
| --- | --- |
| `../other/speclink` | 失敗，message 含 `"../other/speclink"` |
| 專案外的一個絕對路徑 | 失敗，message 含那個路徑 |
| `docs/specs` | 成功，包含 `docs/specs` 裡的文件 |

#### Scenario: 空的專案

- **WHEN** 以 createEngine 指向一個剛初始化、沒有任何變更、規格與討論的本機專案，呼叫 `await engine.exportScope()`
- **THEN** 回傳的包 `formatVersion` 是 2；`documents` 只含初始化時就存在的文件（例如流程設定），沒有的話是空陣列

#### Scenario: 已封存的變更與討論也在包裡

- **WHEN** 本機專案有一個已封存的變更與一份已封存的討論，呼叫 `await engine.exportScope()` 並匯入宿主
- **THEN** 宿主的已封存變更清單與 `discuss list --archived` 都列出它們，內容與本機相同

#### Scenario: 沒有規格資料夾的 root 讓匯出失敗

- **WHEN** 以 createEngine 指向一個沒有 `.speclink.yaml`、也沒有 `speclink/` 與 `openspec/` 的資料夾（例如後端把下載的內容放錯位置），呼叫 `engine.exportScope()`
- **THEN** Promise 以 Error 拒絕，`code` 是 `error`，message 以 `no spec folder: "speclink"` 開頭；沒有回傳任何包，資料夾裡的檔案不變

##### Example: root 底下的內容與結果

| root 底下的內容 | 結果 |
| --- | --- |
| `speclink/specs/auth/spec.md` | 成功，包含那份正式規格 |
| `speclink/` 只有一個空的 `specs/` | 成功，`documents` 是空陣列 |
| `speclink/` 只有一個 `README.md` | 失敗，message 以 `no spec folder: "speclink"` 開頭 |
| 沒有 `speclink/` | 失敗，message 以 `no spec folder: "speclink"` 開頭 |
| `.speclink.yaml` 寫 `spec_dir: docs/specs`，但沒有 `docs/specs/` | 失敗，message 以 `no spec folder: "docs/specs"` 開頭 |
| `.speclink.yaml` 寫 `spec_dir: ../other/speclink` | 失敗，message 含 `spec folder outside the project:` 與 `"../other/speclink"`，不是 `no spec folder:` |

#### Scenario: 只下載規格資料夾時以 specDir 指定名稱

- **WHEN** 後端只把 repo 的規格資料夾下載到 `<tmp>/repo/specs-folder/`（裡面有 `specs/` 與 `changes/`），以 `createEngine({ store: { type: 'fs', root: '<tmp>/repo', specDir: 'specs-folder' } })` 呼叫 `engine.exportScope()`
- **THEN** 成功，包含 `specs-folder` 裡的文件


<!-- @trace
source: show-detail-and-export-check
updated: 2026-11-19T16:02:55+08:00
-->

---
### Requirement: SDK 匯出命令名稱清單

`@speclink/engine` SHALL 匯出 `commandNames`：一個凍結的字串陣列，含引擎每一種命令的名稱，以及宿主套件自己的三個命令名稱 `scope-export`、`scope-import` 與 `put-board-order`（host.handleRequest 寫看板排序時用的名稱），每個名稱正好一次。宿主的 beforeCommand 掛鉤與儲存的 commit 函式收到的 `command` SHALL 一定是這個陣列裡的一個名稱。陣列裡名稱的順序 SHALL NOT 是契約的一部分。`commandNames` SHALL NOT 帶命令的其他性質（例如會不會寫入）。型別定義檔 SHALL 把它宣告成唯讀的字串陣列。

#### Scenario: 匯入得到全部名稱

- **WHEN** 以 `const { commandNames } = require('@speclink/engine')` 取得清單
- **THEN** 它是 57 個字串的陣列，沒有重複；含 `new-change`、`claim-release`、`workflow-config-edit`、`scope-export`、`scope-import` 與 `put-board-order`

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

<!-- @trace
source: sdk-command-names-and-transfer-reasons
updated: 2026-11-20T15:39:51+08:00
-->

---
### Requirement: 匯出與匯入的失敗帶原因代碼

`engine.exportScope`、`host.exportScope` 與 `host.importScope` 三個呼叫 reject 的 Error，在失敗的原因屬於下表時 SHALL 多一個字串屬性 `reason`，值是表裡的代碼。除了下面寫明的產出流程失敗之外，Error 的 `code` 與 `message` SHALL 與引入本需求之前逐字相同。

| 呼叫 | 失敗的原因 | `code` | `reason` |
| --- | --- | --- | --- |
| `engine.exportScope` | `root` 底下沒有規格資料夾，或那個資料夾沒有規格內容 | `error` | `no_spec_folder` |
| `engine.exportScope` | 規格資料夾的名稱指到專案之外 | `error` | `spec_folder_outside_project` |
| `engine.exportScope` | 規格資料夾、通往它的資料夾、或它底下的檔案或資料夾是符號連結；舊位置的 evidence 資料夾 `.speclink/touched/` 裡有符號連結也算 | `error` | `symlink_in_spec_folder` |
| `engine.exportScope` | 規格資料夾底下有資料夾無法列出內容；舊位置的 evidence 資料夾無法列出內容也算 | `error` | `spec_folder_unreadable` |
| `engine.exportScope` | 進行中或已封存的變更的 metadata 檔無法解析 | `error` | `invalid_change_metadata` |
| `engine.exportScope` | 進行中或已封存的變更指名的產出流程不能用：找不到、定義無法載入，或有引擎列不出的檔案樣式 | `error` | `change_schema_unusable` |
| `host.importScope` | 目標範圍已經有文件 | `refused` | `scope_not_empty` |
| `host.importScope` | 匯入包的內容超過宿主設定的大小上限 | `invalid_argv` | `bundle_too_large` |
| `host.importScope` | 匯入包裡有一份文件不合規則：鍵不合引擎的命名規則、digest 不是內容的 digest、鍵重複，或看板排序的內容超過它的大小上限 | `invalid_argv` | `invalid_bundle_document` |

這份清單 SHALL 是固定的：既有代碼的字串與意思 SHALL NOT 改變，之後新增的代碼只會加在還沒有 `reason` 的失敗上。失敗的原因不在表裡時，Error SHALL NOT 有 `reason` 屬性；這包含儲存的錯誤、beforeCommand 掛鉤的拒絕、匯出時範圍在每次嘗試都改變、匯入包的格式版本不支援或不是匯入包。host.dispatch 與 engine.dispatch 的失敗 SHALL NOT 有 `reason` 屬性。host.handleRequest 的 HTTP 回應（`GET /export`、`POST /import`）SHALL 維持既有的狀態碼與 body。

`reason` 是 `invalid_change_metadata` 或 `change_schema_unusable` 時，Error SHALL 另有兩個字串屬性，指出是哪一個變更：`change` 是那個變更在磁碟上的資料夾名稱（已封存的變更是帶日期的名稱）；`path` 是那個變更的 metadata 檔相對於專案根目錄的路徑，使用正斜線、以規格資料夾的名稱開頭。`path` SHALL 指名實際存在的那個檔案：有 `.meta.yaml` 時是它，只有舊檔名 `.openspec.yaml` 時是舊檔名，兩個都沒有時是 `.meta.yaml` 的位置。其他失敗的 Error SHALL NOT 有 `change` 與 `path` 屬性。`change` 與 `path` SHALL NOT 含絕對路徑，SHALL NOT 含任何檔案內容。

`reason` 是 `change_schema_unusable` 時，`message` SHALL 是 `change '<變更的資料夾名稱>': ` 接上引入這個代碼之前的訊息（刻意變更：以前的訊息沒有變更名稱）。`reason` 是 `invalid_change_metadata` 時，`message` SHALL 與以前逐字相同。

#### Scenario: 沒有規格資料夾的匯出帶 no_spec_folder

- **WHEN** `root` 指向一個沒有規格資料夾的資料夾，呼叫 `createEngine({ store: { type: 'fs', root } }).exportScope()`
- **THEN** 呼叫 reject；Error 的 `code` 是 `error`、`reason` 是 `no_spec_folder`、`message` 以 `no spec folder:` 開頭

#### Scenario: 規格資料夾裡有符號連結

- **WHEN** 專案的 `speclink/specs/linked` 是一個符號連結，呼叫 engine.exportScope
- **THEN** 呼叫 reject；Error 的 `code` 是 `error`、`reason` 是 `symlink_in_spec_folder`、`message` 以 `symbolic link in the spec folder:` 開頭

#### Scenario: 匯入到已有文件的範圍

- **WHEN** 目標範圍已經有一份文件，呼叫 host.importScope 帶一份合法的匯入包
- **THEN** 呼叫 reject；Error 的 `code` 是 `refused`、`reason` 是 `scope_not_empty`、`message` 是 `import (create-new): the target scope already holds documents`；儲存的 commit 函式沒有寫入任何文件

#### Scenario: 掛鉤拒絕的匯入沒有 reason

- **WHEN** 宿主的 beforeCommand 對 `scope-import` 回傳拒絕，原因是 `not allowed`，呼叫 host.importScope
- **THEN** 呼叫 reject；Error 的 `code` 是 `refused`、`message` 是 `not allowed`，而且 `'reason' in error` 是 `false`

#### Scenario: 一般動詞的失敗沒有 reason

- **WHEN** 經 host.dispatch 執行 `['show', 'no-such-change', '--json']`
- **THEN** 呼叫 reject；Error 的 `code` 是 `not_found`，而且 `'reason' in error` 是 `false`

#### Scenario: 各種失敗對應的代碼

- **WHEN** 分別造出下表每一種失敗並呼叫對應的方法
- **THEN** Error 的 `code` 與 `reason` 如下表；「沒有」表示 Error 沒有 `reason` 屬性

##### Example: 匯出與匯入的失敗

| 呼叫 | 造出的情況 | `code` | `reason` |
| --- | --- | --- | --- |
| engine.exportScope | `store.specDir` 是 `../outside` | `error` | `spec_folder_outside_project` |
| engine.exportScope | `speclink/changes/demo` 是符號連結 | `error` | `symlink_in_spec_folder` |
| engine.exportScope | `speclink/changes/locked` 的權限是 000（非 Windows） | `error` | `spec_folder_unreadable` |
| engine.exportScope | `speclink/changes/demo/.meta.yaml` 的內容是 `: [` | `error` | `invalid_change_metadata` |
| engine.exportScope | `speclink/changes/demo/.meta.yaml` 的內容是 `schema: nope` | `error` | `change_schema_unusable` |
| engine.exportScope | 專案的 `speclink/schemas/broken/schema.yaml` 的內容是 `: [`，`speclink/changes/demo/.meta.yaml` 的內容是 `schema: broken` | `error` | `change_schema_unusable` |
| host.importScope | 匯入包內容 2,000 位元組，`limits.importBytes` 是 1,024 | `invalid_argv` | `bundle_too_large` |
| host.importScope | 一份文件的 `digest` 不是它內容的 digest | `invalid_argv` | `invalid_bundle_document` |
| host.importScope | 一份文件的鍵是 `notes/a.md` | `invalid_argv` | `invalid_bundle_document` |
| host.importScope | 匯入包的 `formatVersion` 是 99 | `invalid_argv` | 沒有 |
| host.exportScope | 儲存的 list 函式丟出 StoreUnavailableError | `unavailable` | 沒有 |

#### Scenario: 產出流程找不到的匯出指出是哪一個變更

- **WHEN** 專案有一個已封存的變更 `2026-03-03-old-one`，它的 `speclink/changes/archive/2026-03-03-old-one/.meta.yaml` 內容是 `schema: nope`，而專案、使用者層設定目錄與內建都沒有名為 `nope` 的產出流程，呼叫 engine.exportScope
- **THEN** 呼叫 reject，沒有回傳任何包；Error 的 `code` 是 `error`、`reason` 是 `change_schema_unusable`、`change` 是 `2026-03-03-old-one`、`path` 是 `speclink/changes/archive/2026-03-03-old-one/.meta.yaml`、`message` 是 `change '2026-03-03-old-one': Schema not found: Schema 'nope' not found in project, user, or built-in locations`

#### Scenario: metadata 無法解析的匯出帶變更的位置

- **WHEN** 專案的 `speclink/changes/demo/.meta.yaml` 內容是 `: [`，呼叫 engine.exportScope
- **THEN** 呼叫 reject；Error 的 `reason` 是 `invalid_change_metadata`、`change` 是 `demo`、`path` 是 `speclink/changes/demo/.meta.yaml`；`message` 以 `invalid speclink/changes/demo/.meta.yaml:` 開頭，與引入位置屬性之前逐字相同

#### Scenario: 與變更無關的失敗沒有位置

- **WHEN** 專案的 `speclink/specs/linked` 是一個符號連結，呼叫 engine.exportScope
- **THEN** 呼叫 reject；Error 的 `reason` 是 `symlink_in_spec_folder`，而且 `'change' in error` 與 `'path' in error` 都是 `false`

##### Example: 位置屬性在哪些失敗出現

| 造出的情況 | `reason` | `change` | `path` |
| --- | --- | --- | --- |
| 進行中的變更 `demo` 的 `.meta.yaml` 是 `schema: nope` | `change_schema_unusable` | `demo` | `speclink/changes/demo/.meta.yaml` |
| 已封存的變更 `2026-03-03-old-one` 只有舊檔名 `.openspec.yaml`，內容是 `schema: nope` | `change_schema_unusable` | `2026-03-03-old-one` | `speclink/changes/archive/2026-03-03-old-one/.openspec.yaml` |
| 進行中的變更 `demo` 的 `.meta.yaml` 是 `reviewed_rounds: ''` | `invalid_change_metadata` | `demo` | `speclink/changes/demo/.meta.yaml` |
| `root` 底下沒有規格資料夾 | `no_spec_folder` | 沒有 | 沒有 |
| `host.importScope` 匯入到已有文件的範圍 | `scope_not_empty` | 沒有 | 沒有 |


<!-- @trace
source: export-failure-reason-and-location
updated: 2026-11-21T23:50:24+08:00
-->
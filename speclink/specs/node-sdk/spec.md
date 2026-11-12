# node-sdk Specification

## Purpose

Node 綁定對外的 SDK 契約：createEngine 的兩種儲存建構形式、dispatch 的輸入與輸出契約，以及把引擎人眼輸出取回 JS 端的渲染 API（含 worktree 軸）。本 capability 保證 JS 端不必自行拼裝引擎內部結構，且渲染結果與 CLI 同源、不另立一套。

## Requirements

### Requirement: createEngine 的雙形式儲存建構
Node SDK 的 createEngine SHALL 接受兩種儲存形式：內建 fs（指定專案根與選填 spec 目錄名），或宿主實作的 Store 物件（方法回傳值或 Promise 皆可）。以 Store 物件建構且缺少必要方法時，createEngine SHALL 於建構當下拋出錯誤並列出缺少的方法名。

#### Scenario: fs 形式與 CLI 行為對等
- **WHEN** 以 createEngine 的 fs 形式指向某 fixture 專案，執行 await engine.dispatch(['list', '--json'])
- **THEN** 回傳物件與在同一專案執行 speclink list --json 的輸出 JSON 完全一致

#### Scenario: 宿主 Store 物件生效
- **WHEN** 以模擬兩個 change 的 JS Store 物件（方法回傳 Promise）建構引擎並 dispatch(['list', '--json'])
- **THEN** 回傳兩個 change 且欄位名（camelCase）與 CLI 輸出一致

#### Scenario: 缺方法建構即失敗
- **WHEN** 以缺少 artifact 寫入方法的 Store 物件呼叫 createEngine
- **THEN** 同步拋出錯誤，訊息列出缺少的方法名，不產生引擎實例


<!-- @trace
source: node-sdk
updated: 2026-07-05
code:
  - .github/workflows/node-sdk.yml
  - Cargo.lock
  - Cargo.toml
  - README.md
  - crates/speclink-cli/src/commands.rs
  - crates/speclink-core/src/init.rs
  - crates/speclink-core/src/lib.rs
  - crates/speclink-core/src/listing.rs
  - crates/speclink-node/.gitignore
  - crates/speclink-node/Cargo.toml
  - crates/speclink-node/__test__/engine.spec.ts
  - crates/speclink-node/__test__/helpers.ts
  - crates/speclink-node/__test__/render.spec.ts
  - crates/speclink-node/__test__/store-bridge.spec.ts
  - crates/speclink-node/__test__/stress.spec.ts
  - crates/speclink-node/__test__/write-path.spec.ts
  - crates/speclink-node/build.rs
  - crates/speclink-node/index.d.ts
  - crates/speclink-node/index.js
  - crates/speclink-node/package-lock.json
  - crates/speclink-node/package.json
  - crates/speclink-node/src/lib.rs
  - crates/speclink-node/src/render.rs
  - crates/speclink-node/src/store_bridge.rs
  - docs/sdk-node.md
  - docs/sdk-node.zh-TW.md
-->

---
### Requirement: dispatch 的輸入輸出契約

engine.dispatch SHALL 接受 argv 字串陣列（與 CLI 動詞詞彙一對一）與選填第二參數（stdin 內容），回傳 Promise；成功時解析為與 CLI --json 對齊的結構化物件（無 --json 形式的動詞回傳含 output 字串的物件）；失敗時以 Error 拒絕——message 為與 CLI 相同的語義化訊息並附 code 欄位。dispatch SHALL 於背景工作執行，SHALL NOT 阻塞 JS 事件迴圈。

dispatch SHALL 由與 CLI 共用的引擎命令層執行：argv 詞彙、回傳形狀與既有錯誤碼值域維持不變；對相同 workspace 狀態，dispatch 的成功結果與錯誤 SHALL 與 CLI 對應動詞語意一致，錯誤碼 SHALL 出自命令層的封閉註冊表（含 invalid_config 與 refused）。宿主 Store 提供的工作流設定文字存在但無法解析時，讀取政策的 dispatch 呼叫 SHALL 以 Error 拒絕且 code 為 invalid_config，SHALL NOT 以預設政策繼續執行。

dispatch 的 argv SHALL 由與 CLI 共用的同一份動詞解析處理：CLI 接受的寫法（含 `--name=value` 與 CLI 有的短旗標）dispatch SHALL 同樣接受；CLI 不接受的旗標與多出來的位置參數，dispatch SHALL 以 Error 拒絕、code 為 invalid_argv、message 為與 CLI 相同的解析錯誤文字（不含 ANSI 色碼），SHALL NOT 靜默忽略。argv 帶 `--json` 時，回傳的結構化物件與不帶時相同。argv 是 `--help` 時 SHALL 以 Error 拒絕、code 為 invalid_argv、message 為該動詞的說明文字。

dispatch SHALL 接受全部有引擎命令的動詞。只能在有 checkout 的一端執行的動詞（drift、review prepare、review scope、verify scope、帶 --skill 的 instructions）SHALL 以 Error 拒絕、code 為 invalid_argv、message 說明原因。dispatch 的回傳內容 SHALL 由與 CLI 共用的同一份輸出定義產生：有 --json 形式的動詞回傳與 CLI --json 內容相同的結構化物件；沒有 --json 形式的動詞回傳 { output }，output 與 CLI 在 --no-color 下的 stdout 相同。有兩種情況不經引擎命令：claim 由宿主 Store 的 claim 方法裁決，成功時回傳該方法的回傳值；status 沒有給變更名稱、而且專案沒有任何變更時，與 CLI 一樣只有一行提示，回傳 { output }，output 與 CLI 的 stdout 相同。

沒有 --json 形式的動詞，CLI 另有提示文字印到 stderr 時（例如 archive 在變更沒有任務證據時的提示），dispatch 回傳的物件 SHALL 另帶 stderr 欄位，內容與 CLI 為該動詞印到 stderr 的提示文字相同；沒有這種文字時 SHALL NOT 帶 stderr 欄位。discuss conclude 的結論已寫入、而收尾的封存步驟失敗時，dispatch SHALL 解析為 conclude 的結構化物件並另帶 closingError 欄位（失敗原因的文字），SHALL NOT 以 Error 拒絕：結論已經寫入，拒絕會讓呼叫端以為沒有寫入而重試。

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
source: skills-verbs-first-wave
updated: 2026-11-11T17:52:57+08:00
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

createEngine SHALL 接受選填 actor 欄位（"Name <email>" 格式字串），兩種儲存形式皆於建構期收下並綁定於該 engine 實例；dispatch 的操作者身分解析 SHALL 為：建構期 actor 有給值（trim 後非空）一律優先，fs 形式未給時回退 workspace 的 git identity（現行為），宿主 Store 形式未給時維持無章（現行為）。trim 後為空字串的 actor SHALL 視同未給。actor SHALL NOT 能經 dispatch 的 argv 或其他呼叫期參數傳入或覆寫——一個實例一個身分，多身分宿主以多個 engine 實例表達。

#### Scenario: fs 形式明給 actor 優先於 git identity

- **WHEN** 在設有 git user.name 與 user.email 的 fixture 專案，以 createEngine({ store: { type: 'fs', root }, actor: 'Alice <alice@example.com>' }) 建構並 dispatch(['new', 'change', 'demo'])
- **THEN** demo 的 metadata created_by 為 Alice <alice@example.com>，而非 git identity

#### Scenario: fs 形式未給 actor 維持現行回退

- **WHEN** 同一 fixture 專案以不帶 actor 的 createEngine fs 形式建構並 dispatch(['new', 'change', 'demo2'])
- **THEN** demo2 的 created_by 與在該專案執行 CLI speclink new change 的蓋章逐位元一致（git identity 回退不變）

#### Scenario: 宿主 Store 形式帶 actor 落章

- **WHEN** 以 JS Store 物件＋actor: 'Bob <bob@example.com>' 建構引擎並 dispatch(['new', 'change', 'demo3'])
- **THEN** 宿主 Store 收到的 metadata 寫入含 created_by: Bob <bob@example.com>；同引擎後續的 review／verify 蓋章動詞亦以同值落 _by 欄位

#### Scenario: 宿主 Store 形式未給 actor 維持無章

- **WHEN** 以 JS Store 物件建構引擎（不帶 actor）並 dispatch(['new', 'change', 'demo4'])
- **THEN** 寫入的 metadata 不含 created_by（與現行無章行為一致）

#### Scenario: 呼叫期無從覆寫身分

- **WHEN** 檢視 dispatch 的輸入契約並嘗試以任意 argv 影響蓋章身分
- **THEN** dispatch 不存在 actor 參數，蓋章內容只隨建構期 actor（或其回退）改變

<!-- @trace
source: node-host-actor
updated: 2026-08-24
-->

---
### Requirement: dispatch 的蓋章動詞

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


<!-- @trace
source: shared-verb-output
updated: 2026-11-11T14:07:12+08:00
-->

---
### Requirement: dispatch 的流程設定動詞

dispatch SHALL 認得 workflow-config 的全部動詞（show、set、context、rules、review 的四個、subagent-model 的三個），argv 與 stdin 沿用 CLI 的詞彙，回傳內容與 CLI 同一動詞的輸出同源。宿主 Store 物件 SHALL 可提供一個選填方法 writeWorkflowConfig(content)，回傳值或 Promise 皆可。宿主 Store 沒有提供這個方法時，會寫入的 workflow-config 動詞 SHALL 在動手前以語義化訊息拒絕、code 為 refused，宿主 Store 的任何寫入方法 SHALL NOT 被呼叫；show 與帶 --dry-run 的動詞 SHALL 不受影響。argv 帶 --stdin 而 dispatch 沒有收到 stdin 內容時，workflow-config 的動詞 SHALL 以 code invalid_argv 拒絕，宿主 Store 的任何寫入方法 SHALL NOT 被呼叫；空字串是有效的內容（清掉該值）。fs 形式建構的引擎 SHALL 直接寫入規格資料夾的 config.yaml。

#### Scenario: 經 dispatch 設定政策欄位

- **WHEN** 宿主 Store 提供 writeWorkflowConfig，現有流程設定含 tdd: false，執行 dispatch(['workflow-config', 'set', 'tdd', 'true'])
- **THEN** writeWorkflowConfig 被呼叫正好一次，收到的內容裡 tdd 是 true；Promise 解析為成功結果

#### Scenario: 沒有寫入方法時拒絕

- **WHEN** 宿主 Store 沒有 writeWorkflowConfig，執行 dispatch(['workflow-config', 'set', 'tdd', 'true'])
- **THEN** Promise 以 Error 拒絕，code 為 refused，message 指出宿主 Store 缺少 writeWorkflowConfig；宿主 Store 沒有任何寫入方法被呼叫

#### Scenario: 讀取與 dry-run 不需要寫入方法

- **WHEN** 宿主 Store 沒有 writeWorkflowConfig，分別執行 dispatch(['workflow-config', 'show']) 與 dispatch(['workflow-config', 'set', 'tdd', 'true', '--dry-run'])
- **THEN** 兩者都成功

#### Scenario: 帶 --stdin 卻沒有 stdin 內容時拒絕

- **WHEN** 宿主 Store 的流程設定含 rules.tasks 一條規則，執行 dispatch(['workflow-config', 'rules', 'tasks', '--stdin']) 而沒有傳 stdin
- **THEN** Promise 以 Error 拒絕，code 為 invalid_argv；流程設定沒有被改動。改傳 stdin 為空字串時成功，rules.tasks 被移除

#### Scenario: 壞的流程設定經 dispatch 拒絕

- **WHEN** 宿主 Store 的流程設定讀取方法回傳無法解析的 YAML，執行 dispatch(['workflow-config', 'show'])
- **THEN** Promise 以 Error 拒絕，code 為 invalid_config

<!-- @trace
source: workflow-config-commands
updated: 2026-11-11T15:31:28+08:00
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
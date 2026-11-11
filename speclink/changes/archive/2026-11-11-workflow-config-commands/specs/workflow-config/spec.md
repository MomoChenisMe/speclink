## MODIFIED Requirements

### Requirement: workflow-config show 動詞
<!-- BEFORE: show 是周邊設定動詞，不進入命令執行層，由 CLI 直接讀檔 -->

CLI SHALL 提供 speclink workflow-config show（旗標 --json 與 --no-color，無 stdin），顯示 speclink/config.yaml 的正典內容：schema（專案預設 schema 名稱——未設定時標示為未設定與其預設 spec-driven）、政策五欄（locale、spec_locale、tdd、audit、worktree——未設定的欄位標示為未設定與其預設語意）、context（有無與行數）、rules（各 artifact 節的條數）與 review（預設清單模式與三個嚴重度清單的條數）。show SHALL 顯示正典值，SHALL NOT 應用環境變數的覆寫（有效值的三層解析屬 instructions payload 職責）。帶 --json 時 SHALL 輸出 camelCase payload：schema（字串，未設定為 null）、locale、specLocale（未設定為 null）、tdd、audit、worktree（布林）、context（字串或 null）、rules（artifact id 對規則字串陣列的物件）、review（物件：baseline 為字串 "extend" 或 "replace"，critical、warning、suggestion 為規則字串陣列；檔案未寫 review 區塊時為 baseline "extend" 與三個空陣列）、subagentModel（物件：review、verify 為 AI 工具名稱對模型名稱字串的物件，choices 為 AI 工具名稱對模型名稱字串陣列的物件；檔案未寫 subagent_model 區塊時為三個空物件）與 builtinSubagentModels（引擎內建的子代理模型清單：AI 工具名稱對模型名稱字串陣列的物件，恰為 {"claude":["sonnet","opus","haiku","fable"]}，不隨 config 內容改變）。人眼輸出 SHALL 於標題空行之後、locale 行之前，以與其他行相同的標籤欄寬印出 schema 行：值為 config 的 schema 名稱，未設定時為 unset (spec-driven)。人眼輸出 SHALL 於 rules 行之後、以與其他行相同的標籤欄寬印出 review 行：baseline 為 extend（含未設定）且三個清單皆空時值為 none，否則值為「<baseline>, critical <n>, warning <n>, suggestion <n>」。fs 模式讀取 speclink/config.yaml；remote 模式 SHALL 經既有連線讀取 server 端 config 文件，人眼與 --json 輸出形狀與 fs 模式一致。config 文件無法解析時 SHALL 沿用既有 fail-closed 行為：非零 exit code、stderr 指出檔案與解析原因。本機模式與 SDK 的 dispatch SHALL 經引擎的命令執行層執行本動詞，SHALL NOT 自行讀取檔案或儲存；remote 模式的 CLI 經連線取得 server 的文件後，SHALL 以同一個引擎解析與同一份輸出呈現。成功 exit code 0、輸出至 stdout；--no-color 下無 ANSI 色彩。本次修訂為刻意變更：--json 於 review 之後新增 subagentModel 與 builtinSubagentModels 兩個欄位；人眼輸出不顯示 subagent_model，各行與既有 --json 欄位 SHALL 維持既有位元級輸出。

#### Scenario: fs 模式顯示正典值

- **WHEN** speclink/config.yaml 含 locale: tw、tdd: true 與 context，執行 speclink workflow-config show
- **THEN** stdout 顯示 locale 為 tw、tdd 開啟、audit 未設定（預設關閉）、worktree 未設定（預設關閉）、context 存在；exit code 0

#### Scenario: --json payload 形狀

- **WHEN** 執行 speclink workflow-config show --json
- **THEN** stdout 為 JSON，欄位 schema、locale、specLocale、tdd、audit、worktree、context、rules、review、subagentModel、builtinSubagentModels 一律 camelCase；未設定的 schema 為 null、未設定的 specLocale 為 null、未設定的 tdd 為 false、未設定的 worktree 為 false、未寫 review 區塊時 review 為 baseline "extend" 與 critical、warning、suggestion 三個空陣列、未寫 subagent_model 區塊時 subagentModel 為 {"review":{},"verify":{},"choices":{}}、builtinSubagentModels 為 {"claude":["sonnet","opus","haiku","fable"]}

#### Scenario: show 不應用環境變數覆寫

- **WHEN** 設定 SPECLINK_TDD=false 且 speclink/config.yaml 含 tdd: true，執行 speclink workflow-config show --json
- **THEN** payload 的 tdd 為 true（正典值，非解析後有效值）

#### Scenario: remote 模式輸出形狀一致

- **WHEN** 於 remote 綁定的 workspace 執行 speclink workflow-config show --json
- **THEN** stdout 的 JSON 欄位名與 fs 模式完全一致，值來自 server 端 config 文件

#### Scenario: 壞 config fail-closed

- **WHEN** speclink/config.yaml 含 YAML 語法錯誤，執行 speclink workflow-config show
- **THEN** exit code 非 0，stderr 指出該檔與解析原因，stdout 無 payload

#### Scenario: 人眼輸出的 review 行

- **WHEN** 執行 speclink workflow-config show --no-color
- **THEN** stdout 於 rules 行之後恰有一行 review，標籤欄寬與 rules 行相同，值依 review 區塊內容而定；其餘各行與本次修訂前逐位元一致

##### Example: review 行的值

| config.yaml 的 review 區塊 | review 行的值 |
| -------------------------- | ------------- |
| 未寫 review | none |
| critical 兩條、warning 一條、suggestion 一條，未寫 baseline | extend, critical 2, warning 1, suggestion 1 |
| baseline: replace、三個清單皆空 | replace, critical 0, warning 0, suggestion 0 |

#### Scenario: --json 帶出 review 內容

- **WHEN** speclink/config.yaml 含 review 區塊 baseline: replace 與 critical 一條「所有 SQL 一律用參數化查詢」，執行 speclink workflow-config show --json
- **THEN** payload 的 review 為 {"baseline":"replace","critical":["所有 SQL 一律用參數化查詢"],"warning":[],"suggestion":[]}；exit code 0

#### Scenario: --json 帶出子代理模型設定

- **WHEN** speclink/config.yaml 含 subagent_model 區塊，review 為 claude: opus 與 codex: gpt-6.1-sol、verify 為 claude: sonnet、choices 為 codex 兩條 gpt-6.1-sol 與 gpt-6-astra，執行 speclink workflow-config show --json
- **THEN** exit code 0；payload 的 subagentModel 為 {"review":{"claude":"opus","codex":"gpt-6.1-sol"},"verify":{"claude":"sonnet"},"choices":{"codex":["gpt-6.1-sol","gpt-6-astra"]}}，builtinSubagentModels 為 {"claude":["sonnet","opus","haiku","fable"]}

#### Scenario: 人眼輸出不顯示子代理模型設定

- **WHEN** speclink/config.yaml 含上一個 Scenario 的 subagent_model 區塊，執行 speclink workflow-config show --no-color
- **THEN** exit code 0；stdout 與同一份 config 刪掉 subagent_model 區塊後的輸出逐位元一致

#### Scenario: 顯示專案預設 schema

- **WHEN** 執行 speclink workflow-config show --no-color 與 speclink workflow-config show --json
- **THEN** 人眼輸出於 locale 行之前恰有一行 schema，標籤欄寬與 locale 行相同；--json 的 schema 欄位值依 config 的 schema 鍵而定；exit code 0

##### Example: schema 的值

| config.yaml 的 schema 鍵 | 人眼 schema 行的值 | --json 的 schema |
| ------------------------ | ------------------ | ---------------- |
| 未寫 schema | unset (spec-driven) | null |
| schema: spec-driven | spec-driven | "spec-driven" |
| schema: my-flow | my-flow | "my-flow" |

#### Scenario: show 經命令執行層

- **WHEN** 分別執行 speclink workflow-config show --json 與 engine.dispatch(['workflow-config', 'show'])
- **THEN** dispatch 解析出的物件與 CLI 的 stdout 經 JSON 解析後的值相等

## ADDED Requirements

### Requirement: 流程設定的寫入經命令執行層

workflow-config 的全部寫入動詞（set、context、rules、review 的 critical／warning／suggestion／baseline、subagent-model 的 review／verify／choices）SHALL 經引擎的命令執行層執行：由引擎讀出現有內容、套用編輯、驗證、經儲存介面寫回，各入口 SHALL NOT 自行寫檔或自行開交易。引擎 SHALL 另外提供整份取代的寫入命令。寫入命令的結果 SHALL 帶寫入前的內容、寫入後的內容、變動的鍵與是否真的寫入。變動的鍵 SHALL 是下列值的子集，依此順序：schema、locale、spec_locale、tdd、audit、worktree、context、rules.<artifact>、review.baseline、review.critical、review.warning、review.suggestion、subagent_model.review、subagent_model.verify、subagent_model.choices。新內容無法解析或 locale 不合法時，命令 SHALL 在寫入前以 invalid_config 拒絕，儲存 SHALL 沒有變動。新內容與現有內容逐位元相同時，命令 SHALL NOT 寫入。帶 dry-run 時命令 SHALL NOT 寫入，結果仍 SHALL 帶變動的鍵。一個寫入命令 SHALL 最多寫入一次。CLI 既有的指令、旗標、--help、人眼輸出、--json、--dry-run 的差異、exit code，以及本機模式寫出的 config.yaml 內容 SHALL 維持位元級一致。需要 checkout 的處理——關閉 worktree 前檢查進行中的 worktree、worktree 欄位變動後同步技能檔、審查規則與子代理模型變動後的技能檔過期提示——SHALL 由有 checkout 的入口在命令前後執行，SHALL NOT 在引擎內執行。這兩項後置處理 SHALL 依變動的鍵決定：本機模式真的寫入且變動的鍵含 worktree 時同步技能檔；本機與 remote 模式真的寫入且變動的鍵含 review. 或 subagent_model. 開頭的鍵時才探測並輸出過期提示。位元級一致的唯一例外是值沒有變的寫入（變動的鍵是空的）：確認訊息照常輸出，但 SHALL NOT 同步技能檔、SHALL NOT 輸出過期提示；新內容與現有內容逐位元相同時，本機模式另外 SHALL NOT 建立或重寫 config.yaml——變更前的 CLI 在這三處仍會動作。remote 模式的 CLI 流程不變，值沒有變時仍送出寫入。shell 補全腳本裡子指令的排列順序不在位元級一致的範圍內。

#### Scenario: set 回報變動的鍵

- **WHEN** config.yaml 含 tdd: false，經引擎執行「把 tdd 設成 true」的編輯
- **THEN** 結果的變動鍵是 tdd，是否寫入為真，寫入後的內容裡 tdd 是 true，其他行與寫入前逐位元相同

##### Example: 各種編輯的變動鍵

| 編輯 | 變動的鍵 |
| ---- | -------- |
| set locale ja | locale |
| context 取代成新的文字 | context |
| rules proposal 取代成兩條規則 | rules.proposal |
| review critical 取代成一條規則 | review.critical |
| review baseline replace | review.baseline |
| subagent-model review claude opus | subagent_model.review |

#### Scenario: 值沒有變時不寫入

- **WHEN** config.yaml 含 tdd: true，經引擎執行「把 tdd 設成 true」的編輯
- **THEN** 結果的變動鍵是空的，是否寫入為假，儲存沒有收到寫入

#### Scenario: 值沒有變時不同步技能也不提示

- **WHEN** config.yaml 的 worktree 已是 true，執行 speclink workflow-config set worktree true
- **THEN** stdout 只有確認訊息 ✓ worktree = true，沒有技能同步的那一行；config.yaml 與技能檔都沒有被改動；exit code 0

##### Example: 值沒有變的寫入

| 現況 | 指令 | stdout | 沒有發生的事 |
| ---- | ---- | ------ | ------------ |
| worktree: true | set worktree true | ✓ worktree = true | 重寫 config.yaml、同步技能檔 |
| 沒有 worktree 鍵 | set worktree false | ✓ worktree = false | 重寫 config.yaml、同步技能檔 |
| 沒有 review 區塊，技能檔版號舊於引擎 | review baseline extend | ✓ review.baseline = extend (default) | 重寫 config.yaml、過期提示 |
| 沒有 subagent_model 區塊，技能檔版號舊於引擎 | subagent-model verify claude "" | ✓ subagent_model.verify.claude unchanged | 重寫 config.yaml、過期提示 |
| 沒有 config.yaml | set tdd false | ✓ tdd = false | 建立 config.yaml |

#### Scenario: remote 模式值沒有變時不提示

- **WHEN** 於 remote 綁定的 workspace，server 的流程設定沒有 review 區塊，本機技能檔版號舊於引擎，執行 speclink workflow-config review baseline extend
- **THEN** stdout 是 ✓ review.baseline = extend (default)，寫入照常送出，stderr 沒有過期提示，exit code 0

#### Scenario: dry-run 不寫入

- **WHEN** 執行 speclink workflow-config set tdd true --dry-run
- **THEN** stdout 是與變更前相同的差異輸出，config.yaml 沒有被改動，exit code 0

#### Scenario: 不合法的內容在寫入前被拒絕

- **WHEN** 經引擎執行整份取代，內容是無法解析的 YAML
- **THEN** 命令以 invalid_config 拒絕，儲存裡的流程設定與執行前相同

#### Scenario: CLI 的輸出與寫出的檔案不變

- **WHEN** 於這個變更前後，在同一份 config.yaml 上執行 speclink workflow-config rules proposal --stdin 並給同樣的 stdin
- **THEN** stdout、stderr、exit code 逐位元一致，寫出的 config.yaml 逐位元一致

#### Scenario: 關閉 worktree 的檢查留在有 checkout 的入口

- **WHEN** 專案有進行中的 worktree，執行 speclink workflow-config set worktree false
- **THEN** CLI 以與變更前相同的訊息拒絕，config.yaml 沒有被改動

#### Scenario: Server 的整份寫入經同一個命令

- **WHEN** 用戶端以正確的預期版本送出整份流程設定
- **THEN** Server 的回應形狀與變更前相同；內容無法解析時回 invalid_config，預期版本不符時回 revision_conflict，兩者同時成立時回 invalid_config；內容與現有的流程設定逐位元相同時不提交，回應的版本等於寫入前的版本

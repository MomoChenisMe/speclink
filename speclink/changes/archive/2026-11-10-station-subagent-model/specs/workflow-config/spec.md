## ADDED Requirements

### Requirement: config 的 subagent_model 區塊與嚴格解析

speclink/config.yaml SHALL 支援頂層鍵 subagent_model，供品質關卡指定子代理模型。子鍵限三個：review 與 verify（各為 AI 工具名稱對模型名稱的 mapping；省略為空）、choices（AI 工具名稱對模型名稱清單的 mapping，即各 AI 工具的自訂清單；省略為空）。subagent_model 鍵缺席或為 null 時 SHALL 等同三個子鍵皆空——每一站、每一個 AI 工具都沿用主對話的模型。

- AI 工具名稱 SHALL 為 kebab-case：2 到 50 個字元，只含小寫英文字母、數字與連字號，不以連字號開頭或結尾，不含連續連字號。同一個 mapping（review、verify 或 choices）裡，同一個 AI 工具名稱 SHALL 只出現一次。名稱 SHALL NOT 限於本 checkout .speclink.yaml 啟用的工具，也 SHALL NOT 限於內建工具。
- 模型名稱與清單條目 SHALL 為 YAML 字串，讀取時去除頭尾空白；引擎 SHALL NOT 檢查模型名稱是否存在。review 或 verify 的某格值為 null 或只有空白時，SHALL 讀作該格沒設定；清單中的空白條目 SHALL 被略過，null 清單 SHALL 讀作空清單。站上的值 SHALL NOT 被要求出現在 choices。
- subagent_model 不屬政策欄位：SHALL NOT 有環境變數覆寫，SHALL NOT 讀取 .speclink.yaml 的同名鍵。

subagent_model 區塊出現未知子鍵、AI 工具名稱不是 kebab-case 或在同一個 mapping 重複出現、review 或 verify 不是 mapping、某格值不是字串或 null、choices 的值不是字串清單時，引擎 SHALL 將整份 config 視為無法解析並沿用既有 fail-closed：讀取 config 的指令（含 workflow-config show、instructions）以非零 exit code 結束，stderr 指出 speclink/config.yaml 與解析原因，原因 SHALL 點名出錯的鍵或值；SHALL NOT 退回預設值。config.yaml 的頂層 SHALL 維持寬鬆：不認得 subagent_model 的舊版 CLI 讀到它時略過整個區塊。remote 寫入時 server SHALL 以同一引擎解析拒收，回應 invalid_config，store 中文件與 revision 皆不變。

#### Scenario: 未寫 subagent_model 時為預設

- **WHEN** speclink/config.yaml 沒有 subagent_model 鍵，執行 speclink workflow-config show --json
- **THEN** exit code 0，payload 的 subagentModel 為 {"review":{},"verify":{},"choices":{}}

#### Scenario: 子鍵打錯即 fail-closed

- **WHEN** speclink/config.yaml 的 subagent_model 區塊含子鍵 reveiw（review 拼錯），執行 speclink workflow-config show
- **THEN** exit code 非 0，stderr 指出 speclink/config.yaml 且原因含 reveiw；stdout 無輸出

##### Example: 嚴格解析的判定

| subagent_model 區塊內容 | 結果 |
| ----------------------- | ---- |
| reveiw: {claude: opus} | 拒收，原因點名 reveiw |
| review: {Claude: opus} | 拒收，原因點名 Claude 不是 kebab-case |
| review: {claude: 3} | 拒收，原因指出 claude 的值不是字串 |
| review: [claude] | 拒收，原因指出 review 不是 mapping |
| choices: {codex: gpt-6.1-sol} | 拒收，原因指出 codex 的值不是清單 |
| review 底下 claude: opus 與 claude: sonnet 兩行 | 拒收，原因點名重複的 claude |
| review: {claude: opus, my-agent: some-model} | 正常讀取，my-agent 為合法名稱 |
| review: {claude: "  "} | 正常讀取，claude 該格讀作沒設定 |
| review: {claude: not-a-real-model} | 正常讀取，模型名稱不檢查 |

#### Scenario: 其他指令同樣 fail-closed

- **WHEN** speclink/config.yaml 的 subagent_model 區塊含子鍵 model，執行 speclink instructions apply --change <name> --json
- **THEN** exit code 非 0，stderr 指出 speclink/config.yaml 且原因含 model；不輸出以預設值組成的 payload

#### Scenario: 環境變數與 .speclink.yaml 不影響子代理模型

- **WHEN** .speclink.yaml 含 subagent_model 鍵，speclink/config.yaml 沒有 subagent_model 鍵，執行 speclink workflow-config show --json
- **THEN** exit code 0，payload 的 subagentModel 為三個空物件

#### Scenario: remote 寫入以同一解析拒收

- **WHEN** client 以正確的 expected revision 向 server 提交 subagent_model 區塊含子鍵 reveiw 的 config 內容
- **THEN** server 回應 invalid_config 並指出解析原因，store 中文件內容與 revision 皆未改變

---
### Requirement: workflow-config subagent-model 寫入

CLI SHALL 提供 speclink workflow-config subagent-model 的三個寫入形式，皆支援 --dry-run 與 --no-color：

- speclink workflow-config subagent-model review <tool> <model> 與 speclink workflow-config subagent-model verify <tool> <model>：把該站該 AI 工具的模型設為 <model>（去除頭尾空白）；<model> 為空字串或只有空白時 SHALL 移除該格（回到沿用主對話）。這兩個形式沒有 stdin。
- speclink workflow-config subagent-model choices <tool> --stdin：以 stdin 整段代換該 AI 工具的自訂清單——一行一條、頭尾空白去除、空行略過；stdin 為空或只有空白時 SHALL 移除該 AI 工具的清單。未帶 --stdin 時 SHALL 以非零 exit code 於 stderr 說明用法，且無檔案效果。

寫入後 review、verify、choices 皆空時，SHALL 移除整個 subagent_model 鍵。第一個參數不是 review、verify、choices 之一，或 <tool> 不是 kebab-case（規則同 subagent_model 區塊的 AI 工具名稱）時，SHALL 以非零 exit code 拒絕，stderr 列出合法值或名稱規則，且無檔案效果。寫入 SHALL 僅代換 subagent_model 區塊——config.yaml 的其他鍵、註解與空行逐位元保留；區塊內的註解不保留，AI 工具名稱依字母序寫出；以 YAML 保留字元開頭的模型名稱 SHALL 自動加引號，讀回時逐字元一致。--dry-run 行為、remote 模式的讀-改-寫與版本衝突處理、離線與認證失效、壞檔 fail-closed SHALL 與 workflow-config set 政策欄位寫入一致。成功 exit code 0、stdout 單行成功訊息；--no-color 下無 ANSI 色彩。

寫入結果與寫入前的 subagent_model 區塊相同時（例如移除一個沒設定的格子、設成現有的值，或以空 stdin 移除一份不存在的清單），SHALL NOT 改寫區塊：speclink/config.yaml 逐位元不變，stdout 的成功訊息為 `✓ <鍵路徑> unchanged`。

寫入成功後（帶 --dry-run 時除外），CLI SHALL 以「審查規則寫入後的技能檔過期提示」相同的探測與判定條件檢查本機工作區的技能檔，只有提示內容不同：stderr 恰一行以 `speclink: warning: ` 開頭、不帶 ANSI 色彩，列出每個過期工具的名稱與其技能版號、引擎現版號，說明舊版品質關卡技能可能不套用這個子代理模型，並指示執行 speclink update。stdout 與 exit code 不受提示影響；remote 模式以本機工作區判定。

#### Scenario: 設定一格保留其他內容

- **WHEN** config.yaml 含 locale: tw 與 subagent_model.verify 的 claude: sonnet，執行 speclink workflow-config subagent-model review claude opus
- **THEN** exit code 0；subagent_model.review.claude 為 opus、subagent_model.verify.claude 仍為 sonnet，locale 行與 subagent_model 以外的內容逐位元不變；stdout 單行成功訊息

##### Example: 寫入效果與成功訊息

| 操作前 subagent_model 區塊 | 指令 | 寫入後 subagent_model 區塊 | stdout |
| -------------------------- | ---- | -------------------------- | ------ |
| 無 subagent_model 鍵 | subagent-model review claude opus | review 的 claude 為 opus | ✓ subagent_model.review.claude = opus |
| review 的 claude 為 opus | subagent-model review claude sonnet | review 的 claude 為 sonnet | ✓ subagent_model.review.claude = sonnet |
| review 的 claude 為 opus、verify 的 codex 為 gpt-6.1-sol | subagent-model verify codex "" | review 的 claude 為 opus | ✓ subagent_model.verify.codex removed |
| review 的 claude 為 opus | subagent-model review claude "" | subagent_model 鍵被移除 | ✓ subagent_model.review.claude removed |
| 無 subagent_model 鍵 | subagent-model choices codex --stdin（兩行） | choices 的 codex 兩條 | ✓ subagent_model.choices.codex = 2 entries |
| choices 的 codex 兩條 | subagent-model choices codex --stdin（空） | subagent_model 鍵被移除 | ✓ subagent_model.choices.codex removed |

#### Scenario: 寫入的名稱可以是任何 kebab-case 工具

- **WHEN** 本 checkout 的 .speclink.yaml 只啟用 claude，執行 speclink workflow-config subagent-model review my-agent some-model
- **THEN** exit code 0；subagent_model.review 的 my-agent 為 some-model

#### Scenario: 不認得的形式或工具名稱

- **WHEN** 分別執行 speclink workflow-config subagent-model audit claude opus 與 speclink workflow-config subagent-model review Claude opus
- **THEN** 兩者 exit code 皆非 0，前者 stderr 列出 review、verify、choices，後者 stderr 指出 Claude 不是 kebab-case；speclink/config.yaml 逐位元不變

#### Scenario: choices 未帶 --stdin 說明用法

- **WHEN** 執行 speclink workflow-config subagent-model choices codex（無 --stdin）
- **THEN** exit code 非 0，stderr 說明需以 --stdin 提供內容；speclink/config.yaml 逐位元不變

#### Scenario: dry-run 印 diff

- **WHEN** 執行 speclink workflow-config subagent-model verify claude sonnet --dry-run
- **THEN** exit code 0；stdout 為 unified diff，只顯示 subagent_model 區塊的變更；speclink/config.yaml 逐位元不變

#### Scenario: 壞檔拒寫

- **WHEN** speclink/config.yaml 含 YAML 語法錯誤，執行 speclink workflow-config subagent-model review claude opus
- **THEN** exit code 非 0，stderr 指出該檔與解析原因並說明拒寫；檔案逐位元不變

#### Scenario: 技能檔過期時寫入後提示

- **GIVEN** .speclink.yaml 的 tools 為 claude，.claude/skills/ 下技能檔的 frontmatter 版號舊於引擎現版
- **WHEN** 執行 speclink workflow-config subagent-model review claude opus
- **THEN** exit code 0；stdout 恰為 `✓ subagent_model.review.claude = opus`；stderr 恰有一行過期提示；帶 --dry-run 執行時 stderr 不含提示

##### Example: 提示行

| 工作區技能檔 | 引擎現版 | stderr 提示行 |
| ------------ | -------- | ------------- |
| claude v1.44.0 | v1.54.0 | speclink: warning: skill files are older than this engine (claude v1.44.0; engine v1.54.0) — an older quality-station skill may not apply this sub-agent model; run `speclink update` |

#### Scenario: 不改變內容的寫入不動檔案

- **GIVEN** speclink/config.yaml 的 subagent_model 區塊含註解與手寫的行內寫法，review 的 claude 為 opus，verify 沒有 claude
- **WHEN** 分別執行 speclink workflow-config subagent-model verify claude ""、speclink workflow-config subagent-model review claude opus
- **THEN** 兩者 exit code 0，stdout 分別為 `✓ subagent_model.verify.claude unchanged` 與 `✓ subagent_model.review.claude unchanged`；speclink/config.yaml 逐位元不變

#### Scenario: remote 寫入經版本檢查

- **WHEN** 於 remote 模式執行 speclink workflow-config subagent-model review claude opus 且無並行改寫
- **THEN** exit code 0；server 端 config 的 subagent_model.review.claude 為 opus；後續 speclink workflow-config show --json 讀回同一內容

#### Scenario: remote 版本衝突

- **WHEN** 於 remote 模式執行 speclink workflow-config subagent-model choices codex --stdin，而讀取後到寫入前另一個 client 已改寫 config
- **THEN** exit code 非 0，stderr 說明版本衝突並提示重跑指令；server 端 config 維持另一個 client 寫入的內容

## MODIFIED Requirements

### Requirement: workflow-config show 動詞
<!-- BEFORE: --json 的欄位到 review 為止，沒有子代理模型設定與內建清單 -->

CLI SHALL 提供 speclink workflow-config show（旗標 --json 與 --no-color，無 stdin），顯示 speclink/config.yaml 的正典內容：schema（專案預設 schema 名稱——未設定時標示為未設定與其預設 spec-driven）、政策五欄（locale、spec_locale、tdd、audit、worktree——未設定的欄位標示為未設定與其預設語意）、context（有無與行數）、rules（各 artifact 節的條數）與 review（預設清單模式與三個嚴重度清單的條數）。show SHALL 顯示正典值，SHALL NOT 應用環境變數的覆寫（有效值的三層解析屬 instructions payload 職責）。帶 --json 時 SHALL 輸出 camelCase payload：schema（字串，未設定為 null）、locale、specLocale（未設定為 null）、tdd、audit、worktree（布林）、context（字串或 null）、rules（artifact id 對規則字串陣列的物件）、review（物件：baseline 為字串 "extend" 或 "replace"，critical、warning、suggestion 為規則字串陣列；檔案未寫 review 區塊時為 baseline "extend" 與三個空陣列）、subagentModel（物件：review、verify 為 AI 工具名稱對模型名稱字串的物件，choices 為 AI 工具名稱對模型名稱字串陣列的物件；檔案未寫 subagent_model 區塊時為三個空物件）與 builtinSubagentModels（引擎內建的子代理模型清單：AI 工具名稱對模型名稱字串陣列的物件，恰為 {"claude":["sonnet","opus","haiku","fable"]}，不隨 config 內容改變）。人眼輸出 SHALL 於標題空行之後、locale 行之前，以與其他行相同的標籤欄寬印出 schema 行：值為 config 的 schema 名稱，未設定時為 unset (spec-driven)。人眼輸出 SHALL 於 rules 行之後、以與其他行相同的標籤欄寬印出 review 行：baseline 為 extend（含未設定）且三個清單皆空時值為 none，否則值為「<baseline>, critical <n>, warning <n>, suggestion <n>」。fs 模式讀取 speclink/config.yaml；remote 模式 SHALL 經既有連線讀取 server 端 config 文件，人眼與 --json 輸出形狀與 fs 模式一致。config 文件無法解析時 SHALL 沿用既有 fail-closed 行為：非零 exit code、stderr 指出檔案與解析原因。本動詞為周邊設定動詞（與 config 同類），SHALL NOT 進入命令執行層。成功 exit code 0、輸出至 stdout；--no-color 下無 ANSI 色彩。本次修訂為刻意變更：--json 於 review 之後新增 subagentModel 與 builtinSubagentModels 兩個欄位；人眼輸出不顯示 subagent_model，各行與既有 --json 欄位 SHALL 維持既有位元級輸出。

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

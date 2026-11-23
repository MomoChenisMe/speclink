## ADDED Requirements

### Requirement: 渲染 API 的宿主限制選項

skills.render 的 options SHALL 接受兩個選填的鍵，讓宿主說明自己的環境：`skills`（字串陣列，宿主提供的技能名稱，不帶 `speclink-` 前綴）與 `checkout`（布林，預設 true：使用這份技能的環境有沒有程式碼的 checkout 與 git 歷史）。任何一個有作用時，回傳字串 SHALL 在 frontmatter 與既有的前言之後、技能的其餘內容之前多一段，標題是 `## Harness limits`，段落以一行 `---` 結束；段落的文字 SHALL 是英文。兩個選項 SHALL 對每一個 target 與每一支技能可用。

給了 `skills` 時，這一段 SHALL 有一個項目：列出清單裡的每一支技能（以這個 target 引用技能的形式，例如 neutral 是 `speclink-discuss`；順序依 skills.list() 的順序，重複的名稱只列一次），並指示模型：清單外的技能在這裡不存在，不要載入、不要叫使用者執行；文件的步驟指到那種技能時，告訴使用者那一步由工程師在程式碼的 checkout 裡做。`checkout` 為 false 時，這一段 SHALL 有另一個項目，指示模型：這裡沒有程式碼的 checkout 與 git 歷史，跳過讀程式碼檔、搜尋程式碼與執行 git 的步驟，只用 speclink 的動詞查證事實，並在回答裡說明跳過了哪些步驟。兩個都給時，技能的項目 SHALL 在前。

`checkout` 為 false 時，`specInCheckout` 省略 SHALL 視同 false。兩個選項都沒有給、或 `checkout` 是 true 而沒有 `skills` 時，回傳字串 SHALL 與沒有這條需求時逐位元相同，SHALL NOT 含 `## Harness limits`。這兩個選項 SHALL NOT 改變技能原稿的內文：`## Harness limits` 之後的內容 SHALL 與以相同的 target、invocation、specInCheckout、specDir、toolName 而不帶這兩個選項渲染的結果相同。CLI 產生的技能檔 SHALL 不受影響。

下列情況 skills.render SHALL 同步拋出錯誤，SHALL NOT 回傳字串：`skills` 不是字串陣列；`skills` 是空陣列；`skills` 含不在 skills.list() 裡的名稱（訊息 SHALL 指出那個名稱）；`skills` 不含正在渲染的那一支技能；`checkout` 不是布林值（包含 null）；`checkout` 是 false 而 `specInCheckout` 明確是 true（訊息 SHALL 指出兩個選項互相矛盾）。

#### Scenario: 宣告可用的技能

- **WHEN** 執行 skills.render('propose', { target: 'neutral', invocation: 'tool-call', specInCheckout: false, skills: ['discuss', 'trace', 'propose', 'ingest', 'analyze', 'config'] })
- **THEN** 回傳字串在 `## Invocation` 段之後有 `## Harness limits` 段；段裡列出 `speclink-discuss`、`speclink-trace`、`speclink-propose`、`speclink-ingest`、`speclink-analyze`、`speclink-config` 六個名稱，沒有 `speclink-apply`；段裡有一句指示不要載入清單外的技能，也不要叫使用者執行

#### Scenario: 宣告沒有 checkout

- **WHEN** 執行 skills.render('discuss', { target: 'neutral', invocation: 'tool-call', checkout: false })
- **THEN** 回傳字串的 `## Harness limits` 段有一個項目說明沒有程式碼的 checkout 與 git 歷史，並指示跳過讀程式碼與執行 git 的步驟；其餘內容與以 specInCheckout 為 false、不帶 checkout 渲染的結果相同

#### Scenario: 兩個選項同時給

- **WHEN** 執行 skills.render('trace', { target: 'neutral', invocation: 'tool-call', checkout: false, skills: ['discuss', 'trace'] })
- **THEN** `## Harness limits` 段有兩個項目，可用技能的項目在前、沒有 checkout 的項目在後

#### Scenario: 不帶新選項時輸出不變

- **WHEN** 分別以 { target: 'neutral', invocation: 'tool-call', specInCheckout: false } 與同一組選項加上 checkout: true 渲染 propose
- **THEN** 兩個回傳字串相同，與引入這兩個選項之前逐位元相同，都不含 `## Harness limits`

#### Scenario: 其他 target 也可用

- **WHEN** 執行 skills.render('propose', { target: 'claude', skills: ['propose', 'discuss'] })
- **THEN** 回傳字串有 `## Harness limits` 段，技能以 Claude 的引用形式列出；段落之後的內容與不帶 skills 的渲染相同

#### Scenario: 選項的值不合法

- **WHEN** 分別以下表的選項呼叫 skills.render('propose', { target: 'neutral', … })
- **THEN** 每一種都拋出錯誤，沒有任何一種回傳字串

##### Example: 各種不合法的值

| 選項 | 錯誤的原因 |
| --- | --- |
| `skills: 'propose'` | 不是陣列 |
| `skills: []` | 空陣列 |
| `skills: ['propose', 'aply']` | `aply` 不是技能名稱，訊息含 `aply` |
| `skills: ['discuss']` | 清單不含正在渲染的 `propose` |
| `skills: ['propose', 7]` | 有一項不是字串 |
| `checkout: 'false'` | 不是布林 |
| `checkout: null` | 不是布林 |
| `checkout: false, specInCheckout: true` | 兩個選項互相矛盾 |

### Requirement: 渲染 API 拒絕不認得的選項鍵

skills.render 的 options 含不認得的鍵時 SHALL 同步拋出錯誤，SHALL NOT 忽略那個鍵而照常渲染。合法的鍵 SHALL 是 `target`、`invocation`、`specInCheckout`、`specDir`、`toolName`、`skills`、`checkout`。錯誤訊息 SHALL 含那個不認得的鍵，並 SHALL 列出全部合法的鍵。這是刻意的行為變更：在這條需求之前，不認得的鍵被忽略。只含合法鍵的呼叫 SHALL 不受影響。

#### Scenario: 打錯的鍵名被拒絕

- **WHEN** 執行 skills.render('propose', { target: 'neutral', skils: ['propose'] })
- **THEN** 呼叫拋出錯誤，訊息含 `skils`，並列出 `target`、`invocation`、`specInCheckout`、`specDir`、`toolName`、`skills`、`checkout`

#### Scenario: 合法的鍵照常渲染

- **WHEN** 執行 skills.render('propose', { target: 'neutral', invocation: 'tool-call', specInCheckout: false, specDir: 'speclink', toolName: 'speclink' })
- **THEN** 呼叫回傳字串，與引入這條需求之前逐位元相同

### Requirement: Agent 工具說明的動詞篩選

Node SDK SHALL 匯出函式 `agentToolDescription(options)`，回傳 `speclink` 工具的說明（字串，英文）。不帶參數、或 `options.exclude` 是空陣列時，回傳值 SHALL 等於 `agentTool.description`。`options.exclude` 是字串陣列時，每一項 SHALL 是動詞的文字，字與字之間一個空格（例如 `archive`、`review`、`discuss archive`）；說明裡的動詞清單 SHALL 不含「字序列以任何一項開頭」的末端動詞，其餘動詞的各行、它們的順序，以及清單之前的用法說明 SHALL 與 `agentTool.description` 相同。一項 SHALL 只比對完整的字：`archive` SHALL NOT 排除 `discuss archive`。全部動詞都被排除時 SHALL 回傳只有用法說明、動詞清單為空的字串。動詞清單 SHALL 由和 host.dispatch 相同的動詞定義產生。`agentTool` 的 `name`、`description`、`inputSchema` SHALL 不因這個函式改變。

下列情況 SHALL 同步拋出錯誤：`exclude` 不是字串陣列；`exclude` 的某一項比對不到清單裡的任何動詞（訊息 SHALL 指出那一項；需要 checkout 而本來就不在清單裡的動詞也算比對不到）；`options` 含 `exclude` 以外的鍵。

#### Scenario: 不帶參數等於常數的說明

- **WHEN** 執行 agentToolDescription() 與 agentToolDescription({ exclude: [] })
- **THEN** 兩個回傳值都等於 agentTool.description

#### Scenario: 排除一個頂層動詞

- **WHEN** 執行 agentToolDescription({ exclude: ['archive'] })
- **THEN** 回傳的動詞清單沒有頂層的 `archive` 那一行，仍然有 `discuss archive` 那一行；其餘各行與 agentTool.description 相同

#### Scenario: 排除一組動詞

- **WHEN** 執行 agentToolDescription({ exclude: ['review', 'verify'] })
- **THEN** 回傳的動詞清單沒有任何以 `review` 或 `verify` 開頭的動詞（`review show`、`review add-round`、`review stamp`、`review discard` 與 `verify` 的同名動詞都不在）

#### Scenario: 排除一個兩個字的動詞

- **WHEN** 執行 agentToolDescription({ exclude: ['discuss archive'] })
- **THEN** 回傳的動詞清單沒有 `discuss archive`，仍然有 `discuss list` 與頂層的 `archive`

#### Scenario: 留下的動詞都能執行

- **WHEN** 取出 agentToolDescription({ exclude: ['archive', 'review'] }) 的動詞清單
- **THEN** 清單的每一個動詞都能被 host.dispatch 解析，清單等於 agentTool.description 的動詞清單去掉被排除的那些

#### Scenario: 不合法的參數

- **WHEN** 分別執行 agentToolDescription({ exclude: 'archive' })、agentToolDescription({ exclude: ['archiv'] })、agentToolDescription({ exclude: ['drift'] }) 與 agentToolDescription({ include: ['list'] })
- **THEN** 四次都拋出錯誤；第二次的訊息含 `archiv`，第三次的訊息含 `drift`，第四次的訊息含 `include`

#### Scenario: 常數不變

- **WHEN** 比較引入這個函式前後的 agentTool
- **THEN** `name`、`description` 與 `inputSchema` 都相同

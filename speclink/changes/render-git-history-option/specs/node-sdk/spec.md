## MODIFIED Requirements

### Requirement: 渲染 API 的宿主限制選項

<!-- BEFORE: 只有 skills 與 checkout 兩個選項；checkout 把「有沒有程式碼」與「有沒有 git 歷史」合成一個開關，沒有「讀得到程式碼、不能用 git」的狀態 -->

skills.render 的 options SHALL 接受三個選填的鍵，讓宿主說明自己的環境：`skills`（字串陣列，宿主提供的技能名稱，不帶 `speclink-` 前綴）、`checkout`（布林，預設 true：使用這份技能的環境有沒有程式碼的 checkout 與 git 歷史）與 `gitHistory`（布林：這個環境能不能使用 git 與它的歷史；省略時與 `checkout` 的值相同，也就是 `checkout` 省略或為 true 時是 true、`checkout` 為 false 時是 false）。任何一個有作用時，回傳字串 SHALL 在 frontmatter 與既有的前言之後、技能的其餘內容之前多一段，標題是 `## Harness limits`，段落以一行 `---` 結束；段落的文字 SHALL 是英文。三個選項 SHALL 對每一個 target 與每一支技能可用。

給了 `skills` 時，這一段 SHALL 有一個項目：列出清單裡的每一支技能（以這個 target 引用技能的形式，例如 neutral 是 `speclink-discuss`；順序依 skills.list() 的順序，重複的名稱只列一次），並指示模型：清單外的技能在這裡不存在，不要載入、不要叫使用者執行；文件的步驟指到那種技能時，告訴使用者那一步由工程師在程式碼的 checkout 裡做。`checkout` 為 false 時，這一段 SHALL 有另一個項目，指示模型：這裡沒有程式碼的 checkout 與 git 歷史，跳過讀程式碼檔、搜尋程式碼與執行 git 的步驟，只用 speclink 的動詞查證事實，並在回答裡說明跳過了哪些步驟。`checkout` 不是 false 而 `gitHistory` 為 false 時，這一段 SHALL 有一個項目，指示模型：這裡有可以讀與搜尋的程式碼，但沒有 git 歷史、不能執行 git；跳過執行 git 的步驟；步驟寫了取得同一件事實的其他做法時改用那個做法；並在回答裡說明跳過了哪些步驟。這個項目 SHALL NOT 指示跳過讀程式碼檔或搜尋程式碼。`checkout` 為 false 時 SHALL NOT 有這個項目，因為沒有 checkout 的項目已經涵蓋 git；一次呼叫的這一段 SHALL 至多有這兩個項目的其中一個。技能的項目 SHALL 在其他項目之前。

`checkout` 為 false 時，`specInCheckout` 省略 SHALL 視同 false。`gitHistory` SHALL NOT 改變 `specInCheckout` 的預設。三個選項都沒有給、或沒有 `skills` 而 `checkout` 與 `gitHistory` 都不是 false 時，回傳字串 SHALL 與沒有這條需求時逐位元相同，SHALL NOT 含 `## Harness limits`。這三個選項 SHALL NOT 改變技能原稿的內文：`## Harness limits` 之後的內容 SHALL 與以相同的 target、invocation、specInCheckout、specDir、toolName 而不帶這三個選項渲染的結果相同。CLI 產生的技能檔 SHALL 不受影響。

下列情況 skills.render SHALL 同步拋出錯誤，SHALL NOT 回傳字串：`skills` 不是字串陣列；`skills` 是空陣列；`skills` 含不在 skills.list() 裡的名稱（訊息 SHALL 指出那個名稱）；`skills` 不含正在渲染的那一支技能；`checkout` 不是布林值（包含 null）；`checkout` 是 false 而 `specInCheckout` 明確是 true（訊息 SHALL 指出兩個選項互相矛盾）；`gitHistory` 不是布林值（包含 null）；`checkout` 是 false 而 `gitHistory` 明確是 true（訊息 SHALL 指出兩個選項互相矛盾）。

#### Scenario: 宣告可用的技能

- **WHEN** 執行 skills.render('propose', { target: 'neutral', invocation: 'tool-call', specInCheckout: false, skills: ['discuss', 'trace', 'propose', 'ingest', 'analyze', 'config'] })
- **THEN** 回傳字串在 `## Invocation` 段之後有 `## Harness limits` 段；段裡列出 `speclink-discuss`、`speclink-trace`、`speclink-propose`、`speclink-ingest`、`speclink-analyze`、`speclink-config` 六個名稱，沒有 `speclink-apply`；段裡有一句指示不要載入清單外的技能，也不要叫使用者執行

#### Scenario: 宣告沒有 checkout

- **WHEN** 執行 skills.render('discuss', { target: 'neutral', invocation: 'tool-call', checkout: false })
- **THEN** 回傳字串的 `## Harness limits` 段有一個項目說明沒有程式碼的 checkout 與 git 歷史，並指示跳過讀程式碼與執行 git 的步驟；其餘內容與以 specInCheckout 為 false、不帶 checkout 渲染的結果相同

#### Scenario: 兩個選項同時給

- **WHEN** 執行 skills.render('trace', { target: 'neutral', invocation: 'tool-call', checkout: false, skills: ['discuss', 'trace'] })
- **THEN** `## Harness limits` 段有兩個項目，可用技能的項目在前、沒有 checkout 的項目在後

#### Scenario: 宣告讀得到程式碼但不能用 git

- **WHEN** 執行 skills.render('trace', { target: 'neutral', invocation: 'tool-call', specInCheckout: false, gitHistory: false })
- **THEN** 回傳字串的 `## Harness limits` 段恰好有一個項目，說明可以讀與搜尋程式碼但不能執行 git，並指示跳過執行 git 的步驟；這個項目不含跳過讀程式碼檔的指示；其餘內容與不帶 gitHistory 渲染的結果相同

#### Scenario: 技能清單與 gitHistory 同時給

- **WHEN** 執行 skills.render('trace', { target: 'neutral', invocation: 'tool-call', specInCheckout: false, gitHistory: false, skills: ['discuss', 'trace'] })
- **THEN** `## Harness limits` 段有兩個項目，可用技能的項目在前、不能用 git 的項目在後

#### Scenario: 沒有 checkout 時 gitHistory 跟著是 false

- **WHEN** 分別以 { target: 'neutral', invocation: 'tool-call', checkout: false } 與同一組選項加上 gitHistory: false 渲染 discuss
- **THEN** 兩個回傳字串相同，`## Harness limits` 段只有「沒有 checkout」那一個項目，與引入 gitHistory 之前 checkout: false 的輸出逐位元相同

#### Scenario: gitHistory 為 true 時輸出不變

- **WHEN** 分別以 { target: 'neutral', invocation: 'tool-call', specInCheckout: false } 與同一組選項加上 gitHistory: true 渲染 trace
- **THEN** 兩個回傳字串相同，都不含 `## Harness limits`

#### Scenario: gitHistory 不改變 specInCheckout 的預設

- **WHEN** 分別以 { target: 'neutral', gitHistory: false } 與 { target: 'neutral', specInCheckout: true, gitHistory: false } 渲染 discuss
- **THEN** 兩個回傳字串相同

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
| `gitHistory: 'false'` | 不是布林 |
| `gitHistory: null` | 不是布林 |
| `checkout: false, gitHistory: true` | 兩個選項互相矛盾 |

### Requirement: 渲染 API 拒絕不認得的選項鍵

<!-- BEFORE: 合法的鍵是 target、invocation、specInCheckout、specDir、toolName、skills、checkout 七個 -->

skills.render 的 options 含不認得的鍵時 SHALL 同步拋出錯誤，SHALL NOT 忽略那個鍵而照常渲染。合法的鍵 SHALL 是 `target`、`invocation`、`specInCheckout`、`specDir`、`toolName`、`skills`、`checkout`、`gitHistory`。錯誤訊息 SHALL 含那個不認得的鍵，並 SHALL 列出全部合法的鍵。這是刻意的行為變更：在這條需求之前，不認得的鍵被忽略。只含合法鍵的呼叫 SHALL 不受影響。

#### Scenario: 打錯的鍵名被拒絕

- **WHEN** 執行 skills.render('propose', { target: 'neutral', skils: ['propose'] })
- **THEN** 呼叫拋出錯誤，訊息含 `skils`，並列出 `target`、`invocation`、`specInCheckout`、`specDir`、`toolName`、`skills`、`checkout`、`gitHistory`

#### Scenario: 合法的鍵照常渲染

- **WHEN** 執行 skills.render('propose', { target: 'neutral', invocation: 'tool-call', specInCheckout: false, specDir: 'speclink', toolName: 'speclink' })
- **THEN** 呼叫回傳字串，與引入這條需求之前逐位元相同

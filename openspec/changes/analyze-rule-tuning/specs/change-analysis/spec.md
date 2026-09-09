## Purpose

`speclink analyze` 對單一 change 的四面向交叉檢查（Coverage／Consistency／Ambiguity／Gaps）：每個面向的前置 artifact 與跳過語意、每條規則的觸發條件、嚴重度、訊息與訊息 key，以及報告的凍結形狀。本 capability 只定義「哪些情況產生 finding」；artifacts 的結構驗證歸 spec-validation，delta 與正典的合併守門歸 archive-merge，漂移歸 drift-computation。

## ADDED Requirements

### Requirement: 面向的前置 artifact 與報告形狀

`speclink analyze <change>` SHALL 對四個面向各給一個狀態：前置 artifact 不足時為 `Skipped (insufficient artifacts)`，零 finding 時為 `Clean`，否則為 `N issue(s) found`。前置條件 SHALL 為：Coverage 需要 proposal.md 且 specs 目錄與 tasks.md 至少一者存在；Consistency 需要 design.md 與 tasks.md；Ambiguity 需要至少一個 delta spec；Gaps 只要任一 artifact 存在。artifact 存在與否 SHALL 以檔案存在判定，空檔也算存在。每筆 finding SHALL 帶 `id`（面向前綴 COV／CON／AMB／GAP 加面向內流水號）、`dimension`、`severity`（Critical／Warning／Suggestion）、`location`、`summary`、`recommendation`、`summary_msg` 與 `recommendation_msg`（各含 `key` 與 `params`）。findings SHALL 依 Coverage、Consistency、Ambiguity、Gaps 的順序排列。`--json` 輸出 SHALL 維持既有的 snake_case 欄位（`change_id`、`finding_count`、`artifacts_analyzed`、`artifacts_missing`、`summary_msg`、`recommendation_msg`）——這是 verb-contract 凍結的既有形狀，本 capability 不改名。人眼輸出 SHALL 逐面向列狀態與 finding 數，`--no-color` 下去除 ANSI 色彩、內容不變。

#### Scenario: 缺 design 時 Consistency 跳過

- **WHEN** change 有 proposal.md、tasks.md 與 1 個 delta spec，沒有 design.md
- **THEN** Consistency 的狀態為 `Skipped (insufficient artifacts)`、`finding_count` 為 0，其餘三個面向照常判定，`artifacts_missing` 含 `design`

#### Scenario: 空的 tasks.md 仍算存在

- **WHEN** tasks.md 存在但內容為空字串（0 bytes）
- **THEN** Consistency 與 Coverage 的前置條件視 tasks 為存在，不因空檔而跳過；Consistency 對 design.md 的每個 `###` 標題各報 1 筆 `conDesignNotInTasks`

### Requirement: Coverage 規則

Coverage SHALL 產生兩種 finding。`covMissingSpec`（Critical）：proposal.md 的 Capabilities 區段（`## Capabilities`，或其下的 `### New Capabilities`／`### Modified Capabilities`）每一行第一個不含空白的反引號 token 視為 capability 名，該 capability 的 delta spec 檔不存在時報一筆，`location` 為 `proposal.md → Capabilities`。`covMissingTask`（Warning）：每條 delta 需求名（trim 後、大小寫不敏感）SHALL 以連續子字串出現在至少一個 checkbox 任務的描述中，否則報一筆，`location` 為該 delta spec 的相對路徑；tasks.md 不存在時 SHALL NOT 報。REMOVED 區塊的需求 SHALL 同樣受此規則約束——移除功能需要有任務去拆。

#### Scenario: 需求名以子字串命中任務描述

- **WHEN** delta 有需求 `CSV Export`，tasks.md 有 `- [ ] 1.1 Implement CSV exporter`
- **THEN** 不報 `covMissingTask`

#### Scenario: 需求名只出現在群組標題不算命中

- **WHEN** 需求名 `CSV Export` 只出現在 tasks.md 的 `## 2. CSV Export` 標題，沒有任何 checkbox 描述含它
- **THEN** 報一筆 Warning，`summary` 為 `Requirement 'CSV Export' has no matching task`，`summary_msg.key` 為 `covMissingTask.summary`

##### Example: 命中判定

| 需求名 | 任務描述 | 結果 |
| --- | --- | --- |
| `CSV Export` | `Implement csv export` | 命中（大小寫不敏感） |
| `CSV Export` | `Implement csv-export` | 未命中（連字號打斷連續子字串） |
| `CSV Export` | `Export CSV` | 未命中（順序不同） |

### Requirement: Consistency 的 design 標題引用判定

Consistency SHALL 對 design.md 每個 `###` 標題判定「tasks.md 是否引用了它」，未引用時報一筆 `conDesignNotInTasks`（Warning），`location` 為 `design.md`，`summary` 為 `Design topic '<小寫整串標題>' not referenced in tasks`，`summary_msg.params.keyword` 為小寫整串標題。判定 SHALL 先把標題拆成「編號」與「本文」：編號只認三種樣式——`D<數字>`、`決策<數字或一到十的中文數字>`、`Decision <數字>`（大小寫不敏感），後接零或多個空白、零或一個全形或半形冒號、零或多個空白；其餘為本文。tasks.md 全文（大小寫不敏感）含本文，或含編號且編號之後的下一個字元不是 ASCII 數字，任一成立 SHALL 視為已引用。標題沒有編號，或本文為空，SHALL 退回整串標題的子字串比對。tasks.md 的任何一行（群組標題、checkbox、散文）SHALL 都算數。

#### Scenario: 本文命中即算引用

- **WHEN** design.md 有 `### 決策一：整個移除 listDepthLimit 擴充`，tasks.md 有 `## 2. 實作：整個移除 listDepthLimit 擴充`
- **THEN** 不報 `conDesignNotInTasks`

#### Scenario: 編號命中即算引用

- **WHEN** design.md 有 `### D1 違規清單與聚合錯誤形狀`，tasks.md 只有 `- [ ] 1.1 彙整違規（design D1）`
- **THEN** 不報 `conDesignNotInTasks`

#### Scenario: 編號後接數字不算命中

- **WHEN** design.md 有 `### D1 違規清單與聚合錯誤形狀`，tasks.md 只出現 `D12`
- **THEN** 報一筆 Warning，`summary` 為 `Design topic 'd1 違規清單與聚合錯誤形狀' not referenced in tasks`

#### Scenario: 無編號標題維持整串比對

- **WHEN** design.md 有 `### 索引 JSON 的形狀與推導規則`，tasks.md 只含 `索引 JSON 的形狀`
- **THEN** 報一筆 Warning；tasks.md 含整串 `索引 JSON 的形狀與推導規則` 時不報

##### Example: 編號拆解

| design 標題 | 編號 | 本文 |
| --- | --- | --- |
| `決策一：整個移除 listDepthLimit 擴充` | `決策一` | `整個移除 listDepthLimit 擴充` |
| `D3: 搜尋列元件化` | `D3` | `搜尋列元件化` |
| `Decision 2 device code 分權責` | `Decision 2` | `device code 分權責` |
| `索引 JSON 的形狀與推導規則` | （無） | 整串 |
| `D4` | `D4` | （空，退回整串比對 `d4`） |

### Requirement: Ambiguity 的 scenario 缺席與 REMOVED 需求檢查

Ambiguity SHALL 對 ADDED 與 MODIFIED 區塊的每條需求檢查至少有一個 `#### Scenario:`，沒有時報 `ambNoScenario`（Warning），`summary` 為 `Requirement '<name>' has no scenarios`。REMOVED 區塊的需求 SHALL NOT 受 scenario 檢查；改為檢查需求本文是否各有一行 trim 後以 `**Reason**` 與 `**Migration**` 開頭，缺任一者報 `ambRemovedNoReason`（Warning）：`summary` 為 `REMOVED requirement '<name>' has no **Reason**`、`... has no **Migration**` 或 `... has no **Reason** and **Migration**`；`recommendation` 為 `Add **Reason**: and **Migration**: lines under '<name>'`；`summary_msg.key` 為 `ambRemovedNoReason.summary`，`params` 含 `req`（需求名）與 `missing`（`Reason`、`Migration` 或 `Reason and Migration`）；`recommendation_msg.key` 為 `ambRemovedNoReason.recommendation`，`params` 同。這兩種 finding SHALL 共用 AMB 流水號，並落在每個 delta spec 檔的第一段（no-scenario 段），先於該檔的 abstract-scenario 與 weak-language finding。

#### Scenario: REMOVED 需求齊備時零 finding

- **WHEN** delta 的 `## REMOVED Requirements` 下有需求 `Legacy export`，本文只有 `**Reason**: Replaced by v2` 與 `**Migration**: Use /api/v2/export` 兩行，沒有 scenario
- **THEN** Ambiguity 對這條需求不報任何 finding

#### Scenario: REMOVED 需求缺 Migration

- **WHEN** 同上，但只有 `**Reason**` 行
- **THEN** 報一筆 Warning，`summary` 為 `REMOVED requirement 'Legacy export' has no **Migration**`，`summary_msg.params.missing` 為 `Migration`

#### Scenario: ADDED 需求無 scenario 仍報

- **WHEN** `## ADDED Requirements` 下的需求 `Fresh` 沒有任何 `#### Scenario:`
- **THEN** 報一筆 Warning，`summary` 為 `Requirement 'Fresh' has no scenarios`，`summary_msg.key` 為 `ambNoScenario.summary`

### Requirement: Ambiguity 的具體值判定

每個 scenario 若沒有 `##### Example:`，且內文（scenario 標題之後、下一個標題之前的非標題行）沒有任何「具體值字元」，SHALL 報 `ambAbstractScenario`（Suggestion），`summary` 為 `Scenario '<name>' has no concrete examples`。具體值字元 SHALL 為：ASCII 數字 `0`-`9`、反引號、半形雙引號、全形引號 `「」『』`、全形數字 `０`-`９`。中文數字（一、二、三…）與單引號 SHALL NOT 算具體值。

#### Scenario: 全形引號內的字串算具體值

- **WHEN** scenario 內文為 `- **THEN** 卡片顯示「已封存」標籤`，沒有 Example
- **THEN** 不報 `ambAbstractScenario`

#### Scenario: 純敘述仍判為抽象

- **WHEN** scenario 內文為 `- **THEN** 顯示成功訊息`，沒有 Example
- **THEN** 報一筆 Suggestion，`summary` 為 `Scenario '<name>' has no concrete examples`

##### Example: 具體值判定

| scenario 內文 | 結果 |
| --- | --- |
| `回傳 3 筆結果` | 具體（ASCII 數字） |
| `顯示「已封存」` | 具體（全形引號） |
| `顯示『完成』` | 具體（全形引號） |
| `第１頁` | 具體（全形數字） |
| `回傳三筆結果` | 抽象（中文數字不算） |
| `顯示 '完成'` | 抽象（單引號不算） |

### Requirement: Ambiguity 的弱語氣詞偵測——英文樣式 should、may、might、consider、possibly 以字邊界比對

每一非標題行 SHALL 至多報一筆 `ambWeakLanguage`（Suggestion），`location` 為 `<delta 相對路徑>:<行號>`，`summary` 為 `Vague language '<pattern>' found`。檢查順序 SHALL 為：五個英文樣式（依上述順序）、再 `TBD`／`TODO`／`???`／`TKTK`、再 CJK 樣式。五個英文樣式 SHALL 在整行轉小寫後以字邊界比對：命中子字串的前後字元都不是 ASCII 字母才算。`TBD`／`TODO`／`TKTK` SHALL 維持大小寫不敏感的子字串比對，`???` 維持原樣子字串比對。CJK 樣式 SHALL 維持子字串比對，「不可能」豁免照舊（該三字去除後仍含「可能」才報）。以 `#` 開頭的標題行 SHALL NOT 掃描。

#### Scenario: 字邊界擋住 shoulder

- **WHEN** spec 行為 `The shoulder strap SHALL lock`
- **THEN** 不報 `ambWeakLanguage`

#### Scenario: 獨立的 should 仍命中

- **WHEN** spec 行為 `The strap should lock`
- **THEN** 報一筆 Suggestion，`summary` 為 `Vague language 'should' found`

#### Scenario: CJK 樣式維持子字串——可能、應該、考慮

- **WHEN** spec 行為 `系統盡可能鎖定`
- **THEN** 報一筆 Suggestion，`summary` 為 `Vague language '可能' found`；行為 `這不可能發生` 時不報

##### Example: 字邊界判定

| spec 行 | 結果 |
| --- | --- |
| `it should lock` | `should` |
| `Should lock` | `should`（先轉小寫） |
| `the shoulder strap` | 不報 |
| `a considerable delay` | 不報 |
| `the mayor` | 不報 |
| `outbdoor` | `TBD`（子字串比對維持） |

### Requirement: Gaps 規則

Gaps SHALL 產生四種 finding。`gapRestale`（Suggestion）：change 反映的討論被重新結論時，每個討論 slug 報一筆，`location` 為 `change meta`。`gapNoProposal`（Critical）：有 delta spec 但 proposal.md 檔不存在，`location` 為 `change directory`。`gapNoMainSpec`（Warning）：MODIFIED 區塊的需求所屬 capability 沒有正典規格，每個 capability 只報一筆。`gapModifiedNotFound`（Warning）：正典規格存在，但找不到 `### Requirement: <name>` 逐字相符的需求。REMOVED 與 RENAMED 的目標存在與否 SHALL NOT 由 Gaps 判定——那是 archive-merge 的合併守門範圍。

#### Scenario: MODIFIED 需求不在正典

- **WHEN** delta 的 `## MODIFIED Requirements` 下有需求 `R9`，正典 `specs/auth/spec.md` 存在但沒有 `### Requirement: R9`
- **THEN** 報一筆 Warning，`summary` 為 `MODIFIED requirement 'R9' not found in main spec`，`recommendation_msg.params.spec` 為 `auth`

#### Scenario: 無正典時每 capability 一筆

- **WHEN** delta `specs/auth/spec.md` 的 MODIFIED 區塊有 2 條需求，正典 `specs/auth/spec.md` 不存在
- **THEN** 只報 1 筆 `gapNoMainSpec`，`summary` 為 `MODIFIED requirements reference capability 'auth' but no main spec found`

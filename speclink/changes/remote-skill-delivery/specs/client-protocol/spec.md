## ADDED Requirements

### Requirement: 握手宣告技能版本

握手回應的 capabilities SHALL 增列選填的 `skills` 物件，含 `version`（字串，Server 的技能範本版號，形式與技能檔 frontmatter 的版本欄位相同）。這個欄位缺席時用戶端 SHALL 把 Server 視為不提供技能。既有的 `engineVersion` 欄位的意義不變。不認得 `skills` 的舊版用戶端 SHALL 照常完成握手。

#### Scenario: 宣告的解析

- **WHEN** 用戶端解析 capabilities 含 `"skills":{"version":"v1.58.0"}` 的握手回應
- **THEN** 得到 Server 的技能版號 v1.58.0

#### Scenario: 舊 server 沒有宣告

- **WHEN** 用戶端解析沒有 skills 欄位的握手回應
- **THEN** 握手成功，技能版號讀作沒有

#### Scenario: 舊用戶端解析新回應

- **WHEN** 以不認得 skills 的型別解析含該欄位的握手回應
- **THEN** 解析成功，其餘欄位的值不變

### Requirement: 技能清單的請求與回應

Client Protocol SHALL 定義技能清單的請求：對專案底下 skills 的 POST，它是唯讀的請求。請求 payload 為 camelCase：`targets`（陣列，至少一項、至多 32 項；每項含 `tool` 字串，以及選填的 `custom` 物件——含 `invocation`，值為 `cli` 或 `tool-call`；帶 `custom` 的是自訂工具，它的 `tool` SHALL 符合工具描述子的名稱規則（kebab-case，且不是內建工具的名稱）；不帶的 SHALL 是內建工具的名稱）與選填的 `contents`（布林，缺席讀作 false）。回應 payload：`version`（字串，技能範本版號）與 `targets`（陣列，順序與請求相同；每項含 `tool` 與 `skills` 陣列，skills 依名稱排序，每項含 `name` 字串、`digest` 字串，以及只在請求全文時出現的 `content` 字串）。`digest` SHALL 是內容把換行正規化成 LF 之後的 SHA-256，形式為 `sha256:` 加十六進位。回應 SHALL NOT 含檔案路徑：技能寫到哪個資料夾由用戶端依自己的工具設定決定。`targets` 為空或超過 32 項、內建工具的名稱不認得、自訂工具的名稱不合規則、`invocation` 不在值域內時 SHALL 以 invalid_argument 拒絕。這些型別 SHALL 在 protocol 的單一定義處宣告。

#### Scenario: 只要摘要

- **WHEN** 送出 `{"targets":[{"tool":"claude"}]}`
- **THEN** 回應的每支技能有 name 與 digest，沒有 content

#### Scenario: 要全文

- **WHEN** 送出 `{"targets":[{"tool":"claude"}],"contents":true}`
- **THEN** 每支技能多一個 content，其換行正規化後的 SHA-256 等於同一項的 digest

#### Scenario: 自訂工具

- **WHEN** 送出 `{"targets":[{"tool":"my-agent","custom":{"invocation":"tool-call"}}],"contents":true}`
- **THEN** 回應的 targets 有一項 tool 為 my-agent，內容是中性的 tool-call 渲染

#### Scenario: 不認得的內建工具

- **WHEN** 送出 `{"targets":[{"tool":"emacs"}]}`
- **THEN** 回應的 reason 為 invalid_argument

#### Scenario: 空的 targets

- **WHEN** 送出 `{"targets":[]}`
- **THEN** 回應的 reason 為 invalid_argument

#### Scenario: targets 超過上限

- **WHEN** 送出含 33 項的 targets
- **THEN** 回應的 reason 為 invalid_argument

#### Scenario: 自訂工具的名稱不合規則

- **WHEN** 送出 `{"targets":[{"tool":"Not A Name","custom":{"invocation":"cli"}}]}`
- **THEN** 回應的 reason 為 invalid_argument

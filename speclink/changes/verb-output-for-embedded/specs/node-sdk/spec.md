## ADDED Requirements

### Requirement: Agent 工具的說明指出 path 是儲存的鍵

agentTool.description SHALL 含一段給模型看的說明：動詞結果裡的 `path` 是文件在儲存裡的鍵，不是可以打開的檔案；讀文件的內容 SHALL 改用讀取的動詞，說明 SHALL 點名 `artifact cat`、`show` 與 `discuss show`。這段說明 SHALL 是英文，與 description 的其餘部分相同。agentTool 的 `name`、`inputSchema` 與動詞清單 SHALL 不因此改變。

#### Scenario: 說明提到 path 不是檔案

- **WHEN** 檢視 agentTool.description
- **THEN** 它含一句說明 `path` 是儲存的鍵而不是檔案，並點名 `artifact cat`、`show`、`discuss show` 三個讀取的動詞

#### Scenario: 其他欄位不變

- **WHEN** 比較引入本需求前後的 agentTool
- **THEN** `name` 與 `inputSchema` 深度相等；description 的動詞清單所列的動詞集合相同

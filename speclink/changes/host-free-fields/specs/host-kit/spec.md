## ADDED Requirements

### Requirement: 呼叫入口時附上自由欄位

宿主套件的兩個門 SHALL 各接受宿主在這次呼叫附上的一組自由欄位。這組欄位 SHALL 只在這次呼叫會建立變更或討論時生效（建立變更、建立討論、從討論轉出變更）：它 SHALL 併入該命令要寫入的欄位，同名時宿主的值 SHALL 蓋過命令或請求自己帶的值，並 SHALL 與建立在同一次提交。其他命令 SHALL 忽略它，SHALL NOT 因此失敗，也 SHALL NOT 產生任何寫入。合併 SHALL 先於執行前掛鉤，掛鉤收到的參數 SHALL 是最後要寫入的欄位。合併後不合自由欄位的規則時，這次呼叫 SHALL 以 invalid_argv 失敗且儲存不變。動詞文字、stdin 與 HTTP 請求的內容 SHALL NOT 能移除或改寫宿主附上的欄位。宿主 SHALL NOT 能藉這個輸入改變命令的其他部分。

#### Scenario: 建立變更時附上來源

- **WHEN** 宿主以附上的欄位 source=portal 經 Agent 工具的門執行建立變更 demo
- **THEN** demo 建立時就有 source=portal；儲存收到正好一次提交

#### Scenario: 宿主的值蓋過請求的值

- **WHEN** HTTP 請求的建立變更 payload 帶 fields source=cli，宿主在這次呼叫附上 source=portal
- **THEN** 建立的變更的 source 是 portal

#### Scenario: 其他命令忽略

- **WHEN** 宿主以同一組附上的欄位執行列出變更與勾選任務
- **THEN** 兩者照常成功；沒有任何變更的自由欄位被改動

#### Scenario: 掛鉤看到最後的欄位

- **WHEN** 宿主附上 source=portal 執行建立變更，並設有執行前掛鉤
- **THEN** 掛鉤收到的參數的 fields 含 source=portal

#### Scenario: 附上的欄位不合規則

- **WHEN** 宿主附上鍵為 Bad 的欄位執行建立變更
- **THEN** 呼叫以 invalid_argv 失敗；變更沒有被建立

#### Scenario: 握手宣告支援

- **WHEN** 對宿主套件的 HTTP 入口送出握手
- **THEN** capabilities 的 fields 為 true

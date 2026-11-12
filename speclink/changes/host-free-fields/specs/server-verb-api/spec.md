## ADDED Requirements

### Requirement: 自由欄位的寫入端點

server SHALL 提供變更與討論的自由欄位寫入端點（PUT changes/{name}/fields 與 PUT discussions/{slug}/fields），以引擎的欄位編輯命令執行，editor 以上角色可用。規則不符（鍵、值、數量、同一個鍵同時設定與移除）SHALL 以 invalid_argument 回應且儲存不變；變更或討論不存在、討論已封存 SHALL 以 not_found 回應。沒有任何變動時 SHALL 成功回應目前的欄位，範圍版本 SHALL 不前進。建立變更、建立討論與轉出變更的端點 SHALL 把請求的 fields 交給引擎，與建立在同一次提交。變更清單、單筆變更、已封存清單與討論的讀取端點 SHALL 回填 fields，且 SHALL NOT 為此多發儲存呼叫。

#### Scenario: editor 設定欄位

- **WHEN** editor 對 changes/demo/fields 送出 `{"set":{"ticket":"PROJ-1"}}`
- **THEN** 回應是 200，fields 含 ticket；範圍版本前進一次

#### Scenario: reader 被拒

- **WHEN** reader 送出同一個請求
- **THEN** 回應是 403；儲存不變

#### Scenario: 規則不符

- **WHEN** 送出 `{"set":{"Bad":"1"}}`
- **THEN** 回應的 reason 為 invalid_argument；儲存不變

#### Scenario: 沒有變動不前進版本

- **WHEN** 對沒有欄位 ghost 的 demo 送出 `{"unset":["ghost"]}`
- **THEN** 回應是 200，範圍版本與請求前相同

#### Scenario: 建立與欄位同一次提交

- **WHEN** 送出含 fields 的建立變更請求
- **THEN** 儲存收到正好一次提交，其中 metadata 的內容已含 fields 區塊

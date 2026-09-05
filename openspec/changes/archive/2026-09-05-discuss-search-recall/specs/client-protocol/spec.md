## ADDED Requirements

### Requirement: 討論搜尋回應 payload

protocol SHALL 以 Rust 型別定義討論搜尋回應（JSON Schema 為匯出）：頂層 hits 陣列，每筆含與討論資訊 payload 相同的欄位（slug、topic、status、rounds、created、createdBy 選填、kind 選填、promotedTo、concluded 選填、path、archived）加 matches 陣列；每個 match 含 kind、where、text 三個字串欄位，kind 值為 topic、slug、ruled-out、decision、rejected、deferred 之一，where 值為 frontmatter、round-N 或 conclusion。欄位一律 camelCase。typed client SHALL 提供搜尋方法，以關鍵字清單組成空白分隔的 q 呼叫 GET /discussions/search，回應反序列化為 typed 型別，SHALL NOT 走 raw JSON 旁路。既有討論資訊 payload 與清單、詳情回應的欄位 SHALL 逐位元不變。

#### Scenario: typed client 讀取搜尋回應

- **WHEN** typed client 以關鍵字 golden 與 sse 呼叫搜尋方法，server 回一筆命中（matches 含 kind 為 deferred、where 為 conclusion 的項目）
- **THEN** 請求路徑為 /discussions/search 且 q 為 `golden sse`；回應反序列化成功，該筆的 slug、archived 與 matches 三欄位值與 JSON 一致，缺席的 createdBy 與 kind 為 None

#### Scenario: 選填欄位缺席時仍可反序列化

- **WHEN** server 回應的命中項目缺 createdBy、kind、concluded 三個鍵，promotedTo 為空陣列
- **THEN** typed client 反序列化成功，三個選填欄位為 None、promotedTo 為空清單，不報錯

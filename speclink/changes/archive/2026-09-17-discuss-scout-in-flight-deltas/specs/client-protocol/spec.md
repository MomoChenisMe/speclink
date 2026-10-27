## ADDED Requirements

### Requirement: 變更清單的 delta capability 欄位

ChangeSummary SHALL 增選填欄位 deltaCapabilities（camelCase、serde default、空清單即省略），值為該變更 delta 規格的 capability 名稱，升冪，語意與 ChangeStatus 既有的 deltaCapabilities 一致。舊 server 不送此欄位時，client SHALL 以空清單容錯，SHALL NOT 偽造值。

#### Scenario: delta capability 欄位序列化與缺席省略

- **WHEN** 序列化一筆 deltaCapabilities 為 ["client-protocol"] 的 ChangeSummary，再序列化一筆 deltaCapabilities 為空的 ChangeSummary，並反序列化一筆無 deltaCapabilities 鍵的舊 payload
- **THEN** 前者的 JSON 含 "deltaCapabilities": ["client-protocol"]；後者的 JSON 無 deltaCapabilities 鍵；舊 payload 反序列化成功且 deltaCapabilities 為空清單

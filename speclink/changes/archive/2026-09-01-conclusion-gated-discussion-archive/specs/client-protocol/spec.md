## ADDED Requirements

### Requirement: 討論資訊 payload 增選填 concluded 欄位

DiscussionInfo SHALL 增選填 concluded 欄位（camelCase、serde default、缺席即未知），值為該討論的 Conclusion 段是否已寫入內文（scaffold 佔位註解不算內文）。此欄位由 server 於 route 邊緣組裝；引擎側 DiscussionInfo 結構 SHALL NOT 因此欄位改動（引擎明訂結論判定不進列表結構以保 CLI JSON 逐位元不變）。序列化時缺席值 SHALL 省略鍵；組裝端 SHALL 對每筆討論恆填 true 或 false。舊 server 不送時 client SHALL 視為未知，SHALL NOT 把缺席當成 false、SHALL NOT 據此推論結論狀態。

#### Scenario: concluded 序列化與缺席容錯

- **WHEN** 序列化一筆 concluded 為 true 與一筆 concluded 為 false 的 DiscussionInfo，再反序列化一筆無 concluded 鍵的舊 payload
- **THEN** 前兩者 JSON 分別含 concluded: true 與 concluded: false；後者反序列化不失敗且值為未知（缺席），再序列化時無 concluded 鍵

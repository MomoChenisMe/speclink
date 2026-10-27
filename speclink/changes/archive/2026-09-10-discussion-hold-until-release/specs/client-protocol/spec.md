## ADDED Requirements

### Requirement: 討論資訊 payload 增選填 hold 欄位

DiscussionInfo SHALL 增選填 hold 欄位（camelCase、serde default、缺席即未知），值為該討論 frontmatter 是否帶 `hold: true` 行（結論保留在途、還欠尚未建立的變更）。此欄位由 server 於 route 邊緣自引擎的 frontmatter 解析結果組裝；引擎側 DiscussionInfo 的 JSON 投影 SHALL NOT 因此欄位改動（CLI 的 discuss list --json 逐位元不變）。序列化時缺席值 SHALL 省略鍵；組裝端 SHALL 對每筆討論恆填 true 或 false。舊 server 不送時 client SHALL 視為未知，SHALL NOT 把缺席當成 true、分區判準 SHALL 沿缺席前的既有行為。

#### Scenario: hold 序列化與缺席容錯

- **WHEN** 序列化一筆 hold 為 true 與一筆 hold 為 false 的 DiscussionInfo，再反序列化一筆無 hold 鍵的舊 payload
- **THEN** 前兩者 JSON 分別含 hold: true 與 hold: false；後者反序列化不失敗且值為未知（缺席），再序列化時無 hold 鍵

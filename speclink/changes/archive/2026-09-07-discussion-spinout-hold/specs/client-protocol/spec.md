## ADDED Requirements

### Requirement: 討論結論請求與回應攜帶保留旗標

討論結論端點的請求型別 SHALL 增選填 hold 欄位（camelCase 布林、serde default、缺席即 false、值為 false 時不序列化）——true 表示結論後記錄保留在途、不隨轉出變更封存。回應型別 SHALL 增 held 欄位（camelCase 布林、serde default、值為 false 時不序列化）——本次寫入後記錄是否帶保留旗標；缺席與 false 的讀取結果相同，與舊 server 不回報此事實時一致，不需哨兵欄位。既有 restaleFlagged 與 autoArchived 欄位的語意 SHALL 維持不變。

#### Scenario: 請求的 hold 缺席即 false 且 false 不出鍵

- **WHEN** 以只含 content 的 JSON 反序列化討論結論請求；再序列化一筆 hold 為 false 與一筆 hold 為 true 的請求
- **THEN** 前者 hold 讀為 false；後兩者的 JSON 分別無 hold 鍵、含 hold: true

#### Scenario: 回應的 held 缺席容錯與 true 出鍵

- **WHEN** 以空物件反序列化討論結論回應；再序列化一筆 held 為 true 的回應
- **THEN** 前者反序列化成功且 held 為 false、restaleFlagged 為空清單；後者 JSON 含 held: true

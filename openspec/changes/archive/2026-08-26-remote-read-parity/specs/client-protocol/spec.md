## ADDED Requirements

### Requirement: 變更清單的建立者與來源討論欄位

ChangeSummary SHALL 增三個選填欄位（皆 camelCase、serde default）：createdBy SHALL 為 meta 的 created_by（缺席即省略）；created SHALL 為 meta 的 created 日期（缺席即省略）；fromDiscussions SHALL 為 meta 的 from_discussion 鏈（空清單即省略）。舊 server 不送這些欄位時，client 清單消費端 SHALL 以缺席容錯（頭像圓標與來源討論標記不顯示），SHALL NOT 偽造預設值。

#### Scenario: 清單欄位序列化與缺席省略

- **WHEN** 序列化一筆 meta 含 created_by、created 與 from_discussion 的 ChangeSummary，再序列化一筆三者皆缺的
- **THEN** 前者的 JSON 含 createdBy、created 與 fromDiscussions 三鍵；後者三鍵皆不出現；反序列化無此三鍵的舊 payload 不失敗

### Requirement: 單 change 讀取回應的 meta 歸屬欄位

ChangeStatus SHALL 增四個選填欄位（皆 camelCase、serde default、缺席即省略）：createdBy、createdWith、startedAt、startedBy，值各為 change meta 的 created_by、created_with、started_at、started_by。舊 server 不送時 client 的詳情呈現 SHALL 維持對應列缺席，SHALL NOT 偽造預設值。

#### Scenario: 歸屬欄位序列化與缺席省略

- **WHEN** 序列化一筆 meta 四欄俱全的 ChangeStatus，再序列化一筆四欄皆缺的
- **THEN** 前者 JSON 含 createdBy、createdWith、startedAt、startedBy 四鍵；後者四鍵皆不出現；反序列化無此四鍵的舊 payload 不失敗

### Requirement: 討論資訊 payload 增選填 promotedTo 欄位

DiscussionInfo SHALL 增選填 promotedTo 欄位（camelCase、serde default、空清單即省略），值為該討論已轉出／已併入的 change 名稱清單，順序沿 frontmatter promoted_to 的累加順序。此欄位由 server 於 route 邊緣組裝；引擎側 DiscussionInfo 結構 SHALL NOT 因此欄位改動（引擎明訂 promoted_to 不進列表結構以保 CLI JSON 逐位元不變）。舊 server 不送時 client SHALL 以空清單容錯，SHALL NOT 據此推論討論未轉出以外的狀態。

#### Scenario: promotedTo 序列化與缺席容錯

- **WHEN** 序列化一筆 promotedTo 含兩個 change 名的 DiscussionInfo，再序列化一筆空清單的
- **THEN** 前者 JSON 含 promotedTo 且順序保持；後者無 promotedTo 鍵；反序列化無此鍵的舊 payload 不失敗且值為空清單

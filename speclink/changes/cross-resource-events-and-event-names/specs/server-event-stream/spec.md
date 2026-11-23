## ADDED Requirements

### Requirement: 跨資源事件的歸屬

事件歸到資源的對照 SHALL 涵蓋兩個新的事件種類：`discussion-unlinked` SHALL 歸到 scope `discussion`，resourceId 是事件帶的討論 slug；`change-provenance-changed` SHALL 歸到 scope `change`，resourceId 是事件帶的 change 名。一次提交帶多筆事件時，Server SHALL 為每一筆事件各推一筆 invalidation，順序 SHALL 與提交裡事件的順序相同；一筆 invalidation SHALL 仍然只有一個 scope 與一個 resourceId，資料形狀 SHALL 不變。既有事件種類的歸屬 SHALL 不變。用戶端收到 invalidation 之後的行為 SHALL 不變：重新讀取，不從事件取得內容。

未認證與非成員的請求 SHALL 與既有行為相同地被拒絕（401、403）；斷線續傳、保留政策與 reset 訊號 SHALL 不因為一次提交的事件變多而改變：每一筆事件 SHALL 有自己的事件識別，續傳 SHALL 以事件識別為單位。

#### Scenario: 從討論建立變更推兩筆

- **WHEN** 訂閱者連上事件串流，另一個使用者以討論 two 為來源建立變更 add-two
- **THEN** 訂閱者依序收到兩筆 invalidation：scope 為 change、resourceId 為 add-two；scope 為 discussion、resourceId 為 two

#### Scenario: 廢棄變更推出被解開的討論

- **WHEN** 訂閱者連上事件串流，另一個使用者廢棄由討論 two 轉出的變更 add-two
- **THEN** 訂閱者依序收到 scope 為 change、resourceId 為 add-two，以及 scope 為 discussion、resourceId 為 two 的 invalidation

#### Scenario: 重新結論推出受影響的變更

- **WHEN** 訂閱者連上事件串流，另一個使用者對已反映到變更 add-two 的討論 two 重新結論
- **THEN** 訂閱者依序收到 scope 為 discussion、resourceId 為 two，以及 scope 為 change、resourceId 為 add-two 的 invalidation

#### Scenario: 續傳以事件為單位

- **WHEN** 訂閱者在收到「從討論建立變更」的第一筆 invalidation 之後斷線，並以該筆的事件識別續傳
- **THEN** 訂閱者收到第二筆（scope 為 discussion、resourceId 為 two），沒有重複收到第一筆

#### Scenario: 既有事件的歸屬不變

- **WHEN** 另一個使用者勾選變更 demo 的一個任務
- **THEN** 訂閱者收到恰一筆 invalidation，scope 為 change、resourceId 為 demo，與這條需求之前相同

## ADDED Requirements

### Requirement: 審查與驗證事件歸到變更

事件串流 SHALL 把審查與驗證的六種領域事件（兩個關卡各自的新增一輪、蓋章、放棄）以及落地回報的事件對映為「變更」類別，資源識別 SHALL 是該變更的名稱，SHALL NOT 以 unknown 類別發出。事件名稱到資源類別的對照 SHALL 只有一份定義，由 Host 提供，示範用 Server 與自行提供推送的宿主 SHALL 共用它。其他對映不了的事件維持以 unknown 類別照發。

#### Scenario: 蓋章事件歸到該變更

- **WHEN** 訂閱者連上事件串流後，另一個請求對變更 demo 完成審查蓋章
- **THEN** 訂閱者收到一筆 invalidation，scope 為 change，resourceId 為 demo

#### Scenario: 新增一輪與放棄同樣歸類

- **WHEN** 對 demo 新增一輪驗證工單，之後放棄驗證
- **THEN** 訂閱者收到兩筆 invalidation，scope 都是 change，resourceId 都是 demo

#### Scenario: 落地事件歸到該變更

- **WHEN** 對 demo 回報落地
- **THEN** 訂閱者收到 scope 為 change、resourceId 為 demo 的 invalidation

#### Scenario: 其他未知事件維持 unknown

- **WHEN** 看板順序被寫入
- **THEN** 訂閱者收到 scope 為 unknown 的 invalidation

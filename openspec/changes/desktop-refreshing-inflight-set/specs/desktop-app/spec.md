## ADDED Requirements

### Requirement: 首訪載入失敗終態呈現

活躍 workspace 尚無已載入快照（首訪）且整批載入以失敗收場時，看板卡片區 SHALL 顯示載入失敗提示文案，SHALL NOT 顯示與真空 workspace 相同的空態文案，SHALL NOT 停留在 skeleton。提示 SHALL 於下一次成功載入後消失並恢復正常呈現。失敗記錄 SHALL 隨 workspace 快照存續——切走再切回、重試再度失敗後仍顯示失敗提示而非空態；切回觸發的重試在途期間 SHALL 呈現載入骨架（載入中優先於失敗提示，且不顯示空態文案）。已有舊快取的 workspace 重載失敗 SHALL 維持既有行為（沿用最後成功快照靜默呈現，不顯示失敗提示）。

#### Scenario: 首訪載入失敗顯示失敗提示

- **WHEN** 切換至首次造訪的 workspace 且整批載入失敗（如離線或目錄不可讀）
- **THEN** 骨架收掉，卡片區顯示載入失敗提示文案，與空 workspace 的空態呈現可區分

#### Scenario: 失敗後成功載入恢復正常

- **WHEN** 顯示失敗提示的 workspace 隨後一次整批載入成功
- **THEN** 失敗提示消失，顯示實際資料（或真空態文案）

#### Scenario: 失敗記錄隨快照存續

- **WHEN** 自顯示失敗提示的 workspace 切至他處再切回，期間無成功載入
- **THEN** 切回觸發的重試在途期間顯示骨架（載入中優先），重試仍失敗後回到失敗提示；全程不顯示空態文案

#### Scenario: 舊快取重載失敗不受影響

- **WHEN** 已載入過的 workspace 於重載時失敗
- **THEN** 照常顯示最後一次成功快照，無失敗提示、無骨架

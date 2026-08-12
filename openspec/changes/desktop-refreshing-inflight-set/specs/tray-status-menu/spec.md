## ADDED Requirements

### Requirement: 面板分區載入失敗終態

活躍 workspace 首訪整批載入失敗時，tray 面板各分區 SHALL 顯示載入失敗提示、SHALL NOT 與「尚無進行中變更」等空態同貌、SHALL NOT 停留在 skeleton；載入態的翻轉 SHALL 即時抵達面板（不因清單內容去抖而延遲骨架收掉）。失敗終態 SHALL 經 TraySnapshot 自主視窗 store 導出，面板 SHALL NOT 自建失敗狀態。remote 復原態既有的資料遮蔽 SHALL 維持優先——遮蔽期間不顯示失敗提示。

#### Scenario: 面板首訪失敗顯示提示

- **WHEN** 活躍 workspace 首訪整批載入失敗
- **THEN** 面板分區骨架即時收掉，內容顯示載入失敗提示，與空態列可區分

#### Scenario: 復原遮蔽優先於失敗提示

- **WHEN** 活躍分頁為 remote 且處於遮蔽 workspace 資料的復原狀態
- **THEN** 面板顯示既有復原呈現，不顯示失敗提示與 skeleton

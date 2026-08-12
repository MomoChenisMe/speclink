## ADDED Requirements

### Requirement: 面板分頁切換中回饋

tray 面板分頁條在本地分頁的專案探測期間 SHALL 於目標分頁顯示切換中 spinner，分區內容 SHALL 維持原 workspace 資料照常呈現；探測成功或失敗時 spinner SHALL 消失。切換中狀態 SHALL 經 TraySnapshot 自主視窗 store 導出，面板 SHALL NOT 自建切換中狀態。

#### Scenario: 面板點擊分頁立即回饋

- **WHEN** 於 tray 面板點擊非活躍的本地分頁且專案探測尚未完成
- **THEN** 目標分頁立即顯示 spinner，分區維持原 workspace 內容

#### Scenario: 切換完成後高亮更新

- **WHEN** 專案探測成功且活躍分頁切換完成
- **THEN** spinner 消失，面板分頁高亮移至新活躍分頁

### Requirement: 面板分區首訪 skeleton

活躍 workspace 的快照尚未完成首次載入時，tray 面板各分區 SHALL 以 skeleton 佔位列呈現內容、分區標題 SHALL 照常顯示；快照已載入（含舊快取）時 SHALL 直接顯示既有資料，重載完成靜默更新。載入中狀態 SHALL 經 TraySnapshot 導出。remote 復原態既有的資料遮蔽 SHALL 優先於 skeleton——遮蔽期間不顯示骨架。

#### Scenario: 首訪 workspace 分區顯示骨架列

- **WHEN** 切換至首次造訪的 workspace 且整批載入尚未完成
- **THEN** 面板分區標題照常，內容為 skeleton 佔位列，與「尚無進行中變更」空態可區分

#### Scenario: 已訪 workspace 直接顯示舊資料

- **WHEN** 切換至先前造訪過的 workspace
- **THEN** 分區立即顯示上次快照內容，全程無 skeleton

#### Scenario: remote 復原態不出骨架

- **WHEN** 活躍分頁為 remote 且處於遮蔽 workspace 資料的復原狀態
- **THEN** 面板顯示既有復原呈現，不顯示 skeleton

## ADDED Requirements

### Requirement: 壞 metadata 不參與看板排序寫入

`.openspec.yaml` 存在但 YAML 解析失敗的 change：排序寫入（桌面拖排經引擎的 board_rank 寫入）SHALL 在文字手術前拒絕且 SHALL NOT 寫檔；欄內補章 SHALL NOT 將該 change 視為缺 rank 卡——補章 SHALL 僅對 metadata 有效的缺 rank 卡寫入，metadata 損壞的卡片 SHALL 照其階段顯示並帶 invalid 標記，且 SHALL NOT 因補章被寫入。單一損壞卡 SHALL NOT 使整欄補章或看板清單中止。

#### Scenario: 排序寫入對壞 metadata 拒絕

- **WHEN** 對壞 metadata 的 change 執行 board_rank 排序寫入
- **THEN** 回帶檔案位置與解析原因的錯誤；該 `.openspec.yaml` 逐位元不變

#### Scenario: 補章排除 invalid 卡且不中止

- **WHEN** 同一欄內同時存在缺 rank 的有效卡與 metadata 損壞卡，觸發整欄補章
- **THEN** 僅有效卡被寫入 board_rank；損壞卡的 `.openspec.yaml` 逐位元不變；其餘卡片補章照常完成

#### Scenario: 看板照常開啟並標記損壞卡

- **WHEN** 桌面看板載入含 metadata 損壞 change 的 workspace
- **THEN** 看板照常開啟且列出全部卡片；損壞卡帶 invalid 標記；對其發起的變更操作被引擎錯誤拒絕

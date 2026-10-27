## MODIFIED Requirements

### Requirement: 選單內容與看板同源

<!-- BEFORE: 以 openspec/ 與 .openspec.yaml 描述規格資料夾與變更資料檔 -->

選單內容 SHALL 由前端資料層（與看板同一 store）供給，並於資料變動時重建；SHALL NOT 為 tray 另建第二條資料查詢路徑。外部寫者改動 speclink/ 觸發既有自動刷新後，選單 SHALL 反映刷新後的狀態，無需主視窗位於前景。

#### Scenario: 背景寫入後選單反映新進度
- **WHEN** 主視窗不在前景，agent 或 CLI 將某進行中變更的任務自 0/12 推進至 3/12，既有自動刷新管線生效
- **THEN** 下次展開系統匣選單時該變更顯示 3/12，與看板一致

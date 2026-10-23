## ADDED Requirements

### Requirement: 公司 Proxy 更新失敗的定位與呈現

Desktop SHALL 將更新元件未就緒、連線／逾時／TLS／Proxy認證／HTTP／解析／畫面錯誤區分呈現，更新頁可讀最後檢查時間、狀態及可得的診斷operationId。更新檢查、下載與連線測試 SHALL 沿同一已選定Proxy政策，連線失敗不偷偷直連，未知HTTP碼維持null，不從訊息猜根因。Debug開啟時 SHALL 以既有遮罩schema記錄同次工作；Debug關閉仍須可見錯誤狀態。重試前 SHALL 不清除內附日誌或冒稱檢查成功；重試成功才更新結果。SHALL 不為處理公司錯誤關閉憑證／來源／更新簽章檢查。

#### Scenario: 模擬 CONNECT 與407

- **WHEN** 同一手動Proxy先允許HTTPS CONNECT、再要求認證而回407，使用者依序手動檢查更新
- **THEN** 首次能依原驗證流程取得結果，第二次顯示Proxy認證失敗或可得的407；兩次路由一致，失敗時版本／日誌仍可讀且不自動直連

#### Scenario: 無可得HTTP碼的TLS失敗

- **WHEN** Proxy路由可達但TLS憑證不可信、更新外掛沒有公開HTTP狀態
- **THEN** 顯示TLS錯誤、HTTP狀態為null、診斷保留可得類別及operationId，沒有放寬憑證檢查或偽造407

### Requirement: 公司 Proxy 打包後驗收

公司 Proxy 的 Desktop 實機驗收 SHALL 使用打包後的安裝版本，開發測試期間 SHALL 記為延期並跳過操作；手動任務 SHALL 保持未完成，不因模擬測試或公司外網測試通過而勾選。延期 SHALL 不阻擋繼續程式開發或產生測試安裝檔，SHALL NOT 改寫產品任務守門或宣稱正式驗收／封存完成。

交付 SHALL 先將功能分支合併回本專案 main，再從 main 的明確 commit 打包產生安裝檔，最後才在公司執行實機驗收。驗收紀錄 SHALL 包含 main commit、安裝檔名稱與版本；合併 SHALL NOT 視為人工驗收或封存完成。延期的 12.3 SHALL 不阻擋此合併後打包安排，其他合併前必要檢查仍須完成。

#### Scenario: 開發環境無公司安裝條件

- **WHEN** 開發者只能在公司外網執行 npm run dev:desktop，尚未提供公司可安裝的打包版本
- **THEN** 公司驗收記錄說明「開發期間跳過，打包後執行」，任務 12.3 仍未勾選，繼續實作其他任務，不寫公司測試通過

#### Scenario: 打包後進入公司測試

- **WHEN** 功能分支已合併回 main，操作者在公司安裝由 main 明確 commit 打包的版本並驗版本／日誌、手動檢查及下載或手動 Release 入口
- **THEN** 記錄實際版本、OS、Proxy 模式及遮罩結果，只有實際結果符合要求後才由操作者確認；失敗保留具體現象及未完成狀態

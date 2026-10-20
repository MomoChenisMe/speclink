## ADDED Requirements

### Requirement: 公司 Proxy 更新失敗的定位與呈現

Desktop SHALL 將更新元件未就緒、連線／逾時／TLS／Proxy認證／HTTP／解析／畫面錯誤區分呈現，更新頁可讀最後檢查時間、狀態及可得的診斷operationId。更新檢查、下載與連線測試 SHALL 沿同一已選定Proxy政策，連線失敗不偷偷直連，未知HTTP碼維持null，不從訊息猜根因。Debug開啟時 SHALL 以既有遮罩schema記錄同次工作；Debug關閉仍須可見錯誤狀態。重試前 SHALL 不清除內附日誌或冒稱檢查成功；重試成功才更新結果。SHALL 不為處理公司錯誤關閉憑證／來源／更新簽章檢查。

#### Scenario: 模擬 CONNECT 與407

- **WHEN** 同一手動Proxy先允許HTTPS CONNECT、再要求認證而回407，使用者依序手動檢查更新
- **THEN** 首次能依原驗證流程取得結果，第二次顯示Proxy認證失敗或可得的407；兩次路由一致，失敗時版本／日誌仍可讀且不自動直連

#### Scenario: 無可得HTTP碼的TLS失敗

- **WHEN** Proxy路由可達但TLS憑證不可信、更新外掛沒有公開HTTP狀態
- **THEN** 顯示TLS錯誤、HTTP狀態為null、診斷保留可得類別及operationId，沒有放寬憑證檢查或偽造407

## MODIFIED Requirements

### Requirement: 伺服器管理最小面

應用程式設定頁 SHALL 提供伺服器頁籤（app 全域範圍、與任何專案分頁無關）：呈現 saved servers 清單（顯示名、origin、登入狀態與身分）、新增連線（URL 與顯示名）、登入、登出與移除操作。表單控制項 SHALL 使用專案自建 UI 元件、文案為繁體中文。此頁籤為最小管理面。

#### Scenario: 新增後清單即時反映

- **WHEN** 於伺服器頁籤新增 URL 與顯示名
- **THEN** 清單立即出現該條目並進入登入流程；完成登入後條目顯示身分名

# 實作驗證紀錄

日期：2026-10-14（Asia/Taipei）。

## 已驗證

- 外觀：system／light／dark 對系統深淺色的矩陣、讀取缺席／非法值／失敗、保存失敗保留原選擇與配色、重建恢復、系統變更與監聽清理。
- 介面：無專案可操作、中英文選項、鍵盤 Enter、aria-pressed、錯誤 alert、通知接收有效外觀及既有通知即時切換。
- 面板：不依賴工作區更新的即時推送、先完成訂閱再 ready、初次快照套色後呈內容、後續快照與重掛載、延遲訂閱取消清理。
- 目錄：本機根目錄、跨設定頁籤、作用中專案切換、遠端 checkoutRoot／未連結、離線 session、revision 衝突、中文空白及 Windows 長路徑原文保留、HTML-like 路徑以文字呈現。
- desktop 與 server-web production build 成功；共用 CSS 可編譯，仍有原有的大 chunk 建議。
- 桌面全套 Vitest 749／749、共用 UI 614／614 通過；Server Web 首次與其他工作區同時併跑時有三項查找逾時，降低 worker 數後 146／146 通過，未修改 Server Web 原始碼或測試。
- scripts 全組 290／290 通過。
- 額外 tsc 診斷與本輪修改前所記錄的結果相同（僅行號變動）：專案原有測試 fixture、ES lib 與 ArchivedTarget narrowing 錯誤仍存在；本輪新增模組無型別錯誤。Vite build 不執行該全專案型別檢查。

## 邊界檢查（task 5.1）

- 偏好入口僅允許 system／light／dark；未知儲存值不自動寫回，失敗的保存不通知訂閱者或更新畫面。
- app 與面板共用有效配色，面板不保存第二份偏好；延遲註冊後卸載會解除監聽，不送過期 ready。
- 目錄直接來自 session locator，以 React 文字節點輸出，不經 HTML、路徑拼接、外部查詢或寫入。
- 未變更 Server 認證、權限、政策 revision 或 CLI／引擎輸出；既有 golden 無需更新。

## 品質站問題修正：目錄空白保留

- 修正 ProjectSettingsView 的目錄文字：使用 white-space: pre-wrap 保留連續空白、Tab 與換行，原有 overflow-wrap: anywhere 與文字選取維持不變。
- Red：新增三個真實路徑值 `/work/My  Project`、`/work/Tab\tProject`、`/work/Line\nProject` 的回歸案例，檢查原始 textContent 與保留空白且允許換行的計算樣式；修正前三項皆失敗。
- Green：修正後專案設定頁 69／69 通過；全套 desktop 752／752、packages/ui 614／614、server-web 146／146、scripts 290／290 通過，desktop 與 server-web production build 成功。
- 自動化證據確認原文與 white-space 設定，未宣稱取得實際 WebView 選取複製的新證據；先前使用者桌面驗收紀錄保留。Server Web 的 task 5.4 仍未測。

## 使用者桌面實機驗收回報（task 5.2，已完成）

使用者依本次提供的操作清單回報以下結果；這些是使用者的實機驗收結果，不是代理自行操作取得的證據。

| 項目 | 使用者回報 |
| --- | --- |
| 外觀 | 通過 |
| 重開記憶 | 通過 |
| 專案目錄與複製 | 通過 |
| macOS 面板 | 通過 |
| 錯誤通知 | 通過 |

依使用者確認，桌面驗收已全部完成，task 5.2 勾選完成；原本混在此任務的 Server Web 驗收獨立為 task 5.4，既有 task 5.3 的自動化驗證維持完成。

## 尚未完成（task 5.4）

Server Web：未測。剩餘實際瀏覽器驗收：未設定 data-theme 外觀覆寫時，切換系統深淺模式，確認頁面背景、文字與狀態色仍隨系統切換，且不受桌面 app 儲存偏好影響，記錄操作環境與兩種配色結果。自動化測試與建置已通過，但未取代此項實際畫面驗收；僅 task 5.4 維持未勾選。

先前代理實機操作受工具權限限制：Chrome／iab 瀏覽器介面不可用；原生 Chrome 回覆「Computer Use was not approved to use Google Chrome」，以明確路徑選取 Speclink 後回覆「Computer Use permissions are not granted」。未透過其他 UI 操作方式繞過權限；桌面部分後續由使用者完成上述驗收。

## ADDED Requirements

### Requirement: 觸及檔案系統的 command 不佔用主執行緒

桌面 app 中凡會觸及檔案系統或 spawn 子進程（git 等）的 Tauri command SHALL 以 async 形式將工作委派至背景執行緒（spawn_blocking）執行；主執行緒 SHALL NOT 執行任何檔案 IO 或子進程等待。純記憶體讀取與純視窗操作的 command（如行程環境讀取、連線健康狀態、系統匣面板開閉、結束 app）不受此限。command 的對外契約（名稱、參數、回傳資料形狀）SHALL 與同步時代完全一致。

#### Scenario: 引擎讀取耗時期間 UI 保持可回應

- **WHEN** 使用者切換 workspace（主視窗分頁或 tray 面板皆同），且引擎讀取因 repo 忙碌（如 agent 同時在同一 repo 操作）而耗時數秒
- **THEN** 主視窗與系統匣仍可互動——視窗事件與 tray 圖示點擊即時回應，資料以載入中狀態呈現，不發生整窗凍結

#### Scenario: async 化不改變 command 對外契約

- **WHEN** 前端以既有名稱與參數 invoke 任一改為 async 的 command
- **THEN** 回傳資料形狀與錯誤語意與同步時代完全一致，前端無需任何配合改動

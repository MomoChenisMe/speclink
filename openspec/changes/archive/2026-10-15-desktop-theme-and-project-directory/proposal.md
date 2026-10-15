## Why

透過 AI 代理跑 SDD 的開發者、PO 與 PM 在桌面 app 閱讀討論、提案、任務與規格時，需要自行選擇舒適的深淺配色，並在多專案切換時確認目前專案的實際位置。目前配色只能跟隨系統，專案設定只顯示設定檔相對路徑；既有色票與工作區位置資料已足以支援這兩項需求。

## What Changes

- 應用程式設定的本機設定增加「外觀」三選：跟隨系統／淺色／深色，預設跟隨系統；選擇立即生效、保存於此裝置，重開仍記住。
- 全部專案、文件、狀態標示、通知與 macOS 系統匣面板使用同一有效配色；手動選擇優先於系統，跟隨系統時回應系統變更。
- 專案設定頁頂端顯示「專案目錄」及完整根目錄，唯讀、可選取複製、長路徑換行，跟隨作用中專案切換。
- 遠端專案顯示已連結本機副本的目錄，無副本時顯示「未連結本機目錄」。

## Capabilities

### New Capabilities

無。規格掃描命中 desktop-config、desktop-app、tray-status-menu，設定入口、畫面配色與系統匣面板均已有明確歸屬，無需另建能力。

### Modified Capabilities

- `desktop-config`: 新增裝置級外觀偏好與專案目錄展示需求。
- `desktop-app`: 文件與通知的配色由僅依系統改為依有效外觀偏好；所有狀態色與共用控制項一致套用。
- `tray-status-menu`: macOS 面板的主題背景、文字與狀態色跟隨 app 外觀，包含面板首次建立與再次開啟。

## Impact

- Affected specs: desktop-config、desktop-app、tray-status-menu。
- Affected app/package: apps/desktop、packages/ui；apps/server-web 使用共用樣式，須驗證未設定外觀覆寫時仍自動跟隨系統。
- Affected code:
  - New: `apps/desktop/src/theme.ts`、`apps/desktop/src/__tests__/theme.test.ts`。
  - Modified: `apps/desktop/src/main.tsx`、`apps/desktop/src/App.tsx`、`apps/desktop/src/views/AppSettingsView.tsx`、`apps/desktop/src/views/ProjectSettingsView.tsx`、`apps/desktop/src/i18n/messages.ts`、`apps/desktop/src/index.css`、`apps/desktop/src/tray.ts`、`apps/desktop/src/panel/main.tsx`、`packages/ui/src/theme.css`、`packages/ui/src/components/ui/sonner.tsx`、`apps/desktop/src/__tests__/App.test.tsx`、`apps/desktop/src/__tests__/appSettingsView.test.tsx`、`apps/desktop/src/__tests__/projectSettingsView.test.tsx`、`apps/desktop/src/__tests__/tray.test.ts`、`apps/desktop/src/__tests__/trayPanel.test.tsx`、`packages/ui/src/__tests__/theme.test.ts`、`packages/ui/src/__tests__/sonner.test.tsx`。
  - Removed: 無。
- 相容性影響：既有使用者沒有新偏好時維持跟隨系統；不更改工作流或 checkout 設定欄位、CLI、人眼／JSON 輸出、引擎、Server API 或生成技能。僅延伸桌面內部 tray-snapshot 的有效配色欄位；不新增套件依賴或 Rust 指令。

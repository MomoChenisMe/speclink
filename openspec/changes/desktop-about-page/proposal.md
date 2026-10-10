## Summary

為桌面 app（apps/desktop）的設定模式加上「關於」群組與「關於 Speclink」頁：品牌卡（橫式鎖版、版本列、一句產品描述）、版本卡（檢查更新、更新日誌、複製版本資訊）、連結卡（原始碼、說明文件、回報問題）、授權卡（MIT 授權條款、第三方授權）；macOS 原生選單的「關於 Speclink」直接開此頁。

## Motivation

結論把「關於」定為設定模式「關於」分類下唯一一頁，也是橫式鎖版的第二個表面（第一個是零專案空狀態）；使用者在互動原型確認了畫板。目前原生選單的「關於 Speclink」只能先落到設定頁，版本資訊散在軟體更新卡，回報問題時沒有一處能一次複製 app、引擎、平台與技能檔版本。

目標使用者是回報問題、確認版本、找原始碼與文件的使用者。

本 change 依賴 `desktop-settings-mode`（設定分類欄與五個設定零件）。

## Proposed Solution

- **分類欄**：設定分類欄新增群組「關於」：關於 Speclink；麵包屑「設定 / 關於 Speclink」。
- **版本資訊來源**：新增 Tauri 命令回傳 app 版號、引擎版號（speclink-core crate 版本）、平台與架構、技能檔版本（ASSET_VERSION）；前端一次取得。
- **頁內容**（原型「設定關於頁」畫板）：品牌卡——`Wordmark` 40px（深色用深色鎖版）＋灰字版本列「app 版號 · 引擎 版號 · 平台 架構 · 更新狀態籤」＋一句產品描述，不另放裸標記；版本卡三列——檢查更新（說明含上次檢查時間；與更新頁共用同一份更新狀態）、更新日誌（開瀏覽模式對話框）、複製版本資訊（寫入四行版本文字到剪貼簿並顯示已複製）；連結卡三列整列可點＋外開圖示——原始碼、說明文件、回報問題（issue 網址帶入版本資訊）；授權卡兩列——授權條款 MIT（檢視：對話框顯示隨 app 打包的 LICENSE 全文）、第三方授權（檢視：對話框列出桌面 app 與 CLI 用到的開源套件名稱、版本與授權，清單於建置時由腳本自 npm 與 cargo 相依產生）。
- **原生選單**：「關於 Speclink」開此頁（取代目前落到設定頁的暫代）。

## Non-Goals

- 系統原生的關於面板。
- 第三方授權全文——只列套件名稱、版本與授權識別字。
- server-web 的關於頁。

## Alternatives Considered

- 用系統預設的關於面板——放不下鎖版與連結，也不跟 UI 語言走（結論已否決）。
- 第三方授權清單手寫維護——會漂；由腳本自 package-lock 與 cargo metadata 產生，建置時重生。
- 版本資訊逐項由前端拼（getVersion、navigator.platform）——引擎版號與技能檔版本只有 Rust 端知道，一個命令一次回。

## Impact

- Affected specs: desktop-app（ADDED 關於頁；側欄導覽結構 MODIFIED 補「關於」群組）
- Affected code:
  - New: `apps/desktop/src/views/settings/AboutPage.tsx`、`apps/desktop/src/adapter/appInfo.ts`、`apps/desktop/src/components/LicenseDialog.tsx`、`scripts/desktop/third-party-licenses.mjs`、`scripts/desktop/third-party-licenses.test.mjs`、`apps/desktop/src/__tests__/aboutPage.test.tsx`
  - Modified: `apps/desktop/core/src/project.rs`、`apps/desktop/src-tauri/src/lib.rs`、`apps/desktop/package.json`、`.gitignore`、`apps/desktop/src/settingsCatalog.ts`、`apps/desktop/src/components/SettingsColumn.tsx`、`apps/desktop/src/App.tsx`、`apps/desktop/src/main.tsx`、`apps/desktop/src/store.ts`、`apps/desktop/src/views/AppSettingsView.tsx`、`apps/desktop/src/i18n/messages.ts`、`packages/ui/src/components/ui/settings.tsx`、`packages/ui/src/__tests__/settingsParts.test.tsx`、`apps/desktop/src/__tests__/App.test.tsx`、`apps/desktop/src/__tests__/store.test.ts`、`apps/desktop/src/__tests__/settingsColumn.test.tsx`
  - Removed: （無）

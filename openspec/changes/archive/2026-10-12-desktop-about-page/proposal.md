## Summary

為桌面 app（apps/desktop）的設定模式加上「關於」群組與「關於 Speclink」頁：品牌卡（橫式鎖版、版本列附可點的更新狀態籤與「複製版本資訊」鈕、一句產品描述）、連結卡（原始碼、說明文件、回報問題）、授權卡（MIT 授權條款、第三方授權）；macOS 原生選單的「關於 Speclink」直接開此頁。同時收斂應用程式設定的重複：「語言與外觀」併入一般頁，一般頁不再重複更新與 CLI；更新日誌的「檢視」改為捲到該版。

## Motivation

結論把「關於」定為設定模式「關於」分類下唯一一頁，也是橫式鎖版的第二個表面（第一個是零專案空狀態）；使用者在互動原型確認了畫板。目前原生選單的「關於 Speclink」只能先落到設定頁，版本資訊散在軟體更新卡，回報問題時沒有一處能一次複製 app、引擎與平台版本。

實機試用（apply 後的手動驗收）發現同一組資訊散在多頁：「目前版本」「檢查更新」「更新日誌」各在一般、更新、關於三頁出現；一般頁的介面語言卡等於整個「語言與外觀」頁、CLI 卡等於 CLI 頁第一列；更新頁每個舊版本列的「檢視」與「全部更新日誌」開的是同一個對話框。使用者決定每個設定列只留在一個分類頁：更新的操作只在更新頁，關於頁只回答「這是哪一版」，一般頁只放語言與外觀和鍵盤快捷鍵。這改變了 desktop-ui-redesign 結論裡「一般頁為常用項目總覽」與「關於頁版本三列」兩點，以本 change 的 artifact 為準。

目標使用者是回報問題、確認版本、找原始碼與文件的使用者，以及在設定模式裡找某個設定的使用者。

本 change 依賴 `desktop-settings-mode`（設定分類欄與五個設定零件）。

## Proposed Solution

- **分類欄**：設定分類欄新增群組「關於」：關於 Speclink；麵包屑「設定 / 關於 Speclink」。
- **版本資訊來源**：新增 Tauri 命令回傳 app 版號、引擎版號（ASSET_VERSION，與 CLI `--version` 的 engine 相同）、平台與架構；前端一次取得。
- **頁內容**（原型「設定關於頁」畫板，依實機回饋去掉版本卡）：品牌卡——`Wordmark` 40px（深色用深色鎖版）＋灰字版本列「app 版號 · 引擎 版號 · 平台 架構」＋更新狀態籤（與更新頁同一份狀態，點擊進更新頁）＋「複製版本資訊」鈕（寫入三行版本文字到剪貼簿並顯示已複製）＋一句產品描述，不另放裸標記；不設「檢查更新」與「更新日誌」列；連結卡三列整列可點＋外開圖示——原始碼、說明文件、回報問題（issue 網址帶入版本資訊）；授權卡兩列——授權條款 MIT（檢視：對話框顯示隨 app 打包的 LICENSE 全文）、第三方授權（檢視：對話框列出桌面 app 與 CLI 用到的開源套件名稱、版本與授權，清單於建置時由腳本自 npm 與 cargo 的正式相依產生）。
- **原生選單**：「關於 Speclink」開此頁（取代目前落到設定頁的暫代）。
- **一般頁收斂**：刪除「語言與外觀」分類，其兩列併入一般頁的「語言與外觀」卡；一般頁拿掉「軟體更新」卡與「CLI 指令」卡，只剩語言與外觀、鍵盤快捷鍵兩卡；分類欄「應用程式」群組為一般、更新、CLI、伺服器。
- **更新日誌檢視單一版本**：更新頁舊版本列的「檢視」開啟瀏覽模式後把該版小標捲到捲動區頂端；「全部更新日誌」維持自頂端呈現。

## Non-Goals

- 系統原生的關於面板。
- 第三方授權全文——只列套件名稱、版本與授權識別字。
- server-web 的關於頁。
- 背景檢查沒找到新版時更新狀態退回閒置、狀態籤顯示「尚未檢查」——更新頁原本就如此，改它要動更新狀態機的既有行為，另開 change 處理。
- 專案設定四頁——盤點後沒有重複。

## Alternatives Considered

- 用系統預設的關於面板——放不下鎖版與連結，也不跟 UI 語言走（結論已否決）。
- 第三方授權清單手寫維護——會漂；由腳本自 package-lock 與 cargo metadata 產生，建置時重生。
- 版本資訊逐項由前端拼（getVersion、navigator.platform）——引擎版號只有 Rust 端知道，一個命令一次回。

## Impact

- Affected specs: desktop-app（ADDED 關於頁；MODIFIED 側欄導覽結構、桌面自動更新、安裝 CLI 指令到 PATH、更新日誌彈窗）；desktop-config（MODIFIED 設定頁圖形化讀寫兩層設定、UI 介面語言支援 zh-TW 與 en、外觀偏好）
- Affected code:
  - New: `apps/desktop/src/views/settings/AboutPage.tsx`、`apps/desktop/src/adapter/appInfo.ts`、`apps/desktop/src/components/LicenseDialog.tsx`、`scripts/desktop/third-party-licenses.mjs`、`scripts/desktop/third-party-licenses.test.mjs`、`apps/desktop/src/__tests__/aboutPage.test.tsx`
  - Modified: `apps/desktop/core/src/project.rs`、`apps/desktop/src-tauri/src/lib.rs`、`apps/desktop/package.json`、`.gitignore`、`apps/desktop/src/settingsCatalog.ts`、`apps/desktop/src/components/SettingsColumn.tsx`、`apps/desktop/src/App.tsx`、`apps/desktop/src/main.tsx`、`apps/desktop/src/store.ts`、`apps/desktop/src/views/AppSettingsView.tsx`、`apps/desktop/src/views/settings/GeneralPage.tsx`、`apps/desktop/src/views/settings/UpdatesPage.tsx`、`apps/desktop/src/views/settings/CliPage.tsx`、`apps/desktop/src/components/ReleaseNotesDialog.tsx`、`apps/desktop/src/i18n/messages.ts`、`packages/ui/src/components/ui/settings.tsx`、`packages/ui/src/index.ts`、`packages/ui/src/__tests__/settingsParts.test.tsx`、`apps/desktop/src/__tests__/App.test.tsx`、`apps/desktop/src/__tests__/store.test.ts`、`apps/desktop/src/__tests__/settingsColumn.test.tsx`、`apps/desktop/src/__tests__/appSettingsView.test.tsx`、`apps/desktop/src/__tests__/updatesPage.test.tsx`、`apps/desktop/src/__tests__/releaseNotesDialog.test.tsx`、`apps/desktop/src/__tests__/messages.test.ts`
  - Removed: `apps/desktop/src/views/settings/AppearancePage.tsx`（顯示語言列與外觀列移入 GeneralPage.tsx）

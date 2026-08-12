# desktop-release Specification

## Purpose

桌面 app 的發版與更新交付：release 產出三平台安裝檔、隨版發布的更新描述檔與更新包簽章，以及 OS 程式碼簽章這把可插可拔的鑰匙開關。本 capability 保證自動更新的來源可驗證，且本機安裝帶版號新鮮度斷言——裝到舊版會當場被擋下，而非默默沿用。

## Requirements

### Requirement: Release 產出三平台桌面安裝檔

push 符合 v* 的 tag 後，release 管線 SHALL 產出桌面安裝檔並附於同一個 GitHub Release：macOS dmg（aarch64 與 x86_64 各一）、Windows NSIS 安裝器（x86_64）、Linux AppImage 與 deb（x86_64 與 aarch64）。SHA256SUMS.txt SHALL 收錄全部新增檔案。既有 CLI／server 壓縮檔與 Docker 映像的命名與內容 SHALL 維持不變。桌面安裝檔 SHALL 內含同版 speclink CLI binary。

#### Scenario: tag 發布產出完整安裝檔集

- **WHEN** push tag v0.2.0 且 workflow 全部成功
- **THEN** 該 Release 的 assets 同時包含既有五 target 壓縮檔、上列全部桌面安裝檔，且每個檔案在 SHA256SUMS.txt 中有對應條目

#### Scenario: 任一形態失敗則不發布

- **WHEN** 桌面安裝檔任一 target 建置失敗
- **THEN** GitHub Release SHALL NOT 建立（與既有 Docker gating 同一 needs 閘門），不產生缺形態的 Release


<!-- @trace
source: desktop-installer-and-updater
updated: 2026-07-30
code:
  - .github/workflows/release.yml
  - Cargo.lock
  - apps/desktop/package.json
  - apps/desktop/src-tauri/Cargo.toml
  - apps/desktop/src-tauri/capabilities/default.json
  - apps/desktop/src-tauri/src/cli_install.rs
  - apps/desktop/src-tauri/src/lib.rs
  - apps/desktop/src-tauri/tauri.conf.json
  - apps/desktop/src-tauri/windows/hooks.nsh
  - apps/desktop/src/App.tsx
  - apps/desktop/src/__tests__/App.test.tsx
  - apps/desktop/src/__tests__/appSettingsView.test.tsx
  - apps/desktop/src/__tests__/cliInstall.test.ts
  - apps/desktop/src/__tests__/updateBanner.test.tsx
  - apps/desktop/src/__tests__/updater.test.ts
  - apps/desktop/src/adapter/cliInstall.ts
  - apps/desktop/src/adapter/updater.ts
  - apps/desktop/src/components/UpdateBanner.tsx
  - apps/desktop/src/core/cliInstall.ts
  - apps/desktop/src/core/updater.ts
  - apps/desktop/src/i18n/messages.ts
  - apps/desktop/src/main.tsx
  - apps/desktop/src/store.ts
  - apps/desktop/src/views/AppSettingsView.tsx
  - package-lock.json
  - scripts/desktop-sidecar.mjs
  - scripts/release-latest-json.mjs
  - scripts/release-latest-json.test.mjs
-->

---
### Requirement: 更新描述檔隨 release 發布

release 管線 SHALL 組裝更新描述檔 latest.json 附於 Release assets：version 欄位等於 tag 去除 v 前綴、pub_date 為發布時間、platforms 物件至少含 darwin-aarch64、darwin-x86_64、windows-x86_64、linux-x86_64 四鍵，各鍵含 url（指向同一 Release 的對應更新包 asset）與 signature（該更新包的簽章內容）。組裝邏輯 SHALL 有可獨立執行的單元測試。

#### Scenario: 描述檔欄位對齊發布內容

- **WHEN** push tag v0.2.0 且 workflow 成功
- **THEN** latest.json 的 version 為 0.2.0，且 platforms 每一鍵的 url 均指向本次 Release 的 asset 下載路徑

##### Example: v0.2.0 的 platforms 對應

| platforms 鍵 | url 指向的 asset |
| ------------ | ---------------- |
| darwin-aarch64 | v0.2.0 Release 中 aarch64 dmg 對應的更新包 |
| windows-x86_64 | v0.2.0 Release 中 NSIS 對應的更新包 |
| linux-x86_64 | v0.2.0 Release 中 x86_64 AppImage 對應的更新包 |

#### Scenario: 更新端點可匿名讀取

- **WHEN** 以未帶認證的 HTTP GET 請求 releases/latest/download/latest.json 路徑
- **THEN** 回應為可解析的 JSON 且內容為最新 Release 的更新描述檔


<!-- @trace
source: desktop-installer-and-updater
updated: 2026-07-30
code:
  - .github/workflows/release.yml
  - Cargo.lock
  - apps/desktop/package.json
  - apps/desktop/src-tauri/Cargo.toml
  - apps/desktop/src-tauri/capabilities/default.json
  - apps/desktop/src-tauri/src/cli_install.rs
  - apps/desktop/src-tauri/src/lib.rs
  - apps/desktop/src-tauri/tauri.conf.json
  - apps/desktop/src-tauri/windows/hooks.nsh
  - apps/desktop/src/App.tsx
  - apps/desktop/src/__tests__/App.test.tsx
  - apps/desktop/src/__tests__/appSettingsView.test.tsx
  - apps/desktop/src/__tests__/cliInstall.test.ts
  - apps/desktop/src/__tests__/updateBanner.test.tsx
  - apps/desktop/src/__tests__/updater.test.ts
  - apps/desktop/src/adapter/cliInstall.ts
  - apps/desktop/src/adapter/updater.ts
  - apps/desktop/src/components/UpdateBanner.tsx
  - apps/desktop/src/core/cliInstall.ts
  - apps/desktop/src/core/updater.ts
  - apps/desktop/src/i18n/messages.ts
  - apps/desktop/src/main.tsx
  - apps/desktop/src/store.ts
  - apps/desktop/src/views/AppSettingsView.tsx
  - package-lock.json
  - scripts/desktop-sidecar.mjs
  - scripts/release-latest-json.mjs
  - scripts/release-latest-json.test.mjs
-->

---
### Requirement: 更新包簽章

每個平台的更新包 SHALL 以專案自產的 updater 金鑰（minisign）簽章；公鑰 SHALL 隨版本庫提交於桌面設定檔，私鑰與其密語 SHALL 只存於 CI secrets。CI 缺少私鑰 secrets 時桌面建置 SHALL 失敗（fail-closed），不得產出無簽章的更新包。

#### Scenario: 缺私鑰時建置失敗

- **WHEN** CI 環境沒有 updater 私鑰 secret 而執行桌面建置
- **THEN** 建置以非零結束，workflow 標紅，Release 不發布

#### Scenario: 簽章隨更新包產出

- **WHEN** 桌面建置成功
- **THEN** 每個更新包旁存在對應簽章，且其內容與 latest.json 中該平台的 signature 一致


<!-- @trace
source: desktop-installer-and-updater
updated: 2026-07-30
code:
  - .github/workflows/release.yml
  - Cargo.lock
  - apps/desktop/package.json
  - apps/desktop/src-tauri/Cargo.toml
  - apps/desktop/src-tauri/capabilities/default.json
  - apps/desktop/src-tauri/src/cli_install.rs
  - apps/desktop/src-tauri/src/lib.rs
  - apps/desktop/src-tauri/tauri.conf.json
  - apps/desktop/src-tauri/windows/hooks.nsh
  - apps/desktop/src/App.tsx
  - apps/desktop/src/__tests__/App.test.tsx
  - apps/desktop/src/__tests__/appSettingsView.test.tsx
  - apps/desktop/src/__tests__/cliInstall.test.ts
  - apps/desktop/src/__tests__/updateBanner.test.tsx
  - apps/desktop/src/__tests__/updater.test.ts
  - apps/desktop/src/adapter/cliInstall.ts
  - apps/desktop/src/adapter/updater.ts
  - apps/desktop/src/components/UpdateBanner.tsx
  - apps/desktop/src/core/cliInstall.ts
  - apps/desktop/src/core/updater.ts
  - apps/desktop/src/i18n/messages.ts
  - apps/desktop/src/main.tsx
  - apps/desktop/src/store.ts
  - apps/desktop/src/views/AppSettingsView.tsx
  - package-lock.json
  - scripts/desktop-sidecar.mjs
  - scripts/release-latest-json.mjs
  - scripts/release-latest-json.test.mjs
-->

---
### Requirement: OS 程式碼簽章為可插鑰匙開關

macOS 與 Windows 的 OS 程式碼簽章步驟 SHALL 以對應 secrets 是否存在為條件：secrets 不存在時該步驟 SHALL 跳過且 workflow SHALL 成功、產出未簽章安裝檔；secrets 存在時 SHALL 執行簽章。本開關 SHALL NOT 影響更新包的 updater 簽章（兩者正交）。

#### Scenario: 無簽章金鑰照常發布

- **WHEN** repo 未設定任何 OS 簽章 secrets 且 push tag
- **THEN** workflow 全綠，Release 產出未簽章安裝檔，assets 清單與有簽章時一致

#### Scenario: 插入金鑰即啟用簽章

- **WHEN** repo 設定了 macOS 簽章 secrets 後 push tag
- **THEN** macOS 安裝檔經簽章步驟處理，workflow 其餘步驟不變

<!-- @trace
source: desktop-installer-and-updater
updated: 2026-07-30
code:
  - .github/workflows/release.yml
  - Cargo.lock
  - apps/desktop/package.json
  - apps/desktop/src-tauri/Cargo.toml
  - apps/desktop/src-tauri/capabilities/default.json
  - apps/desktop/src-tauri/src/cli_install.rs
  - apps/desktop/src-tauri/src/lib.rs
  - apps/desktop/src-tauri/tauri.conf.json
  - apps/desktop/src-tauri/windows/hooks.nsh
  - apps/desktop/src/App.tsx
  - apps/desktop/src/__tests__/App.test.tsx
  - apps/desktop/src/__tests__/appSettingsView.test.tsx
  - apps/desktop/src/__tests__/cliInstall.test.ts
  - apps/desktop/src/__tests__/updateBanner.test.tsx
  - apps/desktop/src/__tests__/updater.test.ts
  - apps/desktop/src/adapter/cliInstall.ts
  - apps/desktop/src/adapter/updater.ts
  - apps/desktop/src/components/UpdateBanner.tsx
  - apps/desktop/src/core/cliInstall.ts
  - apps/desktop/src/core/updater.ts
  - apps/desktop/src/i18n/messages.ts
  - apps/desktop/src/main.tsx
  - apps/desktop/src/store.ts
  - apps/desktop/src/views/AppSettingsView.tsx
  - package-lock.json
  - scripts/desktop-sidecar.mjs
  - scripts/release-latest-json.mjs
  - scripts/release-latest-json.test.mjs
-->

---
### Requirement: 本機安裝的新鮮度斷言

repo SHALL 提供本機建置安裝入口（node scripts/desktop-install.mjs），把桌面 app 的本機建置與安裝收成單一指令並以版號斷言取代信任。執行時 SHALL 依序：(0) 進入任何步驟之前先行把關——帶 `--install` 而平台非 macOS，或簽章環境變數缺失時，SHALL 單行錯誤（點名平台或缺失的變數名）並以非零結束，SHALL NOT 進入任何建置；(1) 印出當前 HEAD、分支、工作樹是否乾淨與源碼的產物層版號；(2) 執行 sidecar 佈署（永遠重新建置，SHALL NOT 沿用 src-tauri/binaries/ 既有檔案）；(3) 前端建置與 tauri bundle；(4) 斷言 bundle 內 sidecar CLI 的引擎版號等於源碼產物層版號，不等時 SHALL 印出兩邊版號並以非零結束。帶 `--install` 時 SHALL 續行：(5) 確認 app 未執行——執行中 SHALL 單行錯誤停止，SHALL NOT 代為結束程序；(6) 覆蓋 /Applications 安裝，SHALL 先將新版完整佈到暫存路徑再換上，使拷貝失敗時既有安裝原封不動；(7) 斷言安裝版 CLI 的引擎版號同版，不等時 SHALL 印出兩邊版號並以非零結束。安裝步驟（5–7）僅支援 macOS；建置步驟（1–4）平台中立。任一步驟失敗 SHALL 以非零結束且 SHALL NOT 繼續後續步驟。

#### Scenario: 建置並通過 bundle 斷言

- **WHEN** 於簽章環境變數齊備的源碼樹執行 node scripts/desktop-install.mjs
- **THEN** 依序輸出 HEAD 與源碼產物層版號、重建 sidecar、完成 bundle，並以 bundle 內 CLI 的引擎版號等於源碼版號通過斷言，exit code 0

#### Scenario: bundle 版號不符即失敗

- **WHEN** bundle 內 sidecar CLI 的引擎版號與源碼產物層版號不等（如建置鏈沿用了過期 binary）
- **THEN** 印出兩邊版號、exit code 非零、不進行安裝

#### Scenario: 安裝後斷言安裝版同版

- **WHEN** 帶 --install 執行且 app 未執行、建置斷言通過
- **THEN** 覆蓋安裝後以安裝版 CLI 的引擎版號同版通過第二道斷言，exit code 0

#### Scenario: app 執行中拒絕安裝

- **WHEN** 帶 --install 執行而 Speclink app 程序仍在執行
- **THEN** 單行錯誤說明需先關閉 app、exit code 非零、/Applications 零變動

#### Scenario: 簽章環境變數缺失即停

- **WHEN** 簽章環境變數未設定時執行 node scripts/desktop-install.mjs
- **THEN** 單行錯誤指出缺失的變數名、exit code 非零、不進入建置（含 sidecar 重建）

#### Scenario: 非 macOS 帶 --install 即停

- **WHEN** 於非 macOS 平台執行 node scripts/desktop-install.mjs --install
- **THEN** 單行錯誤指出平台、exit code 非零、不進入建置

<!-- @trace
source: instruction-downgrade-guard
updated: 2026-08-06
-->
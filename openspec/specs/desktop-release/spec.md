# desktop-release Specification

## Purpose

TBD - created by archiving change 'desktop-installer-and-updater'. Update Purpose after archive.

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
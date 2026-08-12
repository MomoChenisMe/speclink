## Why

發版管線已能在 push tag 時產出五形態 artifact（CLI／server 壓縮檔、Docker、桌面安裝檔、更新描述檔），但 repo 至今零 tag、從未首發，且缺最後一哩：macOS 安裝檔未簽章未公證（Gatekeeper 會擋）、Windows 無簽章方案（SmartScreen 警告）、CLI 沒有安裝腳本與 Homebrew 通路、README 只教從原始碼建置。討論 release-first-and-distribution 裁定：v0.1.0 首發即納入 Apple Developer 簽章＋公證與 SignPath 開源免費簽章，並補齊 CLI 與 desktop 的快速安裝入口；簽章的帳號申請與 secrets 設定以逐步教學的手動任務落地。

## What Changes

- release 管線補 macOS 公證（notarization）接線：以 Apple ID 三項 secrets 為條件的 env-gated 步驟，與既有憑證開關同一模式；secrets 缺席時維持現行未簽章路徑、workflow 照常全綠
- Windows 簽章接 SignPath 開源簽章服務：桌面建置時以 Tauri signCommand 呼叫 SignPath 送簽（CI 專用 config overlay，SignPath secrets 存在才啟用）；申請未過件的退路是維持未簽章＋文件放行說明
- 新增 CLI 安裝腳本：scripts/install.sh（macOS／Linux）與 scripts/install.ps1（Windows），偵測平台、解析最新 Release、驗證 SHA-256、安裝至使用者 PATH 目錄；支援 dry-run 與環境變數覆寫以利測試
- 新增 Homebrew formula 產生器：讀取 Release 的版號與 checksums、輸出 formula 內容；配套建立自有 tap repo（外部 repo，手動任務）
- README（中英）新增安裝區塊：桌面三平台下載表、CLI 安裝腳本與 Homebrew 指令；cargo install 從原始碼安裝降為開發者段落；getting-started（中英）安裝節同步改寫
- sdk-node 文件（中英）將 npm install 說明改標「尚未發布至 npm」，改教 repo 內建置路徑
- product-status 查核日與相關列刷新
- 主 CI 納入 `scripts` 測試面：本 change 新增的閘門、安裝腳本與 formula 產生器測試原本只在本機 `test:all` 跑得到，CI 三平台皆未涵蓋，等於這批設定契約無人看守
- 首發實跑暴露並修掉兩個潛在缺陷：未知 API 路徑回裸 404 而非正典要求的 JSON 404（四個 smoke target 同時紅燈，既有測試漏驗 JSON 那一半）、Linux arm64 桌面建置缺 AppImage 所需的 xdg-utils
- 首發演練：確認版號同版、推 v0.1.0 tag、驗證 Release 產物（含簽章與公證結果）與各平台安裝實測

## Capabilities

### New Capabilities

- `cli-distribution`: CLI 的安裝通路——安裝腳本（sh／PowerShell）的平台偵測、checksum 驗證與安裝行為，以及 Homebrew formula 產生器的輸出契約

### Modified Capabilities

- `desktop-release`: 「OS 程式碼簽章為可插鑰匙開關」requirement 擴充——macOS 簽章 secrets 存在時同時執行公證；Windows 新增 SignPath signCommand 路徑；開關語意與 updater 簽章正交性維持不變
- `user-documentation`: 新增安裝通路文件 requirement——README 安裝區塊（中英對等）、sdk-node 發布狀態誠實化
- `delivery-baseline`: 「CI 執行完整測試」requirement 擴充——測試面納入 `scripts`，並要求其執行方式相容於 workflow 釘選的 Node 版本與三平台預設 shell

## Impact

- Affected specs: `cli-distribution`（新增）、`desktop-release`、`user-documentation`、`delivery-baseline`
- Affected code:
  - New: `scripts/signing-gate.mjs`、`scripts/signing-gate.test.mjs`、`scripts/install.sh`、`scripts/install.ps1`、`scripts/install.test.mjs`、`scripts/homebrew-formula.mjs`、`scripts/homebrew-formula.test.mjs`、`scripts/signpath-sign.ps1`、`scripts/signpath-sign.test.mjs`
  - Modified: `.github/workflows/release.yml`、`.github/workflows/ci.yml`、`scripts/delivery-gate.test.mjs`、`crates/speclink-server/src/assets.rs`、`crates/speclink-server/tests/it/web_assets.rs`、`README.md`、`README.en.md`、`docs/getting-started.zh-TW.md`、`docs/getting-started.md`、`docs/sdk-node.zh-TW.md`、`docs/sdk-node.md`、`docs/product-status.zh-TW.md`
  - Removed: 無
- repo 之外（不產生本 repo 檔案）：新建 GitHub tap repo（homebrew-tap，含 formula）；GitHub Actions secrets 新增 Apple 公證三項與 SignPath 四項——皆以手動教學任務執行

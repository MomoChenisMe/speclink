## Why

發版管線已能在 push tag 時產出五形態 artifact（CLI／server 壓縮檔、Docker、桌面安裝檔、更新描述檔），但 repo 至今零 tag、從未首發，且缺最後一哩：macOS 安裝檔未簽章未公證（Gatekeeper 會擋）、Windows 無簽章方案（SmartScreen 警告）、CLI 沒有安裝腳本與 Homebrew 通路、README 只教從原始碼建置。討論 release-first-and-distribution 裁定：v0.1.0 首發即納入 Apple Developer 簽章＋公證與 SignPath 開源免費簽章，並補齊 CLI 與 desktop 的快速安裝入口；簽章的帳號申請與 secrets 設定以逐步教學的手動任務落地。

## What Changes

- release 管線補 macOS 公證（notarization）接線：以 Apple ID 三項 secrets 為條件的 env-gated 步驟，與既有憑證開關同一模式；secrets 缺席時維持現行未簽章路徑、workflow 照常全綠
- Windows 簽章：v0.1.2 後裁定不採用 SignPath（其免費方案每個 release 須人工核准，與全自動發版管線相斥），維持未簽章＋README 的 SmartScreen 放行說明；本機憑證 secrets 路徑保留，未來需簽章時偏好可全自動的 Azure Trusted Signing 另立 change
- 新增 CLI 安裝腳本：scripts/install.sh（macOS／Linux）與 scripts/install.ps1（Windows），偵測平台、解析最新 Release、驗證 SHA-256、安裝至使用者 PATH 目錄；支援 dry-run 與環境變數覆寫以利測試
- 新增 Homebrew formula 產生器：讀取 Release 的版號與 checksums、輸出 formula 內容；配套建立自有 tap repo（外部 repo，手動任務）
- README（中英）新增安裝區塊：桌面三平台下載表、CLI 安裝腳本與 Homebrew 指令；cargo install 從原始碼安裝降為開發者段落；getting-started（中英）安裝節同步改寫
- sdk-node 文件（中英）將 npm install 說明改標「尚未發布至 npm」，改教 repo 內建置路徑
- product-status 查核日與相關列刷新
- 主 CI 納入 `scripts` 測試面：本 change 新增的閘門、安裝腳本與 formula 產生器測試原本只在本機 `test:all` 跑得到，CI 三平台皆未涵蓋，等於這批設定契約無人看守
- 首發實跑暴露並修掉兩個潛在缺陷：未知 API 路徑回裸 404 而非正典要求的 JSON 404（四個 smoke target 同時紅燈，既有測試漏驗 JSON 那一半）、Linux arm64 桌面建置缺 AppImage 所需的 xdg-utils
- 發版實跑暴露的 docker 管線問題兩項：QEMU 下跑 Node/V8 的 SIGILL flaky（web 階段釘建置機原生架構根治）；arm64 的 Rust 編譯在 QEMU 模擬下讓 docker job 單跑逾 35 分鐘（改為原生 arm64 runner 分開建＋manifest 合併）
- 安裝體驗補強兩項：Windows NSIS 安裝器內建繁中＋英文語系跟隨系統顯示、安裝器圖示採 Speclink logo；macOS 的 CLI 佈署從使用者顯式動作改為 app 啟動自動佈署（未安裝即裝、版本不符即修復），~/.local/bin 缺席於 PATH 時自動冪等追加至 ~/.zprofile
- server 壓縮檔退出 Release assets：release 管線仍建置各平台 server 並跑無 dist 冒煙（品質閘門保留），但不再打包上傳；server 的官方發布通路收斂為 Docker image，部署文件的 native binary 形態改教從原始碼建置
- Release 說明自動產生下載指南：v0.1.0 首發後確認 assets 清單對一般使用者過於龐雜（安裝檔、CLI 壓縮檔、更新包與簽章檔混列），release job 改以腳本產生「各平台該下載哪個檔案」對照表前置於 Release 說明，自動 changelog 接續其後；已發布的 v0.1.0 就地修正（移除 server 資產、SHA256SUMS.txt 同步、補掛下載指南），不重發 tag
- Homebrew tap 自動推送：release 完成後由管線以 formula 產生器輸出自動更新 tap repo 的 Formula/speclink.rb（跨 repo 憑證 TAP_PUSH_TOKEN 存在才啟用、缺席跳過）；tap repo 建立與 fine-grained PAT 設定以逐步教學的手動任務落地
- server 新增 npm 通路（npx 一行啟動）：esbuild 式主套件＋五平台子套件（os/cpu 對應、optionalDependencies 只裝對應平台），launcher 零參數啟動時仿 compose 模式把環境變數（store 選 sqlite／serverfs／postgres、資料目錄、public_url／port）插值成組態 YAML 落地後帶 --config 起 server——Rust 端不改、fail-closed 契約不動；發布由 release 管線於 NPM_TOKEN 存在時執行，npm 帳號／scope／token 為手動教學任務
- 首發演練：確認版號同版、推 v0.1.0 tag、驗證 Release 產物（含簽章與公證結果）與各平台安裝實測

## Capabilities

### New Capabilities

- `cli-distribution`: CLI 的安裝通路——安裝腳本（sh／PowerShell）的平台偵測、checksum 驗證與安裝行為、Homebrew formula 產生器的輸出契約，以及 release 後 formula 自動推送 tap 的管線契約

### Modified Capabilities

- `desktop-release`: 「OS 程式碼簽章為可插鑰匙開關」requirement 擴充——macOS 簽章 secrets 存在時同時執行公證；Windows 維持本機憑證路徑與未簽章後備（SignPath 裁定不採用，見設計 D2）；開關語意與 updater 簽章正交性維持不變。「Release 產出三平台桌面安裝檔」requirement 修改——assets 不再含 server 壓縮檔、NSIS 安裝器內建繁中英文語系與 Speclink 圖示；新增「Release 說明含下載指南」requirement——release job 產生各平台下載對照表前置於 Release 說明
- `desktop-app`: 「安裝 CLI 指令到 PATH」requirement 修改——macOS 啟動自動佈署與自我修復、PATH 冪等追加至 ~/.zprofile；Windows／Linux 行為不變
- `server-release`: 「release 產物含 server 與部署文件」requirement 修改——server binary 仍建置與冒煙但不打包上 Release，官方發布通路為 Docker image 與 npm 套件；新增「npm 套件一行啟動 server」requirement——平台子套件解析、環境變數插值組態的快速啟動契約；部署文件的 native binary 形態改教從原始碼建置
- `user-documentation`: 新增安裝通路文件 requirement——README 安裝區塊（中英對等）、sdk-node 發布狀態誠實化
- `delivery-baseline`: 「CI 執行完整測試」requirement 擴充——測試面納入 `scripts`，並要求其執行方式相容於 workflow 釘選的 Node 版本與三平台預設 shell

## Impact

- Affected specs: `cli-distribution`（新增）、`desktop-release`、`user-documentation`、`delivery-baseline`、`server-release`、`desktop-app`
- Affected code:
  - New: `scripts/signing-gate.mjs`、`scripts/signing-gate.test.mjs`、`scripts/install.sh`、`scripts/install.ps1`、`scripts/install.test.mjs`、`scripts/homebrew-formula.mjs`、`scripts/homebrew-formula.test.mjs`、`scripts/release-notes.mjs`、`scripts/release-notes.test.mjs`、`packages/server-npm/package.json`、`packages/server-npm/bin/speclink-server.mjs`、`scripts/npm-server-package.mjs`、`scripts/npm-server-package.test.mjs`、`scripts/npm-server-launcher.test.mjs`
  - Modified: `.github/workflows/release.yml`、`.github/workflows/ci.yml`、`scripts/delivery-gate.test.mjs`、`crates/speclink-server/src/assets.rs`、`crates/speclink-server/tests/it/web_assets.rs`、`crates/speclink-server/Dockerfile`、`apps/desktop/src-tauri/tauri.conf.json`、`apps/desktop/src/core/cliInstall.ts`、`apps/desktop/src/adapter/cliInstall.ts`、`apps/desktop/src/__tests__/cliInstall.test.ts`、`README.md`、`README.en.md`、`docs/getting-started.zh-TW.md`、`docs/getting-started.md`、`docs/sdk-node.zh-TW.md`、`docs/sdk-node.md`、`docs/product-status.zh-TW.md`、`docs/server-deployment.zh-TW.md`
  - Removed: 無
- repo 之外（不產生本 repo 檔案）：新建 GitHub tap repo（homebrew-tap，含 formula）；GitHub Actions secrets 新增 Apple 公證三項、TAP_PUSH_TOKEN（tap 推送 PAT）與 NPM_TOKEN（npm 發布）；npm 帳號與 scope（偏好 @speclink，占用時採替代並記錄）——皆以手動教學任務執行

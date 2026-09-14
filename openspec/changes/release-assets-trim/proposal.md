## Why

v0.4.0 的 GitHub Release 頁有 21 個 assets，其中只有 7 個是給人下載的安裝檔；GitHub 依檔名排序（不分大小寫、`-` 排在 `_` 前），訪客看到的前 7 項是 latest.json、SHA256SUMS.txt 與 5 個 CLI 壓縮檔，第一個 dmg 排在第 12 位。第一次來下載的人（透過 AI 代理跑 SDD 的開發者、要看看板的 PO／PM）分不出該抓哪一個。GitHub 沒有「隱藏 asset」功能，唯一解法是讓機器讀的檔案離開這一頁——同時把 CLI 的安裝通路收斂到 npm 這個單一來源。本變更落在 `/release` 技能觸發的發版管線與安裝文件，對應使用情境是「首次安裝」與「版本升級」。

出處：討論 release-assets-trim（2026-09-14 結論，一個 change 做完）。

## What Changes

Release 頁從 21 個檔收成 6 個：universal dmg、universal .app.tar.gz、x64 setup.exe、AppImage×2（x86_64／aarch64）、latest.json。

- **.sig 不再上傳 Release**（5 個檔）：Tauri updater 只讀 latest.json 內嵌的 signature 內容，發布後沒有任何程式再讀 .sig 檔。建置仍產出 .sig，只供組裝 latest.json。
- **macOS 改為 universal 建置**：`tauri build --target universal-apple-darwin` 產出一個同時跑 Apple Silicon 與 Intel 的 dmg 與 .app.tar.gz；latest.json 的 darwin-aarch64 與 darwin-x86_64 兩鍵指向同一個更新包與同一份簽章；CLI sidecar 以 lipo 合成 universal binary（bundler 要求檔名 speclink-universal-apple-darwin）。
- **BREAKING：.deb 停止發布**：deb 只服務 Debian 家族的桌面機，不服務無圖形介面的 Linux（它裝的是桌面 app），且 deb 版 app 的自動更新在現況下會失敗（updater 依包型態查 linux-x86_64-deb 鍵，退回 linux-x86_64 後下載到 AppImage 而報 InvalidUpdaterFormat）。桌面 app 的 linux-deb 分支一併清除。
- **CLI 上 npm 成為唯一 binary 來源**：新增 @speclink/cli 主套件與五個平台子套件（optionalDependencies，比照 @speclink/server 的物化與發布流程）；主套件的 bin 是 JS shim，postinstall 在 macOS／Linux 以原生 binary 原地置換 shim（esbuild 做法），Windows 與 --ignore-scripts 安裝時 shim 以 spawn 執行平台 binary。五個 target 的 CLI 壓縮檔與 SHA256SUMS.txt 離開 Release 頁。
- **Homebrew 留、改抓 npm**：formula 的 url 改指 registry.npmjs.org 的平台子套件 tgz（homebrew-core 對 Node CLI 的慣例），sha256 由 CI 以 npm pack 產出的 tgz 計算；tap 推送改依賴 CLI 的 npm 發布結果——NPM_TOKEN 缺席時 tap 一併跳過。
- **install.sh 留、改抓 npm**：版本問 registry 的 @speclink/cli/latest，下載平台子套件 tgz，以 npm 的 sha512 integrity 驗證，解出 package/speclink 安裝到 ~/.local/bin；服務無圖形介面的 Linux 與 CI。**BREAKING：install.ps1 退役**——Windows 幾乎都有圖形介面，setup.exe 內含 CLI 並管 PATH，其餘走 npm。
- **下載指南與文件改寫**：對照表改為「macOS 一個檔、Windows 一個檔、Linux 桌面機 AppImage、Linux 伺服器 CLI 一行」；CLI 節列 npm、curl|sh、brew 三條；.sig／壓縮檔／SHA256SUMS 的註記移除，.app.tar.gz 與 latest.json 的「自動更新用」註記保留。

相容性影響（無 CLI 子指令與 --json 變動）：

| 面 | 變動 | 既有使用者怎麼遷移 |
| --- | --- | --- |
| Release assets | speclink-v*-<target>.tar.gz／.zip、SHA256SUMS.txt、*.sig、*.deb 消失；macOS 檔名由 aarch64／x64 改為 universal | 自動化抓 Release 壓縮檔的腳本改抓 npm registry 的 tgz，或改跑 install.sh |
| install.ps1 | 檔案刪除，raw 網址回 404 | Windows 改用 npm i -g @speclink/cli，或裝 setup.exe |
| install.sh | 網址不變；來源改 npm；SPECLINK_INSTALL_REPO 由 SPECLINK_INSTALL_REGISTRY 取代；SPECLINK_INSTALL_VERSION 接受帶或不帶 v 前綴 | 無動作；釘選 v0.4.0 以前的版本不再可裝（那些版本不在 npm） |
| Homebrew | formula url 換來源 | brew upgrade 照常 |
| deb 使用者 | 無新版 deb | sudo apt remove speclink 後改裝 AppImage（README 註明） |
| 既有 mac 使用者 | latest.json 的 darwin 兩鍵指向 universal 更新包 | 自動更新照常，更新後為 universal app |

## Capabilities

### New Capabilities

（none）——npm 通路是 CLI 安裝通路的一部分，以 ADDED requirement 落在既有的 `cli-distribution`；掃到的相鄰 specs 為 `desktop-release`（安裝檔與更新描述檔）、`cli-distribution`（安裝腳本、formula、tap）、`server-release`（npm 平台子套件的既有模式，本次不動）。

### Modified Capabilities

- `desktop-release`：「Release 產出三平台桌面安裝檔」改為六個 assets 的封閉集合（macOS universal、無 deb、無 .sig、無 CLI 壓縮檔、無 SHA256SUMS）；「更新描述檔隨 release 發布」改為 darwin 兩鍵共用 universal 更新包；「Release 說明含下載指南」改為新對照表與三條 CLI 指令、移除 .sig 註記。
- `cli-distribution`：「安裝腳本一行安裝對應平台 CLI」改為 POSIX sh 單支、來源 npm registry、sha512 驗證；「Homebrew formula 產生器」改為 url 指 npm tgz、sha256 由 tgz 計算；「Formula 隨發版自動推送 tap」改為依賴 CLI npm 發布結果；ADDED「CLI 以 npm 套件發布」。
- `desktop-app`：「安裝 CLI 指令到 PATH」移除 Linux deb 佈署 /usr/bin 的分句。
- `user-documentation`：「安裝通路文件與發布狀態誠實化」的通路清單改為 npm／install.sh／Homebrew 與三平台安裝檔（無 deb）；「安裝章節載明桌面 app 與 CLI 的佈署衝突」的逐平台表移除 deb 行。

## Impact

- Affected specs：desktop-release、cli-distribution、desktop-app、user-documentation（皆 delta）。
- Affected apps／crates：apps/desktop（Tauri 殼 cli_install.rs 與 core cliInstall.ts 的 Linux 分流）；Rust workspace 其餘 crate 不動。
- Affected code：
  - New：
    - packages/cli-npm/package.json
    - packages/cli-npm/bin/speclink
    - packages/cli-npm/postinstall.mjs
    - packages/cli-npm/platform.mjs
    - scripts/npm/npm-cli-package.mjs
    - scripts/npm/npm-cli-package.test.mjs
    - scripts/npm/npm-cli-launcher.test.mjs
    - scripts/npm/npm-platform-package.mjs（CLI 與 server 共用的「主套件＋五平台子套件」物化規則；品質關卡 Round 1 的重複程式碼發現）
    - .github/workflows/npm-publish.yml（三條 npm 通路共用的 workflow_call 發布 workflow；同上）
  - Modified：
    - .github/workflows/release.yml
    - scripts/npm/npm-server-package.mjs（改為呼叫共用物化規則，參數面不變）
    - packages/server-npm/package.json（補 files 清單，物化時據此帶檔）
    - scripts/release/release-latest-json.mjs
    - scripts/release/release-latest-json.test.mjs
    - scripts/release/release-notes.mjs
    - scripts/release/release-notes.test.mjs
    - scripts/release/homebrew-formula.mjs
    - scripts/release/homebrew-formula.test.mjs
    - scripts/release/delivery-gate.test.mjs
    - scripts/install.sh
    - scripts/install.test.mjs
    - scripts/desktop/desktop-sidecar.mjs
    - scripts/desktop/desktop-sidecar.test.mjs
    - apps/desktop/src-tauri/src/cli_install.rs
    - apps/desktop/src/core/cliInstall.ts
    - apps/desktop/src/__tests__/cliInstall.test.ts
    - apps/desktop/src/views/AppSettingsView.tsx
    - apps/desktop/src/i18n/messages.ts
    - README.md
    - README.en.md
    - docs/getting-started.md
    - docs/getting-started.zh-TW.md
    - docs/development.md
    - docs/development.zh-TW.md
    - docs/product-status.md
    - docs/product-status.zh-TW.md
  - Removed：
    - scripts/install.ps1
- Dependencies：無新 Rust 或 npm 相依；CI 端新增 lipo（Xcode 內建）與 npm pack 的使用。
- 不在範圍：openspec/manual/ 的手冊頁（由 /speclink-manual 依規格重生）；brew cask 裝桌面 app；CI 專用 setup action。

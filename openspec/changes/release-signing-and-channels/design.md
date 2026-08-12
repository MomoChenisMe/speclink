## Context

release 管線（`.github/workflows/release.yml`）已於 push v* tag 時產出五形態 artifact，桌面建置有兩層簽章：updater 更新包的 minisign 簽章（必要，缺私鑰 fail-closed）與 OS 程式碼簽章（env-gated 骨架，secrets 缺席即跳過）。現況缺口：macOS 只有簽章接線、沒有公證（notarization）——已簽章但未公證的 app 在新版 macOS 仍被 Gatekeeper 攔；Windows 的骨架假設本機 PFX 憑證，而裁定採用的 SignPath 開源簽章是雲端送簽服務，接法不同；CLI 沒有安裝腳本與 Homebrew 通路。討論 release-first-and-distribution 已裁定首發（v0.1.0）就要完成兩平台簽章與安裝通路，帳號申請類步驟以手動教學任務落地。

## Goals / Non-Goals

**Goals:**

- macOS 安裝檔在簽章 secrets 齊備時自動完成 Developer ID 簽章＋公證，使用者下載後直接開啟、無 Gatekeeper 攔阻
- Windows 安裝檔在 SignPath secrets 齊備時經 SignPath 服務簽章；未過件前維持未簽章路徑，不阻塞發版
- 使用者能以一行指令安裝 CLI（安裝腳本或 Homebrew tap），並能從 README 快速找到三平台桌面安裝檔
- 文件不再宣稱不存在的入口（sdk-node 的 npm install 說明改標尚未發布）

**Non-Goals:**

- npm 發布 @speclink/engine 或 CLI 的 npm 通路（留待 v0.2 與 engine 一起規劃）
- homebrew-core、winget、scoop、AUR 等需知名度或審核的商店通路
- release 管線自動更新 tap repo 的 formula（首發手動跑產生器；自動化留待後續）
- 桌面 app 的 Homebrew cask
- updater 金鑰體系與 latest.json 組裝邏輯（不動）
- server 交付形態（Docker 等，已存在且不變）

## Decisions

### D1：macOS 公證接在既有簽章開關上，部分設定 fail-closed

公證所需三項 secrets（Apple ID、App 專用密碼、Team ID）與既有憑證三項 secrets 視為同一組能力的兩半：

- 憑證 secrets 全缺 → 跳過簽章與公證，workflow 照常全綠（維持現行行為）
- 憑證 secrets 存在且公證 secrets 齊備 → Tauri 建置時簽章並公證（Tauri v2 bundler 偵測到公證環境變數即自動執行公證與 staple）
- 憑證 secrets 存在但公證 secrets 不全 → workflow 以明確錯誤失敗並列出缺項（fail-closed）

理由：「已簽章但未公證」的產物對使用者與未簽章幾乎等價（照樣被攔），卻讓維護者誤以為已完成——這是設定錯誤，不是合法中間態，與專案「壞設定不得靜默降級」的既有原則一致。替代方案「警告後繼續產出半套」被否決。

閘門邏輯抽成可獨立執行的 `scripts/signing-gate.mjs`（沿用 `scripts/release-latest-json.mjs` 把 workflow 邏輯外置以取得單元測試的既有作法）：讀各 secret 是否非空、絕不讀取或輸出其值，把決策 `SPECLINK_MACOS_SIGNING`（full／none）與 `SPECLINK_WINDOWS_SIGNING`（signpath／certificate／none）寫入 `GITHUB_ENV` 供後續步驟判斷，兩處不再各自推導；部分存在時不寫出任何決策即非零結束，下游無從沿用半套設定。內嵌於 workflow 的 bash 條件無法以三種 secrets 組合實測，故不採。

### D2：Windows SignPath 以 signCommand 接入，CI 專用 config overlay，PFX 路徑保留為後備

- Tauri 的 bundle.windows.signCommand 指向 repo 內的送簽腳本：收到待簽檔路徑後，呼叫 SignPath API 建立 signing request、等候完成、以簽回的檔案原地覆蓋
- signCommand 不寫入 `apps/desktop/src-tauri/tauri.conf.json` 主設定檔，而是 CI 在 SignPath secrets 齊備時以 tauri build 的 config 合併參數注入——本機與開發建置完全不受影響
- 優先序：SignPath secrets 齊備 → SignPath 路徑；否則 WINDOWS_CERTIFICATE 存在 → 現行 PFX 骨架；否則未簽章。SignPath secrets 不全（部分存在）比照 D1 fail-closed
- 簽章順序保證：signCommand 在 Tauri 打包期間執行，updater 的 minisign 簽章在打包完成後產生，因此 .sig 涵蓋的是已簽章的安裝檔，latest.json 的簽章驗證不會失效

理由：signCommand 是 Tauri 官方支援雲端簽章服務的縫；config overlay 讓「簽章屬 CI 發版能力、非開發環境義務」的邊界乾淨。SignPath 需 OSS 申請過件（OSI 授權、CI 建置來源），過件時程不可控，故未過件路徑必須照常可發版。

### D3：安裝腳本以 dry-run 與環境變數覆寫換取可測性

`scripts/install.sh`（macOS／Linux，POSIX sh 相容）與 `scripts/install.ps1`（Windows PowerShell）行為契約相同：

1. 偵測 OS 與 CPU 架構，對映到 release 資產的 target triple 命名（與 release.yml 的五 target 一致）
2. 解析最新（或指定）版本：以 GitHub Releases API 取 latest tag；SPECLINK_INSTALL_VERSION 可釘選版本
3. 下載對應壓縮檔與 SHA256SUMS.txt，驗證 checksum，不符即失敗且不落檔
4. 解壓 speclink binary 至安裝目錄：Unix 預設 ~/.local/bin、Windows 預設使用者層級程式目錄；SPECLINK_INSTALL_DIR 可覆寫；安裝後偵測 PATH 是否涵蓋並提示
5. --dry-run 只輸出解析結果（target、資產 URL、安裝目錄）不碰網路不落檔——單元測試以此驗證平台對映矩陣與 URL 組裝，不依賴真實 Release

測試放 `scripts/install.test.mjs`（沿用 repo 既有 node --test 腳本測試慣例）；ps1 的測試在無 pwsh 的環境跳過（CI 的 Windows runner 會跑到）。

### D4：Homebrew formula 由腳本產生，tap repo 手動維護

`scripts/homebrew-formula.mjs` 讀入 tag 與該版 SHA256SUMS.txt，輸出完整 formula（on_macos／on_linux × arm64／x86_64 四組 url＋sha256，指向 GitHub Release 資產），stdout 即成品；測試以 fixture checksums 驗證輸出結構與四組對應。tap repo（GitHub 上另建 homebrew-tap）由手動任務建立、首發後把產生器輸出貼入 Formula 目錄。理由：checksums 每版都變，手抄必錯；但 tap 自動推送牽涉跨 repo 憑證，首發不做（Non-Goal）。

### D5：文件的「誠實入口」原則落到安裝面

README（中英）安裝區塊：桌面三平台下載表（連到 Releases 頁與 latest.json 說明）、CLI 一行安裝（腳本與 brew tap 指令）、從原始碼建置降為開發者段落；getting-started（中英）安裝節同步。sdk-node（中英）把 npm install 段改為「尚未發布至 npm」並改教 repo 內建置載入路徑。product-status 查核日刷新。中英兩語結構與事實對等（user-documentation 既有 requirement 的延伸）。

## Implementation Contract

**Behavior（發版後可觀察）：**

- 六項 Apple secrets（憑證半組與公證半組）齊備時，push tag 產出的 dmg 內 app 通過 Gatekeeper 評估（spctl 評估通過、公證票證已 staple），使用者雙擊即開
- SignPath secrets 齊備時，NSIS 安裝檔帶有效 Authenticode 簽章（簽發者為 SignPath Foundation）；secrets 全缺時產物與現況一致
- 任一簽章 secrets 組「部分存在」時 workflow 紅燈，錯誤訊息列出缺項
- curl 安裝腳本一行完成後，speclink --version 輸出該 Release 版號；checksum 不符時腳本非零退出且安裝目錄無殘留
- brew install 自有 tap 的 speclink 後，speclink --version 同上
- README 首屏可見安裝區塊；sdk-node 文件不再指示 npm install @speclink/engine

**Interface：**

- scripts/install.sh 與 scripts/install.ps1：支援 --dry-run；環境變數 SPECLINK_INSTALL_VERSION、SPECLINK_INSTALL_DIR、SPECLINK_INSTALL_REPO（測試用 repo 覆寫）
- scripts/homebrew-formula.mjs：參數 --tag 與 --sums（SHA256SUMS.txt 路徑），formula 輸出至 stdout
- release.yml 新增 secrets 契約：APPLE_ID、APPLE_PASSWORD、APPLE_TEAM_ID（公證）；SIGNPATH_API_TOKEN、SIGNPATH_ORGANIZATION_ID、SIGNPATH_PROJECT_SLUG、SIGNPATH_POLICY_SLUG（Windows 簽章）

**Verification：**

- scripts 測試：node --test 跑 install.test.mjs 與 homebrew-formula.test.mjs（dry-run 平台矩陣、URL 組裝、checksum 失敗路徑、formula 四組對應）
- 管線與簽章：首發 v0.1.0 的真實 workflow 執行即端到端驗證（手動任務含產物驗證步驟：macOS 以 spctl 與 stapler 驗證、Windows 檢查簽章有效性、Linux AppImage 可執行）
- 文件：中英對等以 user-documentation 既有查核清單覆蓋

**風險與緩解：**

- Apple Developer 審核需 1～2 天、公證首跑常因 hardened runtime 或 sidecar 簽章細節失敗——教學任務附公證失敗排查步驟（讀公證 log、確認 sidecar CLI 與 app 同鏈簽章）
- SignPath 過件不可控——D2 的後備路徑保證不阻塞；過件後補插 secrets 即生效，無管線改動

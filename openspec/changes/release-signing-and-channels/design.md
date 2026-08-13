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

### D6：server 發布通路收斂為 Docker，release 保留 server 建置與冒煙作品質閘門

v0.1.0 首發後裁定 server 壓縮檔退出 Release assets：server 的使用情境是部署（Docker 直跑或 compose），不是桌面使用者手動下載 binary，五個 server 壓縮檔只是加深 assets 清單的混亂。release 管線的 build job 仍建置各平台 speclink-server 並執行無 dist 冒煙（/login 回 HTML、未知 browser API 回 JSON 404）——這個閘門在首發實跑抓到 JSON 404 規格違反，證明其價值，與是否上傳無關；Package 步驟改為只打包 speclink CLI。捨棄的替代：連建置都拿掉（省 CI 時間但失去五 target 的 release-profile 驗證面，docker job 只涵蓋 linux x86_64）。

### D7：Release 下載指南由腳本產生前置於說明，v0.1.0 就地修正不重發 tag

`scripts/release-notes.mjs` 讀 tag、輸出下載指南 markdown 至 stdout：三平台安裝檔對照表（檔名含版號、與資產命名一致）、CLI 一行安裝與 Homebrew 指令、註明 `.app.tar.gz` 與 `.sig` 為自動更新機制使用毋須手動下載。release job 產出 notes 檔後以 body_path 傳入 action-gh-release，`generate_release_notes` 保留使自動 changelog 接續其後。已發布的 v0.1.0 屬資產層修正（刪 server 資產、SHA256SUMS.txt 過濾重傳、說明補掛指南），tag 與 commit 皆不動——重發 tag 會重跑全部建置且對使用者無感，唯一代價是舊 SHA256SUMS.txt 曾短暫含 server 條目，可接受。

### D8：Homebrew tap 由 release 管線自動推送，單一 secret 為開關

release 成功後的 tap-publish job（needs: release）於 `TAP_PUSH_TOKEN` 存在時執行：從 dist 取 SHA256SUMS.txt、以 formula 產生器輸出內容、經 GitHub Contents API 直接更新 tap repo 的 `Formula/speclink.rb`（單檔 PUT，毋須 clone；commit 訊息含版號）。secret 缺席時整個 job 跳過且不影響 Release 結果；存在而推送失敗時 job 紅燈（存在即必須成功，與簽章開關同語意）。憑證用 fine-grained PAT：只授權 homebrew-tap 一個 repo 的 Contents 讀寫，洩漏面最小。channel job 放在 Release 建立之後：Release 是產物的真相源，通路推送失敗可單獨重跑，不反過來擋發布。這推翻 D4 的「tap 手動維護」Non-Goal——首發已完成，跨 repo 憑證的顧慮以最小權限 PAT 解決；formula 首版仍由手動任務貼入（tap repo 建立時 v0.1.0 已發布，管線只在下一個 tag 才跑）。

### D9：server 的 npm 通路——esbuild 式平台套件，launcher 插值組態、Rust 不動

**套件形狀**：主套件（偏好 `@speclink/server`，scope 占用時採替代並記錄）帶 bin launcher 與五個平台子套件的 optionalDependencies；每個子套件以 `os`／`cpu` 欄位圈平台、內容物只有對應平台的 speclink-server binary——npm 安裝時只會下載符合平台的那個（esbuild／turbo 的成熟模式）。平台子套件由 `scripts/npm-server-package.mjs` 於發布時從 build artifacts 物化產生，repo 內只維護主套件與產生器。

**啟動語意（核心決策）**：server binary 只吃單一 YAML 且 fail-closed（reference-server 契約），compose 已示範正確外掛法——外層把環境變數插值成 YAML。launcher 照抄：零參數（或僅環境變數）啟動時，依 `SPECLINK_STORE`（sqlite 預設／serverfs／postgres）、`SPECLINK_DATA_DIR`（預設 ./speclink-data）、`SPECLINK_PUBLIC_URL`／`SPECLINK_PORT`（連動預設同 compose：public_url 預設 http://localhost:PORT）產生組態 YAML 寫入資料目錄，再帶 --config spawn binary；postgres 時 `SPECLINK_POSTGRES_URL` 必填、缺席即非零退出點名缺項（密碼可拆 `SPECLINK_POSTGRES_PASSWORD`，binary 原生支援）。使用者自帶 --config、設 `SPECLINK_CONFIG` 或使用子命令（invite、backup 等）時 launcher 純透傳，不產生任何組態。Rust 端零改動，fail-closed 契約原封不動——組態永遠是一份落地可檢視的檔案，不是散在環境裡的隱形狀態。

**發布**：npm-publish job（needs: release）於 `NPM_TOKEN` 存在時執行，下載五平台 server binary artifacts、物化套件、`npm publish --access public`，版本＝tag；secret 缺席跳過。捨棄的替代：教 server binary 直接讀環境變數（動 Rust、破壞 fail-closed 的單一組態來源）；只發主套件內含五平台 binary（安裝體積五倍）。

### D10：docker 多架構改原生 runner 分開建＋manifest 合併

QEMU 模擬下的 Rust 編譯讓 docker job 單跑 35 分鐘以上（v0.1.2 實測），且 QEMU 正是 SIGILL flaky 的根源——web 階段釘 $BUILDPLATFORM（任務 2.7）只救了 Node 半邊，Rust 編譯仍在模擬器裡。GitHub 公開 repo 的 arm64 runner 免費、本 repo 的 build matrix 已在用（ubuntu-24.04-arm）：docker 建置拆成 per-arch 兩個 job——ubuntu-latest 原生建 linux/amd64、ubuntu-24.04-arm 原生建 linux/arm64，平行執行、push by digest——再由合併 job 以 docker buildx imagetools create 把兩個 digest 併成版本 tag 與 latest。映像內容與單 job 多平台建置等價（同 Dockerfile、同 source revision）；發布原子性由合併 job 承擔：兩個 digest 都成功才打 tag，等價於現行全有全無語意。捨棄的替代：Dockerfile 內交叉編譯（musl 交叉工具鏈維護面大、單機序跑仍慢）；layer cache（lockfile 一動即失效、Rust 要配 cargo-chef 才有感，效益不穩）。

## Implementation Contract

**Behavior（發版後可觀察）：**

- 六項 Apple secrets（憑證半組與公證半組）齊備時，push tag 產出的 dmg 內 app 通過 Gatekeeper 評估（spctl 評估通過、公證票證已 staple），使用者雙擊即開
- SignPath secrets 齊備時，NSIS 安裝檔帶有效 Authenticode 簽章（簽發者為 SignPath Foundation）；secrets 全缺時產物與現況一致
- 任一簽章 secrets 組「部分存在」時 workflow 紅燈，錯誤訊息列出缺項
- curl 安裝腳本一行完成後，speclink --version 輸出該 Release 版號；checksum 不符時腳本非零退出且安裝目錄無殘留
- brew install 自有 tap 的 speclink 後，speclink --version 同上
- README 首屏可見安裝區塊；sdk-node 文件不再指示 npm install @speclink/engine
- Release assets 不含任何 speclink-server-* 壓縮檔；SHA256SUMS.txt 無 server 條目；Release 說明開頭為下載指南對照表，自動 changelog 接續其後
- release 管線的 build job 仍對五 target 建置 server 並通過無 dist 冒煙，失敗即整體不發布（全有全無閘門不變）
- TAP_PUSH_TOKEN 存在時，發版後 tap repo 的 Formula/speclink.rb 自動更新為該版（brew 使用者直接拿到新版）；缺席時發版不受影響
- NPM_TOKEN 存在時，發版後 npx @speclink/server（或替代 scope）以 sqlite 預設一行啟動：資料目錄與組態 YAML 落地、setup token 印出；SPECLINK_STORE=postgres 而缺 SPECLINK_POSTGRES_URL 時非零退出點名缺項；自帶 --config 或子命令時行為與直接執行 binary 一致

**Interface：**

- scripts/install.sh 與 scripts/install.ps1：支援 --dry-run；環境變數 SPECLINK_INSTALL_VERSION、SPECLINK_INSTALL_DIR、SPECLINK_INSTALL_REPO（測試用 repo 覆寫）
- scripts/homebrew-formula.mjs：參數 --tag 與 --sums（SHA256SUMS.txt 路徑），formula 輸出至 stdout
- scripts/release-notes.mjs：參數 --tag（vX.Y.Z 格式，不符即非零退出），下載指南 markdown 輸出至 stdout；release job 落檔後以 body_path 傳入 action-gh-release
- scripts/npm-server-package.mjs：參數 --version、--binaries（五平台 binary 目錄）、--scope（預設 @speclink）、--out；物化主套件與五平台子套件目錄（name／version／os／cpu／optionalDependencies／bin 欄位齊備）
- packages/server-npm/bin/speclink-server.mjs（launcher）：環境變數 SPECLINK_STORE、SPECLINK_DATA_DIR、SPECLINK_PUBLIC_URL、SPECLINK_PORT、SPECLINK_POSTGRES_URL、SPECLINK_CONFIG；參數與 exit code 對 binary 透傳
- release.yml 新增 secrets 契約：APPLE_ID、APPLE_PASSWORD、APPLE_TEAM_ID（公證）；SIGNPATH_API_TOKEN、SIGNPATH_ORGANIZATION_ID、SIGNPATH_PROJECT_SLUG、SIGNPATH_POLICY_SLUG（Windows 簽章）；TAP_PUSH_TOKEN（tap 推送）；NPM_TOKEN（npm 發布）

**Verification：**

- scripts 測試：node --test 跑 install.test.mjs 與 homebrew-formula.test.mjs（dry-run 平台矩陣、URL 組裝、checksum 失敗路徑、formula 四組對應）；release-notes.test.mjs（檔名對齊版號、.sig 註記、tag 格式 fail-closed）；npm-server-launcher.test.mjs（平台對映、組態插值、透傳、postgres 缺 URL fail-closed）；npm-server-package.test.mjs（套件欄位物化）；delivery-gate.test.mjs 契約斷言 release.yml 的 Package 步驟不含 server、release job 帶 body_path、tap-publish 與 npm-publish job 各自 gated 於對應 secret 且 Release assets 排除 server artifacts
- 管線與簽章：首發 v0.1.0 的真實 workflow 執行即端到端驗證（手動任務含產物驗證步驟：macOS 以 spctl 與 stapler 驗證、Windows 檢查簽章有效性、Linux AppImage 可執行）
- 文件：中英對等以 user-documentation 既有查核清單覆蓋

**風險與緩解：**

- Apple Developer 審核需 1～2 天、公證首跑常因 hardened runtime 或 sidecar 簽章細節失敗——教學任務附公證失敗排查步驟（讀公證 log、確認 sidecar CLI 與 app 同鏈簽章）
- SignPath 過件不可控——D2 的後備路徑保證不阻塞；過件後補插 secrets 即生效，無管線改動

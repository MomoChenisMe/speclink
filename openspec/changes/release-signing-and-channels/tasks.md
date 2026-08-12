## 1. Apple 簽章與公證的帳號準備（逐步教學）

- [x] [M] 1.1 註冊 Apple Developer Program：用你的 Apple ID 到 developer.apple.com/programs 點 Enroll，選 Individual（個人）、完成 US$99/年 付款；審核通常 1～2 個工作天。通過後到帳號的 Membership details 頁記下 Team ID（10 碼英數字）。完成判準：Membership 頁顯示有效會籍與 Team ID <!-- speclink-task:tsk_01KZTBCFP4FRBRJ895JN423731 -->
- [x] [M] 1.2 建立並匯出 Developer ID Application 憑證：在 macOS 開「鑰匙圈存取」→憑證輔助程式→從憑證授權要求憑證，產生 CSR 檔（填 email、選「儲存到磁碟」）；到 developer.apple.com 的 Certificates 頁按＋，選 Developer ID Application，上傳 CSR、下載 .cer 並雙擊裝進鑰匙圈；在鑰匙圈找到該憑證（名稱形如 Developer ID Application: 你的名字 (TeamID)）、展開含私鑰，右鍵匯出為 .p12 並設一組匯出密碼。完成判準：本機有 .p12 檔，鑰匙圈顯示憑證與私鑰成對 <!-- speclink-task:tsk_01KZTBCFP4CKVGH37PG4153BZA -->
- [ ] [M] 1.3 產生 App 專用密碼並填入六項 GitHub secrets：到 account.apple.com→登入與安全性→App 專用密碼，產生一組（用途填 speclink notarize）；在終端機以 base64 -i 憑證檔名.p12 取得 base64 字串；到 GitHub repo Settings→Secrets and variables→Actions 建立：APPLE_CERTIFICATE（p12 的 base64）、APPLE_CERTIFICATE_PASSWORD（匯出密碼）、APPLE_SIGNING_IDENTITY（鑰匙圈裡的憑證全名）、APPLE_ID（Apple 帳號 email）、APPLE_PASSWORD（App 專用密碼）、APPLE_TEAM_ID（Team ID）。完成判準：Actions secrets 清單出現六項 <!-- speclink-task:tsk_01KZTBCFP44DYRN68JRV061D47 -->

## 2. release 管線：公證接線與簽章組閘門

- [ ] 2.1 在 release.yml 的 desktop job 落實 spec「OS 程式碼簽章為可插鑰匙開關」與設計「D1：macOS 公證接在既有簽章開關上，部分設定 fail-closed」——簽章組閘門與公證環境注入：macOS 憑證半組與公證半組六項齊備時，於 Tauri 建置步驟注入公證三項環境變數（Tauri 簽章後自動公證並 staple）；任一平台簽章組「部分存在」時，前置檢查步驟以列出缺項 secrets 名稱的錯誤讓 job 失敗；全缺時維持現行跳過且全綠。驗證：以 bash 條件模擬三種 secrets 組合（全有、全無、缺一）的檢查邏輯正確，workflow 通過 repo 既有的 CI workflow 檢查 <!-- speclink-task:tsk_01KZTBCFP4PHJXM6YB7XKX212Z -->

## 3. Windows SignPath（教學＋接線）

- [ ] [M] 3.1 申請 SignPath 開源簽章並決定首發路徑：到 signpath.org 的 Open Source Code Signing 頁提交申請（填 GitHub repo URL、開源授權、說明由 GitHub Actions 建置）；過件後在 SignPath 後台建立 project 與 signing policy（release-signing），記下 organization ID、project slug、policy slug，並產生 API token，四項填入 GitHub secrets：SIGNPATH_API_TOKEN、SIGNPATH_ORGANIZATION_ID、SIGNPATH_PROJECT_SLUG、SIGNPATH_POLICY_SLUG。完成判準（二擇一）：四項 secrets 已填入；或申請已送出但未過件，明確記錄首發走未簽章後備路徑（設計 D2），不阻塞後續任務 <!-- speclink-task:tsk_01KZTBCFP41VHVM72RT8A1G4JM -->
- [ ] 3.2 SignPath 送簽腳本與 CI config overlay，落實設計「D2：Windows SignPath 以 signCommand 接入，CI 專用 config overlay，PFX 路徑保留為後備」與 spec「OS 程式碼簽章為可插鑰匙開關」的 SignPath 路徑：新增 scripts/signpath-sign.ps1（參數為待簽檔路徑；讀四項環境變數，呼叫 SignPath 建立 signing request、等候完成、下載簽回檔案原地覆蓋；支援 dry-run 只驗參數與 payload 組裝）；release.yml 在 SignPath secrets 齊備時以 tauri build 的 config 合併參數把 bundle.windows.signCommand 指向該腳本，主設定檔 tauri.conf.json 不寫入 signCommand；scripts/signpath-sign.test.mjs 覆蓋參數驗證與 payload 組裝（無 pwsh 環境自動跳過）。驗證：無 SignPath secrets 時本機與 CI 的桌面建置行為不變；node --test scripts/signpath-sign.test.mjs 綠 <!-- speclink-task:tsk_01KZTBCFP45R6858W0DQH8T3A7 -->

## 4. CLI 安裝腳本（測試先行）

- [ ] 4.1 先寫 scripts/install.test.mjs 再實作 scripts/install.sh，落實 spec「安裝腳本一行安裝對應平台 CLI」與設計「D3：安裝腳本以 dry-run 與環境變數覆寫換取可測性」：測試覆蓋 dry-run 的五 target 平台對映矩陣與資產 URL 組裝、SPECLINK_INSTALL_VERSION 釘選、SPECLINK_INSTALL_DIR 覆寫、checksum 不符時非零退出且不落檔（以本地 fixture 檔模擬下載內容）。驗證：node --test scripts/install.test.mjs 全綠 <!-- speclink-task:tsk_01KZTBCFP48V3585R0EF1G9NT4 -->
- [ ] 4.2 補 PowerShell 版：在 scripts/install.test.mjs 增加 pwsh 案例（無 pwsh 環境自動跳過）再實作 scripts/install.ps1，行為契約與 install.sh 一致（dry-run、版本釘選、目錄覆寫、checksum 驗證、PATH 提示）。驗證：本機（若有 pwsh）與 Windows CI 測試綠 <!-- speclink-task:tsk_01KZTBCFP4JH6T70AREQB6EY0A -->

## 5. Homebrew formula 產生器（測試先行）

- [ ] 5.1 先寫 scripts/homebrew-formula.test.mjs 再實作 scripts/homebrew-formula.mjs，落實 spec「Homebrew formula 產生器」與設計「D4：Homebrew formula 由腳本產生，tap repo 手動維護」：輸入 --tag 與 --sums（SHA256SUMS.txt 路徑），stdout 輸出含 on_macos／on_linux × arm64／x86_64 四組 url＋sha256 的 formula；fixture 驗證四組對應正確、缺任一平台條目時非零退出並指出缺項。驗證：node --test scripts/homebrew-formula.test.mjs 全綠 <!-- speclink-task:tsk_01KZTBCFP47BQ62X51YNMZYGQ6 -->

## 6. 文件：安裝通路與誠實化

- [ ] 6.1 README（中英）新增安裝區塊並同步 getting-started（中英）安裝節，落實 spec「安裝通路文件與發布狀態誠實化」與設計「D5：文件的「誠實入口」原則落到安裝面」：桌面三平台下載表（macOS dmg／Windows 安裝器／Linux AppImage 與 deb，連到 GitHub Releases）、CLI 一行安裝（curl 安裝腳本與 brew install tap 指令）、cargo install 從原始碼建置移到開發者導向段落；中英結構與事實對等。驗證：對照 user-documentation 規格的安裝區塊 scenario 逐項自查 <!-- speclink-task:tsk_01KZTBCFP4P7SXKR3NV35FKQB2 -->
- [ ] 6.2 sdk-node（中英）改標尚未發布至 npm 並改教 repo 內建置載入路徑，全文移除 npm install @speclink/engine 指示；docs/product-status.zh-TW.md 刷新查核日期並在 Node N-API SDK 列的限制欄補「尚未發布至 npm」。驗證：兩語 sdk-node 全文搜尋無 npm install @speclink/engine 字樣 <!-- speclink-task:tsk_01KZTBCFP4REZEMG1Y2NM78MWB -->

## 7. 首發 v0.1.0 演練（逐步教學）

- [ ] [M] 7.1 版號確認與推 tag：確認四處版號均為 0.1.0——crates/speclink-cli/Cargo.toml、crates/speclink-core/Cargo.toml、apps/desktop/src-tauri/tauri.conf.json、根 package.json；本 change 全部程式碼任務已合入 main 後，在 main 上執行 git tag v0.1.0 與 git push origin v0.1.0。完成判準：GitHub Actions 的 Release workflow 被 tag 觸發並開始執行 <!-- speclink-task:tsk_01KZTBCFP4CEZ8AVKCN9VG45JA -->
- [ ] [M] 7.2 驗證 Release 產物與三平台安裝實測：Release assets 含五 target 的 CLI／server 壓縮檔、桌面安裝檔（dmg×2、setup.exe、AppImage×2、deb×2）、latest.json 與 SHA256SUMS.txt；macOS 下載 dmg 安裝後以 spctl 評估通過並確認雙擊可開（公證票證已 staple）；Windows 依 3.1 的路徑檢查：有簽章則檢視檔案內容的數位簽章有效、未簽章則確認 README 的 SmartScreen 放行說明可依循；Linux 確認 AppImage 可執行；再以 curl 安裝腳本實測 speclink --version 輸出 0.1.0。完成判準：上述逐項確認完成 <!-- speclink-task:tsk_01KZTBCFP4T4AW1PARCWKX3FZ0 -->
- [ ] [M] 7.3 建立 Homebrew tap 並實測：在 GitHub 建立名為 homebrew-tap 的公開 repo；下載 v0.1.0 的 SHA256SUMS.txt，執行 node scripts/homebrew-formula.mjs --tag v0.1.0 --sums 該檔路徑，輸出存為 tap repo 的 Formula/speclink.rb 並 push；以 brew install 自己帳號/tap/speclink 實測。完成判準：brew 安裝後 speclink --version 輸出 0.1.0 <!-- speclink-task:tsk_01KZTBCFP4HJXJ06QTKPWQH7RQ -->

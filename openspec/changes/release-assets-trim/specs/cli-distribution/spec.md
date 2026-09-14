## ADDED Requirements

### Requirement: CLI 以 npm 套件發布

release 管線 SHALL 於 GitHub Release 建立成功後，在 NPM_TOKEN 存在時發布 CLI 的 npm 套件：主套件 @speclink/cli（bin 為 speclink）與五個平台子套件 @speclink/cli-darwin-arm64、@speclink/cli-darwin-x64、@speclink/cli-linux-x64、@speclink/cli-linux-arm64、@speclink/cli-win32-x64（各只含對應平台的 speclink binary，以 os 與 cpu 欄位圈定），版本等於 tag 去 v 前綴，主套件以 optionalDependencies 引用五個同版子套件。套件 SHALL 從 build artifacts 物化，缺任一平台 binary 時 SHALL 以非零結束且不產出任何套件；每個套件發布前 SHALL 查 registry，同版已存在即跳過；平台子套件 SHALL 先於主套件發布；發布後 SHALL 等待主套件在 registry 可見才以 0 結束。NPM_TOKEN 缺席時 SHALL 跳過發布且 job 綠。

主套件的 speclink 入口 SHALL 為 JS shim：解析本機 os 與 cpu 對應的子套件並以繼承 stdio 的方式執行其 binary，exit code 與訊號原樣帶回；找不到對應子套件時 SHALL 於 stderr 說明該 os 與 cpu 組合不受支援並以非零結束。macOS 與 Linux 上安裝時 postinstall SHALL 以該平台的原生 binary 原地置換 shim 檔，使全域 speclink 直接是原生執行檔；Windows 上與以 --ignore-scripts 安裝時 SHALL 保留 shim 路徑；postinstall 找不到平台 binary 時 SHALL 保留 shim 並以 0 結束。

#### Scenario: 全域安裝後 speclink 為同版原生執行檔

- **WHEN** 在 macOS 或 Linux 執行 npm i -g @speclink/cli@0.5.0 後執行 speclink --version
- **THEN** 輸出的版本為 0.5.0，且 PATH 上解析到的 speclink 檔是原生執行檔而非 JS 檔

#### Scenario: 略過 postinstall 仍可執行

- **WHEN** 以 --ignore-scripts 安裝主套件後執行 speclink --version
- **THEN** shim 執行平台子套件內的 binary，輸出同版版本，exit code 為 0

#### Scenario: 不支援平台的 shim 明確失敗

- **WHEN** 在沒有對應平台子套件的 os 與 cpu 組合上執行 shim
- **THEN** stderr 說明該組合不受支援並列出支援的組合，exit code 非 0

#### Scenario: 缺平台 binary 不發布

- **WHEN** 物化腳本的輸入目錄缺任一 target 的 CLI binary
- **THEN** 腳本以非零結束並點名缺少的 target，輸出目錄不產生任何套件

#### Scenario: 重跑不因同版已上架而失敗

- **WHEN** 發布 job 重跑而部分套件的同版已在 registry
- **THEN** 已存在者跳過、其餘照常發布，job 以 0 結束

## MODIFIED Requirements

### Requirement: 安裝腳本一行安裝對應平台 CLI
<!-- BEFORE: sh 與 PowerShell 兩支；來源為 GitHub Release 壓縮檔與 SHA256SUMS.txt；以 GitHub Releases API 解析版本 -->

專案 SHALL 提供 POSIX sh 相容的安裝腳本（macOS 與 Linux），SHALL NOT 提供 PowerShell 安裝腳本（Windows 走 npm 全域安裝或桌面安裝器）。腳本 SHALL：偵測 OS 與 CPU 架構並對映到 npm 平台子套件名（darwin-arm64、darwin-x64、linux-x64、linux-arm64）；以 npm registry 上 @speclink/cli 的最新版解析版本（環境變數 SPECLINK_INSTALL_VERSION 可釘選，接受帶或不帶 v 前綴）；下載對應平台子套件該版的 tgz；取得 registry 回報的該版 sha512 integrity 並以本機計算值比對；驗證通過後解出 tgz 內的 speclink binary 至安裝目錄（預設 ~/.local/bin，SPECLINK_INSTALL_DIR 可覆寫）；安裝後檢查安裝目錄是否在 PATH 並於缺席時提示。registry 位址 SHALL 可由 SPECLINK_INSTALL_REGISTRY 覆寫，預設 https://registry.npmjs.org。integrity 不符或找不到計算工具時 SHALL 以非零結束且 SHALL NOT 留下任何已落檔的產物。在 Windows 的 shell 環境執行時 SHALL 以非零結束並指向 npm 與桌面安裝器。腳本 SHALL 支援 dry-run 模式：只輸出解析出的平台、版本、tgz URL 與安裝目錄，不發出網路請求、不寫入檔案；平台對映與 URL 組裝 SHALL 有以 dry-run 驗證的自動化測試。

#### Scenario: dry-run 輸出平台對映

- **WHEN** 在測試中以強制指定的 OS 與架構組合執行安裝腳本的 dry-run 模式
- **THEN** 輸出的平台名與 tgz URL 符合 registry 上該平台子套件的 tgz 路徑樣式，且過程無網路請求、無檔案寫入

##### Example: 四組平台對映（版本 0.5.0）

| uname -s／-m | 平台子套件 | tgz URL |
| ------------- | ---------- | ------- |
| Darwin／arm64 | @speclink/cli-darwin-arm64 | https://registry.npmjs.org/@speclink/cli-darwin-arm64/-/cli-darwin-arm64-0.5.0.tgz |
| Darwin／x86_64 | @speclink/cli-darwin-x64 | https://registry.npmjs.org/@speclink/cli-darwin-x64/-/cli-darwin-x64-0.5.0.tgz |
| Linux／x86_64 | @speclink/cli-linux-x64 | https://registry.npmjs.org/@speclink/cli-linux-x64/-/cli-linux-x64-0.5.0.tgz |
| Linux／aarch64 | @speclink/cli-linux-arm64 | https://registry.npmjs.org/@speclink/cli-linux-arm64/-/cli-linux-arm64-0.5.0.tgz |

#### Scenario: checksum 不符即中止

- **WHEN** 下載的 tgz 與 registry 回報的 sha512 integrity 不一致
- **THEN** 腳本以非零結束、錯誤訊息指出 integrity 不符，安裝目錄無新增或被覆寫的檔案

#### Scenario: 安裝完成後版本可驗

- **WHEN** 在支援平台上執行安裝腳本安裝指定版本
- **THEN** 安裝目錄出現 speclink 可執行檔，執行 speclink --version 輸出該版本號

#### Scenario: 釘選版本接受帶或不帶 v 前綴

- **WHEN** 分別以 SPECLINK_INSTALL_VERSION=v0.5.0 與 SPECLINK_INSTALL_VERSION=0.5.0 執行 dry-run
- **THEN** 兩者輸出相同的 tgz URL，版本段皆為 0.5.0

### Requirement: Homebrew formula 產生器
<!-- BEFORE: 輸入 SHA256SUMS.txt，四組 url 指向 GitHub Release 的 CLI 壓縮檔 -->

專案 SHALL 提供 formula 產生器腳本：輸入 release tag 與 CLI npm 發布產出的 tgz 校驗清單（每行為 sha256、兩個空白、tgz 檔名），輸出完整的 Homebrew formula 至 stdout，內容 SHALL 含 macOS 與 Linux 各自的 arm64 與 x86_64 四組資產 URL 與對應 sha256，URL SHALL 指向 npm registry 上該版平台子套件的 tgz（樣式 https://registry.npmjs.org/@speclink/cli-<os>-<cpu>/-/cli-<os>-<cpu>-<版本>.tgz），SHALL NOT 引用 win32 子套件。校驗清單缺少四組中任一平台的條目時 SHALL 以非零結束並指出缺少的平台。產生器 SHALL 有以 fixture 校驗清單驗證輸出的自動化測試。

#### Scenario: 產出四組平台對應

- **WHEN** 以 fixture 的 tag 與含五個平台 tgz 條目的校驗清單執行產生器
- **THEN** stdout 的 formula 含 on_macos 與 on_linux 區塊、arm64 與 x86_64 四組指向 registry.npmjs.org 的 url 與 sha256，每組 sha256 等於 fixture 中對應條目，且不含 win32 條目

#### Scenario: checksums 缺項即失敗

- **WHEN** 校驗清單缺少 darwin-arm64 條目時執行產生器
- **THEN** 產生器以非零結束，錯誤訊息指出缺少該平台條目，stdout 無 formula 輸出

### Requirement: Formula 隨發版自動推送 tap
<!-- BEFORE: 以 Release 的 SHA256SUMS.txt 產 formula；閘門只有 TAP_PUSH_TOKEN -->

release 管線 SHALL 於 GitHub Release 建立成功且 CLI npm 套件發布完成後，在跨 repo 憑證 secret（TAP_PUSH_TOKEN）存在時，以 formula 產生器對該版 tgz 校驗清單的輸出更新 tap repo 的 Formula/speclink.rb（commit 訊息含該版 tag）；TAP_PUSH_TOKEN 缺席、或 CLI npm 發布因 NPM_TOKEN 缺席而跳過時，SHALL 跳過推送且不影響 Release 結果；secret 存在而推送失敗時該 job SHALL 以非零結束。推送 SHALL 發生於 Release 建立之後，SHALL NOT 作為 Release 發布的前置條件。憑證 SHALL 為僅授權 tap repo 內容寫入的 fine-grained token，SHALL NOT 使用授權範圍涵蓋其他 repo 的 token。

#### Scenario: 發版後 formula 自動更新

- **WHEN** TAP_PUSH_TOKEN 與 NPM_TOKEN 皆存在，且 push tag 的 Release 與 CLI npm 發布皆成功
- **THEN** tap repo 的 Formula/speclink.rb 更新為該版產生器輸出（四組 url 指向 registry.npmjs.org 該版 tgz、sha256 對應），commit 訊息含該 tag

#### Scenario: 憑證缺席跳過不紅燈

- **WHEN** TAP_PUSH_TOKEN 未設定且 push tag
- **THEN** tap 推送 job 跳過，Release 照常發布，workflow 整體綠

#### Scenario: 推送失敗不回溯撤銷 Release

- **WHEN** TAP_PUSH_TOKEN 存在但對 tap repo 的更新請求失敗
- **THEN** tap 推送 job 以非零結束供單獨重跑，既已建立的 Release 與其 assets 不受影響

#### Scenario: npm 未發布時 tap 跳過

- **WHEN** NPM_TOKEN 未設定（CLI npm 發布跳過）而 TAP_PUSH_TOKEN 存在
- **THEN** tap 推送 job 跳過且 job 綠，Formula/speclink.rb 不更新

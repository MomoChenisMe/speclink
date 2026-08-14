## Purpose

CLI 的安裝通路契約：安裝腳本（sh 與 PowerShell）如何把 GitHub Release 的 CLI 壓縮檔安全地裝進使用者機器（平台偵測、checksum 驗證、安裝位置與可測性）、Homebrew formula 產生器的輸出契約，以及 release 後 formula 自動推送 tap 的管線契約。邊界：只涵蓋 CLI 散佈；桌面安裝檔與更新屬 desktop-release，安裝文件的呈現屬 user-documentation。

## ADDED Requirements

### Requirement: 安裝腳本一行安裝對應平台 CLI

專案 SHALL 提供 POSIX sh 相容的安裝腳本（macOS 與 Linux）與 PowerShell 安裝腳本（Windows），行為契約一致：偵測 OS 與 CPU 架構並對映到 release 資產的 target triple 命名（與 release 管線的五個 target 一致）、以 GitHub Releases API 解析最新版本（環境變數可釘選指定版本）、下載壓縮檔與 SHA256SUMS.txt 並驗證 checksum、解壓 CLI binary 至安裝目錄（Unix 預設 ~/.local/bin、Windows 預設使用者層級程式目錄，環境變數可覆寫），安裝後 SHALL 檢查安裝目錄是否在 PATH 並於缺席時提示。checksum 驗證失敗時 SHALL 以非零結束且 SHALL NOT 留下任何已落檔的產物。腳本 SHALL 支援 dry-run 模式：只輸出解析出的 target、資產 URL 與安裝目錄，不發出網路請求、不寫入檔案；平台對映與 URL 組裝 SHALL 有以 dry-run 驗證的自動化測試。

#### Scenario: dry-run 輸出平台對映

- **WHEN** 在測試中以強制指定的 OS 與架構組合執行安裝腳本的 dry-run 模式
- **THEN** 輸出的 target triple 與資產 URL 和 release 管線對該平台的命名一致，且過程無網路請求、無檔案寫入

#### Scenario: checksum 不符即中止

- **WHEN** 下載的壓縮檔與 SHA256SUMS.txt 中對應條目不一致
- **THEN** 腳本以非零結束、錯誤訊息指出 checksum 不符，安裝目錄無新增或被覆寫的檔案

#### Scenario: 安裝完成後版本可驗

- **WHEN** 在支援平台上執行安裝腳本安裝指定版本
- **THEN** 安裝目錄出現 speclink 可執行檔，執行 speclink --version 輸出該版本號

### Requirement: Homebrew formula 產生器

專案 SHALL 提供 formula 產生器腳本：輸入 release tag 與該版 SHA256SUMS.txt，輸出完整的 Homebrew formula 至 stdout，內容 SHALL 含 macOS 與 Linux 各自的 arm64 與 x86_64 四組資產 URL 與對應 sha256，URL SHALL 指向該 tag 的 GitHub Release CLI 壓縮檔資產。SHA256SUMS.txt 缺少四組中任一平台的條目時 SHALL 以非零結束並指出缺少的平台。產生器 SHALL 有以 fixture checksums 驗證輸出的自動化測試。

#### Scenario: 產出四組平台對應

- **WHEN** 以 fixture 的 tag 與含五 target 條目的 SHA256SUMS.txt 執行產生器
- **THEN** stdout 的 formula 含 on_macos 與 on_linux 區塊、arm64 與 x86_64 四組 url 與 sha256，且每組 sha256 等於 fixture 中對應條目

#### Scenario: checksums 缺項即失敗

- **WHEN** SHA256SUMS.txt 缺少 macOS arm64 條目時執行產生器
- **THEN** 產生器以非零結束，錯誤訊息指出缺少該平台條目，stdout 無 formula 輸出

### Requirement: Formula 隨發版自動推送 tap

release 管線 SHALL 於 GitHub Release 建立成功後，在跨 repo 憑證 secret（TAP_PUSH_TOKEN）存在時，以 formula 產生器對該版 SHA256SUMS.txt 的輸出更新 tap repo 的 Formula/speclink.rb（commit 訊息含該版 tag）；secret 缺席時 SHALL 跳過推送且不影響 Release 結果；secret 存在而推送失敗時該 job SHALL 以非零結束。推送 SHALL 發生於 Release 建立之後，SHALL NOT 作為 Release 發布的前置條件。憑證 SHALL 為僅授權 tap repo 內容寫入的 fine-grained token，SHALL NOT 使用授權範圍涵蓋其他 repo 的 token。

#### Scenario: 發版後 formula 自動更新

- **WHEN** TAP_PUSH_TOKEN 存在且 push tag 的 Release 建立成功
- **THEN** tap repo 的 Formula/speclink.rb 更新為該版產生器輸出（四組 url 與 sha256 指向該 tag），commit 訊息含該 tag

#### Scenario: 憑證缺席跳過不紅燈

- **WHEN** TAP_PUSH_TOKEN 未設定且 push tag
- **THEN** tap 推送 job 跳過，Release 照常發布，workflow 整體綠

#### Scenario: 推送失敗不回溯撤銷 Release

- **WHEN** TAP_PUSH_TOKEN 存在但對 tap repo 的更新請求失敗
- **THEN** tap 推送 job 以非零結束供單獨重跑，既已建立的 Release 與其 assets 不受影響

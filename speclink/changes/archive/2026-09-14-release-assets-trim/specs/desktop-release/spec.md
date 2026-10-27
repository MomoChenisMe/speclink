## MODIFIED Requirements

### Requirement: Release 產出三平台桌面安裝檔
<!-- BEFORE: macOS dmg 依晶片各一、Linux AppImage 與 deb、SHA256SUMS.txt 收錄全部、既有 CLI 壓縮檔維持不變 -->

push 符合 v* 的 tag 後，release 管線 SHALL 產出桌面安裝檔並附於同一個 GitHub Release，且該 Release 的 assets SHALL 恰為下列六個檔案：macOS universal dmg（Apple Silicon 與 Intel 同一檔，檔名 Speclink_<版本>_universal.dmg）、macOS universal 更新包（Speclink_<版本>_universal.app.tar.gz）、Windows NSIS 安裝器（x86_64，Speclink_<版本>_x64-setup.exe）、Linux AppImage（x86_64 為 Speclink_<版本>_amd64.AppImage、aarch64 為 Speclink_<版本>_aarch64.AppImage）、更新描述檔 latest.json。Release assets SHALL NOT 含 .sig 簽章檔、.deb、CLI 壓縮檔、SHA256SUMS.txt 或 server 壓縮檔（CLI 通路見 cli-distribution，server 發布通路見 server-release）。桌面安裝檔 SHALL 內含同版 speclink CLI binary，macOS 內含的 CLI SHALL 為含 arm64 與 x86_64 兩個架構的 universal binary。Windows NSIS 安裝器 SHALL 內建繁體中文與英文語系並跟隨系統顯示語言（繁中系統顯示繁中、其餘落回英文），安裝器圖示 SHALL 為 Speclink 產品圖示。

#### Scenario: tag 發布產出完整安裝檔集

- **WHEN** push tag v0.5.0 且 workflow 全部成功
- **THEN** 該 Release 的 assets 恰為 Speclink_0.5.0_universal.dmg、Speclink_0.5.0_universal.app.tar.gz、Speclink_0.5.0_x64-setup.exe、Speclink_0.5.0_amd64.AppImage、Speclink_0.5.0_aarch64.AppImage 與 latest.json 六個，無任何 .sig、.deb、.zip、speclink-v* 壓縮檔或 SHA256SUMS.txt

#### Scenario: 任一形態失敗則不發布

- **WHEN** 桌面安裝檔任一 target 建置失敗
- **THEN** GitHub Release SHALL NOT 建立（與既有 Docker gating 同一 needs 閘門），不產生缺形態的 Release

#### Scenario: 繁中系統的安裝精靈顯示繁中

- **WHEN** 在顯示語言為繁體中文的 Windows 上執行 NSIS 安裝器
- **THEN** 安裝精靈以繁體中文顯示，安裝檔圖示為 Speclink 產品圖示；非內建語系的系統落回英文

#### Scenario: universal dmg 在兩種晶片皆可用

- **WHEN** 在 Apple Silicon 與 Intel 的 Mac 上分別以同一個 universal dmg 安裝並啟動 app
- **THEN** 兩台皆能啟動，且 app 內含的 speclink CLI 以 lipo -info 檢視含 arm64 與 x86_64 兩個架構

### Requirement: 更新描述檔隨 release 發布
<!-- BEFORE: darwin-aarch64 與 darwin-x86_64 各指向自己架構的更新包；組裝輸入為每個平台鍵一個子目錄 -->

release 管線 SHALL 組裝更新描述檔 latest.json 附於 Release assets：version 欄位等於 tag 去除 v 前綴、pub_date 為發布時間、platforms 物件至少含 darwin-aarch64、darwin-x86_64、windows-x86_64、linux-x86_64 四鍵，各鍵含 url（指向同一 Release 的對應更新包 asset）與 signature（該更新包的簽章內容）；darwin-aarch64 與 darwin-x86_64 兩鍵 SHALL 指向同一個 universal 更新包且 signature 相同。組裝腳本的輸入 SHALL 為「每個更新包目錄對應一或多個平台鍵」：必要目錄 darwin-universal（對應 darwin 兩鍵）、windows-x86_64、linux-x86_64，可選目錄 linux-aarch64；每個目錄恰含一個更新包與同名 .sig。缺任一必要目錄、缺更新包或缺簽章時 SHALL 以非零結束且不寫出描述檔。組裝邏輯 SHALL 有可獨立執行的單元測試。

#### Scenario: 描述檔欄位對齊發布內容

- **WHEN** push tag v0.5.0 且 workflow 成功
- **THEN** latest.json 的 version 為 0.5.0，且 platforms 每一鍵的 url 均指向本次 Release 的 asset 下載路徑

##### Example: v0.5.0 的 platforms 對應

| platforms 鍵 | url 指向的 asset |
| ------------ | ---------------- |
| darwin-aarch64 | v0.5.0 Release 中的 Speclink_0.5.0_universal.app.tar.gz |
| darwin-x86_64 | 與 darwin-aarch64 相同的 url，signature 亦相同 |
| windows-x86_64 | v0.5.0 Release 中的 Speclink_0.5.0_x64-setup.exe |
| linux-x86_64 | v0.5.0 Release 中的 Speclink_0.5.0_amd64.AppImage |

#### Scenario: 更新端點可匿名讀取

- **WHEN** 以未帶認證的 HTTP GET 請求 releases/latest/download/latest.json 路徑
- **THEN** 回應為可解析的 JSON 且內容為最新 Release 的更新描述檔

#### Scenario: 缺更新包目錄即不組裝

- **WHEN** 組裝腳本的輸入缺 darwin-universal 目錄，或該目錄有更新包但缺同名 .sig
- **THEN** 腳本以非零結束並點名缺失的目錄或簽章，不寫出 latest.json

### Requirement: Release 說明含下載指南
<!-- BEFORE: 對照表 macOS 依晶片各一、Linux AppImage 與 deb；CLI 節為安裝腳本與 Homebrew；註明 .app.tar.gz 與全部 .sig 為自動更新用 -->

release job SHALL 以腳本產生下載指南並置於 GitHub Release 說明開頭，其後 SHALL 接該版的更新日誌片段（由 release-notes 的渲染腳本自 release-notes.json 畫出），GitHub 自動產生的比較連結接續於最後。指南 SHALL 含：一般使用者安裝檔對照表（macOS 一列指向 universal dmg 並註明 Apple Silicon 與 Intel 同一檔、Windows NSIS 安裝器一列、Linux 桌面機依架構各一 AppImage、Linux 伺服器或無圖形介面一列指向 CLI 一行安裝而非任何檔案；檔名含該版號且與資產命名一致）、CLI 的三條一行指令（npm 全域安裝、安裝腳本 curl 一行、Homebrew）、server 的一行啟動節（npx 指令與 Docker image 一行，寫法比照 CLI 安裝節），並 SHALL 註明更新包（.app.tar.gz）與 latest.json 為自動更新機制使用、毋須手動下載。指南 SHALL NOT 提及 .sig、.deb、SHA256SUMS.txt 或 CLI 壓縮檔。產生器 SHALL 於 tag 不符 vX.Y.Z 格式時以非零結束，SHALL 於 release-notes.json 沒有該版條目時以非零結束，且 SHALL 有驗證輸出內容的自動化測試。

#### Scenario: 發布後 Release 頁開頭為下載指南

- **WHEN** push tag 完成發布後檢視該 GitHub Release 頁
- **THEN** 說明開頭為四列安裝檔對照表、三條 CLI 安裝指令與 server 一行啟動節（npx 與 Docker），.app.tar.gz 與 latest.json 標註為自動更新機制使用，其後為該版更新日誌片段（`## X.Y.Z（日期）` 與分組條目），自動產生的比較連結位於最後

#### Scenario: 指南檔名對齊版號與資產命名

- **WHEN** 以 tag vX.Y.Z 執行下載指南產生器
- **THEN** 輸出中的每個檔名含該版號且與 release 管線對應平台的資產命名一致

#### Scenario: JSON 缺該版時產生器失敗

- **WHEN** 以 tag v0.3.0 執行產生器而 release-notes.json 沒有 0.3.0 條目
- **THEN** 產生器以非零結束並於 stderr 說明，Release 說明不產生

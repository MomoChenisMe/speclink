## ADDED Requirements

### Requirement: Release 說明含下載指南

release job SHALL 以腳本產生下載指南並置於 GitHub Release 說明開頭，自動產生的 changelog 接續其後。指南 SHALL 含：三平台一般使用者安裝檔對照表（macOS dmg 依晶片各一、Windows NSIS 安裝器、Linux AppImage 與 deb 依架構各一，檔名含該版號且與資產命名一致）、CLI 的安裝腳本一行指令與 Homebrew 指令，並 SHALL 註明更新包（.app.tar.gz）與全部 .sig 簽章檔為自動更新機制使用、毋須手動下載。產生器 SHALL 於 tag 不符 vX.Y.Z 格式時以非零結束，且 SHALL 有驗證輸出內容的自動化測試。

#### Scenario: 發布後 Release 頁開頭為下載指南

- **WHEN** push tag 完成發布後檢視該 GitHub Release 頁
- **THEN** 說明開頭為三平台安裝檔對照表與 CLI 安裝指令，.sig 與 .app.tar.gz 標註為自動更新機制使用，自動 changelog 位於其後

#### Scenario: 指南檔名對齊版號與資產命名

- **WHEN** 以 tag vX.Y.Z 執行下載指南產生器
- **THEN** 輸出中的每個檔名含該版號且與 release 管線對應平台的資產命名一致

## MODIFIED Requirements

### Requirement: Release 產出三平台桌面安裝檔

push 符合 v* 的 tag 後，release 管線 SHALL 產出桌面安裝檔並附於同一個 GitHub Release：macOS dmg（aarch64 與 x86_64 各一）、Windows NSIS 安裝器（x86_64）、Linux AppImage 與 deb（x86_64 與 aarch64）。SHA256SUMS.txt SHALL 收錄全部新增檔案。既有 CLI 壓縮檔與 Docker 映像的命名與內容 SHALL 維持不變；server 壓縮檔 SHALL NOT 出現在 Release assets（server 發布通路見 server-release）。桌面安裝檔 SHALL 內含同版 speclink CLI binary。

#### Scenario: tag 發布產出完整安裝檔集

- **WHEN** push tag v0.2.0 且 workflow 全部成功
- **THEN** 該 Release 的 assets 同時包含既有五 target CLI 壓縮檔、上列全部桌面安裝檔，每個檔案在 SHA256SUMS.txt 中有對應條目，且無任何 speclink-server-* 壓縮檔

#### Scenario: 任一形態失敗則不發布

- **WHEN** 桌面安裝檔任一 target 建置失敗
- **THEN** GitHub Release SHALL NOT 建立（與既有 Docker gating 同一 needs 閘門），不產生缺形態的 Release

### Requirement: OS 程式碼簽章為可插鑰匙開關

macOS 與 Windows 的 OS 程式碼簽章 SHALL 以對應平台的 secrets 組是否存在為條件：該平台簽章組完全不存在時 SHALL 跳過簽章且 workflow SHALL 成功、產出未簽章安裝檔；完整存在時 SHALL 執行簽章。macOS 簽章組 SHALL 由憑證與公證兩半構成（憑證半組：憑證內容、憑證密碼、簽章身分；公證半組：Apple ID、App 專用密碼、Team ID），齊備時 SHALL 於簽章後執行公證並將票證 staple 至產物。Windows SHALL 支援兩條簽章路徑並依此取捨：SignPath secrets 組（API token、organization、project、policy）齊備時 SHALL 經 SignPath 服務完成 Authenticode 簽章；否則本機憑證 secrets 存在時 SHALL 走既有憑證路徑。任一平台的簽章組部分存在（非全有全無）時 workflow SHALL 以列出缺項 secrets 名稱的錯誤失敗，SHALL NOT 產出半套簽章的安裝檔。本開關 SHALL NOT 影響更新包的 updater 簽章（兩者正交），且 OS 簽章 SHALL 在 updater 簽章產生之前完成，使更新包簽章涵蓋的是已完成 OS 簽章的檔案。

#### Scenario: 無簽章金鑰照常發布

- **WHEN** repo 未設定任何 OS 簽章 secrets 且 push tag
- **THEN** workflow 全綠，Release 產出未簽章安裝檔，assets 清單與有簽章時一致

#### Scenario: 插入金鑰即啟用簽章

- **WHEN** repo 設定了完整 macOS 簽章組（憑證半組與公證半組共六項）後 push tag
- **THEN** macOS 安裝檔內的 app 完成 Developer ID 簽章與公證、票證已 staple，Gatekeeper 評估通過，workflow 其餘步驟不變

#### Scenario: 簽章組部分設定即紅燈

- **WHEN** repo 只設定 macOS 憑證半組而缺公證半組任一項（或 SignPath 組四項缺一）並 push tag
- **THEN** workflow 以非零結束，錯誤訊息列出缺少的 secrets 名稱，Release 不發布

#### Scenario: SignPath 路徑簽章 Windows 安裝檔

- **WHEN** repo 設定了完整 SignPath secrets 組後 push tag
- **THEN** Windows NSIS 安裝檔帶有效 Authenticode 簽章，且 latest.json 中 windows-x86_64 的 signature 對該已簽章檔案驗證通過

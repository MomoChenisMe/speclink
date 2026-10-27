## MODIFIED Requirements

### Requirement: Release 說明含下載指南
<!-- BEFORE: 指南之後接 GitHub 自動產生的 changelog（實際上只剩一行比較連結） -->

release job SHALL 以腳本產生下載指南並置於 GitHub Release 說明開頭，其後 SHALL 接該版的更新日誌片段（由 release-notes 的渲染腳本自 release-notes.json 畫出），GitHub 自動產生的比較連結接續於最後。指南 SHALL 含：三平台一般使用者安裝檔對照表（macOS dmg 依晶片各一、Windows NSIS 安裝器、Linux AppImage 與 deb 依架構各一，檔名含該版號且與資產命名一致）、CLI 的安裝腳本一行指令與 Homebrew 指令、server 的一行啟動節（npx 指令與 Docker image 一行，寫法比照 CLI 安裝節），並 SHALL 註明更新包（.app.tar.gz）與全部 .sig 簽章檔為自動更新機制使用、毋須手動下載。產生器 SHALL 於 tag 不符 vX.Y.Z 格式時以非零結束，SHALL 於 release-notes.json 沒有該版條目時以非零結束，且 SHALL 有驗證輸出內容的自動化測試。

#### Scenario: 發布後 Release 頁開頭為下載指南

- **WHEN** push tag 完成發布後檢視該 GitHub Release 頁
- **THEN** 說明開頭為三平台安裝檔對照表、CLI 安裝指令與 server 一行啟動節（npx 與 Docker），.sig 與 .app.tar.gz 標註為自動更新機制使用，其後為該版更新日誌片段（`## X.Y.Z（日期）` 與分組條目），自動產生的比較連結位於最後

#### Scenario: 指南檔名對齊版號與資產命名

- **WHEN** 以 tag vX.Y.Z 執行下載指南產生器
- **THEN** 輸出中的每個檔名含該版號且與 release 管線對應平台的資產命名一致

#### Scenario: JSON 缺該版時產生器失敗

- **WHEN** 以 tag v0.3.0 執行產生器而 release-notes.json 沒有 0.3.0 條目
- **THEN** 產生器以非零結束並於 stderr 說明，Release 說明不產生

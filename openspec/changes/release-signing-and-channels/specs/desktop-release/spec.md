## MODIFIED Requirements

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

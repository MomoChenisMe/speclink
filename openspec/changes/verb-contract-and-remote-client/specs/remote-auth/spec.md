## ADDED Requirements

### Requirement: PAT 登入與憑證儲存
speclink auth login SHALL 接受 PAT（互動輸入或 --token-stdin），依連接 url 的 origin 存入使用者層級設定目錄的憑證檔（Unix 檔案權限 0600），SHALL NOT 將憑證寫入專案 repo 內的任何檔案。環境變數 SPECLINK_TOKEN 存在時 SHALL 優先於憑證檔。speclink auth status SHALL 查驗當前憑證並顯示身分與 repo 驗證結果。

#### Scenario: 登入後憑證落於使用者目錄
- **WHEN** 於 remote 模式專案執行 speclink auth login 並提供有效 PAT
- **THEN** 憑證寫入使用者層級設定目錄（專案 repo 內無任何新增或變更的檔案），指令顯示登入成功與身分資訊

#### Scenario: SPECLINK_TOKEN 覆寫憑證檔
- **WHEN** 憑證檔含某 token A，環境變數 SPECLINK_TOKEN 設為 token B，執行 speclink auth status
- **THEN** 查驗以 token B 進行

#### Scenario: 未登入的狀態查詢
- **WHEN** 無憑證檔且無 SPECLINK_TOKEN，執行 speclink auth status
- **THEN** 顯示未登入狀態與 speclink auth login 指引，exit code 非 0

### Requirement: 憑證失效的處理
remote 動詞收到未授權回應時，CLI SHALL 以非 0 exit code 結束並提示重新執行 speclink auth login；SHALL NOT 進入重試迴圈、SHALL NOT 靜默改用其他憑證來源。

#### Scenario: token 撤銷後的動詞行為
- **WHEN** 憑證已被 server 撤銷，執行 speclink list
- **THEN** exit code 非 0，stderr 單行訊息說明認證失效並提示 speclink auth login，指令不重試

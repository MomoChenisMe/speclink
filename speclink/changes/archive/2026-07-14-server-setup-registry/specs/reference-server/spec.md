## MODIFIED Requirements

### Requirement: 啟動組態 fail closed

server SHALL 以組態檔啟動，宣告 store driver、identity 資料庫（sqlite 路徑；memory 變體 SHALL 僅供測試組態）、public url 與事件參數。組態檔缺失、不可解析、宣告未知 driver、或任一段形狀不合 SHALL 使啟動失敗並印出指向錯誤的原因，SHALL NOT 以部分預設啟動；組態 SHALL NOT 含 bootstrap token 對 actor 的映射段，SHALL NOT 含 Project/Repo registry 段——registry 的事實來源是 server 資料庫（見 server-setup 能力），殘留 projects 段 SHALL 使啟動失敗並指出已由 registry 取代。sqlite driver SHALL 為預設持久層選項，memory driver SHALL 僅供測試組態。

#### Scenario: 壞組態拒絕啟動

- **WHEN** 以 YAML 不可解析的組態檔啟動 server
- **THEN** 程序以非零 exit code 結束，stderr 指出組態檔路徑與解析原因；不綁定任何連接埠

#### Scenario: 未知 driver 拒絕啟動

- **WHEN** 組態宣告 store driver 為未支援的名稱
- **THEN** 啟動失敗且原因列出支援的 driver 名稱

#### Scenario: 殘留 tokens 段拒絕啟動

- **WHEN** 以仍含舊 bootstrap tokens 段的組態檔啟動 server
- **THEN** 啟動失敗且原因指出該段已由 identity 儲存取代

#### Scenario: 殘留 projects 段拒絕啟動

- **WHEN** 以仍含舊 projects registry 段的組態檔啟動 server
- **THEN** 啟動失敗且原因指出該段已由 server 資料庫的 registry 取代

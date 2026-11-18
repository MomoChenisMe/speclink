## MODIFIED Requirements

### Requirement: identity 儲存獨立且版本守門
<!-- BEFORE: identity 與「TeamStore 的資料庫」分離、不寫入「TeamStore driver 的檔案」 -->

identity 資料（users、memberships、invitations、PATs、sessions）SHALL 存於 server 自有的資料庫，與規格文件的儲存分離，SHALL NOT 寫入規格文件儲存的資料庫或資料夾。identity 資料庫 SHALL 記錄 schema version：空資料庫初始化為現行版本；version 較新或 schema 非本 server 所建 SHALL 使啟動失敗並印出原因，SHALL NOT 寫入。密碼、PAT、邀請 token 與 session 識別 SHALL 僅以 hash 落庫，SHALL NOT 出現在 log。

#### Scenario: 陌生 identity 資料庫拒啟動

- **WHEN** 組態的 identity 路徑指向一個由其他應用建立的 SQLite 檔並啟動 server
- **THEN** 啟動失敗、stderr 指出該檔與原因；檔案內容位元不變

#### Scenario: 憑證不落明文

- **WHEN** 完成邀請接受、登入與 PAT 建立後檢視 identity 資料庫內容
- **THEN** 密碼、邀請 token、session 識別與 PAT 欄位皆為 hash 形式，庫內不存在任何憑證明文

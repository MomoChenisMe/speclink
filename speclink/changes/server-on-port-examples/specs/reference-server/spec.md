## MODIFIED Requirements

### Requirement: 寫入原子提交且 CAS 衝突可辨
<!-- BEFORE: 寫入經 Host 的 UoW/event commit 原子提交至 TeamStore；文件、revision history 與 outbox 事件同 commit 可見 -->

命令路由的寫入 SHALL 經宿主套件以儲存接口的一次提交寫入：文件與事件在同一次提交裡，事件同時進入儲存的事件記錄，SHALL NOT 有繞過宿主套件直寫儲存的路徑。帶 If-Match 的寫入在 revision 不符時 SHALL 回 409 revision_conflict 且帶 expected 與 actual revision；衝突的寫入 SHALL NOT 留下部分變更。server SHALL NOT 執行任何 shell-out git 操作。

#### Scenario: 競寫敗方得衝突且無殘留

- **WHEN** 兩個 client 以同一 If-Match revision 先後寫同一 artifact
- **THEN** 先者成功；後者回 409 revision_conflict 帶 expected/actual；重讀該 artifact 內容為先者版本，store 無後者的部分寫入

#### Scenario: 成功寫入事件同步可見

- **WHEN** 成功執行 task done 命令路由
- **THEN** 該範圍的事件記錄含對應的 task-completed 事件，與任務勾選在同一次提交可見

### Requirement: 啟動組態 fail closed
<!-- BEFORE: 啟動只檢查組態檔；沒有規定儲存位置已有其他配置的資料時的行為 -->

server SHALL 以組態檔啟動，宣告 store driver、identity 資料庫（sqlite 路徑；memory 變體 SHALL 僅供測試組態）、public url 與事件參數。組態檔缺失、不可解析、宣告未知 driver、或任一段形狀不合 SHALL 使啟動失敗並印出指向錯誤的原因，SHALL NOT 以部分預設啟動；組態 SHALL NOT 含 bootstrap token 對 actor 的映射段，SHALL NOT 含 Project/Repo registry 段——registry 的事實來源是 server 資料庫（見 server-setup 能力），殘留 projects 段 SHALL 使啟動失敗並指出已由 registry 取代。sqlite driver SHALL 為預設持久層選項，memory driver SHALL 僅供測試組態。

儲存位置已經有資料、而且是 0.12.0 以前的版本建立的配置時，啟動 SHALL 失敗：exit code 非零、SHALL NOT 綁定任何連接埠、SHALL NOT 改動該資料，stderr SHALL 說明這份資料由較舊的版本建立，並列出三個步驟——以舊版執行 `speclink-server backup --config <file> --output <file>`、把組態指向空的儲存位置與空的 identity 資料庫、以這個版本執行 `speclink-server restore --config <file> --input <file>`。backup 子命令開啟儲存時遇到同樣的資料 SHALL 以同樣的方式失敗。儲存位置的資料帶有其他無法辨認的標記、沒有標記或較新的配置版本時，啟動 SHALL 失敗並印出讀到的標記與配置版本，SHALL NOT 印出上述三個步驟，SHALL NOT 改動該資料。

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

#### Scenario: 舊版建立的 SQLite 資料拒絕啟動

- **WHEN** 組態的 store driver 是 sqlite，路徑指向 0.12.0 建立的資料庫檔，啟動 server
- **THEN** 程序以非零 exit code 結束，不綁定任何連接埠；stderr 說明資料由較舊的版本建立並列出 backup、指向空位置、restore 三個步驟；資料庫檔逐位元不變

#### Scenario: 舊版建立的資料夾與 PostgreSQL 資料同樣拒絕

- **WHEN** 組態的 store driver 是 serverfs 或 postgres，位置是 0.12.0 建立的資料，啟動 server
- **THEN** 結果與 SQLite 相同：非零 exit code、三個步驟、資料不變

#### Scenario: backup 子命令遇到舊版的資料

- **WHEN** 以這個版本對 0.12.0 建立的資料執行 backup 子命令
- **THEN** 非零 exit code；stderr 列出同樣的三個步驟；沒有產生備份檔

#### Scenario: 無法辨認的資料拒絕啟動

- **WHEN** 組態的 store driver 是 serverfs，路徑指向一個放著其他檔案的資料夾，啟動 server
- **THEN** 程序以非零 exit code 結束，stderr 說明這個位置的資料不是 speclink 的儲存；沒有印出三個步驟；資料夾裡沒有新增任何檔案

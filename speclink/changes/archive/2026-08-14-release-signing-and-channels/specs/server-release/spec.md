## MODIFIED Requirements

### Requirement: release 產物含 server 與部署文件

release 工作流 SHALL 於既有矩陣建置各平台 server binary 並執行無前端 dist 環境的冒煙驗證（/login 回 HTML、未知 browser API 回 JSON 404）作為發布品質閘門，但 SHALL NOT 將 server binary 打包上傳至 GitHub Release assets——server 的官方發布通路 SHALL 為版本對齊 tag 的 Docker image 與 npm 套件（見「npm 套件一行啟動 server」），並 SHALL 建置與發布該映像。SHALL 有部署文件涵蓋：npx 快速啟動、Docker 直跑、SQLite compose、PostgreSQL compose 四種官方形態與從原始碼建置 binary 的替代路徑、setup token 取得、單一 instance 限制、環境變數清單、容器內 backup/restore/verify-backup 與 invite 子命令操作、升級步驟；文件 SHALL NOT 指向 Release 的 server 壓縮檔下載。

#### Scenario: release 定義完備

- **WHEN** 檢視 release 工作流定義與部署文件
- **THEN** 工作流含各平台 server 建置與無 dist 冒煙步驟、映像建置發布 job，打包與上傳步驟不含 server 壓縮檔；文件涵蓋四種官方形態、原始碼建置替代路徑與子命令操作，無 Release 壓縮檔下載指示

## ADDED Requirements

### Requirement: npm 套件一行啟動 server

專案 SHALL 提供 server 的 npm 通路：主套件帶 bin launcher 與各平台子套件的 optionalDependencies，每個子套件以 os／cpu 欄位對應平台且內容物為該平台的 speclink-server binary，安裝時只下載符合平台者；套件版本 SHALL 與 release tag 對齊。launcher 於無參數（或僅環境變數）啟動時 SHALL 依環境變數產生組態 YAML 寫入資料目錄後帶 --config 啟動 binary：SPECLINK_STORE 選擇 store driver（sqlite 為預設、serverfs 與 postgres 可選）、SPECLINK_DATA_DIR 指定資料目錄（預設 ./speclink-data）、SPECLINK_PUBLIC_URL 與 SPECLINK_PORT 決定對外位址（public_url 預設連動 port）；SPECLINK_STORE=postgres 而 SPECLINK_POSTGRES_URL 缺席時 SHALL 以非零結束並點名缺項。使用者帶 --config、設 SPECLINK_CONFIG 或使用子命令時 launcher SHALL 純透傳參數與 exit code，SHALL NOT 產生組態。平台無對應子套件時 SHALL 以可讀錯誤點名不支援的平台。發布 SHALL 由 release 管線於 NPM_TOKEN 存在時執行（npm publish --access public），缺席時 SHALL 跳過且不影響 Release 結果。

#### Scenario: 零參數啟動走 sqlite 預設

- **WHEN** 以 npx 執行主套件且未帶參數、未設 SPECLINK_STORE
- **THEN** 資料目錄產生（含組態 YAML 與 sqlite 的 store 與 identity 檔路徑宣告）、server 啟動並於首跑印出 setup token

#### Scenario: postgres 缺連線 URL 即死

- **WHEN** SPECLINK_STORE=postgres 而 SPECLINK_POSTGRES_URL 未設定時啟動
- **THEN** launcher 以非零結束，錯誤訊息點名 SPECLINK_POSTGRES_URL，不啟動 server、不產生組態

#### Scenario: 自帶組態純透傳

- **WHEN** 以 npx 執行主套件並帶 --config 指向既有 YAML（或執行 invite 等子命令）
- **THEN** launcher 將參數原樣傳給 binary、exit code 一致，行為與直接執行 binary 相同，資料目錄不產生新組態

#### Scenario: NPM_TOKEN 缺席跳過發布

- **WHEN** NPM_TOKEN 未設定且 push tag
- **THEN** npm 發布 job 跳過，Release 照常發布，workflow 整體綠

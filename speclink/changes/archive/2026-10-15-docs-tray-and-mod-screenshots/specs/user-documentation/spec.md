## MODIFIED Requirements

### Requirement: 使用者文件以截圖呈現實際介面

面向使用者的文件 SHALL 內嵌 desktop（含 macOS 系統匣面板）、server 後台與 Claude Code 外掛 speclink-skills 的截圖，使讀者在安裝前即可判斷產品樣貌。截圖 SHALL 分兩組存放於 `docs/assets/screenshots/en/`（英文介面）與 `docs/assets/screenshots/zh-TW/`（繁中介面），兩組檔名 SHALL 逐一相同；英文文件 SHALL 只引用英文介面的截圖，繁中文件 SHALL 只引用繁中介面的截圖。`README.md` 與 `README.zh-TW.md` SHALL 於第一個 H2 之前內嵌一張 desktop 看板截圖，並 SHALL 於畫面一覽段內嵌至少三張截圖，其中 SHALL 含 server 後台、系統匣面板與 Claude Code 外掛各至少一張。截圖 SHALL 以相對路徑內嵌於版本庫，SHALL NOT 依賴外部圖床。

截圖場景 SHALL 由版本庫內的腳本佈置為不含任何使用者真實資料的示範 workspace；該腳本 SHALL 在佈置前備份 desktop 的使用者狀態目錄，並在收尾或中斷時還原，SHALL 於 app 執行中時拒絕開始而不代為結束 app。示範 workspace 的 git 身分 SHALL 為腳本內建的示範身分，SHALL NOT 沿用使用者的全域 git 設定。截圖腳本 SHALL 列出每一個截圖檔名與兩個語言目錄；每個檔名在每個語言目錄的截圖 SHALL 至少被一份同語言的使用者文件引用，SHALL NOT 留下沒有文件使用的截圖。

截圖 SHALL 只含 app 視窗或瀏覽器頁面本身，SHALL NOT 含作業系統選單列、Dock、桌布、通知或其他 app。截圖畫面 SHALL NOT 出現使用者帳號名稱、含家目錄或系統暫存目錄的路徑、`example.com` 與 `speclink.invalid` 以外的信箱、主機名稱、使用者真實的專案或連線名稱、存取金鑰明文或前綴、setup token 與 server 資料目錄路徑。Claude Code 外掛的截圖 SHALL 只呈現 Claude Code 本身與 speclink-skills 的介面，SHALL NOT 出現使用者自行安裝的其他外掛或狀態列輸出（例如用量百分比），且拍攝過程 SHALL NOT 送出任何提示給模型。系統匣面板的截圖 SHALL 只含面板視窗。截圖檔 SHALL NOT 帶 PNG 文字中繼資料區塊（`tEXt`、`iTXt`、`zTXt`、`eXIf`）。

#### Scenario: README 可見產品畫面

- **WHEN** 讀者在 GitHub 上開啟任一語言的 README
- **THEN** 第一個 H2 之前可見一張該語言介面的 desktop 看板截圖，畫面一覽段可見至少三張該語言介面的截圖，其中含 server 後台、系統匣面板與 Claude Code 外掛各一張，圖片皆以版本庫內的相對路徑載入

#### Scenario: 英文文件顯示英文介面

- **WHEN** 讀者開啟任一英文文件中的截圖
- **THEN** 圖片路徑位於 `docs/assets/screenshots/en/`，畫面中的介面文字為英文；同一位置在繁中文件引用 `docs/assets/screenshots/zh-TW/` 的同名截圖，介面文字為繁體中文

#### Scenario: 沒有孤兒截圖

- **WHEN** 執行文件守門測試
- **THEN** 截圖腳本列出的每一個檔名，在兩個語言目錄的截圖都至少出現在一份同語言的 README 或 docs/ 文件中；任一張沒有被引用時測試以非零結果點名該檔

#### Scenario: 拍攝不損毀使用者既有狀態

- **WHEN** 執行截圖場景腳本並在拍攝中途以中斷訊號結束
- **THEN** 腳本仍還原使用者原本的 desktop 狀態目錄，使用者的 workspace 分頁與 Server 連線設定與執行前一致

#### Scenario: app 執行中拒絕開始

- **WHEN** desktop app 正在執行時啟動截圖場景腳本
- **THEN** 腳本以非零結束並說明須先關閉 app，不搬移任何目錄、不代為結束 app

#### Scenario: 截圖不含個人或機敏資訊

- **WHEN** 逐張檢視 `docs/assets/screenshots/en/` 與 `docs/assets/screenshots/zh-TW/` 的截圖並列出每個 PNG 的區塊清單
- **THEN** 畫面只有 app 視窗或瀏覽器頁面，不見使用者帳號名稱、家目錄或暫存目錄路徑、非示範信箱、主機名稱、真實專案名稱、存取金鑰、setup token 與資料目錄路徑；區塊清單不含 `tEXt`、`iTXt`、`zTXt`、`eXIf`

#### Scenario: Claude Code 外掛截圖只呈現 speclink-skills

- **WHEN** 檢視 `claude-code-panel.png`、`claude-code-quality.png` 與 `claude-code-hash-menu.png` 的中英兩組截圖
- **THEN** 畫面只有一個終端機視窗，內容是 Claude Code 與 speclink-skills 的技能列、側邊面板或 `#` 選單；不見使用者其他外掛的輸出、用量百分比、帳號名稱、信箱與家目錄路徑

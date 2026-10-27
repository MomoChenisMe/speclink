## MODIFIED Requirements

### Requirement: 使用者文件採漸進揭露與單一責任

Speclink 使用者文件 SHALL 以 README、getting-started、workflow、product-status 與 roadmap 形成漸進揭露入口。README 第一個 H2 之前 SHALL 呈現品牌圖片、標語（繁中版「一套 SDD Engine，支援 Local Repo 與 Remote Store」，英文版「One SDD Engine for Local Repo and Remote Store」）、徽章、語言切換、Rust SDD 引擎與工具平台定位、PM／PO／RD／AI Agent 共用 change／artifact／task／verify／archive 語意、Local Repo／Remote Store 雙路徑，以及一張 desktop 看板截圖。README SHALL 以 H2 提供：為什麼用 Speclink、快速開始、畫面一覽、特色、最短流程心智模型（工作流程）、安裝、自架 server、文件地圖、由 product-status 校正的目前狀態摘要、參與貢獻、致謝與授權；快速開始 SHALL 位於安裝之前，並以安裝 CLI、`speclink init` 與請 Agent 執行 propose 三步呈現 Local Repo 開始入口。設計之初以 Spectra App 2.3.1 CLI 為行為參考的歷史起源 SHALL 寫在致謝段，同段 SHALL 致謝 OpenSpec 的目錄結構；該起源 SHALL NOT 寫在第一個 H2 之前。getting-started SHALL 只承載可直接完成的 Local Repo 第一輪，workflow SHALL 作為完整使用流程正典，product-status SHALL 作為目前能力狀態正典，roadmap SHALL 作為尚未完成的方向的唯一落點。各文件 SHALL 以連結導向下一層細節，SHALL NOT 在 README 或 getting-started 複製完整架構與狀態矩陣。

#### Scenario: README 保留專案定位與起源

- **WHEN** 使用者開啟繁體中文或英文 README 判斷 Speclink 是什麼及為何存在
- **THEN** 第一個 H2 之前可見品牌圖片、SDD Engine 標語、語言切換、Rust 實作與共同流程語意、Local Repo／Remote Store 說明及 desktop 看板截圖；致謝段可見 Spectra App 2.3.1 行為參考起源與 OpenSpec；目前狀態段清楚區分已可運作與規劃中內容，並連到專案路線圖
- **AND** 文件整理只校正過時事實、術語與連結，不得將第一個 H2 之前的內容改成僅含導覽連結的入口

#### Scenario: 首次使用者由 README 到完成第一輪

- **WHEN** 首次使用者從繁體中文或英文 README 尋找安裝與第一輪 Local Repo 操作
- **THEN** README 的快速開始段以三步呈現安裝 CLI、`speclink init` 與請 Agent 執行 propose，並提供可見的 getting-started 入口；getting-started 以目前存在的 CLI／skill 完成 init、提案、實作檢查與封存，遇到選用分支時可連到 workflow

#### Scenario: 進階使用者查詢目前能力與目標

- **WHEN** 使用者要判斷 Desktop Remote Workspace 或 Server 某項能力目前是否可用及最終目標
- **THEN** README 導向 product-status 取得目前狀態、證據與限制，並由該列導向專案路線圖取得之後的方向，兩者 SHALL NOT 混成同一狀態描述

### Requirement: 中英文文件保持結構與事實對等

`README.zh-TW.md`／`README.md`（繁中／英文），以及 `docs/` 下每一組 `<名稱>.zh-TW.md`／`<名稱>.md`（getting-started、workflow、product-status、roadmap、remote-getting-started、development、configuration、verb-contract、sdk-node、server-deployment、server-store-drivers、server-backup）SHALL 分別保持相同的 H2 章節集合與順序、狀態矩陣列集合、命令語意、截圖檔名序列及交叉連結。英文版 SHALL 為無語言後綴的檔名，繁中版 SHALL 帶 `.zh-TW` 後綴。每個 H2 SHALL 寫成 `## <a id="<錨點>"></a><標題>`，兩版的錨點序列 SHALL 逐項相同；標題文字 SHALL 各用該版的語言，SHALL NOT 在單一語言版本使用雙語標題。兩版 README SHALL 共用品牌圖片；截圖 SHALL 以相同檔名、各自引用該語言介面的那一組。兩版 README SHALL 保持標語、產品定位、共同流程語意、Local Repo／Remote Store、Spectra App 2.3.1 行為參考起源及目前狀態摘要的概念對等。繁體中文散文 SHALL 使用 `openspec/LANGUAGE.md` 的正典詞彙；引擎動詞、CLI 命令、欄位名與程式識別符 SHALL 保留於 code span，不以避免詞取代使用者文案。

#### Scenario: 語言切換不遺失流程資訊

- **WHEN** 使用者在任一成對文件切換繁體中文與英文版本
- **THEN** 兩版呈現相同流程階段、決策分支、能力列與限制，只改變自然語言，不新增或遺漏事實

#### Scenario: 繁體中文採正典詞彙

- **WHEN** 繁體中文文件描述 discuss promote、promoted discussion 與 archive
- **THEN** 散文分別使用「轉為變更」「已轉出變更」「封存」，`promote`、`promoted`、`archive` 只在 CLI／欄位／code span 或必要的引擎動詞對照中出現

#### Scenario: 截圖引用兩版一致

- **WHEN** 並列比對任一組中英成對文件的截圖引用
- **THEN** 兩版引用相同的檔名序列與相同的引用數量，繁中版的每個引用都在 `docs/assets/screenshots/zh-TW/`、英文版的每個引用都在 `docs/assets/screenshots/en/`

#### Scenario: README 語言切換互指

- **WHEN** 使用者在 GitHub 開啟版本庫首頁
- **THEN** 顯示的 `README.md` 為英文版，頂部語言切換連到 `README.zh-TW.md`；`README.zh-TW.md` 的語言切換連回 `README.md`；README 與 docs 沒有任何連結指向根目錄的 `README.en.md`（品牌資產與外掛目錄各自的 `README.en.md` 不在此限）

### Requirement: 開發者入口文件雙語對

docs/development.md（英文）與 docs/development.zh-TW.md（繁體中文）SHALL 存在且章節結構與事實內容對等（僅語言不同），並受既有「中英文文件保持結構與事實對等」與「文件準確性具可重複驗證清單」需求約束。內容 SHALL 涵蓋：npm run dev、npm run dev:server、npm run dev:desktop、npm run dev:reset、npm run cli 五個入口各自的用途、前置條件與預期可觀察結果；下載安裝檔的未簽章放行步驟（macOS 系統設定放行、Windows SmartScreen 仍要執行），放行一節的產物名稱 SHALL 與 desktop-release 規格一致。文件中出現的指令 SHALL 全部為已驗證可執行的入口。README.md 與 README.zh-TW.md SHALL 各含一處指向對應語言 development 文件的連結。

#### Scenario: 一鍵入口涵蓋完整

- **WHEN** 檢視 development 文件任一語言版
- **THEN** 五個 npm 入口各有一節，敘明用途、前置條件與預期可觀察結果（如 dev:server 會印出 /setup 連結、dev:desktop 會開啟本地模式視窗）

#### Scenario: 雙語結構對等

- **WHEN** 對照 docs/development.md 與 docs/development.zh-TW.md 的章節骨架
- **THEN** 兩檔章節一一對應、事實一致，僅語言不同

#### Scenario: README 導流

- **WHEN** 檢視 README.md 與 README.zh-TW.md
- **THEN** README.md 含指向 docs/development.md 的連結、README.zh-TW.md 含指向 docs/development.zh-TW.md 的連結

#### Scenario: 未簽章放行教學可依循

- **WHEN** 使用者依放行一節在 macOS 開啟未簽章 dmg 安裝的 app、或在 Windows 執行未簽章安裝器
- **THEN** 文件步驟足以完成放行（macOS：系統設定 > 隱私權與安全性的強制打開；Windows：SmartScreen 其他資訊 > 仍要執行），且步驟中的產物名稱與實際 Release assets 相符

### Requirement: 使用者文件以截圖呈現實際介面

面向使用者的文件 SHALL 內嵌 desktop 與 server 後台的截圖，使讀者在安裝前即可判斷產品樣貌。截圖 SHALL 分兩組存放於 `docs/assets/screenshots/en/`（英文介面）與 `docs/assets/screenshots/zh-TW/`（繁中介面），兩組檔名 SHALL 逐一相同；英文文件 SHALL 只引用英文介面的截圖，繁中文件 SHALL 只引用繁中介面的截圖。`README.md` 與 `README.zh-TW.md` SHALL 於第一個 H2 之前內嵌一張 desktop 看板截圖，並 SHALL 於畫面一覽段內嵌至少三張截圖，其中至少一張為 server 後台。截圖 SHALL 以相對路徑內嵌於版本庫，SHALL NOT 依賴外部圖床。

截圖場景 SHALL 由版本庫內的腳本佈置為不含任何使用者真實資料的示範 workspace；該腳本 SHALL 在佈置前備份 desktop 的使用者狀態目錄，並在收尾或中斷時還原，SHALL 於 app 執行中時拒絕開始而不代為結束 app。示範 workspace 的 git 身分 SHALL 為腳本內建的示範身分，SHALL NOT 沿用使用者的全域 git 設定。截圖腳本 SHALL 列出每一個截圖檔名與兩個語言目錄；每個檔名在每個語言目錄的截圖 SHALL 至少被一份同語言的使用者文件引用，SHALL NOT 留下沒有文件使用的截圖。

截圖 SHALL 只含 app 視窗或瀏覽器頁面本身，SHALL NOT 含作業系統選單列、Dock、桌布、通知或其他 app。截圖畫面 SHALL NOT 出現使用者帳號名稱、含家目錄或系統暫存目錄的路徑、`example.com` 與 `speclink.invalid` 以外的信箱、主機名稱、使用者真實的專案或連線名稱、存取金鑰明文或前綴、setup token 與 server 資料目錄路徑。截圖檔 SHALL NOT 帶 PNG 文字中繼資料區塊（`tEXt`、`iTXt`、`zTXt`、`eXIf`）。

#### Scenario: README 可見產品畫面

- **WHEN** 讀者在 GitHub 上開啟任一語言的 README
- **THEN** 第一個 H2 之前可見一張該語言介面的 desktop 看板截圖，畫面一覽段可見至少三張該語言介面的截圖（含 server 後台），圖片皆以版本庫內的相對路徑載入

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

### Requirement: 安裝章節載明桌面 app 與 CLI 的佈署衝突

getting-started（中英）的安裝節 SHALL 載明桌面 app 與 CLI 共用同一個佈署位置所造成的覆蓋行為：macOS 上桌面 app 於每次啟動檢查該位置，未安裝或版本與 app 不同時刪除原有檔案並換為指向內建 CLI 的 symlink，Linux AppImage 僅於版本不符時覆蓋，Windows 由安裝器管理而不動該位置。說明 SHALL 一併給出保留自有 CLI 的做法（改安裝目錄並調整 PATH 順序），SHALL 指出釘選版本會一併失效，SHALL 列出安裝腳本可用的環境變數（安裝位置、釘選版本、npm registry），並 SHALL 為既有 deb 安裝者註明遷移方式（移除 deb 套件後改裝 AppImage）。README（中英）的安裝段 SHALL 以連結導向該節，SHALL NOT 重複上述覆蓋表、環境變數表與 deb 遷移說明。

#### Scenario: 先裝 CLI 再裝桌面 app 的人讀得到後果

- **WHEN** 讀者在 README 的安裝段比較桌面 app 與 CLI 兩條路
- **THEN** README 提供連到 getting-started 安裝節的連結，該節可見覆蓋行為的逐平台差異、對釘選版本的影響，以及保留自有 CLI 的具體做法

#### Scenario: deb 安裝者讀得到遷移方式

- **WHEN** 既有 deb 安裝者在任一語言的 getting-started 安裝節尋找升級方式
- **THEN** 可見「移除 deb 套件後改裝 AppImage」的說明，且 README 與 getting-started 的安裝入口都不再列 deb

## ADDED Requirements

### Requirement: 開源社群檔案引導貢獻與漏洞回報

版本庫根目錄 SHALL 提供英文的 `CONTRIBUTING.md` 與 `SECURITY.md`；`.github/ISSUE_TEMPLATE/` SHALL 提供 bug 回報與功能建議兩份 YAML issue form 及一份 `config.yml`；`.github/` SHALL 提供 `PULL_REQUEST_TEMPLATE.md`。

`CONTRIBUTING.md` SHALL 涵蓋：開發環境入口（連到 `docs/development.md`）、送出 PR 前要執行的測試指令、Conventional Commits 規範、以 Speclink 變更（`openspec/changes/`）描述行為改動的做法，以及 issue 與 PR 可用英文或繁體中文撰寫。

`SECURITY.md` SHALL 指示以 GitHub 私下漏洞回報（Security 分頁的 Report a vulnerability）回報漏洞，SHALL 載明受支援的版本為最新發布版本，SHALL NOT 要求在公開 issue 揭露漏洞細節，SHALL NOT 列出任何個人信箱。

bug 回報表單 SHALL 以必填欄位要求 Speclink 版本、作業系統、使用模式（Local 或 Remote）、重現步驟、預期結果與實際結果，並 SHALL 提醒回報者不要貼上存取金鑰、setup token 或內部網址。`config.yml` SHALL 提供一個指向 SECURITY 私下回報的聯絡連結。兩版 README 的參與貢獻段 SHALL 連到 `CONTRIBUTING.md` 與 `SECURITY.md`。

#### Scenario: 外部貢獻者找到送 PR 的方式

- **WHEN** 外部貢獻者從任一語言 README 的參與貢獻段開啟 `CONTRIBUTING.md`
- **THEN** 可見開發環境入口連結、送出 PR 前要執行的測試指令、Conventional Commits 規範，以及可用英文或繁體中文撰寫 issue 與 PR 的說明

#### Scenario: 漏洞回報者被導向私下管道

- **WHEN** 發現漏洞的人開啟 `SECURITY.md`，或在新增 issue 的選單中尋找漏洞回報方式
- **THEN** 文件與選單都指向 GitHub 私下漏洞回報，不要求公開 issue，且全文不含個人信箱

#### Scenario: bug 回報表單要求必要資訊

- **WHEN** 使用者以 bug 回報表單新增 issue 而未填 Speclink 版本或重現步驟
- **THEN** GitHub 依表單的必填設定拒絕送出；表單說明可見「不要貼上存取金鑰、setup token 或內部網址」的提醒

## MODIFIED Requirements

### Requirement: 使用者文件採漸進揭露與單一責任

Speclink 使用者文件 SHALL 以 README、getting-started、workflow、product-status 與 roadmap 形成漸進揭露入口。README SHALL 保留品牌圖片、標語「一套 SDD Engine，支援 Local Repo 與 Remote Store」、語言切換、Rust SDD 引擎與工具平台定位、PM／PO／RD／AI Agent 共用 change／artifact／task／verify／archive 語意、Local Repo／Remote Store 雙路徑、設計之初以 Spectra App 2.3.1 CLI 為行為參考的歷史起源，並 SHALL 提供由 product-status 校正的目前狀態摘要、最短流程心智模型、Local Repo 開始入口與文件地圖。getting-started SHALL 只承載可直接完成的 Local Repo 第一輪，workflow SHALL 作為完整使用流程正典，product-status SHALL 作為目前能力狀態正典，roadmap SHALL 作為尚未完成的方向的唯一落點。各文件 SHALL 以連結導向下一層細節，SHALL NOT 在 README 或 getting-started 複製完整架構與狀態矩陣。

#### Scenario: README 保留專案定位與起源

- **WHEN** 使用者開啟繁體中文或英文 README 判斷 Speclink 是什麼及為何存在
- **THEN** 首段可見品牌圖片、SDD Engine 標語、語言切換、Rust 實作與共同流程語意、Local Repo／Remote Store 說明及 Spectra App 2.3.1 行為參考起源，後續目前狀態清楚區分已可運作與規劃中內容，並連到專案路線圖
- **AND** 文件整理只校正過時事實、術語與連結，不得將上述首段改成僅含導覽連結的入口

#### Scenario: 首次使用者由 README 到完成第一輪

- **WHEN** 首次使用者從繁體中文或英文 README 尋找安裝與第一輪 Local Repo 操作
- **THEN** README 提供可見的 getting-started 入口，getting-started 以目前存在的 CLI／skill 完成 init、提案、實作檢查與封存，遇到選用分支時可連到 workflow

#### Scenario: 進階使用者查詢目前能力與目標

- **WHEN** 使用者要判斷 Desktop Remote Workspace 或 Server 某項能力目前是否可用及最終目標
- **THEN** README 導向 product-status 取得目前狀態、證據與限制，並由該列導向專案路線圖取得之後的方向，兩者 SHALL NOT 混成同一狀態描述

### Requirement: 中英文文件保持結構與事實對等

`README.md`／`README.en.md`，以及 `docs/` 下每一組 `<名稱>.zh-TW.md`／`<名稱>.md`（getting-started、workflow、product-status、roadmap、remote-getting-started、development、configuration、verb-contract、sdk-node、server-deployment、server-store-drivers、server-backup）SHALL 分別保持相同的 H2 章節集合與順序、狀態矩陣列集合、命令語意、截圖引用集合及交叉連結。每個 H2 SHALL 寫成 `## <a id="<錨點>"></a><標題>`，兩版的錨點序列 SHALL 逐項相同；標題文字 SHALL 各用該版的語言，SHALL NOT 在單一語言版本使用雙語標題。兩版 README SHALL 共用品牌圖片與截圖，並保持標語、產品定位、共同流程語意、Local Repo／Remote Store、Spectra App 2.3.1 行為參考起源及目前狀態摘要的概念對等。繁體中文散文 SHALL 使用 `openspec/LANGUAGE.md` 的正典詞彙；引擎動詞、CLI 命令、欄位名與程式識別符 SHALL 保留於 code span，不以避免詞取代使用者文案。

#### Scenario: 語言切換不遺失流程資訊

- **WHEN** 使用者在任一成對文件切換繁體中文與英文版本
- **THEN** 兩版呈現相同流程階段、決策分支、能力列與限制，只改變自然語言，不新增或遺漏事實

#### Scenario: 繁體中文採正典詞彙

- **WHEN** 繁體中文文件描述 discuss promote、promoted discussion 與 archive
- **THEN** 散文分別使用「轉為變更」「已轉出變更」「封存」，`promote`、`promoted`、`archive` 只在 CLI／欄位／code span 或必要的引擎動詞對照中出現

#### Scenario: 截圖引用兩版一致

- **WHEN** 並列比對任一組中英成對文件的截圖引用
- **THEN** 兩版引用相同的圖片檔與相同的引用數量

### Requirement: 文件準確性具可重複驗證清單

版本庫 SHALL 提供可重複執行的文件查核（`node --test scripts/*.test.mjs scripts/*/*.test.mjs` 涵蓋的文件守門測試），改動使用者文件的變更 SHALL 在 tasks 中執行它：所有相對 Markdown 連結目標存在；中英文成對文件的 H2 錨點序列一致；product-status 成對矩陣列一致且每列有證據、限制／下一步與查核日期；getting-started／workflow 的 CLI 命令與旗標可由目前 help 觀察；skill 名存在於相應生成 surface；繁體中文散文遵循正典詞彙。已確認缺失且刻意延後的文件 SHALL 以純文字缺口呈現，SHALL NOT 建立失效連結或空白 placeholder。

#### Scenario: 文件連結與語言對等查核通過

- **WHEN** 維護者執行文件守門測試的相對連結與中英文 H2 錨點／矩陣結構查核
- **THEN** 所有實際連結目標存在、成對文件結構與狀態列一致，且查核以 exit code 0 完成

#### Scenario: 不存在的 skill 或旗標使查核失敗

- **WHEN** getting-started 或 workflow 把目前 help／生成 surface 不存在的 skill、CLI 子指令或旗標寫成可直接使用
- **THEN** 文件查核以非零結果指出該名稱，該文件 SHALL 在交付前修正為現行入口或明確的 Partial／Planned 限制

#### Scenario: 已知文件缺口不偽裝成完成

- **WHEN** 稽核發現一份已被引用但尚未撰寫的進階文件（例如「從零做一個客戶端」），且當下的變更明確不補其內容
- **THEN** 文件地圖或 product-status 以無超連結的缺口項目記錄並導向後續 change，SHALL NOT 產生空檔、失效連結或宣稱其已交付

### Requirement: 帳號、PAT 與 membership 的操作邊界明確

Remote Getting Started SHALL 指示瀏覽器開啟 `/account` 管理自身 PAT，由該頁送出 POST `/api/speclink/v1/web/account/tokens`；文件 SHALL 明說以瀏覽器 GET 開啟 `/api/speclink/v1/web/account/tokens` 會得到 HTTP 405 Method Not Allowed。PAT 沒有專案範圍，權限等於帳號在各專案的 membership。文件 SHALL 說明建立 Project／Repo registry 不會授予 membership，Server Admin 身分不會繞過 Project membership；管理員 SHALL 由 `/admin/users` 對帳號授予或更新 `reader`／`editor`，Desktop scope 清單才顯示該 Project 及其 Repos。PAT 明文只顯示一次，文件 SHALL NOT 以 URL、repo 設定或帶值的 shell argument 示範保存 PAT。

#### Scenario: 使用者直接開啟 PAT action URL

- **WHEN** 使用者在瀏覽器網址列開啟 `http://localhost:8080/api/speclink/v1/web/account/tokens`
- **THEN** 故障排除指出 405 代表該網址只接受 POST，並導向 `http://localhost:8080/account` 登入後由存取金鑰表單建立 PAT

#### Scenario: Desktop scopes 回傳空清單

- **WHEN** Desktop 顯示「此帳號目前沒有任何 Project／Repo membership」但 registry 已有 Project 與 Repo
- **THEN** 故障排除指示管理員到 `/admin/users` 對 Desktop 實際登入帳號授予該 Project 的 `reader` 或 `editor`，說明 Admin flag 不是 scope bypass，並要求回到 Desktop 重新載入 chooser

#### Scenario: PAT 不進入不安全載體

- **WHEN** 使用者選擇 Desktop PAT fallback 或 CLI `auth login`
- **THEN** 教學要求從 `/account` 複製只顯示一次的 PAT並貼入應用或 stdin，不把真實 PAT 放進 URL、`.speclink.yaml`、repo、文件範例或 shell history

### Requirement: Remote 教學具雙語導流與可重複查核

README 與產品能力狀態的中英文版本 SHALL 連到對應語言的 Remote Getting Started，既有繁中 Server 部署指南 SHALL 連到繁中 Remote Getting Started；兩份新教學 SHALL 維持相同 H2 集合與順序、命令語意、網址角色、membership 規則、故障排除症狀集合及目前能力邊界。文件查核 SHALL 驗證所有相對 Markdown 連結目標存在、現行 Server 路由與 CLI 指令可由 source／help 觀察、繁中與英文 H2 對等，並 SHALL 驗證關鍵字串涵蓋 `/account`、POST `/api/speclink/v1/web/account/tokens`、`/admin/users`、403、`permission_denied`、membership、project-scoped URL、spec-only、checkout、offline 與 `npm run dev:reset`。

#### Scenario: 使用者從既有文件找到 Remote 教學

- **WHEN** 使用者從任一語言 README、任一語言產品能力狀態或繁中 Server 部署指南尋找 Remote Server／Desktop／CLI 的首次操作方式
- **THEN** 文件提供有效連結到同語言 Remote Getting Started

#### Scenario: 雙語與入口查核阻止文件漂移

- **WHEN** 維護者執行 tasks 指定的雙語 H2、相對連結、路由、CLI surface 與關鍵流程檢查
- **THEN** 全部檢查以 exit code 0 完成；任一語言缺少步驟、連到不存在檔案、把 `/api/speclink/v1/web/account/tokens` 寫成可直接開啟的頁面或引用不存在指令時以非零結果指出缺口

### Requirement: 安裝通路文件與發布狀態誠實化

README（中英）SHALL 提供安裝區塊：桌面三平台安裝檔的下載入口（macOS 單一 universal dmg 並註明兩種晶片同一檔、Windows 安裝器、Linux AppImage 依架構各一）、CLI 的三條一行安裝指令（npm 全域安裝、安裝腳本 curl 一行、Homebrew tap），並 SHALL 註明 Windows 沒有安裝腳本（走 npm 或桌面安裝器）、無圖形介面的 Linux 走 CLI 一行安裝，將從原始碼建置的安裝方式降為開發者導向段落；getting-started（中英）的安裝節 SHALL 與 README 呈現的通路一致。文件 SHALL NOT 指示尚不存在的安裝入口；發布管線已接、只待首個 release 才上架的通路 SHALL 以「管線已接＋生效時點」表述，SHALL NOT 讓讀者誤以為現在就裝得到：@speclink/engine 的 sdk-node 文件（中英）SHALL 以 `npm install @speclink/engine` 為主路徑，同段 SHALL 寫明可從 npm 安裝的起始版本（0.2.0），並 SHALL 保留自 repo 建置作為替代路徑。中英兩語版本 SHALL 維持結構與事實對等。

#### Scenario: README 安裝區塊涵蓋桌面與 CLI 通路

- **WHEN** 使用者開啟任一語言的 README 尋找安裝方式
- **THEN** 安裝區塊列出桌面三平台的下載入口（macOS 單一 universal dmg、Windows 安裝器、Linux AppImage）與 CLI 的三條一行安裝指令（npm、安裝腳本、Homebrew tap），不出現 deb 與 PowerShell 安裝腳本，從原始碼建置位於開發者導向段落而非首選位置

#### Scenario: sdk-node 以 npm install 為主路徑並標注生效時點

- **WHEN** 讀者依任一語言的 sdk-node 文件嘗試取得 @speclink/engine
- **THEN** 文件以 `npm install @speclink/engine` 為主路徑，同段寫明自 0.2.0 起可從 npm 安裝，且自 repo 建置仍以替代路徑呈現

### Requirement: 使用者文件以截圖呈現實際介面

面向使用者的文件 SHALL 內嵌 desktop 與 server 後台的截圖，使讀者在安裝前即可判斷產品樣貌。`README.md` 與 `README.en.md` SHALL 於定位段落之後至少內嵌一張 desktop 截圖。截圖 SHALL 以相對路徑內嵌於版本庫、中英兩版共用同一組圖片檔，SHALL NOT 依賴外部圖床。截圖場景 SHALL 由版本庫內的腳本佈置為不含任何使用者真實資料的示範 workspace；該腳本 SHALL 在佈置前備份 desktop 的使用者狀態目錄，並在收尾或中斷時還原，SHALL 於 app 執行中時拒絕開始而不代為結束 app。截圖腳本列出的每一張截圖 SHALL 至少被一份使用者文件引用，SHALL NOT 留下沒有文件使用的截圖。

#### Scenario: README 可見產品畫面

- **WHEN** 讀者在 GitHub 上開啟任一語言的 README
- **THEN** 定位段落之後可見至少一張 desktop 截圖，圖片以版本庫內的相對路徑載入

#### Scenario: 沒有孤兒截圖

- **WHEN** 執行文件守門測試
- **THEN** 截圖腳本列出的每一張截圖都至少出現在 README 或 docs/ 的一份文件中；任一張沒有被引用時測試以非零結果點名該檔

#### Scenario: 拍攝不損毀使用者既有狀態

- **WHEN** 執行截圖場景腳本並在拍攝中途以中斷訊號結束
- **THEN** 腳本仍還原使用者原本的 desktop 狀態目錄，使用者的 workspace 分頁與 Server 連線設定與執行前一致

#### Scenario: app 執行中拒絕開始

- **WHEN** desktop app 正在執行時啟動截圖場景腳本
- **THEN** 腳本以非零結束並說明須先關閉 app，不搬移任何目錄、不代為結束 app

### Requirement: 使用者面路線圖與內部交付順序分列

版本庫 SHALL 提供面向使用者的路線圖文件 `docs/roadmap.zh-TW.md` 與 `docs/roadmap.md`，涵蓋 SDK 發布、以引擎自建客戶端與 server 端（使用者以 SDK 引擎自行開發桌面、其他前端或自家 server）、遠端協作完整化、agent 工具整合與系統整合五條方向，每條載明要解決的問題、目前進度與可觀察的下一步。該文件 SHALL NOT 出現版本號或日期承諾。尚未完成的方向 SHALL 集中寫在路線圖；product-status 的規劃中列 SHALL 只連到路線圖，SHALL NOT 另行敘述規劃內容。

#### Scenario: 對外方向不含時程承諾

- **WHEN** 讀者開啟任一語言的使用者面路線圖
- **THEN** 五條方向各自可見問題、進度與下一步，且全文不含版本號或日期形式的交付承諾

### Requirement: 安裝章節載明桌面 app 與 CLI 的佈署衝突

安裝章節 SHALL 載明桌面 app 與 CLI 共用同一個佈署位置所造成的覆蓋行為：macOS 上桌面 app 於每次啟動檢查該位置，未安裝或版本與 app 不同時刪除原有檔案並換為指向內建 CLI 的 symlink，Linux AppImage 僅於版本不符時覆蓋，Windows 由安裝器管理而不動該位置。說明 SHALL 一併給出保留自有 CLI 的做法（改安裝目錄並調整 PATH 順序），SHALL 指出釘選版本會一併失效，並 SHALL 為既有 deb 安裝者註明遷移方式（移除 deb 套件後改裝 AppImage）。

#### Scenario: 先裝 CLI 再裝桌面 app 的人讀得到後果

- **WHEN** 讀者在安裝章節比較桌面 app 與 CLI 兩條路
- **THEN** 可見覆蓋行為的逐平台差異、對釘選版本的影響，以及保留自有 CLI 的具體做法

#### Scenario: deb 安裝者讀得到遷移方式

- **WHEN** 既有 deb 安裝者在任一語言的安裝章節尋找升級方式
- **THEN** 可見「移除 deb 套件後改裝 AppImage」的說明，且章節內不再列 deb 為安裝入口


## REMOVED Requirements

### Requirement: 目標架構與目前狀態維持清楚邊界

**Reason**: 平台架構藍圖（`docs/design/platform-architecture.zh-TW.md`）與實作重構路線圖（`docs/design/implementation-refactor-roadmap.zh-TW.md`）已於 2026-10-07 的文件整理移除；仍有效的未來規劃已併入使用者面路線圖。

**Migration**: 目前能不能用看 `docs/product-status.zh-TW.md`；之後的方向看 `docs/roadmap.zh-TW.md`；行為的正式定義看 `openspec/specs/`。

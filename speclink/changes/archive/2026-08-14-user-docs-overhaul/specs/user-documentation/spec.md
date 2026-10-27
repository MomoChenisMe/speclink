## ADDED Requirements

### Requirement: 使用者文件以截圖呈現實際介面

面向使用者的文件 SHALL 內嵌 desktop 與 server 後台的截圖，使讀者在安裝前即可判斷產品樣貌。`README.md` 與 `README.en.md` SHALL 於定位段落之後至少內嵌一張 desktop 截圖。截圖 SHALL 以相對路徑內嵌於版本庫、中英兩版共用同一組圖片檔，SHALL NOT 依賴外部圖床。截圖場景 SHALL 由版本庫內的腳本佈置為不含任何使用者真實資料的示範 workspace；該腳本 SHALL 在佈置前備份 desktop 的使用者狀態目錄，並在收尾或中斷時還原，SHALL 於 app 執行中時拒絕開始而不代為結束 app。

#### Scenario: README 可見產品畫面

- **WHEN** 讀者在 GitHub 上開啟任一語言的 README
- **THEN** 定位段落之後可見至少一張 desktop 截圖，圖片以版本庫內的相對路徑載入

#### Scenario: 拍攝不損毀使用者既有狀態

- **WHEN** 執行截圖場景腳本並在拍攝中途以中斷訊號結束
- **THEN** 腳本仍還原使用者原本的 desktop 狀態目錄，使用者的 workspace 分頁與 Server 連線設定與執行前一致

#### Scenario: app 執行中拒絕開始

- **WHEN** desktop app 正在執行時啟動截圖場景腳本
- **THEN** 腳本以非零結束並說明須先關閉 app，不搬移任何目錄、不代為結束 app

### Requirement: 工作流正典逐站列出技能與完成判準

`docs/workflow.zh-TW.md` 與 `docs/workflow.md` SHALL 以單一結構列出 SDD 全部站別——onboard、discuss、improve、propose、apply、ingest、quality、review、verify、archive 與 worktree 流程——每站 SHALL 載明用途、對應的 `/speclink-*` 技能名稱、完成判準與下一站。讀者 SHALL NOT 需要跨文件拼湊任一站的上述四項資訊。

#### Scenario: 逐站資訊完整

- **WHEN** 讀者在任一語言的工作流文件查找任一站
- **THEN** 該站的用途、對應技能名稱、完成判準與下一站四項均可在該文件內找到

### Requirement: 本地與遠端能力對照集中呈現

`docs/product-status.zh-TW.md` 與 `docs/product-status.md` SHALL 提供本地與遠端兩條路徑的能力對照，逐項標示各路徑的可用狀態，作為兩者差異的單一入口。查核日期 SHALL 反映最近一次查核，且文件 SHALL NOT 記載已不成立的文件缺口。

#### Scenario: 差異可一次讀完

- **WHEN** 讀者想知道某項能力在本地與遠端是否都可用
- **THEN** 於產品能力狀態文件的對照中可同時看到該能力在兩條路徑的狀態，無須開啟其他文件

#### Scenario: 缺口記載與現況一致

- **WHEN** 檢視文件中記載的已知文件缺口
- **THEN** 每一條缺口所指的文件確實不存在於版本庫

### Requirement: 使用者面路線圖與內部交付順序分列

版本庫 SHALL 提供面向使用者的路線圖文件 `docs/roadmap.zh-TW.md` 與 `docs/roadmap.md`，涵蓋 SDK 發布、以引擎自建客戶端與 server 端（使用者以 SDK 引擎自行開發桌面、其他前端或自家 server）、遠端協作完整化、agent 工具整合與系統整合五條方向，每條載明要解決的問題、目前進度與可觀察的下一步。該文件 SHALL NOT 出現版本號或日期承諾。使用者文件 SHALL NOT 引用維護者自用的架構文件（`docs/platform-architecture.zh-TW.md`、`docs/implementation-refactor-roadmap.zh-TW.md`）——那兩份不面向使用者且將被移除，任何指向它們的連結都是未來的斷鏈。

#### Scenario: 對外方向不含時程承諾

- **WHEN** 讀者開啟任一語言的使用者面路線圖
- **THEN** 五條方向各自可見問題、進度與下一步，且全文不含版本號或日期形式的交付承諾

### Requirement: 文件內部連結全部可解析

全部使用者文件的相對連結與圖片路徑 SHALL 指向版本庫中存在的檔案，且此性質 SHALL 可由一道可重複執行的檢查驗證。

#### Scenario: 連結掃描零斷鏈

- **WHEN** 掃描全部使用者文件的相對連結與圖片路徑
- **THEN** 每一個路徑都對應到版本庫中存在的檔案

### Requirement: 使用者文件採簡化技術英文的寫作紀律

全部使用者文件的散文 SHALL 依 ASD-STE100（Simplified Technical English）的紀律撰寫：一句只講一件事、句子短、用主動語態並點名動作者、同一個動作固定用同一個動詞而不輪換近義詞、三個以上的步驟或並列條件改用清單而不埋進一句話裡、段落先講結果再講細節。英文版 SHALL NOT 使用完成式。專案動詞名與站別名（`validate`、`verify`、`review`、`archive` 等）SHALL 保留原字，不因換詞規則被改寫——它們是識別符而非用語選擇。程式碼區塊、行內程式碼、路徑、識別符與引用原文 SHALL 維持原樣。

此紀律 SHALL 以逐份文件的人工通讀落實，SHALL NOT 以正則式腳本代行——句子是否只講一件事、動詞是否輪換、段落是否先講結果，都不是機器判得準的性質。

#### Scenario: 逐份通讀後無長句與被動堆疊

- **WHEN** 逐份通讀任一使用者文件的散文
- **THEN** 句子各只講一件事、動作者可辨識，且三個以上的並列項以清單而非單句呈現

#### Scenario: 同一動作不輪換動詞

- **WHEN** 同一份文件多次描述同一個動作
- **THEN** 該動作每次都用同一個動詞，近義詞不交替出現

### Requirement: 使用者文件載明本地產物的 OpenSpec 結構相容性

使用者文件 SHALL 說明 **Local 模式**的產物沿用 OpenSpec 的目錄結構（並明示此相容性不適用遠端模式——遠端的正典在 Store，本機只有唯讀投影）——`specs/<capability>/spec.md`、`changes/<名稱>/`、`changes/archive/` 與 `config.yaml`——且內容為純 Markdown 與 YAML、可不經 Speclink 讀寫、每次變動皆呈現於 Git diff。說明 SHALL 一併載明 Speclink 在該結構上的兩項擴充：`discussions/` 與各變更目錄的 `.openspec.yaml`。此說明 SHALL 出現於 README 中英兩版與入門教學中英兩版。

#### Scenario: 讀者判斷得出資料可攜

- **WHEN** 讀者在 README 或入門教學讀到本地產物的說明
- **THEN** 可見沿用的目錄結構、純文字格式的事實，以及 Speclink 額外新增的兩樣東西

### Requirement: 安裝章節載明桌面 app 與 CLI 的佈署衝突

安裝章節 SHALL 載明桌面 app 與 CLI 共用同一個佈署位置所造成的覆蓋行為：macOS 上桌面 app 於每次啟動將該位置換為指向內建 CLI 的 symlink 並刪除原有檔案，Linux AppImage 僅於版本不符時覆蓋，Windows 與 deb 由安裝器與套件管理器管理而不動該位置。說明 SHALL 一併給出保留自有 CLI 的做法（改安裝目錄並調整 PATH 順序），並指出釘選版本會一併失效。

#### Scenario: 先裝 CLI 再裝桌面 app 的人讀得到後果

- **WHEN** 讀者在安裝章節比較桌面 app 與 CLI 兩條路
- **THEN** 可見覆蓋行為的逐平台差異、對釘選版本的影響，以及保留自有 CLI 的具體做法

### Requirement: 官方 server 定位為參考實作而非唯一路徑

使用者文件描述遠端模式時 SHALL 說明 `speclink-server` 是官方的參考實作，用途是開箱即用與試用遠端功能，SHALL NOT 把它寫成遠端模式的唯一路徑。文件 SHALL 說明遠端模式由 Host 與 Protocol 兩份公開契約定義（`openspec/specs/` 的 `host-runtime` 與 `client-protocol`），使用者得以 Speclink 引擎自行實作 server 端並接上既有的認證、資料庫與權限模型，且 CLI 與桌面 app 對自建 server 同樣可用。此說明 SHALL 出現於 README 中英兩版、Remote 入門教學中英兩版、Node SDK 中英兩版與 Server 部署文件。

#### Scenario: 讀者判斷得出 server 端可替換

- **WHEN** 讀者在 README 或 Remote 入門教學讀到遠端模式
- **THEN** 可見官方 server 是參考實作的陳述、定義遠端模式的兩份契約名稱，以及自建 server 端的可行性

#### Scenario: 能力對照表標明量測對象

- **WHEN** 讀者查閱本地與遠端能力對照表
- **THEN** 表格載明 Remote 欄以官方參考 server 為量測對象，且同樣的欄位適用於自建 server

## MODIFIED Requirements

### Requirement: 中英文文件保持結構與事實對等

`README.md`／`README.en.md`、`docs/getting-started.zh-TW.md`／`docs/getting-started.md`、`docs/workflow.zh-TW.md`／`docs/workflow.md`、`docs/product-status.zh-TW.md`／`docs/product-status.md`、`docs/roadmap.zh-TW.md`／`docs/roadmap.md` SHALL 分別保持相同的 H2 章節集合與順序、狀態矩陣列集合、命令語意、截圖引用集合及交叉連結。兩版 README SHALL 共用品牌圖片與截圖，並保持標語、產品定位、共同流程語意、Local Repo／Remote Store、Spectra App 2.3.1 行為參考起源及目前狀態摘要的概念對等。繁體中文散文 SHALL 使用 `openspec/LANGUAGE.md` 的正典詞彙；引擎動詞、CLI 命令、欄位名與程式識別符 SHALL 保留於 code span，不以避免詞取代使用者文案。僅提供單一語言的 server 部署、Store driver 選型與備份還原文件不在本要求範圍。

#### Scenario: 語言切換不遺失流程資訊

- **WHEN** 使用者在任一成對文件切換繁體中文與英文版本
- **THEN** 兩版呈現相同流程階段、決策分支、能力列與限制，只改變自然語言，不新增或遺漏事實

#### Scenario: 繁體中文採正典詞彙

- **WHEN** 繁體中文文件描述 discuss promote、promoted discussion 與 archive
- **THEN** 散文分別使用「轉為變更」「已轉出變更」「封存」，`promote`、`promoted`、`archive` 只在 CLI／欄位／code span 或必要的引擎動詞對照中出現

#### Scenario: 截圖引用兩版一致

- **WHEN** 並列比對任一組中英成對文件的截圖引用
- **THEN** 兩版引用相同的圖片檔與相同的引用數量

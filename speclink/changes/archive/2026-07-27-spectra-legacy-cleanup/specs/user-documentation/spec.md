## MODIFIED Requirements

### Requirement: 使用者文件採漸進揭露與單一責任

Speclink 使用者文件 SHALL 以 README、getting-started、workflow、product-status、平台架構與實作路線圖形成漸進揭露入口。README SHALL 保留品牌圖片、標語「一套 SDD Engine，支援 Local Repo 與 Remote Store」、語言切換、Rust SDD 引擎與工具平台定位、PM／PO／RD／AI Agent 共用 change／artifact／task／verify／archive 語意、Local Repo／Remote Store 雙路徑、設計之初以 Spectra App 2.3.1 CLI 為行為參考的歷史起源，並 SHALL 提供由 product-status 校正的目前狀態摘要、最短流程心智模型、Local Repo 開始入口與文件地圖。getting-started SHALL 只承載可直接完成的 Local Repo 第一輪，workflow SHALL 作為完整使用流程正典，product-status SHALL 作為目前能力狀態正典，平台架構 SHALL 維持唯一目標架構正典，實作路線圖 SHALL 維持其下的交付順序伴隨文件。各文件 SHALL 以連結導向下一層細節，SHALL NOT 在 README 或 getting-started 複製完整架構與狀態矩陣。

#### Scenario: README 保留專案定位與起源

- **WHEN** 使用者開啟繁體中文或英文 README 判斷 Speclink 是什麼及為何存在
- **THEN** 首段可見品牌圖片、SDD Engine 標語、語言切換、Rust 實作與共同流程語意、Local Repo／Remote Store 說明及 Spectra App 2.3.1 行為參考起源，後續目前狀態清楚區分已可運作與分階段建置內容，並連到實作重構路線圖
- **AND** 文件整理只校正過時事實、術語與連結，不得將上述首段改成僅含導覽連結的入口

#### Scenario: 首次使用者由 README 到完成第一輪

- **WHEN** 首次使用者從繁體中文或英文 README 尋找安裝與第一輪 Local Repo 操作
- **THEN** README 提供可見的 getting-started 入口，getting-started 以目前存在的 CLI／skill 完成 init、提案、實作檢查與封存，遇到選用分支時可連到 workflow

#### Scenario: 進階使用者查詢目前能力與目標

- **WHEN** 使用者要判斷 Desktop Remote Workspace 或 Server 某項能力目前是否可用及最終目標
- **THEN** README 導向 product-status 取得目前狀態、證據與限制，並由該列導向平台架構或實作路線圖取得目標與交付順序，兩者 SHALL NOT 混成同一狀態描述

### Requirement: 中英文文件保持結構與事實對等

`README.md`／`README.en.md`、`docs/getting-started.zh-TW.md`／`docs/getting-started.md`、`docs/workflow.zh-TW.md`／`docs/workflow.md`、`docs/product-status.zh-TW.md`／`docs/product-status.md` SHALL 分別保持相同的 H2 章節集合與順序、狀態矩陣列集合、命令語意及交叉連結。兩版 README SHALL 共用品牌圖片並保持標語、產品定位、共同流程語意、Local Repo／Remote Store、Spectra App 2.3.1 行為參考起源及目前狀態摘要的概念對等。繁體中文散文 SHALL 使用 `openspec/LANGUAGE.md` 的正典詞彙；引擎動詞、CLI 命令、欄位名與程式識別符 SHALL 保留於 code span，不以避免詞取代使用者文案。

#### Scenario: 語言切換不遺失流程資訊

- **WHEN** 使用者在任一成對文件切換繁體中文與英文版本
- **THEN** 兩版呈現相同流程階段、決策分支、能力列與限制，只改變自然語言，不新增或遺漏事實

#### Scenario: 繁體中文採正典詞彙

- **WHEN** 繁體中文文件描述 discuss promote、promoted discussion 與 archive
- **THEN** 散文分別使用「轉為變更」「已轉出變更」「封存」，`promote`、`promoted`、`archive` 只在 CLI／欄位／code span 或必要的引擎動詞對照中出現

## MODIFIED Requirements

### Requirement: 使用者文件採漸進揭露與單一責任

Speclink 使用者文件 SHALL 以 README、getting-started、workflow、product-status 與 roadmap 形成漸進揭露入口。README 第一個 H2 之前 SHALL 呈現品牌圖片、標語（繁中版「讓 AI 寫的每個變更，都對得上規格、查得到證據」，英文版「Every AI-built change, checked against its spec」）、徽章、語言切換、定位段、雙路徑段，以及一張 desktop 看板截圖。定位段 SHALL 說明 Speclink 是以 Rust 實作的 SDD 引擎，並以可驗收為主軸：每個變更先寫一小份規格、交給 AI agent 實作，封存前由審查與驗證兩道品質關卡檢查並留下證據，規格是 repo 裡的純文字；定位段 SHALL NOT 出現 PM 與 PO。雙路徑段 SHALL 說明 Local Repo／Remote Store 兩條路徑：一個人用 Local Repo 即足夠，團隊要共用規格再接 Remote Store；PM／PO／RD／AI Agent 共用 change／artifact／task／verify／archive 語意，以及 PM、PO 在團隊這一層參與討論與提案的說明，SHALL 放在雙路徑段。README SHALL 以 H2 提供：為什麼用 Speclink、快速開始、畫面一覽、特色、最短流程心智模型（工作流程）、安裝、自架 server、文件地圖、由 product-status 校正的目前狀態摘要、參與貢獻、致謝與授權；快速開始 SHALL 位於安裝之前，並以安裝 CLI、`speclink init` 與請 Agent 執行 propose 三步呈現 Local Repo 開始入口。設計之初以 Spectra App 2.3.1 CLI 為行為參考的歷史起源 SHALL 寫在致謝段，同段 SHALL 致謝 OpenSpec 的目錄結構；該起源 SHALL NOT 寫在第一個 H2 之前。getting-started SHALL 只承載可直接完成的 Local Repo 第一輪，workflow SHALL 作為完整使用流程正典，product-status SHALL 作為目前能力狀態正典，roadmap SHALL 作為尚未完成的方向的唯一落點。各文件 SHALL 以連結導向下一層細節，SHALL NOT 在 README 或 getting-started 複製完整架構與狀態矩陣。

#### Scenario: README 保留專案定位與起源

- **WHEN** 使用者開啟繁體中文或英文 README 判斷 Speclink 是什麼及為何存在
- **THEN** 第一個 H2 之前可見品牌圖片、可驗收標語、語言切換、以 Rust 實作的 SDD 引擎與可驗收定位、Local Repo／Remote Store 說明（含 PM、PO 參與的團隊情境）及 desktop 看板截圖；致謝段可見 Spectra App 2.3.1 行為參考起源與 OpenSpec；目前狀態段清楚區分已可運作與規劃中內容，並連到專案路線圖
- **AND** 文件整理只校正過時事實、術語與連結，不得將第一個 H2 之前的內容改成僅含導覽連結的入口

#### Scenario: 定位段以可驗收開場、團隊角色放在雙路徑段

- **WHEN** 讀者只讀繁體中文或英文 README 的標語與第一段
- **THEN** 標語為「讓 AI 寫的每個變更，都對得上規格、查得到證據」（英文版「Every AI-built change, checked against its spec」），第一段依序可見「先寫一小份規格」「交給 AI agent 實作」「封存前由審查與驗證兩道品質關卡檢查並留下證據」
- **AND** 第一段不出現 PM 與 PO；PM、PO 與共用語意的說明出現在緊接其後的雙路徑段，且與 Remote Store 的團隊情境寫在一起

#### Scenario: 首次使用者由 README 到完成第一輪

- **WHEN** 首次使用者從繁體中文或英文 README 尋找安裝與第一輪 Local Repo 操作
- **THEN** README 的快速開始段以三步呈現安裝 CLI、`speclink init` 與請 Agent 執行 propose，並提供可見的 getting-started 入口；getting-started 以目前存在的 CLI／skill 完成 init、提案、實作檢查與封存，遇到選用分支時可連到 workflow

#### Scenario: 進階使用者查詢目前能力與目標

- **WHEN** 使用者要判斷 Desktop Remote Workspace 或 Server 某項能力目前是否可用及最終目標
- **THEN** README 導向 product-status 取得目前狀態、證據與限制，並由該列導向專案路線圖取得之後的方向，兩者 SHALL NOT 混成同一狀態描述

## REMOVED Requirements

### Requirement: 使用者文件載明本地產物的 OpenSpec 結構相容性

**Reason**: OpenSpec 相容性改為單向：speclink 不再保證 OpenSpec CLI 讀得回 speclink 專案，從 OpenSpec 專案匯入目前也沒有乾淨的路徑（`speclink init` 遇到既有 `openspec/` 會拒絕，`--force` 會把 config.yaml 蓋回範本），文件不得再宣稱「相容 OpenSpec」或「沿用 OpenSpec 的目錄結構」

**Migration**: 由「使用者文件載明本地產物的純文字可攜性」承接——目錄位置、純 Markdown 與 YAML、不經 Speclink 可讀寫、Git diff 可見、Speclink 的兩項擴充、只適用 Local 模式與四份文件的出現位置全部保留；拿掉與 OpenSpec 相容的宣稱，並明定不宣稱可匯入；OpenSpec 的致謝仍留在 README 致謝段

## ADDED Requirements

### Requirement: 使用者文件載明本地產物的純文字可攜性

使用者文件 SHALL 說明 **Local 模式**的產物位置——`specs/<capability>/spec.md`、`changes/<名稱>/`、`changes/archive/` 與 `config.yaml`——並說明內容為純 Markdown 與 YAML、可不經 Speclink 讀寫、每次變動皆呈現於 Git diff；說明 SHALL 明示此可攜性不適用遠端模式（遠端的正式規格在 Store，本機只有唯讀投影）。說明 SHALL 一併載明 Speclink 在這些產物上的兩項擴充：`discussions/` 與各變更目錄的 `.openspec.yaml`。此說明 SHALL 出現於 README 中英兩版與入門教學中英兩版。這些說明 SHALL NOT 宣稱與 OpenSpec 相容或沿用 OpenSpec 的目錄結構，也 SHALL NOT 宣稱可從 OpenSpec 專案匯入；README 致謝段對 OpenSpec 目錄結構的致謝不受此限。

#### Scenario: 讀者判斷得出資料可攜

- **WHEN** 讀者在 README 或入門教學（中英任一版）讀到本地產物的說明
- **THEN** 可見產物的目錄位置、純文字格式、不裝 Speclink 也能讀寫、Git diff 可見的事實，以及 Speclink 額外新增的兩樣東西，並可見此說明只適用 Local 模式

#### Scenario: 本地產物說明不承諾 OpenSpec 相容或匯入

- **WHEN** 讀者在 README 特色段或入門教學的本地產物說明中，尋找 speclink 與 OpenSpec 的關係
- **THEN** 說明中沒有「相容 OpenSpec」「沿用 OpenSpec 的目錄結構」或「可從 OpenSpec 匯入」的宣稱（英文版同樣沒有 OpenSpec-compatible、OpenSpec folder layout 或 import 的宣稱）
- **AND** README 裡的 OpenSpec 專案名稱只出現在致謝段（資料夾名稱 `openspec/` 不算在內）

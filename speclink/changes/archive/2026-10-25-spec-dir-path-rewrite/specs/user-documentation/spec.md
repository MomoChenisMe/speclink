## MODIFIED Requirements

### Requirement: 中英文文件保持結構與事實對等

<!-- BEFORE: 以 openspec/ 與 .openspec.yaml 描述規格資料夾與變更資料檔 -->

`README.zh-TW.md`／`README.md`（繁中／英文），以及 `docs/` 下每一組 `<名稱>.zh-TW.md`／`<名稱>.md`（getting-started、workflow、product-status、roadmap、remote-getting-started、development、configuration、verb-contract、sdk-node、server-deployment、server-store-drivers、server-backup）SHALL 分別保持相同的 H2 章節集合與順序、狀態矩陣列集合、命令語意、截圖檔名序列及交叉連結。英文版 SHALL 為無語言後綴的檔名，繁中版 SHALL 帶 `.zh-TW` 後綴。每個 H2 SHALL 寫成 `## <a id="<錨點>"></a><標題>`，兩版的錨點序列 SHALL 逐項相同；標題文字 SHALL 各用該版的語言，SHALL NOT 在單一語言版本使用雙語標題。兩版 README SHALL 共用品牌圖片；截圖 SHALL 以相同檔名、各自引用該語言介面的那一組。兩版 README SHALL 保持標語、產品定位、共同流程語意、Local Repo／Remote Store、Spectra App 2.3.1 行為參考起源及目前狀態摘要的概念對等。繁體中文散文 SHALL 使用 `speclink/LANGUAGE.md` 的正典詞彙；引擎動詞、CLI 命令、欄位名與程式識別符 SHALL 保留於 code span，不以避免詞取代使用者文案。

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

### Requirement: 官方 server 定位為參考實作而非唯一路徑

<!-- BEFORE: 以 openspec/ 與 .openspec.yaml 描述規格資料夾與變更資料檔 -->

使用者文件描述遠端模式時 SHALL 說明 `speclink-server` 是官方的參考實作，用途是開箱即用與試用遠端功能，SHALL NOT 把它寫成遠端模式的唯一路徑。文件 SHALL 說明遠端模式由 Host 與 Protocol 兩份公開契約定義（`speclink/specs/` 的 `host-runtime` 與 `client-protocol`），使用者得以 Speclink 引擎自行實作 server 端並接上既有的認證、資料庫與權限模型，且 CLI 與桌面 app 對自建 server 同樣可用。此說明 SHALL 出現於 README 中英兩版、Remote 入門教學中英兩版、Node SDK 中英兩版與 Server 部署文件。

#### Scenario: 讀者判斷得出 server 端可替換

- **WHEN** 讀者在 README 或 Remote 入門教學讀到遠端模式
- **THEN** 可見官方 server 是參考實作的陳述、定義遠端模式的兩份契約名稱，以及自建 server 端的可行性

#### Scenario: 能力對照表標明量測對象

- **WHEN** 讀者查閱本地與遠端能力對照表
- **THEN** 表格載明 Remote 欄以官方參考 server 為量測對象，且同樣的欄位適用於自建 server

### Requirement: 開源社群檔案引導貢獻與漏洞回報

<!-- BEFORE: 以 openspec/ 與 .openspec.yaml 描述規格資料夾與變更資料檔 -->

版本庫根目錄 SHALL 提供英文的 `CONTRIBUTING.md` 與 `SECURITY.md`；`.github/ISSUE_TEMPLATE/` SHALL 提供 bug 回報與功能建議兩份 YAML issue form 及一份 `config.yml`；`.github/` SHALL 提供 `PULL_REQUEST_TEMPLATE.md`。

`CONTRIBUTING.md` SHALL 涵蓋：開發環境入口（連到 `docs/development.md`）、送出 PR 前要執行的測試指令、Conventional Commits 規範、以 Speclink 變更（`speclink/changes/`）描述行為改動的做法，以及 issue 與 PR 可用英文或繁體中文撰寫。

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

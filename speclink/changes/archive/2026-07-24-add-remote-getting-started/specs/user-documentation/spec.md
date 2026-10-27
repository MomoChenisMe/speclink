## ADDED Requirements

### Requirement: Remote Getting Started 提供可重複的完整操作路徑

中英文 Remote Getting Started SHALL 以目前已交付的入口，提供從全新本地開發資料到可操作 Remote Desktop 與 Remote CLI 的端到端流程。文件 SHALL 依序涵蓋 `npm run dev:reset`、`npm run dev`、stdout 的 `/setup?token=...`、建立第一位 Admin 與第一組 Project／Repo、在 `/admin/users` 授予實際登入帳號 Project membership、以 `/account` 建立 PAT、Desktop device login 與 PAT fallback、Project／Repo scope 選擇、spec-only 與 checkout 分流、CLI `link`／`auth login`／讀寫 smoke test、一般重啟持久化、離線唯讀與恢復、完全重置。文件 SHALL 明確區分 Server base URL、瀏覽器帳號／管理頁 URL 與 project-scoped API URL，且 SHALL NOT 把架構目標或 Phase 4 能力寫成現行操作。

#### Scenario: 第一位 Admin 完成 Desktop Remote Workspace

- **WHEN** 使用者從沒有 `.dev/` 的本地 checkout 依繁體中文或英文教學啟動 Server 與 Desktop、完成 setup，並使用 setup 建立的 Admin 登入 Desktop
- **THEN** 教學指示使用者先在 `/admin/users` 為該 Admin 授予所建 Project 的 `editor` membership，再重新載入 Desktop scope 清單，選到所建 Project／Repo 並開啟 spec-only 或 checkout workspace

#### Scenario: Remote CLI 使用 project-scoped URL

- **WHEN** 使用者依教學在獨立測試資料夾連接 setup 顯示的 Project 與 Repo
- **THEN** 教學使用 `speclink link http://localhost:8080/api/speclink/v1/projects/demo --repo backend`、`speclink auth login` 與目前存在的讀寫動詞完成 smoke test，並警告不得在產品 repo 根目錄誤寫測試 binding

#### Scenario: 持久化、失聯與重置行為可觀察

- **WHEN** 使用者依教學停止並重啟 server，再執行完全重置
- **THEN** 教學分別要求觀察一般重啟不重印 setup token且資料與 remote tab 保留、失聯期間 snapshot 僅可讀且寫入被拒、恢復後以 Query／ETag 收斂，以及 `npm run dev:reset` 後重新出現全新 setup token

### Requirement: 帳號、PAT 與 membership 的操作邊界明確

Remote Getting Started 與架構交叉連結 SHALL 指示瀏覽器開啟 `/account` 管理自身 PAT，由該頁表單 POST `/account/tokens`；文件 SHALL 明說直接以 GET 開啟 `/account/tokens` 會得到 HTTP 405 Method Not Allowed。文件 SHALL 說明建立 Project／Repo registry 不會授予 membership，Server Admin 身分不會繞過 Project membership；管理員 SHALL 由 `/admin/users` 對帳號授予或更新 `reader`／`editor`，Desktop scope 清單才顯示該 Project 及其 Repos。PAT 明文只顯示一次，文件 SHALL NOT 以 URL、repo 設定或帶值的 shell argument 示範保存 PAT。

#### Scenario: 使用者直接開啟 PAT action URL

- **WHEN** 使用者在瀏覽器網址列開啟 `http://localhost:8080/account/tokens`
- **THEN** 故障排除指出 405 代表該路由只接受表單 POST，並導向 `http://localhost:8080/account` 登入後由 Personal Access Tokens 表單建立 PAT

#### Scenario: Desktop scopes 回傳空清單

- **WHEN** Desktop 顯示「此帳號目前沒有任何 Project／Repo membership」但 registry 已有 Project 與 Repo
- **THEN** 故障排除指示管理員到 `/admin/users` 對 Desktop 實際登入帳號授予該 Project 的 `reader` 或 `editor`，說明 Admin flag 不是 scope bypass，並要求回到 Desktop 重新載入 chooser

#### Scenario: PAT 不進入不安全載體

- **WHEN** 使用者選擇 Desktop PAT fallback 或 CLI `auth login`
- **THEN** 教學要求從 `/account` 複製只顯示一次的 PAT並貼入應用或 stdin，不把真實 PAT 放進 URL、`.speclink.yaml`、repo、文件範例或 shell history

### Requirement: Remote 教學具雙語導流與可重複查核

README 與產品能力狀態的中英文版本 SHALL 連到對應語言的 Remote Getting Started，既有繁中 Server 部署指南 SHALL 連到繁中 Remote Getting Started；兩份新教學 SHALL 維持相同 H2 集合與順序、命令語意、網址角色、membership 規則、故障排除症狀集合及目前能力邊界。文件查核 SHALL 驗證所有相對 Markdown 連結目標存在、現行 Server 路由與 CLI 指令可由 source／help 觀察、繁中與英文 H2 對等，並 SHALL 驗證關鍵字串涵蓋 `/account`、POST `/account/tokens`、`/admin/users`、HTTP 405、membership、project-scoped URL、spec-only、checkout、offline 與 `npm run dev:reset`。

#### Scenario: 使用者從既有文件找到 Remote 教學

- **WHEN** 使用者從任一語言 README、任一語言產品能力狀態或繁中 Server 部署指南尋找 Remote Server／Desktop／CLI 的首次操作方式
- **THEN** 文件提供有效連結到同語言 Remote Getting Started，而平台架構仍只作目標與安全邊界的正典

#### Scenario: 雙語與入口查核阻止文件漂移

- **WHEN** 維護者執行 tasks 指定的雙語 H2、相對連結、路由、CLI surface 與關鍵流程檢查
- **THEN** 全部檢查以 exit code 0 完成；任一語言缺少步驟、連到不存在檔案、把 `/account/tokens` 寫成 GET 頁面或引用不存在指令時以非零結果指出缺口

## MODIFIED Requirements

### Requirement: Remote Getting Started 提供可重複的完整操作路徑

中英文 Remote Getting Started SHALL 以目前已交付的入口，提供從零到可操作 Remote Desktop 與 Remote CLI 的端到端流程，並以 npx 為最短路徑首選：文件 SHALL 依序涵蓋 `npx @speclink/server` 一行啟動與其資料目錄、stdout 的 `/setup?token=...`、healthz 確認、建立第一位 Admin 與第一組 Project／Repo、在 `/admin/users` 授予實際登入帳號 Project membership、以 `/account` 建立 PAT、Desktop device login 與 PAT fallback、Project／Repo scope 選擇、spec-only 與 checkout 分流、CLI `link`／`auth login`／讀寫 smoke test、一般重啟持久化、離線唯讀與恢復、完全重置。第一位 Admin 為自己授予所建 Project 的 membership SHALL 明示為必經步驟，不得僅以邀請他人的情境帶過。checkout 內開發的 `npm run dev:reset`／`npm run dev` 路徑 SHALL 保留為開發者路徑並連往開發環境文件。文件 SHALL 明確區分 Server base URL、瀏覽器帳號／管理頁 URL 與 project-scoped API URL，且 SHALL NOT 把架構目標或規劃中能力寫成現行操作。

#### Scenario: 第一位 Admin 完成 Desktop Remote Workspace

- **WHEN** 使用者在無任何既有資料的環境依繁體中文或英文教學以 npx 啟動 Server、完成 setup，並使用 setup 建立的 Admin 登入 Desktop
- **THEN** 教學指示使用者先在 `/admin/users` 為該 Admin 授予所建 Project 的 `editor` membership（明示為必經步驟），再重新載入 Desktop scope 清單，選到所建 Project／Repo 並開啟 spec-only 或 checkout workspace

#### Scenario: Remote CLI 使用 project-scoped URL

- **WHEN** 使用者依教學在獨立測試資料夾連接 setup 顯示的 Project 與 Repo
- **THEN** 教學使用 `speclink link http://localhost:8080/api/speclink/v1/projects/demo --repo backend`、`speclink auth login` 與目前存在的讀寫動詞完成 smoke test，並警告不得在產品 repo 根目錄誤寫測試 binding

#### Scenario: 持久化、失聯與重置行為可觀察

- **WHEN** 使用者依教學停止並重啟 server，再執行完全重置
- **THEN** 教學分別要求觀察一般重啟不重印 setup token且資料與 remote tab 保留、失聯期間 snapshot 僅可讀且寫入被拒、恢復後以 Query／ETag 收斂，以及 npx 路徑刪除資料目錄（或 checkout 路徑 `npm run dev:reset`）後重新出現全新 setup token

## Why

目前 Remote Server、Remote CLI 與 Desktop Remote Workspace 已有可操作入口，但文件只在架構藍圖、部署指南與能力狀態中零散描述，缺少 PM／PO／RD／開發者可從全新環境照做的完整流程。使用者已實際因把 `/account/tokens` 當成可直接開啟的 GET 頁面而遇到 HTTP 405，也會因第一位 Admin 未自動取得 Project membership 而在 Desktop 看到空 scope；現在需要以可重複驗證的操作教學閉合文件入口。

## What Changes

- 新增繁體中文與英文 Remote Getting Started，涵蓋本地開發 server 啟動、`/setup`、Project／Repo registry、Admin 與 membership 的差異、`/account` 建 PAT、Desktop device login／PAT fallback、spec-only／checkout、Remote CLI、持久化、失聯恢復與重置。
- 加入明確的網址與方法邊界：瀏覽器進 `/account`，由表單 POST `/account/tokens`；直接 GET `/account/tokens` 會得到 405，且 project-scoped API URL 不是瀏覽器帳號頁入口。
- 加入首次 Admin 的權限操作：建立 Project／Repo 不等於授予 membership；至 `/admin/users` 為實際帳號指派 reader／editor，再重新載入 Desktop scope 清單。
- 加入以症狀為入口的故障排除表，至少涵蓋 405、空 membership、401／重新認證、server offline／stale、marker 衝突與重置後需重新 setup。
- 從中英文 README、中英文產品狀態與既有繁中 Server 部署指南導向對應教學；修正平台架構中容易被誤讀為可直接瀏覽 `/account/tokens` 的措辭。
- 以雙語結構、相對連結、實際路由／CLI surface 與關鍵流程字串檢查驗證文件，不以 runtime crate 或 GUI 行為變更替代文件驗收。

## Non-Goals

- 不修改 `/setup` 自動授予第一位 Admin membership 的 runtime 行為；該產品行為若要改動需另立 change。
- 不修改 Server、Desktop、CLI、Protocol、認證、角色或 Store 的行為與輸出。
- 不新增 MCP、Copilot Tools、SSO、Cluster 或其他尚未交付能力的教學。
- 不把 PAT 寫入命令列範例、repo、URL 或文件中的真實值；僅說明安全貼入與一次性顯示語意。

## Capabilities

### New Capabilities

（無）

### Modified Capabilities

- `user-documentation`: 增加 Remote Server／Desktop／CLI 的雙語端到端操作教學、入口導流、方法邊界與故障排除要求。

## Impact

- Affected specs: user-documentation
- Affected code:
  - New: docs/remote-getting-started.zh-TW.md, docs/remote-getting-started.md
  - Modified: README.md, README.en.md, docs/server-deployment.zh-TW.md, docs/product-status.zh-TW.md, docs/product-status.md, docs/platform-architecture.zh-TW.md, openspec/specs/user-documentation/spec.md
  - Removed: none
- Runtime crates: `crates/speclink-core` 與 `crates/speclink-cli` 均不修改；只以其現行 CLI surface 作文件驗證依據。
- Compatibility: CLI 人眼輸出、`--json`、exit code、stdin、設定欄位、Claude／Codex skills 與既有 parity／golden tests 皆不變。

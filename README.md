<p align="center">
  <img src="docs/assets/brand/transparent/speclink-logo-horizontal.png" alt="Speclink" width="440" />
</p>

<p align="center">
  <b>一套 SDD Engine，支援 Local Repo 與 Remote Store</b>
</p>

<p align="center">
  <b>繁體中文</b> · <a href="README.en.md">English</a>
</p>

Speclink 是以 Rust 實作的 Spec-Driven Development（SDD）引擎與工具平台。它讓 PM、PO、RD 與 AI Agent
使用同一套 change、artifact、task、verify 與 archive 語意，同時保留兩種部署路徑：

- **Local Repo**：規格位於 repo 的 `openspec/`，由 Git 協作，不需要 server。
- **Remote Store**：規格位於共享 Store，由 Speclink Host 統一處理認證、revision、交易、事件與流程裁決。

Local CLI 設計之初以 [Spectra App 2.3.1](https://github.com/kaochenlong/spectra-app) 所附 CLI 為行為參考；
人眼輸出、`--json` shape 與核心工作流由 golden 與 CLI 整合測試保護；Speclink 在此基礎上加入 discussion、Desktop、
Store abstraction、Node SDK 與 Remote Platform 等延伸。

> **目前狀態（2026-07-17）：**Local Repo、CLI、Local Desktop、Node N-API SDK、TeamStore contract 與三個官方
> Store drivers、`speclink-server`、Server setup/admin/auth/backup，以及 Remote CLI／Context Projection 已有可運作實作。
> Desktop Server Connections 已可用，但完整 Desktop Remote Workspace 與 verify/evidence 仍為部分可用；MCP／Copilot
> Tools、SSO、runtime plugins 與 cluster mode 尚在規劃中。舊 remote REST v1 已棄用，不再是目標架構。

目前能力的逐項證據、限制與最後查核日期見[產品能力狀態](docs/product-status.zh-TW.md)；程式碼與目標架構的差距、
交付順序及各 Phase 驗收 gate 見[現況對齊與重構路線圖](docs/implementation-refactor-roadmap.zh-TW.md)。

## Current capabilities / 目前能力

- **可用：**Local Repo CLI、Local Desktop、Node N-API SDK、Command Runtime／Host／Protocol、SQLite／Server FS／PostgreSQL TeamStore、單節點 Server、Admin/Auth、Remote CLI 與 Server 營運。
- **部分可用：**生成 Agent skills、Desktop Remote Workspace、verify／task evidence；可用子集與缺口以 product-status 為準。
- **規劃中：**MCP／Copilot in-process tools、SSO、runtime plugins、cluster mode 與獨立 verb-contract 使用者指南。
- **已棄用：**legacy remote REST v1 prototype；新工作使用目前 Client Protocol／Host 路徑。

完整矩陣不在 README 重複維護，請直接查[產品能力狀態](docs/product-status.zh-TW.md)。

## SDD workflow / SDD 工作流

```text
onboard? → discuss? → propose → apply ⇄ ingest → archive
                         ↑
                 resume after pause: drift first

utilities: validate / analyze / audit / commit / verify and evidence
```

`discuss` 只在需求需要收斂時使用；需求明確可直接 `propose`。實作途中需求改變走 `ingest`，閒置後續作先跑
`drift`。討論結論後的完整提案、快速轉為變更、併入既有 change 與決定不做等分流，見[完整 SDD 工作流](docs/workflow.zh-TW.md)。

## Local Repo quick start / Local Repo 快速開始

需要 stable Rust toolchain：

```bash
cargo install --path crates/speclink-cli
speclink --version
```

在要導入 Speclink 的 repo：

```bash
speclink init --tools claude,codex
speclink list
```

接著在 Claude 呼叫 `/speclink-propose <change>`，或在 Codex 呼叫 `$speclink-propose <change>`；Agent 會依 schema DAG
建立必要 artifacts。可複製的第一輪與直接 CLI 對照見[Local Repo 入門教學](docs/getting-started.zh-TW.md)。

## Deployment paths / 部署路徑

- **Local Repo：**Embedded Rust Runtime → FsStore → `openspec/` → Git；適合單一 repo、本機與離線協作。
- **Remote Store：**CLI／Desktop／其他 Client → Speclink Host → 同一 Rust Runtime → TeamStore；適合共享規格正典、集中認證、revision、交易與事件。

Remote Store 不會同步成第二份可寫的本地真相；有 checkout 的 Agent 使用唯讀 `.speclink/context/`，遠端寫入仍走
Host command。目標邊界以[平台架構藍圖](docs/platform-architecture.zh-TW.md)為準，現行 Server 操作見
[Remote 入門教學](docs/remote-getting-started.zh-TW.md)、[部署](docs/server-deployment.zh-TW.md)、
[Store drivers](docs/server-store-drivers.zh-TW.md)與[備份／還原](docs/server-backup.zh-TW.md)。

## Documentation map / 文件地圖

| 文件 | 用途 |
| --- | --- |
| [Local Repo 入門教學](docs/getting-started.zh-TW.md) | 目前可複製的第一輪 Local Repo 流程 |
| [Remote Server、Desktop 與 CLI 入門教學](docs/remote-getting-started.zh-TW.md) | 從 `/setup`、membership、登入到 Desktop／CLI 與失聯恢復的完整流程 |
| [完整 SDD 工作流](docs/workflow.zh-TW.md) | 每個階段的用途、使用時機、分支、完成判準與恢復方式 |
| [產品能力狀態](docs/product-status.zh-TW.md) | Available／Partial／Planned／Deprecated、證據與限制 |
| [設定說明](docs/configuration.zh-TW.md) | Local／Remote 設定歸屬與目前欄位 |
| [Node SDK](docs/sdk-node.zh-TW.md) | `@speclink/engine` 安裝、Store bridge 與 dispatch surface |
| [平台架構藍圖](docs/platform-architecture.zh-TW.md) | 唯一目標架構：Engine、Host、Store、Protocol、Server、Desktop 與 Agent |
| [實作重構路線圖](docs/implementation-refactor-roadmap.zh-TW.md) | 目標架構下的交付順序、Phase 與 Gate |
| [Server 部署](docs/server-deployment.zh-TW.md) | native／Docker／Compose 與升級操作 |
| [Server Store drivers](docs/server-store-drivers.zh-TW.md) | SQLite／Server FS／PostgreSQL 選型與前提 |
| [Server 備份與還原](docs/server-backup.zh-TW.md) | backup／verify-backup／restore |
| [品牌資產](docs/assets/brand/README.md) | Logo、配色與使用方式 |

`openspec/changes/archive/` 與 `openspec/discussions/archive/` 是歷史稽核資料，不是目前操作手冊。進階
`docs/verb-contract.md` 使用者指南尚未建立；目前契約以 canonical specs 為準，缺口記錄於 product-status。

## Development / 開發

開發環境的一鍵入口（整套 `npm run dev`、只跑 server、只跑 desktop、checkout 內 CLI）與下載安裝檔的未簽章放行步驟，見[開發環境入口](docs/development.zh-TW.md)。

```bash
cargo test --workspace
npm --workspace @speclink/desktop test
npm --workspace @speclink/desktop run build
npm --workspace @speclink/engine test
```

CLI 可觀察輸出由 golden 與 CLI 整合測試保護；Server／Store／Desktop 的特定測試前提與目前限制請查 product-status
及責任文件。

## License / 授權

[MIT](LICENSE)

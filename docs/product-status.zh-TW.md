# Speclink 產品能力狀態

**繁體中文** · [English](product-status.md)

最後查核日期：**2026-07-17**。本文是「目前能不能用」的正典；[平台架構藍圖](platform-architecture.zh-TW.md)描述唯一目標架構，[實作重構路線圖](implementation-refactor-roadmap.zh-TW.md)描述其下的交付順序。檔案、crate 或正典 spec 單獨存在不代表產品路徑已完整交付。

要從全新本地資料實際操作 Remote Server、Desktop 與 CLI，請依
[Remote 入門教學](remote-getting-started.zh-TW.md)完成 setup、membership、登入、workspace 與恢復測試。

## Status model / 狀態模型

- **Available（可用）**：有可操作入口，並有至少兩項獨立證據或一項端到端證據。
- **Partial（部分可用）**：可操作子集已存在，但完整流程仍有明確缺口。
- **Planned（規劃中）**：只有目標設計、基礎型別或尚未閉合的入口，不應寫成目前支援。
- **Deprecated（已棄用）**：相容或歷史路徑仍可被找到，但已不是目標架構。

## Capability matrix / 能力矩陣

| Capability / 能力 | Status / 狀態 | User entry / 使用者入口 | Evidence / 證據 | Limits and next step / 限制與下一步 | Checked / 查核 |
| --- | --- | --- | --- | --- | --- |
| Local Repo CLI | Available（可用） | `speclink init`、`list`、`show`、`status`、`validate`、`analyze`、`drift`、`archive` 與 discussion verbs | [`speclink-cli` 入口](../crates/speclink-cli/src/main.rs)<br>[CLI 整合測試](../crates/speclink-cli/tests/doc_verbs.rs) | Local Repo 完全不需要 server；進階使用時仍須依各子指令 `--help` 判斷旗標。 | 2026-07-17 |
| Generated Agent Skills | Partial（部分可用） | Claude `/speclink-*`、Codex `$speclink-*` | [目前生成的 apply skill](../.agents/skills/speclink-apply/SKILL.md)<br>[引擎內建 skill assets](../crates/speclink-core/assets/skills/verify.md) | 目前生成面有 onboard／discuss／propose／apply／ingest／drift／audit／commit／archive；`verify.md` 等 asset 存在於引擎，但此 repo 未生成 `$speclink-verify`，不得把它寫成可直接呼叫入口。 | 2026-07-17 |
| Local Desktop | Available（可用） | Tauri/React change 看板、spec、discussion、archive、tasks、設定與 tray | [Desktop scripts](../apps/desktop/package.json)<br>[Desktop UI tests](../apps/desktop/src/__tests__/App.test.tsx) | Local workspace 可用；Remote Workspace 的完成度另見本表。 | 2026-07-17 |
| Node N-API SDK | Available（可用） | `@speclink/engine` 的 Store bridge、render 與 `dispatch` | [Node 套件入口](../crates/speclink-node/package.json)<br>[dispatch contract tests](../crates/speclink-node/__test__/dispatch-contract.spec.ts) | 目前是 Engine／Store bridge；Phase 4 的完整 Node Host 與 Copilot Tool 套件尚未交付。 | 2026-07-17 |
| Command Runtime, Host and Protocol | Available（可用） | Rust crates 供 CLI、Server 與 Node adapter 共用 | [Host 雙路徑測試](../crates/speclink-host/tests/bridge_dual_path.rs)<br>[Client Protocol spec](../openspec/specs/client-protocol/spec.md) | 基礎 typed command/query/context 路徑已存在；Agent 生態包裝與部分進階 gate 仍分列為 Partial／Planned。 | 2026-07-17 |
| SQLite TeamStore | Available（可用） | `speclink-server` 的預設 `sqlite` driver | [SQLite conformance tests](../crates/speclink-store-sqlite/tests/conformance.rs)<br>[driver 選型文件](server-store-drivers.zh-TW.md) | 單一 instance 定位；cluster 不在目前能力內。 | 2026-07-17 |
| Server FS TeamStore | Available（可用） | server config 的 `serverfs` driver | [Server FS conformance tests](../crates/speclink-store-fs/tests/conformance.rs)<br>[atomic publish tests](../crates/speclink-store-fs/tests/atomic_publish.rs) | 需要可靠的 OS advisory lock／flock 語意，單一資料目錄只允許一個 server。 | 2026-07-17 |
| PostgreSQL TeamStore | Available（可用） | server config 的 `postgres` driver | [PostgreSQL conformance tests](../crates/speclink-store-postgres/tests/conformance.rs)<br>[resilience tests](../crates/speclink-store-postgres/tests/resilience.rs) | 完整測試需要 PostgreSQL 15 與 `SPECLINK_TEST_POSTGRES_URL`；目前 server 產品仍以單一 instance 為定位。 | 2026-07-17 |
| `speclink-server` | Available（可用） | native binary／Docker，HTTP Command／Query／Context／Event API | [Server binary](../crates/speclink-server/src/main.rs)<br>[CLI-to-server E2E](../crates/speclink-server/tests/e2e_cli.rs) | 單節點 server 已可運作；遠端 task touched-file evidence 尚未完整落庫，見 Verify/evidence 列。 | 2026-07-17 |
| Server Admin, setup and identity | Available（可用） | `/setup`、`/admin`、`/account`、PAT／device flow／invite 與 headless admin commands | [Admin E2E tests](../crates/speclink-server/tests/admin_e2e.rs)<br>[Device-flow E2E tests](../crates/speclink-server/tests/device_e2e.rs) | 目前涵蓋單節點安裝與帳號管理；SSO 與 cluster 管理仍是規劃能力。 | 2026-07-17 |
| Desktop Server Connections | Available（可用） | Desktop 設定中的 Server 清單、device login、PAT fallback、logout 與 OS Keychain | [Tauri connection orchestration](../apps/desktop/src-tauri/src/connections.rs)<br>[Servers panel tests](../apps/desktop/src/__tests__/serversPanel.test.tsx) | 可管理連線與身分，但登入後尚不能建立完整 Remote Workspace。 | 2026-07-17 |
| Desktop Remote Workspace | Partial（部分可用） | `WorkspaceLocator` remote 型別與 Server connection UI | [Workspace locator](../apps/desktop/src/session.ts)<br>[Workspace-session spec](../openspec/specs/workspace-session/spec.md) | remote locator 目前沒有建構路徑；spec-only、remote+checkout、offline/conflict 工作階段仍待後續 change 閉合。 | 2026-07-17 |
| Remote CLI and Context Projection | Available（可用） | `speclink link`、`auth`、`artifact` 與唯讀 `.speclink/context/` | [Remote CLI tests](../crates/speclink-cli/tests/remote_read_path.rs)<br>[Context materializer](../crates/speclink-host/src/projection.rs) | 現行 Client Protocol 路徑可用；Desktop Remote Workspace 與遠端 evidence 缺口不因此視為完成。 | 2026-07-17 |
| Verify and task evidence | Partial（部分可用） | `speclink task done`、Local evidence、`validate`／`analyze`；引擎內有 verify workflow asset | [Host evidence implementation](../crates/speclink-host/src/evidence.rs)<br>[Phase 2 chain test](../crates/speclink-server/tests/phase2_chain.rs) | 此 repo 未生成 `$speclink-verify`；server 端目前丟棄 remote task 的 touched files，測試以 ignored defect case 明確保留缺口。 | 2026-07-17 |
| Server operations | Available（可用） | native／Docker／Compose、health/readiness、backup／verify-backup／restore | [部署文件](server-deployment.zh-TW.md)<br>[Backup E2E tests](../crates/speclink-server/tests/backup_e2e.rs) | 備份目前要求維護窗口；沒有滾動升級或 cluster 操作。 | 2026-07-17 |
| MCP and Copilot in-process tools | Planned（規劃中） | 尚無可安裝的 `@speclink/copilot-tools` 或 MCP adapter | [目前 workspace package inventory](../package.json)<br>[Phase 4 目標](implementation-refactor-roadmap.zh-TW.md) | 不得把 README 的架構示意當成目前套件；後續需完成 tool adapter、identity closure 與端到端測試。 | 2026-07-17 |
| SSO, runtime plugins and cluster mode | Planned（規劃中） | 尚無產品入口 | [平台架構藍圖](platform-architecture.zh-TW.md)<br>[實作重構路線圖](implementation-refactor-roadmap.zh-TW.md) | 屬後續平台／生態能力；目前 Server 與 drivers 的正式定位仍是單一 instance。 | 2026-07-17 |
| Legacy remote REST v1 | Deprecated（已棄用） | 歷史 remote client prototype | [舊路徑盤點](implementation-refactor-roadmap.zh-TW.md)<br>[README 架構聲明](../README.md) | 不作為新 Client Protocol 的相容負擔或正式 Server contract；新文件只說明遷移方向，不教學此路徑。 | 2026-07-17 |
| Advanced verb-contract user guide | Planned（規劃中） | 尚無獨立使用者指南 | [Canonical verb contract](../openspec/specs/verb-contract/spec.md)<br>[Client Protocol spec](../openspec/specs/client-protocol/spec.md) | `docs/verb-contract.md` 目前不存在；本 change 只誠實記錄缺口，不建立空檔或失效連結。 | 2026-07-17 |

## Verification baseline / 查核基線

本次狀態判定可用下列方式重做：

1. 執行 `speclink --help` 與相關子指令 `--help`，確認 Local／Remote CLI surface。
2. 執行 `speclink-server --help`，確認 server、identity 與 backup 操作入口。
3. 比較 `.agents/skills/*/SKILL.md` 與 `crates/speclink-core/assets/skills/*.md`，區分「引擎有 asset」和「目前 Host 已生成 skill」。
4. 由 workspace `Cargo.toml`、各 package scripts、integration／E2E／conformance tests 與正典 specs 交叉驗證；沒有使用者入口的能力不得因 crate 存在而標為 Available。

套用本 change 前可重現的文件失敗為：中英文 README 均將已可執行的 `speclink-server` 寫成尚未交付；中英文 getting-started 呼叫未生成的 `speclink-verify`；並把 proposal、delta spec、design、tasks 寫成每個 change 固定四份，而非依 schema DAG／`applyRequires` 決定。

## Known documentation gap / 已知文件缺口

進階 verb／Protocol 契約目前只有 canonical specs，沒有獨立的 `docs/verb-contract.md` 使用者指南。這是公開記錄的文件缺口，不是可點擊文件，也不代表契約不存在；後續 change 應從 canonical `verb-contract` 與 `client-protocol` specs 產生可維護的進階指南。

## Target references / 目標參考

- [完整 SDD 工作流](workflow.zh-TW.md)：用途、使用時機、分支與恢復方式。
- [平台架構藍圖](platform-architecture.zh-TW.md)：唯一目標架構。
- [實作重構路線圖](implementation-refactor-roadmap.zh-TW.md)：目標架構下的交付順序與 gate。
- [Server 部署](server-deployment.zh-TW.md)、[Store drivers](server-store-drivers.zh-TW.md)、[備份與還原](server-backup.zh-TW.md)：目前 Server 營運方式。

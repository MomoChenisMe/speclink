# Speclink 專案能力狀態

**繁體中文** · [English](product-status.md)

最後查核日期：**2026-08-25**。本文是「目前能不能用」的正典。行為與邊界的正典則是 `openspec/specs/` 底下的規格，[專案路線圖](roadmap.zh-TW.md)則描述對使用者有意義的方向。

檔案、crate 或正典 spec 單獨存在，不代表交付路徑已完整。

要從全新本地資料實際操作 Remote Server、Desktop 與 CLI，請依
[Remote 入門教學](remote-getting-started.zh-TW.md)完成 setup、membership、登入、workspace 與恢復測試。

## Status model / 狀態模型

- **Available（可用）**：有可操作入口，並有至少兩項獨立證據或一項端到端證據。
- **Partial（部分可用）**：可操作子集已存在，但完整流程仍有明確缺口。
- **Planned（規劃中）**：只有目標設計、基礎型別或尚未閉合的入口，不應寫成目前支援。
- **Deprecated（已棄用）**：相容或歷史路徑仍可被找到，但已不是目標架構。

## Local and Remote / 本地與遠端能力對照

兩條路徑的差異集中在這一張表，不必跨文件拼湊。表中的 Remote Store 一欄以官方參考 server `speclink-server` 為量測對象；遠端模式本身由 Host 與 Protocol 契約定義，自建 server 端同樣適用這些欄位。CLI 動詞的模式歸屬由[動詞契約正典](../openspec/specs/verb-contract/spec.md)單點宣告。絕大多數動詞是 **Dual**：本地與遠端各有一臂，缺任一臂就構成建置失敗。只有 `demo` 限本地，`claim` 限遠端。

| Capability / 能力 | Local Repo | Remote Store | Note / 說明 |
| --- | --- | --- | --- |
| 規格與變更的讀寫 | Available（可用） | Available（可用） | 本地直接讀寫 `openspec/`；遠端一律經 Host command，不落第二份可寫本地真相。 |
| 變更生命週期動詞（`propose`→`apply`→`archive`） | Available（可用） | Available（可用） | `status`、`instructions`、`new`、`task`、`in-progress`、`archive`、`discard` 皆為 Dual。遠端不支援批次封存，一次一個。 |
| 討論（`discuss`） | Available（可用） | Available（可用） | Dual；轉為變更、併入既有變更與封存流程兩邊相同。 |
| 品質關卡（`review`／`verify`） | Available（可用） | Available（可用） | Dual；工單、輪與蓋章語意兩邊一致。 |
| 認領變更（`claim`） | 不適用 | Available（可用） | RemoteOnly——本地模式以非零 exit code 明確拒絕。 |
| 示範資料（`demo`） | Available（可用） | 不適用 | FsOnly——遠端模式明確拒絕，且不發出任何 server 請求。 |
| Agent 讀取脈絡 | Available（可用） | Available（可用） | 本地直接讀 repo；遠端讀唯讀的 `.speclink/context/`，寫入仍走 Host command。 |
| Desktop 看板與詳情面板 | Available（可用） | Partial（部分可用） | 連線、登入與 chooser 開遠端看板皆可用；剩餘小縫（capability 清單、change 詮釋資料、離線衝突）見能力表。 |
| task 的 touched-file evidence | Available（可用） | Available（可用） | 本地落在變更目錄的 `.evidence.json`；遠端把回報的 touched files 存進 Store，`GET /changes/{name}/evidence` 讀得回。 |
| 帳號、PAT 與 membership | 不適用 | Available（可用） | 本地路徑不需要帳號；遠端有 `/setup`、invite、PAT 與 device login。 |
| 備份還原 | 由 Git 承擔 | Available（可用） | 遠端有 `backup`／`verify-backup`／`restore`，目前需要維護窗口。 |
| 離線工作 | Available（可用） | 需連線 | 本地完全不需要 server；遠端寫入需要可連得上 Host。 |

## Capability matrix / 能力矩陣

| Capability / 能力 | Status / 狀態 | User entry / 使用者入口 | Evidence / 證據 | Limits and next step / 限制與下一步 | Checked / 查核 |
| --- | --- | --- | --- | --- | --- |
| Local Repo CLI | Available（可用） | `speclink init`、`list`、`show`、`status`、`validate`、`analyze`、`drift`、`archive` 與 discussion verbs | [`speclink-cli` 入口](../crates/speclink-cli/src/main.rs)<br>[CLI 整合測試](../crates/speclink-cli/tests/it/doc_verbs.rs) | Local Repo 完全不需要 server；進階使用時仍須依各子指令 `--help` 判斷旗標。 | 2026-08-13 |
| Generated Agent Skills | Available（可用） | Claude `/speclink-*`、Codex `$speclink-*`（也可從 `/skills` 清單挑選） | [生成的 apply skill](../.agents/skills/speclink-apply/SKILL.md)<br>[生成的 verify skill](../.agents/skills/speclink-verify/SKILL.md) | 生成面已涵蓋 onboard／discuss／improve／propose／apply／worktree／ingest／drift／quality／review／verify／archive 與 audit／commit／config；唯一不對稱是 `analyze` 只有 Claude 側，Codex 直接用 CLI。生成數量取決於 `worktree` 政策：關閉時 Claude 15 個、Codex 14 個，開啟時各多兩個 worktree 技能（本 repo 已開啟，因此是 17 與 16）。 | 2026-08-13 |
| Local Desktop | Available（可用） | Tauri/React change 看板、spec、discussion、archive、tasks、設定與 tray | [Desktop scripts](../apps/desktop/package.json)<br>[Desktop UI tests](../apps/desktop/src/__tests__/App.test.tsx) | Local workspace 可用；Remote Workspace 的完成度另見本表。 | 2026-08-13 |
| Quality stations（review／verify） | Available（可用） | `/speclink-review`、`/speclink-verify`、`/speclink-quality`；CLI 為 `speclink review`／`speclink verify` | [review 站實作](../crates/speclink-core/src/review.rs)<br>[蓋章與工單語意](../crates/speclink-core/src/station.rs) | 兩道關卡都落工單、多輪，必修集合為空才蓋章（SUGGESTION 不擋章）；蓋章後範圍內檔案再被改會降級為「其後有變動」。 | 2026-08-13 |
| Node N-API SDK | Partial（部分可用） | 自本 repo 建置 `crates/speclink-node` 後以路徑載入 | [Node 套件入口](../crates/speclink-node/package.json)<br>[dispatch contract tests](../crates/speclink-node/__test__/dispatch-contract.spec.ts) | **尚未發布至 npm**：`npm install @speclink/engine` 取不到套件，目前只能自 repo 建置（需 Rust 工具鏈）。Engine／Store bridge 本身可用；完整 Node Host 與 Copilot Tool 套件尚未交付。 | 2026-08-13 |
| Install channels / 安裝通路 | Available（可用） | 桌面安裝檔（macOS dmg、Windows NSIS、Linux AppImage 與 deb）、CLI 安裝腳本與 Homebrew tap、server 的 npx 與 Docker | [安裝腳本測試](../scripts/install.test.mjs)<br>[Homebrew formula 產生器](../scripts/homebrew-formula.mjs) | 桌面與 CLI 三平台皆有通路；Windows 安裝檔目前未經程式碼簽章，首次執行需放行 SmartScreen。 | 2026-08-13 |
| Command Runtime, Host and Protocol | Available（可用） | Rust crates 供 CLI、Server 與 Node adapter 共用 | [Host 雙路徑測試](../crates/speclink-host/tests/bridge_dual_path.rs)<br>[Client Protocol spec](../openspec/specs/client-protocol/spec.md) | 基礎 typed command/query/context 路徑已存在；Agent 生態包裝與部分進階 gate 仍分列為 Partial／Planned。 | 2026-08-13 |
| SQLite TeamStore | Available（可用） | `speclink-server` 的預設 `sqlite` driver | [SQLite conformance tests](../crates/speclink-store-sqlite/tests/conformance.rs)<br>[driver 選型文件](server-store-drivers.zh-TW.md) | 單一 instance 定位；cluster 不在目前能力內。 | 2026-08-13 |
| Server FS TeamStore | Available（可用） | server config 的 `serverfs` driver | [Server FS conformance tests](../crates/speclink-store-fs/tests/it/conformance.rs)<br>[atomic publish tests](../crates/speclink-store-fs/tests/it/atomic_publish.rs) | 需要可靠的 OS advisory lock／flock 語意，單一資料目錄只允許一個 server。 | 2026-08-13 |
| PostgreSQL TeamStore | Available（可用） | server config 的 `postgres` driver | [PostgreSQL conformance tests](../crates/speclink-store-postgres/tests/it/conformance.rs)<br>[resilience tests](../crates/speclink-store-postgres/tests/it/resilience.rs) | 完整測試需要 PostgreSQL 與 `SPECLINK_TEST_POSTGRES_URL`；目前 server 仍以單一 instance 為定位。 | 2026-08-13 |
| `speclink-server` | Available（可用） | native binary／Docker／npx，HTTP Command／Query／Context／Event API | [Server binary](../crates/speclink-server/src/main.rs)<br>[CLI-to-server E2E](../crates/speclink-server/tests/it/e2e_cli.rs) | 單節點 server 已可運作；遠端 task done 回報的 touched-file evidence 已落庫可查，見 Remote task evidence 一列。 | 2026-08-25 |
| Server Admin, setup and identity | Available（可用） | `/setup`、`/admin`、`/account`、PAT／device flow／invite 與 headless admin commands | [Admin E2E tests](../crates/speclink-server/tests/it/admin_e2e.rs)<br>[Device-flow E2E tests](../crates/speclink-server/tests/it/device_e2e.rs) | 目前涵蓋單節點安裝與帳號管理；SSO 與 cluster 管理仍是規劃能力。 | 2026-08-13 |
| Desktop Server Connections | Available（可用） | Desktop 設定中的 Server 清單、device login、PAT fallback、logout 與 OS Keychain | [Tauri connection orchestration](../apps/desktop/src-tauri/src/connections.rs)<br>[Servers panel tests](../apps/desktop/src/__tests__/serversPanel.test.tsx) | 可管理連線與身分；登入後即可在 chooser 開遠端 workspace，剩餘小縫見下一列。 | 2026-08-23 |
| Desktop Remote Workspace | Partial（部分可用） | Workspace chooser 的遠端開啟：skip（免 checkout）與 folder（綁本機 checkout）兩模式 | [Workspace chooser](../apps/desktop/src/components/WorkspaceChooser.tsx)<br>[Remote session 工廠](../apps/desktop/src/session.ts)<br>[遠端開啟測試](../apps/desktop/src/__tests__/remoteOpen.test.ts) | 遠端看板可開、可勾任務、可讀寫 artifact。剩餘小縫：capability 清單與 change 詮釋資料在遠端不支援、討論的 promotedTo 以空清單補齊、離線衝突處理尚未完成。 | 2026-08-23 |
| Remote CLI and Context Projection | Available（可用） | `speclink link`、`auth`、`artifact` 與唯讀 `.speclink/context/` | [Remote CLI tests](../crates/speclink-cli/tests/it/remote_read_path.rs)<br>[Context materializer](../crates/speclink-host/src/projection.rs) | 現行 Client Protocol 路徑可用；Desktop Remote Workspace 的剩餘小縫不因此視為完成。 | 2026-08-13 |
| Remote task evidence | Available（可用） | 本地 `speclink task done` 落 `.evidence.json`；遠端同動詞把 touched files 存進 Store，`GET /changes/{name}/evidence` 讀得回 | [Task evidence 實作](../crates/speclink-core/src/tasks.rs)<br>[遠端證據端到端測試](../crates/speclink-server/tests/it/phase2_chain.rs) | evidence 與任務勾選、task-completed 事件在同一交易落地，隨 change 封存或廢棄同生命週期移動。Desktop 遠端勾任務不送 touched files，沿「無新髒檔不新增記錄」語意。 | 2026-08-23 |
| Server operations | Available（可用） | native／Docker／Compose、health/readiness、backup／verify-backup／restore | [部署文件](server-deployment.zh-TW.md)<br>[Backup E2E tests](../crates/speclink-server/tests/it/backup_e2e.rs) | 備份目前要求維護窗口；沒有滾動升級或 cluster 操作。 | 2026-08-13 |
| MCP and Copilot in-process tools | Planned（規劃中） | 尚無可安裝的 Copilot tools 套件或 MCP adapter | [目前 workspace package inventory](../package.json)<br>[方向與可觀察下一步](roadmap.zh-TW.md) | 不得把架構示意當成目前套件；後續需完成 tool adapter、身分收口與端到端測試。 | 2026-08-13 |
| SSO, runtime plugins and cluster mode | Planned（規劃中） | 尚無可用入口 | [方向與可觀察下一步](roadmap.zh-TW.md) | 屬後續平台／生態能力，尚未排定先後；目前 Server 與 drivers 的正式定位仍是單一 instance。 | 2026-08-13 |
| Legacy remote REST v1 | Deprecated（已棄用） | 歷史 remote client prototype | [歷史 prototype crate](../crates/speclink-remote/src/lib.rs)<br>[現行 Client Protocol 正典](../openspec/specs/client-protocol/spec.md) | 不作為新 Client Protocol 的相容負擔或正式 Server contract；新文件只說明遷移方向，不教學此路徑。 | 2026-08-13 |
| Advanced verb-contract user guide | Available（可用） | [動詞契約指南](verb-contract.zh-TW.md)（中英兩版） | [Canonical verb contract](../openspec/specs/verb-contract/spec.md)<br>[Client Protocol spec](../openspec/specs/client-protocol/spec.md) | 指南已建立，涵蓋動詞的模式歸屬、兩模式輸出同形與端點契約；正典仍以 specs 為準，指南隨其更新。 | 2026-08-13 |

## Verification baseline / 查核基線

本次狀態判定可用下列方式重做：

1. 執行 `speclink --help` 與相關子指令 `--help`，確認 Local／Remote CLI surface。
2. 執行 `speclink-server --help`，確認 server、identity 與 backup 操作入口。
3. 比較 `.claude/skills/` 與 `.agents/skills/` 兩個生成面的目錄清單，區分「引擎有 asset」和「目前 Host 已生成技能」。兩邊的差額只有 `speclink-analyze`（僅 Claude 側）；總數則隨 `worktree` 政策而變。
4. 以[動詞契約正典](../openspec/specs/verb-contract/spec.md)的模式分岔宣告核對本地與遠端對照表——該宣告是單點來源，逐動詞歸屬 ModeFree／Dual／FsOnly／RemoteOnly。
5. 由 workspace `Cargo.toml`、各 package scripts、integration／E2E／conformance tests 與正典 specs 交叉核對。沒有使用者入口的能力，不得因為 crate 存在就標為 Available。

## Known documentation gap / 已知文件缺口

`@speclink/engine` 尚未發布至 npm。文件示範的是自 repo 建置的載入路徑，見 [Node SDK](sdk-node.zh-TW.md)。npm 通路的方向與可觀察下一步見[專案路線圖](roadmap.zh-TW.md)。

## Target references / 目標參考

- [完整 SDD 工作流](workflow.zh-TW.md)：每一站的用途、對應技能、完成判準與下一站。
- [專案路線圖](roadmap.zh-TW.md)：對使用者有意義的方向。
- [Server 部署](server-deployment.zh-TW.md)、[Store drivers](server-store-drivers.zh-TW.md)、[備份與還原](server-backup.zh-TW.md)：目前 Server 營運方式。

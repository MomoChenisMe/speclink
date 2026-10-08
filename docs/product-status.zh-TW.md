# 專案能力狀態

**繁體中文** · [English](product-status.md)

最後查核日期：**2026-10-07**（0.8.0）。

這份文件回答「這項能力現在能不能用」。行為的正式定義在 `openspec/specs/`；還沒做的方向在[專案路線圖](roadmap.zh-TW.md)。只有程式碼、crate 或正式規格存在，不代表使用者已經用得到。

要實際操作 Remote Server、Desktop 與 CLI，照 [Remote 入門](remote-getting-started.zh-TW.md)做一次。

## <a id="status-model"></a>狀態怎麼判定

| 狀態 | 意思 |
| --- | --- |
| Available（可用） | 有使用者入口，而且有兩項互相獨立的證據，或一項端到端測試。 |
| Partial（部分可用） | 有一部分可以用，完整流程還有明確的缺口。 |
| Planned（規劃中） | 還沒有可用的入口。細節見[專案路線圖](roadmap.zh-TW.md)。 |
| Deprecated（已棄用） | 程式裡還找得到，但已經不是目標架構。目前沒有這類項目。 |

## <a id="local-and-remote"></a>本地與遠端對照

本地與遠端的差異都在這張表。Remote Store 一欄以官方參考 server `speclink-server` 量測；遠端模式由 Host 與 Protocol 契約定義，自建 server 同樣適用這些欄位。

大多數 CLI 動詞在本地與遠端都能用（Dual）。例外如下，完整清單見[動詞與旗標契約](verb-contract.zh-TW.md)：

- 只限本地：`demo`、`trace`、`change rank`、`plan --strict-overlap`。
- 只限遠端：`claim`。

在錯的模式下執行這些動詞，CLI 會明確拒絕，不會改走另一邊。

| 能力 | Local Repo | Remote Store | 說明 |
| --- | --- | --- | --- |
| 規格與變更的讀寫 | 可用 | 可用 | 本地直接讀寫 `openspec/`；遠端一律經 Host 指令寫入，不會在本機留第二份可寫的資料。 |
| 變更生命週期（`propose` → `apply` → `archive`） | 可用 | 可用 | 遠端一次只能封存一個變更。 |
| 討論（`discuss`） | 可用 | 可用 | 轉為變更、併入既有變更與封存，兩邊相同。 |
| 品質關卡（`review`、`verify`） | 可用 | 可用 | 工單、輪與蓋章的規則兩邊相同。 |
| 執行順序（`plan`、`change depends`） | 可用 | 可用 | 0.6.0 起遠端也能讀 plan、寫前置。`change rank` 只限本地。 |
| 溯源（`trace`） | 可用 | 不適用 | 只限本地。 |
| 認領變更（`claim`） | 不適用 | 可用 | 認領人存進變更資料，重啟後還在。認領別人持有的變更會回 409，訊息寫出持有人。 |
| 示範資料（`demo`） | 可用 | 不適用 | 只限本地；遠端模式直接拒絕，不送出請求。 |
| Agent 讀取背景資料 | 可用 | 可用 | 本地直接讀 repo；遠端讀唯讀的 `.speclink/context/`。 |
| 任務的 touched-file 證據 | 可用 | 可用 | 本地寫進 `.evidence.json`；遠端存進 Store，可經 `GET /changes/{name}/evidence` 讀回。 |
| 桌面看板與詳情面板 | 可用 | 部分可用 | 遠端看板在桌面上勾任務時，不回報 touched files。 |
| 帳號、存取金鑰與 membership | 不適用 | 可用 | 本地不需要帳號。 |
| 備份還原 | 由 Git 負責 | 可用 | 遠端備份目前要停機進行。 |
| 離線工作 | 可用 | 需連線 | 遠端斷線時只能讀，寫入會被拒絕。 |

## <a id="capabilities"></a>能力清單

| 能力 | 狀態 | 使用者入口 | 證據 | 限制與下一步 | 查核 |
| --- | --- | --- | --- | --- | --- |
| Local Repo CLI | 可用 | `speclink init`、`list`、`show`、`status`、`validate`、`analyze`、`drift`、`archive`、`discuss` 等 | [CLI 入口](../crates/adapters/speclink-cli/src/main.rs)<br>[CLI 整合測試](../crates/adapters/speclink-cli/tests/it/doc_verbs.rs) | 不需要 server。旗標以各子指令的 `--help` 為準。 | 2026-10-07 |
| Agent 技能 | 可用 | Claude 與 GitHub Copilot `/speclink-*`、Codex `$speclink-*` | [生成的 apply 技能](../.agents/skills/speclink-apply/SKILL.md)<br>[生成的 manual 技能](../.claude/skills/speclink-manual/SKILL.md)<br>[自訂工具描述子測試](../crates/adapters/speclink-cli/tests/it/tools_descriptor.rs)<br>[Copilot 技能快照](../crates/engine/speclink-core/tests/golden/copilot.snapshot.md) | `analyze` 技能只有 Claude 有，Codex 與 GitHub Copilot 直接用 CLI。技能數量：`worktree` 政策關閉時 Claude 17 個、Codex 與 GitHub Copilot 各 16 個；開啟時各多 2 個。其他 AI 工具可用自訂工具描述子產生技能。 | 2026-10-08 |
| 品質關卡 | 可用 | `/speclink-review`、`/speclink-verify`、`/speclink-quality`；CLI `speclink review`、`speclink verify` | [關卡的蓋章與工單規則](../crates/engine/speclink-core/src/quality/station.rs)<br>[review 動詞測試](../crates/adapters/speclink-cli/tests/it/review_verbs.rs) | SUGGESTION 不擋章。蓋章後範圍內的檔案再被改，章會降級為「其後有變動」。 | 2026-10-07 |
| 執行順序 | 可用 | `speclink plan`、`speclink change depends`、`speclink change rank`；桌面詳情面板的「排程」分頁 | [plan 動詞測試](../crates/adapters/speclink-cli/tests/it/plan_verbs.rs)<br>[server 的 plan API 測試](../crates/host/speclink-server/tests/it/api/plan_api.rs) | `change rank` 與 `plan --strict-overlap` 只限本地。 | 2026-10-07 |
| 手冊與溯源 | 可用 | `/speclink-manual`、桌面「手冊」頁；`speclink trace`、`/speclink-trace` | [手冊頁測試](../packages/ui/src/__tests__/manualPage.test.tsx)<br>[trace 測試](../crates/adapters/speclink-cli/tests/it/trace.rs) | 遠端專案還不能生成手冊（導覽可以）。`trace` 只限本地。 | 2026-10-07 |
| Local 桌面 app | 可用 | 看板、規格、討論、已封存、手冊、設定與系統匣 | [Desktop scripts](../apps/desktop/package.json)<br>[Desktop UI 測試](../apps/desktop/src/__tests__/App.test.tsx)<br>[自動更新測試](../apps/desktop/src/__tests__/updater.test.ts) | 遠端 workspace 的狀態另列在下面。 | 2026-10-07 |
| 安裝通路 | 可用 | 桌面安裝檔（macOS universal dmg、Windows 安裝器、Linux AppImage）；CLI 的 npm、安裝腳本、Homebrew；server 的 npx 與 Docker | [npm 啟動器測試](../scripts/npm/npm-cli-launcher.test.mjs)<br>[安裝腳本測試](../scripts/install.test.mjs)<br>[Homebrew formula 產生器](../scripts/release/homebrew-formula.mjs) | Windows 安裝檔沒有程式碼簽章，第一次執行要放行 SmartScreen。 | 2026-10-07 |
| Node SDK（`@speclink/engine`） | 可用 | `npm install @speclink/engine` | [套件入口](../crates/adapters/speclink-node/package.json)<br>[dispatch 契約測試](../crates/adapters/speclink-node/__test__/dispatch-contract.spec.ts)<br>[版號測試](../scripts/npm/npm-engine-package.test.mjs) | 0.2.0 起在 npm 上，含五個平台子套件。還沒有：型別化的 JS 方法、JS 版遠端 client、Agent 工具層。見[路線圖](roadmap.zh-TW.md)。 | 2026-10-07 |
| Command Runtime、Host 與 Protocol | 可用 | CLI、server 與 Node SDK 共用的 Rust crates | [Host 雙路徑測試](../crates/host/speclink-host/tests/bridge_dual_path.rs)<br>[Client Protocol 正式規格](../openspec/specs/client-protocol/spec.md) | 給 Agent 用的工具包裝還沒做。 | 2026-10-07 |
| SQLite TeamStore | 可用 | `speclink-server` 預設的 `sqlite` | [SQLite 一致性測試](../crates/store/speclink-store-sqlite/tests/conformance.rs)<br>[儲存後端選擇](server-store-drivers.zh-TW.md) | 只支援單一 server instance。 | 2026-10-07 |
| Server FS TeamStore | 可用 | server 設定的 `serverfs` | [Server FS 一致性測試](../crates/store/speclink-store-fs/tests/it/conformance.rs)<br>[原子發布測試](../crates/store/speclink-store-fs/tests/it/atomic_publish.rs) | 需要可靠的檔案鎖（flock）；一個資料目錄只能有一個 server。 | 2026-10-07 |
| PostgreSQL TeamStore | 可用 | server 設定的 `postgres` | [PostgreSQL 一致性測試](../crates/store/speclink-store-postgres/tests/it/conformance.rs)<br>[韌性測試](../crates/store/speclink-store-postgres/tests/it/resilience.rs) | server 仍是單一 instance。 | 2026-10-07 |
| `speclink-server` | 可用 | npx、Docker、Compose；自原始碼建置 native binary | [server 入口](../crates/host/speclink-server/src/main.rs)<br>[CLI 到 server 端到端測試](../crates/host/speclink-server/tests/it/e2e_cli.rs) | Release 不附 native binary。 | 2026-10-07 |
| Server 後台、設定與帳號 | 可用 | `/setup`、`/admin`、`/account`、存取金鑰、device 登入、邀請與命令列管理指令 | [後台端到端測試](../crates/host/speclink-server/tests/it/admin/e2e.rs)<br>[device 登入端到端測試](../crates/host/speclink-server/tests/it/identity/device_e2e.rs) | 沒有 SSO，見[路線圖](roadmap.zh-TW.md)。 | 2026-10-07 |
| Server 營運 | 可用 | 部署、健康檢查、`backup`、`verify-backup`、`restore` | [部署文件](server-deployment.zh-TW.md)<br>[備份端到端測試](../crates/host/speclink-server/tests/it/admin/backup_e2e.rs) | 備份要停機；沒有滾動升級與多節點。 | 2026-10-07 |
| Remote CLI 與唯讀投影 | 可用 | `speclink link`、`auth`、`artifact`；唯讀的 `.speclink/context/` | [Remote CLI 測試](../crates/adapters/speclink-cli/tests/it/remote_read_path.rs)<br>[投影實作](../crates/host/speclink-host/src/projection.rs) | — | 2026-10-07 |
| 遠端任務證據 | 可用 | 遠端的 `speclink task done` 把 touched files 存進 Store | [證據實作](../crates/engine/speclink-core/src/lifecycle/tasks.rs)<br>[遠端證據端到端測試](../crates/host/speclink-server/tests/it/phase2_chain.rs) | 桌面遠端看板勾任務時不送 touched files。 | 2026-10-07 |
| 桌面 Server 連線 | 可用 | 設定裡的 Server 清單、device 登入、存取金鑰、登出與 OS Keychain | [連線流程](../apps/desktop/src-tauri/src/connections.rs)<br>[Servers 面板測試](../apps/desktop/src/__tests__/serversPanel.test.tsx) | — | 2026-10-07 |
| 桌面遠端 workspace | 部分可用 | workspace chooser 的遠端開啟：只看規格，或綁本機 checkout；詳情面板可認領 | [Workspace chooser](../apps/desktop/src/components/WorkspaceChooser.tsx)<br>[遠端開啟測試](../apps/desktop/src/__tests__/remoteOpen.test.ts) | 可瀏覽、勾任務、讀寫 artifact。缺口：勾任務不回報 touched files；認領沒有釋放或接手的動詞。 | 2026-10-07 |
| Claude Code 外掛（`speclink-skills`） | 部分可用 | `/plugin install speclink-skills --marketplace MomoChenisMe/speclink` | [外掛測試](../integrations/claude-code/speclink-skills/tests/register.test.tsx)<br>[技能分組測試](../scripts/claude-code/skill-groups.test.mjs) | 建立在 Claude Code 的 early-access 外掛介面上，Claude Code 改版時可能要跟著更新。Windows 尚未實機測試。 | 2026-10-07 |
| MCP 與 Agent 工具套件 | 規劃中 | 沒有入口 | [方向與下一步](roadmap.zh-TW.md) | 見路線圖的「Agent 工具整合」。 | 2026-10-07 |
| SSO、runtime plugin 與多節點 | 規劃中 | 沒有入口 | [方向與下一步](roadmap.zh-TW.md) | 見路線圖的「系統整合」。 | 2026-10-07 |

## <a id="recheck"></a>怎麼重新查核

之後要更新這份文件時，用當下的 checkout 重做判斷，不要沿用舊日期的結論：

1. 執行 `speclink --help` 與各子指令的 `--help`，核對 CLI 入口。
2. 執行 `speclink-server --help`，核對 server、帳號與備份的入口。
3. 比較 `.claude/skills/` 與 `.agents/skills/` 的目錄清單。兩邊只差 `speclink-analyze`（只有 Claude 有）。
4. 用[動詞契約的正式規格](../openspec/specs/verb-contract/spec.md)核對本地與遠端對照表。
5. 用 `npm view @speclink/engine version` 等指令核對 npm 上的版本。
6. 交叉核對 `Cargo.toml`、各 package 的 scripts、整合與端到端測試，以及正式規格。沒有使用者入口的能力，不能因為 crate 存在就標成可用。

## <a id="doc-gaps"></a>已知文件缺口

- 還沒有「從零做一個客戶端」的文件。方向見[路線圖](roadmap.zh-TW.md)的「以引擎自建客戶端」。

## <a id="related"></a>相關文件

- [完整 SDD 工作流](workflow.zh-TW.md)：每一站的用途、技能、完成判準與下一站。
- [專案路線圖](roadmap.zh-TW.md)：還沒做的方向。
- [Server 部署](server-deployment.zh-TW.md)、[儲存後端](server-store-drivers.zh-TW.md)、[備份與還原](server-backup.zh-TW.md)：server 的營運方式。

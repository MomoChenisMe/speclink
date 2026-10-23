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
| 桌面看板與詳情頁 | 可用 | 部分可用 | 遠端看板在桌面上勾任務時，不回報 touched files。 |
| 帳號、存取金鑰與 membership | 不適用 | 可用 | 本地不需要帳號。 |
| 備份還原 | 由 Git 負責 | 可用 | 遠端備份目前要停機進行。 |
| 離線工作 | 可用 | 需連線 | 遠端斷線時只能讀，寫入會被拒絕。 |

## <a id="capabilities"></a>能力清單

| 能力 | 狀態 | 使用者入口 | 證據 | 限制與下一步 | 查核 |
| --- | --- | --- | --- | --- | --- |
| Local Repo CLI | 可用 | `speclink init`、`list`、`show`、`status`、`validate`、`analyze`、`drift`、`archive`、`discuss` 等 | [CLI 入口](../crates/adapters/speclink-cli/src/main.rs)<br>[CLI 整合測試](../crates/adapters/speclink-cli/tests/it/doc_verbs.rs) | 不需要 server。旗標以各子指令的 `--help` 為準。 | 2026-10-07 |
| Agent 技能 | 可用 | Claude、GitHub Copilot 與 Antigravity `/speclink-*`、Codex `$speclink-*` | [生成的 apply 技能](../.agents/skills/speclink-apply/SKILL.md)<br>[生成的 manual 技能](../.claude/skills/speclink-manual/SKILL.md)<br>[自訂工具描述子測試](../crates/adapters/speclink-cli/tests/it/tools_descriptor.rs)<br>[Copilot 技能快照](../crates/engine/speclink-core/tests/golden/copilot.snapshot.md) | `analyze` 技能只有 Claude 有，Codex、GitHub Copilot 與 Antigravity 直接用 CLI。技能數量：`worktree` 政策關閉時 Claude 18 個、Codex、GitHub Copilot 與 Antigravity 各 17 個；開啟時各多 2 個。其他 AI 工具可用自訂工具描述子產生技能。 | 2026-10-21 |
| 品質關卡 | 可用 | `/speclink-review`、`/speclink-verify`、`/speclink-quality`；CLI `speclink review`、`speclink verify` | [關卡的蓋章與工單規則](../crates/engine/speclink-core/src/quality/station.rs)<br>[review 動詞測試](../crates/adapters/speclink-cli/tests/it/review_verbs.rs) | SUGGESTION 不擋章。蓋章後範圍內的檔案再被改，章會降級為「其後有變動」。 | 2026-10-07 |
| 執行順序 | 可用 | `speclink plan`、`speclink change depends`、`speclink change rank`；桌面變更詳情頁的「排程」文件 | [plan 動詞測試](../crates/adapters/speclink-cli/tests/it/plan_verbs.rs)<br>[server 的 plan API 測試](../crates/host/speclink-server/tests/it/api/plan_api.rs) | `change rank` 與 `plan --strict-overlap` 只限本地。 | 2026-10-15 |
| 手冊與溯源 | 可用 | `/speclink-manual`、桌面「手冊」頁；`speclink trace`、`/speclink-trace` | [手冊頁測試](../packages/ui/src/__tests__/manualPage.test.tsx)<br>[trace 測試](../crates/adapters/speclink-cli/tests/it/trace.rs) | 遠端專案還不能生成手冊（導覽可以）。`trace` 只限本地。 | 2026-10-07 |
| Local 桌面 app | 可用 | 看板、詳情頁、規格、已封存、手冊、設定模式（應用程式設定、專案設定與「關於 Speclink」頁）與系統匣 | [Desktop scripts](../apps/desktop/package.json)<br>[Desktop UI 測試](../apps/desktop/src/__tests__/App.test.tsx)<br>[設定模式測試](../apps/desktop/src/__tests__/settingsColumn.test.tsx)<br>[關於頁測試](../apps/desktop/src/__tests__/aboutPage.test.tsx)<br>[自動更新測試](../apps/desktop/src/__tests__/updater.test.ts) | 遠端專案的狀態另列在下面。 | 2026-10-15 |
| 桌面配色與設定導覽 | 可用 | 七色固定深淺、主視窗與系統匣共用；設定頁點專案返回看板 | [配色測試](../apps/desktop/src/__tests__/projectColors.test.ts)<br>[導覽測試](../apps/desktop/src/__tests__/store.test.ts) | 色票只存這台裝置，與團隊政策分開。 | 2026-10-19 |
| 桌面網路與診斷 | 可用 | Debug 開關、受限紀錄與 ZIP 匯出、系統／手動／直接 Proxy、手動更新連線測試 | [網路設定頁測試](../apps/desktop/src/__tests__/networkDiagnosticsPage.test.tsx)<br>[Proxy 與錯誤分類測試](../crates/host/speclink-host/src/network.rs) | [固定 Proxy 範圍與限制](configuration.zh-TW.md#desktop-network-diagnostics)；不支援企業驗證或自訂 CA，公司失敗原因仍需實際紀錄。 | 2026-10-19 |
| 安全標準離線交換 | 可用 | Desktop 與 CLI 匯出單版本／全部有效版本，匯入 v1 單包與 v2 集合包 | [CLI 往返測試](../crates/adapters/speclink-cli/tests/it/security.rs)<br>[集合包驗證與原子輸出測試](../crates/engine/speclink-fs/src/security_packages.rs) | [集合匯入結果](configuration.zh-TW.md#offline-standard-bundles)採逐版本提交；失敗或取消保留先前成功項，不保證整批回復。 | 2026-10-19 |
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
| 桌面 Server 連線 | 可用 | 設定的「伺服器」頁、device 登入、存取金鑰、登出與 OS Keychain | [連線流程](../apps/desktop/src-tauri/src/connections.rs)<br>[Servers 面板測試](../apps/desktop/src/__tests__/serversPanel.test.tsx) | — | 2026-10-15 |
| 桌面遠端專案 | 部分可用 | 「新增專案」對話框的遠端開啟：只看規格，或綁本機 checkout；詳情頁可認領 | [新增專案對話框](../apps/desktop/src/components/WorkspaceChooser.tsx)<br>[遠端開啟測試](../apps/desktop/src/__tests__/remoteOpen.test.ts) | 可瀏覽、勾任務、讀寫 artifact。缺口：勾任務不回報 touched files；認領沒有釋放或接手的動詞。 | 2026-10-07 |
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

## <a id="offline-security-workflow"></a>桌面流程與離線安全審查（本次實作）

本分支更新來源為 `mosutw/speclink`，保留原作者的獨立版本與連結，缺簽章更新包時提供 Release 手動下載。README 分別標示 mosutw 維護、Codex 的 AI 協作與原作者；GitHub 側欄 Contributors 仍依提交署名統計。

安全標準資料可在裝置層設定位置，預設本機，Windows 支援 UNC；同機 Desktop／CLI 使用相同路徑，只共用標準。唯讀共用可審查／匯出，自訂根停用刪除，工作中不能切換，斷線不改到本機重下載。私人報告與 `.security_kb/` 快照留本機。本機模擬與 Windows 鎖定／搬移測試通過；實際 NAS／SMB 與公司 Proxy 尚待驗收。完成檢查後合併 `main`，從該筆 commit 打包，再做公司驗收。操作與舊寫入者升級限制見[開發說明](development.zh-TW.md#desktop-tags)。

以下入口已於目前原始碼實作；安裝版須更新至包含本變更的版本。Remote 必須回報 `securityReviewV2` 與 `checkHistory` 能力，舊 server 明確停用新政策與寫入，不把不支援當作通過。

1. 左側專案列可拖曳，或使用上移／下移；順序只存這台裝置，重開還原。
2. 設定 → 軟體更新可直接讀內附版本與更新日誌。手動連線檢查另顯示 Proxy、TLS、407 或解析失敗；公司 Proxy 尚待[實機驗收](../openspec/changes/archive/2026-10-23-desktop-workflow-and-offline-security-review/evidence/company-proxy.md)，不宣稱已修復公司環境。
3. 專案設定 → 審查規則 → 預設清單 →「查看 12 條」，每項問號說明用途、例子與效果，可用鍵盤開啟。
4. 系統設定 → 安全標準資料先下載／離線匯入指定版本的原文、對照表與修復最佳實踐文件；CLI `speclink security kb prepare --root <專案>` 或 Desktop 的準備知識庫建立 `.security_kb/`。缺件拒絕且保留舊快照，審查期間不得下載資源。
5. 只用專案設定選定的標準、版本與 ASVS 等級，不自行補查其他版本或標準。
6. 審查與驗證蓋章前先保存完成紀錄；安全報告同樣在提案的 `checks/`。Desktop 活動與已封存詳情的「檢查紀錄」可隨時唯讀查詢；CLI `review history`／`verify history` 與 `security report ID --change CHANGE` 可讀回，不重新掃描。
7. 專案設定 → 產出流程的審查／驗證／安全審查保留選項預設全開，僅影響封存要帶走的完整紀錄。活動提案仍保存所有完成紀錄，未結工單仍隨提案保留；不回溯刪除先前封存資料。納入驗收的封存另保留可查的安全驗收摘要，包含指紋與接受理由，不包含完整報告或程式片段。
8. 安全資料下載清單由固定來源與 SHA 的 `spec_registry.json` 定義；附件僅是資料結構參考，不能當作執行指令或未核實來源。
9. 內建基本掃描保留；`/speclink-security-review CHANGE`（Codex 使用 `$speclink-security-review`）另外讀取本機知識庫，針對五種語言的架構、業務邏輯與資料流產出繁中九項結構報告。skill 是使用者明確執行的深度審查，不以基本掃描替代，也不把連網 AI 稱為離線。
10. OWASP／CWE 主開關下的「納入驗收條件」預設不勾選：結果只供參考。勾選並啟用安全審查後，缺資料、過期、未驗證及未處理的 CRITICAL／HIGH／MEDIUM 會阻擋驗證蓋章與封存；LOW 只列建議。需要接受風險時，用 `security review accept CHANGE --stdin` 提供報告、發現編號與具體理由，不能用一般 `--accept` 跳過。關閉主開關暫停生效但保留勾選值。
11. 審查與驗證紀錄檔名以該輪開始時間（UTC，`YYYYMMDDHHmmss`）開頭，舊版 ULID 檔名仍可讀。Desktop 詳情的品質關卡中，點「已審查」或「已驗證」可各自展開開始時間倒序的紀錄；選取後在既有檢視器唯讀查看。

所有專案的提案預設以條列方式呈現，包含既有散文提案；不重寫舊文件。Remote 基本掃描可離線產生待發佈草稿，僅明確提交深度報告時連線發佈成對報告；讀取自己的 server 紀錄不等於下載標準資源。Remote 原始碼新鮮度由當次授權 checkout 核對並標示 client-attested，server 不會假稱自行讀過原始碼。通過只涵蓋本次證據與規則，並非資安認證。

驗證見[封存保留](../openspec/changes/archive/2026-10-23-desktop-workflow-and-offline-security-review/evidence/archive-retention.md)、[Remote 安全流程](../openspec/changes/archive/2026-10-23-desktop-workflow-and-offline-security-review/evidence/remote-security.md)與[Desktop 安全流程](../openspec/changes/archive/2026-10-23-desktop-workflow-and-offline-security-review/evidence/desktop-security.md)。

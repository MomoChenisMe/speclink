# 專案能力狀態

**繁體中文** · [English](product-status.md)

最後查核日期：**2026-11-18**（0.12.0 之後的原始碼）。

這份文件回答「這項能力現在能不能用」。行為的正式定義在 `speclink/specs/`；還沒做的方向在[專案路線圖](roadmap.zh-TW.md)。只有程式碼、crate 或正式規格存在，不代表使用者已經用得到。

要實際操作遠端專案（示範用 server、Speclink Desktop 與 CLI），照 [Remote 入門](remote-getting-started.zh-TW.md)做一次。

## <a id="status-model"></a>狀態怎麼判定

| 狀態 | 意思 |
| --- | --- |
| Available（可用） | 有使用者入口，而且有兩項互相獨立的證據，或一項端到端測試。 |
| Partial（部分可用） | 有一部分可以用，完整流程還有明確的缺口。 |
| Planned（規劃中） | 還沒有可用的入口。細節見[專案路線圖](roadmap.zh-TW.md)。 |
| Deprecated（已棄用） | 程式裡還找得到，但已經不是目標架構。目前沒有這類項目。 |

## <a id="local-and-remote"></a>本機專案與遠端專案對照

本機專案與遠端專案的差異都在這張表。「遠端專案」一欄以官方的示範用 server `speclink-server` 為量測對象。內嵌了引擎的宿主和它用同一套宿主套件、同一份儲存契約，所以行為相同；差異列在[已知缺口](#known-gaps)。

大多數 CLI 動詞在本機專案與遠端專案都能用。例外只有兩個，完整清單見[動詞與旗標契約](verb-contract.zh-TW.md)：

- 只限本機專案：`demo`。
- 只限遠端專案：`claim`。

`trace`、`change rank` 與 `plan --strict-overlap` 兩邊都能用。在不適用的專案執行 `demo` 或 `claim`，CLI 會明確拒絕，不會改走另一邊。

| 能力 | 本機專案 | 遠端專案 | 說明 |
| --- | --- | --- | --- |
| 規格與變更的讀寫 | 可用 | 可用 | 本機專案直接讀寫 `speclink/`；遠端專案一律經 Server 寫入，checkout 裡沒有可寫的規格檔。 |
| 變更生命週期（`propose` → `apply` → `archive`） | 可用 | 可用 | 遠端專案一次只能封存一個變更，封存不留本機備份。`status` 與 `instructions` 的 `--schema` 旗標在遠端專案不接受，產出流程由 Server 的工作流設定決定。 |
| 討論（`discuss`） | 可用 | 可用 | 轉為變更、併入既有變更與封存，兩邊相同。 |
| 品質關卡（`review`、`verify`） | 可用 | 可用 | 工單、輪與蓋章的規則兩邊相同。 |
| 執行順序（`plan`、`change depends`、`change rank`、`change schema`） | 可用 | 可用 | `change rank` 與 `plan --strict-overlap` 兩邊都能用。遠端專案的 `change schema` 只解析內建的產出流程。 |
| 溯源（`trace`） | 可用 | 可用 | 兩邊的輸出相同。 |
| 認領變更與釋放認領（`claim`、`claim --release`） | 不適用 | 可用 | 認領人存進變更資料，Server 重啟後還在。認領別人持有的變更會回 409，訊息寫出持有人。有寫入權限的人都可以釋放認領；要更嚴格的宿主在掛鉤裡拒絕釋放。`show --json` 看得到認領的人。 |
| 示範資料（`demo`） | 可用 | 不適用 | 只限本機專案；在遠端專案直接拒絕，不送出請求。 |
| Agent 讀取背景資料 | 可用 | 可用 | 本機專案直接讀 repo；遠端專案讀唯讀的 `.speclink/context/`。 |
| 任務動到哪些檔案的證據 | 可用 | 可用 | 本機專案寫進 `.evidence.json`；遠端專案由 Server 保存，可用 `speclink evidence show` 或 `GET /changes/{name}/evidence` 讀回。 |
| Speclink Desktop 的看板與詳情頁 | 可用 | 部分可用 | 在 Speclink Desktop 勾遠端專案的任務時，不回報動到的檔案。 |
| 帳號、存取金鑰與成員資格 | 不適用 | 可用 | 本機專案不需要帳號。示範用 server 自己管帳號；內嵌的宿主用自己的帳號與登入。 |
| 備份還原 | 由 Git 負責 | 可用 | 示範用 server 的備份要停機進行。內嵌的宿主備份自己的資料庫，或用 `host.exportScope` 匯出一個範圍。 |
| 離線工作 | 可用 | 需連線 | 遠端專案斷線時只能讀，寫入會被拒絕。 |

## <a id="capabilities"></a>能力清單

| 能力 | 狀態 | 使用者入口 | 證據 | 限制與下一步 | 查核 |
| --- | --- | --- | --- | --- | --- |
| 本機專案的 CLI | 可用 | `speclink init`、`list`、`show`、`status`、`validate`、`analyze`、`drift`、`archive`、`discuss` 等 | [CLI 入口](../crates/adapters/speclink-cli/src/main.rs)<br>[CLI 整合測試](../crates/adapters/speclink-cli/tests/it/doc_verbs.rs) | 不需要 Server。旗標以各子指令的 `--help` 為準。 | 2026-11-18 |
| Agent 技能 | 可用 | Claude、GitHub Copilot 與 Cursor `/speclink-*`、Codex `$speclink-*`、pi `/skill:speclink-*` | [生成的 apply 技能](../.agents/skills/speclink-apply/SKILL.md)<br>[生成的 manual 技能](../.claude/skills/speclink-manual/SKILL.md)<br>[自訂工具描述子測試](../crates/adapters/speclink-cli/tests/it/tools_descriptor.rs)<br>[Copilot 技能快照](../crates/engine/speclink-core/tests/golden/copilot.snapshot.md)<br>[pi 技能快照](../crates/engine/speclink-core/tests/golden/pi.snapshot.md)<br>[Cursor 技能快照](../crates/engine/speclink-core/tests/golden/cursor.snapshot.md) | `analyze` 技能只有 Claude 有，Codex、GitHub Copilot、pi 與 Cursor 直接用 CLI。技能數量：`worktree` 政策關閉時 Claude 17 個，Codex、GitHub Copilot、pi 與 Cursor 各 16 個；開啟時各多 2 個。其他 AI 工具可用自訂工具描述子產生技能。 | 2026-11-10 |
| 品質關卡 | 可用 | `/speclink-review`、`/speclink-verify`、`/speclink-quality`；CLI `speclink review`、`speclink verify` | [關卡的蓋章與工單規則](../crates/engine/speclink-core/src/quality/station.rs)<br>[review 動詞測試](../crates/adapters/speclink-cli/tests/it/review_verbs.rs) | SUGGESTION 不擋章。蓋章後範圍內的檔案再被改，章會降級為「其後有變動」。 | 2026-10-07 |
| 執行順序 | 可用 | `speclink plan`、`speclink change depends`、`speclink change rank`、`speclink change schema`；桌面變更詳情頁的「排程」文件 | [plan 動詞測試](../crates/adapters/speclink-cli/tests/it/plan_verbs.rs)<br>[change schema 測試](../crates/adapters/speclink-cli/tests/it/change_schema.rs)<br>[遠端專案的 `change rank` 測試](../crates/adapters/speclink-cli/tests/it/remote_change_rank.rs)<br>[server 的 plan API 測試](../crates/host/speclink-server/tests/it/api/plan_api.rs) | 遠端專案只能用內建的產出流程，見[已知缺口](#known-gaps)。 | 2026-11-18 |
| 手冊與溯源 | 可用 | `/speclink-manual`、桌面「手冊」頁；`speclink trace`、`/speclink-trace` | [手冊頁測試](../packages/ui/src/__tests__/manualPage.test.tsx)<br>[trace 測試](../crates/adapters/speclink-cli/tests/it/trace.rs)<br>[遠端專案的唯讀動詞測試](../crates/adapters/speclink-cli/tests/it/remote_read_verbs.rs) | 遠端專案還不能生成手冊（導覽可以），見[已知缺口](#known-gaps)。 | 2026-11-18 |
| 本機專案的桌面 app | 可用 | 看板、詳情頁、規格、已封存、手冊、設定模式（應用程式設定、專案設定與「關於 Speclink」頁）與系統匣 | [Desktop scripts](../apps/desktop/package.json)<br>[Desktop UI 測試](../apps/desktop/src/__tests__/App.test.tsx)<br>[設定模式測試](../apps/desktop/src/__tests__/settingsColumn.test.tsx)<br>[關於頁測試](../apps/desktop/src/__tests__/aboutPage.test.tsx)<br>[自動更新測試](../apps/desktop/src/__tests__/updater.test.ts) | 遠端專案的狀態另列在下面。 | 2026-11-18 |
| 以網址開啟詳情頁 | 可用 | 外部程式送出 `speclink://change/<變更名稱>?project=<專案路徑>` 或 `speclink://discussion/<討論 slug>?project=<專案路徑>`，桌面 app 打開該專案的詳情頁；送出 `speclink://project?project=<專案路徑>` 時，桌面 app 切到該專案並顯示看板 | [網址解析與派送測試](../apps/desktop/src/__tests__/deepLink.test.ts) | 只開啟與檢視，只限本機專案（帶 remote 標記的資料夾改開「新增專案」對話框）。Windows、Linux 只能開一個 app，再次啟動會把既有視窗帶到前景。 | 2026-11-04 |
| 安裝通路 | 可用 | 桌面安裝檔（macOS universal dmg、Windows 安裝器、Linux AppImage）；CLI 的 npm、安裝腳本、Homebrew；示範用 server 的 npx 與 Docker | [npm 啟動器測試](../scripts/npm/npm-cli-launcher.test.mjs)<br>[安裝腳本測試](../scripts/install.test.mjs)<br>[Homebrew formula 產生器](../scripts/release/homebrew-formula.mjs) | Windows 安裝檔沒有程式碼簽章，第一次執行要放行 SmartScreen。 | 2026-11-18 |
| Node SDK 與宿主套件（`@speclink/engine`） | 可用 | `npm install @speclink/engine`；`createEngine`（本機專案）、`createHost`（內嵌）、`runStoreConformance`（合格測試）、`agentTool`（Agent 工具定義）；`list --json --detail` 一次呼叫就帶出全部變更的關卡、認領、分支與落地 | [套件入口](../crates/adapters/speclink-node/package.json)<br>[宿主套件測試](../crates/adapters/speclink-node/__test__/host.spec.ts)<br>[HTTP 入口測試](../crates/adapters/speclink-node/__test__/handle-request.spec.ts)<br>[合格測試的測試](../crates/adapters/speclink-node/__test__/port-conformance.spec.ts)<br>[版號測試](../scripts/npm/npm-engine-package.test.mjs) | 動詞都經 `dispatch` 以字串參數呼叫，還沒有型別化的 JS 方法，見[路線圖](roadmap.zh-TW.md)。內嵌的限制見[已知缺口](#known-gaps)。 | 2026-11-18 |
| 引擎、宿主套件與線上協定 | 可用 | CLI、示範用 server 與 Node SDK 共用的 Rust crates：`speclink-core`、`speclink-host`、`speclink-protocol` | [本機與儲存接口兩條路徑的對照測試](../crates/host/speclink-host/tests/port_dual_path.rs)<br>[HTTP 入口測試](../crates/host/speclink-host/tests/http_door.rs)<br>[Client Protocol 正式規格](../speclink/specs/client-protocol/spec.md) | HTTP 入口固定掛在 `/api/speclink/v1/` 底下。其餘限制見[已知缺口](#known-gaps)。 | 2026-11-18 |
| 儲存接口的範例 | 可用 | `crates/store/` 底下的三個 Rust crate：SQLite、PostgreSQL、檔案系統。它們也是示範用 server 的 `sqlite`、`postgres`、`serverfs` 三種儲存後端，說明見[儲存接口的範例](server-store-drivers.zh-TW.md) | [SQLite 範例的合格測試](../crates/store/speclink-store-port-sqlite/tests/it/conformance.rs)<br>[PostgreSQL 範例的合格測試](../crates/store/speclink-store-port-postgres/tests/it/conformance.rs)<br>[檔案系統範例的合格測試](../crates/store/speclink-store-port-fs/tests/it/conformance.rs) | 沒有 JavaScript 寫的範例，見[已知缺口](#known-gaps)。範例不做連線池、TLS 與配置版本之間的遷移，也不保留文件的舊版內容。 | 2026-11-18 |
| 示範用 server（`speclink-server`） | 可用 | npx、Docker、Compose；自原始碼建置 native binary | [server 入口](../crates/host/speclink-server/src/main.rs)<br>[CLI 到 server 端到端測試](../crates/host/speclink-server/tests/it/e2e_cli.rs) | 給示範與試用；正式使用請把引擎內嵌進自己的系統。Release 不附 native binary。只支援單一實例；進行中的變更很多時，列出變更對儲存的呼叫數會成長。兩項都在[已知缺口](#known-gaps)。 | 2026-11-18 |
| 示範用 server 的後台、設定與帳號 | 可用 | `/setup`、`/admin`、`/account`、存取金鑰、裝置登入、邀請與命令列管理指令 | [後台端到端測試](../crates/host/speclink-server/tests/it/admin/e2e.rs)<br>[裝置登入端到端測試](../crates/host/speclink-server/tests/it/identity/device_e2e.rs) | 沒有 SSO，見[路線圖](roadmap.zh-TW.md)。 | 2026-11-18 |
| 示範用 server 的營運 | 可用 | 部署、健康檢查、`backup`、`verify-backup`、`restore` | [部署文件](server-deployment.zh-TW.md)<br>[備份端到端測試](../crates/host/speclink-server/tests/it/admin/backup_e2e.rs)<br>[舊資料拒絕啟動的測試](../crates/host/speclink-server/tests/it/legacy_layout.rs) | 備份要停機；沒有滾動升級與多節點。從 0.12.0 或更早的版本升級要走備份與還原，見 [Server 部署](server-deployment.zh-TW.md#upgrade-from-0-12)。 | 2026-11-18 |
| 遠端專案的 CLI 與唯讀投影 | 可用 | `speclink link`、`auth`、`update`（技能由 Server 提供）、`artifact`；唯讀的 `.speclink/context/` | [遠端專案的 CLI 測試](../crates/adapters/speclink-cli/tests/it/remote_read_path.rs)<br>[技能取自 Server 的測試](../crates/adapters/speclink-cli/tests/it/remote_skills.rs)<br>[投影實作](../crates/host/speclink-host/src/projection.rs) | 連到沒有宣告技能的舊版 Server 時，改用本機的範本，見[已知缺口](#known-gaps)。 | 2026-11-18 |
| 遠端專案的任務證據 | 可用 | 在遠端專案執行 `speclink task done`，動到的檔案交給 Server 保存；`speclink evidence show` 讀回 | [證據實作](../crates/engine/speclink-core/src/lifecycle/tasks.rs)<br>[遠端證據端到端測試](../crates/host/speclink-server/tests/it/phase2_chain.rs) | 在 Speclink Desktop 勾遠端專案的任務時不送出動到的檔案。宿主經 `host.dispatch` 勾任務時也不記檔案，見[已知缺口](#known-gaps)。 | 2026-11-18 |
| 桌面 Server 連線 | 可用 | 設定的「伺服器」頁、瀏覽器登入、裝置登入、存取金鑰、登出與 OS Keychain | [連線流程](../apps/desktop/src-tauri/src/connections.rs)<br>[Servers 面板測試](../apps/desktop/src/__tests__/serversPanel.test.tsx) | 瀏覽器登入要 Server 宣告它；沒有宣告時用 裝置登入或存取金鑰。 | 2026-11-18 |
| Speclink Desktop 的遠端專案 | 部分可用 | 「新增專案」對話框的遠端開啟：只看規格，或綁本機 checkout；詳情頁可認領；「遷移到 Server」把本機專案搬上去 | [新增專案對話框](../apps/desktop/src/components/WorkspaceChooser.tsx)<br>[遠端開啟測試](../apps/desktop/src/__tests__/remoteOpen.test.ts) | 可瀏覽、勾任務、讀寫 artifact。缺口：勾任務不回報動到的檔案；沒有釋放認領的按鈕（改用 CLI 的 `speclink claim <變更> --release`）。 | 2026-11-18 |
| Claude Code 外掛（`speclink-skills`） | 部分可用 | `/plugin install speclink-skills --marketplace MomoChenisMe/speclink` | [外掛測試](../integrations/claude-code/speclink-skills/tests/register.test.tsx)<br>[技能分組測試](../scripts/claude-code/skill-groups.test.mjs) | 建立在 Claude Code 的 early-access 外掛介面上，Claude Code 改版時可能要跟著更新。Windows 尚未實機測試。 | 2026-10-07 |
| Herdr mod（`speclink`） | 部分可用 | `cp -R integrations/herdr/mods/speclink integrations/herdr/mods/speclink-new ~/.config/herdr-app/mods/` | [面板邏輯測試](../scripts/herdr/panel-model.test.mjs)<br>[安裝說明](../integrations/herdr/README.md) | 停靠的版面、側欄清單、檔案和變更面板是 Herdr 內建版面與 mod 的複製版，Herdr 更新後要重做停靠。內建畫面的翻譯靠文件沒寫的行為，Herdr 改掉後會回到英文。貼進輸入框和開頁面只有 macOS。沒有用 Codex 測試過。 | 2026-11-04 |
| MCP | 規劃中 | 沒有入口 | [方向與下一步](roadmap.zh-TW.md) | 見路線圖的「Agent 工具整合」。 | 2026-11-18 |
| SSO、runtime plugin 與多節點 | 規劃中 | 沒有入口 | [方向與下一步](roadmap.zh-TW.md) | 見路線圖的「系統整合」。 | 2026-10-07 |

## <a id="known-gaps"></a>已知缺口

這一節列出這一版刻意延後或不做的項目。每一項寫明影響誰，以及今天可以怎麼做。屬於之後方向的項目，在[專案路線圖](roadmap.zh-TW.md)有落點。

| 缺口 | 影響誰 | 目前的替代做法 |
| --- | --- | --- |
| 沒有本機規格檔的內嵌宿主不能用需要 checkout 的動詞：`drift`、`review prepare`、`review scope`、`verify scope`，以及帶 `--skill` 的 `instructions`。技能裡「讀使用者給的檔案路徑」這種輸入，也只在 Agent 有檔案系統時可用。 | 經 `host.dispatch` 執行動詞的宿主與它的 Agent 工具。這幾個動詞會被拒絕，錯誤碼是 `invalid_argv`。 | 由工程師在連到這個遠端專案的 checkout 用 CLI 執行這幾個動詞。Agent 沒有檔案系統時，請使用者把文件內容直接貼進對話。 |
| `task done` 不替宿主收集改了哪些檔；蓋章與開工標記也不替宿主讀 commit 與分支。宿主套件不讀 git。 | 經 `host.dispatch` 勾任務、蓋章或標記開工的宿主。任務照常勾選，但 evidence 不記檔案、repo 與 commit。 | 需要 evidence 時，由工程師在 checkout 用 CLI 執行 `speclink task done`，CLI 會收集。蓋章的 commit 由宿主放進 `--stdin` 的 JSON（`headCommit`）。落地的 commit 與分支用 `speclink change landed <名稱> --commit <sha> --branch <分支>` 回報。 |
| 用 Node 內嵌的宿主沒有現成的推送事件。 | 連到這種宿主的 Speclink Desktop 與其他用戶端：畫面靠定時詢問更新，不是即時。 | 不用設定，用戶端會自己輪詢。要即時更新的宿主自己提供 SSE 或 WebSocket 串流，並用 `createHost` 的 `pushTransports` 宣告網址。 |
| Speclink Desktop 沒有釋放認領的按鈕。 | 只用 Speclink Desktop 操作遠端專案，又要交出自己認領的變更、或接手離開的人認領的變更的人。 | 在連到這個遠端專案的 checkout 執行 `speclink claim <變更> --release`。在內嵌引擎的宿主裡，Agent 工具執行同一個動詞。釋放之後，再回 Speclink Desktop 認領。 |
| `baseline` 與 `manual` 兩支技能在遠端專案被拒絕。 | 遠端專案裡，要從既有程式碼建立規格基準或要生成手冊的人。 | `manual` 的導覽模式照常可用（`/speclink-manual 導覽`）。規格基準先在本機專案做完，再用 Speclink Desktop 的「遷移到 Server」搬到一個空的遠端專案。遠端專案的手冊今天沒有替代做法。 |
| 連到沒有宣告技能的舊版 Server 時，CLI 用本機的範本，worktree 的兩支技能不會產生。 | 連到舊版示範用 server 的遠端專案使用者。 | 把 Server 升級到會提供技能的版本，再執行 `speclink update`。升級之前，本機範本產生的其他技能照常可用。 |
| 已封存的變更與討論不能改自由欄位；也沒有依欄位查詢的動詞。 | 用自由欄位把變更對到自己的工單或看板的宿主。 | 封存之前先設好欄位，欄位會跟著文件進封存。要查詢時，讀 `speclink list --json`、`speclink show <名稱> --json` 與 `speclink discuss list --json` 的 `fields`，在自己的程式裡篩選。 |
| CLI 沒有匯入與匯出的動詞。 | 要把專案搬進、搬出遠端專案，或要定期備份的人。 | 本機專案搬到 Server：用 Speclink Desktop 的「遷移到 Server」。宿主的程式：呼叫 `host.exportScope` 與 `host.importScope`，或使用 HTTP 入口的 `GET /export` 與 `POST /import`。示範用 server：用後台「系統」頁的匯出，或 `speclink-server backup`。 |
| 沒有 JavaScript 寫的儲存接口範例。 | 用 Node 內嵌、要自己寫三個儲存函式的整合者。 | 從 Node SDK 文件[實作三個儲存函式](sdk-node.zh-TW.md#store-port)裡用 Map 寫的最小範例起步，對照三個 Rust 範例的資料表與提交步驟，寫完用 `runStoreConformance` 檢查。 |
| 示範用 server 只支援單一實例。 | 示範用 server 的管理者。 | 一份儲存只跑一個 server 程序。需要多個實例時，把引擎內嵌進自己的系統，由自己的資料庫負責交易。 |
| 規格放在 checkout 之外的本機位置。`spec_dir` 今天不檢查絕對路徑與 `../`：寫了會被接受，但行為未定義。 | 想把規格資料夾放到 repo 外面的本機專案使用者。 | `spec_dir` 只寫專案根目錄底下的相對路徑。要讓幾個 repo 共用一份規格，改用遠端專案。 |
| 唯讀引用另一套規格。 | 想在一個專案裡引用另一個專案的正式規格的人。 | 沒有內建的引用。要對照時，到另一個專案執行 `speclink show <capability> --item-type spec` 讀出內容。 |
| 遠端專案的自訂產出流程。 | 想在遠端專案使用自己定義的產出流程的團隊。 | 用內建的兩種：`spec-driven` 與 `no-spec-change`。自訂的產出流程只在本機專案可用。 |
| 嚴格一致的讀取快照。一次讀取可能對儲存發出兩到三次呼叫；中間剛好有人提交時，會短暫看到新舊混合的內容。 | 讀看板、清單與搜尋的人。寫入不受影響，仍由提交時的版本比對把關。 | 重新讀一次就一致。要嚴格一致的宿主，在自己的 `list` 與 `read` 函式裡處理。`host.exportScope` 不受影響：匯出期間範圍有變動時它會重來。 |
| 多個宿主實例之間的推送事件。 | 跑多個宿主實例、又要即時更新的系統。 | 預設用輪詢，實例之間不必傳遞事件。要推送時，宿主自己做跨實例的傳遞（事件在儲存的 `commit` 收到），並用 `pushTransports` 宣告串流。 |
| 沒有依範圍版本號的快取。進行中的變更很多時，列出變更對儲存的呼叫數會跟著成長。 | 進行中的變更很多、或儲存延遲高的宿主與示範用 server。 | 做完的變更盡快封存，讓進行中的清單保持短；讀取類的動詞不讀已封存文件的內容。 |
| 遠端專案的封存不留本機備份；回應的 `snapshotCreated` 一律是 `false`。 | 習慣用本機封存備份還原的遠端專案使用者。 | 封存後的內容在 Server 上，用 `speclink artifact cat` 或 Speclink Desktop 的「已封存」頁讀回。要備份整個範圍，用 `host.exportScope` 或示範用 server 的 `speclink-server backup`。 |

先前的提案寫過、後來已經解決的缺口，不在上面的表裡：

- 連到示範用 server 時清單沒有最後修改時間：已解決，`speclink list --json` 帶 `modifiedAt`。
- Node SDK 只支援 9 個動詞、不能用工作流設定的動詞：已解決，全部有引擎命令的動詞都能用。
- 宿主不能交出自己的儲存、沒有每次呼叫帶身分的入口、沒有執行前的檢查：已解決，見 `createHost`、`host.dispatch` 與執行前掛鉤。
- 內嵌的宿主不能匯入：已解決，見 `host.importScope` 與 HTTP 入口的 `POST /import`。
- 沒有任何 Server 回答支援哪些登入方式：已解決，宿主用 `authMethods` 宣告，示範用 server 也會回答。
- 遠端專案的技能來自本機的 CLI、worktree 的兩支技能不會產生：已解決，技能由 Server 提供。連上遠端專案之後技能檔不會自動重新產生，這一步寫在 [Remote 入門](remote-getting-started.zh-TW.md)：執行 `speclink update`。
- 沒有把封存的討論記錄搬回來的動詞：已解決，執行 `speclink discuss restore <slug>`，同一個 slug 最新的一份封存記錄會回到進行中，內容不變。本機專案、遠端專案與內嵌的宿主都能用。
- 沒有寫入詞彙表的動詞：已解決，用 `speclink language show` 讀出全文，修改後用 `speclink language replace --stdin` 整份寫回。本機專案、遠端專案與內嵌的宿主都能用。

## <a id="recheck"></a>怎麼重新查核

之後要更新這份文件時，用當下的 checkout 重做判斷，不要沿用舊日期的結論：

1. 執行 `speclink --help` 與各子指令的 `--help`，核對 CLI 入口。
2. 執行 `speclink-server --help`，核對示範用 server、帳號與備份的入口。
3. 比較 `.claude/skills/` 與 `.agents/skills/` 的目錄清單。兩邊只差 `speclink-analyze`（只有 Claude 有）。
4. 用 [CLI 的模式宣告](../crates/adapters/speclink-cli/src/main.rs)（`dispatch` 與 `spec_dispatch` 兩個函式）與[動詞契約的正式規格](../speclink/specs/verb-contract/spec.md)，核對本機專案與遠端專案的對照表。
5. 用 [Node SDK 的型別宣告](../crates/adapters/speclink-node/index.d.ts)核對 Node SDK 與宿主套件的匯出，以及需要 checkout 的動詞清單。
6. 用 `npm view @speclink/engine version` 等指令核對 npm 上的版本。
7. 交叉核對 `Cargo.toml`、各 package 的 scripts、整合與端到端測試，以及正式規格。沒有使用者入口的能力，不能因為 crate 存在就標成可用。
8. 逐項對照已知缺口與程式。已經解決的缺口移到表格下面的清單，不要直接刪掉。

## <a id="doc-gaps"></a>已知文件缺口

這一節只列還沒寫的文件。功能上的缺口在[已知缺口](#known-gaps)。

- 還沒有「從零做一個用戶端」的文件。方向見[路線圖](roadmap.zh-TW.md)的「以引擎自建用戶端與內嵌宿主」。

## <a id="related"></a>相關文件

- [完整 SDD 工作流](workflow.zh-TW.md)：每一站的用途、技能、完成判準與下一站。
- [專案路線圖](roadmap.zh-TW.md)：還沒做的方向。
- [Node SDK](sdk-node.zh-TW.md)：把引擎內嵌進自己的系統時要寫的程式。
- [Server 部署](server-deployment.zh-TW.md)、[儲存接口的範例](server-store-drivers.zh-TW.md)、[備份與還原](server-backup.zh-TW.md)：示範用 server 的營運方式。

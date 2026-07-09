> **Roadmap**: 四情境預設 GUI 工具矩陣的第 ③ 刀（共 5，序 4→3→2→1）。來源討論 `四情境預設-gui-工具矩陣`。
> **依賴**: ① desktop-shell-and-browser（復用其 React 元件庫）；概念上依賴 verb-contract／remote-connection 正典（已歸檔）。**下游**: ④ web-agent-channel 疊在本刀的 web server 之上；⑤ web-role-views 已取消（desktop 遠端模式取代 PO 瀏覽器零安裝，見下方 2026-07-09 重塑）。
> **狀態**: 待完整 propose（本檔為 promote 骨架）。
> **現況更新（2026-07-05，① 完成後）**: 可復用的 packages/ui 已含 shadcn/Tailwind 設計系統（teal 主色）與完整元件集（KanbanBoard 生命週期看板、RichDetailDrawer 詳情抽屜、TaskList 互動任務、ArchivedList 封存頁、Markdown 富文本）；資料源介面 SpeclinkDataSource 除唯讀外含寫入方法（setTaskDone/moveTask/deleteChange）——本刀的 HTTP adapter 需將其對應到動詞契約端點，或對 desktop 專屬操作優雅缺席（propose 時定案）。
> **現況更新（2026-07-06，desktop-board-parity／desktop-discussion-board 提案後）**: (1) in-progress 標記真相遷入 change meta（started_at/started_by/started_with）——標記騎在文件真相上，Postgres store 同步文件即同步狀態，**無需另設狀態表**；看板欄位派生規則（全完成＝已就緒＞有 started＝進行中＞其餘提案中）已定於 desktop-app spec。(2) Store trait 新增封存 artifact 讀取與封存 capability 列舉（帶預設實作）、active meta 原文讀寫對、discussions 既有方法——Postgres store 實作這些即得封存瀏覽與討論看板。(3) 桌面的即時刷新信號源是檔案監看（宿主層 wiring、不在 SpeclinkDataSource）——本刀的等價物是 server push（Postgres LISTEN/NOTIFY → SSE/WebSocket），propose 時定案傳輸形式。(4) SpeclinkDataSource 已再擴：封存文件讀取、封存 capability 列舉、討論清單/記錄/促轉/歸檔——HTTP adapter 需對應端點。(5) 狀態轉換分層定案：引擎（server 內嵌，經 createEngine）控轉換語意、store 控持久化、端點控授權與併發——多人下多步驟流程（如促轉＝建 change＋預填＋標記）的原子性與衝突語意為本刀 propose 時的設計題（Postgres 交易包覆 vs 端點層衝突回報；verb-contract 的 If-Match/409 reason 為既有雛形）。(6) 認領與開工的綁定為本刀 propose 時的定案項：verb-contract 既有 claim/ownership（可搶佔轉移、409 ownership_lost）與 change meta 的 started_*（冪等首次開工里程碑）是相鄰但不同的狀態——建議首次 claim 成功即蓋 started_*、ownership 轉移不改寫 started_*、看板「進行中」兩模式一致以 started_* 派生、目前持有人為 remote 加值顯示。(7) verb-contract 的 store 文件讀取動詞現僅涵蓋 active change（artifact cat）——封存文件讀取端點為本刀對 verb-contract 的 delta 新增項。
> **重塑（2026-07-09，manual-spec-edit-integrity 討論折入）**: 本刀範圍縮小而非擴大——server 轉為 **headless（無內建 web GUI）**，遠端 GUI 沿用桌面殼。理由：desktop 前端既隔著 SpeclinkDataSource 縫，新增 httpDataSource 即讓 desktop app 以遠端模式直連動詞契約端點，PO/RD 皆裝 desktop 連遠端，等於原計畫的 web GUI 前端寄宿在桌面殼裡（HTTP adapter 本就要做，成本幾乎免費）。連帶：(a) 瀏覽器零安裝 web GUI 遞延（React 元件庫保留日後補瀏覽器宿主的可能）；(b) ⑤ web-role-views 取消（desktop 遠端模式取代 PO 零安裝需求）；(c) 本刀交付的遠端 server 是「防止繞過動詞」強保證的唯一可達處——檔案模式追一致性＋知情使用、PAT 身分讓 started_by 不再自我宣告、轉換語意由 server 端點強制；檔案模式定位與限制須文件化。(d) 開箱即用傾向 docker-compose 打包 server＋Postgres（形態 design 定案）。② desktop-acp-agent 前提修正、Layer 1 不變量檢查另行處理，非本刀。

## Why

情境 3（本地 CLI＋遠端文件）目前只能對 tiny_http 假伺服器測試，沒有真正的 server 可連。本刀交付一個 **headless（無內建 web GUI）開箱即用的自架團隊 server**：經 `createEngine` 內嵌引擎、以 PostgreSQL 為 store 真相、對外暴露動詞契約 REST 端點（本地 CLI remote 模式 `link` 上來）。**遠端 GUI 沿用桌面殼**——desktop app 新增遠端模式，以 httpDataSource 實作既有 SpeclinkDataSource 直連同一組動詞契約端點；PO/RD 皆裝 desktop app 連遠端，不再需要瀏覽器零安裝版。這仍是情境 1/2 的共同底座——後兩刀只需在此之上疊 agent 通道與角色切面。

本刀同時是重塑後檔案模式定位的落地：本機個人（檔案）模式追狀態一致性、狀態記錄於文件跟隨 repo，供個人與小型團隊知情使用；要「防止繞過動詞」的強保證——PAT 身分讓 started_by 不再自我宣告、轉換語意由 server 端點強制（verb-contract 既有 If-Match/409、claim/ownership）——唯一可達處即本刀交付的遠端 server。

儲存用 PostgreSQL（team server 多人並發、中央治理），以 TypeScript 實作 Store（Node SDK 的 store bridge 已支援 JS 實作 Store），非另開 Rust adapter。開箱即用傾向 docker-compose 打包 server＋Postgres（具體形態 design 定案）。

## What Changes

- 新增 Node **headless** web server：經 `@speclink/engine` 的 `createEngine` 內嵌引擎；不內建 web GUI（瀏覽器版遞延，React 元件庫保留日後補瀏覽器宿主）。
- TypeScript PostgreSQL Store：以 pg 驅動實作 Store 介面（change/artifact/discussion/spec/WorkflowConfig），作為 store 真相。
- 動詞契約 REST 端點：對齊 verb-contract 正典，供 CLI remote 薄 client 呼叫（含 PAT 認證、repo 歸屬驗證）。
- **desktop 新增遠端模式**：以 httpDataSource 實作 SpeclinkDataSource 直連動詞契約端點；設定落 desktop 設定頁——server URL/repo 入 .speclink.yaml 頁簽（隨 repo 共享正確），PAT 沿用 CLI 使用者層級憑證存放（origin→token、0600、SPECLINK_TOKEN 可覆蓋，絕不入 repo）。
- **開箱即用部署**：docker-compose 打包 server＋PostgreSQL（具體形態 design 定案）。
- **定位文件化**：於產品文件（docs/team-mode.md 與 README）明載本機個人（檔案）模式的適用與限制、何時該轉遠端模式。

<!-- 細節（PostgreSQL schema、REST 端點對應、server push 傳輸形式 LISTEN/NOTIFY→SSE/WS、server 與 desktop 遠端模式同刀或分刀、docker-compose 具體形態、認證落地）待 /speclink-propose 於 design 階段定案 -->

## Non-Goals

- **不內建瀏覽器 web GUI**：遠端 GUI 由 desktop 遠端模式承接；瀏覽器零安裝版遞延至情境 1/2 真需要時（React 元件庫保留日後補瀏覽器宿主的可能）。
- **不在檔案模式加防篡改機關**（journal／鎖檔／checksum／常駐偵測）：檔案即 API 的相容性定位使「防止手改」結構上不可達，強保證由本刀的遠端 server 承接。
- **不含 agent 面板／通道**：agent 為外部獨立 Copilot SDK app（經 CLI remote 連本 server）或 server 側 SDK 直接串接（④ web-agent-channel 領域），非本刀範圍。
- **不做 Layer 1 不變量檢查**（懸空 promoted_to、meta 解析失敗浮現而非靜默吞掉）：對純個人模式亦有防呆價值但成本低，遞延為另立小 change。

## Capabilities

### New Capabilities

- `web-server`: Node **headless** web server——內嵌引擎、PostgreSQL Store、動詞契約 REST 端點，交付情境 3 的團隊 server（不含 web GUI）。
- `desktop-remote-mode`: desktop app 遠端模式——httpDataSource 實作 SpeclinkDataSource 直連動詞契約端點，設定頁配置 server URL/repo（入 .speclink.yaml）與 PAT（使用者層級憑證），使 desktop 成為遠端 server 的 GUI。

<!-- 上列兩能力是否同刀交付或拆刀（server 先、desktop 遠端模式後）為 design 階段定案項 -->

## Impact

- 新增: Node headless web server（含 TypeScript PostgreSQL Store）；docker-compose 部署清單。
- 修改: desktop app——新增 httpDataSource（SpeclinkDataSource 實作）與設定頁遠端模式配置（.speclink.yaml server URL/repo、PAT 使用者層級憑證）。
- 修改: 產品文件——docs/team-mode.md 與 README 明載本機個人模式適用/限制與轉遠端時機。
- 復用: ① 的 React 元件庫（寄宿於 desktop 遠端模式，非瀏覽器）。
- 消費既有正典: verb-contract、remote-connection、remote-auth（皆已歸檔）——本刀是其第一個真實 server 端。
- 外部依賴: PostgreSQL 實例（docker-compose 打包）。
- Roadmap 連動: ⑤ web-role-views 取消（desktop 遠端模式取代 PO 瀏覽器零安裝）；② desktop-acp-agent 前提修正另行 ingest；④ web-agent-channel（Copilot SDK 作為後端情境）繼續有效。
- 驗證紅利: CLI remote 模式首次有真 server 可對打（取代假伺服器測試）。

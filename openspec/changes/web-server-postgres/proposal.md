> **Roadmap**: 四情境預設 GUI 工具矩陣的第 ③ 刀（共 5，序 4→3→2→1）。來源討論 `四情境預設-gui-工具矩陣`。
> **依賴**: ① desktop-shell-and-browser（復用其 React 元件庫）；概念上依賴 verb-contract／remote-connection 正典（已歸檔）。**下游**: ④ web-agent-channel、⑤ web-role-views 皆疊在本刀的 web server 之上。
> **狀態**: 待完整 propose（本檔為 promote 骨架）。
> **現況更新（2026-07-05，① 完成後）**: 可復用的 packages/ui 已含 shadcn/Tailwind 設計系統（teal 主色）與完整元件集（KanbanBoard 生命週期看板、RichDetailDrawer 詳情抽屜、TaskList 互動任務、ArchivedList 封存頁、Markdown 富文本）；資料源介面 SpeclinkDataSource 除唯讀外含寫入方法（setTaskDone/moveTask/deleteChange）——本刀的 HTTP adapter 需將其對應到動詞契約端點，或對 desktop 專屬操作優雅缺席（propose 時定案）。
> **現況更新（2026-07-06，desktop-board-parity／desktop-discussion-board 提案後）**: (1) in-progress 標記真相遷入 change meta（started_at/started_by/started_with）——標記騎在文件真相上，Postgres store 同步文件即同步狀態，**無需另設狀態表**；看板欄位派生規則（全完成＝已就緒＞有 started＝進行中＞其餘提案中）已定於 desktop-app spec。(2) Store trait 新增封存 artifact 讀取與封存 capability 列舉（帶預設實作）、active meta 原文讀寫對、discussions 既有方法——Postgres store 實作這些即得封存瀏覽與討論看板。(3) 桌面的即時刷新信號源是檔案監看（宿主層 wiring、不在 SpeclinkDataSource）——本刀的等價物是 server push（Postgres LISTEN/NOTIFY → SSE/WebSocket），propose 時定案傳輸形式。(4) SpeclinkDataSource 已再擴：封存文件讀取、封存 capability 列舉、討論清單/記錄/促轉/歸檔——HTTP adapter 需對應端點。(5) 狀態轉換分層定案：引擎（server 內嵌，經 createEngine）控轉換語意、store 控持久化、端點控授權與併發——多人下多步驟流程（如促轉＝建 change＋預填＋標記）的原子性與衝突語意為本刀 propose 時的設計題（Postgres 交易包覆 vs 端點層衝突回報；verb-contract 的 If-Match/409 reason 為既有雛形）。(6) 認領與開工的綁定為本刀 propose 時的定案項：verb-contract 既有 claim/ownership（可搶佔轉移、409 ownership_lost）與 change meta 的 started_*（冪等首次開工里程碑）是相鄰但不同的狀態——建議首次 claim 成功即蓋 started_*、ownership 轉移不改寫 started_*、看板「進行中」兩模式一致以 started_* 派生、目前持有人為 remote 加值顯示。(7) verb-contract 的 store 文件讀取動詞現僅涵蓋 active change（artifact cat）——封存文件讀取端點為本刀對 verb-contract 的 delta 新增項。

## Why

情境 3（本地 CLI＋遠端文件）目前只能對 tiny_http 假伺服器測試，沒有真正的 server 可連。本刀交付一個 Node web 應用當團隊 server：經 `createEngine` 內嵌引擎、以 PostgreSQL 為 store 真相、對外暴露動詞契約 REST 端點（本地 CLI remote 模式 `link` 上來）、並復用第 ① 刀的 React 元件庫呈現 web GUI。這同時是情境 1/2 的共同底座——後兩刀只需在此之上疊 agent 通道與角色切面。

儲存用 PostgreSQL（team server 多人並發、中央治理），以 TypeScript 實作 Store（Node SDK 的 store bridge 已支援 JS 實作 Store），非另開 Rust adapter。

## What Changes

- 新增 Node web 應用：經 `@speclink/engine` 的 `createEngine` 內嵌引擎。
- TypeScript PostgreSQL Store：以 pg 驅動實作 Store 介面（change/artifact/discussion/spec/WorkflowConfig），作為 store 真相。
- 動詞契約 REST 端點：對齊 verb-contract 正典，供 CLI remote 薄 client 呼叫（含 PAT 認證、repo 歸屬驗證）。
- web GUI：復用 ① 的 React 元件庫（看板/文件樹/spec 瀏覽），資料源改為 server。

<!-- 細節（PostgreSQL schema、REST 端點對應、認證落地、部署形態）待 /speclink-propose 於 design 階段定案 -->

## Capabilities

### New Capabilities

- `web-server`: Node web 應用——內嵌引擎、PostgreSQL Store、動詞契約 REST 端點、復用 React 元件的 web GUI，交付情境 3。

## Impact

- 新增: Node web 應用（含 TypeScript PostgreSQL Store）。
- 復用: ① 的 React 元件庫。
- 消費既有正典: verb-contract、remote-connection、remote-auth（皆已歸檔）——本刀是其第一個真實 server 端。
- 外部依賴: PostgreSQL 實例。
- 驗證紅利: CLI remote 模式首次有真 server 可對打（取代假伺服器測試）。

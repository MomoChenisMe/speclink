> **Roadmap**: 四情境預設 GUI 工具矩陣的第 ③ 刀（共 5，序 4→3→2→1）。來源討論 `四情境預設-gui-工具矩陣`。
> **依賴**: ① desktop-shell-and-browser（復用其 React 元件庫）；概念上依賴 verb-contract／remote-connection 正典（已歸檔）。**下游**: ④ web-agent-channel、⑤ web-role-views 皆疊在本刀的 web server 之上。
> **狀態**: 待完整 propose（本檔為 promote 骨架）。
> **現況更新（2026-07-05，① 完成後）**: 可復用的 packages/ui 已含 shadcn/Tailwind 設計系統（teal 主色）與完整元件集（KanbanBoard 生命週期看板、RichDetailDrawer 詳情抽屜、TaskList 互動任務、ArchivedList 封存頁、Markdown 富文本）；資料源介面 SpeclinkDataSource 除唯讀外含寫入方法（setTaskDone/moveTask/deleteChange）——本刀的 HTTP adapter 需將其對應到動詞契約端點，或對 desktop 專屬操作優雅缺席（propose 時定案）。

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

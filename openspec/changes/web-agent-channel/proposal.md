> **Roadmap**: 四情境預設 GUI 工具矩陣的第 ④ 刀（共 5，序 4→3→2→1）。來源討論 `四情境預設-gui-工具矩陣`。
> **依賴**: ③ web-server-postgres（在其 web 應用內加 agent 通道）；② desktop-acp-agent 的 agent 選型/接線經驗。**下游**: 無（⑤ 與本刀並列、皆疊在 ③ 之上）。
> **狀態**: 待完整 propose（本檔為 promote 骨架）。

## Why

情境 2（所有角色都在 Agent 系統中執行完整 SDD）需要 web 應用內建對話式 agent，讓使用者在瀏覽器內就能跑 discuss/propose/apply/ingest/archive 全流程。本刀在第 ③ 刀的 web 應用內嵌 Copilot SDK agent 通道（`defineTool("speclink")` → dispatch，wadpilot 已生產驗證此模式），交付情境 2。

## What Changes

- web 應用內嵌 Copilot SDK agent：`createSession({ tools })` ＋ `defineTool("speclink", { handler → engine.dispatch })`。
- 經 render API 注入指令（systemMessage）與 skills（skillDirectories），對齊「內容送達三分」。
- 瀏覽器內對話式 SDD 介面（訊息串、工具呼叫、權限）。

<!-- 細節（Copilot SDK 版本、tool 定義、session 管理、skills 部署）待 /speclink-propose 於 design 階段定案 -->

## Capabilities

### New Capabilities

- `web-agent`: web 應用內嵌 Copilot SDK agent 通道，瀏覽器內完整對話式 SDD，交付情境 2。

## Impact

- 新增: web 應用（③ 交付）內的 Copilot SDK agent 通道與對話介面。
- 消費既有 API: Node SDK 的 dispatch／skills.render／instructions.render。
- 不影響 CLI、桌面與引擎行為。

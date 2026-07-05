> **Roadmap**: 四情境預設 GUI 工具矩陣的第 ⑤ 刀（共 5，序 4→3→2→1）。來源討論 `四情境預設-gui-工具矩陣`。
> **依賴**: ③ web-server-postgres（在其 web 應用上加角色切面）；④ web-agent-channel（PO 端 agent 操作）。**下游**: 無（本刀收尾情境 1）。
> **狀態**: 待完整 propose（本檔為 promote 骨架）。

## Why

情境 1（PO/PM 在系統執行 discuss/propose/ingest/archive、RD 在本地用 Claude Code 執行 apply/drift/verify）是情境 2 與情境 3 的混合切面：同一個 web 應用，PO/PM 走 web GUI＋agent，RD 走本地 remote 模式 CLI。本刀在 web 應用加角色視圖與交接動線，讓兩側各取所需、change↔repo 歸屬清楚，交付情境 1。

## What Changes

- web 應用角色切面：PO/PM 視圖（change 看板、discuss/propose/ingest/archive、指派）與其權限。
- PO web 端與 RD 本地 CLI 的交接動線：change 歸屬、狀態流轉在 web 端可見。
- （若需要）通知/指派的最小呈現。

<!-- 細節（角色/權限模型、交接 UX、通知範圍）待 /speclink-propose 於 design 階段定案 -->

## Capabilities

### New Capabilities

- `web-roles`: web 應用角色切面與 PO↔RD 交接動線，交付情境 1。

## Impact

- 新增: web 應用（③④ 交付）的角色視圖、權限與交接動線。
- 消費既有正典: verb-contract 的 repo 歸屬與狀態語意。
- 不影響 CLI、桌面與引擎行為。

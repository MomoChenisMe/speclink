## Why

團隊系統（第一個消費者為 wadpilot，Node.js/Fastify）需要在 server 端內嵌 speclink 引擎來實作動詞契約的 server 側：以自家資料庫作為儲存、讓 AI agent（如 Copilot SDK 的自訂 tool）in-process 呼叫引擎。若以 TypeScript 重刻引擎會造成雙引擎漂移（討論第 4 輪否決）。本 change 交付 Node SDK：napi-rs 綁定的 @speclink/engine 套件——統一入口 dispatch(argv)、宿主自實作 Store 的橋接、以及流程知識的渲染 API。Rust SDK 即 speclink-core crate 本身，不需額外交付。

目標使用者：以 Node.js 建置團隊系統／AI Agent 系統的平台開發者（情境 1 與情境 2 的 server 端）。

## What Changes

- **新增 napi-rs 綁定 crate**：發佈為 npm 套件 @speclink/engine，隨附 TypeScript 型別定義與各平台預編譯二進位。
- **createEngine 建構**：接受內建 fs 儲存（指定專案根）或宿主以 JavaScript 實作的 Store 物件（方法可回傳 Promise）；引擎核心維持同步、無 async runtime——async 橋接發生在綁定層，且 dispatch 一律以背景工作執行避免阻塞 JS 事件迴圈。
- **dispatch(argv) 統一入口**：與 CLI 動詞詞彙一對一；回傳與既有 `--json` 對齊的結構化物件（camelCase）；失敗以帶語義化訊息的例外拋出。宿主據此以一個名為 speclink 的 tool（參數為 argv 陣列）接上任何 agent 框架。
- **渲染 API**：instructions.render 與 skills.list／skills.render，參數涵蓋渲染矩陣（目標＝內建工具或中性、措辭＝cli 或 tool-call、store 模式＝fs 或 remote）——宿主取得字串後自行注入 system prompt 或 skillDirectories。
- 新增 SDK 整合雙語文件與教學（含 Copilot SDK defineTool 完整範例、Store 介面實作指南、部署注意事項），README 增列連結。

## Non-Goals

（範圍排除與被否決方案記錄於 design.md 的 Goals / Non-Goals 章節。）

## Capabilities

### New Capabilities

- `node-sdk`: createEngine（fs 內建與宿主 Store 雙形式）、dispatch(argv) 的輸入輸出契約與錯誤傳遞、JS Store 橋接語意、渲染 API 的參數與輸出。

### Modified Capabilities

（無——SDK 是新增消費介面，CLI 既有行為不變。）

## Impact

- Affected specs: 新增 `node-sdk`
- Affected crates: speclink-node（新 crate，napi-rs 綁定）、speclink-core 與 speclink-fs（僅消費其既有公開介面，必要時補 pub 可見度，不改行為）
- 相容性影響: CLI 無任何行為變更；新增 npm 發佈面（@speclink/engine），其 API 自本版起為對外契約
- 設定欄位: （無新增設定檔欄位——SDK 以建構參數取代設定檔）
- 技能/marker 影響: 渲染 API 公開既有渲染矩陣，生成內容與 CLI 生成一致（同一渲染程式碼）
- Affected code:
  - New: `crates/speclink-node/Cargo.toml`、`crates/speclink-node/src/lib.rs`、`crates/speclink-node/src/store_bridge.rs`、`crates/speclink-node/src/render.rs`、`crates/speclink-node/package.json`、`crates/speclink-node/index.d.ts`、`crates/speclink-node/__test__/engine.spec.ts`、`docs/sdk-node.md`、`docs/sdk-node.zh-TW.md`
  - Modified: `Cargo.toml`（workspace members）、`crates/speclink-core/src/lib.rs`（必要的 pub 可見度）、`README.md`、`.github/workflows/`（新增各平台預編譯管線）
  - Removed: （無）

## Why

情境 3 的原第 ③ 刀重切為三後，本刀（①）是可複用核心。使用者要「先包出 SDK 功能、拆出儲存邏輯」作為可獨立交付、可整合自家系統的頭牌——headless server（② web-server-postgres）只是其上的開箱即用範例，不是每個使用者都直接用，可能整合自家系統＋SDK＋自家儲存。

SDK（`@speclink/engine`）與 Store 縫（`speclink-core` 的 `Store` trait ＋ JS bridge）已存在但未完備：dispatch 只路由 4 動詞、analyze/validate/drift 無 server 端運算路徑、Store 縫未文件化為公開整合面、推播通道無宣告約定。本刀補完這些，讓任何整合者以 SDK ＋自實作 Store 建自家 server；② 與 ③ 增量疊加。

同時落地兩項正典級決策（來源討論 sdk-storage-seam-and-remote-desktop、server-auth-and-push-transport）：遠端模式下 analyze/validate/drift 改 **server 端運算**（team 一致性——client 端算會因各人引擎版本歧異使全隊分析結果分裂，team server 應釘住分析語意）；遠端 agent 一律經 CLI 動詞讀「文件」而非本地檔。

## What Changes

- `@speclink/engine` 的 dispatch 補完**可遠端託管的完整動詞集**（archive、task done、artifact cat、instructions、language、config、spec 讀取、discuss 全套），複用 `speclink-core` 既有邏輯、不改語意。
- 新增 **analyze/validate/drift 的 server 端運算路徑**：引擎能對 host store 的文件算出報告並經 dispatch 暴露；verb-contract 新增對應端點；CLI remote 模式改路由 analyze/validate/drift 至端點（修訂原「client-side」決策）。
- verb-contract 新增**可選、傳輸無關的推播通道宣告欄** `events:{url,transport}`（掛 whoami/config metadata；明標 push 在請求/回應契約外、僅供發現）；引擎本體零推播機制。
- **遠端模式 agent 文件存取**：確立遠端技能與 marker 導引 agent 一律經 `artifact cat`／`language show`／`discuss show`／`show` 讀文件（涵蓋補完後的動詞集），禁讀本地路徑。
- **Store 縫文件化為公開整合面**：文件說明 host 如何實作 `Store` ＋以 `createEngine` 建自家 server（「拆出儲存邏輯」的可複用交付）。

## Non-Goals

- 不含 PostgreSQL Store 實作與 demo server（② web-server-postgres 範圍）。
- 不含 desktop 遠端模式（③ desktop-remote-mode 範圍）。
- 引擎**不實作任何推播傳輸**（SSE／WebSocket 屬 server／client 層，非引擎；本刀只加傳輸無關的宣告欄）。
- 不實作任何 client 的推播消費（③ 範圍）。
- 不改動 analyze/validate/drift/discuss 各動詞的**業務語意**——僅路由暴露與運算位置。

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `node-sdk`: dispatch 補完遠端託管動詞集；新增 analyze/validate/drift 的 server 端運算路徑；Store 公開整合面文件化。
- `verb-contract`: 遠端 analyze/validate/drift 改 server 端運算（新增端點、CLI remote 路由至之）；新增可選推播通道宣告欄；遠端 agent 經動詞讀文件涵蓋補完後的動詞集。

## Impact

- Affected specs: node-sdk、verb-contract
- Affected code:
  - Modified: crates/speclink-node/src/lib.rs（dispatch router 擴充＋analyze/validate/drift 運算路由）、crates/speclink-node/index.d.ts（型別補充）、crates/speclink-remote/src/client.rs（analyze/validate/drift 端點 client 方法）、crates/speclink-cli/src/remote_commands.rs（遠端 analyze/validate/drift 路由至端點）、docs/verb-contract.md（server 端運算端點＋推播宣告欄＋§6 涵蓋圖修訂）、docs/sdk-node.md（Store 整合面說明）
  - New: docs/integration.md（host 實作 Store ＋以 createEngine 建自家 server 的整合指南）
- 消費既有正典: store-abstraction（Store 縫既成資產，本刀文件化為公開整合面）
- 下游連動: ② web-server-postgres 消費本刀補完的 SDK ＋契約端點；③ desktop-remote-mode 消費遠端 server 端運算與推播宣告欄

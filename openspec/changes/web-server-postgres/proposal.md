> **Roadmap**: 四情境預設 GUI 工具矩陣原第 ③ 刀，2026-07-10 重切為三（討論 sdk-storage-seam-and-remote-desktop ＋ server-auth-and-push-transport）：① speclink-sdk-and-store-seam（可複用核心）、**② web-server-postgres（本刀＝pg 參考 Store ＋開箱即用範例 server）**、③ desktop-remote-mode（消費者）。
> **依賴**: ① speclink-sdk-and-store-seam（消費其補完的 dispatch 全動詞、analyze/validate/drift server 運算、推播宣告欄）；消費既有正典 verb-contract／remote-connection／remote-auth（已歸檔）。**下游**: ③ desktop-remote-mode 連本 server；④ web-agent-channel 疊其上。

## Why

情境 3（本地 CLI ＋遠端文件）迄今只能對假伺服器測試，沒有真正的 server 可連。本刀交付 **pg 參考 Store ＋開箱即用的自架團隊 server**：以 ① 補完的 `@speclink/engine` 經 `createEngine` 內嵌引擎、以 TypeScript 實作 PostgreSQL Store（乾淨可複用的參考實作）、對外暴露動詞契約 REST 端點（含 ① 的 analyze/validate/drift server 端運算端點）。

本刀是「拆出儲存邏輯」的落地範例：pg Store 示範 host 如何實作 SDK 的 `Store` 縫；薄 server 示範如何以 `createEngine` ＋自實作 Store 組出團隊 server。不是每個使用者都直接用本 server——可能整合自家系統 ＋SDK ＋自家儲存；本 server 是簡易方案兼使用範例。但因「**開箱即用**」，其認證認真規劃、可直接給小團隊用，非丟棄式 demo。

儲存用 PostgreSQL（多人並發、中央治理）；認證 ＋admin 管理內建；即時刷新以 SSE ＋Postgres LISTEN/NOTIFY 實作，並經 ① 的契約宣告欄對外宣告 `transport:"sse"`（客戶端輪詢地基 ＋可選推播、永不鎖死任何 server）。

## What Changes

- **TypeScript PostgreSQL Store（參考實作）**：以 pg 實作 ① 的 SDK `Store` 介面（changes／artifacts／canonical specs／discussions／archive／workflow config／language ＋ meta 讀寫對）；文件即真相、artifacts／canonical_specs 帶 `version` 欄承載樂觀並行。
- **薄 headless server**：`createEngine({ store: pgStore })` 內嵌 ① 的引擎，暴露 `docs/verb-contract.md` v1 端點（含 ① 的 analyze/validate/drift server 運算端點），實際文件變動委派 dispatch；HTTP ＋認證 ＋並發 guard 為薄層。
- **樂觀並行與生命週期裁決**：version／If-Match、claim 原子 compare-and-set、archive check-all-then-apply、ownership，以 pg 交易實作、對齊 `docs/verb-contract.md` §4 reason 目錄。
- **認證與 admin 管理（開箱堪用、非丟棄式）**：Bearer PAT 驗證 →whoami；admin REST 端點管理 token（建／列／撤）與 repos 註冊；首個 admin token 由環境變數（`SPECLINK_ADMIN_TOKEN`）啟動種入。
- **SSE ＋LISTEN/NOTIFY**：文件變動經 pg `NOTIFY` → server `LISTEN` → SSE 廣播 invalidate 事件；並經 whoami／config 的 `events` 宣告欄對外宣告 `transport:"sse"`。
- **docker-compose**：一鍵起 server ＋PostgreSQL、首次啟動 migration。
- **定位文件化**：`docs/team-mode.md` 與 `README` 明載檔案模式適用／限制、防繞過強保證唯遠端可達、何時轉遠端。

## Non-Goals

- 不含 SDK dispatch 擴充、analyze/validate/drift 運算路徑、verb-contract 修訂、推播宣告欄定義（① speclink-sdk-and-store-seam 範圍）——本刀**消費**之。
- 不含 desktop 遠端模式（③ desktop-remote-mode 範圍）。
- 不做瀏覽器 web GUI（遞延）、agent 通道（④ web-agent-channel）。
- 引擎不實作推播傳輸——SSE 是本 server 的實作與宣告，非引擎。
- 不含任何 client 端的推播消費（③ 範圍）。

## Capabilities

### New Capabilities

- `web-server`: 開箱即用的 headless 團隊 server——pg 參考 Store ＋內嵌 ① 的引擎 ＋動詞契約 REST 端點（含 server 端運算）＋認證/admin ＋SSE 推播 ＋docker-compose。

### Modified Capabilities

(none)

## Impact

- Affected specs: web-server（新）
- Affected code:
  - New: apps/server/（Node headless server ＋ TypeScript PostgreSQL Store，消費 @speclink/engine）、docker-compose.yml、資料庫 migration（於 apps/server 內）
  - Modified: docs/team-mode.md（定位更新）、README.md（定位）
- 依賴: ① speclink-sdk-and-store-seam（補完的 SDK dispatch 全動詞、analyze/validate/drift server 端運算端點、推播宣告欄）
- 消費既有正典: verb-contract、remote-connection、remote-auth（皆已歸檔）
- 外部依賴: PostgreSQL（docker-compose 打包）
- 下游連動: ③ desktop-remote-mode 連本 server；④ web-agent-channel 疊其上

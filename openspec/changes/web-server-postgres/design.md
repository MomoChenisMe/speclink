## Context

三刀重切後本刀（②）交付 pg 參考 Store ＋開箱即用的範例 server，**消費 ①（speclink-sdk-and-store-seam）補完的 SDK**（dispatch 全動詞、analyze/validate/drift server 端運算、推播宣告欄）。

現況（決定取捨的既有事實）：
- **① 提供補完的引擎**：`@speclink/engine` 的 dispatch 已涵蓋完整動詞集 ＋ analyze/validate/drift server 端運算；本刀以 `createEngine({ store })` 內嵌之。
- **Store 縫既成且 ① 文件化**：本刀以 TypeScript 實作 pg Store 為其參考實作。
- **契約正典**：`docs/verb-contract.md` v1（① 已加 analyze/validate/drift 端點與 `events` 宣告欄）；本刀 server 應答之。
- **CLI remote client 既有**：`crates/speclink-remote` 已覆蓋端點；本刀是其第一個真實 server 端。
- **無既有 Node HTTP 框架**：server 為 greenfield。
- **推播分層既定**（server-auth-and-push-transport 討論）：client 地基＝輪詢；推播為可選、可宣告、傳輸無關；本 server 實作 SSE 並宣告 `transport:"sse"`。

## Goals / Non-Goals

**Goals:**
- TypeScript PostgreSQL Store（乾淨可複用參考實作）實作 ① 的 `Store` 介面。
- 薄 headless server：`createEngine({ store: pgStore })` 內嵌 ① 引擎，暴露 `docs/verb-contract.md` v1 端點（含 server 運算端點），委派 dispatch。
- 樂觀並行與生命週期裁決：version/If-Match、claim CAS、archive check-all-then-apply、ownership，以 pg 交易對齊 reason 目錄。
- 開箱堪用的認證與 admin 管理（PAT ＋admin REST 端點 ＋env bootstrap）。
- SSE ＋LISTEN/NOTIFY，並經 `events` 宣告欄宣告 `transport:"sse"`。
- docker-compose ＋定位文件。

**Non-Goals:**
- 不含 SDK dispatch 擴充、契約修訂、推播宣告欄定義（①）；不含 desktop 遠端模式（③）。
- 引擎不實作推播傳輸（SSE 是本 server 的實作）；不含 client 推播消費（③）。

## Decisions

### D1: 消費 ① 的 SDK，server 為薄層

server 不重寫流程邏輯：`createEngine({ store: pgStore })` 內嵌 ① 補完的引擎，各端點的實際文件變動與運算（含 analyze/validate/drift）**委派 `engine.dispatch`**。server 只做 HTTP 傳輸/路由、認證（PAT→whoami）、repos 歸屬驗證、並行/生命週期 guard 層。

替代方案：server 以 TS 自實作各動詞裁決。**駁回**——複製 ① 的 core 邏輯、必然版本漂移，違「canon 在 core 單一真相」。

### D2: Fastify 框架

server greenfield。選 Fastify：成熟、路由/hooks 便於掛認證與版本協商、原生支援 SSE 所需 raw stream 回應。

替代方案：`node:http` 原生（樣板過多）、Express（較舊）、Hono（更輕但 Fastify 生態更穩）。apply 時若有強烈精簡偏好可換 Hono，介面契約不變。

### D3: pg 參考 Store 文件即真相與 version 樂觀並行

表結構鏡射文件模型、不另設狀態表（lifecycle 由文件派生）：`changes`（name, repo, meta_text, created_at, updated_at）、`artifacts`（change, artifact_path, content, **version**）、`canonical_specs`（capability, content, **version**）、`discussions`、`archived_changes`、`workflow_config`、`language`；治理表 `repos`、`tokens`。pg Store 的 `writeArtifact` 每次遞增 `version`；server 直接以 pg 連線（在 Store 介面外）查/比對 version 做 If-Match，並以條件式 `UPDATE ... WHERE version = $ifMatch` 於交易內做原子 compare-and-set。

替代方案：擴 Store 介面加 version 方法。**駁回**——引擎/fs 後端不需 version，為單一遠端後端污染共用介面違背 store.rs 原則。

### D4: 認證與 admin 管理開箱堪用

`tokens` 表存 PAT→身分（id/name/handle）與 repo 關聯；PAT 驗證產出 whoami，401 分 `token_missing`/`token_invalid`/`token_expired`/`token_revoked`。契約外 admin REST 端點（獨立路徑、admin 權限）管理 token 建/列/撤與 repos 註冊/列。首個 admin token 由 `SPECLINK_ADMIN_TOKEN` 啟動種入。定位為 batteries-included 堪用預設（開箱即用），整合者帶自家系統時換掉整個 server 層、自帶認證。

替代方案：丟棄式靜態 demo token（**駁回**——本 server 開箱即用，認證須認真規劃）；契約內 token 發放（**駁回**——`docs/verb-contract.md` §7 明列 PAT 發放不在契約）。

### D5: SSE server 加 LISTEN/NOTIFY 並宣告 transport

server 提供已認證、repo-scoped 的 SSE 端點；文件變動經 pg `NOTIFY` → server `LISTEN` → 向該 scope 廣播 `{ type:"invalidate", scope }`（不攜文件內容）；並於 whoami/config 的 `events` 宣告欄宣告 `{ url, transport:"sse" }`（① 定義的可選傳輸無關約定）。client 靠宣告發現、不支援則退回輪詢——本 server 不假設任何特定 client。

替代方案：不宣告、寫死路徑（**駁回**——違 ① 的傳輸無關發現約定，鎖死其他 client）；WebSocket（**駁回**——需求單向、SSE 更契合，見 server-auth-and-push-transport 討論）。

### D6: 並發與生命週期 guard 由 server pg 交易裁決

version/If-Match、claim 原子 CAS、archive check-all-then-apply（tasks_incomplete/version_conflict conflicts[]/gate_pending）、applying owner-only 寫、archived 寫拒（change_busy），皆由 server 於 pg 交易內裁決、翻譯為 `docs/verb-contract.md` §4 reason；guard 通過後委派 dispatch 做實際變動。

替代方案：把裁決塞進引擎。**駁回**——version/ownership 是 pg 交易概念、server 獨佔 pg 連線，裁決放 server 最貼合（① 的引擎保持 store-agnostic）。

## Implementation Contract

#### web-server（新 capability）

- **可觀察行為**：一個可執行的 headless Node 程序，監聽 HTTP、以 PostgreSQL 為真相、應答 `docs/verb-contract.md` v1 全部端點（含 ① 的 analyze/validate/drift server 運算端點）。CLI remote `link` 上來後 fs 模式的人眼與 `--json` 輸出於 remote 逐位元一致。
- **必備標頭**：`Authorization: Bearer <PAT>`、`X-Speclink-Api-Version: 1`（缺/不支援→`400 api_version_unsupported`）、`X-Speclink-Repo`（多 repo 必備，缺→`400 repo_required`）。
- **樂觀並行**：artifact 讀回傳 `version`（自 1 遞增）；PUT 必帶 `If-Match`（`0`=create-only、陳舊→`409 version_conflict`、缺→`428 if_match_required`）。
- **生命週期**：claim 原子 CAS（已認領→`409 ownership_lost`）；applying owner-only 寫；archive check-all-then-apply（`409 tasks_incomplete`／`version_conflict` conflicts[]／`gate_pending`）；archived 寫→`409 change_busy`（lifecycle:archived）。
- **錯誤信封**：非 2xx 帶 `{ reason, message, ...context }`，409 恆帶 reason，對齊 §4 目錄。
- **repo 歸屬**：一 change 屬一 repo；list 依請求 repo 過濾；跨 repo→`403 repo_mismatch`。
- **認證/admin**：PAT→whoami（401 四 reason）；admin REST 端點管 token/repos；`SPECLINK_ADMIN_TOKEN` bootstrap。
- **SSE/宣告**：已認證 repo-scoped SSE 端點廣播 invalidate 事件；whoami/config 帶 `events:{url,transport:"sse"}`。
- **部署/文件**：docker-compose 一鍵起 server＋pg＋migration；`docs/team-mode.md`＋`README` 明載模式對照。
- **驗收**：(a) CLI remote 既有劇本（`crates/speclink-cli/tests/remote_*.rs`）對真 server 通過；(b) reason 目錄一致性測試套（鏡射 §4）；(c) If-Match/claim 並發測試（恰一成功）；(d) `docker-compose up` 後 link＋auth login＋完整 propose→apply→archive；(e) SSE 事件於文件變動後送達訂閱者、whoami 帶宣告欄。
- **In scope**：pg Store、薄 server、並發/生命週期 guard、認證/admin、SSE＋宣告、docker、定位文件。**Out**：SDK/契約修訂（①）、desktop（③）、瀏覽器 GUI、agent 通道、推播 client 消費。

## Risks / Trade-offs

- **[並發裁決在 server、業務變動委派引擎，易語意漂移]** → 以鏡射 §4 的 reason 一致性測試套為紅線；guard 集中單一模組。
- **[PostgreSQL 新外部依賴，TDD 需可測 pg]** → docker-compose 帶 migration；測試採 ephemeral pg（testcontainers 或 CI pg，apply 定案）。
- **[CLI 遠端輸出須與 fs 逐位元一致]** → 複用 `crates/speclink-cli/tests/remote_*.rs` 對真 server 跑。
- **[依賴 ① 未先完成則 server 無完整引擎可用]** → 依賴序 ①→②；② 測試可先以 ① 的補完 SDK（或其 mock）對打。

## Migration Plan

- 部署：`docker-compose up` 起 server＋pg（migration）；`SPECLINK_ADMIN_TOKEN` 種入 admin token；admin 端點註冊 repo、發 PAT。
- 接入：CLI `speclink init --store remote --url <url> --repo <name>` ＋ `speclink auth login`。
- 回滾：server 與 pg 資料獨立於 client repo；停 server 即回退（client 端移除 remote 設定回 fs）。

## Open Questions

- pg 測試載具選型（testcontainers vs CI 服務 pg）——apply 定案。
- SSE 失效事件粒度（whole-scope vs `change:<name>`）——apply 依體感定案。
- server 套件落點（apps/server vs packages/server）——apply 定案（本設計以 apps/server 為錨）。

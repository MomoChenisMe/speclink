## Context

審查／驗證工單是引擎擁有的 append-only 文件（`review.md`／`verify.md`），蓋章或放棄時刪除，封存時可用 `--carry-review`／`--carry-verify` 帶走。桌面現況：

- 清單資料已由「工單檔存在」算出四種狀態（`apps/desktop/core/src/query.rs` 的 `list_changes` 疊上 `reviewStatus`／`verifyStatus`；封存清單在 `cache.rs` 疊上 `reviewedNotPassed`／`verifiedNotPassed`），檔案監看為遞迴，工單出現或消失時看板即刷新。
- 變更詳情面板（`packages/ui/src/components/RichDetailDrawer.tsx`）有五個分頁：提案、設計、任務、規格、排程；文件走 `loadDocument(change, artifact)`，三態（載入中 skeleton／無檔空態／有內容）已是正典。已封存面板（`ArchivedDrawer.tsx`）四個分頁、同一套載入器。
- 引擎已把工單解析成 `Ticket { rounds: [Round { index, phase, patch_hash, scope, findings: [Finding { severity, path, text }] }] }`（`crates/engine/speclink-core/src/quality/station.rs`），`parse_ticket(station, text)` 是私有函式；CLI `speclink review show --json` 與 server `GET /changes/{name}/review`／`verify` 都輸出同構的 `rounds`（`ReviewRoundDto`）。
- 遠端讀活變更文件走 `GET /changes/{name}/artifacts/{artifact}`，引擎 `artifact cat` 的名稱白名單只放行 proposal／design／tasks／specs/<cap>，工單名會被拒；遠端讀封存文件走 `GET /archived/{dated}/artifacts/{*artifact}`，無白名單。`speclink-remote` client 已有 `station_ticket_if_any(station, name)`（404 → `None`）。

來源討論 drawer-quality-ticket-tab 已定案：兩個條件式分頁（審查／驗證）而非單一分頁或左右對照；第一版即結構化渲染；已封存側納入；階段文案「首輪」「複驗」。

## Goals / Non-Goals

**Goals:**

- 工單存在時，變更詳情面板與已封存面板各出現「審查」「驗證」分頁，內容為結構化工單，工單消失時分頁消失。
- 本機（含 worktree 覆蓋層）與遠端、活變更與已封存，四條路回同一份 JSON 形狀，前端只有一個工單元件。
- 引擎行為不變：不改 CLI 輸出、不改 wire、不放寬 `artifact cat` 白名單。

**Non-Goals:**

- 點狀態列章籤直接跳到對應分頁（延後）。
- 蓋章後的工單歷史回看（工單已刪；fs 只在 git 歷史、remote 不保留）。
- 在分頁內執行動詞（蓋章、放棄、追加輪）——分頁唯讀；動詞仍由技能與封存對話框承載。
- 單一「品質關卡」分頁、左右對照輪次、分頁內站別切換鈕、markdown 原文渲染（討論已排除）。
- `apps/server-web`：它不使用 `RichDetailDrawer`，本變更不動。

## Decisions

### D1 工單 JSON 形狀鏡射 CLI `review show --json` 的 `rounds`

桌面資料來源回傳 `StationTicket | null`：

```
{ "rounds": [ { "index": 1, "phase": "discovery" | "validation" | null,
                "patchHash": "sha256:…" | null, "scope": ["src/a.rs"],
                "findings": [ { "severity": "CRITICAL"|"WARNING"|"SUGGESTION",
                                "path": "src/a.rs", "text": "Correctness: …" } ] } ] }
```

欄位名與型別與 `ReviewRoundDto`（protocol）及 CLI `--json` 的 `rounds` 一字不差，camelCase。不帶 `lastRound`（前端取陣列末項）、不帶 `content`（不渲染原文）。理由：wire 上已有這個形狀，遠端活變更可零轉換直接傳回；本機路徑用同形狀就不需要第二套型別。替代方案「回原文字串、前端自己解析」被排除：解析規則（Phase／Patch 成對、legacy 輪）已在引擎，前端重寫會分岔。

### D2 解析只在引擎，desktop core 做序列化，四條路徑共用

- `speclink-core`：`quality::station::parse_ticket(st: &Station, text: &str) -> Result<Ticket>` 由私有升為 `pub`。純函式、無 I/O、無 ANSI。既有 `show` 改為呼叫它（實際上已是），行為不變。
- `speclink-desktop-core`（`apps/desktop/core/src/query.rs`）新增：
  - `station_ticket_at(root, change, station: &str) -> Option<Value>`：`station` 為 `"review"`／`"verify"`，其他值回 `None`；經 `context_for_change`（worktree 覆蓋層）取 store，`store.read_artifact(change, st.doc)` 無檔回 `None`，有檔以 `parse_ticket` 解析後轉 D1 形狀；解析失敗（格式壞掉）回 `None`——面板顯示空態而非錯誤，與其他文件分頁「缺件即空態」同款。
  - `archived_station_ticket_at(root, dated_name, station) -> Option<Value>`：同法，改讀 `store.read_archived_artifact(dated_name, st.doc)`。
  - `ticket_json(&Ticket) -> Value`：D1 形狀的唯一序列化落點；`ticket_json_from_text(station, text) -> Option<Value>` 供遠端封存原文使用。
- `apps/desktop/src-tauri/src/lib.rs` 四個 Tauri 指令，每個單行委派：
  - `station_ticket(root, change, station)` → `query::station_ticket_at`
  - `archived_station_ticket(root, dated_name, station)` → `query::archived_station_ticket_at`
  - `remote_station_ticket(connection_id, project, repo, change, station)` → `with_remote` 內呼叫 `RemoteWorkspace` 新增的 `station_ticket(credentials, station, change)`（包 client 的 `station_ticket_if_any`），`Some(resp)` 時回 `{ "rounds": resp.rounds }`（serde 直接序列化 `ReviewRoundDto`，形狀即 D1），`None` 回 `null`。
  - `remote_archived_station_ticket(connection_id, project, repo, dated_name, station)` → `with_remote` 內用既有 `archived_document(credentials, dated_name, "review.md"|"verify.md")` 取原文，404 回 `null`，其他以 `ticket_json_from_text` 解析。
- 遠端活變更不走原文＋解析：`GET /changes/{name}/artifacts/review.md` 會被引擎白名單拒絕，且 `GET /changes/{name}/review` 已回結構化 `rounds`。不放寬白名單——白名單是 verb-contract 的一部分，放寬會連動 CLI 與規格。

錯誤面：本機 I/O 失敗與遠端非 404 錯誤都以 `Err(String)` 上拋（既有 `document` 指令同款），前端以 `.catch` 落到「載入完成、無工單」空態；離線不會被誤讀成「沒有工單」——分頁仍在（狀態來自清單），只是內容空態。

### D3 分頁顯示條件沿用清單狀態，Tabs 改受控以處理退場

- 不新增「工單是否存在」查詢。活面板：`change.reviewStatus === "inReview"` 出「審查」分頁、`change.verifyStatus === "inVerify"` 出「驗證」分頁；已封存面板：`reviewStatus === "reviewedNotPassed"`／`verifyStatus === "verifiedNotPassed"`。分頁排在「排程」（活面板）或「規格」（已封存面板）之後，審查在前、驗證在後，與狀態列章籤同序。
- 工單載入時機：分頁出現時載入（不是面板開啟就載），並隨既有 `refreshGen`（外部檔案變更）重載——工單追加輪後內容要跟著更新。載入器由 host（`apps/desktop/src/App.tsx`）以 `loadStationTicket(change, station)`／`loadArchivedStationTicket(datedName, station)` props 傳入，與 `loadDocument` 同款。
- `Tabs` 由 `defaultValue` 改為受控 `value`／`onValueChange`：當前分頁為「審查」或「驗證」而對應狀態翻為非進行中（看板刷新後工單已刪）時，把 `value` 設回 `"proposal"`。切換變更（`change.name` 改變）時同樣重設為 `"proposal"`（現行 uncontrolled 已是如此，受控後要明寫）。

替代方案「面板開啟就預載兩份工單」被排除：多數變更沒有工單，會多兩次無意義 I/O；「工單消失時留在空分頁」被排除：分頁已不存在，留在其內容區是壞狀態。

### D4 `TicketView` 的呈現規則

新元件 `packages/ui/src/components/TicketView.tsx`，props：`station: "review" | "verify"`、`ticket: StationTicket`。

- 段標題：站名（「審查」／「驗證」）、「第 N 輪」（N 為末輪 index）、括號內末輪階段詞、末輪三級計數「CRITICAL n · WARNING n · SUGGESTION n」（嚴重度標籤維持英文，與工單原文一致）。
- 輪次列表：依 index 升序，每輪一個可收合區塊；末輪預設展開，其餘預設收合。輪標題：「第 N 輪」＋階段詞（`discovery` →「首輪」、`validation` →「複驗」、`null` → 不顯示階段詞）＋「範圍 N 檔」（N 為 `scope.length`；點擊展開檔案路徑清單，等寬字）＋「M 條」（`findings.length`）。
- finding 列：嚴重度色章（CRITICAL 用 destructive 紅、WARNING 用既有 amber、SUGGESTION 用 muted 灰，色票沿用 `reviewStyle.tsx` 已定義的三紅分工原則）、路徑等寬字、描述原文。描述行尾若以結構 token `(accepted)` 結尾（允許尾端空白），去掉 token 後顯示「已接受」籤；token 只在行尾辨識，行中出現不處理。描述不翻譯、不截斷、可換行。
- 零 findings 的輪顯示「本輪無 findings」。
- 空態（分頁存在但載入完成無內容）：「工單尚未抵達或已被刪除」。
- 大小：無虛擬化——工單輪數通常個位數，findings 數十條以內。

替代方案「以 finding 為主、不分輪次」被排除：驗證輪會逐字帶前輪未解 findings，不分輪會重複顯示且看不出收斂。

### D5 已封存面板同元件、同載入器形狀

`ArchivedDrawer` 新增 `loadStationTicket(datedName, station)` prop，分頁條件用既有 `reviewStatus`／`verifyStatus` props（`reviewedNotPassed`／`verifiedNotPassed`）。內容以同一個 `TicketView` 渲染。`DataSource` 介面（`packages/ui/src/adapter.ts`）新增 `getStationTicket(change, station)` 與 `getArchivedStationTicket(datedName, station)`，`apps/desktop` 的 `tauriDataSource.ts` 與 `remoteDataSource.ts` 各接對應 Tauri 指令；`App.test.tsx` 的資料源假件補兩個方法。

### D6 詞彙：「首輪」「複驗」入 LANGUAGE.md

`openspec/LANGUAGE.md` 新增兩條詞條：「首輪」（definition：工單的 discovery 輪——第一輪、唯一的全面盤點；avoid：發現輪、discovery 輪〔中文散文中〕）、「複驗」（definition：工單的 validation 輪——只判原 findings 與修補 patch 的直接回歸；avoid：驗證輪、複審、validation 輪〔中文散文中〕；why：「驗證輪」與驗證站撞名）。並於「換頁」詞條補一句：詳情面板的 tab 稱「分頁」，不稱「頁籤」。i18n 的 en 對應：`First pass`／`Re-check`。

## Implementation Contract

**行為**

- 活變更：`reviewStatus === "inReview"` 時面板分頁列為「提案／設計／任務／規格／排程／審查」，`verifyStatus === "inVerify"` 時再加「驗證」；兩者皆非時維持五個。已封存：`reviewedNotPassed`／`verifiedNotPassed` 時於「規格」後加對應分頁。
- 分頁內容依 D4；載入三態依既有 skeleton 正典。
- 看板刷新後狀態翻為非進行中：分頁消失；若當前分頁正是它，切回「提案」。
- 遠端：活變更走 `GET /changes/{name}/{station}`；封存走 `GET /archived/{dated}/artifacts/review.md`（或 `verify.md`）。404 → 空態；其他錯誤 → 空態（分頁仍在）。

**介面**

- `DataSource.getStationTicket(change: string, station: "review" | "verify"): Promise<StationTicket | null>`
- `DataSource.getArchivedStationTicket(datedName: string, station: "review" | "verify"): Promise<StationTicket | null>`
- `StationTicket`／`TicketRound`／`TicketFinding` 型別（D1）匯出自 `packages/ui`。
- Tauri 指令：`station_ticket`、`archived_station_ticket`、`remote_station_ticket`、`remote_archived_station_ticket`，回 `Option<Value>`。
- desktop core：`station_ticket_at`、`archived_station_ticket_at`、`ticket_json`、`ticket_json_from_text`。
- 引擎：`speclink_core::station::parse_ticket` 為 `pub`。
- i18n 新詞條（tw／en）：`drawer.tab.review`（審查／Review）、`drawer.tab.verify`（驗證／Verify）、`ticket.round`（第 {n} 輪／Round {n}）、`ticket.phase.discovery`（首輪／First pass）、`ticket.phase.validation`（複驗／Re-check）、`ticket.scopeFiles`（範圍 {n} 檔／{n} files in scope）、`ticket.findings`（{n} 條／{n} findings）、`ticket.accepted`（已接受／Accepted）、`ticket.noFindings`（本輪無 findings／No findings this round）、`ticket.empty`（工單尚未抵達或已被刪除／Ticket not loaded or already removed）。

**驗收**

- `packages/ui`：`ticketView.test.tsx`（段標題計數、末輪展開舊輪收合、階段詞三態、accepted 籤、零 findings）；`richDrawer.test.tsx`（分頁條件出現／缺席、載入器以正確 station 呼叫、狀態翻轉後分頁消失且退回提案）；`archivedDrawer.test.tsx`（封存側條件分頁）。指令：`npm test -w packages/ui`。
- `apps/desktop`：`App.test.tsx` 假件補方法後既有測試綠；`npm test -w apps/desktop`。
- `speclink-desktop-core`：`query.rs` 測試——本機活工單解析成 D1 形狀（欄位存在、camelCase）、無檔 `None`、worktree 覆蓋層下讀 worktree 副本、封存工單解析、壞格式回 `None`；`cargo test -p speclink-desktop-core`。
- `speclink-core`：`parse_ticket` 公開後 `cargo test -p speclink-core` 綠；golden 不變（無輸出改動）。
- Tauri 殼：`cargo build -p speclink-desktop` 通過（指令註冊）；遠端指令的 404 → `null` 映射以 `remote.rs` 既有測試風格補一條。
- 詞彙守門：`node --test scripts/*.test.mjs scripts/*/*.test.mjs` 綠（LANGUAGE.md 新 avoid 詞不得在使用者可見文案中出現）。

**範圍界線**

- In：上述四層（引擎公開函式、desktop core、Tauri 殼、`packages/ui`＋`apps/desktop` 接線）、`desktop-app` delta 規格、LANGUAGE.md、i18n。
- Out：server、CLI、`speclink-node`、`apps/server-web`、任何動詞、章籤點擊跳分頁、歷史回看。

## Risks / Trade-offs

- [D2 遠端封存原文在殼層解析——殼層多了一行引擎呼叫] → 呼叫的是 desktop core 的 `ticket_json_from_text`，殼仍是單行委派；解析本體在引擎。
- [工單格式日後演進（新欄位）] → D1 只鏡射既有 `ReviewRoundDto`，新增欄位為 additive，前端忽略未知欄位；`#[serde(default)]` 已在 DTO 上。
- [受控 Tabs 改動既有五分頁的切換行為] → `richDrawer.test.tsx` 既有分頁測試作回歸；切換變更時重設為「提案」與現行 uncontrolled 行為一致。
- [`speclink-desktop`（Tauri 殼）測試需手補 sidecar 與 server-web dist，且 desktop lib 測試極慢] → 本變更對殼只加委派指令，驗收以 `cargo build -p speclink-desktop` 為主，行為測試落在 desktop core 與前端。
- [跨平台：scope 路徑以 `/` 顯示] → 工單內路徑本來就是 repo-root 相對、正斜線（review-station 規格），前端原樣顯示，不做平台轉換。
- [golden 與 CLI 測試] → 無 CLI 人眼或 `--json` 輸出改動，golden 不動；`parse_ticket` 公開不改簽名以外的任何行為。
- [remote 舊 server 無 `/{station}` 端點] → `station_ticket_if_any` 對 404 回 `None`，分頁顯示空態；不視為錯誤。

## Migration Plan

純新增，無資料遷移。回滾＝還原前端分頁與四個指令；引擎的 `pub` 可保留。

## Open Questions

無。

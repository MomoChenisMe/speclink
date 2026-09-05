# server-read-api Specification

## Purpose

server 的唯讀查詢面：scopes 清單依 membership 過濾、正典 spec 內文的讀取、已封存內容的瀏覽端點，以及 workspace 全文搜尋。本 capability 保證任何具身分者都能取得自己可見的 scope 清單、搜尋語意與桌面端對齊，且已封存內容的列舉是 Store 契約的一部分而非各端自行拼裝。

## Requirements

### Requirement: scopes 清單依 membership 過濾且身分即可呼叫

server SHALL 提供授權範圍查詢端點：以 Bearer 憑證（PAT 或 access token）驗身分即可呼叫、SHALL NOT 要求 repo 綁定 header——此端點正是用來選擇 scope 的。回應 SHALL 為呼叫者具 membership 的 Projects 及其 Repos（識別、key、顯示名）；無任何 membership SHALL 回空清單而非錯誤；停用帳號 SHALL 拒於 403、缺憑證 401。admin SHALL NOT 特權繞過 membership 過濾。

#### Scenario: 不同 membership 互不可見

- **WHEN** 使用者甲僅具專案 A 的 membership、使用者乙僅具專案 B 的，各自呼叫 scopes 端點
- **THEN** 甲只見 A 及其 repos、乙只見 B；互相看不見對方的專案

#### Scenario: 無 membership 回空清單

- **WHEN** 具有效憑證但無任何專案 membership 的使用者呼叫 scopes 端點
- **THEN** 回應為空清單、狀態 200，非 403


<!-- @trace
source: server-scope-read-api
updated: 2026-07-20
code:
  - apps/desktop/src-tauri/src/lib.rs
  - apps/desktop/src-tauri/src/remote.rs
  - apps/desktop/src-tauri/tests/remote_data.rs
  - apps/desktop/src/__tests__/helpers/remoteFixtures.ts
  - apps/desktop/src/__tests__/remoteCapabilities.test.tsx
  - apps/desktop/src/__tests__/remoteDataSource.test.ts
  - apps/desktop/src/__tests__/remoteOpen.test.ts
  - apps/desktop/src/adapter/remoteDataSource.ts
  - crates/speclink-core/src/store.rs
  - crates/speclink-core/src/teststore.rs
  - crates/speclink-fs/src/lib.rs
  - crates/speclink-fs/tests/store_fs.rs
  - crates/speclink-host/src/bridge.rs
  - crates/speclink-host/src/drift.rs
  - crates/speclink-protocol/src/query.rs
  - crates/speclink-remote/src/client.rs
  - crates/speclink-remote/tests/read_api.rs
  - crates/speclink-server/src/app.rs
  - crates/speclink-server/src/auth.rs
  - crates/speclink-server/src/lib.rs
  - crates/speclink-server/src/read_api.rs
  - crates/speclink-server/tests/read_api.rs
  - scripts/dev.mjs
-->

---
### Requirement: 正典 spec 內文可讀

server SHALL 提供綁定 scope 下正典 spec 內文的讀取端點（capability 定址）：存在回內文、缺席回 404（沿用既有 wire 錯誤詞彙）；SHALL 為唯讀且不觸發任何寫入。

#### Scenario: 讀取存在的正典 spec

- **WHEN** 對已封存合併過的 capability 呼叫 spec 內文端點
- **THEN** 回應內文與 store 中正典 spec.md 一致


<!-- @trace
source: server-scope-read-api
updated: 2026-07-20
code:
  - apps/desktop/src-tauri/src/lib.rs
  - apps/desktop/src-tauri/src/remote.rs
  - apps/desktop/src-tauri/tests/remote_data.rs
  - apps/desktop/src/__tests__/helpers/remoteFixtures.ts
  - apps/desktop/src/__tests__/remoteCapabilities.test.tsx
  - apps/desktop/src/__tests__/remoteDataSource.test.ts
  - apps/desktop/src/__tests__/remoteOpen.test.ts
  - apps/desktop/src/adapter/remoteDataSource.ts
  - crates/speclink-core/src/store.rs
  - crates/speclink-core/src/teststore.rs
  - crates/speclink-fs/src/lib.rs
  - crates/speclink-fs/tests/store_fs.rs
  - crates/speclink-host/src/bridge.rs
  - crates/speclink-host/src/drift.rs
  - crates/speclink-protocol/src/query.rs
  - crates/speclink-remote/src/client.rs
  - crates/speclink-remote/tests/read_api.rs
  - crates/speclink-server/src/app.rs
  - crates/speclink-server/src/auth.rs
  - crates/speclink-server/src/lib.rs
  - crates/speclink-server/src/read_api.rs
  - crates/speclink-server/tests/read_api.rs
  - scripts/dev.mjs
-->

---
### Requirement: archived 瀏覽三端點

server SHALL 提供綁定 scope 下 archived changes 的清單（dated name 降冪，含 date、name、任務計數、觸及規格數、建立者與來源討論——自 archived meta 與 artifacts 衍生）、單一 archived change 的 artifact 內文（缺席 404）與 delta capabilities 清單。三端點 SHALL 唯讀且不跨 scope 可見。

#### Scenario: archive 後即可瀏覽

- **WHEN** 一個 change 經 archive 動詞封存後呼叫 archived 清單與其 proposal 內文端點
- **THEN** 清單含該 dated name 且欄位如實，內文與封存時一致


<!-- @trace
source: server-scope-read-api
updated: 2026-07-20
code:
  - apps/desktop/src-tauri/src/lib.rs
  - apps/desktop/src-tauri/src/remote.rs
  - apps/desktop/src-tauri/tests/remote_data.rs
  - apps/desktop/src/__tests__/helpers/remoteFixtures.ts
  - apps/desktop/src/__tests__/remoteCapabilities.test.tsx
  - apps/desktop/src/__tests__/remoteDataSource.test.ts
  - apps/desktop/src/__tests__/remoteOpen.test.ts
  - apps/desktop/src/adapter/remoteDataSource.ts
  - crates/speclink-core/src/store.rs
  - crates/speclink-core/src/teststore.rs
  - crates/speclink-fs/src/lib.rs
  - crates/speclink-fs/tests/store_fs.rs
  - crates/speclink-host/src/bridge.rs
  - crates/speclink-host/src/drift.rs
  - crates/speclink-protocol/src/query.rs
  - crates/speclink-remote/src/client.rs
  - crates/speclink-remote/tests/read_api.rs
  - crates/speclink-server/src/app.rs
  - crates/speclink-server/src/auth.rs
  - crates/speclink-server/src/lib.rs
  - crates/speclink-server/src/read_api.rs
  - crates/speclink-server/tests/read_api.rs
  - scripts/dev.mjs
-->

---
### Requirement: workspace 全文搜尋與桌面語意對齊

server SHALL 提供綁定 scope 下的全文搜尋端點：不分大小寫子字串比對 active 變更全部 artifacts 與 live 討論記錄全文，每卡回傳首個命中（卡片種類、識別、命中 artifact 檔名、含命中原文的前後文 snippet，截斷端補 …）；空或全空白查詢 SHALL 回空陣列。語意 SHALL 與桌面本地搜尋一致，使 remote 分頁的搜尋結果形狀與本地同形。

#### Scenario: 搜尋命中變更與討論

- **WHEN** 對含關鍵字的 change artifact 與討論記錄各一的 scope 以該關鍵字搜尋
- **THEN** 回應恰兩筆命中：一筆 kind 為 change、一筆為 discussion，各含 artifact 檔名與 snippet


<!-- @trace
source: server-scope-read-api
updated: 2026-07-20
code:
  - apps/desktop/src-tauri/src/lib.rs
  - apps/desktop/src-tauri/src/remote.rs
  - apps/desktop/src-tauri/tests/remote_data.rs
  - apps/desktop/src/__tests__/helpers/remoteFixtures.ts
  - apps/desktop/src/__tests__/remoteCapabilities.test.tsx
  - apps/desktop/src/__tests__/remoteDataSource.test.ts
  - apps/desktop/src/__tests__/remoteOpen.test.ts
  - apps/desktop/src/adapter/remoteDataSource.ts
  - crates/speclink-core/src/store.rs
  - crates/speclink-core/src/teststore.rs
  - crates/speclink-fs/src/lib.rs
  - crates/speclink-fs/tests/store_fs.rs
  - crates/speclink-host/src/bridge.rs
  - crates/speclink-host/src/drift.rs
  - crates/speclink-protocol/src/query.rs
  - crates/speclink-remote/src/client.rs
  - crates/speclink-remote/tests/read_api.rs
  - crates/speclink-server/src/app.rs
  - crates/speclink-server/src/auth.rs
  - crates/speclink-server/src/lib.rs
  - crates/speclink-server/src/read_api.rs
  - crates/speclink-server/tests/read_api.rs
  - scripts/dev.mjs
-->

---
### Requirement: archived 列舉為 Store 契約的一部分

core Store 契約 SHALL 提供 archived changes 的列舉（dated name 清單）；各實作站點 SHALL 補齊——測試 store 與 host 的 TeamStore 橋接回真值、drift 專用唯讀最小 adapter 顯式回空並註明用途限定、檔案系統實作以 archive 目錄列舉。列舉排序 SHALL 為 dated name 降冪。

#### Scenario: 橋接列舉與點讀一致

- **WHEN** 對 TeamStore 橋接先 archive 一個 change 再呼叫列舉與 exists 點讀
- **THEN** 列舉含該 dated name 且 exists 為真，兩者一致

<!-- @trace
source: server-scope-read-api
updated: 2026-07-20
code:
  - apps/desktop/src-tauri/src/lib.rs
  - apps/desktop/src-tauri/src/remote.rs
  - apps/desktop/src-tauri/tests/remote_data.rs
  - apps/desktop/src/__tests__/helpers/remoteFixtures.ts
  - apps/desktop/src/__tests__/remoteCapabilities.test.tsx
  - apps/desktop/src/__tests__/remoteDataSource.test.ts
  - apps/desktop/src/__tests__/remoteOpen.test.ts
  - apps/desktop/src/adapter/remoteDataSource.ts
  - crates/speclink-core/src/store.rs
  - crates/speclink-core/src/teststore.rs
  - crates/speclink-fs/src/lib.rs
  - crates/speclink-fs/tests/store_fs.rs
  - crates/speclink-host/src/bridge.rs
  - crates/speclink-host/src/drift.rs
  - crates/speclink-protocol/src/query.rs
  - crates/speclink-remote/src/client.rs
  - crates/speclink-remote/tests/read_api.rs
  - crates/speclink-server/src/app.rs
  - crates/speclink-server/src/auth.rs
  - crates/speclink-server/src/lib.rs
  - crates/speclink-server/src/read_api.rs
  - crates/speclink-server/tests/read_api.rs
  - scripts/dev.mjs
-->

---
### Requirement: 討論定案搜尋端點

server SHALL 提供綁定 scope 下的唯讀端點 GET /discussions/search，query 參數 q 為以空白分隔的關鍵字。server SHALL 以空白切詞後執行與本機 speclink discuss search 相同語意的搜尋（範圍涵蓋在途與封存記錄、比對限 topic、slug 與四種決定行、排序 topic 或 slug 命中優先），回應為 hits 陣列，每筆為該討論的資訊欄位加 matches 陣列（每個 match 含 kind、where、text）。q 缺席或全空白 SHALL 回 HTTP 400、error reason 為 invalid_argument。端點 SHALL 對具讀取權限者開放（reader role 可呼叫），SHALL NOT 寫入任何資料。既有 GET /search 端點的語意（在途記錄全文、每卡首個命中、與桌面本地搜尋對齊）SHALL 維持不變。未綁定、離線與認證失效的可觀察行為 SHALL 沿既有讀取端點的錯誤分類。

#### Scenario: 在途與封存各一筆命中

- **WHEN** scope 內有一筆在途記錄的 topic 含 drawer、一筆封存記錄第 2 輪的 `**Ruled out**:` 行含 drawer，呼叫 GET /discussions/search?q=drawer
- **THEN** HTTP 200；hits 恰兩筆，topic 命中的在途記錄在前（archived 為 false、matches 含 kind 為 topic 的項目），封存記錄在後（archived 為 true、matches 含 kind 為 ruled-out、where 為 round-2 的項目）

#### Scenario: q 缺席或全空白

- **WHEN** 呼叫 GET /discussions/search 不帶 q，或帶 q=%20
- **THEN** HTTP 400、error reason 為 invalid_argument；不寫入任何資料

#### Scenario: reader role 可呼叫

- **WHEN** 以 reader role 憑證呼叫 GET /discussions/search?q=golden
- **THEN** HTTP 200，回應形狀與 editor role 相同

#### Scenario: 既有全文搜尋端點不變

- **WHEN** 於本變更前後對同一 scope 呼叫 GET /search?q=drawer
- **THEN** 回應逐位元一致：仍只含在途記錄與變更 artifacts 的首個命中，不含封存記錄

<!-- @trace
source: discuss-search-recall
updated: 2026-09-05
-->
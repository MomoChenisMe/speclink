## Why

remote-data-source 刀落地時如實停用了五個 server 沒有端點的讀取面：授權範圍內的 Project/Repo 清單（下一刀 chooser 的硬前置——§10.5 是「選」不是「打字」）、正典 spec 內文（PM spec-only workspace 看得到規格卡、打不開內文，最痛）、archived 瀏覽、全文搜尋。這些全是純讀取，store 與 identity 的地基都已存在（DocumentId 的 CanonicalSpec/ArchivedChange 定址、identity 的 list_projects/list_repos/list_memberships），缺的只是端點、client 方法與桌面解鎖。

## What Changes

- server 新增五個讀取端點（全部沿用既有三 headers 認證）：
  - GET /scopes——呼叫者授權範圍內的 Project 與其 Repos 清單（identity 的 projects 交集 memberships；無 membership 即空清單，fail-closed）。此端點在 binding 之前使用（就是用來選 scope 的），採「僅 Bearer 身分、不需 X-Speclink-Repo」的新認證抽取形狀。
  - GET /specs/{capability}/document——正典 spec 內文；缺席 404。
  - GET /archived 與 GET /archived/{datedName}/artifacts/{*artifact} 與 GET /archived/{datedName}/capabilities——archived changes 清單（datedName、date、name、任務計數、specCount、createdBy、fromDiscussions，與桌面封存卡欄位對齊）、文件內文、delta capabilities。
  - GET /search——workspace 全文查詢，語意與桌面 D6 對齊：不分大小寫子字串、範圍為 active 變更 artifacts 與 live 討論記錄、每卡回傳首個命中與 snippet；空查詢回空陣列。
- core Store trait 補 archived changes 列舉 seam（現況只有點讀與 exists；archived 討論有列舉、archived changes 沒有）：teststore 與 host BridgeStore 實作真值、drift 最小 adapter 顯式回空、CLI 側 fs 實作補齊。
- speclink-protocol 新增對應 DTOs；speclink-remote client 新增五個方法。
- Desktop 解鎖：RemoteDataSource 的 listArchived／getArchivedDocument／archivedCapabilities／getSpecDocument／searchWorkspace 由「不支援」改直達端點，capability 描述翻正，remote 分頁的 archived 頁與 spec 內文提示卡移除、搜尋啟用。scopes 端點本刀只落 server 與 client 方法，桌面消費屬下一刀 chooser。

## Capabilities

### New Capabilities

- `server-read-api`: server 讀取面補洞的行為保證——scopes 清單的身分認證與 membership 過濾、正典 spec 內文、archived 三端點、search 的 D6 同語意，全部唯讀且 fail-closed。

### Modified Capabilities

- `remote-workspace-data`: capability 停用清單縮減——封存瀏覽、全文搜尋、正典 spec 內文改為直達；validate/analyze、刪除變更、任務拖排、看板拖排維持停用（server 仍無端點）。

## Impact

- 相容性影響：全部純新增端點與唯讀查詢，既有端點與 CLI 輸出零改動；Store trait 加列舉方法屬內部契約擴充（各實作站點同步補齊）。桌面解鎖後 remote 分頁的封存/搜尋/spec 內文行為向本地看齊。
- Affected specs: `server-read-api`（新增）、`remote-workspace-data`（修改）
- Affected code:
  - New: crates/speclink-server/src/read_api.rs、crates/speclink-server/tests/read_api.rs
  - Modified: crates/speclink-server/src/app.rs、crates/speclink-server/src/auth.rs、crates/speclink-protocol/src/query.rs、crates/speclink-remote/src/client.rs、crates/speclink-core/src/store.rs、crates/speclink-core/src/teststore.rs、crates/speclink-host/src/bridge.rs、crates/speclink-host/src/drift.rs、apps/desktop/src-tauri/src/remote.rs、apps/desktop/src/adapter/remoteDataSource.ts、apps/desktop/src/__tests__/remoteDataSource.test.ts、Cargo.lock
  - Removed: 無

## Context

remote-data-source 的覆蓋矩陣把五個讀取面標為「明確不支援」，其 design 明文補端點屬後續獨立 server 刀——本刀即是。地基盤點：store 契約的 DocumentId 已有 CanonicalSpec 與 ArchivedChange 定址；core Store trait 有 read_canonical_spec／list_canonical_capabilities／read_archived_artifact／archived_delta_capabilities 點讀，但 archived changes 無列舉（archived 討論有 list_archived_discussions）；identity store 有 list_projects／list_repos(project_key)／list_memberships(user_id)；server 既有路由全部經 Binding 抽取器（需 X-Speclink-Repo），/scopes 卻是「選 scope 之前」要用的端點；桌面 D6 搜尋語意在 desktop-core 有既成參照（不分大小寫子字串、每卡首個命中、snippet 裁切）。

## Goals / Non-Goals

**Goals:**

- 五端點落地且全部唯讀、fail-closed；chooser（下一刀）拿到 /scopes 前置。
- 桌面 remote 分頁解鎖封存瀏覽、spec 內文、搜尋——與本地行為看齊。
- Store trait 的 archived 列舉 seam 一次補齊所有實作站點。

**Non-Goals:**

- 不加任何寫入或動詞端點（validate/analyze、deleteChange、moveTask、reorderCard 維持停用——它們是動詞/寫入面，各自需要獨立的語意決策）。
- 不動桌面對 /scopes 的消費（chooser 刀）；不動 CLI 命令面（CLI 本地模式不受影響，remote 模式的既有動詞不變）。
- 不做搜尋索引或分頁——團隊規模的線性掃描；量級成為問題時再立刀。

## Decisions

### 決策 1：/scopes 走「身分不綁定」抽取器

新增 IdentityOnly 認證抽取器：驗 Bearer（PAT 或 access token）與帳號狀態（停用即 403），不要求 X-Speclink-Repo。/scopes 為唯一使用者；回應＝list_projects 過濾至呼叫者 memberships（project 層），每項附 list_repos 的 repos。admin 不特權——同樣走 membership（要看見就先給自己 membership），fail-closed 一致。替代案「repo header 帶假值走既有 Binding」被否：語意撒謊且 binding 失敗路徑會誤導審計。

### 決策 2：archived 列舉 seam 加在 core Store trait

新增 list_archived_changes 回 dated name 清單（排序＝dated name 降冪，與桌面封存頁一致）；各站點：teststore 真值、host BridgeStore 以 TeamStore 查詢實作（與既有 list_changes 同構路徑——它解決過同形問題）、drift 的唯讀最小 adapter 顯式回空（其用途僅 drift 計算，permanent 註明）、CLI 側 fs Store 實作以 archive 目錄列舉補齊。清單端點的欄位衍生（任務計數自 tasks.md checkbox、specCount 自 capabilities、createdBy/fromDiscussions 自 archived meta）在 server 的 read_api 組合層做，不下沉 trait。

### 決策 3：search 語意＝桌面 D6 逐字對齊

範圍：active 變更的全部 artifacts＋live 討論記錄全文；不分大小寫子字串；每卡回傳首個命中（kind、id、artifact 檔名、含命中原文的前後文 snippet 兩端截斷補 …）；空或全空白查詢回空陣列。實作在 server read_api 組合層以 core Store 讀取線性掃描——不進引擎（引擎無搜尋概念，桌面同樣在宿主層實作）。回應 DTO 欄位與桌面 SearchHit 對齊（kind：change｜discussion），RemoteDataSource 直通零轉換。

### 決策 4：端點形狀與錯誤語意

/specs/{capability}/document 與 /archived/{datedName}/artifacts/{*artifact} 回 JSON 包 content 欄位（與既有 get_artifact 端點同形），缺席 404 用既有 wire 錯誤詞彙；/archived 清單一次回全量（無分頁，Non-Goal 已記）；全部端點走既有 Binding（除 /scopes），維持 scope 隔離——archived 與 specs 皆屬綁定 scope 的資料。protocol DTOs 進 query.rs（ScopesResponse、SpecDocumentResponse、ArchivedListResponse/ArchivedItem、SearchResponse/SearchHit），schemars 照慣例。

### 決策 5：桌面解鎖的邊界

RemoteDataSource 五方法（listArchived、getArchivedDocument、archivedCapabilities、getSpecDocument、searchWorkspace）改走新 remote_* 命令直達；capability 描述矩陣翻正三項（封存瀏覽、spec 內文、搜尋），validate/analyze、刪除、拖排維持停用與提示。remote 分頁的 archived 提示卡與 spec 內文提示卡移除——同一 UI 路徑回歸本地同形。/scopes 的 client 方法（speclink-remote）本刀落地供 chooser 消費，桌面不接 UI。

## Implementation Contract

- server 整合測試（crates/speclink-server/tests/read_api.rs，in-process＋memory identity/store）：/scopes 依 membership 過濾（雙使用者不同 membership 斷言互不可見、無 membership 空清單、停用帳號 403、缺 Bearer 401）；spec 內文與 archived 三端點對播種資料回真值、缺席 404、跨 scope 不可見；search 的 D6 語意（大小寫不敏感、首個命中、snippet 截斷、空查詢空陣列）。
- Store trait 列舉：teststore 與 BridgeStore 的 list_archived_changes 在既有測試基建下驗證（archive 後列舉可見、排序正確）；drift adapter 回空有註記與測試。
- 桌面：remoteDataSource.test.ts 更新五方法映射；capability 翻正的停用測試更新（封存/搜尋/spec 內文改為可用、其餘維持停用斷言不變）。
- 手動驗證（remote-dev-harness）：npm run dev → remote 分頁開封存頁見清單與內文、規格卡開得了內文、搜尋可用；以無 membership 帳號登入 CLI 呼叫 /scopes 得空清單。
- 回歸：cargo test --workspace、npm test -w apps/desktop、CLI 輸出凍結不受影響（不動 CLI）；golden 不涉及（未動內嵌技能 assets）。

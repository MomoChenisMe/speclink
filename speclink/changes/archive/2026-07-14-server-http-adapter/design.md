## Context

typed client（crates/speclink-remote）對 project-scoped 基底 URL 送出三個標頭（Authorization bearer、X-Speclink-Api-Version、X-Speclink-Repo），路徑形如 /binding、/changes、/changes 下的 artifacts 與 tasks、/discussions 系列；錯誤以 status、reason、message 三元組解讀，reason 屬八值 registry。speclink-host 已有 resolve_binding（缺失/多義 fail closed）、SpeclinkExecutionContext、begin_unit_of_work 與 commit_with_events（文件＋outbox 原子提交，已對 in-memory reference 驗證），但尚無「engine 動詞在 TeamStore 上執行」的路徑——engine 命令層 execute 消費的是本地 fs 導向的 Store trait。twin harness（crates/speclink-cli/tests/remote_read_path.rs）以 tiny_http stub 固定了 remote 模式的輸出期望，fs 模式是形狀權威。

## Goals / Non-Goals

**Goals:**

- HTTP adapter → Host → Engine → TeamStore 一條正路打通：所有寫入經 UoW/CAS 原子提交、事件落 outbox，無任何繞過 Host 的旁路。
- typed client 現有全部動詞路徑有真 server 可打，輸出與 stub 對測期望一致。
- binding、認證、組態全部 fail closed；Query＋ETag 是唯一更新地基。

**Non-Goals:**

- 不做 SSE/WebSocket push（後續子刀；/sync-state 輪詢已足以收斂）。
- 不做 invite、PAT 自助管理、device flow、/setup、/login、/account、/admin 與任何 Web UI（auth/admin 子刀）。
- 不做 backup 排程、restore validation、Docker/compose 發布產物（發布子刀）。
- 不做 ServerFS 與 PostgreSQL driver；不宣稱 cluster。
- 不動桌面 app 與本地 fs 模式的任何行為。
- server 不 shell-out 任何 git 操作；code evidence 一律由 client 上行（路線圖 §6 反模式）。

## Decisions

### 決策 1：橋接落在 host，形狀是「snapshot 唯讀視圖＋寫入捕捉」

speclink-host 新增 bridge 模組：以 TeamStore snapshot 實作 engine 命令層所需的 store 讀取視圖，變更型動詞的寫入不落檔案系統而是捕捉為 UnitOfWork 的 staged ops，成功後連同領域事件經既有 commit_with_events 原子提交。engine 命令層本身不改——它已是 typed command/outcome/error，橋接只是第二個 store 供應者。驗收基準是雙路徑一致：同一動詞對同一內容分別經 fs seam 與經橋接執行，typed outcome、錯誤分類、領域事件相同。

### 決策 2：HTTP 框架用 axum，async 只進 server crate

server crate 用 axum＋tokio：路由、標頭處理、graceful shutdown 成熟，後續子刀的 SSE 與內嵌 Admin UI 靜態資源都有直接支援。engine/host/store 維持同步——handler 以 spawn_blocking 呼叫橋接執行，每個 project scope 一個寫入序列化點（single-node 定位，與 SQLite 單寫者一致）。async 邊界止於 speclink-server crate，不外洩進其他 crate 的介面。

### 決策 3：組態檔宣告 store、registry 與 bootstrap token，解析 fail closed

server 以 --config 指定 YAML 組態檔：store driver（sqlite 路徑或 memory）、projects（key、名稱、repos 清單）、tokens（bearer token → actor id/display）。組態檔缺失、不可解析、或宣告了未知 driver 即啟動失敗並印出原因——與 change-metadata-fail-closed 同一原則，絕不靜默退預設。此為 bootstrap 認證：token 是運維者手動配置的長字串，帳號系統與 PAT 生命週期屬後續子刀，屆時本組態段被 store 內的帳號資料取代。

### 決策 4：binding 裁決重用 host 的 resolve_binding

/binding 與所有動詞路由共用同一前置：token → actor（未知回 401 permission_denied）；URL project key 對 registry（未註冊回 404 not_found）；X-Speclink-Repo 標頭對該 project 的 repos——標頭存在但未註冊回 not_found、缺標頭且 repos 多於一個依 resolve_binding 回多義拒絕（refused），恰一個 repo 時綁定它。X-Speclink-Api-Version 不相容回 refused 帶版本原因。任何一步失敗都不進入動詞執行。

### 決策 5：ETag 是 scope 的單調狀態記號，If-Match 沿用文件 revision

查詢回應的 ETag 與 /sync-state 的狀態記號同源：該 scope 全部文件 revision 的聚合摘要（單調變化，任何 commit 後必變）。client 以 If-None-Match 輪詢 /sync-state 判斷「有沒有變」，變了就走 Query 重讀——事件漏光也能收斂。寫入的 If-Match 維持 typed client 既有語意：對單一文件的 expected revision，不符即 409 revision_conflict（帶 expected/actual），由橋接的 CAS 失敗一路映射出來。

### 決策 6：錯誤映射單點實作

store 六類錯誤、host binding 拒絕、engine 命令層五碼到 wire 八值 reason 的映射在 server 內恰一處：revision_conflict→409 revision_conflict、not_found→404 not_found、permission_denied→401/403 permission_denied、invalid_argv→400 invalid_argument、invalid_config→422 invalid_config、refused→409 refused、unavailable→503 unavailable、其餘→500 internal。message 沿用 engine 現行錯誤訊息文字，讓 typed client 的訊息對映維持逐位元一致。三套語彙各守一層，不合併不擴值域。

### 決策 7：端到端以真 CLI 對真 server 重放 twin 情境

e2e 測試啟動真 server（tempdir SQLite 資料庫、測試組態），以環境變數把真實 CLI binary 指向它，重放 twin harness 全部情境並比對 stdout/stderr/exit code 與既有期望（fs 模式是形狀權威）。資料播種走正門：以命令路由建 change、寫 artifact，不直接對資料庫塞資料。stub 對測維持不動——stub 驗 client 行為，e2e 驗 server 行為，兩者互補不互代。

## Implementation Contract

- Behavior：運維者以組態檔啟動 speclink-server 後，團隊成員用現行 CLI remote 模式（連線 URL＋token）執行全部 remote 動詞，行為與 stub 對測期望一致；兩個 client 競寫同一 artifact 時敗方得到現行衝突訊息；server 重啟後資料完整（SQLite 持久化）。
- Interface / data shape：路由基底 /api/speclink/v1/projects/ 加 project key；請求回應皆為 speclink-protocol DTO；錯誤為 status、reason、message 三元組；/healthz 回程序存活、/readyz 回 store health 可用；/sync-state 回 scope 狀態記號並支援 ETag/If-None-Match。組態檔形狀：store 段（driver 與路徑）、projects 段（key/name/repos）、tokens 段（token/actor）。
- Failure modes：組態不可解析或 driver 未知 → 啟動失敗印原因（exit 非零）；未知 token → 401 permission_denied；未註冊 project → 404 not_found；repo 多義 → refused 且訊息指出候選需明示；CAS 敗 → 409 revision_conflict 帶 expected/actual；store unavailable → 503 unavailable 且 /readyz 轉不可用。
- Acceptance criteria：cargo test -p speclink-server 全綠（路由單元測試＋e2e）；cargo test -p speclink-host 橋接雙路徑一致測試全綠；npm run test:all 全綠且既有 parity/color/twin 凍結零 diff。

## Risks / Trade-offs

- axum/tokio 依賴樹大 → 只進 server crate，其他 crate 介面不沾 async；換取 SSE 與靜態資源服務的直接支援。
- scope 級 ETag 粒度粗（任何文件變動都使快取失效）→ 正確性優先，粒度優化留待有量測證據再做。
- bootstrap token 是明文組態 → 檔案權限與部署文件註記責任，帳號子刀落地後退場；不因此先建半套帳號系統。
- 橋接讓 engine 在無檔案系統的 store 視圖上執行，個別動詞可能暗依賴檔案路徑語意 → 雙路徑一致測試逐動詞覆蓋，發現的每個暗依賴修在橋接視圖而非 engine 分叉。

## Migration Plan

純新增：本地模式不動，既有 remote 模式（對自訂服務）不動——本刀交付的是第一個官方 server 實作，client 端零變更。部署順序：sqlite-team-store 刀先落地，本刀 e2e 才能以 SQLite 跑；若先行開發，橋接與路由以 in-memory store 驗證，SQLite 接線任務置後。回退即停用 server binary，資料留在 SQLite 檔中可 export。

## Open Questions

（無）

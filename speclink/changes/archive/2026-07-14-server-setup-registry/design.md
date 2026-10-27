## Context

binding 裁決目前在 auth.rs：URL project key 對 state.config.projects（組態靜態表）查核、X-Speclink-Repo 對該 project 的 repos 清單裁決。identity 資料庫（server 自有 SQLite，schema version 演進與守門機制已就緒）存 users/memberships/invitations/PATs/sessions/device 憑證；invite 子命令直接對 identity 資料庫建邀請並以組態 public_url 組 URL。組態檔已歷經一次段落汰換（bootstrap tokens 段由 server-identity-pat 移除，殘留即啟動失敗）。藍圖 §13.2 規定 /setup 以首次啟動輸出的一次性 bootstrap token 進入，完成第一位 Admin、Store driver/能力測試、migration、第一個 Project/Repo 與初始連線資訊，完成後 token 失效且 route 關閉；§13.4 的開箱流程以 /setup 為起點。

## Goals / Non-Goals

**Goals:**

- registry 成為庫中事實：admin 子刀屆時只需加管理介面，不再動資料模型；binding 語意與錯誤分類零變更。
- 運維者從 docker compose up 到第一組可用的 Project/Repo 全程瀏覽器完成；bootstrap token 一次性且完成即關門。
- 組態檔收斂為純部署關注點（store 位置、identity 位置、public url、事件參數、bind 位址）。

**Non-Goals:**

- 不做 /admin Web UI 與 audit log（server-admin-audit 刀）；本刀的 registry 寫入介面只服務 /setup 與測試播種，一般營運的 registry 管理屬 admin 刀。
- 不做 registry 的刪除與改名語意（admin 刀連同其對既有 binding/資料的影響一併設計）；本刀只有建立與讀取。
- 不做 backup/restore（server-backup-restore 刀）。
- 不動 PAT/device 認證、SSE、engine 橋接與任何 API 路由行為。
- public url 維持部署組態單一來源：/setup 顯示連線資訊，不寫入 public url——避免組態與資料庫雙源打架。

## Decisions

### 決策 1：registry 落 identity 資料庫，schema 演進一版

projects（key、名稱）與 repos（所屬 project、key、名稱）兩張表落既有 identity 資料庫，schema version 遞增一版，沿用「舊版 migrate 升級、較新拒開、既有資料完整保留」的守門契約。identity 層介面新增 registry 讀寫（列 projects、查 project、列 repos、建 project、建 repo）；不另起第二個資料庫檔——registry 與帳號同屬 server 營運資料，備份與守門一體。

### 決策 2：binding 裁決改讀庫，語意逐位元不變

auth.rs 的 project 查核與 repo 裁決改查 registry 介面：未註冊 project key 回 404 not_found、repo 標頭未註冊回 not_found、多 repo 缺標頭拒絕不代選、恰一 repo 綁定——錯誤分類、reason 與 message 全部沿用現值，既有 binding 測試改播種方式後期望不變。組態 projects 段移除；殘留即啟動失敗，錯誤訊息指出該段已由 registry 取代（沿用 tokens 段汰換的同一報告模式）。

### 決策 3：bootstrap token 以「無 admin」為生成條件，完成即永久關門

server 啟動時檢查 identity 儲存：不存在任何 admin 使用者（且無未過期的 setup token）→ 生成高熵 bootstrap token，hash 落庫（帶到期，預設 24 小時），明文印於 stdout 並提示 /setup 路徑；存在 admin → 不生成、/setup 回 404。setup 完成（第一位 Admin 建立成功）即耗用 token；token 過期而 setup 未完成 → 重啟 server 生成新 token（舊 hash 作廢）。token 只出現在 stdout——不落 log 檔、不進組態。

### 決策 4：/setup 是 token 門禁的單一流程頁

/setup 以 bootstrap token 進入（URL query 或表單輸入，比對 hash）；流程單頁分節：(1) 建立第一位 Admin——email、顯示名、密碼，直接建立 active user 帶 admin 旗標，不走邀請；(2) 顯示 Store 狀態——manifest（driver、contract version、capabilities）、health 結果與 identity schema version，異常即明示（fail closed 已由啟動守門把關，此處是可視化）；(3) 建立第一組 Project 與 Repo（寫 registry）；(4) 顯示初始連線資訊——public url（出自部署組態）、project key、repo key 與「用 invite 子命令或後續 admin 介面邀請成員」指引。完成後 setup route 關閉；中途離開可憑同一 token 續作（冪等：已建的 admin/project 不重建）。POST 沿用既有同源驗證。

### 決策 5：invite 與 /setup 共用 registry 查核

invite 子命令的 --project 改對 registry 查核，未註冊 key 以非零 exit code 拒絕並列出既有 project keys；URL 基底維持組態 public_url。/setup 建立的第一位 Admin 不經邀請，其後全部成員仍走既有邀請鏈（CLI 或後續 admin 刀）。測試播種 helper 自 config projects 遷移為 registry 介面呼叫，涵蓋既有 binding/routes/e2e 測試的組態改寫。

## Implementation Contract

- Behavior：全新資料庫首次啟動的 server 在 stdout 印一次性 setup token 與 /setup 指引；運維者於瀏覽器完成 Admin、檢視 store 狀態、建立第一組 Project/Repo 後，即可用 invite 子命令邀請成員、成員以 PAT/device flow 連線；重啟後 /setup 回 404 且不再印 token。組態含 projects 段即啟動失敗。
- Interface / data shape：identity 資料庫新增 projects/repos 表與 setup token 記錄，schema version 遞增一版；identity 層 registry 介面（list/get project、list repos、create project、create repo）；GET/POST /setup（token 門禁、同源驗證）；組態檔欄位集 = store、identity、public url、事件段、bind 位址。
- Failure modes：無效/過期 setup token → 統一「無效」回應不區分原因；已有 admin 時訪問 /setup → 404；殘留 projects 段 → 啟動失敗指出已由 registry 取代；registry 內建立重複 project/repo key → 表單錯誤拒絕；invite 對未註冊 project → 非零 exit code 並列出既有 keys；識別資料庫較新版本 → 既有守門拒開。
- Acceptance criteria：cargo test -p speclink-server 全綠（registry 遷移後既有 binding 測試期望不變、setup 流程測試、子命令查核）；npm run test:all 全綠且 parity/color/twin 凍結零 diff。

## Risks / Trade-offs

- registry 與帳號同庫使 identity 資料庫概念擴大為「server 營運資料庫」→ 命名維持 identity 段不改組態鍵，避免又一次 BREAKING；文件與 admin 刀再統一稱謂。
- setup token 印在 stdout，容器環境會進 docker logs → 到期（24 小時）與一次性把窗口收窄；§13.2 本就以此為開箱機制，部署文件註記完成 setup 後 token 即失效。
- 冪等續作讓 /setup 中途狀態存在（admin 已建、project 未建）→ 每節獨立冪等、token 未耗用前可續作，重啟生成新 token 舊 token 作廢，不留半開門。
- binding 每請求多一次 registry 庫查 → 與 identity PAT 查驗同庫同模式，single-node 定位可接受。

## Migration Plan

apply 順序在 admin 與 backup 刀之前（兩者都依賴 registry 在庫）。identity schema 由 migrate 自動升級；組態遷移＝刪除 projects 段（測試資產同步改播種介面）。回退即回捨 change——較新 schema 對舊 binary 被守門拒開，需同時還原資料庫檔（無正式部署，可接受）。

## Open Questions

（無）

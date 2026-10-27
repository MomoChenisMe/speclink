## Why

Phase 1 已備齊三塊地基：wire contract 由 speclink-protocol 定義且 typed client 全面採用、TeamStore 契約與 conformance 已定、speclink-host 的 UoW/event commit 路徑已對 in-memory reference 驗證——但沒有任何東西在網路上服務這份契約：typed client 至今只能對 twin harness 的 stub 講話，host 的 commit 路徑也還沒有真正的消費者。平台藍圖（docs/platform-architecture.zh-TW.md §13.1、§14 Phase 2 第 1 項）要求官方 server 直接編譯成 Rust binary 呼叫同一份 Rust command runtime；路線圖（docs/implementation-refactor-roadmap.zh-TW.md §4.2 順位 8、§6）警告不得先建只有 CRUD 的 server 再事後補 CAS/history——所以第一子刀就要把「HTTP adapter → Host → Engine → TeamStore」整條正路打通，而不是繞過 Host 另寫一套。

目標使用者：以 CLI/Agent 連遠端規格庫的團隊成員（他們的動詞第一次有真 server 可打）、以及後續 auth/admin、SSE、backup 子刀的實作者（在本刀的 runtime 骨架上疊加）。

## What Changes

- speclink-host 新增 engine-over-TeamStore 執行橋接：把 TeamStore snapshot 呈現為 engine 命令層的唯讀 store 視圖、把變更型動詞的寫入捕捉進 UnitOfWork、經既有 commit 組合路徑（含領域事件映射至 outbox）原子提交。同一動詞經本橋接與經本地 fs seam 執行，typed outcome、錯誤分類與領域事件一致。
- 新增 `speclink-server` crate（binary）：HTTP adapter 服務 Client Protocol——路由以 /api/speclink/v1/projects/ 加 project key 為基底，涵蓋 typed client 現有全部查詢與命令路徑，加上 binding handshake、/healthz、/readyz 與 /sync-state（Query＋ETag 輪詢地基）。請求與回應全數為 speclink-protocol DTO，錯誤以 status、reason、message 三元組回應且 reason 屬八值封閉 registry。
- 啟動組態 fail closed：server 以組態檔宣告 Store driver（sqlite 為預設、memory 供測試）、Project/Repo registry 與 bootstrap bearer token 對 actor 的映射。token 缺失或未知即拒絕；project key 未註冊、repo 標頭未知或多義即依 host 的 binding 裁決拒絕，SHALL NOT 自動選擇候選。此為 bootstrap 認證，invite/PAT/device flow 屬後續子刀。
- 查詢回應攜 ETag、寫入驗 If-Match：revision 不符回 revision_conflict；漏事件的 client 可經 /sync-state 的 ETag 比對收斂（本刀不做 SSE/WS push）。
- 端到端驗證：以真實 CLI binary 對真實 server（SQLite store）重放 twin harness 全部情境，stdout/stderr/exit code 與 fs 模式的形狀權威逐位元一致。

## Capabilities

### New Capabilities

- `reference-server`: 官方 server 的 HTTP adapter 契約——路由與 DTO 服務、binding handshake 裁決、bootstrap 認證與 registry fail closed、ETag/If-Match、健康檢查與輪詢端點。

### Modified Capabilities

- `host-runtime`: 新增 engine-over-TeamStore 執行橋接需求——engine 動詞可於 TeamStore snapshot 上執行並經 UoW/event commit 原子提交，與 fs seam 的 typed outcome 一致。

## Impact

- 相容性影響：本地模式（fs seam）行為零變更，parity 31 項、color 16 項凍結不動；twin harness 8 情境除既有 stub 對測外新增對真 server 的重放，期望輸出不變。新增 axum/tokio 依賴只進 speclink-server crate，engine/host/store 維持無 async runtime。前置依賴：SQLite driver（sqlite-team-store 刀）須先落地，server 預設 driver 才能接線；橋接與路由開發可先以 in-memory store 進行。
- Affected specs: `reference-server`（新增）、`host-runtime`（修改）
- Affected code:
  - New: crates/speclink-server/Cargo.toml、crates/speclink-server/src/main.rs、crates/speclink-server/src/config.rs、crates/speclink-server/src/routes.rs、crates/speclink-server/src/auth.rs、crates/speclink-server/tests/e2e_cli.rs、crates/speclink-host/src/bridge.rs
  - Modified: Cargo.toml、Cargo.lock、crates/speclink-host/src/lib.rs
  - Removed: 無

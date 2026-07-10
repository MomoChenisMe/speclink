## 1. pg 參考 Store 與 schema

- [ ] 1.1 撰寫 pg Store 往返失敗測試（Red）：對 ephemeral PostgreSQL 驗證 ① 的 `Store` 介面各方法（changes／artifacts／canonical specs／discussions／archive／workflow config／language ＋ meta 讀寫對）往返，`artifacts`／`canonical_specs` 的 `version` 每次寫入遞增；測試先紅
- [ ] 1.2 定義 pg schema 與 migration（design D3: pg 參考 Store 文件即真相與 version 樂觀並行）：`changes`／`artifacts(version)`／`canonical_specs(version)`／`discussions`／`archived_changes`／`workflow_config`／`language` ＋治理表 `repos`／`tokens`；文件即真相、不另設狀態表；migration 於空庫套用成功
- [ ] 1.3 實作 TypeScript pg Store 並以 `createEngine({ store: pgStore })` 內嵌 ① 補完的 SDK，建構不拋（必要方法齊全）；1.1 測試轉綠
- [ ] 1.4 驗證（`Headless server serves the verb contract over PostgreSQL` 的儲存底座）：`dispatch(['list','--json'])` 對 pg Store 回正確 `changes[]`、欄位 camelCase

## 2. server 骨架與請求身分

- [ ] 2.1 撰寫請求身分失敗測試（Red）：斷言 `Request identity, API version negotiation, and repo ownership`——缺 `X-Speclink-Api-Version`→`400 api_version_unsupported`、多 repo 缺 `X-Speclink-Repo`→`400 repo_required`；測試先紅
- [ ] 2.2 建 Fastify server 骨架（design D2: Fastify 框架；design D1: 消費 ① 的 SDK，server 為薄層）：版本協商 hook、repo 標頭解析、專案 base path，接上 `createEngine` 做薄層委派 dispatch；2.1 測試轉綠
- [ ] 2.3 實作 repo 歸屬鏈：`GET /changes` 依請求 repo 過濾、change-scoped 動詞 `change.repo`≠request repo→`403 repo_mismatch`；對應測試轉綠
- [ ] 2.4 驗證：版本協商、repo 過濾與 mismatch 測試全綠

## 3. 讀取端點、fs-parity 與 server 運算端點

- [ ] 3.1 撰寫讀取端點失敗測試（Red）：`Headless server serves the verb contract over PostgreSQL`——whoami/config/language/specs/changes/status/artifacts get/instructions/discussions 讀，以及 ① 的 analyze/validate/drift 運算端點，payload 對齊 `docs/verb-contract.md` §5；測試先紅
- [ ] 3.2 實作讀取端點與運算端點（design D1）：委派 `engine.dispatch`（含 analyze/validate/drift 運算）／store 讀取，回 camelCase、artifact 讀帶 `version`、缺件→`404 not_found`；3.1 測試轉綠
- [ ] 3.3 驗證：CLI remote `list`／`status`／`artifact cat`／`analyze` 對真 server 輸出與 fs 模式逐位元一致（複用 `crates/speclink-cli/tests/remote_read_path.rs` 劇本）

## 4. 樂觀並行與寫入端點

- [ ] 4.1 撰寫 If-Match 失敗測試（Red）：`Optimistic concurrency on artifact writes`——PUT artifact 的 If-Match 矩陣（current→200 遞增、stale→`409 version_conflict`、`0`-on-existing→`409`、缺→`428 if_match_required`）與 `POST /changes`；測試先紅
- [ ] 4.2 實作寫入端點（design D6: 並發與生命週期 guard 由 server pg 交易裁決）：PUT artifact 於 pg 交易做 If-Match compare-and-set、`POST /changes`（already_exists/validation_failed）；4.1 測試轉綠
- [ ] 4.3 撰寫並發測試並實作（Red→Green）：兩並發 PUT 同 artifact 恰一成功、敗者 `409 version_conflict`
- [ ] 4.4 驗證：`crates/speclink-cli/tests/remote_write_path.rs` 劇本對真 server 全綠

## 5. 生命週期裁決

- [ ] 5.1 撰寫生命週期失敗測試（Red）：`Change lifecycle adjudication`——並發 claim 恰一勝（敗者 `409 ownership_lost`）、非 owner 寫 `409 ownership_lost`、archive 未完成任務 `409 tasks_incomplete`、archived change 寫 `409 change_busy`（lifecycle:archived）；測試先紅
- [ ] 5.2 實作裁決（design D6）：claim 原子 CAS、applying owner-only 寫、archive check-all-then-apply（tasks_incomplete/version_conflict conflicts[]/gate_pending）、archived 寫拒；委派 dispatch 做實際變動；5.1 測試轉綠
- [ ] 5.3 實作 `task done` 端點（owner-only、applying-only、alreadyDone），委派 dispatch；對應測試轉綠
- [ ] 5.4 驗證：lifecycle 測試組全綠，archive 端點成功併入 delta 並回 `specs[]` 含 version

## 6. 錯誤 reason 目錄一致性

- [ ] 6.1 撰寫 reason 目錄一致性測試套（Red）：`Error reason envelope aligned with the contract catalog`——鏡射 `docs/verb-contract.md` §4，每個事實情境映射到對應 reason 與 HTTP 碼；測試先紅
- [ ] 6.2 實作集中錯誤信封層：非 2xx 皆 `{ reason, message, ...context }`、409 恆帶 reason、`422 validation_failed`（errors[]）；6.1 測試轉綠
- [ ] 6.3 驗證：reason 一致性套全綠，CLI 對各 reason 顯示單行語意訊息（對照 `translate_status`）

## 7. 認證與 admin 管理

- [ ] 7.1 撰寫認證失敗測試（Red）：`PAT authentication and admin management`——無 token→`401 token_missing`、未知→`401 token_invalid`、`GET /whoami` 回 user＋repos；admin 建/列/撤 token、註冊/列 repos、非 admin 被拒、env 種入 admin token；測試先紅
- [ ] 7.2 實作 Bearer PAT 驗證與 whoami（design D4: 認證與 admin 管理開箱堪用）：401 四 reason、`GET /whoami` 回身分與 repos；對應測試轉綠
- [ ] 7.3 實作 admin REST 端點與 bootstrap：admin token/repos 管理、`SPECLINK_ADMIN_TOKEN` 種入；套用 sharp-edges audit checklist（權限預設安全、空 token／空 repo 安全）；對應測試轉綠
- [ ] 7.4 驗證：認證與 admin 測試綠，`speclink auth login`＋`auth status` 對真 server 通過

## 8. SSE 推播與宣告

- [ ] 8.1 撰寫 SSE 失敗測試（Red）：`SSE live invalidation via LISTEN/NOTIFY`——文件變動後該 repo 訂閱者收 `{ type:"invalidate", scope }`、無 PAT 連線 `401`、事件不攜文件內容、whoami/config 帶 `events:{url,transport:"sse"}`；測試先紅
- [ ] 8.2 實作 SSE 端點＋pg LISTEN/NOTIFY 與宣告欄（design D5: SSE server 加 LISTEN/NOTIFY 並宣告 transport）：寫入時 NOTIFY、server LISTEN 後向 scope 廣播、metadata 宣告 `transport:"sse"`；8.1 測試轉綠
- [ ] 8.3 驗證：SSE 測試綠，手動以兩 client 驗一方寫入另一方即收事件、宣告欄可被發現

## 9. Docker 部署

- [ ] 9.1 撰寫 docker-compose（server＋PostgreSQL）與首次啟動 migration：交付 `One-command Docker deployment`
- [ ] 9.2 驗證：fresh host `docker-compose up` 後 server 監聽、pg schema 已 migrate，`speclink link`＋`auth login` 後完整 propose→apply→archive 對真 server 成功

## 10. 定位文件

- [ ] 10.1 更新 `docs/team-mode.md` 與 `README`：交付 `Team-mode positioning documentation`——明載檔案模式適用/限制、防繞過強保證唯遠端 server 可達、何時轉遠端模式
- [ ] 10.2 驗證：`docs/team-mode.md` 內容審視確認兩模式對照與轉換時機齊備

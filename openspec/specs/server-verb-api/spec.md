# server-verb-api Specification

## Purpose

動詞在 server 側的 HTTP 端點面：validate 與 analyze 作為唯讀衍生查詢、DELETE change 走 discard 全語意、任務搬移與其重編號效果、討論寫入端點，以及開工標記的加入與成鏡像的移除、archive 與品質站工單讀取的完整結果回填。本 capability 保證寫入動詞限定於 editor 以上角色，且單一 change 的讀取回應攜帶 show 所需的組合欄位——remote 端不必靠多次請求拼湊。

## Requirements

### Requirement: validate 與 analyze 為唯讀衍生查詢端點

server SHALL 提供 GET /changes/{name}/validate 與 GET /changes/{name}/analyze：經 Command gateway 執行與本地相同的引擎運算（validate 固定 spec_driven schema、單 change；analyze 回完整 AnalyzeReport），回應為 typed DTO 附 scope ETag。兩端點 SHALL 對 reader 與 editor 皆可用、SHALL NOT 產生任何寫入或事件發布；change 不存在時 SHALL 回 404 與語義化訊息。同一 scope 內容下，端點回傳的驗證錯誤集合與 analyze findings SHALL 與本地 fs 模式對同一內容的結果一致。

#### Scenario: validate 結果與本地一致

- **WHEN** 對 server 上一個 proposal 缺 Why 段的 change 呼叫 GET /changes/{name}/validate
- **THEN** 回應列出與 fs 模式 speclink validate 同一 change 內容時相同的錯誤項，HTTP 200 且 scope revision 不前進

#### Scenario: reader 可執行唯讀動詞

- **WHEN** 以 reader role 的成員憑證呼叫 GET /changes/{name}/analyze
- **THEN** 回應為完整 AnalyzeReport（HTTP 200），不因 role 被拒

#### Scenario: 缺席 change 回 404

- **WHEN** 呼叫 GET /changes/no-such/validate
- **THEN** HTTP 404，body 含語義化訊息指出該 change 不存在

---

<!-- @trace
source: remote-verb-parity
updated: 2026-07-23
code:
  - apps/desktop/core/src/manage.rs
  - apps/desktop/src-tauri/src/lib.rs
  - apps/desktop/src-tauri/src/remote.rs
  - apps/desktop/src-tauri/tests/remote_data.rs
  - apps/desktop/src/__tests__/remoteDataSource.test.ts
  - apps/desktop/src/adapter/remoteDataSource.ts
  - crates/speclink-cli/src/commands.rs
  - crates/speclink-cli/src/remote_commands.rs
  - crates/speclink-cli/tests/remote_verb_parity.rs
  - crates/speclink-core/src/command/mod.rs
  - crates/speclink-core/src/tasks.rs
  - crates/speclink-host/src/bridge.rs
  - crates/speclink-host/src/commit.rs
  - crates/speclink-protocol/src/binding.rs
  - crates/speclink-protocol/src/command.rs
  - crates/speclink-protocol/src/query.rs
  - crates/speclink-remote/src/client.rs
  - crates/speclink-remote/src/convert.rs
  - crates/speclink-remote/src/lib.rs
  - crates/speclink-server/src/app.rs
  - crates/speclink-server/src/auth.rs
  - crates/speclink-server/src/events.rs
  - crates/speclink-server/src/routes.rs
  - crates/speclink-server/tests/verb_api.rs
  - docs/verb-contract.md
  - docs/verb-contract.zh-TW.md
-->

---
### Requirement: DELETE change 為 discard 全語意

server SHALL 提供 DELETE /changes/{name}（force 布林 query 參數、預設 false）：執行 fail-closed meta 檢查、started-work guard、來源討論 unlink、change 全部文件的原子刪除與 touched 記錄清理。force=false 且該 change 帶開工痕跡（started_at 已蓋或任一任務已勾）時 SHALL 拒絕且無任何寫入，錯誤 reason SHALL 機器可判為需要 force 的拒絕；meta 損壞時 SHALL 拒絕（含 force=true）。刪除成功 SHALL 於同一提交單元發布事件使訂閱端收到 invalidate。

#### Scenario: 未開工 change 直接刪除

- **WHEN** 對無開工痕跡的 change 呼叫 DELETE /changes/{name}（force=false）
- **THEN** 該 change 的 meta 與全部 artifacts 自 scope 消失、後續 list 不含它，SSE 訂閱端收到 invalidate

#### Scenario: 已開工 change 需要 force

- **WHEN** 對已勾選任務的 change 呼叫 DELETE /changes/{name}（force=false）
- **THEN** 回拒絕錯誤且 reason 機器可判為需要 force，scope 內容零改動；改以 force=true 重呼叫則刪除成功

#### Scenario: 刪除連帶 unlink 來源討論

- **WHEN** 刪除一個由討論 promote 而來的 change
- **THEN** 該討論的 promoted_to 清單移除此 change 名（清單空時討論狀態回復），與刪除同次操作完成

---

<!-- @trace
source: remote-verb-parity
updated: 2026-07-23
code:
  - apps/desktop/core/src/manage.rs
  - apps/desktop/src-tauri/src/lib.rs
  - apps/desktop/src-tauri/src/remote.rs
  - apps/desktop/src-tauri/tests/remote_data.rs
  - apps/desktop/src/__tests__/remoteDataSource.test.ts
  - apps/desktop/src/adapter/remoteDataSource.ts
  - crates/speclink-cli/src/commands.rs
  - crates/speclink-cli/src/remote_commands.rs
  - crates/speclink-cli/tests/remote_verb_parity.rs
  - crates/speclink-core/src/command/mod.rs
  - crates/speclink-core/src/tasks.rs
  - crates/speclink-host/src/bridge.rs
  - crates/speclink-host/src/commit.rs
  - crates/speclink-protocol/src/binding.rs
  - crates/speclink-protocol/src/command.rs
  - crates/speclink-protocol/src/query.rs
  - crates/speclink-remote/src/client.rs
  - crates/speclink-remote/src/convert.rs
  - crates/speclink-remote/src/lib.rs
  - crates/speclink-server/src/app.rs
  - crates/speclink-server/src/auth.rs
  - crates/speclink-server/src/events.rs
  - crates/speclink-server/src/routes.rs
  - crates/speclink-server/tests/verb_api.rs
  - docs/verb-contract.md
  - docs/verb-contract.zh-TW.md
-->

---
### Requirement: 任務搬移端點與重編號效果

server SHALL 提供 POST /changes/{name}/tasks/move（from、to 為 1-based checkbox ordinal，before 為可省略側別）：僅搬移 checkbox 行本身（群組標題與其他行不動），省略 before 時依方向推斷（向上插錨前、向下插錨後），成功後重算「數字.數字」編號前綴並一次寫回，效果與本地任務拖排逐位元一致。from/to 越界或該 change 無 tasks.md 時 SHALL 拒絕且無任何寫入。搬移成功 SHALL 發布事件使訂閱端收到 invalidate。

#### Scenario: 跨群組搬移重編號

- **WHEN** 對 tasks.md 含兩個編號群組的 change 呼叫 move 把第 1 個任務移到第 3 個任務之後
- **THEN** tasks.md 的該 checkbox 行落於錨行之後、兩群組的「數字.數字」前綴依新序重算，其餘行逐字元不變

#### Scenario: 越界拒絕零副作用

- **WHEN** 對只有 3 個任務的 change 呼叫 move（from=5）
- **THEN** 回拒絕錯誤指出索引超界，tasks.md 內容與 scope revision 皆不變

---

<!-- @trace
source: remote-verb-parity
updated: 2026-07-23
code:
  - apps/desktop/core/src/manage.rs
  - apps/desktop/src-tauri/src/lib.rs
  - apps/desktop/src-tauri/src/remote.rs
  - apps/desktop/src-tauri/tests/remote_data.rs
  - apps/desktop/src/__tests__/remoteDataSource.test.ts
  - apps/desktop/src/adapter/remoteDataSource.ts
  - crates/speclink-cli/src/commands.rs
  - crates/speclink-cli/src/remote_commands.rs
  - crates/speclink-cli/tests/remote_verb_parity.rs
  - crates/speclink-core/src/command/mod.rs
  - crates/speclink-core/src/tasks.rs
  - crates/speclink-host/src/bridge.rs
  - crates/speclink-host/src/commit.rs
  - crates/speclink-protocol/src/binding.rs
  - crates/speclink-protocol/src/command.rs
  - crates/speclink-protocol/src/query.rs
  - crates/speclink-remote/src/client.rs
  - crates/speclink-remote/src/convert.rs
  - crates/speclink-remote/src/lib.rs
  - crates/speclink-server/src/app.rs
  - crates/speclink-server/src/auth.rs
  - crates/speclink-server/src/events.rs
  - crates/speclink-server/src/routes.rs
  - crates/speclink-server/tests/verb_api.rs
  - docs/verb-contract.md
  - docs/verb-contract.zh-TW.md
-->

---
### Requirement: 寫入動詞 editor 限定

DELETE /changes/{name} 與 POST /changes/{name}/tasks/move SHALL 檢查呼叫者 membership role：reader SHALL 收 403 且 reason 機器可判為權限不足、scope 零改動；editor 放行。capability 描述 SHALL 對 reader 將刪除與任務拖排標示為停用，validate/analyze 對全 role 標示可用。

#### Scenario: reader 的刪除被拒

- **WHEN** 以 reader role 憑證呼叫 DELETE /changes/{name}
- **THEN** HTTP 403、reason 機器可判為權限不足，該 change 完整保留

#### Scenario: capability 依 role 呈現

- **WHEN** reader 與 editor 各自完成 handshake 取得 capability 描述
- **THEN** reader 的描述中刪除與任務拖排為停用、validate/analyze 為可用；editor 的四項皆可用

<!-- @trace
source: remote-verb-parity
updated: 2026-07-23
code:
  - apps/desktop/core/src/manage.rs
  - apps/desktop/src-tauri/src/lib.rs
  - apps/desktop/src-tauri/src/remote.rs
  - apps/desktop/src-tauri/tests/remote_data.rs
  - apps/desktop/src/__tests__/remoteDataSource.test.ts
  - apps/desktop/src/adapter/remoteDataSource.ts
  - crates/speclink-cli/src/commands.rs
  - crates/speclink-cli/src/remote_commands.rs
  - crates/speclink-cli/tests/remote_verb_parity.rs
  - crates/speclink-core/src/command/mod.rs
  - crates/speclink-core/src/tasks.rs
  - crates/speclink-host/src/bridge.rs
  - crates/speclink-host/src/commit.rs
  - crates/speclink-protocol/src/binding.rs
  - crates/speclink-protocol/src/command.rs
  - crates/speclink-protocol/src/query.rs
  - crates/speclink-remote/src/client.rs
  - crates/speclink-remote/src/convert.rs
  - crates/speclink-remote/src/lib.rs
  - crates/speclink-server/src/app.rs
  - crates/speclink-server/src/auth.rs
  - crates/speclink-server/src/events.rs
  - crates/speclink-server/src/routes.rs
  - crates/speclink-server/tests/verb_api.rs
  - docs/verb-contract.md
  - docs/verb-contract.zh-TW.md
-->

---
### Requirement: 討論寫入動詞端點補齊

server SHALL 補齊討論的寫入動詞端點：POST /discussions 的請求 SHALL 接受選填 slug 欄位並轉傳引擎（未帶時行為與現行完全相同，非法值由引擎拒絕並映射為語義化錯誤、不落檔）；DELETE /discussions/{slug} SHALL 以 force query 參數直通引擎 discard（0 輪即刪、有輪無 force 拒絕），並比照 change 刪除做 editor 限定（reader 收 403、scope 零改動）；POST /discussions/{slug}/link 與 POST /discussions/{slug}/seal SHALL 以 body 攜帶 change 名稱直通對應引擎命令。四者皆為 unit of work：成功寫入時 scope revision 前進、事件照引擎 outcome 發布；討論或 change 不存在時 SHALL 回 404 與語義化訊息。

#### Scenario: 建立討論轉傳 slug

- **WHEN** 以合法 slug 欄位呼叫 POST /discussions（topic 為中文）
- **THEN** HTTP 200，回應的 slug 為覆寫值、topic 為原文，server store 以該 slug 建檔；非法 slug 時回語義化錯誤且不落檔

#### Scenario: 討論 discard 的 guard 經端點生效

- **WHEN** 對 0 輪討論呼叫 DELETE /discussions/{slug}，再對有輪討論呼叫同端點（無 force）
- **THEN** 前者刪除成功、scope revision 前進；後者被拒且記錄完整保留、revision 不前進；帶 force=true 重呼叫則刪除成功

#### Scenario: reader 的討論刪除被拒

- **WHEN** 以 reader role 憑證呼叫 DELETE /discussions/{slug}
- **THEN** HTTP 403、reason 機器可判為權限不足，該討論完整保留

#### Scenario: link 與 seal 直通引擎

- **WHEN** 依序呼叫 POST /discussions/{slug}/link 與 POST /discussions/{slug}/seal，body 帶既有 change 名稱
- **THEN** 兩者 HTTP 200：link 後 change meta 的 from_discussion 鏈含該 slug，seal 後討論標記 promoted；不存在的討論或 change 回 404 與語義化訊息

<!-- @trace
source: remote-cli-parity
updated: 2026-07-31
code:
  - apps/desktop/src-tauri/src/remote.rs
  - apps/desktop/src-tauri/tests/remote_data.rs
  - apps/desktop/src/__tests__/remoteDataSource.test.ts
  - crates/speclink-cli/src/commands.rs
  - crates/speclink-cli/src/remote_commands.rs
  - crates/speclink-cli/tests/discuss_slug.rs
  - crates/speclink-cli/tests/remote_verb_parity.rs
  - crates/speclink-cli/tests/remote_write_path.rs
  - crates/speclink-protocol/src/command.rs
  - crates/speclink-protocol/src/query.rs
  - crates/speclink-remote/src/client.rs
  - crates/speclink-remote/tests/typed_client.rs
  - crates/speclink-server/src/app.rs
  - crates/speclink-server/src/routes.rs
  - crates/speclink-server/tests/backup_e2e.rs
  - crates/speclink-server/tests/discussion_routes.rs
  - crates/speclink-server/tests/e2e_cli.rs
  - crates/speclink-server/tests/query_routes.rs
  - crates/speclink-server/tests/verb_api.rs
  - docs/verb-contract.md
  - docs/verb-contract.zh-TW.md
-->

---
### Requirement: 單 change 讀取回應攜帶 show 組合欄位

GET /changes/{name} 的回應 SHALL 攜帶七個選填欄位供 client 端 show 讀取與詳情呈現組合（皆 camelCase、serde default）：created SHALL 僅於該 change meta 同時具有 schema 與 created 時出現（引擎 ShowChange 的成對回報規則）；fromDiscussions SHALL 為 meta 的 from_discussion 鏈（空清單即省略）；deltaCapabilities SHALL 為該 change 的 delta spec capability 名單（空清單即省略）；createdBy、createdWith、startedAt、startedBy SHALL 各為 meta 的 created_by、created_with、started_at、started_by（缺席即省略）。七欄皆由 server 於既有路由自 meta 與 scope 文件組裝；舊 server 不送這些欄位時，client 的對應區塊 SHALL 維持缺席、SHALL NOT 偽造預設值。

#### Scenario: show 組合欄位隨單 change 讀取上 wire

- **WHEN** 對 meta 含 schema、created 與 from_discussion，且帶有一個 delta spec 的 change 呼叫 GET /changes/{name}
- **THEN** 回應含 created（等於 meta 的 created）、fromDiscussions（含該討論 slug）與 deltaCapabilities（含該 capability）；對 meta 無 created 且無鏈、無 delta spec 的 change，三個鍵皆不出現於回應

#### Scenario: meta 歸屬欄位隨單 change 讀取上 wire

- **WHEN** 對 meta 含 created_by、created_with 且已蓋開工章（started_at 與 started_by）的 change 呼叫 GET /changes/{name}，再對四欄皆缺的 change 呼叫同端點
- **THEN** 前者回應含 createdBy、createdWith、startedAt、startedBy 四鍵且值等於 meta；後者四鍵皆不出現

##### Example: 欄位組裝

| change meta 與文件 | GET /changes/{name} 額外欄位 |
| ------------------ | ---------------------------- |
| `schema: spec-driven`＋`created: 2026-07-29`＋`from_discussion: auth-scope`＋`specs/auth/spec.md` | `"created":"2026-07-29","fromDiscussions":["auth-scope"],"deltaCapabilities":["auth"]` |
| 上列再加 `created_by: Demo <d@e.com>`＋`started_at: 2026-08-25T00:00:00Z`＋`started_by: Demo <d@e.com>` | 額外再含 `"createdBy":"Demo <d@e.com>","startedAt":"2026-08-25T00:00:00Z","startedBy":"Demo <d@e.com>"` |
| `schema: spec-driven`（無 created、無鏈、無 delta spec、無歸屬欄位） | （七鍵皆缺席） |


<!-- @trace
source: remote-read-parity
updated: 2026-08-26
-->

---
### Requirement: 變更開工標記端點

server SHALL 提供 POST /changes/{name}/in-progress，經 Command gateway 直通引擎的開工標記命令：change 存在且未開工時以呼叫者認證身分蓋 started_at 與 started_by 進 meta、發布領域事件、scope revision 前進；change 不存在或已開工時 SHALL 維持引擎的靜默成功語意——HTTP 200、零文件寫入、零事件、revision 不前進。

#### Scenario: 首次蓋章寫入與事件

- **WHEN** 對未開工的 change 呼叫 POST /changes/{name}/in-progress
- **THEN** HTTP 200，meta 新增 started_at 與 started_by（呼叫者認證身分）、既有欄位逐字元保留，事件發布且 revision 前進

#### Scenario: 重複與未知名稱皆靜默成功

- **WHEN** 對已開工的 change 與不存在的 change 名稱各呼叫一次該端點
- **THEN** 兩者皆 HTTP 200，server 零文件寫入、零事件、revision 不前進，已開工者的首章逐字元保留

<!-- @trace
source: remote-cli-parity
updated: 2026-07-31
code:
  - apps/desktop/src-tauri/src/remote.rs
  - apps/desktop/src-tauri/tests/remote_data.rs
  - apps/desktop/src/__tests__/remoteDataSource.test.ts
  - crates/speclink-cli/src/commands.rs
  - crates/speclink-cli/src/remote_commands.rs
  - crates/speclink-cli/tests/discuss_slug.rs
  - crates/speclink-cli/tests/remote_verb_parity.rs
  - crates/speclink-cli/tests/remote_write_path.rs
  - crates/speclink-protocol/src/command.rs
  - crates/speclink-protocol/src/query.rs
  - crates/speclink-remote/src/client.rs
  - crates/speclink-remote/tests/typed_client.rs
  - crates/speclink-server/src/app.rs
  - crates/speclink-server/src/routes.rs
  - crates/speclink-server/tests/backup_e2e.rs
  - crates/speclink-server/tests/discussion_routes.rs
  - crates/speclink-server/tests/e2e_cli.rs
  - crates/speclink-server/tests/query_routes.rs
  - crates/speclink-server/tests/verb_api.rs
  - docs/verb-contract.md
  - docs/verb-contract.zh-TW.md
-->

---
### Requirement: in-progress 標記移除端點與加入端點成鏡像

server SHALL 提供 DELETE /changes/{name}/in-progress 端點,與既有 POST /changes/{name}/in-progress 同資源、反向語意,並循既有動詞端點的認證與 binding 規則。守門 SHALL 由引擎同一裁決點執行,與本地 CLI 行為一致:零工作痕跡(已勾任務數為 0 且 touched 記錄兩清單皆空)時移除 started_* 三欄位,HTTP 200 回 Ack 並發佈變更退回事件(事件經 SSE 以 invalidation hint 流動);對未開工的 change SHALL 冪等成功——HTTP 200、零寫入、不 commit、不發事件。有工作痕跡時 SHALL 回 HTTP 409,error payload SHALL 含 camelCase 證據欄位:checkedTasks(數字,已勾任務數)與 touchedFiles(字串陣列,touched 記錄檔案清單聯集去重);不存在的 change SHALL 回 HTTP 404。既有 POST /changes/{name}/in-progress 端點的行為與回應 SHALL 不變。

#### Scenario: 零痕跡變更移除成功並發事件

- **WHEN** 認證通過的呼叫者對一個零工作痕跡的進行中 change 發 DELETE /changes/{name}/in-progress
- **THEN** HTTP 200 回 Ack,該 change 的 started_* 欄位消失且其餘 meta 內容不變,SSE 串流出現對應的 invalidation hint 事件

#### Scenario: 未開工變更冪等成功且不發事件

- **WHEN** 對一個從未開工的 change 發 DELETE /changes/{name}/in-progress
- **THEN** HTTP 200 回 Ack,無任何寫入與 commit,SSE 串流不出現新事件

#### Scenario: 有工作痕跡時回 409 與結構化證據

- **WHEN** 對一個已勾 2 個任務且 touched 記錄含 src/a.rs 的 change 發 DELETE /changes/{name}/in-progress
- **THEN** HTTP 409,error payload 的 checkedTasks 為 2、touchedFiles 為 ["src/a.rs"],該 change 的 meta 與 touched 記錄皆不變

##### Example: 證據欄位形狀

- **GIVEN** 一個 change 已勾任務數 3,touched 記錄檔案清單為 src/x.rs 與 docs/y.md
- **WHEN** 對其發 DELETE /changes/{name}/in-progress
- **THEN** 409 的 error payload 含 "checkedTasks": 3 與 "touchedFiles": ["src/x.rs", "docs/y.md"]

#### Scenario: 不存在的 change 回 404

- **WHEN** 對不存在的 change 名稱發 DELETE /changes/{name}/in-progress
- **THEN** HTTP 404,無任何寫入

<!-- @trace
source: revert-in-progress-to-proposed
updated: 2026-07-31
code:
  - apps/desktop/core/src/manage.rs
  - apps/desktop/src-tauri/src/lib.rs
  - apps/desktop/src-tauri/src/remote.rs
  - apps/desktop/src-tauri/tests/remote_data.rs
  - apps/desktop/src-tauri/tests/remote_runtime.rs
  - apps/desktop/src/App.tsx
  - apps/desktop/src/__tests__/App.test.tsx
  - apps/desktop/src/__tests__/remoteDataSource.test.ts
  - apps/desktop/src/__tests__/tauriDataSource.test.ts
  - apps/desktop/src/adapter/remoteDataSource.ts
  - apps/desktop/src/adapter/tauriDataSource.ts
  - apps/desktop/src/i18n/messages.ts
  - apps/desktop/src/store.ts
  - crates/speclink-cli/src/commands.rs
  - crates/speclink-cli/src/main.rs
  - crates/speclink-cli/src/remote_commands.rs
  - crates/speclink-cli/tests/in_progress_remove.rs
  - crates/speclink-cli/tests/remote_write_path.rs
  - crates/speclink-core/assets/skills/apply.md
  - crates/speclink-core/src/command/mod.rs
  - crates/speclink-core/src/inprogress.rs
  - crates/speclink-core/tests/golden/claude.snapshot.md
  - crates/speclink-core/tests/golden/codex.snapshot.md
  - crates/speclink-core/tests/golden/neutral-cli.snapshot.md
  - crates/speclink-core/tests/golden/neutral-tool-call.snapshot.md
  - crates/speclink-host/src/bridge.rs
  - crates/speclink-host/src/commit.rs
  - crates/speclink-protocol/src/command.rs
  - crates/speclink-protocol/src/error.rs
  - crates/speclink-remote/src/client.rs
  - crates/speclink-remote/src/device.rs
  - crates/speclink-remote/src/events.rs
  - crates/speclink-remote/src/lib.rs
  - crates/speclink-remote/tests/reauth_retry.rs
  - crates/speclink-remote/tests/typed_client.rs
  - crates/speclink-server/src/app.rs
  - crates/speclink-server/src/error.rs
  - crates/speclink-server/src/events.rs
  - crates/speclink-server/src/routes.rs
  - crates/speclink-server/tests/verb_api.rs
  - packages/ui/src/__tests__/discussionColumn.test.tsx
  - packages/ui/src/__tests__/discussionDrawer.test.tsx
  - packages/ui/src/__tests__/kanban.test.tsx
  - packages/ui/src/__tests__/revertBlockedDialog.test.tsx
  - packages/ui/src/__tests__/richDrawer.test.tsx
  - packages/ui/src/adapter.ts
  - packages/ui/src/components/ChangeCard.tsx
  - packages/ui/src/components/DiscussionColumn.tsx
  - packages/ui/src/components/DiscussionDrawer.tsx
  - packages/ui/src/components/KanbanBoard.tsx
  - packages/ui/src/components/RevertBlockedDialog.tsx
  - packages/ui/src/components/RichDetailDrawer.tsx
  - packages/ui/src/i18n.tsx
  - packages/ui/src/index.ts
-->

---
### Requirement: archive 與工單讀取端點回填完整結果

封存端點 SHALL 自引擎的封存結果回填擴充欄位：datedName、各 capability 的 added／modified／removed／renamed 計數、snapshotCreated、archivedDiscussions（封存時一併封存的來源討論清單——僅含 Conclusion 已寫入而實際隨行封存者）、evidenceRecorded。review 與 verify 的工單讀取端點 SHALL 回填 content 為 store 中工單文件的原文全文。討論結論端點 SHALL 回填 restaleFlagged 為引擎打回重收的變更名清單，且於引擎順手封存討論（閉環觸發）時 SHALL 回填 autoArchived: true（camelCase 布林、未觸發時省略鍵）。討論轉出端點 SHALL NOT 回傳新變更的目錄位置——那是 store 端的檔案系統位置，對呼叫端無意義。上列端點的既有欄位、狀態碼與錯誤語意 SHALL 維持不變。

#### Scenario: 封存端點回填完整結果

- **WHEN** 對一筆就緒的變更呼叫封存端點且封存改動了規格
- **THEN** 200 回應含 datedName 與各 capability 的四項計數，其值與引擎在 store 中實際落地的封存效果一致

#### Scenario: 封存端點回填來源討論與證據旗標

- **WHEN** 封存一筆帶來源討論（該討論的衍生變更僅剩此筆）且無任務證據的變更
- **THEN** 回應的 archivedDiscussions 含該討論的 slug 與封存檔名，evidenceRecorded 為 false

#### Scenario: 工單讀取端點回填原文

- **WHEN** 對一筆存在工單的變更呼叫工單讀取端點（review 或 verify）
- **THEN** 200 回應的 content 等於該站工單文件的原文全文，rounds 與 lastRound 照舊在場

#### Scenario: 結論端點回填被打回的變更

- **WHEN** 對一份已轉出變更的討論呼叫結論端點，且該變更仍在進行中
- **THEN** 200 回應的 restaleFlagged 含該變更名

#### Scenario: 工單不存在時行為不變

- **WHEN** 對一筆無工單的變更呼叫工單讀取端點
- **THEN** 回應維持既有的 404 語意與錯誤形狀，不因欄位擴充而改變

#### Scenario: 討論結論端點回填順手封存事實

- **WHEN** 對閉環條件成立（promoted_to 非空且全數轉出變更已封存）的討論呼叫討論結論端點，再對閉環條件不成立的討論呼叫同端點
- **THEN** 前者 200 回應含 autoArchived: true 且該討論自 live 清單消失、出現於封存清單；後者回應無 autoArchived 鍵、討論維持於 live 清單


<!-- @trace
source: conclusion-gated-discussion-archive
updated: 2026-09-01
-->

---
### Requirement: task done 消費 touchedFiles 且 evidence 有唯讀端點

task done 端點 SHALL 消費請求 payload 的 touchedFiles 並作為 Host 解析的候選交給 Engine，SHALL NOT 丟棄；payload 未攜帶 touchedFiles 時 SHALL 視為無候選（沿無新髒檔語意），SHALL NOT 視為錯誤。server SHALL 提供該 change 的 evidence 唯讀端點：viewer 以上角色可讀，回應欄位為 camelCase 的 evidence 記錄集合；記錄缺席 SHALL 回空集合而非 not_found——缺席是正常狀態，SHALL NOT 讓讀取端以錯誤碼區分「change 存在但無 evidence」。

#### Scenario: task done 攜檔案後 evidence 端點可讀回

- **WHEN** 以 editor 角色對某任務執行 task done 且 payload 攜帶 touchedFiles，隨後以 viewer 角色讀取該 change 的 evidence 端點
- **THEN** 回應含該任務 entry，touchedFiles 與 payload 一致，欄位為 camelCase

#### Scenario: 無 evidence 回空集合

- **WHEN** 對存在但從未落 evidence 的 change 讀取 evidence 端點
- **THEN** 回應為成功的空集合，非 not_found

<!-- @trace
source: remote-task-evidence
updated: 2026-08-25
-->

---
### Requirement: 變更清單回應攜帶建立者與來源討論欄位

GET /changes 的清單項 SHALL 沿既有的逐筆 meta 組裝路徑（startedAt 的同一條）補三個選填欄位：createdBy、created 與 fromDiscussions，語意與 wire 欄位定義一致（缺席或空清單即省略）。meta 解析失敗的 change SHALL 維持既有 metaError 容錯路徑，三欄不出現、清單不失敗。

#### Scenario: 清單項攜帶建立者與來源討論

- **WHEN** scope 內有一個 meta 含 created_by、created 與 from_discussion 的 change 與一個三者皆缺的 change，呼叫 GET /changes
- **THEN** 前者清單項含 createdBy、created 與 fromDiscussions；後者三鍵缺席；回應整體成功

<!-- @trace
source: remote-read-parity
updated: 2026-08-26
-->

---
### Requirement: 討論列表回應攜帶 promotedTo

GET /discussions 的每筆討論 SHALL 由 server 於 route 邊緣以引擎的 promoted_to 查詢函式組裝 promotedTo 欄位（空清單即省略）；引擎的討論列表結構與 CLI 的 discuss list --json 輸出 SHALL 維持逐位元不變。查詢失敗的單筆討論 SHALL 以欄位缺席容錯、列表不失敗。

#### Scenario: 已轉出與未轉出討論的列表欄位

- **WHEN** scope 內有一筆 promoted_to 含兩個 change 名的討論與一筆未轉出的討論，呼叫 GET /discussions
- **THEN** 前者含 promotedTo 且順序沿 frontmatter 累加順序；後者無 promotedTo 鍵；本地 CLI 的 discuss list --json 輸出與改動前逐位元相同

<!-- @trace
source: remote-read-parity
updated: 2026-08-26
-->

---
### Requirement: claim 端點持久化與 ownership 衝突語意

POST /changes/{name}/claim SHALL 經 Command gateway 直通引擎的 Claim 命令（移除回聲 stub）：認領成功時回應攜帶認領後的 claimedBy 且寫入隨 Unit of Work 落盤；同人重複認領回應成功且零寫入；已被他人認領 SHALL 回 HTTP 409、reason 為八值封閉 registry 既有的 refused、message 含目前持有人與建議動作（SHALL NOT 擴充 error reason registry）；change 不存在回 404；本端點 SHALL 比照其他寫入動詞為 editor 限定（reader 收 403、scope 零改動）。變更清單與單 change 讀取回應的 claimedBy SHALL 自 meta 的 claimed_by 組裝（未認領即省略），使認領跨重啟、跨裝置可見。

#### Scenario: 認領落盤且清單可見

- **WHEN** editor 對未認領的 change 呼叫 POST /changes/{name}/claim，隨後呼叫 GET /changes 與 GET /changes/{name}
- **THEN** claim 回應含呼叫者為 claimedBy；兩個讀取回應的該 change 皆含同值 claimedBy；server 重啟後讀取結果不變

#### Scenario: 他人認領衝突與 reader 拒絕

- **WHEN** 另一 editor 對已認領的 change 呼叫同端點，接著一位 reader 對未認領的 change 呼叫同端點
- **THEN** 前者收 409、reason 為 refused、message 含目前持有人、meta 零改動；後者收 403、scope 零改動

<!-- @trace
source: remote-claim-ownership
updated: 2026-08-27
-->

---
### Requirement: 討論列表回應攜帶 concluded

GET /discussions 的每筆討論 SHALL 由 server 於 route 邊緣以引擎的結論查詢組裝 concluded 欄位（camelCase 布林、恆填 true 或 false——true 即該討論的 Conclusion 段已寫入內文，scaffold 佔位註解不算）；引擎的討論列表結構與 CLI 的 discuss list --json 輸出 SHALL 維持逐位元不變。查詢失敗的單筆討論 SHALL 以欄位缺席容錯、列表不失敗。

#### Scenario: 已結論與未結論討論的列表欄位

- **WHEN** scope 內有一筆已寫入結論的 promoted 討論與一筆 Conclusion 仍為佔位註解的 promoted 討論，呼叫 GET /discussions
- **THEN** 前者含 concluded: true、後者含 concluded: false；本地 CLI 的 discuss list --json 輸出與改動前逐位元相同

<!-- @trace
source: conclusion-gated-discussion-archive
updated: 2026-09-01
-->
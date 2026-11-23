# server-event-stream Specification

## Purpose

server 的事件推送：SSE 串流自 transactional outbox 讀取、事件只作為 invalidation hint（提示重查，不搬運資料）、斷線 resume 續傳不漏不重、保留政策與超出保留範圍時的 reset 訊號，以及慢消費者的有界處置。本 capability 保證 client 不會因錯過事件而停在過期畫面，也不會讓一個讀不動的連線拖垮伺服器；transport 由 binding 明確宣告。

## Requirements

### Requirement: SSE 串流自 outbox 且事件是 invalidation hint

server SHALL 提供 project-scoped 的 SSE 事件端點，沿用既有 bearer 前置與 binding 裁決（未認證 401、非成員 403，與其他路由一致）。推播內容 SHALL 出自該 scope 的 outbox 讀取：SSE 事件識別 SHALL 為 outbox 序號，data SHALL 為 protocol 定義的 invalidation DTO——僅攜事件識別、scope 種類、資源識別與 revision，SHALL NOT 承載規格文件內容。寫入 commit 成功後 SHALL 向該 scope 全部訂閱者推播對應事件；對映不了資源類別的領域事件 SHALL 以 unknown 類別照發，SHALL NOT 靜默略過。連線期間 SHALL 以固定間隔送出 SSE 註解心跳。

#### Scenario: 寫入即推播 invalidation

- **WHEN** client A 訂閱 /events 期間 client B 成功執行 task done
- **THEN** client A 收到一筆事件：識別為該筆 outbox 序號、資源識別為該 change、revision 為 commit 後值；data 不含任務文件內容

#### Scenario: 多訂閱者同事件

- **WHEN** 兩個 client 同時訂閱同一 scope 期間發生一筆寫入
- **THEN** 兩者各收到恰一筆相同識別的事件

#### Scenario: 未授權連線拒於門外

- **WHEN** 以無效 token 或非成員身分連線 /events
- **THEN** 回應與其他路由一致的 401/403 三元組；不建立串流

---

<!-- @trace
source: server-sse-events
updated: 2026-07-14
code:
  - Cargo.lock
  - crates/speclink-protocol/src/events.rs
  - crates/speclink-server/Cargo.toml
  - crates/speclink-server/src/app.rs
  - crates/speclink-server/src/auth.rs
  - crates/speclink-server/src/config.rs
  - crates/speclink-server/src/events.rs
  - crates/speclink-server/src/lib.rs
  - crates/speclink-server/src/main.rs
  - crates/speclink-server/src/routes.rs
  - crates/speclink-server/src/state.rs
  - crates/speclink-server/src/verb.rs
  - crates/speclink-server/tests/auth_device.rs
  - crates/speclink-server/tests/auth_pat.rs
  - crates/speclink-server/tests/binding.rs
  - crates/speclink-server/tests/common/mod.rs
  - crates/speclink-server/tests/device_e2e.rs
  - crates/speclink-server/tests/device_flow.rs
  - crates/speclink-server/tests/query_routes.rs
  - crates/speclink-server/tests/refresh_rotation.rs
  - crates/speclink-server/tests/sse_events.rs
  - crates/speclink-server/tests/web_account.rs
  - crates/speclink-server/tests/web_activate.rs
  - crates/speclink-server/tests/web_device_sessions.rs
  - crates/speclink-server/tests/web_invite.rs
-->

---
### Requirement: resume 續傳不漏不重

帶 Last-Event-ID 的重連 SHALL 自該序號之後續傳：重連前後收到的事件序列 SHALL 與不斷線時一致——無缺漏、無重複、序號單調遞增。Last-Event-ID 格式非法 SHALL 視同未帶（自最新事件之後開始）。續傳與即時推播 SHALL 讀自同一 outbox 序列，SHALL NOT 出現兩者順序分叉。

#### Scenario: 斷線期間事件補齊

- **WHEN** client 收到序號 n 的事件後斷線，期間發生三筆寫入，client 以 Last-Event-ID 為 n 重連
- **THEN** client 依序收到 n 之後的三筆事件各恰一次，序號單調遞增

---

<!-- @trace
source: server-sse-events
updated: 2026-07-14
code:
  - Cargo.lock
  - crates/speclink-protocol/src/events.rs
  - crates/speclink-server/Cargo.toml
  - crates/speclink-server/src/app.rs
  - crates/speclink-server/src/auth.rs
  - crates/speclink-server/src/config.rs
  - crates/speclink-server/src/events.rs
  - crates/speclink-server/src/lib.rs
  - crates/speclink-server/src/main.rs
  - crates/speclink-server/src/routes.rs
  - crates/speclink-server/src/state.rs
  - crates/speclink-server/src/verb.rs
  - crates/speclink-server/tests/auth_device.rs
  - crates/speclink-server/tests/auth_pat.rs
  - crates/speclink-server/tests/binding.rs
  - crates/speclink-server/tests/common/mod.rs
  - crates/speclink-server/tests/device_e2e.rs
  - crates/speclink-server/tests/device_flow.rs
  - crates/speclink-server/tests/query_routes.rs
  - crates/speclink-server/tests/refresh_rotation.rs
  - crates/speclink-server/tests/sse_events.rs
  - crates/speclink-server/tests/web_account.rs
  - crates/speclink-server/tests/web_activate.rs
  - crates/speclink-server/tests/web_device_sessions.rs
  - crates/speclink-server/tests/web_invite.rs
-->

---
### Requirement: 保留政策與 reset 訊號

server SHALL 依保留政策 ack outbox（每 scope 保留最近 N 筆，N 可組態、有預設值），讓 driver 得以清理更舊事件。重連的 Last-Event-ID 落在已清理範圍時，server SHALL 立即送出明確的 reset 訊號事件（獨立的 SSE 事件種類）再繼續推播新事件；SHALL NOT 猜測或重建已清理的事件。重連的 Last-Event-ID 大於該 scope 目前最大的序號時（例如還原之後事件序號重新開始），server SHALL 同樣立即送出 reset 訊號，再從目前最大的序號之後繼續推播新事件，SHALL NOT 等到序號超過該 Last-Event-ID 才開始推播。client 據 reset 以 Query 與 ETag 全部重新收斂的路徑 SHALL 可用（既有輪詢端點不受本能力影響）。

#### Scenario: 過期 cursor 得到 reset

- **WHEN** 某 scope 已 ack 清理至序號 m，client 以小於 m 的 Last-Event-ID 重連
- **THEN** client 先收到 reset 訊號事件，隨後的新寫入照常推播；server 未嘗試重送已清理序號

#### Scenario: 漏光事件仍能收斂

- **WHEN** client 收到 reset 後改以既有輪詢端點與查詢路由重讀
- **THEN** 重讀結果反映全部已 commit 的變更；後續可重新訂閱自最新序號續聽

#### Scenario: Last-Event-ID 比 server 的序號新

- **WHEN** 某 scope 目前最大的事件序號是 3，client 以 Last-Event-ID 為 500 重連，之後發生一筆寫入
- **THEN** client 先收到 reset 訊號，再收到序號 4 的事件


<!-- @trace
source: server-on-port-examples
updated: 2026-11-16T23:24:33+08:00
-->

---
### Requirement: 慢消費者有界處置

每條 SSE 連線的送出緩衝 SHALL 有界（可組態、有預設值）；緩衝溢出 SHALL 使 server 斷開該連線，SHALL NOT 為單一訂閱者無界堆積記憶體，且 SHALL NOT 影響其他訂閱者收事件。被斷開的 client 以 Last-Event-ID 重連 SHALL 依 resume 與 reset 規則恢復。

#### Scenario: 慢消費者不拖累他人

- **WHEN** 兩個訂閱者之一停止讀取直到其緩衝溢出，期間持續發生寫入
- **THEN** 停讀者被斷線；另一訂閱者持續即時收到全部事件無缺漏

---

<!-- @trace
source: server-sse-events
updated: 2026-07-14
code:
  - Cargo.lock
  - crates/speclink-protocol/src/events.rs
  - crates/speclink-server/Cargo.toml
  - crates/speclink-server/src/app.rs
  - crates/speclink-server/src/auth.rs
  - crates/speclink-server/src/config.rs
  - crates/speclink-server/src/events.rs
  - crates/speclink-server/src/lib.rs
  - crates/speclink-server/src/main.rs
  - crates/speclink-server/src/routes.rs
  - crates/speclink-server/src/state.rs
  - crates/speclink-server/src/verb.rs
  - crates/speclink-server/tests/auth_device.rs
  - crates/speclink-server/tests/auth_pat.rs
  - crates/speclink-server/tests/binding.rs
  - crates/speclink-server/tests/common/mod.rs
  - crates/speclink-server/tests/device_e2e.rs
  - crates/speclink-server/tests/device_flow.rs
  - crates/speclink-server/tests/query_routes.rs
  - crates/speclink-server/tests/refresh_rotation.rs
  - crates/speclink-server/tests/sse_events.rs
  - crates/speclink-server/tests/web_account.rs
  - crates/speclink-server/tests/web_activate.rs
  - crates/speclink-server/tests/web_device_sessions.rs
  - crates/speclink-server/tests/web_invite.rs
-->

---
### Requirement: binding 宣告 sse transport

/binding 的 capabilities SHALL 宣告 events transports 含 sse（含事件端點 url 與 resume 支援為真），polling 宣告 SHALL 維持不變。宣告 SHALL 與實際服務一致：宣告的 url 即本能力的事件端點。invalidation DTO SHALL 定義於 protocol crate（camelCase、可匯出 JSON Schema、序列化往返穩定）。

#### Scenario: 宣告與服務一致

- **WHEN** 完成 handshake 取得 capabilities 後對宣告的 sse url 建立訂閱
- **THEN** 宣告含 type 為 sse、resume 為 true 的 transport 與不變的 polling 宣告；該 url 的訂閱可收到 invalidation 事件

<!-- @trace
source: server-sse-events
updated: 2026-07-14
code:
  - Cargo.lock
  - crates/speclink-protocol/src/events.rs
  - crates/speclink-server/Cargo.toml
  - crates/speclink-server/src/app.rs
  - crates/speclink-server/src/auth.rs
  - crates/speclink-server/src/config.rs
  - crates/speclink-server/src/events.rs
  - crates/speclink-server/src/lib.rs
  - crates/speclink-server/src/main.rs
  - crates/speclink-server/src/routes.rs
  - crates/speclink-server/src/state.rs
  - crates/speclink-server/src/verb.rs
  - crates/speclink-server/tests/auth_device.rs
  - crates/speclink-server/tests/auth_pat.rs
  - crates/speclink-server/tests/binding.rs
  - crates/speclink-server/tests/common/mod.rs
  - crates/speclink-server/tests/device_e2e.rs
  - crates/speclink-server/tests/device_flow.rs
  - crates/speclink-server/tests/query_routes.rs
  - crates/speclink-server/tests/refresh_rotation.rs
  - crates/speclink-server/tests/sse_events.rs
  - crates/speclink-server/tests/web_account.rs
  - crates/speclink-server/tests/web_activate.rs
  - crates/speclink-server/tests/web_device_sessions.rs
  - crates/speclink-server/tests/web_invite.rs
-->

---
### Requirement: 審查與驗證事件歸到變更

事件串流 SHALL 把審查與驗證的六種領域事件（兩個關卡各自的新增一輪、蓋章、放棄）以及落地回報的事件對映為「變更」類別，資源識別 SHALL 是該變更的名稱，SHALL NOT 以 unknown 類別發出。事件名稱到資源類別的對照 SHALL 只有一份定義，由 Host 提供，示範用 Server 與自行提供推送的宿主 SHALL 共用它。其他對映不了的事件維持以 unknown 類別照發。

#### Scenario: 蓋章事件歸到該變更

- **WHEN** 訂閱者連上事件串流後，另一個請求對變更 demo 完成審查蓋章
- **THEN** 訂閱者收到一筆 invalidation，scope 為 change，resourceId 為 demo

#### Scenario: 新增一輪與放棄同樣歸類

- **WHEN** 對 demo 新增一輪驗證工單，之後放棄驗證
- **THEN** 訂閱者收到兩筆 invalidation，scope 都是 change，resourceId 都是 demo

#### Scenario: 落地事件歸到該變更

- **WHEN** 對 demo 回報落地
- **THEN** 訂閱者收到 scope 為 change、resourceId 為 demo 的 invalidation

#### Scenario: 其他未知事件維持 unknown

- **WHEN** 看板順序被寫入
- **THEN** 訂閱者收到 scope 為 unknown 的 invalidation

<!-- @trace
source: station-status-and-landing
updated: 2026-11-13T09:30:51+08:00
-->

---
### Requirement: 宣告的串流網址的寫法與心跳的契約

握手宣告的 sse transport 的 url SHALL 是下列兩種寫法之一：沒有協定與主機、以 `/` 開頭的路徑（由用戶端接在該專案的 API 位址之後），或與 Server 同一個 origin 的完整網址。示範用 Server 宣告的 url SHALL 維持 `/events`。宿主經宿主套件宣告推送時，宣告的內容 SHALL 原樣出現在握手回應；宿主套件 SHALL NOT 改寫 url。

提供事件串流的一方（示範用 Server，或自行提供串流的宿主）SHALL 在串流閒置時送出 SSE 的註解行當作心跳。示範用 Server 的心跳間隔預設 SHALL 是 15 秒。用戶端在 45 秒內沒有收到任何位元組（心跳也算）時 SHALL 把連線視為中斷並重新連線；因此心跳間隔 SHALL 小於 45 秒，串流才不會被反覆重連。

示範用 Server 的設定檔的 `events.heartbeat_secs` SHALL 小於 45。值是 45 以上時，Server SHALL 在綁定任何埠之前以設定錯誤結束（exit code 非零），stderr 的訊息 SHALL 指出設定檔的路徑、欄位 `events.heartbeat_secs` 與規則 `must be less than 45`。值是 44 以下、或沒有設定這個欄位（預設 15）時，行為 SHALL 與引入本需求之前相同。

使用者文件 SHALL 寫明這三件事：url 的兩種寫法與同 origin 的限制、15 秒與 45 秒這兩個數字、以及事件只指出哪個資源變了而用戶端收到後重新讀取。

#### Scenario: 示範用 Server 的宣告不變

- **WHEN** 對示範用 Server 完成握手
- **THEN** 回應宣告的 sse transport 的 url 是 `/events`、resume 是 true，polling 宣告的 url 是 `/sync-state`

#### Scenario: 宿主宣告的 url 原樣出現在握手

- **WHEN** 宿主建立宿主套件時宣告 sse transport 的 url 是 `https://specs.example.com/push/acme`，用戶端完成握手
- **THEN** 握手回應的 transports 含 type 為 sse、url 為 `https://specs.example.com/push/acme` 的一項

#### Scenario: 閒置的串流在 45 秒內有心跳

- **WHEN** 訂閱者連上示範用 Server（預設設定）的事件串流，之後沒有任何寫入
- **THEN** 訂閱者每隔約 15 秒收到一行註解；連線沒有因為閒置被用戶端判定為中斷

#### Scenario: 示範用 Server 拒絕 45 秒以上的心跳設定

- **WHEN** 設定檔寫 `events.heartbeat_secs: 45`，啟動示範用 Server
- **THEN** Server 沒有綁定任何埠就結束，exit code 非零；stderr 含設定檔路徑與 `'events.heartbeat_secs' must be less than 45`

##### Example: 各種 heartbeat_secs 的結果

| 設定 | 結果 |
| --- | --- |
| 沒有 `events` 區段 | 啟動，心跳每 15 秒 |
| `heartbeat_secs: 5` | 啟動，心跳每 5 秒 |
| `heartbeat_secs: 44` | 啟動，心跳每 44 秒 |
| `heartbeat_secs: 45` | 不啟動，訊息指出 `events.heartbeat_secs` 與 `must be less than 45` |
| `heartbeat_secs: 60` | 不啟動，同一則訊息 |

#### Scenario: 超過 45 秒沒有資料時用戶端重連

- **WHEN** 一條串流連上之後超過 45 秒沒有送出任何位元組
- **THEN** 用戶端把這條連線視為中斷，以輪詢比對是否有錯過的變更，並重新訂閱

<!-- @trace
source: event-payloads-and-push-url
updated: 2026-11-19T10:40:03+08:00
-->

---
### Requirement: 使用者失去權限時關閉串流

示範用 Server 的事件串流在開著的期間，SHALL 每隔一個心跳間隔（設定檔的 `events.heartbeat_secs`，預設 15 秒）確認訂閱的使用者仍是可用的帳號，且仍是該 project 的成員。任一項不成立時 Server SHALL 結束該串流（關閉連線），結束之後 SHALL NOT 再對它送出任何事件。之後以同一憑證重新訂閱 SHALL 依既有的認證與成員資格規則回應（不是成員為 403 permission_denied）。

這項確認 SHALL 只看帳號狀態與成員資格，SHALL NOT 因訂閱時使用的 access token 到期而結束串流。確認時身分儲存無法讀取 SHALL NOT 結束串流，Server SHALL 於下一個間隔再確認。帳號與成員資格沒有變動的串流 SHALL NOT 因這項確認而中斷，事件 SHALL 維持不漏不重。

自行提供串流的宿主 SHALL 由宿主自己在使用者失去權限時關閉串流；宿主套件不提供串流，因此不代為關閉。使用者文件 SHALL 寫明兩件事：宿主要在使用者失去權限時關閉自己的串流；用戶端在串流被關閉後會重新訂閱並輪詢，兩者都被拒時 Speclink Desktop 顯示「沒有存取權限」。

#### Scenario: 成員資格被移除後串流結束

- **WHEN** 成員 alice 訂閱 project demo 的事件串流，之後管理者把 alice 移出 demo
- **THEN** 在一個心跳間隔內 Server 結束這條串流；alice 以同一憑證重新訂閱時回 403 且 reason 為 permission_denied

#### Scenario: 帳號被停用後串流結束

- **WHEN** 成員 alice 訂閱事件串流，之後管理者停用 alice 的帳號
- **THEN** 在一個心跳間隔內 Server 結束這條串流

#### Scenario: 權限沒有變動的串流不中斷

- **WHEN** 成員 bob 訂閱事件串流並維持三個心跳間隔以上，期間另一個請求建立一個變更
- **THEN** 串流沒有被 Server 結束，bob 收到該變更的 invalidate 事件恰一次

#### Scenario: 身分儲存暫時無法讀取

- **WHEN** 某一次確認時身分儲存回報錯誤，下一次確認時恢復，使用者仍是成員
- **THEN** 串流沒有被結束

<!-- @trace
source: project-whoami-and-denied-state
updated: 2026-11-21T17:43:14+08:00
-->

---
### Requirement: 跨資源事件的歸屬

事件歸到資源的對照 SHALL 涵蓋兩個新的事件種類：`discussion-unlinked` SHALL 歸到 scope `discussion`，resourceId 是事件帶的討論 slug；`change-provenance-changed` SHALL 歸到 scope `change`，resourceId 是事件帶的 change 名。一次提交帶多筆事件時，Server SHALL 為每一筆事件各推一筆 invalidation，順序 SHALL 與提交裡事件的順序相同；一筆 invalidation SHALL 仍然只有一個 scope 與一個 resourceId，資料形狀 SHALL 不變。既有事件種類的歸屬 SHALL 不變。用戶端收到 invalidation 之後的行為 SHALL 不變：重新讀取，不從事件取得內容。

未認證與非成員的請求 SHALL 與既有行為相同地被拒絕（401、403）；斷線續傳、保留政策與 reset 訊號 SHALL 不因為一次提交的事件變多而改變：每一筆事件 SHALL 有自己的事件識別，續傳 SHALL 以事件識別為單位。

#### Scenario: 從討論建立變更推兩筆

- **WHEN** 訂閱者連上事件串流，另一個使用者以討論 two 為來源建立變更 add-two
- **THEN** 訂閱者依序收到兩筆 invalidation：scope 為 change、resourceId 為 add-two；scope 為 discussion、resourceId 為 two

#### Scenario: 廢棄變更推出被解開的討論

- **WHEN** 訂閱者連上事件串流，另一個使用者廢棄由討論 two 轉出的變更 add-two
- **THEN** 訂閱者依序收到 scope 為 change、resourceId 為 add-two，以及 scope 為 discussion、resourceId 為 two 的 invalidation

#### Scenario: 重新結論推出受影響的變更

- **WHEN** 訂閱者連上事件串流，另一個使用者對已反映到變更 add-two 的討論 two 重新結論
- **THEN** 訂閱者依序收到 scope 為 discussion、resourceId 為 two，以及 scope 為 change、resourceId 為 add-two 的 invalidation

#### Scenario: 續傳以事件為單位

- **WHEN** 訂閱者在收到「從討論建立變更」的第一筆 invalidation 之後斷線，並以該筆的事件識別續傳
- **THEN** 訂閱者收到第二筆（scope 為 discussion、resourceId 為 two），沒有重複收到第一筆

#### Scenario: 既有事件的歸屬不變

- **WHEN** 另一個使用者勾選變更 demo 的一個任務
- **THEN** 訂閱者收到恰一筆 invalidation，scope 為 change、resourceId 為 demo，與這條需求之前相同

<!-- @trace
source: cross-resource-events-and-event-names
updated: 2026-11-23T21:04:06+08:00
-->
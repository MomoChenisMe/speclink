## Why

遠端 client 目前唯一的更新途徑是輪詢 /sync-state——正確但延遲取決於輪詢間隔，多人協作時看板要等下一輪才動。平台藍圖 §9.2 把 SSE 定為官方建議的預設 push transport：事件只是 invalidation hint（eventId、scope、resource id、revision），client 收到後仍以 Query＋ETag 重讀正典，漏事件永遠能靠輪詢收斂——正確性地基在第一子刀已落地，本刀補通知層。地基全部就緒卻閒置著：TeamStore 的 outbox 從 teamstore-contract-v2 起就隨每筆 commit 原子記錄事件、protocol 的 events 宣告型別含 sse 種類、typed client 的 handshake 已解析並保存 capabilities——只差 server 真的串流與宣告。Phase 3 的 Desktop 遠端 Workspace 需要「SSE/WS 中斷時以 Polling + ETag 恢復」（roadmap §5 Phase 3 gate），server 側不先就緒，Desktop 刀就沒有對手方。

目標使用者：Phase 3 起的 Desktop/Web UI 使用者（看板即時反映他人變更）與長時間 Agent workflow（§9.2：watch 類流程才訂閱事件）。

## What Changes

- 新增 /events SSE 端點（project-scoped 路由，沿用 bearer 前置與 binding 裁決）：自 TeamStore outbox 串流 invalidation 事件，每筆事件的 SSE id 為該 scope 的 outbox 序號，data 為 protocol 定義的 invalidation DTO——只攜事件識別、scope 種類、資源識別與 revision，不承載規格內容。連線期間以 SSE 註解心跳維持。
- resume 語意：帶 Last-Event-ID 重連自該序號之後續傳、不漏不重；序號已因保留政策被清理時 SHALL 收到明確的 reset 訊號事件——client 據此改走 Query＋ETag 全量收斂，server 不猜測遺失內容。
- server 內建 outbox 消費者：commit 後的新事件廣播給該 scope 的全部訂閱者；依保留政策（每 scope 保留固定筆數）ack outbox 讓 driver 可清理；慢消費者的連線緩衝有界，溢出即斷線（client 以 resume 恢復）。
- protocol 的 events 模組新增 invalidation 事件 DTO（camelCase、JSON Schema 匯出、序列化往返測試）；binding capabilities 宣告升級——transports 含 sse（url、resume: true），polling 宣告不變。client 端零行為變更（handshake 解析在 protocol-typed-client 刀已就緒且不建立連線）。
- 失敗模型端到端：斷線重連續傳無縫、重複投遞由 client 以事件識別去重（測試驗證 server 續傳不重送）、cursor 過期轉輪詢收斂、多訂閱者同時收到同一事件。

## Capabilities

### New Capabilities

- `server-event-stream`: server 的事件串流——outbox 到 SSE 的廣播、invalidation hint 形狀、resume 與 reset 訊號、保留政策與慢消費者處置。

### Modified Capabilities

- `reference-server`: /binding 的 capabilities 宣告由「不宣告 push transport」升級為宣告 sse transport（url 與 resume 支援），polling 宣告不變。

## Impact

- 相容性影響：純新增端點與宣告欄位；既有 client 對 capabilities 的解析已容忍 transports 內容（typed 保存、不建立連線），CLI 行為零變更；parity 31 項、color 16 項、twin 8 情境凍結不動（twin 的 stub binding 回應本就含空 transports，期望不變）。outbox 開始被 ack 後，SQLite driver 的既有清理語意首次實際運轉。
- Affected specs: `server-event-stream`（新增）、`reference-server`（修改）
- Affected code:
  - New: crates/speclink-server/src/events.rs、crates/speclink-server/tests/sse_events.rs
  - Modified: crates/speclink-protocol/src/events.rs、crates/speclink-server/src/app.rs、crates/speclink-server/src/auth.rs、crates/speclink-server/src/state.rs、crates/speclink-server/src/verb.rs、Cargo.lock
  - Removed: 無

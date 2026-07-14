## Context

TeamStore 的 outbox 自 teamstore-contract-v2 起隨每筆 commit 原子記錄事件：OutboxEntry 含單調序號 seq、revision 與 EventRecord（事件名、payload、actor、時戳）；read_outbox 自指定 cursor 之後讀取、ack_outbox 宣告消費進度讓 driver 可清理。server 的寫入路徑（host 橋接＋commit_with_events）已把領域事件落 outbox，但至今無消費者——/sync-state 的 ETag 走 revision 聚合，不讀 outbox。protocol 的 EventsDeclaration/EventTransport/TransportKind::Sse 型別在 protocol-typed-client 刀就緒，typed client 的 handshake 解析並保存宣告且不建立連線；server 的 binding 宣告目前只有 polling。藍圖 §9.1 定事件為 invalidation hint、§9.2 定 SSE 為預設 push transport 與宣告形狀、§15.3 列失敗模型（斷線、重複、亂序、resume cursor 過期）。

## Goals / Non-Goals

**Goals:**

- outbox 到 SSE 的單一廣播路徑：commit 成功即推播 invalidation hint，重連以 Last-Event-ID 續傳不漏不重。
- cursor 過期有明確 reset 訊號，client 永遠能以 Polling＋ETag 收斂——事件是加速，不是正確性來源。
- binding capabilities 如實宣告 sse transport，Phase 3 Desktop 按宣告對接。

**Non-Goals:**

- 不做 WebSocket transport（藍圖 §9.2：需要雙向通道的服務才選 WS，屬視需求的後續 Phase）。
- 不動 client 端訂閱邏輯（CLI 一般命令不訂閱；Desktop 訂閱屬 Phase 3 刀）；本刀 e2e 以 HTTP 測試消費 SSE。
- 不做跨 server instance 的事件分發（single-node 定位；cluster 屬 PostgreSQL coordination 之後）。
- 不做事件內容過濾或訂閱粒度選擇：一條連線即該 binding scope 的全部 invalidation 事件。
- 不改 /sync-state 與 ETag 語意（輪詢地基不動）。
- session cookie 認證的 SSE（瀏覽器 EventSource 情境）不在本刀：/events 走既有 bearer 前置；Web UI 需要 cookie 語意時隨該 UI 刀補。

## Decisions

### 決策 1：事件形狀是 invalidation hint 的 typed DTO

protocol 的 events 模組新增 invalidation 事件 DTO：事件識別（該 scope 的 outbox 序號的字串形）、scope 種類（change、discussion、spec 等資源類別）、資源識別（change 名或 discussion slug）、revision。SSE 的 id 欄位即 outbox 序號、data 欄位即此 DTO 的 JSON。不承載事件名以外的領域內容——client 一律經 Query＋ETag 重讀正典（§9.1）。領域事件名到 scope 種類與資源識別的對映在 server 內單點實作（EventRecord 的事件名與 payload 是輸入），對映不了的事件種類以資源類別 unknown 照發不吞——寧可多一次無效重讀，不可漏 invalidation。

### 決策 2：每 scope 一個廣播器，outbox 是唯一事實來源

server 內每個活躍 scope 一個廣播器：寫入路徑 commit 成功後通知廣播器，廣播器自 read_outbox 拉新事件推給全部訂閱者——推播內容永遠出自 outbox 讀取，不從記憶體中的領域事件直接組裝，確保與 resume 續傳讀到的是同一份序列（不亂序、不分叉）。無訂閱者時廣播器閒置，不輪詢。

### 決策 3：保留政策驅動 ack，過期即 reset 訊號

廣播器依保留政策 ack outbox：每 scope 保留最近 N 筆（組態可調，預設 1024），更舊的 ack 給 driver 清理。重連的 Last-Event-ID 落在已清理範圍（小於現行 acked cursor）時，server 立即送一筆 reset 訊號事件（獨立的 SSE event 種類）再維持連線推新事件；client 收到 reset 即走 Query＋ETag 全量收斂。不猜測、不重建已清理的事件（§9.2 選擇規則 4）。

### 決策 4：/events 沿用 bearer 前置，慢消費者有界緩衝

/events 是 project-scoped 路由，複用既有認證與 binding 前置（PAT 或 access token、membership 檢查、repo 裁決）——身分與授權語意跟其他路由零差異。每條連線的送出緩衝有界（組態可調），慢消費者塞滿即斷線——client 以 Last-Event-ID 重連續傳，正確性不受影響；不為慢消費者無界堆積記憶體。連線期間每固定間隔送 SSE 註解行心跳，避免中介 proxy 判閒置斷線。

### 決策 5：twin 與既有測試零擾動

binding 宣告升級後，server 的 /binding 回應 transports 含 sse——twin harness 的 stub 是獨立的 mock（其 binding body 維持空 transports），期望不變；client 的宣告解析在 protocol-typed-client 刀已測「sse 與 polling 兩種宣告的型別保存」。本刀新增的 e2e 只驗 server 行為，不動任何既有凍結。

## Implementation Contract

- Behavior：client 連上 /events 後，另一 client 完成任一寫入動詞，前者在心跳間隔內收到含該資源識別與 revision 的 invalidation 事件；斷線後帶 Last-Event-ID 重連，期間發生的事件依序補齊；重連點已被清理時先收到 reset 訊號。
- Interface / data shape：GET /events（project-scoped，SSE 回應）；SSE 事件 id 為 outbox 序號、data 為 protocol invalidation DTO（camelCase）；reset 訊號為獨立 SSE event 種類；心跳為 SSE 註解行。/binding 的 capabilities.events.transports 含 type sse、url /events、resume true；polling 宣告不變。組態新增事件段（保留筆數、連線緩衝、心跳間隔，皆有預設值，段缺席即全預設）。
- Failure modes：未認證/非成員連 /events → 與其他路由相同的 401/403 三元組；Last-Event-ID 非法格式 → 視同未帶（從最新開始）；Last-Event-ID 已清理 → reset 訊號；慢消費者緩衝溢出 → server 斷線（client 重連恢復）；store 讀取失敗 → 連線以 SSE 錯誤終止，client 重連或轉輪詢。
- Acceptance criteria：cargo test -p speclink-server 全綠（廣播、resume、reset、慢消費者、多訂閱者）；cargo test -p speclink-protocol 全綠（DTO 匯出往返）；npm run test:all 全綠且 parity/color/twin 凍結零 diff。

## Risks / Trade-offs

- ack 讓 outbox 可清理，等於 SSE 廣播器成為 outbox 的唯一消費者 → 後續若有第二消費者（如 audit）需改共用 cursor 協定；本刀在組態註記保留筆數語意，audit 刀屆時自行決策。
- 每 scope 保留 N 筆的記號是 acked cursor，重啟後廣播器自 acked cursor 續讀 → 重啟期間斷線的 client 若 Last-Event-ID 仍在未清理範圍即正常續傳，否則 reset——無隱藏狀態遺失。
- 心跳與有界緩衝的預設值是經驗值 → 全部組態可調；錯的預設頂多造成多餘重連，正確性由 resume/reset 保證。
- 資源類別 unknown 的事件照發 → client 可能做一次全量重讀；比靜默吞事件安全。

## Migration Plan

純新增。部署後 outbox 首次被 ack，SQLite driver 的清理路徑開始實際運轉（conformance 已覆蓋 ack 語意）。回退即回捨本 change：/events 消失、binding 宣告回到只有 polling，既有 client 因 handshake 只保存宣告而零影響；已 ack 的 outbox 記錄不可恢復，但事件本就不是正確性來源。

## Open Questions

（無）

## 1. protocol DTO 與事件對映

- [x] 1.1 【紅→綠】speclink-protocol 的 events 模組新增 invalidation 事件 DTO（事件識別、scope 種類、資源識別、revision；camelCase）並納入既有 JSON Schema 匯出與序列化往返測試。驗收：cargo test -p speclink-protocol 全綠。 <!-- speclink-task:tsk_01KXFV9VHJY15KS8RE6CVEQDTE -->
- [x] 1.2 【紅→綠】server 內領域事件對映單點：EventRecord 的事件名與 payload 對映到 scope 種類與資源識別（change 系列、discussion 系列、archive 產出的 spec），對映不了的事件名以 unknown 類別照發不吞。以單元測試固定每一種既有領域事件名的對映結果。 <!-- speclink-task:tsk_01KXFV9VHJMXHNGWMEGPVGMS1A -->

## 2. 廣播器與保留政策

- [x] 2.1 【紅】針對「SSE 串流自 outbox 且事件是 invalidation hint」與「保留政策與 reset 訊號」的廣播器層寫測試（不經 HTTP，直接對廣播器）：commit 通知後訂閱者收到出自 read_outbox 的事件且序號單調；每 scope 保留 N 筆之外的舊事件被 ack；自小於 acked cursor 的序號續讀得到 reset 指示。驗收：測試存在且此時失敗。 <!-- speclink-task:tsk_01KXFV9VHJPJQZZ0EK7676SPJY -->
- [x] 2.2 【綠】實作 crates/speclink-server/src/events.rs 廣播器：每活躍 scope 一個、寫入路徑 commit 成功後通知、自 read_outbox 拉取推播（推播與續傳同源）、依保留政策（組態預設 1024 筆）ack、無訂閱者時閒置。2.1 全綠。 <!-- speclink-task:tsk_01KXFV9VHJE2YZFT4PHQRJS5VD -->

## 3. /events 端點

- [x] 3.1 【紅】針對 /events HTTP 層寫測試：未認證/非成員回與其他路由一致的 401/403 三元組；訂閱期間另一 client 寫入即收到 invalidation 事件（id 為 outbox 序號、data 為 DTO、不含文件內容）；兩訂閱者各收恰一筆；Last-Event-ID 續傳補齊斷線期間三筆事件不漏不重；非法 Last-Event-ID 視同未帶；已清理序號重連先收 reset 訊號事件；固定間隔收到 SSE 註解心跳。驗收：測試存在且此時失敗。 <!-- speclink-task:tsk_01KXFV9VHJA3V2TKRW8HT6RK7F -->
- [x] 3.2 【綠】實作 /events 路由（axum SSE 回應、複用 bearer 前置與 binding 裁決、心跳間隔與連線緩衝走組態預設值），3.1 全綠。 <!-- speclink-task:tsk_01KXFV9VHJ4GBHDJFK0AF1DVQW -->
- [x] 3.3 【紅→綠】慢消費者處置：一訂閱者停讀至緩衝溢出被斷線、另一訂閱者持續收事件無缺漏；被斷線者以 Last-Event-ID 重連依 resume/reset 規則恢復。組態新增事件段（保留筆數、連線緩衝、心跳間隔，段缺席即全預設，形狀不合啟動 fail closed 沿用既有組態錯誤報告）。 <!-- speclink-task:tsk_01KXFV9VHJ247E7SFCS0RNWQAQ -->

## 4. binding 宣告與收斂閉環

- [x] 4.1 【紅→綠】/binding 的 capabilities 宣告升級：transports 含 type sse、url 為事件端點、resume true，polling 宣告不變；宣告的 url 與實際路由一致。測試涵蓋「capabilities 宣告含 sse 與 polling」情境；確認 twin harness 的 stub 期望不受影響（stub 自帶 binding body，不動）。 <!-- speclink-task:tsk_01KXFV9VHJKFV6RHZTNT8BKCV3 -->
- [x] 4.2 【紅→綠】漏事件收斂閉環 e2e：對真 server（SQLite store）訂閱後製造 cursor 過期（寫入超過保留筆數再以舊序號重連）→ 收到 reset → 以 /sync-state 的 ETag 比對與查詢路由全量重讀，結果反映全部已 commit 變更 → 重新訂閱自最新序號續聽新寫入。驗收：cargo test -p speclink-server 全綠。 <!-- speclink-task:tsk_01KXFV9VHJ6DHJFW6GR1WP8XNX -->

## 5. 回歸

- [x] 5.1 執行 npm run test:all 確認全 workspace 回歸：parity 31 項、color 16 項、twin 8 情境凍結零 diff；CLI client 零變更。驗收：全數通過。 <!-- speclink-task:tsk_01KXFV9VHJ2P33XM5EE97J5C0R -->

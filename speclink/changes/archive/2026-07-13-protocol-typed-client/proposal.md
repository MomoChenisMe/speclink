## Why

平台架構藍圖 §4.5 要求自訂服務與官方 Server 實作同一套 Client Protocol——Command／Query／Context API、以 Query＋ETag 為必要恢復地基的 Event discovery、API version、標準 error reason、ETag/If-Match 與 actor/repo scope——並發布 schema 與 Client SDK，「避免每個自訂服務對相同動詞產生不同語意」。現況（重構路線圖 §2 與結論第 4 點）的遠端路徑是實驗性旁路：speclink-remote 以 raw serde_json::Value 收發逐 verb endpoint，CLI 的 remote 攔截層自行重組結果，兩者共約 1,360 行且不承諾保留；路線圖 §7 遷移表明定「remote error translation 經驗移入正式 protocol client，raw serde_json::Value 改 typed DTO」。藍圖 §4.7 要求 binding handshake：Agent session 開始前取得不可含糊的 binding 與 capabilities，缺失或多義即停止。若 Phase 2 的 Server 先於 Protocol 動工，wire contract 會由實作反推、重演 REST v1 的旁路歷史。

本刀與後續的 context-materializer 合計構成路線圖 §4.2 順位 7（protocol-client-context）；因交付物橫跨四個子系統，拆為兩刀。目標使用者：Phase 2（reference-server 以本刀的 protocol 型別實作端點）與 Phase 3（Desktop RemoteDataSource 以 typed client 為地基）的實作者，以及現有 remote 模式使用者——其 CLI 可見行為與輸出完全不變。

## What Changes

- 新增 `speclink-protocol` crate（Cargo workspace 成員）：Client Protocol 的唯一 Rust 定義——Command／Query／Context API 的 typed request/response DTO（serde camelCase）、API version 常數、標準 error reason registry（含既有 remote error mapping 經驗的正式化）、ETag/If-Match 與 actor/repo scope 的型別、Event discovery 宣告（transports、polling、resume 能力，對齊藍圖 §9.2 的 capability discovery 形狀）、binding handshake 的請求回應形狀（藍圖 §4.7 的 GET /binding payload）。
- protocol 型別可匯出 JSON Schema（供非 Rust 實作與文件），Rust 型別為正典。
- `speclink-remote` 重構為 typed client：client 的收發改以 speclink-protocol DTO（raw serde_json::Value 全數退場）、error translation 移入 protocol error reason 對映、新增 binding handshake 呼叫；auth（PAT）與 project-scoped URL 語意保留。
- CLI 的 remote 攔截層改消費 typed client：remote 模式下各動詞的人眼與 --json 輸出、exit code 與現行逐位元一致（twin 對照全綠；verb-contract 既有「remote 輸出形狀與 fs 一致」需求不變）。
- client 端 protocol 對測：以 stub server（沿既有 twin harness 基建）驗證 typed client 的請求形狀、ETag/If-Match 行為、error reason 對映與 handshake fail-closed（缺失／多義 binding 拒絕）。
- Event discovery 只交付宣告與型別：不實作 SSE/WS transport 與訂閱。

## Non-Goals

- 不實作 Server 端點與 server-side protocol conformance suite（Phase 2 reference-server 消費本刀型別）。
- 不做 Context Materializer、.speclink/context/ 與 remote skill 調整（下一刀 context-materializer）。
- 不實作 SSE/WebSocket transport、訂閱與事件恢復（Phase 2）；本刀只有 discovery 宣告型別。
- 不做 Desktop RemoteDataSource 與 WorkspaceSession（Phase 3）。
- 不改 remote 模式任何現行輸出與 .speclink.yaml 設定欄位；不改 fs 模式任何行為。
- 不發布 npm 套件與 OpenAPI 文件站（JSON Schema 匯出為程式產物即可）。

## Capabilities

### New Capabilities

- `client-protocol`: Client Protocol 的 typed 契約——Command/Query/Context DTO、API version、標準 error reason registry、ETag/If-Match 與 scope 型別、Event discovery 宣告、binding handshake 形狀、JSON Schema 匯出，以及 typed client 取代 raw JSON 旁路後的行為不變保證。

### Modified Capabilities

- `remote-connection`: 遠端連線的 client 實作由逐 verb raw JSON 改經 typed protocol client 與 binding handshake；連線設定、模式解析與使用者可見行為不變（需求層面：handshake 失敗與 binding 多義的 fail-closed 行為納入正典）。

## Impact

- 影響的 crate：新增 `speclink-protocol`；`speclink-remote`（typed client 重構）；`speclink-cli`（remote 攔截層改消費 typed client，輸出不變）；根 Cargo workspace 設定隨成員追加而動。fs 模式路徑零改動。
- 相容性影響：remote 與 fs 模式的人眼與 --json 輸出、exit code 逐位元不變；twin harness 8 情境、parity 31 項、color 16 項全綠。`.speclink.yaml` 欄位與語意不變。speclink-remote 的公開 API 屬工作區內部（無外部發布），typed 化不構成對外破壞。
- Affected specs: `client-protocol`（新增）、`remote-connection`（修改）。
- Affected code:
  - New: crates/speclink-protocol/Cargo.toml、crates/speclink-protocol/src/lib.rs、crates/speclink-protocol/src/command.rs、crates/speclink-protocol/src/query.rs、crates/speclink-protocol/src/context.rs、crates/speclink-protocol/src/events.rs、crates/speclink-protocol/src/error.rs、crates/speclink-protocol/src/binding.rs
  - Modified: Cargo.toml、Cargo.lock、crates/speclink-remote/src/lib.rs、crates/speclink-remote/src/client.rs、crates/speclink-remote/src/auth.rs、crates/speclink-cli/src/remote_commands.rs
  - Removed: 無

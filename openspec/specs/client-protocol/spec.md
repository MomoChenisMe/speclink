# client-protocol Specification

## Purpose

TBD - created by archiving change 'protocol-typed-client'. Update Purpose after archive.

## Requirements

### Requirement: protocol 型別是 wire contract 的唯一定義

Client Protocol 的 Command、Query、Context 請求與回應 SHALL 以 typed DTO 定義（序列化欄位 camelCase），Rust 型別為正典並 SHALL 可匯出 JSON Schema；API version SHALL 為 protocol 常數並隨請求與 handshake 回應攜帶。client 與未來 server SHALL 消費同一份型別，SHALL NOT 各自以 raw JSON 重組 wire payload。

#### Scenario: DTO 可匯出 schema 且序列化穩定

- **WHEN** 對 protocol 的 command 與 query DTO 執行 JSON Schema 匯出與序列化往返測試
- **THEN** 匯出成功且欄位皆為 camelCase；反序列化回相同值


<!-- @trace
source: protocol-typed-client
updated: 2026-07-13
code:
  - Cargo.lock
  - Cargo.toml
  - crates/speclink-cli/Cargo.toml
  - crates/speclink-cli/src/commands.rs
  - crates/speclink-cli/src/remote_commands.rs
  - crates/speclink-cli/tests/doc_verbs.rs
  - crates/speclink-cli/tests/no_raw_wire_json.rs
  - crates/speclink-cli/tests/remote_handshake_gate.rs
  - crates/speclink-cli/tests/remote_read_path.rs
  - crates/speclink-cli/tests/remote_write_path.rs
  - crates/speclink-protocol/Cargo.toml
  - crates/speclink-protocol/src/binding.rs
  - crates/speclink-protocol/src/command.rs
  - crates/speclink-protocol/src/context.rs
  - crates/speclink-protocol/src/error.rs
  - crates/speclink-protocol/src/events.rs
  - crates/speclink-protocol/src/lib.rs
  - crates/speclink-protocol/src/query.rs
  - crates/speclink-remote/Cargo.toml
  - crates/speclink-remote/src/client.rs
  - crates/speclink-remote/src/lib.rs
  - crates/speclink-remote/tests/client_errors.rs
  - crates/speclink-remote/tests/handshake.rs
  - crates/speclink-remote/tests/typed_client.rs
-->

---
### Requirement: 標準 error reason registry

protocol 的錯誤回應 SHALL 為 status、reason、message 三元組；reason SHALL 屬封閉 registry：not_found、permission_denied、revision_conflict、invalid_argument、invalid_config、refused、unavailable、internal。typed client SHALL 把 reason 對映到 CLI 既有錯誤訊息，同一 reason 的訊息文字 SHALL 與現行 remote error translation 逐位元一致；未知 reason SHALL 對映為一般錯誤而非崩潰。

#### Scenario: reason 對映沿用現行訊息

- **WHEN** stub server 對某動詞回 revision_conflict 的錯誤回應
- **THEN** CLI 以非零 exit code 結束且 stderr 訊息與現行 409 情境的訊息逐位元一致

#### Scenario: 未知 reason 不崩潰

- **WHEN** stub server 回傳 registry 之外的 reason 字串
- **THEN** client 以一般錯誤處理並保留 message 供顯示，不 panic


<!-- @trace
source: protocol-typed-client
updated: 2026-07-13
code:
  - Cargo.lock
  - Cargo.toml
  - crates/speclink-cli/Cargo.toml
  - crates/speclink-cli/src/commands.rs
  - crates/speclink-cli/src/remote_commands.rs
  - crates/speclink-cli/tests/doc_verbs.rs
  - crates/speclink-cli/tests/no_raw_wire_json.rs
  - crates/speclink-cli/tests/remote_handshake_gate.rs
  - crates/speclink-cli/tests/remote_read_path.rs
  - crates/speclink-cli/tests/remote_write_path.rs
  - crates/speclink-protocol/Cargo.toml
  - crates/speclink-protocol/src/binding.rs
  - crates/speclink-protocol/src/command.rs
  - crates/speclink-protocol/src/context.rs
  - crates/speclink-protocol/src/error.rs
  - crates/speclink-protocol/src/events.rs
  - crates/speclink-protocol/src/lib.rs
  - crates/speclink-protocol/src/query.rs
  - crates/speclink-remote/Cargo.toml
  - crates/speclink-remote/src/client.rs
  - crates/speclink-remote/src/lib.rs
  - crates/speclink-remote/tests/client_errors.rs
  - crates/speclink-remote/tests/handshake.rs
  - crates/speclink-remote/tests/typed_client.rs
-->

---
### Requirement: binding handshake 前置且 fail closed

typed client SHALL 提供 binding handshake：回應含 actor、project、repo、apiVersion、engineVersion 與 capabilities（含 events 的 transports 與 polling 宣告）。API version 不相容、binding 缺失、無權限或多義時 handshake SHALL 以帶原因的錯誤拒絕，SHALL NOT 自動選擇候選；handshake 失敗時 SHALL NOT 進入動詞流程。events 宣告 SHALL 解析為型別保存；本能力 SHALL NOT 建立 SSE 或 WebSocket 連線。

#### Scenario: version 不相容即停

- **WHEN** stub server 的 handshake 回應宣告不相容的 apiVersion
- **THEN** client 回帶版本原因的拒絕；後續動詞請求不被送出

#### Scenario: capabilities 宣告解析保存

- **WHEN** handshake 回應宣告 sse 與 polling 兩種更新方式
- **THEN** client 的 capabilities 型別含該宣告內容；不建立任何事件連線


<!-- @trace
source: protocol-typed-client
updated: 2026-07-13
code:
  - Cargo.lock
  - Cargo.toml
  - crates/speclink-cli/Cargo.toml
  - crates/speclink-cli/src/commands.rs
  - crates/speclink-cli/src/remote_commands.rs
  - crates/speclink-cli/tests/doc_verbs.rs
  - crates/speclink-cli/tests/no_raw_wire_json.rs
  - crates/speclink-cli/tests/remote_handshake_gate.rs
  - crates/speclink-cli/tests/remote_read_path.rs
  - crates/speclink-cli/tests/remote_write_path.rs
  - crates/speclink-protocol/Cargo.toml
  - crates/speclink-protocol/src/binding.rs
  - crates/speclink-protocol/src/command.rs
  - crates/speclink-protocol/src/context.rs
  - crates/speclink-protocol/src/error.rs
  - crates/speclink-protocol/src/events.rs
  - crates/speclink-protocol/src/lib.rs
  - crates/speclink-protocol/src/query.rs
  - crates/speclink-remote/Cargo.toml
  - crates/speclink-remote/src/client.rs
  - crates/speclink-remote/src/lib.rs
  - crates/speclink-remote/tests/client_errors.rs
  - crates/speclink-remote/tests/handshake.rs
  - crates/speclink-remote/tests/typed_client.rs
-->

---
### Requirement: typed client 全面取代 raw JSON 旁路

speclink-remote 與 CLI remote 攔截層的 wire payload 處理 SHALL 全數經 protocol DTO；SHALL NOT 殘留以通用 JSON 值重組回應的路徑。ETag 與 If-Match SHALL 以型別攜帶：帶 If-Match 的寫入在 revision 不符時 SHALL 得到 revision_conflict。remote 模式全部現行動詞的人眼輸出、--json 輸出與 exit code SHALL 與重構前逐位元一致。

#### Scenario: 寫入攜 If-Match 且衝突可辨

- **WHEN** typed client 以既知 ETag 執行寫入動詞而 stub server 判定 revision 已前進
- **THEN** 請求標頭含 If-Match；client 收到 revision_conflict reason 並對映現行衝突訊息

#### Scenario: remote 輸出凍結

- **WHEN** 對 stub server 於重構前後執行 twin harness 的全部情境
- **THEN** remote 與 fs 模式的 stdout、stderr 與 exit code 逐位元一致，8 情境全綠

<!-- @trace
source: protocol-typed-client
updated: 2026-07-13
code:
  - Cargo.lock
  - Cargo.toml
  - crates/speclink-cli/Cargo.toml
  - crates/speclink-cli/src/commands.rs
  - crates/speclink-cli/src/remote_commands.rs
  - crates/speclink-cli/tests/doc_verbs.rs
  - crates/speclink-cli/tests/no_raw_wire_json.rs
  - crates/speclink-cli/tests/remote_handshake_gate.rs
  - crates/speclink-cli/tests/remote_read_path.rs
  - crates/speclink-cli/tests/remote_write_path.rs
  - crates/speclink-protocol/Cargo.toml
  - crates/speclink-protocol/src/binding.rs
  - crates/speclink-protocol/src/command.rs
  - crates/speclink-protocol/src/context.rs
  - crates/speclink-protocol/src/error.rs
  - crates/speclink-protocol/src/events.rs
  - crates/speclink-protocol/src/lib.rs
  - crates/speclink-protocol/src/query.rs
  - crates/speclink-remote/Cargo.toml
  - crates/speclink-remote/src/client.rs
  - crates/speclink-remote/src/lib.rs
  - crates/speclink-remote/tests/client_errors.rs
  - crates/speclink-remote/tests/handshake.rs
  - crates/speclink-remote/tests/typed_client.rs
-->
# server-context-api Specification

## Purpose

remote 模式下 context 投影的來源端點：以一致快照回傳工作區的規格與變更內容、依指定 change 縮小範圍並透傳 flow，以及 typed client 對應的 context snapshot 方法。本 capability 保證投影拿到的是同一時點的完整視圖，而非多次請求拼湊出的混合狀態。

## Requirements

### Requirement: 一致快照端點

server SHALL 提供 project-scoped 的 context snapshot 端點（沿用 bearer 前置與 binding 裁決），接受 protocol 的 ContextSnapshotRequest、回應 ContextSnapshot。全部 documents SHALL 讀自同一個 store snapshot，SHALL NOT 逐檔對 live store 分次讀取；snapshot id SHALL 與 scope 狀態記號同源（該 scope 任何成功 commit 後必變）；policy revision SHALL 為 workflow config 文件於該 snapshot 的 revision（文件不存在時缺席）；每份文件 SHALL 附契約 digest（與投影驗證同源的 content_digest）。回應 SHALL 攜 scope 狀態記號；請求帶 If-None-Match 且記號未變 SHALL 回 304 無 body。

#### Scenario: 快照一致且識別真實

- **WHEN** 取得快照後另一 client 完成一筆寫入，再次取得快照
- **THEN** 兩次快照各自內部一致（documents 與 digest 對應同一狀態）；兩次的 snapshot id 不同；第二次含寫入後內容

#### Scenario: 未變回 304

- **WHEN** 以現值 snapshot id 作 If-None-Match 請求快照且期間無任何 commit
- **THEN** 回 304 無 body；發生一筆 commit 後同樣請求回 200 與新快照

#### Scenario: store 失聯回 unavailable

- **WHEN** store 後端不可用時請求快照
- **THEN** 回 503 且 reason 為 unavailable

---

<!-- @trace
source: server-context-api
updated: 2026-07-15
code:
  - crates/speclink-cli/src/remote_commands.rs
  - crates/speclink-cli/tests/remote_read_path.rs
  - crates/speclink-host/src/bridge.rs
  - crates/speclink-host/src/projection.rs
  - crates/speclink-host/tests/bridge_dual_path.rs
  - crates/speclink-remote/src/client.rs
  - crates/speclink-remote/tests/typed_client.rs
  - crates/speclink-server/src/app.rs
  - crates/speclink-server/src/auth.rs
  - crates/speclink-server/src/backup.rs
  - crates/speclink-server/src/context.rs
  - crates/speclink-server/src/lib.rs
  - crates/speclink-server/tests/binding.rs
  - crates/speclink-server/tests/context_api.rs
  - crates/speclink-server/tests/e2e_cli.rs
  - crates/speclink-store-sqlite/src/lib.rs
  - crates/speclink-store/src/types.rs
-->

---
### Requirement: change 縮小與 flow 透傳

請求指定 change 時，documents SHALL 涵蓋：該 change 的全部 artifacts、該 change 的 delta specs、全部正典 specs、config 與 LANGUAGE；指定的 change 不存在 SHALL 回 404 not_found。未指定 change SHALL 回全量投影內容（正典 specs、全部 changes 的文件、config、LANGUAGE）。flow 欄位 SHALL 原樣透傳而 SHALL NOT 影響 server 的文件集——依流程縮小維持 materializer 職責。

#### Scenario: change 縮小的文件集完備

- **WHEN** 對含兩個 changes 與三個正典 specs 的 scope 以 change A 請求快照
- **THEN** documents 含 A 的 artifacts 與 delta specs、全部三個正典 specs、config 與 LANGUAGE；不含 change B 的文件

#### Scenario: 未知 change 拒絕

- **WHEN** 以不存在的 change 名請求快照
- **THEN** 回 404 且 reason 為 not_found

---

<!-- @trace
source: server-context-api
updated: 2026-07-15
code:
  - crates/speclink-cli/src/remote_commands.rs
  - crates/speclink-cli/tests/remote_read_path.rs
  - crates/speclink-host/src/bridge.rs
  - crates/speclink-host/src/projection.rs
  - crates/speclink-host/tests/bridge_dual_path.rs
  - crates/speclink-remote/src/client.rs
  - crates/speclink-remote/tests/typed_client.rs
  - crates/speclink-server/src/app.rs
  - crates/speclink-server/src/auth.rs
  - crates/speclink-server/src/backup.rs
  - crates/speclink-server/src/context.rs
  - crates/speclink-server/src/lib.rs
  - crates/speclink-server/tests/binding.rs
  - crates/speclink-server/tests/context_api.rs
  - crates/speclink-server/tests/e2e_cli.rs
  - crates/speclink-store-sqlite/src/lib.rs
  - crates/speclink-store/src/types.rs
-->

---
### Requirement: typed client 的 context snapshot 方法

typed client SHALL 提供 context snapshot 方法：輸入 ContextSnapshotRequest 與選填的既知 snapshot id，輸出區分「未變」與「新快照」二值；請求 SHALL 走既有請求骨架（三標頭、handshake 前置、錯誤翻譯），SHALL NOT 以 raw JSON 拼裝。

#### Scenario: client 方法的二值輸出

- **WHEN** 分別以「現值 id 且無變更」與「無 id」呼叫 context snapshot 方法
- **THEN** 前者得到「未變」；後者得到含 documents 的新快照；錯誤情境（如 503）翻譯為既有 remote 錯誤形狀

<!-- @trace
source: server-context-api
updated: 2026-07-15
code:
  - crates/speclink-cli/src/remote_commands.rs
  - crates/speclink-cli/tests/remote_read_path.rs
  - crates/speclink-host/src/bridge.rs
  - crates/speclink-host/src/projection.rs
  - crates/speclink-host/tests/bridge_dual_path.rs
  - crates/speclink-remote/src/client.rs
  - crates/speclink-remote/tests/typed_client.rs
  - crates/speclink-server/src/app.rs
  - crates/speclink-server/src/auth.rs
  - crates/speclink-server/src/backup.rs
  - crates/speclink-server/src/context.rs
  - crates/speclink-server/src/lib.rs
  - crates/speclink-server/tests/binding.rs
  - crates/speclink-server/tests/context_api.rs
  - crates/speclink-server/tests/e2e_cli.rs
  - crates/speclink-store-sqlite/src/lib.rs
  - crates/speclink-store/src/types.rs
-->
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

- **WHEN** 執行 `crates/speclink-cli/tests/remote_read_path.rs` 對 stub server 與 fs 模式雙跑同一動詞的全部對照情境
- **THEN** remote 與 fs 模式的 `--json` 欄位形狀（key 集合）一致，全部對照情境全綠


<!-- @trace
source: stale-verification-vehicles
updated: 2026-07-27
code:
  - docs/implementation-refactor-roadmap.zh-TW.md
-->

---
### Requirement: 變更清單的審查狀態欄位

desktop 協定的 change 清單項 SHALL 增列 `reviewStatus` 欄位（字串 enum：`"none"`／`"inReview"`／`"reviewed"`／`"reviewedStale"`），且於章存在時附 `reviewedAt`（字串）與 `reviewedBy`（字串）。狀態判定：工單存在 → `inReview`；章存在且任務錨與內容指紋錨皆相符 → `reviewed`；章存在而任一錨不符 → `reviewedStale`；皆無 → `none`。凍結度重算 SHALL 於有工作樹的 client 端執行。內容指紋錨的檔案現值 SHALL 逐 change 解析讀取根：該 change 有 worktree 映射時讀該 worktree 副本的檔案，無映射時讀主 checkout——與同一清單項的任務錨同源，SHALL NOT 出現任務錨取自 worktree、指紋錨取自主 checkout 的劈半。scope 檔於解析後的根下不存在時維持「缺檔即不符 → Stale」語意。CLI `speclink list --json` SHALL NOT 包含上述任何欄位（相容性釘住歸 review-station 規格）。

#### Scenario: 四態判定

- **WHEN** desktop 載入變更清單
- **THEN** 每個 change 項含 `reviewStatus`，其值依「工單存在／章存在／雙錨相符」的組合為四態之一

##### Example: 章在但指紋不符

- **GIVEN** 某 change 的 metadata 含全套 reviewed 欄位，且 `reviewed_scope` 中一個檔的現值雜湊與記錄不符
- **WHEN** desktop 取得該 change 的清單項
- **THEN** `reviewStatus` 為 `"reviewedStale"`，`reviewedAt`／`reviewedBy` 仍存在

#### Scenario: 審查中的清單項

- **WHEN** 某 change 有 review.md 而無章
- **THEN** 清單項 `reviewStatus` 為 `"inReview"`，無 `reviewedAt`／`reviewedBy`

#### Scenario: worktree 中蓋章的凍結度以 worktree 現值判定

- **WHEN** 某 change 有 worktree 映射，`reviewed_scope` 各檔於 worktree 副本內的現值雜湊與記錄相符，主 checkout 的同名檔仍為蓋章前舊內容
- **THEN** 清單項 `reviewStatus` 為 `"reviewed"`

##### Example: worktree 內蓋章後又改檔才轉 stale

- **GIVEN** change fix-auth 有 worktree 映射，蓋章時 `reviewed_scope` 記錄 src/auth.rs 的雜湊；主 checkout 的 src/auth.rs 為未實作的舊內容
- **WHEN** worktree 副本的 src/auth.rs 與蓋章時一致 → 清單項判定；其後於 worktree 內再修改該檔 → 再次判定
- **THEN** 前者 `reviewStatus` 為 `"reviewed"`，後者為 `"reviewedStale"`


<!-- @trace
source: worktree-data-routing
updated: 2026-08-05
-->

---
### Requirement: 已封存清單的審查結局欄位

desktop 協定的已封存清單項 SHALL 增列 `reviewStatus`（字串 enum：`"none"`／`"reviewed"`／`"reviewedNotPassed"`）：封存目錄含章 → `reviewed`；含工單而無章 → `reviewedNotPassed`；皆無 → `none`。已封存側 SHALL NOT 做凍結度重算（封存即定格）。

#### Scenario: 化石工單的封存項

- **WHEN** desktop 載入已封存清單且某項的封存目錄含 review.md 而 metadata 無 reviewed 欄位
- **THEN** 該項 `reviewStatus` 為 `"reviewedNotPassed"`

<!-- @trace
source: code-review-stage
updated: 2026-08-02
code:
  - AGENTS.md
  - CLAUDE.md
  - README.en.md
  - README.md
  - apps/desktop/core/src/cache.rs
  - apps/desktop/core/src/query.rs
  - apps/desktop/core/src/verbs.rs
  - apps/desktop/src-tauri/src/lib.rs
  - apps/desktop/src-tauri/src/remote.rs
  - apps/desktop/src/App.tsx
  - apps/desktop/src/__tests__/store.test.ts
  - apps/desktop/src/__tests__/tauriDataSource.test.ts
  - apps/desktop/src/adapter/tauriDataSource.ts
  - apps/desktop/src/i18n/messages.ts
  - apps/desktop/src/store.ts
  - crates/speclink-cli/src/commands.rs
  - crates/speclink-cli/src/main.rs
  - crates/speclink-cli/src/remote_commands.rs
  - crates/speclink-cli/tests/review_verbs.rs
  - crates/speclink-core/assets/skills/review.md
  - crates/speclink-core/src/archive.rs
  - crates/speclink-core/src/command/mod.rs
  - crates/speclink-core/src/init.rs
  - crates/speclink-core/src/inprogress.rs
  - crates/speclink-core/src/lib.rs
  - crates/speclink-core/src/listing.rs
  - crates/speclink-core/src/model.rs
  - crates/speclink-core/src/review.rs
  - crates/speclink-core/src/skills.rs
  - crates/speclink-core/src/store.rs
  - crates/speclink-core/src/teststore.rs
  - crates/speclink-core/src/util.rs
  - crates/speclink-core/tests/golden/assets.lock
  - crates/speclink-core/tests/golden/claude.snapshot.md
  - crates/speclink-core/tests/golden/codex.snapshot.md
  - crates/speclink-core/tests/golden/neutral-cli.snapshot.md
  - crates/speclink-core/tests/golden/neutral-tool-call.snapshot.md
  - crates/speclink-core/tests/golden/remote-claude.marker.md
  - crates/speclink-core/tests/render_golden.rs
  - crates/speclink-fs/src/lib.rs
  - crates/speclink-host/src/bridge.rs
  - crates/speclink-host/src/commit.rs
  - crates/speclink-node/src/store_bridge.rs
  - crates/speclink-protocol/src/command.rs
  - crates/speclink-remote/src/client.rs
  - crates/speclink-remote/tests/client_errors.rs
  - crates/speclink-remote/tests/typed_client.rs
  - crates/speclink-server/src/app.rs
  - crates/speclink-server/src/routes.rs
  - crates/speclink-server/tests/e2e_cli.rs
  - crates/speclink-server/tests/read_api.rs
  - crates/speclink-server/tests/review_api.rs
  - packages/ui/src/__tests__/reviewBadge.test.tsx
  - packages/ui/src/adapter.ts
  - packages/ui/src/components/ArchivedDrawer.tsx
  - packages/ui/src/components/ArchivedList.tsx
  - packages/ui/src/components/ChangeCard.tsx
  - packages/ui/src/components/ReviewArchiveDialog.tsx
  - packages/ui/src/components/RichDetailDrawer.tsx
  - packages/ui/src/components/reviewStyle.tsx
  - packages/ui/src/i18n.tsx
  - packages/ui/src/index.ts
-->
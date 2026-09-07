# client-protocol Specification

## Purpose

client 與 server 之間 wire contract 的定義：以 Rust protocol 型別為唯一正典（JSON Schema 是匯出）、標準 error reason registry，以及變更清單、已封存清單、討論、封存結果與品質站工單等 payload 的欄位形狀。本 capability 保證 binding handshake 為前置且 fail closed，所有呼叫走 typed client——不留 raw JSON 旁路。

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

---
### Requirement: 變更清單的驗證狀態欄位

desktop 協定的 change 清單項 SHALL 增列 `verifyStatus`（字串 enum：`"none"`／`"inVerify"`／`"verified"`／`"verifiedStale"`），章存在時附 `verifiedAt`／`verifiedBy`（字串）。判定規則與審查狀態同構：工單存在 → `inVerify`；章在且雙錨相符 → `verified`；章在而任一錨不符 → `verifiedStale`；皆無 → `none`。CLI `speclink list --json` SHALL NOT 包含上述欄位。

#### Scenario: 驗證四態判定

- **WHEN** desktop 載入變更清單
- **THEN** 每個 change 項含 `verifyStatus`，其值依「工單存在／章存在／雙錨相符」為四態之一，與 `reviewStatus` 各自獨立判定

##### Example: 兩站狀態獨立

- **GIVEN** 某 change 有審查章（雙錨相符）且存在未結驗證工單
- **WHEN** desktop 取得該 change 的清單項
- **THEN** `reviewStatus` 為 `"reviewed"` 且 `verifyStatus` 為 `"inVerify"`

<!-- @trace
source: verify-station-parity
updated: 2026-08-06
-->

---
### Requirement: 已封存清單的驗證結局欄位

desktop 協定的已封存清單項 SHALL 增列 `verifyStatus`（`"none"`／`"verified"`／`"verifiedNotPassed"`）：封存目錄含驗證章 → `verified`；含驗證工單而無章 → `verifiedNotPassed`；皆無 → `none`。已封存側不重算凍結度。

#### Scenario: 化石驗證工單的封存項

- **WHEN** desktop 載入已封存清單且某項封存目錄含 verify.md 而 metadata 無 verified 欄位
- **THEN** 該項 `verifyStatus` 為 `"verifiedNotPassed"`

<!-- @trace
source: verify-station-parity
updated: 2026-08-06
-->

---
### Requirement: 討論資訊 payload 增選填 kind 欄位

討論列表與單筆讀取的 payload(CLI --json、server 讀取路徑與型別化 client 共用同一 wire contract)SHALL 增選填欄位 kind(字串,目前唯一合法值 improve):記錄有 kind 時 SHALL 曝露、無 kind 時 SHALL 省略該鍵,既有 payload 形狀 SHALL 逐位元不變。欄位名 SHALL 為 camelCase 的 kind。本欄位為唯讀資訊,SHALL NOT 改變 remote 路徑既有的離線、認證失效與 revision 行為。

#### Scenario: 改進討論經讀取路徑曝露 kind

- **WHEN** 對 kind 為 improve 的討論執行討論列表或單筆讀取(本地 --json 或經 server 讀取路徑)
- **THEN** payload 含 kind 欄位且值為 improve

#### Scenario: 一般討論 payload 形狀不變

- **WHEN** 對無 kind 欄位的既有討論執行討論列表或單筆讀取
- **THEN** payload 不含 kind 鍵,形狀與本欄位引入前逐位元一致

<!-- @trace
source: add-improve-flow
updated: 2026-08-07
-->

---
### Requirement: 封存回應的完整結果欄位

封存端點的回應 payload SHALL 增列下列欄位（camelCase，皆帶序列化預設值以向後相容）：datedName（字串，選填——封存目的地的 dated 名稱）、specs 清單各項的 added／modified／removed／renamed（整數，預設 0）、snapshotCreated（布林，選填）、archivedDiscussions（物件清單，各項含 slug 與 file 字串，預設空清單）、evidenceRecorded（布林，選填）。datedName 是新舊 server 的哨兵欄位：讀取方 SHALL 以其在場與否判定回應是否攜帶完整封存結果。

#### Scenario: 缺新欄位的回應可反序列化

- **WHEN** 以不含任何新欄位的既有形狀 JSON（僅 specs 清單）反序列化封存回應
- **THEN** 反序列化成功，datedName 缺席、各計數為 0、archivedDiscussions 為空清單——與既有 server 的回應相容

#### Scenario: 新 server 回應攜帶完整結果

- **WHEN** 新版 server 完成一筆會改動規格與封存來源討論的封存
- **THEN** 回應含 datedName、各 capability 的四項計數、archivedDiscussions 清單與 evidenceRecorded

<!-- @trace
source: cli-render-unification
updated: 2026-08-08
-->

---
### Requirement: 開工標記移除回應的移除旗標

開工標記移除端點 SHALL 有具名回應型別，攜帶 removed 欄位（布林，帶序列化預設值 true）——區分實際移除與「本來就沒開工」的冪等 no-op，兩者的人眼輸出是不同的行。缺席讀作 true：既有 server 的裸確認回應對呼叫端一律代表已移除，語意不變，因此不需哨兵欄位。

#### Scenario: 缺欄位的移除回應可反序列化

- **WHEN** 以空物件反序列化開工標記移除回應
- **THEN** 反序列化成功且 removed 為 true——與既有 server 的裸確認回應同義

#### Scenario: 冪等 no-op 可辨識

- **WHEN** 以 removed 為 false 的 JSON 反序列化開工標記移除回應
- **THEN** removed 讀出 false，呼叫端得以印出「本來就沒開工」的行

<!-- @trace
source: cli-render-unification
updated: 2026-08-08
-->

---
### Requirement: 工單回應的原文欄位

review 與 verify 兩站共用的工單讀取回應 SHALL 增列 content 欄位（字串，選填，帶序列化預設值）攜帶工單文件原文全文。content 是工單人眼輸出的哨兵欄位：讀取方 SHALL 以其在場與否判定 server 新舊，缺席時 SHALL 退回既有的結構化摘要呈現。

<!-- @trace
source: cli-render-unification
updated: 2026-08-08
-->

---
### Requirement: 討論結論回應的重收清單

討論結論端點 SHALL 有具名回應型別，攜帶 restaleFlagged 欄位（字串清單，帶序列化預設值）——re-conclude 打回重收的變更名。空清單即「無變更被打回」，與既有 server 不回報此事實時的讀取結果相同，因此不需哨兵欄位。

#### Scenario: 缺欄位的結論回應可反序列化

- **WHEN** 以空物件反序列化討論結論回應
- **THEN** 反序列化成功且 restaleFlagged 為空清單

#### Scenario: 結論回應攜帶被打回的變更名

- **WHEN** 以含 restaleFlagged 兩筆變更名的 JSON 反序列化討論結論回應
- **THEN** 兩筆變更名依序讀出

#### Scenario: 缺 content 的工單回應可反序列化

- **WHEN** 以不含 content 的既有形狀 JSON 反序列化工單讀取回應
- **THEN** 反序列化成功且 content 缺席——與既有 server 的回應相容

#### Scenario: 新 server 工單回應攜帶原文

- **WHEN** 新版 server 回應一筆存在工單的變更的工單讀取請求
- **THEN** content 等於 store 中工單文件的原文全文，結構化欄位（rounds、lastRound）同時照舊在場

<!-- @trace
source: cli-render-unification
updated: 2026-08-08
-->

---
### Requirement: 變更清單的寫碼進度欄位

desktop 協定的 change 清單項 SHALL 增列 `codeTotal`/`codeComplete`/`codeRemaining` 三欄(寫碼任務的總數/完成數/剩餘數;`[M]` 手動測試任務不計),計數 SHALL 取自引擎任務雙組計數的同一入口——與品質站守門及失效判定的任務錨同源,SHALL NOT 於呈現層另行過濾。欄位命名 SHALL 與 instructions apply payload 的寫碼進度欄位一致。CLI `speclink list --json` SHALL NOT 包含此三欄;remote 變更摘要 payload SHALL NOT 增列(待手測標示為 local-only,沿審查狀態欄位的先例)。

#### Scenario: 清單項帶寫碼進度

- **WHEN** desktop 載入變更清單且某 change 有 9 個已勾寫碼任務與 1 個未勾 `[M]` 任務
- **THEN** 該清單項含 codeTotal=9、codeComplete=9、codeRemaining=0,既有欄位(completedTasks=9、totalTasks=10 等)不變

#### Scenario: CLI 清單不含寫碼進度欄位

- **WHEN** 執行 speclink list --json
- **THEN** change 項不含 codeTotal/codeComplete/codeRemaining,輸出與本需求引入前逐位元一致

<!-- @trace
source: task-marker-ui-and-parallel-removal
updated: 2026-08-11
-->

---
### Requirement: 已封存清單的呈現輔助欄位

desktop 協定的已封存清單項 SHALL 增列兩個選填欄位：`whyExcerpt`（字串——封存目錄 proposal.md 的 Why 區段首個非空行）與 `created`（字串 YYYY-MM-DD——封存目錄 metadata 的建立日期）。任一欄位的來源不可讀或缺席（proposal.md 不存在、無 Why 區段、metadata 無建立日期）時該欄位 SHALL 缺席（不序列化），SHALL NOT 以空字串或 null 佔位，清單其餘欄位照常回傳。兩欄位 SHALL 由清單載入一次帶出，SHALL NOT 要求前端逐項讀取封存文件。

#### Scenario: 封存項帶 Why 首句與建立日期

- **WHEN** desktop 載入已封存清單且某項的封存目錄含有 Why 區段的 proposal.md 與含建立日期的 metadata
- **THEN** 該項 `whyExcerpt` 為 Why 區段首個非空行、`created` 為該建立日期

#### Scenario: 來源缺席時欄位缺席

- **WHEN** desktop 載入已封存清單且某項的封存目錄無 proposal.md、metadata 無建立日期
- **THEN** 該項無 `whyExcerpt` 與 `created` 鍵，日期、名稱、任務數等既有欄位照常存在

<!-- @trace
source: desktop-archived-parity
updated: 2026-08-11
-->

---
### Requirement: 變更清單的建立者與來源討論欄位

ChangeSummary SHALL 增三個選填欄位（皆 camelCase、serde default）：createdBy SHALL 為 meta 的 created_by（缺席即省略）；created SHALL 為 meta 的 created 日期（缺席即省略）；fromDiscussions SHALL 為 meta 的 from_discussion 鏈（空清單即省略）。舊 server 不送這些欄位時，client 清單消費端 SHALL 以缺席容錯（頭像圓標與來源討論標記不顯示），SHALL NOT 偽造預設值。

#### Scenario: 清單欄位序列化與缺席省略

- **WHEN** 序列化一筆 meta 含 created_by、created 與 from_discussion 的 ChangeSummary，再序列化一筆三者皆缺的
- **THEN** 前者的 JSON 含 createdBy、created 與 fromDiscussions 三鍵；後者三鍵皆不出現；反序列化無此三鍵的舊 payload 不失敗

<!-- @trace
source: remote-read-parity
updated: 2026-08-26
-->

---
### Requirement: 單 change 讀取回應的 meta 歸屬欄位

ChangeStatus SHALL 增四個選填欄位（皆 camelCase、serde default、缺席即省略）：createdBy、createdWith、startedAt、startedBy，值各為 change meta 的 created_by、created_with、started_at、started_by。舊 server 不送時 client 的詳情呈現 SHALL 維持對應列缺席，SHALL NOT 偽造預設值。

#### Scenario: 歸屬欄位序列化與缺席省略

- **WHEN** 序列化一筆 meta 四欄俱全的 ChangeStatus，再序列化一筆四欄皆缺的
- **THEN** 前者 JSON 含 createdBy、createdWith、startedAt、startedBy 四鍵；後者四鍵皆不出現；反序列化無此四鍵的舊 payload 不失敗

<!-- @trace
source: remote-read-parity
updated: 2026-08-26
-->

---
### Requirement: 討論資訊 payload 增選填 promotedTo 欄位

DiscussionInfo SHALL 增選填 promotedTo 欄位（camelCase、serde default、空清單即省略），值為該討論已轉出／已併入的 change 名稱清單，順序沿 frontmatter promoted_to 的累加順序。此欄位由 server 於 route 邊緣組裝；引擎側 DiscussionInfo 結構 SHALL NOT 因此欄位改動（引擎明訂 promoted_to 不進列表結構以保 CLI JSON 逐位元不變）。舊 server 不送時 client SHALL 以空清單容錯，SHALL NOT 據此推論討論未轉出以外的狀態。

#### Scenario: promotedTo 序列化與缺席容錯

- **WHEN** 序列化一筆 promotedTo 含兩個 change 名的 DiscussionInfo，再序列化一筆空清單的
- **THEN** 前者 JSON 含 promotedTo 且順序保持；後者無 promotedTo 鍵；反序列化無此鍵的舊 payload 不失敗且值為空清單

<!-- @trace
source: remote-read-parity
updated: 2026-08-26
-->

---
### Requirement: 討論資訊 payload 增選填 concluded 欄位

DiscussionInfo SHALL 增選填 concluded 欄位（camelCase、serde default、缺席即未知），值為該討論的 Conclusion 段是否已寫入內文（scaffold 佔位註解不算內文）。此欄位由 server 於 route 邊緣組裝；引擎側 DiscussionInfo 結構 SHALL NOT 因此欄位改動（引擎明訂結論判定不進列表結構以保 CLI JSON 逐位元不變）。序列化時缺席值 SHALL 省略鍵；組裝端 SHALL 對每筆討論恆填 true 或 false。舊 server 不送時 client SHALL 視為未知，SHALL NOT 把缺席當成 false、SHALL NOT 據此推論結論狀態。

#### Scenario: concluded 序列化與缺席容錯

- **WHEN** 序列化一筆 concluded 為 true 與一筆 concluded 為 false 的 DiscussionInfo，再反序列化一筆無 concluded 鍵的舊 payload
- **THEN** 前兩者 JSON 分別含 concluded: true 與 concluded: false；後者反序列化不失敗且值為未知（缺席），再序列化時無 concluded 鍵

<!-- @trace
source: conclusion-gated-discussion-archive
updated: 2026-09-01
-->

---
### Requirement: 討論搜尋回應 payload

protocol SHALL 以 Rust 型別定義討論搜尋回應（JSON Schema 為匯出）：頂層 hits 陣列，每筆含與討論資訊 payload 相同的欄位（slug、topic、status、rounds、created、createdBy 選填、kind 選填、promotedTo、concluded 選填、path、archived）加 matches 陣列；每個 match 含 kind、where、text 三個字串欄位，kind 值為 topic、slug、ruled-out、decision、rejected、deferred 之一，where 值為 frontmatter、round-N 或 conclusion。欄位一律 camelCase。typed client SHALL 提供搜尋方法，以關鍵字清單組成空白分隔的 q 呼叫 GET /discussions/search，回應反序列化為 typed 型別，SHALL NOT 走 raw JSON 旁路。既有討論資訊 payload 與清單、詳情回應的欄位 SHALL 逐位元不變。

#### Scenario: typed client 讀取搜尋回應

- **WHEN** typed client 以關鍵字 golden 與 sse 呼叫搜尋方法，server 回一筆命中（matches 含 kind 為 deferred、where 為 conclusion 的項目）
- **THEN** 請求路徑為 /discussions/search 且 q 為 `golden sse`；回應反序列化成功，該筆的 slug、archived 與 matches 三欄位值與 JSON 一致，缺席的 createdBy 與 kind 為 None

#### Scenario: 選填欄位缺席時仍可反序列化

- **WHEN** server 回應的命中項目缺 createdBy、kind、concluded 三個鍵，promotedTo 為空陣列
- **THEN** typed client 反序列化成功，三個選填欄位為 None、promotedTo 為空清單，不報錯

<!-- @trace
source: discuss-search-recall
updated: 2026-09-05
-->

---
### Requirement: 討論結論請求與回應攜帶保留旗標

討論結論端點的請求型別 SHALL 增選填 hold 欄位（camelCase 布林、serde default、缺席即 false、值為 false 時不序列化）——true 表示結論後記錄保留在途、不隨轉出變更封存。回應型別 SHALL 增 held 欄位（camelCase 布林、serde default、值為 false 時不序列化）——本次寫入後記錄是否帶保留旗標；缺席與 false 的讀取結果相同，與舊 server 不回報此事實時一致，不需哨兵欄位。既有 restaleFlagged 與 autoArchived 欄位的語意 SHALL 維持不變。

#### Scenario: 請求的 hold 缺席即 false 且 false 不出鍵

- **WHEN** 以只含 content 的 JSON 反序列化討論結論請求；再序列化一筆 hold 為 false 與一筆 hold 為 true 的請求
- **THEN** 前者 hold 讀為 false；後兩者的 JSON 分別無 hold 鍵、含 hold: true

#### Scenario: 回應的 held 缺席容錯與 true 出鍵

- **WHEN** 以空物件反序列化討論結論回應；再序列化一筆 held 為 true 的回應
- **THEN** 前者反序列化成功且 held 為 false、restaleFlagged 為空清單；後者 JSON 含 held: true

<!-- @trace
source: discussion-spinout-hold
updated: 2026-09-07T17:58:41+08:00
-->
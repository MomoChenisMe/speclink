# review-station Specification

## Purpose

TBD - created by archiving change 'code-review-stage'. Update Purpose after archive.

## Requirements

### Requirement: 審查工單的建立與追加

系統 SHALL 提供 speclink review add-round <change> --stdin：自 stdin 讀入一輪審查內容，於 `openspec/changes/<change>/review.md` 追加 `## Round N` 區段（工單不存在時建立並自 Round 1 起算）。每輪內容 SHALL 含 `**Scope**:` repo-root 相對路徑清單與零或多行分級 findings（severity ∈ CRITICAL／WARNING／SUGGESTION）。

結構化新輪次 SHALL 同時含 `**Phase**: discovery|validation` 與 `**Patch**: sha256:<64 lowercase hex>`；兩欄只出現其一、phase token 無效或 hash 格式無效時 SHALL 非零拒絕且工單零寫入。工單首個結構化 round SHALL 為 discovery；已有結構化 round 後追加者 SHALL 為 validation。既有兩欄皆缺席的 legacy round SHALL 保持可建立、追加與解析；legacy ticket 後 SHALL 能追加 validation round。

工單 SHALL 為 append-only：既有輪次不因追加而改寫。工單檔 SHALL NOT 註冊進 workflow schema，speclink status／validate 輸出 SHALL 不因工單存在而改變。

#### Scenario: 首輪建立 structured discovery 工單

- **WHEN** 對無工單的 change 執行 review add-round，stdin 含 Phase=discovery、合法 Patch、Scope 與 findings
- **THEN** exit code 0，`review.md` 建立且含 `## Round 1`、phase／patch 原文與 stdout 確認訊息

#### Scenario: 追加 validation 不改寫既有輪

- **WHEN** 對已有 structured Round 1 的工單追加 Phase=validation 的合法 Round 2
- **THEN** exit code 0，`review.md` 新增 `## Round 2` 且 Round 1 位元級不變

#### Scenario: 第二個 discovery 被拒絕

- **WHEN** structured Round 1 已是 discovery，又追加 Phase=discovery
- **THEN** exit code 非零、stderr 說明後續輪只能是 validation，工單位元級不變

#### Scenario: phase 與 patch 必須成對

- **WHEN** stdin 只有 Phase 沒有 Patch
- **THEN** exit code 非零、stderr 說明兩欄必須同時存在，工單零寫入

#### Scenario: legacy round 保持相容

- **WHEN** stdin 只含既有 Scope 與 findings，不含 Phase／Patch
- **THEN** add-round 維持既有成功行為，該輪 phase 與 patchHash 解析為 null

#### Scenario: change 不存在

- **WHEN** 對不存在的 change 執行 review add-round
- **THEN** exit code 非零，stderr 說明找不到變更，無檔案建立

#### Scenario: 內容缺少 Scope

- **WHEN** stdin 不含 `**Scope**:` 行
- **THEN** exit code 非零，stderr 說明格式要求，工單不變


<!-- @trace
source: converge-review-remediation-rounds
updated: 2026-08-03
code:
  - AGENTS.md
  - CLAUDE.md
  - crates/speclink-cli/src/commands.rs
  - crates/speclink-cli/src/main.rs
  - crates/speclink-cli/src/remote_commands.rs
  - crates/speclink-cli/tests/it/remote_verb_parity.rs
  - crates/speclink-cli/tests/it/review_verbs.rs
  - crates/speclink-core/assets/skills/apply.md
  - crates/speclink-core/assets/skills/review.md
  - crates/speclink-core/src/init.rs
  - crates/speclink-core/src/review.rs
  - crates/speclink-core/src/tasks.rs
  - crates/speclink-core/src/workspace.rs
  - crates/speclink-core/tests/golden/assets.lock
  - crates/speclink-core/tests/golden/claude.snapshot.md
  - crates/speclink-core/tests/golden/codex.snapshot.md
  - crates/speclink-core/tests/golden/neutral-cli.snapshot.md
  - crates/speclink-core/tests/golden/neutral-tool-call.snapshot.md
  - crates/speclink-core/tests/golden/remote-claude.marker.md
  - crates/speclink-core/tests/it/render_golden.rs
  - crates/speclink-host/src/change_diff.rs
  - crates/speclink-host/src/lib.rs
  - crates/speclink-protocol/src/command.rs
  - crates/speclink-remote/src/client.rs
  - crates/speclink-remote/tests/it/typed_client.rs
  - crates/speclink-server/src/routes.rs
  - crates/speclink-server/tests/it/review_api.rs
-->

---
### Requirement: 審查工單的讀取

系統 SHALL 提供 speclink review show <change> [--json]。人眼路徑 SHALL 將工單原文印至 stdout，`--no-color` 下不含 ANSI。`--json` payload SHALL 含 change:string、rounds:array、lastRound:object；rounds 每項 SHALL 含 index:number、phase:string|null、patchHash:string|null、scope:string[]、findings:array；findings 每項 SHALL 維持 severity:string、path:string、text:string。lastRound SHALL 與 rounds 末項同形。

Local fs、remote CLI、typed client 與 server response SHALL 輸出相同 camelCase 欄位與 null 語意。phase／patchHash 是刻意的 additive shape change；既有欄位名稱、型別與 findings 順序 SHALL 維持。

#### Scenario: 讀取 structured 兩輪 JSON

- **WHEN** 工單含 discovery Round 1 與 validation Round 2，執行 review show <change> --json
- **THEN** exit code 0、stdout 為合法 JSON、rounds 長度為 2、lastRound.index 為 2、lastRound.phase 為 validation、lastRound.patchHash 為 `sha256:` digest

#### Scenario: legacy JSON 使用 null

- **WHEN** legacy 工單 round 不含 Phase／Patch，執行 review show --json
- **THEN** phase 與 patchHash 明確輸出 null，既有 index、scope、findings 內容不變

#### Scenario: local 與 remote payload 同構

- **WHEN** 同一 structured ticket 分別經 local fs 與 remote server 讀取
- **THEN** 兩份 rounds／lastRound 的欄位集合、camelCase 名稱、null 與值逐項相同

#### Scenario: 無工單

- **WHEN** 對無工單的 change 執行 review show
- **THEN** exit code 非零，stderr 說明該 change 無審查工單


<!-- @trace
source: converge-review-remediation-rounds
updated: 2026-08-03
code:
  - AGENTS.md
  - CLAUDE.md
  - crates/speclink-cli/src/commands.rs
  - crates/speclink-cli/src/main.rs
  - crates/speclink-cli/src/remote_commands.rs
  - crates/speclink-cli/tests/it/remote_verb_parity.rs
  - crates/speclink-cli/tests/it/review_verbs.rs
  - crates/speclink-core/assets/skills/apply.md
  - crates/speclink-core/assets/skills/review.md
  - crates/speclink-core/src/init.rs
  - crates/speclink-core/src/review.rs
  - crates/speclink-core/src/tasks.rs
  - crates/speclink-core/src/workspace.rs
  - crates/speclink-core/tests/golden/assets.lock
  - crates/speclink-core/tests/golden/claude.snapshot.md
  - crates/speclink-core/tests/golden/codex.snapshot.md
  - crates/speclink-core/tests/golden/neutral-cli.snapshot.md
  - crates/speclink-core/tests/golden/neutral-tool-call.snapshot.md
  - crates/speclink-core/tests/golden/remote-claude.marker.md
  - crates/speclink-core/tests/it/render_golden.rs
  - crates/speclink-host/src/change_diff.rs
  - crates/speclink-host/src/lib.rs
  - crates/speclink-protocol/src/command.rs
  - crates/speclink-remote/src/client.rs
  - crates/speclink-remote/tests/it/typed_client.rs
  - crates/speclink-server/src/routes.rs
  - crates/speclink-server/tests/it/review_api.rs
-->

---
### Requirement: 蓋章守門與蓋章效果

系統 SHALL 提供 `speclink review stamp <change> [--accept]`。守門條件：(1) change 的任務全數完成；(2) 工單末輪零未解 findings——`--accept` SHALL 僅豁免條件 (2)。守門通過時系統 SHALL 於同一原子寫入內：將 `reviewed_at`／`reviewed_by`／`reviewed_with`／`reviewed_tasks_total`（蓋章時任務總數）／`reviewed_scope`（指紋清單）寫入該 change 的 `.openspec.yaml`，並刪除 `review.md`。不得出現「章已寫入而工單仍存在」的中間狀態。

#### Scenario: 任務未全完成即拒絕

- **WHEN** change 的任務為 4/5 完成時執行 `review stamp`
- **THEN** exit code 非零，stderr 說明任務未全數完成，metadata 與工單皆不變

#### Scenario: 末輪有未解 findings 且未帶 --accept

- **WHEN** 工單末輪含至少一筆 findings 且執行 `review stamp`（無 `--accept`）
- **THEN** exit code 非零，stderr 說明有未解事項並提示 `--accept` 或先修正重審

#### Scenario: 帶保留蓋章

- **WHEN** 同上情境但帶 `--accept`
- **THEN** exit code 0，章寫入且工單刪除

#### Scenario: 乾淨蓋章

- **WHEN** 任務 5/5 完成且工單末輪 findings 為空時執行 `review stamp`
- **THEN** exit code 0，`.openspec.yaml` 含五個 reviewed 欄位且 `review.md` 不存在

##### Example: 蓋章寫入的任務錨

- **GIVEN** change 有 5 個任務全數勾選，工單 Round 2 的 findings 為空
- **WHEN** `review stamp` 成功
- **THEN** `.openspec.yaml` 內 `reviewed_tasks_total` 為 5


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
### Requirement: 內容指紋錨與失效判定

蓋章時系統 SHALL 以工單各輪 Scope 的聯集為範圍，逐檔記錄 `{ path, hash }` 至 `reviewed_scope`：path 為 repo-root 相對且以 `/` 分隔（Windows 路徑正規化後寫入）；hash 為檔案內容經行尾 CRLF→LF 正規化後的 SHA-256。聯集中已不存在於工作樹的檔（修正過程刪除或改名）SHALL 排除於指紋之外、不入 `reviewed_scope`，蓋章不因死檔而失敗；聯集全數不存在時 SHALL 拒絕蓋章（exit code 非零並列出檔案）；存在但無法以 UTF-8 讀取的檔 SHALL 仍使蓋章失敗。remote 模式下工作樹持有者 SHALL 於 stamp 請求明示宣告已不存在的檔（`missing` 清單），server SHALL 驗證「提交指紋的 path 集合 ∪ missing ＝工單聯集且兩者不相交」，分割不成立即拒；`missing` 缺席讀作空清單（既有嚴格集合相等）。失效判定 SHALL 為：任務狀態不再是「蓋章當時任務總數的全完成」，或任一 scope 檔內容雜湊不符（含檔案已不存在）→ 該章判為過期（stale）；全部相符 → 有效（fresh）。判定結果 SHALL 僅經 desktop 協定曝光，CLI 輸出不含。

#### Scenario: 修正刪除早輪 scope 檔後仍可蓋章

- **WHEN** 工單 Round 1 的 Scope 含檔案 A 與 B，修正過程刪除 B 後任務全完成且末輪零 findings，執行 `review stamp`
- **THEN** 蓋章成功，`reviewed_scope` 僅含 A 的指紋，不含 B

#### Scenario: 聯集全數消失時拒絕蓋章

- **WHEN** 工單各輪 Scope 的所有檔案皆已不存在於工作樹，執行 `review stamp`
- **THEN** exit code 非零，stderr 列出已消失的檔案並指引還原或 `review discard`

#### Scenario: 蓋章後修改範圍檔

- **WHEN** 蓋章成功後修改任一 scope 檔的內容
- **THEN** 失效判定為 stale

#### Scenario: 行尾差異不觸發失效

- **WHEN** scope 檔內容僅行尾由 LF 變為 CRLF
- **THEN** 失效判定仍為 fresh

##### Example: 指紋比對

- **GIVEN** `reviewed_scope` 含 `{ path: "crates/a/src/lib.rs", hash: H1 }` 且該檔現值雜湊為 H1
- **WHEN** 該檔追加一行後重新判定
- **THEN** 現值雜湊不為 H1，判定 stale


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
### Requirement: 放棄審查

系統 SHALL 提供 `speclink review discard <change>`：刪除工單、不寫任何 metadata。

#### Scenario: 放棄既有工單

- **WHEN** 對有工單的 change 執行 `review discard`
- **THEN** exit code 0，`review.md` 不存在，`.openspec.yaml` 不變

#### Scenario: 無工單可放棄

- **WHEN** 對無工單的 change 執行 `review discard`
- **THEN** exit code 非零，stderr 說明無工單


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
### Requirement: 封存的未結工單守門

`speclink archive` 偵測到 change 存在 `review.md` 時 SHALL 預設拒絕封存（exit code 非零），stderr 列出三種處置：完成蓋章（review stamp）、放棄審查（review discard）、或以 `--carry-review` 明示帶走。帶 `--carry-review` 時工單 SHALL 隨 change 目錄移入封存區。無工單時 archive 的行為與輸出 SHALL 維持既有位元級輸出不變（本需求僅在「工單存在」的新情境觸發，屬刻意的行為新增）。

#### Scenario: 有工單預設拒絕

- **WHEN** 對有 `review.md` 的 change 執行 `speclink archive`
- **THEN** exit code 非零，stderr 同時含 stamp、discard 與 `--carry-review` 三種處置指引，change 未被搬移

#### Scenario: 明示帶走

- **WHEN** 同上情境但帶 `--carry-review`
- **THEN** 封存成功，封存目錄內含 `review.md`

#### Scenario: 無工單行為不變

- **WHEN** 對無工單的 change 執行 `speclink archive`
- **THEN** 行為與導入本功能前完全一致


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
### Requirement: CLI 清單輸出的相容性釘住

`speclink list --json` 的輸出 SHALL 不因 metadata 含任何 reviewed 欄位而改變形狀：含全套 reviewed 欄位的 change 與不含者 SHALL 序列化出同形（位元級同構的欄位集合）。

#### Scenario: 帶章的 change 不外洩新欄位

- **WHEN** 某 change 的 `.openspec.yaml` 含全套 reviewed 欄位時執行 `speclink list --json`
- **THEN** 該 change 的 JSON 項目與無 reviewed 欄位的 change 具有相同的欄位集合


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
### Requirement: remote 模式下的動詞行為

review 動詞家族於 remote workspace SHALL 經 store 文件管道讀寫（工單與 metadata 皆不直接落地本地投影）。revision 衝突、離線或認證失效時 SHALL 以非零 exit code 與 stderr 訊息回報，行為與既有寫入動詞一致；本地 Context Projection 於下次讀取時反映最新工單狀態。

#### Scenario: 離線時追加輪次

- **WHEN** remote workspace 離線狀態下執行 `review add-round`
- **THEN** exit code 非零，stderr 回報連線錯誤，遠端與本地投影皆不變

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
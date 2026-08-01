# review-skill Specification

## Purpose

TBD - created by archiving change 'code-review-stage'. Update Purpose after archive.

## Requirements

### Requirement: 審查技能的生成與正典化

`speclink update` SHALL 生成 `/speclink-review` 技能檔至 claude 與 codex 兩工具的技能目錄，內容以引擎內的正典模板為準（golden 對照涵蓋）。同次更新 SHALL 將生成之 CLAUDE.md／AGENTS.md 的 workflow 行改為含並行品質站的版本（`discuss? → propose → apply ⇄ ingest → (review? ∥ verify?) → archive`），並於技能使用清單加入審查站的觸發時機（實作完成、封存之前、由使用者判斷是否執行）。

#### Scenario: 技能檔生成

- **WHEN** 於已啟用 speclink 的專案執行 `speclink update`
- **THEN** claude 與 codex 的技能目錄各出現 speclink-review 技能檔，且內容與 golden 對照一致

#### Scenario: workflow 行更新

- **WHEN** `speclink update` 完成後讀取生成的 CLAUDE.md
- **THEN** workflow 行含 `(review? ∥ verify?)` 且技能清單含審查站條目


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
### Requirement: 審查流程的技能行為

技能文件 SHALL 指示主線 orchestrator 依序執行：(1) 選定 change（無參數時比照 verify 的選擇邏輯）；(2) 守門自檢——任務未全數完成即停止並說明；(3) 定界範圍——工單存在且末輪帶 findings 時以 `review show --json` 的末輪 findings 檔集為續輪範圍；工單存在但末輪零 findings（如蓋章曾被拒的乾淨輪）時 SHALL 以同一 payload 的各輪 Scope 聯集（去除已不存在的檔案）定界，不得重掃整個 change；無工單時以 touched 記錄為初審範圍，touched 缺失時詢問使用者 git 比較基準；(4) 讀取 change artifacts 作為判準脈絡；(5) 平行派出 Standards（repo 慣例文件＋smell baseline 正典原文——見「Standards 軸的 smell baseline」、repo 文件優先）與 Correctness（bug 獵捕）兩個 read-only sub-agent，各以 400 字內回報並以 CRITICAL／WARNING／SUGGESTION 分級；(6) 兩軸結果並列呈現、不合併不重排、附一行總結；(7) 以 `review add-round` 寫入工單。artifacts 稀薄時 sub-agent SHALL 僅憑 code 與測試判斷，不臆造需求。

#### Scenario: 任務未完成即停

- **WHEN** 對任務 3/5 的 change 執行 `/speclink-review`
- **THEN** 技能停止並說明審查站要求任務全數完成，不派出 sub-agent、不寫工單

#### Scenario: 續輪只審上輪範圍

- **WHEN** 工單已有 Round 1（findings 涉及 2 個檔）時執行 `/speclink-review`
- **THEN** 本輪範圍為該 2 檔加上修正過程新動到的檔，不重掃整個 change

#### Scenario: 末輪零 findings 的續輪以各輪 Scope 聯集定界

- **WHEN** 工單末輪 findings 為空（零 findings 輪蓋章曾因任務回退被拒而留存）時執行 `/speclink-review`
- **THEN** 本輪範圍為工單各輪 Scope 的聯集（去除已不存在的檔案），不重掃整個 change，也不因末輪空集合而無法記錄新輪


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
### Requirement: Standards 軸的 smell baseline

技能文件的 Standards sub-agent 指示 SHALL 逐字內嵌 smell baseline 正典原文（來源：Matt Pocock code-review skill，MIT；正典文字見 design「D7a」）：引言（"On top of whatever the repo documents…"）、兩條約束規則（The repo overrides——repo 文件化標準勝出、被其背書者壓下不報；Always a judgement call——smells 一律為 "possible X" 標籤式啟發、永不作 hard violation，且 tooling 已強制的規則跳過不報）、"(Refactoring, ch.3)" 出處，以及 12 條 smells 逐項（what it is → how to fix）。12 個專有名詞 SHALL 原文不動：Mysterious Name、Duplicated Code、Feature Envy、Data Clumps、Primitive Obsession、Repeated Switches、Shotgun Surgery、Divergent Change、Speculative Generality、Message Chains、Middle Man、Refused Bequest。生成之技能檔 SHALL 含一行出處註記。

#### Scenario: 生成技能檔含完整 baseline

- **WHEN** `speclink update` 生成 speclink-review 技能檔（claude 與 codex）
- **THEN** 兩份檔內皆含 12 條 smells 專有名詞逐項、兩條約束規則與 "(Refactoring, ch.3)" 出處，內容與 design「D7a」正典文字逐字一致

##### Example: 專有名詞逐一在場

- **GIVEN** 生成的 speclink-review 技能檔
- **WHEN** 逐一檢索 Mysterious Name、Duplicated Code、Feature Envy、Data Clumps、Primitive Obsession、Repeated Switches、Shotgun Surgery、Divergent Change、Speculative Generality、Message Chains、Middle Man、Refused Bequest
- **THEN** 12 個名稱皆命中，且各項皆帶「→」修法句


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
### Requirement: 審查產出的語言綁定

技能文件 SHALL 將 workflow config 解析出的 locale（守門自檢步驟所取 payload 的 `locale` 欄位）綁定整條審查產出鏈：兩軸 sub-agent 指示 SHALL 攜帶該語言並要求 finding 描述以該語言撰寫；並列呈現與 `review add-round` 寫入工單的 findings 描述 SHALL 與 sub-agent 產出同語言，主線 SHALL NOT 另行翻譯。severity 標籤（CRITICAL／WARNING／SUGGESTION）、軸前綴（`Standards:`／`Correctness:`）、檔案路徑與指令行 SHALL 維持英文；工單固定骨架屬動詞所有文法（見 review-station 規格），不在本條範圍。locale 未設定時，sub-agent 報告、呈現與工單記錄 SHALL 為英文。smell baseline 正典原文（design「D7a」）屬 sub-agent 指示本文，SHALL 維持逐字英文內嵌，不受本條影響。

#### Scenario: locale tw 時工單 findings 為中文

- **WHEN** 專案 openspec/config.yaml 設 locale: tw，審查輪產出 findings 並以 `review add-round` 寫入工單
- **THEN** 該輪 findings 行的描述文字為繁體中文，severity 標籤與 `Standards:`／`Correctness:` 前綴維持英文

#### Scenario: locale 未設定時全英文

- **WHEN** 專案未設定 locale，執行一輪審查
- **THEN** sub-agent 報告、並列呈現與工單 findings 描述皆為英文


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
### Requirement: 審查後的迴圈與收尾

技能 SHALL 於每輪呈現後依 findings 有無分流：有 findings 時依「審查結果的裁量分類」的推薦詢問使用者三選項——修正後重審（有必修項時列出必修清單）／接受現狀蓋章（`review stamp --accept`）／先不蓋章結束（工單保留供後續接手）；findings 為空時執行 `review stamp` 蓋章。修正 SHALL 一律回主線執行（依專案 TDD 慣例），sub-agent 不得修改任何檔案。互動詢問於不支援選單工具的環境 SHALL 以純文字詢問並等待回覆。

#### Scenario: 乾淨輪自動蓋章

- **WHEN** 某輪兩軸皆零 findings
- **THEN** 技能執行 `review stamp` 並回報蓋章完成

#### Scenario: 先不蓋章離場

- **WHEN** 使用者於三選項選擇「先不蓋章」
- **THEN** 技能結束，工單保留，metadata 不變


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
### Requirement: 審查結果的裁量分類

技能 SHALL 於兩軸結果並列呈現後、詢問使用者之前，對本輪每筆 finding 給出處置分類並隨報告一併呈現（不改動工單記錄格式）：**必修**——CRITICAL 級、Correctness 軸判定有現實觸發路徑的 bug（含 WARNING 級）、文件化 repo 標準的明確違反；**可裁**——"possible X" 措辭的 smell 判斷與 SUGGESTION 級事項，每筆附一行修繕成本與效益的裁量理由。三選項詢問 SHALL 帶明確推薦：仍有必修項時推薦「修正後重審」並列出必修清單；僅剩可裁項時推薦「接受現狀蓋章」並說明保留事項將以 `review stamp --accept` 記錄。

#### Scenario: 有必修項時推薦修正

- **WHEN** 某輪 findings 含一筆 CRITICAL 與三筆 possible-X 措辭的 SUGGESTION
- **THEN** 呈現的分類為 1 筆必修、3 筆可裁，三選項詢問以「修正後重審」為推薦選項且附必修清單

#### Scenario: 僅剩可裁項時推薦接受

- **WHEN** 某輪 findings 僅含 possible-X 措辭的 WARNING／SUGGESTION，無 CRITICAL、無現實路徑 bug、無文件化標準違反
- **THEN** 三選項詢問以「接受現狀蓋章」為推薦選項，並說明將以 `review stamp --accept` 帶保留蓋章


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
### Requirement: 修復迴圈的驗證門

使用者選擇「修正後重審」時，技能 SHALL 於修正完成後、派出下一輪 sub-agent 之前，執行專案的完整建置與測試驗證（依 repo 慣例的全量測試指令）並要求全數通過；未通過時 SHALL 先修復至全綠，不得帶著建置或測試失敗進入下一輪。

#### Scenario: 修正引入的編譯錯誤被驗證門攔下

- **WHEN** 修正過程改壞另一呼叫端導致專案建置失敗
- **THEN** 技能於派出下一輪 sub-agent 之前發現並修復，下一輪開始時建置與測試全綠


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
### Requirement: 已接受事項的續輪前饋

已裁定接受而未修正的 findings，技能於續輪 SHALL 雙軌處置：(1) 續輪 sub-agent 的指示 SHALL 附上該清單並明令不得重報同一事項或其近似變體；(2) 續輪記錄 SHALL 由主線將這些事項原樣帶入該輪 findings 清單，並於行末附結構性標記 `(accepted)`（比照 severity 標籤維持英文、不隨 locale 翻譯），使末輪工單忠實反映殘留保留事項，蓋章走 `review stamp --accept`。跨 session 接手時，技能 SHALL 以末輪帶 `(accepted)` 標記的行重建不重報清單。

#### Scenario: 接受過的事項不再重報

- **WHEN** Round N 的一筆 possible Duplicated Code 裁定接受後執行下一輪
- **THEN** 續輪 sub-agent 指示含該事項的不重報清單，且 Round N+1 的工單記錄由主線原樣帶入該筆事項並以 `(accepted)` 標記收尾

#### Scenario: 跨 session 重建不重報清單

- **WHEN** 另一 session 對末輪含 `(accepted)` 標記行的工單執行 `/speclink-review`
- **THEN** 該 session 的 sub-agent 指示以標記行重建不重報清單，標記事項不被重報

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
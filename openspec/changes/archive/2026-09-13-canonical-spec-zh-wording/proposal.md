## Why

本專案對照 OpenSpec，規格分兩種：變更裡的 delta 規格（`openspec/changes/<name>/specs/`）與合併後的 specs（`openspec/specs/<capability>/spec.md`）。繁中文案把後者叫「正典規格」，但「正典」讀不出「合併後的定案」，使用者裁定改叫「正式規格」，與「delta 規格」成對；英文維持 specs／canonical specs 不改（討論 canonical-spec-zh-wording，2026-09-12）。

漂移其實已經發生：`openspec/LANGUAGE.md` 的「規格基準」「手冊」兩條定義、`baseline-skill` 與 `manual-skill` 兩份規格、CHANGELOG 與三份九月討論的結論都已寫「正式規格」，其餘地方仍是「正典規格」「正典 spec(s)」「規格正典」或光桿「正典」。同一個東西目前有五種寫法，而且沒有詞條擋它漂回去。

目標使用者是透過 AI 代理跑 SDD 的開發者／PO／PM；使用情境橫跨桌面規格頁（標題與空狀態）、手冊、使用者文件，以及 propose／archive／analyze 這幾站規格散文裡 delta 與正式規格的對照句。

## What Changes

- **`openspec/LANGUAGE.md` 新增「正式規格」詞條**：definition 點名對照詞「delta 規格」與檔案位置；avoid 收「正典規格」「正典 spec」「正典 specs」「規格正典」（含中文字、無限定語，進機械守門）與「正典（指這個規格集合時）」（帶語境限定，守門跳過——光桿「正典」另有別義）；why 記裁定日期與不改的別義清單。
- **使用者可見面同批清零**（守門一加 avoid 立刻紅，必須同一批）：`packages/ui/src/i18n.tsx` 的 specs.heading 改「正式規格」、specs.empty 改「此專案尚無正式規格」，`packages/ui/src/__tests__/specList.test.tsx` 的斷言同步；README 與 docs 共 11 份的複合寫法全改，光桿「正典」指 specs 這一側的一併改（如 README「共用同一份規格正典」→「共用同一份正式規格」）。
- **正式規格散文 25 份直接改字**：複合寫法全改；光桿「正典」依判準逐句改——指 `openspec/specs/` 合併後的規格集合才改，正典詞彙、正典值、正典化、正典載入、正典順序、「本文是⋯的正典」不動。含 16 個 Requirement／Scenario 標題改名（例：spec-validation「validate --specs 驗證正典規格」→「validate --specs 驗證正式規格」、archive-merge「snapshot 先於正典寫入」→「snapshot 先於正式規格寫入」）。直接改字、不經封存合併，不動 `@trace` 時戳。
- **手冊 16 頁直接改字、不重生**：直編規格不動 `@trace` 時戳，手冊不會被標「可能過期」，重生不會觸發，所以手動改；`generated` 欄位維持原值。
- **程式碼註解與測試只改兩類**：複合寫法（crates 16 檔、apps 6 檔、packages 7 檔、scripts 1 檔），以及引用了改名標題的註解（如 newcmd 與 new_artifact 測試引用的「命中正典名稱照常放行」、analyzer 測試引用的「MODIFIED 需求不在正典」）。光桿「正典」註解不動；例外是審查站點名的同句混用（同一句或同一段一半已改、一半還是「正典」），這種一併改齊。
- **零行為變化、零 delta、零版號**：不改任何識別符、CLI 旗標、`--json` 形狀或技能資產。

### 相容性影響

- 人眼輸出：桌面 zh-TW 規格頁標題與空狀態兩條文字改字面；CLI 輸出為英文，不受影響。`--json` 欄位名與 shape 完全不變。
- 回歸對照：技能資產零命中，golden 與 assets.lock 不動，無 ASSET_VERSION 進版。
- 遷移：無。既有工作區不需執行任何指令。

### 影響的 crate 與 app

`speclink-core`、`speclink-cli`、`speclink-server`（皆只動註解與測試字串）、`apps/desktop`（core 與 src-tauri 註解、App 測試註解）、`packages/ui`（兩條 i18n 字串、一個測試斷言、註解）。不動 `speclink-host`、`speclink-node`、`apps/server-web` 的任何檔案。

### 技能與工具影響

無。技能資產（`crates/engine/speclink-core/assets/skills/`）與 init 範本零命中，claude 與 codex 兩工具的產出不變。新詞條經 `speclink language show` 立即對代理生效，不需再生。

## Capabilities

### New Capabilities

（無）

### Modified Capabilities

（無）——理由與規格掃描結果見 Impact 的「規格掃描」一段。

## Impact

- 規格掃描（step-3）：命中三份——ui-copy-vocabulary（守門詞由 LANGUAGE.md 動態解析，新增詞條自動生效，需求不變）、user-documentation（要求繁中文件用詞彙表的正典詞，本變更是讓文件回到該要求）、manual-pages（頁格式契約不變，只改內文字面）；desktop-app 未把規格頁標題字面寫進需求。16 個標題改名是字面更動、不是行為變更，所以無 delta；規格散文直接改字，先例為 spec-purpose-backfill（直編 67 份規格、零 delta）。
- Affected specs: 無 delta。直接改字的正式規格 25 份：openspec/specs/archive-merge/spec.md、openspec/specs/archive-skill/spec.md、openspec/specs/capability-naming-guard/spec.md、openspec/specs/change-analysis/spec.md、openspec/specs/change-lifecycle/spec.md、openspec/specs/context-projection/spec.md、openspec/specs/delivery-baseline/spec.md、openspec/specs/desktop-app/spec.md、openspec/specs/desktop-manual-page/spec.md、openspec/specs/discuss-skill/spec.md、openspec/specs/drift-computation/spec.md、openspec/specs/manual-pages/spec.md、openspec/specs/manual-skill/spec.md、openspec/specs/phase2-acceptance/spec.md、openspec/specs/propose-skill/spec.md、openspec/specs/remote-workspace-data/spec.md、openspec/specs/server-context-api/spec.md、openspec/specs/server-read-api/spec.md、openspec/specs/spec-validation/spec.md、openspec/specs/task-identity/spec.md、openspec/specs/trace-skill/spec.md、openspec/specs/trace-verb/spec.md、openspec/specs/user-documentation/spec.md、openspec/specs/verb-contract/spec.md、openspec/specs/verify-evidence/spec.md
- Affected code:
  - New: 無
  - Modified:
    - openspec/LANGUAGE.md
    - packages/ui/src/i18n.tsx
    - packages/ui/src/__tests__/specList.test.tsx
    - README.md
    - docs/getting-started.zh-TW.md、docs/workflow.zh-TW.md、docs/product-status.zh-TW.md、docs/roadmap.zh-TW.md、docs/sdk-node.zh-TW.md、docs/verb-contract.zh-TW.md、docs/configuration.zh-TW.md、docs/server-deployment.zh-TW.md、docs/design/implementation-refactor-roadmap.zh-TW.md、docs/design/platform-architecture.zh-TW.md
    - 手冊 16 頁：openspec/manual/about.md、openspec/manual/analyze.md、openspec/manual/archive.md、openspec/manual/baseline.md、openspec/manual/data-layout.md、openspec/manual/desktop-browse.md、openspec/manual/desktop-manual.md、openspec/manual/discuss.md、openspec/manual/drift-ingest.md、openspec/manual/index.md、openspec/manual/manual.md、openspec/manual/migrate.md、openspec/manual/propose.md、openspec/manual/remote-context.md、openspec/manual/remote-overview.md、openspec/manual/trace.md
    - crates/engine/speclink-core/src/lifecycle/capname.rs、crates/engine/speclink-core/src/lifecycle/model.rs、crates/engine/speclink-core/src/lifecycle/trace.rs、crates/engine/speclink-core/src/lifecycle/newcmd.rs、crates/engine/speclink-core/src/lifecycle/archive/tests.rs、crates/engine/speclink-core/src/command/tests.rs、crates/engine/speclink-core/src/quality/validate.rs、crates/engine/speclink-core/src/quality/validate/tests.rs、crates/engine/speclink-core/src/quality/analyzer/tests.rs
    - crates/adapters/speclink-cli/src/verbs/checks.rs、crates/adapters/speclink-cli/tests/it/manual_pages_dir_ignored.rs、crates/adapters/speclink-cli/tests/it/new_artifact.rs、crates/adapters/speclink-cli/tests/it/remote_read_path.rs、crates/adapters/speclink-cli/tests/it/trace.rs、crates/adapters/speclink-cli/tests/it/validate_specs.rs
    - crates/host/speclink-server/tests/it/phase2_chain.rs
    - apps/desktop/core/src/manual.rs、apps/desktop/core/src/query.rs、apps/desktop/src-tauri/src/lib.rs、apps/desktop/src-tauri/src/remote.rs、apps/desktop/src-tauri/tests/it/remote_data.rs、apps/desktop/src/__tests__/App.test.tsx
    - packages/ui/src/adapter.ts、packages/ui/src/components/SpecDrawer.tsx、packages/ui/src/components/SpecList.tsx、packages/ui/src/trace.ts、packages/ui/src/__tests__/delta.test.ts、packages/ui/src/__tests__/richDrawer.test.tsx、packages/ui/src/__tests__/trace.test.ts
    - scripts/release/delivery-gate.test.mjs
  - Removed: 無
- 不動：openspec/changes/archive/（82 檔）、openspec/discussions/archive/（8 檔）、CHANGELOG.md、apps/desktop/src/release-notes/release-notes.json、openspec/config.yaml、技能資產、任何識別符與 CSS 類名。

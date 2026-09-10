## Why

封存的守門鏈今天在三個入口各跑一遍：

| 入口 | 守門序 |
| --- | --- |
| `run_archive`（crates/speclink-core/src/command/mod.rs，CLI 單筆與 server 走這裡） | linked worktree → meta → 未結工單 → merge 守門 → [帶 `--mark-tasks-complete` 時：章失效 → **代勾寫入 tasks.md**] → `archive()` |
| `archive()`（crates/speclink-core/src/archive.rs，desktop 直呼） | linked worktree → meta → 未結工單 → 任務完成度（旗標豁免）→ 章失效 → 同日撞名 → 結構 validate → 計畫階段（merge 守門）→ 提交階段 |
| bulk（crates/speclink-cli/src/verbs/lifecycle.rs） | merge 守門 → 結構 validate → 任務完成度，任一不過就「跳過」；其餘守門交給 `Command::Archive`，失敗即「中止」 |

`run_archive` 前四道與 `archive()` 前三道逐字重複。存在的理由只有一個：`--mark-tasks-complete` 的代勾寫入落在 `archive()` **外面**，而規格要求拒絕路徑對 tasks.md 零寫入，所以 `run_archive` 得在寫入前把 `archive()` 內部的守門全部再抄一遍、順序也要一樣（註解明寫「core archive flow gates again for entry points that call it directly (desktop)」）。validate-merge-gate 剛落地時就必須同時動 `run_archive`、`archive()`、bulk 三處；下一道守門也是三處。

而且抄得不完整：`archive()` 內的「同日撞名」與「結構 validate」兩道在代勾**之後**才跑。今天對一個結構不合法的 change 執行 `speclink archive <name> --mark-tasks-complete`，封存被拒、但 tasks.md 已經被全勾——違反 archive-merge「驗證階段的任何違規 SHALL 使封存在零檔案效果下結束」。

本 change 是討論 improve-lifecycle-layer 結論的最後一刀（候選 3；候選 4 已於 lifecycle-station-anchors 落地）。**目標使用者與情境**：透過 AI 代理跑 SDD 的開發者，`/speclink-archive` 的單筆、`speclink archive --all` 的批次、desktop 的封存按鈕、server 的封存路由。

## What Changes

- **代勾寫入搬進 `archive()`**，落在計畫階段之後、提交階段的第一個效果——所有守門（環境、meta、未結工單、任務完成度、章失效、同日撞名、結構 validate、merge 計畫）全過，才有第一個寫入。`archive()` 本來就是「守門全過才有檔案效果」的擁有者，代勾是檔案效果，歸它。
- **`run_archive` 縮成兩步**：`resolve_change` → `archive()`。`guard_linked_worktree`／`guard_meta`／`guard_open_tickets`／`merge_violations`＋`merge_refusal`／`guard_stale_stamps` 五道在 command 層的重複呼叫全部刪除；`guard_meta` 也不需要——`archive()` 內的 `require_valid_meta` 產生的 `MetaError` 經既有 `classify` 映射為同一個 `invalid_config` 與同一段訊息。
- **`guard_*` 縮回 archive.rs 私有**：`guard_linked_worktree`、`guard_open_tickets`、`guard_stale_stamps`、`merge_refusal` 由 `pub(crate)` 改 `fn`；`merge_violations` 維持 `pub`（validate.rs 與 drift.rs 仍共用，規格 archive-merge「過期判定單源共用」）。
- **bulk 改吃 `archive::skip_reason(store, change, opts) -> Option<SkipReason>`**：`SkipReason` 三值列舉 `MergeRefused(Vec<MergeViolation>)`／`StructuralInvalid`／`TasksIncomplete { complete, total }`，內含 `skip_specs`／`no_validate`／`mark_tasks_complete` 三個旗標的豁免語意，判定順序固定為 merge → validate → tasks、命中第一條即回。bulk 保留自己的三段字串（`"{n} delta operation(s) archive would refuse — run /speclink-drift {name}"` 含 Purpose 點名、`"validation failed"`、`"tasks incomplete ({c}/{t})"`）與「取第一條跳過」的既有優先序，只是不再自己算條件。
- **不動**：`archive()` 的拒絕字串（任務完成度 `Refusal`、`"Validation failed:\n…"`、`merge_refusal`、章失效、未結工單、linked worktree）逐字凍結；bulk 的「跳過 vs 中止」分類；desktop `archive_with` 與 server 路由的呼叫方式（它們不傳 `mark_tasks_complete`）；in-progress 標記在封存時不動。

**相容性影響**：CLI 人眼輸出與 `--json`、server wire、desktop payload 逐位元不變；三個入口的守門順序與訊息對齊到 `archive()` 一份。可觀察差異有兩條。第一條是既有規格已禁止的路徑收斂：帶 `--mark-tasks-complete` 的單筆封存被「同日撞名」或「結構 validate」拒絕時，今天 tasks.md 已被全勾、改後逐位元不變（規格 archive-merge「兩階段合併計畫與零半套寫入」與 change-lifecycle「單筆封存的任務完成度守門」的零效果要求已涵蓋，不補 delta，補測試）。desktop 直呼 `archive()` 帶旗標時（目前沒有這種呼叫）從此與 CLI 同語意。第二條是拒絕訊息的先後：今天 `run_archive` 的 merge 守門跑在 `archive()` 之前，所以 CLI 單筆與 server 路由對「delta 過期」的 change 一律先收到 merge 拒絕，即使它同時任務未完成、章失效、同日撞名或結構不合法；desktop 直呼 `archive()` 則先收到後面那四道的拒絕。改後四個入口一律走 `archive()` 的順序：任務完成度 → 章失效 → 同日撞名 → 結構 validate → merge，merge 拒絕成為最後一道，與 desktop 今天的行為一致（規格對 merge 守門與其他四道的先後無規定）。不涉及 CLI 子指令、旗標、stdin、exit code 的變更；不涉及設定欄位；不涉及生成技能；golden 零變動。

## Non-Goals

（本 change 建 design.md，範圍排除與已否決做法記錄在 design 的 Goals／Non-Goals。）

## Capabilities

### New Capabilities

（none）——step 3 掃描：change-lifecycle（單筆封存的任務完成度守門、封存的 linked worktree 環境守門、封存的章失效守門）、archive-merge（封存合併 fail-closed 守門、兩階段合併計畫與零半套寫入、過期判定單源共用）、review-station／verify-station（封存的未結工單守門）已涵蓋三個入口的全部可觀察行為；本 change 只換守門的落點。

### Modified Capabilities

（none）——守門的順序、訊息、豁免旗標語意全部不變；拒絕路徑零寫入是既有要求。

## Impact

- Affected specs: 無 delta
- Affected crates／apps: speclink-core、speclink-cli
- Affected code:
  - New: （無新檔）
  - Modified: crates/speclink-core/src/archive.rs（代勾寫入、`SkipReason`／`skip_reason`、`guard_*` 可見度、既有測試 `mark_tasks_complete_flag_passes_the_gate_without_pre_write` 的斷言改為「封存後 tasks.md 已全勾」）、crates/speclink-core/src/command/mod.rs（`run_archive` 與守門序回歸測試）、crates/speclink-cli/src/verbs/lifecycle.rs（bulk 預檢）、crates/speclink-core/src/validate.rs（`validate_change_structural` 收 `pub(crate)`、移除未使用的 schema 參數）、crates/speclink-cli/tests/it/validate_specs.rs（一條 fixture 先勾齊任務）
  - Removed: （無檔案刪除；刪除的是 `run_archive` 的守門鏈與 bulk 的三段自算）
- 測試面：CLI `archive_readiness_gate`／`archive_merge_gate`／`manual_task_gates` 家族與 command 層封存測試斷言不改；新增 archive.rs「帶旗標被結構 validate 拒絕時 tasks.md 逐位元不變」、`skip_reason` 三旗標豁免、以及「任務／章失效／結構 validate 三道先於 merge 拒絕」的單元測試，command 層再釘一條「任務守門先於 merge」。

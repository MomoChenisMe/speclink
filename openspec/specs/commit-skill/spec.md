# commit-skill Specification

## Purpose

/speclink-commit 技能的確認閘門：提交前把實際要進這次 commit 的檔案清單攤給使用者過目，使用者看到的就是最後簽下去的那一份。本 capability 保證跨變更混改的工作樹裡，不會有無關檔案被順手掃進某個變更的提交。

## Requirements

### Requirement: commit 確認閘門所見即所簽

內嵌 speclink-commit 技能（事實來源 crates/engine/speclink-core/assets/skills/commit.md，經 init 與 update 渲染至各工具技能目錄）SHALL 規定以下確認流程：commit 訊息 SHALL 於使用者確認之前生成；commit 計畫（分組檔案清單）與 commit 訊息 SHALL 在呼叫確認工具之前以可見文字一次輸出；確認問題的文字 SHALL NOT 指涉對話中未曾輸出的內容。本能力屬 Speclink 自身延伸；渲染產物內容由 speclink-core 的 render_golden 測試（cargo test）保護，golden 快照更新屬刻意變更。

#### Scenario: 渲染產物將訊息生成排在確認之前

- **WHEN** 執行 speclink init 或 speclink update 渲染 claude 與 codex 工具的技能檔
- **THEN** 產出的 speclink-commit 技能檔 SHALL 將「生成 commit 訊息」步驟置於使用者確認步驟之前，且確認步驟 SHALL 引用先前已輸出的計畫與訊息

#### Scenario: 渲染產物含可見性守則

- **WHEN** 檢視渲染產出的 speclink-commit 技能檔的 Guardrails 段落
- **THEN** 該段落 SHALL 含以下守則：呼叫確認工具前，commit 計畫與 commit 訊息必須已作為可見文字輸出；確認問題不得指涉未曾輸出的內容

#### Scenario: archive 子流程後重新確認

- **WHEN** 使用者於確認閘門選擇 Archive first, then commit together 且 archive 子流程執行完成
- **THEN** 技能檔 SHALL 規定重新輸出更新後的 commit 計畫與含 Archived: yes 的 commit 訊息，並再次經確認工具確認後才執行暫存與提交

#### Scenario: 確認工具不可用時的降級

- **WHEN** 執行環境沒有 AskUserQuestion 工具
- **THEN** 技能檔 SHALL 規定以純文字提出相同的確認問題並等待使用者回覆，且同樣 SHALL 先輸出計畫與訊息後才提問


<!-- @trace
source: spectra-legacy-cleanup
updated: 2026-07-27
code:
  - README.en.md
  - README.md
  - apps/desktop/src/App.tsx
  - apps/desktop/src/components/ProjectTabs.tsx
  - apps/desktop/src/index.css
  - crates/speclink-cli/src/color.rs
  - crates/speclink-cli/src/commands.rs
  - crates/speclink-cli/src/main.rs
  - crates/speclink-cli/tests/discuss_promote_snapshot.rs
  - crates/speclink-cli/tests/task_done_stamps.rs
  - crates/speclink-core/assets/skills/archive.md
  - crates/speclink-core/src/analyzer.rs
  - crates/speclink-core/src/archive.rs
  - crates/speclink-core/src/command/mod.rs
  - crates/speclink-core/src/config.rs
  - crates/speclink-core/src/demo.rs
  - crates/speclink-core/src/discuss.rs
  - crates/speclink-core/src/drift.rs
  - crates/speclink-core/src/init.rs
  - crates/speclink-core/src/instructions.rs
  - crates/speclink-core/src/lib.rs
  - crates/speclink-core/src/listing.rs
  - crates/speclink-core/src/model.rs
  - crates/speclink-core/src/newcmd.rs
  - crates/speclink-core/src/preflight.rs
  - crates/speclink-core/src/schema.rs
  - crates/speclink-core/src/skills.rs
  - crates/speclink-core/src/status.rs
  - crates/speclink-core/src/tasks.rs
  - crates/speclink-core/src/validate.rs
  - crates/speclink-core/tests/golden/claude.snapshot.md
  - crates/speclink-core/tests/golden/codex.snapshot.md
  - crates/speclink-core/tests/golden/neutral-cli.snapshot.md
  - crates/speclink-core/tests/golden/neutral-tool-call.snapshot.md
  - crates/speclink-host/src/context.rs
  - docs/platform-architecture.zh-TW.md
  - packages/ui/src/__tests__/delta.test.ts
  - packages/ui/src/__tests__/taskList.test.tsx
  - packages/ui/src/components/ChangeList.tsx
  - packages/ui/src/components/DeltaBadges.tsx
  - packages/ui/src/components/RichDetailDrawer.tsx
  - packages/ui/src/delta.ts
  - packages/ui/src/index.ts
  - packages/ui/src/theme.css
-->

---
### Requirement: 先封存子流程的順序提示與收尾提醒

內嵌 speclink-commit 技能（事實來源 crates/engine/speclink-core/assets/skills/commit.md，經 init 與 update 渲染至工具技能目錄）的「先封存再一起提交」子流程 SHALL 在執行 speclink archive 之前執行 speclink plan --json：目標 change 的 `blockedBy` 非空時 SHALL 提醒「plan 建議先封存 <blockedBy 的名稱>，再封存 <name>」，此為僅建議——SHALL NOT 阻擋、使用者確認後照常封存；`blockedBy` 為空、目標不在 plan 內或 plan 失敗時 SHALL NOT 提及順序。子流程封存成功後 SHALL 敘明兩件事：工作區存在 openspec/manual/ 時提醒可跑 manual 技能檢查手冊是否過期（條件僅為目錄存在，僅提醒、SHALL NOT 代跑）；再執行 speclink plan --json，`next` 非 null 時提「plan 的下一個可開工：<next>，執行 /speclink-apply <next>」，有效 worktree 政策開啟且第 1 波兩個以上可開工時列可並行名單，`next` 為 null 或 plan 失敗時不提；皆僅提醒、SHALL NOT 代跑。

#### Scenario: 子流程封存前的順序提示

- **WHEN** 使用者於 commit 技能選擇「先封存再一起提交」，目標 change 在 plan 中的 blockedBy 為 ["add-a"]
- **THEN** 技能檔指示在執行 speclink archive 前提醒「plan 建議先封存 add-a，再封存 <name>」，使用者確認後照常封存

#### Scenario: 無阻擋不提

- **WHEN** 目標 change 的 blockedBy 為空
- **THEN** 技能檔指示不提及順序，直接進入封存

#### Scenario: 子流程封存後的收尾提醒

- **WHEN** 子流程封存成功、工作區存在 openspec/manual/、plan 的 next 為 add-b
- **THEN** 技能檔指示提醒手冊可能過期，並提「下一個可開工：add-b，執行 /speclink-apply add-b」，兩者皆僅提醒

<!-- @trace
source: add-plan-handoff-after-archive
updated: 2026-09-16T16:37:51+08:00
-->
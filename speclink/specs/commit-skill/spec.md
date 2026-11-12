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

內嵌 speclink-commit 技能（事實來源 crates/engine/speclink-core/assets/skills/commit.md，經 init 與 update 渲染至工具技能目錄）的「先封存再一起提交」子流程 SHALL 在執行 speclink archive 之前執行 speclink plan --json：目標 change 的 `archiveAfter` 非空時 SHALL 提醒「plan 建議先封存 <archiveAfter 的名稱>；它們封存後，重讀本 change 對同名 requirement 的區塊、對照正式規格重寫（走 ingest）再封存 <name>」；`requirementOverlap` 含 `conflict` 為 true 者時 SHALL 提醒兩邊都新增（或改名成）、或都移除（或改名掉）同名 requirement，要先改掉其中一邊；`blockedBy` 非空時 SHALL 另提「宣告前置 <名稱> 尚未封存」；三者皆為僅建議——SHALL NOT 阻擋、使用者確認後照常封存，SHALL NOT 寫「跑 drift 即可」；三者皆空、目標不在 plan 內或 plan 失敗時 SHALL NOT 提及順序。子流程 SHALL 同時記下 `archiveAfter` 含目標 change 的其他 change。子流程封存成功後 SHALL 敘明三件事：工作區存在 speclink/manual/ 時提醒可跑 manual 技能檢查手冊是否過期（條件僅為目錄存在，僅提醒、SHALL NOT 代跑）；封存前記下的 change 非空時提醒它們先對照正式規格重寫（走 ingest）再封存（僅提醒、SHALL NOT 單獨觸發詢問、SHALL NOT 代跑 ingest）；再執行 speclink plan --json，`next` 非 null 時提「plan 的下一個可開工：<next>，執行 /speclink-apply <next>」，有效 worktree 政策開啟且第 1 波兩個以上可開工時列可並行名單，`next` 為 null 或 plan 失敗時不提；皆僅提醒、SHALL NOT 代跑。

#### Scenario: 子流程封存前的順序提示

- **WHEN** 使用者於 commit 技能選擇「先封存再一起提交」，目標 change 在 plan 中的 archiveAfter 為 ["add-a"]
- **THEN** 技能檔指示在執行 speclink archive 前提醒「plan 建議先封存 add-a；add-a 封存後對照正式規格重寫再封存 <name>」，使用者確認後照常封存

#### Scenario: 無阻擋不提

- **WHEN** 目標 change 的 archiveAfter、blockedBy 皆為空且無 conflict
- **THEN** 技能檔指示不提及順序，直接進入封存

#### Scenario: 子流程封存後的收尾提醒

- **WHEN** 子流程封存成功、工作區存在 speclink/manual/、封存前 add-c 的 archiveAfter 含目標 change、plan 的 next 為 add-b
- **THEN** 技能檔指示提醒手冊可能過期，提醒 add-c 先對照正式規格重寫再封存，並提「下一個可開工：add-b，執行 /speclink-apply add-b」，三者皆僅提醒


<!-- @trace
source: spec-dir-path-rewrite
updated: 2026-10-25T21:44:20+08:00
-->

---
### Requirement: commit 的檔案清單取自 evidence 動詞

commit 技能 SHALL 以 speclink evidence show <change> --json 取得這個變更改到的程式檔（`files` 與逐任務的 `entries`），SHALL NOT 指示直接讀取 evidence 檔。任務進度 SHALL 取自 speclink list --json；先封存子流程中 delta 的檢查與改寫 SHALL 與 archive 技能用相同的動詞做法。規格資料夾底下的變動（變更的 artifact、封存造成的搬移、手冊資料夾的提醒）SHALL 只在規格資料夾位於這個 checkout 時列入檔案清單與提醒；不位於時技能 SHALL 說明這次 commit 只含程式碼的檔案、規格的變動已經在 Server 上。確認閘門顯示的檔案清單 SHALL 等於實際會 stage 的檔案，兩種情況都成立。逐檔 stage 與不使用整批 stage 的既有規則不變。

#### Scenario: 檔案清單以動詞取得

- **WHEN** 檢視產生的 commit 技能中收集這個變更的程式檔的步驟
- **THEN** 該步驟以 speclink evidence show 表述，全文不在條件區塊以外提到 evidence 的檔名

#### Scenario: remote 專案的 commit 只含程式碼

- **WHEN** 以「規格資料夾不在這個 checkout」產生 commit 技能
- **THEN** 檔案清單的步驟只列 evidence 的檔案與使用者確認的其他髒檔，並說明規格的變動已經在 Server 上；不含規格資料夾的路徑

#### Scenario: 本機專案的 commit 含規格檔

- **WHEN** 以「規格資料夾在這個 checkout」產生 commit 技能
- **THEN** 檔案清單的步驟另外列入這個變更在規格資料夾底下的變動，並保留封存後的搬移一併提交的步驟

#### Scenario: 確認閘門與實際一致

- **WHEN** 檢視兩種情況產生的 commit 技能的確認閘門
- **THEN** 閘門顯示的檔案清單來源與之後 stage 的檔案來源相同

<!-- @trace
source: skills-verbs-second-wave
updated: 2026-11-12T16:54:51+08:00
-->
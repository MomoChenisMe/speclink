## Why

add-change-plan-engine 把執行順序的判定放進引擎，archive 技能在封存前會以 `speclink plan --json` 建議先封存前置。但封存有兩條路：commit 技能的「先封存再一起提交」子流程直接執行 `speclink archive`，繞過了這段提示，也漏了 archive 技能尾段的手冊過期提醒；而不論走哪條路，封存完成後都沒有人告訴使用者「下一個可開工的 change 是誰」，使用者得自己再跑一次 plan 或開看板。本變更是討論 plan-handoff-after-archive 的結論：把封存這一端接上 plan。目標使用者是透過 AI 代理跑 SDD 的開發者，對應 archive 與 commit 兩個技能的收尾階段。

## What Changes

- commit 技能的先封存子流程（7a）在執行 `speclink archive` 之前新增與 archive 技能同一段「Plan order hint」：跑 `speclink plan --json`，目標 change 的 `blockedBy` 非空就建議先封存那些前置；僅建議，不阻擋。子流程成功後補上手冊過期提醒（條件為 openspec/manual/ 目錄存在，僅提醒、不代跑）。
- archive 技能的「After the archive」尾段與 commit 子流程成功後都新增「下一個可開工」提示：跑 `speclink plan --json`，`next` 非 null 時提一句「plan 的下一個可開工：X，執行 /speclink-apply X」；有效 worktree 政策開啟且第 1 波有兩個以上可開工的 change 時，列出可並行名單（各開 session 走 apply-with-worktree）；`next` 為 null 或 plan 失敗（依賴成環）時不提。一律僅提醒、不代跑。
- archive 技能第 1 步（未指名時的選擇）改以 `speclink plan --json` 列候選：依配置順序、每個候選標出 `blockedBy`；仍由使用者選、不自動選。commit、review、verify、drift、analyze 的候選清單維持 `speclink list --json`。
- ASSET_VERSION 自 v1.36.0 升為 v1.37.0，claude／claude-worktree／codex／neutral-cli／neutral-tool-call 五份 golden 與 assets.lock 同批更新，`speclink update` 再生 SKILL.md。影響 claude 與 codex 兩個工具的 speclink-archive 與 speclink-commit 技能。
- 相容性影響：不動任何 CLI 指令與引擎行為；只有兩份技能資產的文字與 golden 快照變更。

## Non-Goals

- 不把 commit 子流程改為呼叫整個 archive 技能（技能不跨技能委派）。
- 不對封存加軟依賴的硬守門，也不加引擎新指令（硬信號由合併閘擋）。
- 不改 review、verify、quality、worktree-merge、ingest、drift 的交棒句（下一步固定，或已由 apply 守門）。
- 不改 commit、review、verify、drift、analyze 的候選清單排序。
- 不併入 add-change-plan-desktop 或 add-change-plan-remote（兩者不碰技能資產）。

## Capabilities

### New Capabilities

（無）

### Modified Capabilities

- `archive-skill`: 「封存完成後的收尾提交提醒」補「下一個可開工」提示；新增「未指名時的候選清單依 plan 順序」需求。
- `commit-skill`: 新增「先封存子流程的順序提示與收尾提醒」需求。
- `skill-routing`: 「出口交棒由技能結尾承載」的 archive 終點敘述補「下一個可開工」提示，交棒表 archive row 同步。

## Impact

- Affected specs: `archive-skill`、`commit-skill`、`skill-routing`（修改）
- Affected code:
  - New: 無
  - Modified: crates/engine/speclink-core/assets/skills/archive.md、crates/engine/speclink-core/assets/skills/commit.md、crates/engine/speclink-core/src/workspace/init.rs（ASSET_VERSION）、crates/engine/speclink-core/tests/golden/assets.lock、crates/engine/speclink-core/tests/golden/claude.snapshot.md、crates/engine/speclink-core/tests/golden/claude-worktree.snapshot.md、crates/engine/speclink-core/tests/golden/codex.snapshot.md、crates/engine/speclink-core/tests/golden/neutral-cli.snapshot.md、crates/engine/speclink-core/tests/golden/neutral-tool-call.snapshot.md、crates/engine/speclink-core/tests/it/render_golden.rs（依規格場景新增兩個渲染內容測試）、由 speclink update 再生的 .claude/skills 與 .agents/skills 下的 SKILL.md
  - Removed: 無
- 與在途變更的關係：add-change-plan-desktop 與 add-change-plan-remote 不碰技能資產，delta 不重疊，可並行。

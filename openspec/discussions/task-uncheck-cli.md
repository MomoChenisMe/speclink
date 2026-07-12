---
topic: apply 標記 task 打勾的 CLI 只能打勾、無法取消勾選，LLM 被迫直接修改 tasks.md——確認現況並決定修法
slug: task-uncheck-cli
status: promoted
promoted_to: task-uncheck-cli
created: 2026-07-11
created_by: MomoChen <momochenisme@gmail.com>
---

# Discussion: apply 標記 task 打勾的 CLI 只能打勾、無法取消勾選，LLM 被迫直接修改 tasks.md——確認現況並決定修法

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

使用者在 apply 流程中觀察到：task 打勾走 CLI 指令，但 CLI 只能打勾、無法取消勾選，LLM 要取消時被迫直接修改 tasks.md。討論目標是驗證此觀察並決定修法。

模式：假設模式——codebase scout 找到四處直接相關原始碼（crates/speclink-core/src/tasks.rs、crates/speclink-cli/src/main.rs、apps/desktop/core/src/manage.rs、.claude/skills/speclink-apply/SKILL.md），足以形成假設。

相關 changes/specs：無既有 change 涵蓋此題。相關先例：desktop-task-interactions design D1（批次動詞單指令雙用，done=false 純行編輯無側效）。關鍵背景：專案有 remote store 路徑（crates/speclink-remote），直接改檔會繞過 Store 抽象。

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-07-11)

**Focus**: 「CLI 只能打勾不能取消」的現況驗證與修法方向
**Position**: 觀察屬實（CLI 側），修法方向經使用者全數確認：
- 現況：CLI 僅有 `task done`（crates/speclink-cli/src/main.rs:553），core 僅 `mark_done` 做 `[ ]`→`[x]` 單向翻轉（crates/speclink-core/src/tasks.rs:59）；apply 技能只教打勾（SKILL.md:185），LLM 取消勾選只能直接改 tasks.md
- 桌面已有雙向 `set_task_done_at(done: bool)`（apps/desktop/core/src/manage.rs:78），但 uncheck 分支是桌面自製 regex 純行編輯（manage.rs:97-106），未進 speclink-core——邏輯存在但長錯層
- 直接改檔繞過 Store 抽象；remote store（crates/speclink-remote）情境下 LLM 直接改本地檔會改錯地方
- 修法：speclink-core 新增反向函式 `tasks::uncomplete`，CLI 曝露 `speclink task undone <task-id>`；取消勾選為純狀態翻轉——不回收 touched 記錄、不撤 started_at 開工章（沿 desktop-task-interactions D1「done=false 無側效」先例）
- 桌面 uncheck 分支收斂 delegate 到 core 新函式，恢復 complete() 註解宣稱的「單一協作點」
- apply 技能指示同步教 undone 動詞——受內嵌技能三處同步約束（core assets、repo 技能實例、render golden；golden 須乾淨樹再生）
- 呈現層：CLI 對「已是未勾選」報錯（與 task done 對已完成的呈現對稱），桌面維持冪等成功
**Ruled out**: 維持直接改檔（remote store 下錯誤、繞過 Store）；CLI 層自做行編輯（同樣繞過 core 單一協作點）；反向側效回滾 touched/開工章（竄改 trace 歷史、複雜度暴增）；`task done --undo` 旗標（對稱動詞 `undone` 對 agent 更直覺）
**Open**: 無——五項假設經使用者全數確認

## Conclusion

**Decision**: speclink-core 新增反向完成函式 `tasks::uncomplete`，CLI 新增對稱動詞 `speclink task undone <task-id> --change <name>`。取消勾選為純狀態翻轉（`[x]`→`[ ]`）、無反向側效：不回收 touched 記錄、不撤 started_at 開工章。桌面 manage.rs 的 uncheck regex 分支收斂 delegate 到 core 新函式。apply 技能指示同步加入 undone 動詞（內嵌 assets、repo 技能實例、render golden 三處同步，golden 於乾淨樹再生）。呈現層：CLI 對「已是未勾選」報錯，桌面維持冪等成功。
**Rationale**: LLM 在 apply 中沒有取消勾選動詞，被迫直接改 tasks.md——這繞過 Store 抽象，在 remote store 情境下會改錯地方。uncheck 邏輯其實已存在於桌面端但長錯層；收斂到 core 才符合 complete() 的「單一協作點」設計，且成本最低（邏輯搬家而非新造）。
**Rejected alternatives**: 維持直接改檔（remote store 下錯誤）；CLI 層自做行編輯（繞過 core 協作點，與桌面邏輯分岔）；反向側效回滾 touched/開工章（竄改 trace 歷史、複雜度不成比例）；`task done --undo` 旗標（對稱動詞對 agent 更直覺、--help 可發現性更好）。
**Deferred**: none
**Capture to**: proposal
**Next**: /speclink-propose --from-discussion task-uncheck-cli

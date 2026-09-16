---
topic: 封存這一端接 plan：commit 先封存子流程漏掉順序提示、封存後提示下一個可開工
slug: plan-handoff-after-archive
status: promoted
created: 2026-09-16
created_by: MomoChen <momochenisme@gmail.com>
promoted_to: add-plan-handoff-after-archive
---

# Discussion: 封存這一端接 plan：commit 先封存子流程漏掉順序提示、封存後提示下一個可開工

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

使用者在 change-execution-order 三刀立案後追問：commit 技能的「先封存再一起提交」路徑是否也會提醒，若會是否該跟隨新的 plan 邏輯；並要求掃其他流程有無同樣情境。需求可驗證，未經 grill 直接進假設。
掃描結果：commit 技能 7a 子流程直接執行 speclink archive，沒有 archive 技能 3b 的 Plan order hint，也沒有尾段的手冊過期提醒；archive 技能尾段只提醒提交與手冊，封存完成後沒有「下一個可開工」；review／verify／quality／worktree-merge 的下一步固定是封存、ingest／drift 導向 apply（apply 第一步已跑 plan 守門）、discuss promote 立骨架後仍由 propose 建 artifacts 並盤點——皆不需動。選 change 模糊時各動詞用 list --json 列候選，只有 archive 的候選順序影響合併閘。
相關規格：archive-skill、commit-skill、skill-routing、change-plan。相關變更：add-change-plan-engine（已封存，da75931c）、add-change-plan-desktop、add-change-plan-remote（提案中）。
Prior discussions: propose-apply-handoff-updates, change-execution-order, hold-countdown-auto-close

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-09-16)

**Focus**: 封存這一端要不要接 plan——commit 子流程的缺口、封存後的下一步提示、其他流程盤點
**Position**: 五條假設全數成立：補 commit 子流程的順序提示、封存後提示下一個可開工、archive 候選照 plan 排、其他交棒不動、落點為獨立小 change。
- A 事實缺口：commit.md 7a 直接跑 speclink archive，繞過 archive.md 3b 的 Plan order hint，也漏尾段的手冊過期提醒；補回同一段提示與手冊提醒
- B 封存完成後（archive 尾段與 commit 子流程成功後）跑 speclink plan --json：next 非 null 提一句「plan 的下一個可開工：X，執行 /speclink-apply X」；worktree 政策開且第 1 波兩個以上可開工時列可並行名單；next 為 null 或 plan 失敗不提
- C archive 技能第 1 步選 change 模糊時改以 plan --json 列候選（plan 順序＋標出 blockedBy）；commit／review／verify／drift／analyze 維持 list --json，順序對它們無意義
- D 其他交棒不動：review／verify／quality／worktree-merge 下一步固定封存；ingest／drift 導向 apply 而 apply 已守門；discuss promote 的盤點在 propose 收尾
- E 落點：新的小 change 只動 archive.md 與 commit.md、ASSET_VERSION 三連動；規格 delta 落 archive-skill、commit-skill、skill-routing 交棒表，不動 change-plan 規格，與刀二／刀三無 delta 重疊、可並行
- 先前討論 propose-apply-handoff-updates 否決「每次 propose 都展開盤點」：本案的提示只在 next 非 null 時一行，不構成翻案
**Ruled out**: 併入刀二或刀三（它們不碰技能資產，且會讓 remote 刀多背一次 ASSET_VERSION 三連動）；所有選 change 的動詞都改用 plan 順序（review／verify 等不在乎順序，徒增噪音）；封存後硬擋亂序（原討論已定封存不加軟依賴硬守門，合併閘已擋硬信號）
**Open**: 無

## Conclusion

**Decision**: 封存這一端接上 plan，一個獨立小 change 落地，只動技能資產。（1）commit 技能的「先封存再一起提交」子流程（7a）在執行 speclink archive 之前補回與 archive 技能 3b 同一段 Plan order hint（跑 speclink plan --json，目標 blockedBy 非空即建議先封存前置，僅建議不阻擋），子流程成功後補上手冊過期提醒（條件為 openspec/manual/ 存在，僅提醒）。（2）封存完成後——archive 技能尾段與 commit 子流程成功後——跑 speclink plan --json：next 非 null 時提一句「plan 的下一個可開工：X，執行 /speclink-apply X」；有效 worktree 政策開啟且第 1 波有兩個以上可開工者時列出可並行名單（各開 session 走 apply-with-worktree）；next 為 null 或 plan 失敗（成環）時不提；一律僅提醒、不代跑。（3）archive 技能第 1 步選 change 模糊時改以 speclink plan --json 列候選：依配置順序、每個候選標出 blockedBy；commit／review／verify／drift／analyze 的候選清單維持 list --json。（4）其他交棒不動：review／verify／quality／worktree-merge 下一步固定封存，ingest／drift 導向 apply（apply 第一步已跑 plan 守門），discuss promote 的盤點留在 propose 收尾。（5）canon deltas：archive-skill 規格 MODIFIED「封存完成後的收尾提交提醒」補「下一個可開工」提示與候選清單改 plan；commit-skill 規格 ADDED「先封存子流程的順序提示與收尾提醒」；skill-routing 交棒表 archive row 與 commit row 補字面。asset 異動走 ASSET_VERSION／golden／assets.lock 三連動，speclink update 再生 SKILL.md。
**Rationale**: 刀一把順序判定放進引擎，但封存有兩條路——archive 技能與 commit 的先封存子流程——後者直接呼叫引擎動詞，繞過了技能層的提示；不補就等於建議只對一半路徑生效。封存完成是使用者下一個決策點（做誰），propose 那一端已有盤點，封存這一端補一行 next 就把「重看一次」消掉。獨立 change 因為刀一已封存、刀二刀三不碰技能資產，且與兩刀無 delta 重疊、可並行。
**Rejected alternatives**: 併入刀二或刀三（不碰技能資產，且 remote 刀多背一次 ASSET_VERSION 三連動、落地時間拖到 remote 之後）；所有選 change 的動詞都改用 plan 順序（review／verify／drift／analyze 不在乎順序，徒增噪音）；封存後硬擋亂序或引擎新守門（原討論已定封存不加軟依賴硬守門，硬信號由合併閘擋）；commit 子流程改為呼叫整個 archive 技能（技能不能跨技能委派，已有既定紀律）。
**Deferred**: 無。
**Capture to**: proposal（新 change：add-plan-handoff-after-archive）；archive-skill、commit-skill、skill-routing 三份規格 delta。
**Next**: /speclink-propose --from-discussion plan-handoff-after-archive

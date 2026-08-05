---
topic: desktop worktree 開關與技能注入模式，及多 change 與先 apply 的防護
slug: worktree-toggle-and-guards
status: promoted
promoted_to: worktree-toggle-and-guards
created: 2026-08-04
created_by: MomoChen <momochenisme@gmail.com>
---

# Discussion: desktop worktree 開關與技能注入模式，及多 change 與先 apply 的防護

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

起因：worktree 第一刀（引擎聚合讀＋config 欄位＋兩技能）已封存，前一輪 ask-shaped 問答確認 desktop 端 UI 屬第二刀且尚未排程。使用者裁示：①開始規劃 desktop 的 worktree 開關；②期望模式為「開啟→引擎組裝並自動注入 worktree 技能到專案；關閉→移除」；③要求評估「一次給多個 change」與「先 apply 再轉 worktree」兩個防護。

模式：assumptions——掃描找到充足脈絡（crates/speclink-core/src/skills.rs 的 registry、crates/speclink-core/src/init.rs 的生成與 prune_footprint、apps/desktop/core/src/settings.rs 的 carry_over_worktree、crates/speclink-core/tests/it/render_golden.rs 的 golden 釘住、crates/speclink-core/src/workspace.rs:82 的 .evidence.json 落點）。

相關：已封存討論 worktree-parallel-apply（第二刀範疇原定含 desktop toggle＋卡片標示＋抽屜分支資訊）；specs：worktree-apply-skill、worktree-merge-skill、worktree-overlay、workflow-config。

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-08-04)

**Focus**: 第二刀範疇定調與技能注入機制的現況盤點
**Position**: 開關落產出政策區、注入改為政策條件式、兩防護落技能 preflight——五項假設攤給使用者校正：
- 現況：兩顆 worktree 技能是「無條件注入＋執行期 P1 拒跑」（crates/speclink-core/src/init.rs:516 的 registry 迴圈不看政策）；移除機制現成（init.rs:554 prune_footprint）
- 假設一：desktop 開關進產出政策區比照 tdd／audit，存檔走既有 settings seam，settings.rs:182 的 carry_over_worktree 退役
- 假設二：注入跟 config 檔值走（env SPECLINK_WORKTREE 不影響注入，僅為執行期逃生口）；P1 執行期檢查保留為第二道防線；代價＝切開關會寫／刪三處工具目錄的技能檔、git 樹變髒；golden 測試多一個政策維度
- 假設三：關閉時若有活躍 worktree（host worktree.rs discover 可查）→ 擋下並列清單，防止 worktree-merge 技能被移除後無收尾工具
- 假設四：多 change 防護＝技能輸入層一次只收一個，多給→AskUserQuestion 挑一個並印多 session 配方；不做靜默依序批次
- 假設五：先 apply 再 worktree 防護＝新增 P3.5，讀 .evidence.json（workspace.rs:82，受版控隨 P3 commit 進 worktree）touched 清單對主樹 git status，髒→停下推薦先走 /speclink-commit
**Open**: 五項假設待校正；第二刀原定的卡片 worktree 標示與抽屜分支資訊是否併入本次範疇；落地拆幾個 change（建議拆「開關＋注入」與「技能防護」兩刀）

### Round 2 — assumptions (2026-08-04)

**Focus**: 範疇與拆法的裁決
**Position**: 一刀全包——五項假設成立，範疇併入 desktop 呈現：
- 使用者裁定：卡片 worktree 標示＋抽屜分支資訊併入本次
- 使用者裁定：單一 change 完成，不拆
- 五項假設未被挑戰，視為成立：開關落產出政策區、注入跟 config 檔值（env 不影響注入、P1 保留）、關閉遇活躍 worktree 擋下、多 change 輸入拒收、P3.5 分家偵測
- 補盤：desktop 端 worktree 僅存在於 settings.rs 的 carry_over 保值邏輯；看板資料流與 watcher 尚未接 worktree facts（apps/desktop 無其他 worktree 引用；listing.rs 的 worktree 欄位僅 CLI list 組裝在用）——卡片標示含資料管線，非純 UI
**Ruled out**: 拆二～三個 change 落地——使用者裁定一刀全包
**Open**: 無（worktree 存在時 desktop 動詞的防護細節留 design 階段，承原討論的 Deferred）

## Conclusion

**Decision**: worktree 第二刀以單一 change 落地，含四塊：
- 開關：desktop 產出政策區新增 worktree toggle，比照 tdd／audit 走既有 settings seam；settings.rs 的 carry_over_worktree 保值邏輯退役
- 條件式注入：技能生成期讀 openspec/config.yaml 的 worktree 值——true 注入兩顆 worktree 技能、false prune（沿用 init.rs 既有 prune 機制）；SPECLINK_WORKTREE env 不影響注入（僅執行期逃生口）；技能內 P1 執行期檢查保留為第二道防線；關閉開關時若有活躍 worktree（host discover 可查）→ 擋下並列出清單，防止 merge 技能被移除後無收尾工具；golden 測試新增政策開／關維度
- 技能防護：①多 change 輸入拒收——偵測到多個 change 名以 AskUserQuestion 挑一個並印多 session 配方，不做靜默依序批次；②P3.5「進度與程式碼分家」偵測——讀 openspec/changes/<change>/.evidence.json 的 touched 清單對主樹 git status，髒→停下，推薦選項依序為「先走 /speclink-commit」「照樣繼續」「停止」
- desktop 呈現：看板資料流接 worktree facts（listing 的 worktree 欄位已備，desktop 端管線未接）、卡片 worktree 標示、抽屜分支與路徑資訊、watcher 擴充（監看各 worktree 的 openspec/changes/<change>/ 與 .git/worktrees/ 增減）
**Rationale**: 「開關可見即可用」——技能存在與否與開關綁定，使用者從 GUI 一鍵決定專案是否具備 worktree 流程，代價（切開關寫／刪三處工具目錄技能檔、git 樹變髒）經使用者點頭接受；防護的核心張力是「進度記錄與程式碼實體分家」，以受版控的 .evidence.json 為偵測依據，零新增儲存
**Rejected alternatives**: 維持無條件注入＋僅執行期擋（關閉時技能清單仍見無用動詞）；注入跟隨 env 有效值（env 是單次逃生口，非專案持久狀態）；多 change 靜默依序批次（單 session 上下文撐爆、模糊「平行＝多 session」模型）；關閉遇活躍 worktree 警告放行（使用者可能鎖進「有 worktree 沒工具」的狀態）；拆二～三個 change 落地（使用者裁定一刀全包）
**Deferred**: worktree 存在時 desktop 動詞的防護（如對 worktree 中的 change 執行 archive／discard）——design 階段定（承原討論 worktree-parallel-apply）；desktop 卡片上的 merge 按鈕——後續視需求（承原討論）
**Capture to**: proposal（promote 本討論為單一 change）
**Next**: /speclink-propose --from-discussion worktree-toggle-and-guards

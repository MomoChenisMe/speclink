---
topic: speclink-commit 確認前未顯示 commit 計畫與訊息——診斷與修法
slug: commit-plan-visibility
status: promoted
promoted_to: commit-plan-visibility
created: 2026-07-12
created_by: MomoChen <momochenisme@gmail.com>
---

# Discussion: speclink-commit 確認前未顯示 commit 計畫與訊息——診斷與修法

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

使用者截圖回報：verify → archive → commit 連跑時，speclink-commit 在沒有顯示任何 commit 計畫或訊息的情況下，直接以 AskUserQuestion 問「依上述計畫 commit？」。本討論診斷成因並決定修法。模式：assumptions（偵查找到 .claude/skills、.agents/skills 兩份 render 產物與事實來源 crates/speclink-core/assets/skills/commit.md，脈絡充足）。相關規則：CLAUDE.md 內嵌技能三處同步＋乾淨樹重生 golden（crates/speclink-core/tests/golden/ 四份 snapshot）。

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-07-12)

**Focus**: 為什麼 speclink-commit 在未顯示任何計畫或 commit 訊息時就問「依上述計畫 commit？」
**Position**: 兩層問題疊加——模型跳過技能 Step 5 的可見輸出，且技能本身把 commit 訊息生成排在確認之後：
- 直接原因：執行模型把 commit 計畫留在內部推理、未輸出可見文字即呼叫 AskUserQuestion；誘因是 verify → archive → commit 連跑，archive 剛輸出的檔案異動被模型當成「上述」已展示內容。
- 結構缺陷：SKILL.md 把「Generate commit message」排在 Step 7（Step 6 確認之後），「Show the message and allow editing」無確認把關、實務上被輾過——照章執行使用者也看不到 commit 訊息。
- 修正點：crates/speclink-core/assets/skills/commit.md（單一事實來源）；.claude/skills 與 .agents/skills 為 render 產物（僅差 / 與 $ 前綴），改後須乾淨樹上 UPDATE_GOLDEN=1 重生 golden。
- 修法：重排為「收集檔案 → 生成 commit 訊息 → 一次以可見文字輸出計畫＋訊息 → 單一 AskUserQuestion 確認」，並加 guardrail：提問前計畫與訊息必須已作為可見訊息輸出，問題文字不得指涉對話中不存在的「上述」。
- 純技能散文變更，不觸及引擎程式碼，interface depth check 不適用；因動到內嵌 assets 需 golden 再生，走一個小 change。
**Ruled out**: 兩段確認（先確認檔案、再確認訊息）——互動成本較高，使用者選擇單一確認閘門；只修 repo 技能實例不動 assets——下次 render 會覆蓋，非事實來源。
**Open**: 無——四項假設全數獲使用者確認（「OK可以」）。

## Conclusion

**Decision**: 修改 speclink-commit 內嵌技能（crates/speclink-core/assets/skills/commit.md）：重排步驟為「收集檔案 → 生成 commit 訊息 → 以可見文字一次輸出計畫＋訊息 → 單一 AskUserQuestion 確認 → 暫存並提交」，並新增 guardrail：呼叫 AskUserQuestion 前，commit 計畫與訊息必須已作為可見訊息輸出；問題文字不得指涉未曾輸出的「上述計畫」。render 後同步 .claude/skills 與 .agents/skills，乾淨樹上重生 golden。
**Rationale**: 截圖事故是兩層疊加——散文式「Display」指示無防呆，模型把計畫留在內部推理就提問；且訊息生成排在確認之後，使 commit 訊息永遠不在確認範圍內。單一確認閘門同時涵蓋檔案清單與訊息，把關落在保證可見的 AskUserQuestion 上，消除 Step 7 無把關的「allow editing」。
**Rejected alternatives**: 兩段確認（先檔案、後訊息）——互動成本較高、無對應收益；只修 repo 技能實例——非事實來源，下次 render 即被覆蓋。
**Deferred**: none
**Capture to**: proposal
**Next**: /speclink-propose --from-discussion commit-plan-visibility

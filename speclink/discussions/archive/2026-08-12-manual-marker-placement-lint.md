---
topic: 為什麼 [M] 標記在 desktop 有時解析有時不行——validate/analyze 與 propose skill 該不該把關
slug: manual-marker-placement-lint
status: promoted
promoted_to: manual-marker-placement-lint
created: 2026-08-12
created_by: MomoChen <momochenisme@gmail.com>
---

# Discussion: 為什麼 [M] 標記在 desktop 有時解析有時不行——validate/analyze 與 propose skill 該不該把關

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

使用者發現手動測試標記 [M] 在 desktop 有時被解析為徽章、有時以原文殘留，另一 AI 的備忘指出「[M] 必須緊貼 checkbox，寫在編號後引擎不認、codeRemaining 卡住不降，validate/analyze 都抓不到」。使用者問：是不同 AI 模型產出格式不同所致嗎？validate/analyze 是否該驗？propose skill 是否該明文規定正規寫法？

模式：assumptions（掃到 crates/speclink-core/src/tasks.rs、validate.rs、analyzer.rs、packages/ui/src/tasks.ts、assets/skills/propose.md 等相關原始碼，足以形成立場）。

相關變更／規格：manual-task-marker spec（解析 Example 表凍結行為）；已封存 manual-task-marker-gates、task-marker-ui-and-parallel-removal（兩套解析器的由來）；活案例 desktop-loading-skeleton-ux（tasks.md:29 即誤寫行）。

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-08-12)

**Focus**: [M] 時靈時不靈的根因，以及引擎側修法方向
**Position**: 完全由行內位置決定、非隨機——解析器只認 checkbox 後緊貼的前綴槽；修法選 validate 加 lint 而非放寬解析器。
- strip_markers（crates/speclink-core/src/tasks.rs:101）只剝行首的 `[M] `／`[P] `；「編號在前」（`- [ ] 6.2 [M] …`）不認，checkbox 後雙空格也不認（tasks.rs:1126 單元測試釘死）
- desktop 與 CLI 同一解析器（apps/desktop/core/src/query.rs:60 呼叫 speclink_core::tasks），非 desktop 特有問題
- repo 兩型並存：atomic-file-writes 寫對、desktop-archived-parity 寫錯；活案例＝desktop-loading-skeleton-ux tasks.md:29（未勾、正卡 codeRemaining）
- 根因非「模型不同」：propose skill 已有正確範例仍失守，「編號在前」是任何起草者的自然順序，而 validate/analyze 對此零檢查、錯誤全靜默
- 提出四假設：(1) lint 優於放寬解析器 (2) 判斷式抓「編號+[M]」與「行首殘留 [M]」兩錯型、出 error 不出 warning (3) 落點僅 validate_change（analyzer 三鏡頭是內容品質非格式守門）(4) 順手修正 skeleton-ux 6.2 誤寫行
**Ruled out**: 放寬解析器（接受編號後的 [M]）——解析行為被 manual-task-marker spec 的 Example 表凍結，且 packages/ui/src/tasks.ts 有第二套同步解析器，波及 spec＋雙解析器＋golden；描述含字面 [M] 時寬容解析會誤判並靜默改變守門行為
**Open**: 使用者對四假設的裁定；propose skill 是否也要明文規定正規寫法（使用者主動提出，下一輪處理）

### Round 2 — assumptions (2026-08-12)

**Focus**: propose skill 是否要明文規定 [M] 的正規寫法
**Position**: 要，但定位是第二層防線——lint 仍是主承重，文字規定只降低發生率、不負責攔截。
- 現行 asset（crates/speclink-core/assets/skills/propose.md:266）已有正確範例 `- [ ] [M] 3.2 …` 仍失守：desktop-loading-skeleton-ux 在該文字存在之後起草照樣寫錯，證明「只給正例」對起草 agent 不夠
- 強化形式＝對比對：正例與誤例（`- [ ] 3.2 [M] …` ❌）並列，附一句後果（引擎不認、codeRemaining 卡住）與「checkbox 後恰一個空格」規則
- 波及面輕但有慣例：asset 內文變動走 MARKER_VERSION／golden／assets.lock 三連動
- 附帶發現：ingest 也會補任務行，但其 asset 對 [M] 零規範——同批補一句，凡起草任務行處都指向同一條正規寫法
**Open**: 使用者對第一輪四假設的裁定；skill 文字補強是否與 lint 併入同一變更

## Conclusion

**Decision**: 兩關防護＋現場修正，解析器本體不動——
- validate 關：`validate_change` 加 error 級 lint，抓「編號+[M]」（`- [ ] 6.2 [M] …`）與「行首殘留 [M]」（雙空格漏接）兩錯型
- 技能關：propose 與 ingest 的 asset 補對比對規則——正誤例並列、附後果（引擎不認、codeRemaining 卡住）與「checkbox 後恰一個空格」；asset 變動走 MARKER_VERSION／golden／assets.lock 三連動
- 現場：修正 desktop-loading-skeleton-ux tasks.md 6.2 誤寫行（[M] 移到編號前）
**Rationale**: 正確範例早已存在仍失守，證明文字規定只能降發生率；validate 的 error 才是保證攔截的那層。嚴格但會報錯，優於寬容但靜默改變守門行為。
**Rejected alternatives**: 放寬解析器（解析行為被 manual-task-marker spec 的 Example 表凍結，波及 spec＋core/ui 雙解析器＋golden；描述含字面 [M] 會誤判且靜默）；lint 出 warning（會被忽略，重演「validate 全綠但 gate 卡死」）；analyze 重複報（三鏡頭是內容品質檢查，格式守門歸 validate 單一落點）。
**Deferred**: none
**Capture to**: proposal
**Next**: speclink discuss promote manual-marker-placement-lint，續以 /speclink-propose 產出其餘 artifacts

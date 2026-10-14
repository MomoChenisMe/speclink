---
topic: 任務文件的手動任務提示框：勾選完的手動任務要從提示框剔除
slug: manual-callout-unchecked-only
status: promoted
created: 2026-10-14
created_by: MomoChen <momochenisme@gmail.com>
promoted_to: manual-callout-unchecked-only
---

# Discussion: 任務文件的手動任務提示框：勾選完的手動任務要從提示框剔除

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

使用者在 add-ask-user-tool 的任務文件看到：38 條任務全部勾完（含 4 條 [M]），閱讀卡的「4 個手動任務」提示框仍列出全部四條，問能否偵測勾選、把勾完的項目剔除。需求已明確（勾完就不再列），不需要 grill 階段，直接以假設清單推進。
相關規格：desktop-app（「任務文件以群組卡呈現」段落第 1542 行的手動任務提示框、「任務列的手動任務徽章」、「看板卡片的待手動標示」、「介面狀態語意色分層」）；程式碼 packages/ui/src/components/changeDocSummary.tsx 的 tasksSummary；進行中變更 server-web-list-pages 只改 server-admin、server-web-console，與 desktop-app 無交集。
Prior discussions: discussion-stage-and-unpromoted-cuts

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-10-14)

**Focus**: 手動任務提示框該不該隨勾選狀態剔除項目，以及數字磚與規格怎麼跟著動
**Position**: 使用者確認五條假設全部成立：提示框只列未勾的 [M]，全部勾完即消失，「手動」數字磚維持總數，以一個小變更改 desktop-app 規格。
- 技術上不需要額外偵測：changeDocSummary.tsx:166 的 `manual = tasks.filter((x) => x.manual)` 不看 `done`，同函式上一行已用 `x.done` 算已完成；勾選寫回後頁面重讀 tasks.md，加一個 `!x.done` 條件即可
- 提示框標題的 N 與列表只算未勾的 [M]；與任務列淡底「未勾選時才有」、看板「待手動」章「只算未勾 [M]」同一語意
- 全部 [M] 勾完時提示框缺席；比照「N 刀尚未轉出」提示框在沒有未轉出刀時缺席（discussion-stage-and-unpromoted-cuts）；已封存檢視通常全勾，也自然不出提示框
- 「手動」數字磚維持帶 [M] 的任務總數：數字磚是清點，「任務」也是總數、完成數另由「已完成」表示
- 與正典衝突：desktop-app/spec.md:1542「有手動任務時 SHALL 有提示框……沒有手動任務時提示框缺席」的「手動任務」含已勾者，要改規格；desktop-app 沒有在途 delta
- 範圍：程式一個條件、規格一句、changeDetailPage.test.tsx:2052 的「全部勾選後」段落補提示框缺席的斷言
**Ruled out**: 全勾完改顯示「手動任務都完成了」的提示框——不給新資訊，只多一個框；只改程式不改規格——規格與行為脫節，審查與驗證關卡會抓到；「手動」數字磚改成剩餘數——破壞數字磚「清點總數」的一致規則

## Conclusion

**Decision**: 任務文件閱讀卡的手動任務提示框只列未勾選的 [M] 任務，全部勾完即缺席；以一個小變更改 desktop-app 規格與 packages/ui 的 tasksSummary。
- 提示框標題「N 個手動任務」的 N 與列表只計未勾選的 [M] 任務；每列的序號、標題與「前往」鈕照舊
- 所有 [M] 任務都已勾選時提示框缺席；沒有任何 [M] 任務時同樣缺席（現行行為不變）
- 「手動」數字磚維持為帶 [M] 的任務總數，不隨勾選變動
- 勾選狀態的來源就是已解析的 tasks.md：packages/ui/src/components/changeDocSummary.tsx 的 tasksSummary 在 `manual` 篩選加上未勾選條件；勾選寫回後頁面重讀 tasks.md，提示框跟著更新，不另做偵測機制
- 規格：desktop-app 第 1542 行段落改寫為「有未勾選的手動任務時 SHALL 有提示框……所有手動任務都已勾選或沒有手動任務時提示框缺席」，並補一個情境
- 測試：changeDetailPage.test.tsx 既有「全部勾選後」段落補提示框缺席的斷言，另補「部分勾選」的列數斷言
- 例：add-ask-user-tool 有 4 條 [M]（4.3、5.8、6.9、7.6）。只勾 4.3 → 提示框標題「3 個手動任務」，列出 5.8、6.9、7.6；四條全勾 → 提示框缺席；兩種情況「手動」數字磚都是 4
**Rationale**: 提示框的用途是提醒「還有事要你親自做」，已勾的項目留在框裡只會讓做完的人以為還沒完；改成只算未勾的項目，就與任務列淡底、看板「待手動」章同一個語意，而數字磚維持清點總數，與「任務／已完成」的分工一致。
**Rejected alternatives**:
- 全部勾完改顯示「手動任務都完成了」的提示框 — 不提供新資訊，只多一個框
- 只改程式、不改規格 — 規格與行為脫節，審查與驗證關卡會抓到
- 「手動」數字磚改為剩餘數 — 破壞數字磚清點總數的一致規則
**Deferred**: none
**Capture to**: spec | tasks
**Next**: /speclink-propose --from-discussion manual-callout-unchecked-only

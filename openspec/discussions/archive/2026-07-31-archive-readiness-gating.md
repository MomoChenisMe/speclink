---
topic: 封存階段守門一致性:看板卡鈕/抽屜/拖曳與 CLI 的封存邏輯規劃
slug: archive-readiness-gating
status: promoted
promoted_to: archive-readiness-gating
created: 2026-07-31
created_by: MomoChen <momochenisme@gmail.com>
---

# Discussion: 封存階段守門一致性:看板卡鈕/抽屜/拖曳與 CLI 的封存邏輯規劃

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

revert-in-progress-to-proposed(17/17)落地後,拖曳已收斂為純排序+封存落點。使用者原以為封存缺按鈕、問「要不要補按鈕並保留拖曳封存」;codebase 掃描發現封存鈕已存在(ready 卡+抽屜),真正的不一致是**階段守門不對稱**:卡鈕限已就緒、抽屜與拖曳全階段開放;引擎端批次 archive 有任務完成度守門(commands.rs:861)、單筆 run_archive 沒有。討論轉為:如何規劃封存的守門邏輯(UIUX+CLI)。

模式:assumptions(相關檔案充足:ChangeCard.tsx、KanbanBoard.tsx、boardDnd.ts、RichDetailDrawer.tsx、archive.rs、command/mod.rs、commands.rs)。

相關 change:revert-in-progress-to-proposed(其確立的「UI 依派生階段決定可見性、引擎是唯一裁決點」模式是本題的先例)。相關 specs:desktop-app「拖曳封存落點以浮層呈現」、board-card-order「跨欄拖曳不改變變更階段」。

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-07-31)

**Focus**: 封存鈕是否缺席?拖曳封存要不要保留?
**Position**: 封存鈕已存在,真正的題目是階段守門不對稱:
- 卡片封存鈕僅已就緒卡顯示(ChangeCard.tsx:138);抽屜封存鈕全階段顯示(RichDetailDrawer.tsx:371);拖曳落點對所有階段的變更卡浮現(boardDnd.ts:29 只查卡片種類)
- 三條路皆走同一確認對話框(App.tsx:866)→ 引擎 archive 動詞
- 引擎自身也不一致:批次 archive 守任務完成度(commands.rs:861,未完成即 skip、--mark-tasks-complete 可先全勾再封),單筆 run_archive(desktop 同路徑)不檢查
- 使用者方向:非已就緒不應給封存 → 討論聚焦「守門邏輯如何規劃(UIUX+CLI)」
**Ruled out**: 「新增封存鈕」——按鈕已存在,題目不成立;「移除拖曳封存」——使用者傾向保留,且 spec 已兩處 pin 此行為
**Open**: 守門放引擎還是 UI?抽屜封存鈕非 ready 時 disabled 還是隱藏?未完成 change 的放棄出路(delete vs 留紀錄封存)?單筆與批次的其他 readiness 檢查(stale delta assumptions)是否也收斂?

### Round 2 — assumptions (2026-07-31)

**Focus**: 封存守門邏輯如何分層(引擎 vs UI 三表面)?
**Position**: 沿用 revert 確立的「UI 依派生階段決定可見性、引擎是唯一裁決點」模式:
- 引擎:單筆 run_archive 補 fail-closed 守門(total>0 且 complete<total → 拒絕,列證據 N/M 與出路),與批次(commands.rs:861)收斂同一條件;豁免沿用既有 --mark-tasks-complete(先全勾再封,語意誠實),不發明新旗標
- UI 卡鈕:維持僅 ready,零改動
- UI 拖曳落點:archiveZoneVisible 加階段條件,僅拖 ready 卡時浮現(拖非 ready 卡=純排序,落點不出現)
- UI 抽屜鈕:非 ready 時 disabled + tooltip 原因(使用者裁定;沿用 UnavailableAction 既有模式)
- 併發保底:引擎拒絕走既有 store.archiveFailed toast——可見性過濾≠預判守門
**Ruled out**: 抽屜鈕非 ready 時直接隱藏——失去「為什麼不能封存」的可發現性,使用者裁定 disabled+原因;單筆守門一併收斂 stale delta assumptions(drift)檢查——範圍膨脹,drift 留給 verify skill 流程把關
**Open**: 刪除鈕是否也要鏡像守門?提案中抽屜的封存鈕如何呈現?

### Round 3 — assumptions (2026-07-31)

**Focus**: 刪除是否比照守門?提案中抽屜的封存鈕呢?
**Position**: 刪除與封存是鏡像對稱的一對,且刪除的引擎守門早已存在——真正的洞在 desktop 繞過它:
- 引擎 discard 動詞(discard.rs)已守「開工痕跡」:started_at 或任何已勾任務 → typed Refusal,--force 才放行;還負責來源討論 unlink(promoted_to 回復)與 touched 記錄清理
- desktop 的 delete_change(manage.rs:57-68)直接 remove_dir_all,繞過守門、unlink、touched 清理——對已轉出討論的 change 刪除會留下懸空的 promoted_to
- 修法:desktop 刪除改接引擎 discard 動詞(force=false);UI 抽屜刪除鈕僅 proposed 可按,非 proposed 時 disabled + 原因(與封存鈕同模式)
- 對稱結構:封存=僅 ready 給、引擎守任務全完成、CLI 豁免 --mark-tasks-complete;刪除=僅 proposed 給、引擎守零開工痕跡、CLI 豁免 --force;desktop 不提供任何 force 通道
- 提案中抽屜的封存鈕:落入同一規則——非 ready → disabled + 原因(任務未完成 0/N)
**Ruled out**: desktop 提供 force 刪除通道——毀壞已開工的工作紀錄應是 CLI 明示操作,與 revert 不給機械強制出路的先例一致
**Open**: 0 任務 change 的邊界(派生 proposed、但引擎 total>0 條件放行封存)是否需要處理?

## Conclusion

**Decision**: 封存與刪除採鏡像階段守門——封存僅已就緒(引擎單筆 run_archive 補任務完成度守門,與批次 commands.rs:861 同條件,豁免沿用 --mark-tasks-complete)、刪除僅提案中(desktop 刪除改接既有 discard 動詞 force=false,--force 豁免僅限 CLI);UI 三表面收斂:卡鈕不變(封存僅 ready、退回僅 in-progress)、拖曳封存落點僅拖 ready 卡時浮現(archiveZoneVisible 加階段條件)、抽屜封存/刪除鈕於非法階段 disabled + tooltip 原因(沿用 UnavailableAction 模式);併發保底走引擎拒絕 + 既有 toast。
**Rationale**: 引擎批次封存與 discard 動詞早已內建這兩道守門,現狀是單筆封存與 desktop 刪除(manage.rs 直接 remove_dir_all,連討論 unlink 與 touched 清理都繞過)繞道而行——本案是補齊繞道使引擎自身一致,不是發明新規則;UI 分工沿用 revert-in-progress-to-proposed 確立的「派生階段管可見性、引擎唯一裁決」先例。生命週期語意收斂:提案中=可反悔(刪除)、進行中=只能往前或退回、已就緒=可收檔(封存)。
**Rejected alternatives**: 移除拖曳封存(spec 兩處已 pin、失去最快手勢);抽屜鈕非法階段直接隱藏(失去「為什麼不能」的可發現性);desktop 提供 force 刪除通道(毀壞已開工紀錄應為 CLI 明示操作,與 revert 不給機械強制出路同哲學);單筆封存一併收斂 stale delta assumptions(drift)檢查(範圍膨脹,留給 verify skill 流程);新增豁免旗標(--mark-tasks-complete 已存在且語意誠實)。
**Deferred**: 0 任務 change 邊界(派生提案中、但引擎 total>0 條件放行封存,CLI 與批次現狀一致,desktop 以 stage 收斂即可);「留紀錄不套 specs 的放棄型封存」是否需要獨立動詞。
**Capture to**: proposal
**Next**: /speclink-propose --from-discussion archive-readiness-gating

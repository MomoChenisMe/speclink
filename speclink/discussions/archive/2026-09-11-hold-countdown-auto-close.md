---
topic: 帶 hold 的討論在所有轉出變更封存後應自動連帶封存，不靠人自己判斷
slug: hold-countdown-auto-close
status: promoted
created: 2026-09-11
created_by: MomoChen <momochenisme@gmail.com>
promoted_to: discussion-last-cut-release
---

# Discussion: 帶 hold 的討論在所有轉出變更封存後應自動連帶封存，不靠人自己判斷

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

**起因**：`improve-repo-layout`（improve，2026-09-09，`conclude --hold`）三刀 repo-layout-groups／core-module-groups／server-module-groups 於 2026-09-10～11 全部封存，記錄仍因 `hold: true` 留在途，得手動 `speclink discuss archive` 收尾。使用者於 2026-09-11 認為這是規格設計遺留的 bug，期望「所有結論都有轉出、且都完成封存的那一刻，討論連帶封存，不靠人判斷」。

**查證（2026-09-11 main）**：不是遺留 bug——`multi-cut-discussion-hold`（2026-09-10）刻意選的取捨，正典 discussion-docs「conclude 以 --hold 保留討論在途」明文「帶 hold 的討論在其所有轉出變更封存後 SHALL 維持在途」，Example「分期三刀的生命週期」寫的正是本情況；該討論接受的代價是「最後一刀封存後使用者 discuss archive 一次」。`improve-repo-layout` 是新語意上路後第一個走到底的多刀系列（前一個 `improve-lifecycle-layer` 在舊語意下 hold 於轉出時被清、最後一刀順帶收走）。引擎沒有訊號能分辨「三刀已是全部」與「刀四還沒立案」：`close_if_finished`（lifecycle/discuss.rs:1069）三條件之「無 hold」不成立即回 `None`，archive.rs:815-826 靜默略過，CLI（verbs/lifecycle.rs:111）只印隨行封存成功行，archive 技能 asset 全文無 hold。

**需求已夠銳利**（可驗證：最後一刀封存時記錄自動移入 openspec/discussions/archive/，全程不跑 discuss archive），未經 grill 直接進 assumptions。

**相關規格**：discussion-docs（「conclude 以 --hold 保留討論在途」、隨行封存三條件、conclude 閉環）、discuss-skill（「分期轉出帶 --hold」scenario）、client-protocol／server-verb-api（conclude 請求 `hold: bool`、DiscussionInfo `hold: Option<bool>`）。相關變更：無在途；已封存 discussion-spinout-hold（09-07）、discussion-hold-until-release（09-10）。
Prior discussions: multi-cut-discussion-hold, discussion-planned-spinout-hold, discussion-auto-archive-before-conclusion

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-09-11)

**Focus**: 「最後一刀封存時討論自動連帶封存」是遺留 bug，還是要重開的設計？
**Position**: 使用者裁定重開——目標是零人工判斷，正典「帶 hold 就留在途」要改。
- 不是 bug：discussion-docs「conclude 以 --hold 保留討論在途」明文帶 hold 的討論在所有轉出變更封存後 SHALL 維持在途；`multi-cut-discussion-hold`（09-10）接受「最後一刀封存後 discuss archive 一次」為代價
- 該代價第一次真的落到使用者身上：`improve-repo-layout` 是新語意（轉出不清 hold）上路後第一個走到底的多刀系列
- 引擎現況分不出「三刀已是全部」與「刀四還沒立案」：hold 的定義是「還欠至少一個尚未建立的變更」，最後一刀封存那一刻旗標仍在
- 另一個真缺口：該提醒的那一刻沒有聲音——archive.rs:815-826 對 held 討論靜默、CLI 只印隨行封存成功行、archive 技能無 hold 處理；桌面看板的「已轉出・保留中」是唯一提示
- 同題目已被否決兩次的方向：A2 hold 帶計數（09-07、09-10：「寫 3 實際立 2 就永遠不封」）、A3 promote --last（多一個入口）；重開要說明理由為何不再成立
**Ruled out**: 維持正典＋封存時多印一行提示＋archive 技能問「這是最後一刀嗎？」——使用者否決：仍是人自己判斷，與需求相反。
**Open**: 引擎要靠什麼訊號知道「所有結論都有轉出」（hold 帶刀數倒數？別的？）；舊記錄的 `hold: true` 怎麼算；conclude 的 wire 請求要帶數字、remote 一致；discard 一刀時倒數要不要回補；improve-repo-layout 本身今天仍需手動收。

### Round 2 — interview (2026-09-11)

**Focus**: 刀數在寫結論那一刻可靠嗎——結論之後的拆刀／併刀、其他討論後續的結論會不會讓倒數算錯？
**Position**: 不可靠，倒數方案撤回；改為「最後一刀轉出時標記 `--last`」——把「這是最後一刀」的事實留到最晚、也最確定的那一刻才交給引擎。
- 實證：`improve-lifecycle-layer` 結論寫「分三刀」，實際 promoted_to 四個變更——刀三在立案期拆成 lifecycle-station-anchors 與 lifecycle-archive-gate-owner，後者在前者封存之後才立案（multi-cut-discussion-hold Context：「本系列被連帶封存三次」）。若當時用 `--hold 3`，第三次轉出即歸零，station-anchors 封存時記錄被吞，正是危險方向
- 根因：刀數是結論那一刻的意圖快照；拆刀／併刀在 propose 期才發生，其他討論之後的結論也可能吸收或新增一刀。任何在結論時固定的數字都可能被之後的決定推翻
- `--last` 掛在三條轉出路徑（`discuss promote`、`new change --from-discussion`、`discuss seal`）——累加新名字時順手移除 hold 行；其後既有三條件（無在途引用、有結論、無 hold）在最後一個封存時自動收尾，封存那段程式碼不動。忘記帶＝今天的行為（手動 archive）；帶錯只在「標記那刀封存之後才又要加一刀」時吞記錄，靠 git mv 救
- 判定「最後一刀」的是 propose 技能：讀結論的刀清單與 promoted_to，立到結論規劃的最後一刀就帶 `--last`；立案期拆刀時它自己就知道不是最後一刀。人在最後一刀封存後不必記得任何事
- 09-10 否決 A3 的理由「多一個入口，只為保住最後一刀自動收尾」——自動收尾現在就是需求本身；且它是既有轉出動詞的旗標，不是新動詞
**Ruled out**: 刀數倒數（`--hold N`）——結論時的數字擋不住立案期的拆刀，lifecycle 系列兩天前就會被吞；允許從已封存記錄轉出（取消 hold）——「已封存」不再等於「已完成」，看板「已轉出・保留中」失去意義、封存輸出「Discussion archived」誤導；每次 propose 問「這是最後一刀嗎」——每刀一問，比今天更煩。
**Open**: 使用者是否接受「由 propose 技能依結論判定最後一刀並帶 `--last`」算作零人工判斷；`--hold` 維持布林、舊記錄不動；wire 只在 promote／new change／seal 請求加 `last`（缺席＝false）、看板標籤不變；discard 掉標記那刀時不回補 hold；improve-repo-layout 本身今天手動收。

### Round 3 — interview (2026-09-11)

**Focus**: 六條假設的裁定——`--last` 掛三條轉出路徑、wire 只加 `last`、propose 技能判定最後一刀、discard 不回補、正典改動面、improve-repo-layout 手動收。
**Position**: 使用者全數確認，收斂寫結論。
- 「由 propose 技能依結論的刀清單與 promoted_to 判定最後一刀並帶 `--last`」被接受為零人工判斷：判定發生在做事的當下，不是事後要記得的雜務
- `--hold` 維持布林、舊記錄不動；封存那段程式碼與三條件不動
**Open**: 無。

## Conclusion

**Decision**: 多刀系列的討論在最後一刀封存時自動隨行封存，靠「最後一刀轉出時標記」達成，不靠刀數、不靠人事後判斷。（1）引擎：三條轉出路徑——`speclink discuss promote`、`speclink new change --from-discussion`、`speclink discuss seal`——各加布林旗標 `--last`；帶旗標且本次累加了新名字時（`DiscussionHead::promote` 回 true），於同一次寫入移除 frontmatter 的 `hold:` 行；名字已在清單（重複轉出）時旗標無效、記錄不動。hold 的解除點因此有三個：不帶 --hold 的 conclude、`discuss archive`、帶 --last 的轉出。隨行封存三條件（無在途引用、已有結論、無 hold）與 `close_if_finished`、archive.rs 的隨行封存段一行不改——最後一刀不論封存順序，最後一個封存的變更觸發收尾。不帶 --last 的轉出行為與輸出逐位元不變。`--hold` 維持布林，frontmatter `hold: true` 格式與舊記錄不動；discard 掉帶過 --last 的那一刀不回補 hold。（2）wire：`PromoteDiscussionRequest`（protocol command.rs:327）與 new change、seal 對應請求各加 `last: bool`（缺席＝false，舊 client 不受影響）；conclude 請求、`DiscussionInfo.hold: Option<bool>`、看板「已轉出・保留中」標籤與 tray 分區規則不動；remote 模式可觀察行為與本機一致。（3）技能：propose 技能從討論轉出時讀結論的刀清單（Decision 的刀一／刀二／…）與 `promoted_to`，立到結論規劃的最後一刀即帶 `--last`；立案期拆刀時不帶（拆出的最後一段才帶）；discuss 技能的多刀說明改為「conclude --hold 一次，最後一刀由 propose 帶 --last，封存時自動收尾；忘了帶就 discuss archive 一次」；improve 技能同句。（4）正典：discussion-docs「conclude 以 --hold 保留討論在途」MODIFIED（第三個解除點；Example「分期三刀的生命週期」改為第三刀帶 --last、第三次封存自動移入 archive）；discuss-skill「分期轉出帶 --hold」MODIFIED；propose-skill 新增「從討論轉出最後一刀時帶 --last」要求；client-protocol／server-verb-api 的轉出請求 MODIFIED。propose、discuss、improve 三份技能 asset 連動 ASSET_VERSION。（5）`improve-repo-layout` 已無下一刀可帶旗標，今天以 `speclink discuss archive` 手動收尾。
**Rationale**: 引擎需要的訊號是「這是最後一刀」，而這個事實只在立最後一刀的那一刻才確定——拆刀、併刀、其他討論後續的結論都在 propose 期才定案。寫結論時的刀數只是意圖快照：`improve-lifecycle-layer` 結論寫「分三刀」、實際四刀，刀三拆出的後半在前半封存後才立案，`--hold 3` 會在前半封存時吞掉記錄。旗標掛在轉出動詞、由 propose 技能依結論比對 promoted_to 判定，判定發生在做事的當下，人在最後一刀封存後不必記得任何事。忘了帶＝今天的行為（手動 archive 一次），帶錯只在「標記那刀封存之後才又要加一刀」時吞記錄、git mv 可救；不加新狀態、不加新動詞、封存程式碼不動。09-10 否決 A3 的理由「多一個入口，只為保住最後一刀自動收尾」已由需求本身推翻：自動收尾就是目標，且旗標不是新入口。
**Rejected alternatives**: 維持正典＋封存時多印一行提示＋archive 技能問「最後一刀？」——使用者否決，仍是人自己判斷；刀數倒數 `--hold N`（09-07／09-10 的 A2）——結論時的數字擋不住立案期的拆刀，lifecycle 系列的實例證明危險方向真的發生；`discuss hold <slug> N` 調數動詞——多一個入口只為修補倒數；允許從已封存記錄轉出、取消 hold——「已封存」不再等於「已完成」，看板「已轉出・保留中」失去意義、封存輸出「Discussion archived」誤導；每次 propose 問「這是最後一刀嗎」——每刀一問，比今天更煩；引擎自行推論最後一刀——沒有訊號，無出路。
**Deferred**: 帶過 --last 的那一刀被 discard 後不回補 hold——重立通常緊接其後、其他刀在途即撐住記錄，否則 git mv，再發生一次才立案；封存時對 held 討論多印一行提示——正常路徑已由自動收尾覆蓋，若「忘帶 --last」重複發生再議；看板顯示「還欠幾刀」——沒有刀數，無此需求。
**Capture to**: proposal
**Next**: /speclink-propose --from-discussion hold-countdown-auto-close

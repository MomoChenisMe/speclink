---
topic: discuss 中途轉出的 change 先封存時，未有結論的討論被連帶封存
slug: discussion-auto-archive-before-conclusion
status: promoted
promoted_to: conclusion-gated-discussion-archive
created: 2026-08-31
created_by: MomoChen <momochenisme@gmail.com>
---

# Discussion: discuss 中途轉出的 change 先封存時，未有結論的討論被連帶封存

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

使用者發現：discuss 中途 spin-out 一份 change 後，若該 change 先完成封存，尚未有結論的討論會被連帶封存。查證屬實——archive.rs:737 的連帶封存唯一守門是「還有無其他在途 change 引用」，archive_discussion（discuss.rs:834）完全不看討論狀態與 Conclusion 是否已寫。封存後 add-round／conclude 被拒，僅能手動搬檔救援。測試僅覆蓋已寫結論的 promoted 文件，屬規格盲區；且與 skill 文件「promotion does not close the record」的承諾矛盾。無需 grill：問題本身已附驗證目標。相關程式面：crates/speclink-core/src/archive.rs、discuss.rs（conclusion_text() 於 :452 可判斷結論是否已寫）；desktop 呈現鏈：speclink-protocol query.rs DiscussionInfo → server routes → packages/ui adapter.ts DiscussionItem → DiscussionColumn.tsx／TrayPanel.tsx。無既有 change 或 spec 直接涵蓋此行為。

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-08-31)

**Focus**: 連帶封存是否會掃走未有結論的討論？屬實後補救方向選哪條？
**Position**: 屬實，補救走方向 A——Conclusion 未寫就不連帶封存（使用者確認）。
- 證據：archive.rs:737 過濾器只看「其他在途 change 是否仍引用」；discuss.rs:834 archive_discussion 不檢查狀態或結論
- 封存後 add-round／conclude 被拒（discuss.rs:248），救援僅手動搬檔
- 三個連帶封存測試（archive.rs:1195 起）全用已寫結論的 promoted 文件，未結論情境零覆蓋＝規格盲區
- 守門判準必須用 conclusion_text()（discuss.rs:452）看 Conclusion 內文；不能用 status==concluded，因 promoted 討論寫完結論後 status 仍為 promoted（discuss.rs:873 註解）
- A 的閉環子案：conclude 時回頭檢查 promoted_to 的 change 是否全數封存，是則順手封存討論，不留孤兒
**Ruled out**: B（討論沒結論就擋 change 封存）——因果顛倒，change 完工不該被在途討論扣住；C（維持現狀＋unarchive 救援動詞）——治標，留髒狀態
**Open**: desktop 應能看出討論是否已有結論（使用者新增需求）；A 閉環子案（conclude 順手封存）的細節確認

### Round 2 — assumptions (2026-08-31)

**Focus**: desktop 如何呈現「已轉出但尚無結論」的討論？
**Position**: 走 (b) 改語意——未結論的 promoted 討論留在看板上區全尺寸卡，有結論才收進「已轉出」收合列（使用者確認）。
- 現況缺口：DiscussionInfo（protocol query.rs:562）與 DiscussionItem（ui adapter.ts:115）皆無「結論已寫」欄位；promoted 討論的 status 永遠停在 promoted，寫完結論也不變，前端無從分辨
- 現況語意缺陷：DiscussionColumn.tsx:295 以 status==promoted 一律收進欄底收合列，轉出當下還在進行的討論就從上區消失；TrayPanel.tsx:470 同樣二分
- 配套（事實非決策）：資料鏈加 concluded 布林欄位，引擎以 conclusion_text() 派生，經 protocol → server route → ui adapter 傳到前端；上區的 promoted 未結論卡帶「已轉出・尚無結論」狀態標
- (b) 與引擎修法 A 同語意：討論的生命由結論決定，不由轉出決定
**Ruled out**: (a) 只在收合列加「尚無結論」小標——治不了「進行中的卡從上區消失」的問題；可作為 (b) 的一部分保留標示本身
**Open**: 無——標示文案與樣式細節留給 propose/design 階段

## Conclusion

**Decision**: 引擎與 desktop 兩側對齊「討論的生命由結論決定，不由轉出決定」。引擎：連帶封存守門擴充——Conclusion 未寫（以 conclusion_text() 判斷，不用 status）就不隨 change 封存；conclude 時回頭檢查 promoted_to 的 change 是否全數封存，是則順手封存討論。desktop：資料鏈（core → protocol DiscussionInfo → server route → ui DiscussionItem）加 concluded 布林欄位；看板與 tray 的分區改語意——未結論的 promoted 討論留在上區全尺寸卡並帶「已轉出・尚無結論」標示，有結論才收進「已轉出」收合列。
**Rationale**: 現行連帶封存唯一守門是「其他在途 change 是否仍引用」（archive.rs:737），未結論討論被掃走後 add-round/conclude 全被拒、僅能手動搬檔，且與 skill「promotion does not close the record」的承諾矛盾；status 對 promoted 討論是死路（寫完結論仍為 promoted），故引擎守門與前端呈現都必須改看 Conclusion 內文。
**Rejected alternatives**: B 討論未結論就擋 change 封存——因果顛倒，change 完工不該被在途討論扣住；C 維持現狀＋unarchive 救援動詞——治標且留髒狀態；(a) 僅在收合列加「尚無結論」小標——治不了進行中的卡從上區消失的問題（標示本身併入 (b) 保留）。
**Deferred**: 「已轉出・尚無結論」標示的文案與樣式細節，留給 propose/design 階段。
**Capture to**: proposal
**Next**: /speclink-propose --from-discussion discussion-auto-archive-before-conclusion

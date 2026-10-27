---
topic: change 砍掉另開時的討論生命週期——廢棄動詞設計
slug: discard-change-verb
status: promoted
promoted_to: discard-change-verb
created: 2026-07-09
---

# Discussion: change 砍掉另開時的討論生命週期——廢棄動詞設計

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

承接 rediscuss-promoted-change 的收尾討論：使用者指出三個相連的漏洞——死掉的變更名留在討論的 promoted_to、移除 change 只能手動刪目錄（繞過所有生命週期機制）、砍掉後原討論可能不再繼續（promoted 孤兒討論在看板上沒有收尾入口，GUI 僅 concluded 卡有封存動詞）。

模式：assumptions（碼庫命中：apps/desktop/core/src/verbs.rs 的 GUI 動詞面僅 validate/analyze/archive、discuss.rs 的狀態機寫入點與 discuss discard 守衛先例、inprogress.rs 的 started_at 開工標記）。引擎與 GUI 均無任何 change 刪除指令。

相依變更：rediscuss-promoted-change（from_discussion 改累積器）——解鏈邏輯須逐 slug 建立在其上。

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-07-09)

**Focus**: change 廢棄的機制層級、討論側處置與守衛方向
**Position**: 引擎新增頂層廢棄動詞 speclink discard <change>——刪除變更目錄並對討論側逐 slug 解鏈；使用者確認守衛方向：
- 手動刪目錄繞過生命週期是漏洞根源；GUI 動詞面（verbs.rs）僅 validate/analyze/archive，引擎無任何刪除指令——動詞化才能在刪除同時做討論側清理
- 解鏈語意：從每份 linked discussion 的 promoted_to 移除該變更名；清單空時狀態回退——有結論退 concluded、無結論退 open（link 允許 open 討論併入）；回退讓討論重接既有出路（concluded 卡的 GUI 封存動詞、再 promote 開後繼）
- frontmatter 連結欄位是可變 metadata（mark_promoted 本就改寫它），rounds/結論才是 append-only ledger——解鏈不是改寫歷史
- 守衛鏡射 discuss discard：meta 有 started_at 或 tasks.md 有已勾任務即拒絕，--force 放行；使用者確認砍 change 幾乎都在動工前——動工後異動走 discuss+ingest 修正現有變更，或 archive 後新開 discuss+propose——守衛不會淪為每天 --force 的儀式
- 與 rediscuss-promoted-change 相依：解鏈須逐 slug 走 from_discussion 累積器讀取，實作順序在其後
**Ruled out**: 留 stale 條目＋放寬「promoted 孤兒討論可封存」守衛——看板衛生差、機制多一套，且詞彙定義「已轉出變更＝至少連結一個變更」在最後連結死亡後不再真實；僅文件慣例不動引擎——無法阻止黑戶刪除；支援動工後砍除——非正常流程，ingest 與 archive 已覆蓋
**Open**: GUI 是否提供捨棄動詞（暫傾向比照討論 discard 排除於 GUI，屬 agent/CLI 領域）

## Conclusion

**Decision**: 新增引擎頂層廢棄動詞 speclink discard <change>（拼寫鏡射頂層 archive）：刪除變更目錄，並對其 from_discussion 清單中的每份討論解鏈——promoted_to 移除該變更名；清單因此變空時狀態回退（記錄有結論→concluded、無結論→open）並移除空的 promoted_to 行；輸出報告每份解鏈討論與其回退後狀態。守衛：變更有動工痕跡（meta 的 started_at 或 tasks.md 任何已勾任務）時拒絕，--force 放行；變更不存在時報錯。
**Rationale**: 手動刪目錄是繞過生命週期的黑戶操作，正是 stale promoted_to、promoted 孤兒討論兩個漏洞的共同根源。解鏈＋狀態回退讓討論重接既有出路：concluded 卡在 GUI 本就有封存動詞（孤兒問題消失），也可再 promote 開後繼變更（砍掉另開流程：discard c1 → promote D --name c2，promoted_to 無殘留）。使用者確認砍 change 幾乎都在動工前；動工後異動走 discuss+ingest 或 archive 後新開 discuss+propose，故嚴格守衛不會儀式化。
**Rejected alternatives**: 留 stale 條目＋放寬 promoted 孤兒討論的封存守衛（看板衛生差、機制多一套、promoted 狀態在最後連結死亡後不再真實）；僅文件慣例不動引擎（無法阻止黑戶刪除）；支援動工後砍除為一級流程（非正常時機，ingest／archive 已覆蓋）。
**Deferred**: GUI 是否提供捨棄動詞——暫傾向比照討論 discard 排除於 GUI（agent/CLI 領域），留待 propose 裁定；與 rediscuss-promoted-change 的實作順序相依（解鏈逐 slug 走累積器讀取）須寫入新變更的 proposal。
**Capture to**: proposal
**Next**: /speclink-propose --from-discussion discard-change-verb

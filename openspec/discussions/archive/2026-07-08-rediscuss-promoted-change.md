---
topic: 當 discuss 已延伸出 changes 後，某個 change 需要重新 discuss 的設計解法
slug: rediscuss-promoted-change
status: promoted
promoted_to: rediscuss-promoted-change
created: 2026-07-08
---

# Discussion: 當 discuss 已延伸出 changes 後，某個 change 需要重新 discuss 的設計解法

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

使用者發現：discuss 已扇出 changes 後，某個 change 需要再次 discuss 時，現行設計卡住——`discuss link` 在 change 已有 `from_discussion` 時直接 bail（discuss.rs:404），skill 文件的「結論導向既有 change → link → ingest」流程恰好在 change 出身自討論時斷路，而這種 change 正是最可能需要再討論的。

模式：assumptions（codebase 大量命中：model.rs:16-18 的 `from_discussion: Option<String>` 單值、discuss.rs 的 link/mark_promoted、archive.rs:218-229 的封存共行、App.tsx:215-222 的 sibling 群組）。討論側 `promoted_to` 已是逗號累積器（1→N），change 側仍是單值（N→1），但 re-discussion 情境揭示關係本質是 M↔N。

相關 in-flight changes：無直接關聯。

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-07-08)

**Focus**: 病灶定位——再討論已轉出的 change 時，斷點在哪一層
**Position**: 病灶是 change 側 `from_discussion` 的單值防衛，不是 discuss 生命週期本身；解法方向為改成逗號累積器，關係升級 M↔N：
- discuss.rs:402-404 的 link 防衛在 change 已有 `from_discussion` 時 bail，re-discussion 的新討論無法 link 回既有 change，卡死在 skill 已文件化的 conclude→link→ingest 流程上
- 解法鏡射 `promoted_to` 既有累積器模式（discuss.rs:299-316）：單值是逗號清單的退化情形，既有記錄零遷移；第一個條目天然保留「出身討論」語意
- 封存共行改為逐 slug 判定（archive.rs:218-229），「討論隨最後一個引用它的存活 change 封存」規則不變，只是從單值變迴圈
- GUI 影響面小：App.tsx:215-222 的 sibling 判定改為集合有交集、「來自討論」chip 支援多值；看板討論卡收合群組走 promoted_to 方向，不受影響
**Ruled out**: 重開原討論續談——原討論可能已扇出多個 changes，re-discussion 通常只針對其一，scope 不符；rounds append-only 會被跨主題輪次污染；狀態機 open/concluded→promoted 單向（discuss.rs:293-298），逆轉會攪亂封存共行
**Open**: 使用者提出替代案「把舊討論中對應該 change 的內容抽到新討論作第一輪，讓舊討論不再被依賴」——抽取 vs. 連結待裁定；CLI JSON 與 desktop bridge 的輸出形狀如何演進

### Round 2 — assumptions (2026-07-08)

**Focus**: 抽取舊討論內容 vs. 連結引用——新討論如何自足
**Position**: 採「累積器＋Context 引用」，否決抽取；使用者確認買單：
- 累積器是純連結不搬移：唯一改動是 change meta 的 `from_discussion` 追加一個 slug，兩份討論文件一字不動、各自成篇
- 抽取的拆不乾淨是結構性的：一輪常同時裁定多個 change 的邊界，抽「某 change 的部分」必為有損切割，且切完舊記錄敘事也破
- 抽取違反 rounds append-only：抽走後舊討論的 `promoted_to` 仍列該 change 是謊言、移除則是改寫歷史
- 「讓舊討論不被依賴」要解的問題不存在：結論後的討論唯讀，被引用零維護成本
- 上下文自足的正當需求由新討論的 Context 區摘要引用舊裁定達成（skill 本就必填 Context，零引擎改動；LLM 風險從拆整份文件縮到寫一段話）
**Ruled out**: 抽取舊討論內容到新討論作第一輪——結構性有損＋違反 append-only＋為不痛的依賴付出破壞 ledger 的代價
**Open**: 無——方向收斂，進結論

## Conclusion

**Decision**: change 側 `from_discussion` 從單值改為逗號累積器（鏡射討論側 `promoted_to` 的既有模式），`discuss link` 對已連結的 change 從 bail 改為追加（同 pair 重複 link 維持冪等跳過）；封存共行改為逐 slug 判定（每個 linked discussion 各自檢查是否仍被存活 change 引用）；re-discussion 以「新討論＋Context 區摘要引用舊裁定」進行，不抽取舊討論內容。
**Rationale**: re-discussion 情境揭示討論↔change 關係本質是 M↔N，而現行 change 側單值（model.rs:16-18）讓 skill 已文件化的 conclude→link→ingest 流程恰在 change 出身自討論時斷路。累積器模式在討論側已驗證（discuss.rs:299-316），單值是逗號清單的退化情形，既有記錄零遷移；第一個條目天然保留「出身討論」語意。
**Rejected alternatives**: 重開原討論續談（原討論可能扇出多個 changes，scope 不符；污染 rounds append-only；逆轉單向狀態機）；抽取舊討論中對應該 change 的內容到新討論作第一輪（一輪常同時裁定多個 change，抽取必為結構性有損切割；抽走後舊記錄的 promoted_to 成謊言或改寫歷史；「不被依賴」要解的問題不存在——結論後的討論唯讀零維護成本）。
**Deferred**: CLI JSON 與 desktop bridge（apps/desktop/core/src/discussions.rs）對多值 from_discussion 的輸出形狀——CLI 輸出是回歸保護對象，留待 propose 的 design 裁定；「來自討論」chip 與 sibling 群組（App.tsx:215-222）的多值呈現細節。
**Capture to**: proposal
**Next**: /speclink-propose --from-discussion rediscuss-promoted-change

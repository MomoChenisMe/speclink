---
topic: analyze 規則優化：design 標題比對、REMOVED 需求檢查與其他誤報面
slug: analyze-rule-tuning
status: promoted
promoted_to: analyze-rule-tuning, validate-merge-gate
created: 2026-09-09
created_by: MomoChen <momochenisme@gmail.com>
---

# Discussion: analyze 規則優化：design 標題比對、REMOVED 需求檢查與其他誤報面

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

起因：另一個 session 在 wadpilot repo 跑 speclink analyze，對 change remove-list-tab-depth-cap 留下三個 Warning（CON-1／CON-2 design 標題未出現在 tasks、AMB-1 REMOVED 需求無 scenario），判斷為誤報並請本 session 評估規則。本 session 讀 crates/speclink-core/src/analyzer.rs 後確認三個都是 analyze 的規則問題，validate 無責。使用者先問 validate 能怎麼優化，釐清分工後把主題收斂到 analyze，並要求檢視其餘規則有無可優化處。
需求已夠清楚（三個具體誤報＋量化證據），不需 grill 階段。
相關規格與程式：analyze 四個面向目前沒有正典規格（只有 drift-computation 與 desktop-app 順帶提到）；鄰近 capability 為 spec-validation（validate）、drift-computation（drift）。規則實作全在 crates/speclink-core/src/analyzer.rs，該檔零個單元測試；CLI 端 crates/speclink-cli/tests/it/remote_verb_parity.rs 只比對輸出形狀。凍結權威自 2026-07-27（commit 3b8661e0）起為 speclink 自身契約，允許與 OpenSpec 原版分岔。
Prior discussions: manual-marker-placement-lint

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-09-09)

**Focus**: 三個回報的 Warning 是誰的問題、要不要改，以及改的方向
**Position**: 三個都是 analyze 的規則問題，CON 與 AMB 要改，COV 維持；規則變更需要新開 capability 承載
- 決策樹：① 歸屬（validate／analyze）→ ② 三條規則各自去留 → ③ 比對規則的形狀 → ④ 規格落點 → ⑤ 其餘規則掃描
- ① 歸屬：三個 Warning 全部出自 analyzer.rs；validate 只管結構（操作區塊、重複需求名、[M] 標記位置、新 capability Purpose）。先前討論 manual-marker-placement-lint 已定「格式守門歸 validate 單一落點、analyze 是內容品質檢查」，本次沿用
- ② conDesignNotInTasks：把 design 的 ### 標題整串轉小寫去 tasks.md 全文找。封存 change 統計：743 個標題、468 個帶序號前綴（D1／D1：／決策一：／Decision 1）、653 個整串被抄進 tasks。規則實際效果是逼人抄標題，不是檢查任務有沒有落實設計
- ② ambNoScenario：不分區塊對所有需求要 scenario；但 speclink 自己的範本（fork.schema.yaml 的 REMOVED 範例）只寫 Reason 與 Migration。工具自相矛盾
- ② covMissingTask：移除功能要有任務去拆，要求 tasks 提到需求名站得住。封存統計 880 條需求名命中、33 條未命中，規則可用
- ③ 新比對規則（使用者裁定「編號比對做」）：標題拆成「編號」與「本文」，tasks 含本文或含編號任一即算引用。編號只認 D1／決策一／Decision 1 三種樣式（涵蓋全部 468 個），D1 要檢查後一字非數字免被 D12 誤命中；無前綴標題行為與現在完全相同
- ③ REMOVED 需求：no-scenario 檢查跳過 REMOVED；改查 **Reason** 與 **Migration** 是否齊備（封存統計 18 條 REMOVED 需求、1 條缺）
- ④ 規格落點（使用者裁定「開新 capability」）：analyze 四個面向沒有正典規格，新開一個 capability 把規則寫進正典；名稱候選 change-analysis，與 spec-validation／drift-computation 同一命名線
**Ruled out**: 只做前綴去除不做編號比對（tasks 仍要抄本文，抄寫稅沒消）；模糊／token 比對（難解釋、命中不可預測）；covMissingTask 對 REMOVED 改看 Migration（比對更模糊）；把合併守門搬進 analyze（歸 validate，另案處理）
**Open**: 其餘規則（ambAbstractScenario、ambWeakLanguage、Gaps 兩條、parse_capabilities）有無可優化處；capability 名稱定案；--strict 是否影響 analyze

### Round 2 — assumptions (2026-09-09)

**Focus**: analyze 其餘規則有無可優化處，以及合併守門的歸屬
**Position**: 這次 change 再多做兩條 Ambiguity 微調，Gaps 不動；合併守門接進 validate 另開 change
- ambAbstractScenario：具體值字元集加全形引號「」『』與全形數字。封存 delta 被判抽象的 scenario 1776 個，363 個含這些字元；Suggestion 等級，不擋蓋章但每次都吵
- ambWeakLanguage：英文樣式改字邊界比對（shoulder／mayor／considerable 不再命中）。本 repo 正典只有 2 處英文命中，但 wadpilot 的規格是英文，那邊有感
- Gaps 兩條（gapNoMainSpec、gapModifiedNotFound）不動。archive.rs 的 merge_violations 看得更全（含 REMOVED／RENAMED／ADDED 撞名），但它是「archive 會不會拒收」的結構判斷，依先前討論 manual-marker-placement-lint 的分工歸 validate；使用者裁定另開 validate 的 change
- 不需改的項目（事實核對）：analyze 無 --strict 旗標；proposal 的 capability 抽取對全部封存 change 零誤抽；design.md 的 ### 標題 743 個只有 4 個落在 Decisions 區段外，限縮區段改變不了結果
- 新 capability 名稱定為 change-analysis，與 spec-validation／drift-computation 同一「對象加動作」命名線
**Ruled out**: 反引號內識別符豁免弱語氣（沒有資料證明需要）；CJK 弱語氣詞對專案術語（如「可能過期」）的豁免表（要讀 LANGUAGE.md，這次不值得，延後）；conDesignNotInTasks 限縮到 Decisions 區段（零效果）；把合併守門放進 analyze（歸 validate）
**Open**: 無。validate 接合併守門的 change 是獨立主題，需求已夠清楚，直接 propose 不必再討論

## Conclusion

**Decision**: 開一個 change，新開 capability `change-analysis` 把 analyze 四個面向的規則寫進正典，並修四條規則：(1) Consistency 的 design 標題比對改為「本文或編號任一出現在 tasks 即算引用」，編號只認 D1／決策一／Decision 1 三種樣式，D1 後一字須非數字，無前綴標題行為不變；(2) Ambiguity 的 no-scenario 檢查跳過 REMOVED 需求，改對 REMOVED 需求檢查 **Reason** 與 **Migration** 齊備（Warning）；(3) Ambiguity 的具體值判斷加全形引號「」『』與全形數字；(4) Ambiguity 的英文弱語氣詞改字邊界比對。Coverage 與 Gaps 不動。analyzer.rs 補單元測試模組，至少涵蓋三種前綴、無前綴回歸、REMOVED 跳過與 Reason／Migration 檢查、全形具體值、字邊界。--json 輸出形狀與既有訊息文字不變。
**Rationale**: 現行 design 標題規則逼人把標題整串抄進 tasks（封存 change 88% 照抄），沒有在檢查任務有沒有落實設計；REMOVED 需求被要求寫 scenario 與 speclink 自己的範本矛盾。凍結權威自 2026-07-27 起為 speclink 自身契約，允許分岔；規則沒有正典規格是這次順手補的原因。
**Rejected alternatives**: 只去前綴不做編號比對（抄寫稅沒消）；模糊／token 比對（難解釋、命中不可預測）；covMissingTask 對 REMOVED 改看 Migration（更模糊，且移除功能本來就該有任務去拆）；合併守門放進 analyze（依 manual-marker-placement-lint 的分工歸 validate）；限縮 design 標題到 Decisions 區段（743 個標題只有 4 個在區段外，零效果）；反引號豁免弱語氣（無資料支持）。
**Deferred**: validate 接上 archive 的合併守門 merge_violations，讓 validate 早期抓到 archive 會拒收的 delta（MODIFIED 目標不存在、ADDED 撞名、未宣告的 scenario 移除等）——使用者裁定另開 validate 的 change，需求已夠清楚，直接 /speclink-propose 不必再討論。CJK 弱語氣詞對專案術語（如「可能過期」）的豁免表——延後，等有第二個實例再看。
**Capture to**: spec
**Next**: /speclink-propose --from-discussion analyze-rule-tuning

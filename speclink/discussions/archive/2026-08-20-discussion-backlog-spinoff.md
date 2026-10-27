---
topic: 多需求討論的 backlog、恢復摘要與中途轉出
slug: discussion-backlog-spinoff
status: concluded
created: 2026-08-20
created_by: MomoChen <momochenisme@gmail.com>
---

# Discussion: 多需求討論的 backlog、恢復摘要與中途轉出

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

起因：使用者揭露討論記錄功能的原始動機（spectra 時代三痛點：session 壓縮／重啟丟內容、回來忘了談到哪要翻對話、一次談 5-10 個需求想中途先 propose 幾個）。查核結果：前兩痛已由現行設計對應（落盤記錄、Open 邊界欄），第三痛半套——引擎已支援中途轉出（promote 不需 conclude，discuss.rs 測試 promote_prefills_topic_when_no_conclusion；add_round 無已結論閘門，discuss.rs:303-327），但 skill 沒教，且多需求無結構化追蹤。

模式：assumptions——scout 命中正典 discussion-docs、discuss-skill 兩 capability 與 crates/speclink-core/src/discuss.rs、assets/skills/discuss.md。

相關討論：discuss-spec-grounding（已結論，同屬 discuss skill 優化系列）。

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-08-20)

**Focus**: 恢復討論時使用者最想先看到什麼？
**Position**: 摘要優先——「每輪談了什麼＋決策一句話怎麼走」，其次才是剩餘清單。
- 使用者親答：最想看到「當前都討論了什麼、最後決策怎麼走」
- 關鍵發現：這份摘要現有格式已可機械推導——各輪 Focus＝談了什麼、Position 首句（規則五已強制一句話定論）＝決策怎麼走、最後一輪 Open＝剩餘邊界；不需要新格式、不需要引擎改動
- 恢復儀式因此定形：續用 open 討論時，skill 要求先呈現「逐輪 Focus→定論一句話」清單＋目前 Open 邊界，再繼續
- backlog 慣例縮小為補充：多需求（5-10 個）第一輪把全清單攤進 Open，之後每輪 Open 復述剩餘項（規則六本就要求 Open＝完整邊界），已定案項的去向靠該輪 Position 首句承載
**Ruled out**: 新增結構化 backlog 區段（動引擎、動骨架）——摘要可從既有欄位推導，新區段是多餘結構
**Open**: 中途轉出的 skill 教學怎麼寫？promote 後 status=promoted、conclude 的字串翻轉只認 status: open（discuss.rs:761）——中途轉出後補結論的狀態行為要查證，可能需引擎小修

### Round 2 — assumptions (2026-08-20)

**Focus**: 中途轉出後補結論，引擎狀態行為是否需要修？
**Position**: 不需修——引擎已刻意支援此流程，優化收斂為純 skill 文字改動。
- discuss.rs:759-761 註解明言：promoted 討論沒有 status: open 可翻轉，re-conclude 刻意保留 promoted；Conclusion 區段照寫（replace_section）
- discuss.rs:773-776 stamp_restale：對已轉出討論補結論時，其活躍變更全部標為待重新反映並回報 CLI
- 完整流程引擎皆通：中途 promote（無結論用 topic 預填）→ 繼續 add-round → 最後 conclude（保留 promoted、寫結論、標 stale）
- 小瑕疵誠實面：stamp_restale 不分項目——中途轉出的變更 A 與後續結論無關時也會被標 stale，多一次確認噪音；屬可接受代價，標記本意就是「去看一眼」
**Ruled out**: 引擎小修（結論按項目選擇性標 stale）——噪音一次一眼，選擇性標記要引擎理解「項目」概念，複雜度不成比例
**Open**: 三條優化（恢復摘要儀式／backlog 慣例／中途轉出教學）併入 discuss skill 資產同一提案，或與 discuss-spec-grounding 的兩案合流成一個變更？

## Conclusion

**Decision**: 三條優化全數採納，且與 discuss-spec-grounding 的兩案（漏斗接地＋grill 模式重構）合成單一變更提案，一次改 discuss skill 資產。
1. 恢復摘要儀式：續用 open 討論時，skill 先呈現「逐輪 Focus→Position 首句定論」清單＋最後一輪 Open 邊界，再繼續討論；摘要自既有欄位機械推導，零新格式。
2. Backlog 慣例：多需求討論第一輪把全清單攤進 Open，之後每輪 Open 復述剩餘項；已定案項去向由該輪 Position 首句承載。
3. 中途轉出教學：談定一項即可 promote（引擎無結論時以 topic 預填提案），討論繼續加輪，最後補 conclude 保留 promoted 狀態並將已轉出變更標為待重新反映（stamp_restale）。
**Rationale**: 使用者原始三痛點中「多需求中途分岔」是唯一半套的一項；引擎（discuss.rs:303-327、759-776、promote_prefills_topic_when_no_conclusion）早已支援全流程，缺的只是 skill 教學與恢復呈現慣例——零引擎改動即補完。合流成一案：同檔同性質，省一次 MARKER_VERSION／golden／assets.lock 三連動，避免兩案互等。
**Rejected alternatives**: 結構化 backlog 區段（動引擎動骨架，摘要可推導故多餘）；stamp_restale 按項目選擇性標 stale（引擎需理解「項目」概念，複雜度不成比例，確認噪音一眼可解）；兩討論分開兩個變更（同檔資產、互相等待）。
**Deferred**: none
**Capture to**: proposal——與 discuss-spec-grounding 同一變更（crates/speclink-core/assets/skills/discuss.md；Impact 含三連動與 32 份 SKILL.md 再生）
**Next**: /speclink-propose --from-discussion discuss-spec-grounding 建立變更後，speclink discuss link discussion-backlog-spinoff <change-name> 把本討論鏈上同一變更

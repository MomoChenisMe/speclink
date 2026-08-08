---
topic: discuss 技能的 grill me 落地檢討——可行性先行拷問還需要嗎
slug: feasibility-first-discuss
status: concluded
created: 2026-08-07
created_by: MomoChen <momochenisme@gmail.com>
---

# Discussion: discuss 技能的 grill me 落地檢討——可行性先行拷問還需要嗎

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

起因:使用者觀察 grill me(mattpocock 的 grilling)併入後,discuss 仍以 assumptions 模式為主、沒有拷問感;當初納入 grill 的動機是「先依討論內容研究 codebase 回答可行性,確認所有可行性後再依結論依序拷問」。本討論檢討 grill 落地與該需求是否仍成立。模式選 assumptions:相關脈絡充分(.claude/skills/speclink-discuss/SKILL.md、openspec/specs/discuss-skill/spec.md 六條 Requirement、discussions/archive/2026-07-30-grill-mode-in-discuss.md、引擎 discuss.rs)。前史:grill-mode-in-discuss(7/30 結論、promoted → discuss-decision-tree-interview)、discuss-propose-from-docs(文件輸入三分診)。血緣:speclink 版 discuss 源自 spectra-discuss(使用者提供原文),外部參照 Fission-AI/OpenSpec 的 /opsx:explore。

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-08-07)

**Focus**: 目前 discuss 的實際行為為何、grill me 併入後落在哪裡
**Position**: grill 當初被拆半落地,現行為與 7/30 裁定一致;使用者無感其來有自:
- 決策樹拷問紀律只落在 interview 模式(依賴序一次一題、每題附建議答案與 Evidence、事實自查決策問人);relentless 停止條件當初即否決
- 模式閘門(3+ 相關檔 → assumptions)使成熟 codebase 幾乎永遠走 assumptions,樹紀律形同休眠——拷問感從未在日常路徑出現
- 「先全面研究再拷問」在 7/30 Round 4 被明確否決,改為「開場淺掃選模式、沿樹逐節點查證」
- 使用者要的形狀(先逐條查可行性、再只拿真決策依序問)已存在於 Document input 三分診,但僅主題為文件路徑時觸發
- 使用者本輪表態:修改前的 discuss 已經不錯——可行性先行的需求可能不再成立,方向轉向檢討 grill 殘留去留
**Open**: 可行性先行需求是否仍成立;grill 殘留(interview 樹紀律)去留;OpenSpec 的 explore 是否才是該需求的歸屬

### Round 2 — assumptions (2026-08-07)

**Focus**: 可行性先行需求是否仍成立;explore 是否為其歸屬;grill 殘留去留
**Position**: 三個開放問題一次收攏——需求不再成立,discuss 維持現狀:
- 查證 OpenSpec /opsx:explore 原文(src/core/templates/workflows/explore.ts):無固定步驟、明文禁止漏斗式問題鏈("Don't funnel them through a single path of questions")、不強迫收斂——「先研究」半段與需求重疊,「依序拷問」半段正好相反,explore 不是需求歸屬
- 與 spectra-discuss 原文比對(使用者提供):grill 變更只動了 How to Discuss 一節與記錄慣例第 6 條;assumptions 模式閘門 spectra 原版即有
- 修改前後在使用者日常路徑(assumptions 模式)行為相同——「修改前已不錯」等價於「現在也不錯」,痛點不足以立案
- grill 殘留(interview 樹紀律)保留:回滾需動三處技能檔+MARKER_VERSION/golden/assets.lock 三連動+規格四條 Requirement,換得的行為差異幾乎不可見
**Ruled out**: 把文件三分診推廣為所有主題開場(上輪提案)——需求不再成立,無痛點支撐;回滾 grill 樹紀律——churn 大於收益;引進 explore 型動詞——形狀與「依序拷問」需求相反
**Open**: none

## Conclusion

**Decision**: discuss 技能維持現狀,不做任何調整——不推廣可行性先行三分診、不回滾 interview 樹紀律、不引進 explore 型動詞。
**Rationale**: 「先確認所有可行性再依序拷問」的痛點經檢驗不成立——grill 變更只動了 interview 模式,而日常路徑幾乎都走 assumptions,修改前後行為相同,使用者對修改前的滿意等價於對現狀的滿意;grill 樹紀律睡在 interview 模式不礙事且綠地題目仍有用,回滾成本(三處技能檔+MARKER_VERSION/golden/assets.lock 三連動+規格四條 Requirement)遠大於幾乎不可見的行為差異。
**Rejected alternatives**: 文件三分診推廣為通用開場——無痛點支撐;回滾 grill 樹紀律——花一次變更買無感差異;explore 型動詞——OpenSpec explore 明文反收斂、反漏斗提問,與「依序拷問」需求相反。
**Deferred**: none
**Capture to**: 無 artifacts 需更新(結論=不改)
**Next**: speclink discuss archive feasibility-first-discuss

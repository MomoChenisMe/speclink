---
topic: verify/review 蓋章阻斷集與 skill must-fix 定義的落差怎麼調
slug: stamp-blocking-set-alignment
status: promoted
promoted_to: stamp-blocking-set-alignment
created: 2026-08-08
created_by: MomoChen <momochenisme@gmail.com>
---

# Discussion: verify/review 蓋章阻斷集與 skill must-fix 定義的落差怎麼調

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

起點是記憶條目「verify stamp 卡在任何 finding」:引擎的乾淨蓋章要求末輪零 finding(含 SUGGESTION),但兩站 skill 的「阻斷集」只算 must-fix,落差每次都撞到,還產生「記錄前先想清楚值不值得」的反誘因——agent 為避免摩擦而不記 SUGGESTION,紀錄誠實性被扭曲。

模式:assumptions(程式碼證據充足——station.rs、verify.rs、review.rs、兩份 SKILL.md、四份 spec)。

偵察結果:
- 引擎守門在 station.rs:419,`findings.len()` 不分嚴重度;正典為 verify-station/spec.md:130 與 review-station/spec.md:150 的「末輪零未解 findings,--accept 僅豁免此條」。
- skill 側 verify SKILL.md:188-190、review SKILL.md:123-125 定義阻斷集=must-fix(CRITICAL+有現實觸發路徑的 WARNING),SUGGESTION 歸可裁。
- quality skill 收尾補蓋已用「must-fix 淨空」當門檻(speclink-quality SKILL.md:51)。
- desktop GUI 僅 store 測試碰到 Severity,無「零 findings 才能蓋」的呈現依賴。
- meta 蓋章五欄不記錄是否 --accept——帶保留與乾淨章在 meta 上無法區分,誠實性靠工單 git 歷史。

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-08-08)

**Focus**: 兩套阻斷定義(引擎「零 findings」vs skill「零 must-fix」)該往哪邊收斂
**Position**: 改引擎門檻,讓嚴重度標籤成為阻斷分界的正典——使用者確認全部四項假設:
- 乾淨章守門 (2) 從「末輪零 findings」改成「末輪零未解 must-fix」;SUGGESTION 不擋章,記了也能直接蓋(station.rs:419 改為過濾 Severity::Suggestion)
- 阻斷分界=嚴重度:CRITICAL/WARNING 擋章、SUGGESTION 不擋;skills 端收緊「可裁 ⇒ 一律記 SUGGESTION」(review SKILL.md:102 本就允許 smell 記 WARNING 或 SUGGESTION,只是收緊裁量)
- --accept 語意不變,「保留」收窄為 must-fix 級:帶 (accepted) 的 must-fix 照樣擋乾淨章、照樣要 --accept;SUGGESTION 從此不需任何人批准
- 波及面:verify-station/review-station 兩份 spec 守門條文與 Scenario、verify-skill/review-skill 兩份 spec 三選項敘述、station.rs+各層測試(verify_verbs.rs、review_verbs.rs、golden)、兩份 SKILL.md 的 MARKER_VERSION/golden/assets.lock 三連動;desktop 不用動
- 改完 quality skill 反而更對齊——收尾補蓋本來就以 must-fix 淨空為門檻
**Ruled out**: 只改 skill 措辭對齊引擎——驚訝消失但摩擦留著,每筆 SUGGESTION 仍逼一次使用者互動,反誘因不動;行內發明「可裁」新標記讓引擎讀懂判斷式分類——格式面、解析器波及都更大,嚴重度已足以承載分界;放行 accepted must-fix——掏空 --accept 的誠實儀式
**Open**: meta 要不要記錄 --accept 蓋章或殘留 SUGGESTION 數(傾向 YAGNI 先不加,誠實性維持靠工單 git 歷史)

## Conclusion

**Decision**: 乾淨蓋章的守門 (2) 改為「末輪零未解 must-fix」——嚴重度即阻斷分界:CRITICAL/WARNING 擋章、SUGGESTION 不擋;skills 端同步收緊「可裁 ⇒ 一律記 SUGGESTION」;--accept 語意不變但「保留」收窄為 must-fix 級(含 (accepted) 標記行)。
**Rationale**: 消滅「記一筆 SUGGESTION 就注定要修或問 --accept」的反誘因——它扭曲紀錄誠實性(agent 為省麻煩不記);引擎算不出 skill 的判斷式 must-fix 分類,唯一可計算的正典分界是嚴重度標籤,所以兩邊往嚴重度收斂。
**Rejected alternatives**: 只改 skill 措辭(摩擦與反誘因原封不動);行內「可裁」新標記(格式/解析器波及大,嚴重度已足以承載);放行 accepted must-fix(掏空 --accept 的誠實儀式)。
**Deferred**: meta 是否記錄 --accept 蓋章或殘留 SUGGESTION 數——YAGNI 先不加,誠實性維持靠工單 git 歷史;若日後需要在已封存變更上區分帶保留章再開新討論。
**Capture to**: proposal(一個 change 涵蓋:兩站 spec 守門條文與 Scenario、兩份 skill spec、station.rs+測試、兩份 SKILL.md 三連動)
**Next**: /speclink-propose --from-discussion stamp-blocking-set-alignment

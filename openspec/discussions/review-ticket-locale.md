---
topic: code-review-stage 規格產出的 review.md 沒有跟著專案 locale 語系，固定變成英文
slug: review-ticket-locale
status: promoted
promoted_to: code-review-stage
created: 2026-08-01
created_by: MomoChen <momochenisme@gmail.com>
---

# Discussion: code-review-stage 規格產出的 review.md 沒有跟著專案 locale 語系，固定變成英文

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

使用者觀察到 review 站產出的 review.md 工單沒有跟隨專案 locale（openspec/config.yaml 設 locale: tw、spec_locale: tw），固定變成英文。採 Assumptions 模式：codebase 偵察命中 crates/speclink-core/src/review.rs（工單骨架與文法）、crates/speclink-core/src/instructions.rs（locale/spec_locale 解析與注入）、crates/speclink-core/assets/skills/review.md（正典 skill 模板）、openspec/config.yaml，脈絡充足。相關 change：code-review-stage（in-progress 19/20，持有 specs/review-skill delta）、verify-station-parity。相關規格：workflow-config（locale 四層解析）、verify.md 資產的 locale 綁定句為現成前例。本題僅動 skill 文案與規格條文，不引入新架構縫，介面深度檢查跳過。

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-08-01)

**Focus**: review.md 工單為何固定英文——實際跑壞、還是契約缺口？缺口在哪、修法落在哪？
**Position**: 是契約缺口而非個案跑壞——skill 只把 locale 綁在「呈現給使用者」的內容，寫進工單的 findings 與 sub-agent brief 完全沒有語言約束；四項假設經使用者一次全數確認：
- 本 repo 的 code-review-stage/review.md 三輪 findings 其實是中文，但那是 .claude/CLAUDE.md 全域 zh-tw 規則的意外副作用；下游專案（經 speclink update 收到 skill）設 locale: tw 也拿不到中文工單
- 缺口有兩處：step 5 sub-agent brief 的回報契約（"under 400 words, `- [SEVERITY] path — description`"）無語言要求 → 預設吐英文；step 7 add-round 對記錄語言隻字未提；step 6 同時要求 "render verbatim" 與 "write in the resolved locale"，自相矛盾（assets/skills/review.md:25,83）
- verify.md:39,168 有現成綁定句可仿：報告散文走 locale，severity 標籤、指令行、code 參照留英文，locale 未設定則英文
- 骨架維持英文不動引擎：`# Review —`、`## Round N`、`**Scope**:`、`[CRITICAL]`、`Standards:`/`Correctness:` 是 verb-owned 文法（review.rs:101,106，parse_round 逐行驗證），且 LANGUAGE.md 明文結構標記不在詞彙範圍
- 修法落點：specs/review-skill delta 的「審查流程的技能行為」補 locale 條文與 scenario、正典 skill 模板補綁定句（三處技能同步）、golden 再生——全收進 in-flight 的 code-review-stage（19/20，delta 還在它手上），走 link + ingest
**Ruled out**: 本地化工單骨架與嚴重度標籤——需動 parser、golden 與既有工單相容性，且違反 LANGUAGE.md 結構標記慣例；只綁 add-round 不綁 brief——主線每輪手工翻譯 sub-agent 英文輸出，翻譯漂移進永久記錄；另開小 change——同一份 spec delta 被兩個 change 分持，封存順序成新協調問題
**Open**: 無——動 change 前的平行 session 確認屬執行期衛生，記入結論 Deferred

## Conclusion

**Decision**: review skill 把 locale 綁進整條產出鏈——step 5 兩軸 sub-agent brief 攜帶解析後的 locale（finding 描述以該語言撰寫）、step 6 呈現與 step 7 add-round 寫入工單的 findings 同語言（消除 verbatim 與 locale 的矛盾：sub-agent 產出即為 locale 語言，主線不翻譯）；工單骨架、severity 標籤、`Standards:`/`Correctness:` 前綴、檔案路徑維持英文；locale 未設定則全英文。落地方式：specs/review-skill delta 補 locale 條文與 scenario、正典 skill 模板（assets/skills/review.md）補綁定句並三處技能同步、golden 再生，全數收進 in-flight 的 code-review-stage。不動引擎。
**Rationale**: 本 repo 工單是中文屬全域 zh-tw 規則的意外正確，契約層（spec delta 與 skill 模板）對記錄語言零約束——測不到的邊界＝沒有契約；仿 verify.md:39,168 既有綁定句可讓兩個品質站的 locale 行為對稱，成本最低。
**Rejected alternatives**: 本地化骨架與嚴重度標籤（動 parser＋golden＋既有工單相容，違反 LANGUAGE.md 結構標記慣例）；只綁 add-round 不綁 brief（主線逐輪手工翻譯，漂移進永久記錄）；另開小 change（同一份 spec delta 兩 change 分持，封存順序成協調問題）。
**Deferred**: 動 code-review-stage 前確認無平行 session 在處理同一 change（執行期衛生，非設計問題）。
**Capture to**: 既有 change code-review-stage 的 specs/review-skill delta 與 tasks.md（經 ingest）
**Next**: speclink discuss link review-ticket-locale code-review-stage → /speclink-ingest code-review-stage

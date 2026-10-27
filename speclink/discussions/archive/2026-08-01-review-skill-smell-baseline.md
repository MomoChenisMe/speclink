---
topic: /speclink-review Standards 軸完整內嵌 Matt Pocock 12 smells 檢查表的照抄範圍
slug: review-skill-smell-baseline
status: promoted
promoted_to: code-review-stage
created: 2026-08-01
created_by: MomoChen <momochenisme@gmail.com>
---

# Discussion: /speclink-review Standards 軸完整內嵌 Matt Pocock 12 smells 檢查表的照抄範圍

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

apply 進行中（code-review-stage，0/18）之際的需求收斂：使用者要求 /speclink-review 技能的 Standards 軸完整內嵌 Matt Pocock code-review skill 的 12 種 code smells 檢查表（專有名詞英文原文不動），而非 artifacts 現行一句帶過的「Fowler smells 基線」。模式：文件輸入（原 skill 全文與使用者列舉的 12 smells 為待分揀主張）。掃描對象：change artifacts（design D7、spec review-skill、task 5.1）與 crates/speclink-core/src/skills.rs（正典模板落點）。原 repo MIT 授權，可照抄附出處。
Source doc: https://github.com/mattpocock/skills/blob/main/skills/engineering/code-review/SKILL.md

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-08-01)

**Focus**: 「完整照抄」的範圍怎麼劃——逐項 smells 檢查表，還是連流程結構一起抄
**Position**: 逐項照抄 12 smells 檢查表（Mysterious Name、Duplicated Code 等專有名詞英文原文不動），內嵌 skills.rs 正典模板的 Standards sub-agent 指示；流程結構不抄。使用者已確認此劃界。
- 缺口證實：design D7／spec review-skill／task 5.1 對 smells 僅一句「Fowler smells 基線」，無逐項內容——照現行 artifacts 實作，模板作者屆時得憑印象自編
- 原版的兩軸平行、read-only sub-agent、並列呈現不合併不重排、400 上限、repo 文件優先等概念 artifacts 已涵蓋，無需改
- MIT 授權；生成 SKILL.md 本為英文，照抄零損耗；模板加一行出處註記
**Ruled out**: 照抄原 skill 的 Spec 軸與 git fixed-point 定界——前身討論 code-review-stage 已定案：Spec 軸讓給既有 verify、初審範圍用 touched 檔集（git 基準僅備援），抄回會與既有機制打架（proposal Non-Goals、design D7）
**Open**: (1) smell baseline 的兩條約束規則原文（The repo overrides／Always a judgement call）是否保留 (2) 出處標記 (Refactoring, ch.3) 是否保留、其意涵 (3) 「never a hard violation」與工單 CRITICAL／WARNING／SUGGESTION 三級制怎麼相容

## Conclusion

**Decision**: skills.rs 正典模板的 Standards sub-agent 指示內嵌 Matt Pocock code-review skill 的 smell baseline 全段英文原文——含引言（"On top of whatever the repo documents…"）、兩條約束規則（The repo overrides／Always a judgement call，含 skip anything tooling already enforces）、"(Refactoring, ch.3)" 出處、12 條 smells 逐項（what it is → how to fix，Mysterious Name 等專有名詞不動）；模板加一行出處註記（MIT）。實作時以原 repo raw 檔逐字為準，不用轉述版。
**Rationale**: 缺了逐項清單與約束契約，模板作者只能憑印象自編「smells 基線」；照抄正典原文一次釘死，且與既有工單三級制（smells 以 "possible X" 措辭落 WARNING／SUGGESTION，CRITICAL 留給文件化標準明確違反與 Correctness bug）、repo 文件優先原則零衝突。
**Rejected alternatives**: 照抄整個 skill 流程（Spec 軸、git fixed-point 定界）——與前身討論定案的 verify 分工、touched 檔集定界打架；翻譯成中文——生成 SKILL.md 本為英文，翻譯反而失真。
**Deferred**: 無
**Capture to**: 既有 change code-review-stage（spec review-skill＋design D7＋task 5.1 措辭）
**Next**: /speclink-ingest code-review-stage

---
topic: implementation-refactor-roadmap 在既有 platform-architecture 交付順序中的安插方式
slug: refactor-roadmap-sequencing
status: concluded
created: 2026-07-10
created_by: MomoChen <momochenisme@gmail.com>
---

# Discussion: implementation-refactor-roadmap 在既有 platform-architecture 交付順序中的安插方式

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

本討論由兩份架構文件都包含交付順序、但層級容易被讀成兩套 roadmap 所觸發。採 assumptions mode，因為已檢視目標架構、implementation companion、進行中的 engine-typed-core artifacts，以及 Store、CLI、Node 與 Desktop 的現行入口。相關現行變更為 engine-typed-core；本輪只釐清文件與執行順序，不修改實作。

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-07-10)

**Focus**: implementation-refactor-roadmap 應如何放進 platform-architecture 已定義的交付順序
**Position**: implementation roadmap 應作為平台架構 Phase 1 的執行展開，而不是另一套並列或新增的產品 Phase。
- platform-architecture 保留 Phase 1 至 Phase 4、架構不變式與每階段 outcome，仍是唯一目標架構正典。
- implementation-refactor-roadmap 在 Phase 1 內依賴排序 change slices：engine typed runtime、metadata fail-closed、TeamStore、Host/binding/policy、stable task/evidence、drift split、protocol/client/context。
- G0 放在 Phase 1 之前但只作交付基線 gate；它不改產品架構，也不應編成 Phase 0。
- Phase 2 至 Phase 4 直接沿用平台架構邊界，roadmap 只補 change 名、依賴、可平行項目與驗收 gate。
- 目前 roadmap 的方向正確；應改善的是標示層級，避免 Phase 1A 至 1E 的五個群組與七個 change 列表看似一對一但實際不一致。
**Ruled out**: 把整份 implementation roadmap 插在 Phase 1 之後，因為 TeamStore、Host、evidence 與 protocol 本來就是 Phase 1；另開平行 roadmap 也被排除，因為會形成第二套順序正典。
**Open**: 無；若要實際改文件，可另開只處理 roadmap 呈現與交叉引用的文件變更。

## Conclusion

**Decision**: platform-architecture 保留 Phase 1–4 的產品交付主幹；implementation-refactor-roadmap 只展開 Phase 1 的 change 依賴，並以前置 G0 交付 gate 起始，Phase 2–4 則沿用主幹並補實作刀組與驗收。
**Rationale**: 架構順序與實作切片是不同粒度；採巢狀關係可保留唯一架構正典，同時讓現況缺口有可執行的遷移順序。
**Rejected alternatives**: 將 roadmap 放在 Phase 1 之後會錯置本屬 Phase 1 的正確性工作；維護兩條平行 roadmap 會造成排序與 Phase 邊界漂移；把 G0 稱為 Phase 0 會混淆交付護欄與產品能力。
**Deferred**: 是否實際重寫兩份文件的章節與圖表，留待使用者確認。
**Capture to**: docs/implementation-refactor-roadmap.zh-TW.md 的執行層級與 docs/platform-architecture.zh-TW.md §14 的交叉引用。
**Next**: 若要落地文件調整，建立一個只改文件結構與對應表的 change。

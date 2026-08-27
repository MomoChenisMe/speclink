---
topic: worktree apply 收尾交棒句漏列 /speclink-quality 入口
slug: worktree-handoff-quality-mention
status: promoted
promoted_to: worktree-handoff-quality-mention
created: 2026-08-27
created_by: MomoChen <momochenisme@gmail.com>
---

# Discussion: worktree apply 收尾交棒句漏列 /speclink-quality 入口

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

使用者實跑 worktree apply（fix-discuss-section-anchor）後發現收尾提示只列 /speclink-review ∥ /speclink-verify，沒有 /speclink-quality。該提示是 crates/speclink-core/assets/skills/apply-worktree-post.md W3 段的逐字模板；同檔 agent 端 Next steps 有列 quality，但不會唸給使用者。需求明確（可驗證的字面缺口），未經磨題階段。相關規格：skill-routing（apply scenario 明寫「品質站（review、verify 或 quality）」、apply-with-worktree 只有 Example row）；相關變更：已封存的 2026-08-27-propose-apply-handoff-updates，其 Non-Goals 明寫不動此資產字面，只檢查了「可略過品質站」而未檢查入口清單完整性。

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-08-27)

**Focus**: W3 交棒句漏列 /speclink-quality 是漏網還是刻意，以及修法與路由
**Position**: 五條假設全數獲使用者確認——這是字面漏網，修法為資產字面＋正典 scenario 雙補：
- 漏網成因：propose-apply-handoff-updates 的 Non-Goals 宣告不動 apply-worktree-post.md，只檢查「可略過品質站」路徑，未檢查入口清單完整性；同 change 卻把 apply.md 完工模板改成三入口，兩模板因此分岔
- quality 在 worktree 內可用：quality 資產出邊已有 worktree 分支（quality.md:81）；正典傘詞「品質站」明含 quality（skill-routing apply scenario）
- 修法單點：改 asset 檔 W3 句補 /speclink-quality（兩站合跑）；該句全 repo 四處，其餘三份（.claude／.agents SKILL.md、golden snapshot）皆衍生物，走 ASSET_VERSION／golden／assets.lock 三連動後 speclink update 再生
- 防回歸：skill-routing spec 補 apply-with-worktree 的交棒 scenario，字面要求 W3 明列三入口——這次漏網正因只有 Example row、無 scenario 級要求
- 路由：新開小 change（原 change 已封存不能 ingest，進行中三變更主題無關）
**Ruled out**: 刻意排除 quality 的解讀——quality 無任何 worktree 限制，找不到排除理由
**Open**: 無——全部節點已定，進入結論

## Conclusion

**Decision**: 補 apply-worktree-post.md 的 W3 逐字交棒句——在 /speclink-review ∥ /speclink-verify 之後補列 /speclink-quality（兩站合跑）入口；同時在 skill-routing spec 為 apply-with-worktree 新增交棒 scenario，字面要求 W3 明列三個品質站入口。asset 異動走 ASSET_VERSION／golden／assets.lock 三連動，speclink update 再生 .claude 與 .agents 的 SKILL.md 及 golden snapshot。
**Rationale**: 使用者可見的逐字模板與三處既有字面分岔——agent 端 Next steps、apply.md 完工模板、正典傘詞「品質站（review、verify 或 quality）」都含 quality，唯獨使用者實際看到的 W3 句沒有；quality 在 worktree 內可用（quality.md 出邊已有 worktree 分支），沒有排除理由。
**Rejected alternatives**: 只改字面不釘正典——這次漏網正因 apply-with-worktree 只有 Example row、無 scenario 級字面要求；ingest 進既有 change——原 change 已封存，進行中三變更主題無關。
**Deferred**: none
**Capture to**: proposal
**Next**: /speclink-propose --from-discussion worktree-handoff-quality-mention

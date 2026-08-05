---
topic: worktree-toggle-and-guards 新增 worktree UI/UX 項目後，semantic-color-system 是否也要補上 worktree 的設計考量
slug: worktree-color-semantics
status: promoted
promoted_to: semantic-color-system
created: 2026-08-05
created_by: MomoChen <momochenisme@gmail.com>
---

# Discussion: worktree-toggle-and-guards 新增 worktree UI/UX 項目後，semantic-color-system 是否也要補上 worktree 的設計考量

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

worktree-toggle-and-guards（已完成 17/17）落地了三個 worktree UI 落點：卡片 worktree 標示、抽屜分支＋路徑、設定頁開關。使用者問 semantic-color-system（進行中 0/10 未開工）是否也要補上 worktree 的設計考量。

模式：assumptions——偵察找到 ChangeCard.tsx、RichDetailDrawer.tsx、ProjectSettingsView.tsx 三個落點與兩個 change 的完整 artifacts，證據足夠先列假設。

相關脈絡：三層色彩角色規則（主色=連結/互動/進度、狀態=語意色、靜態=中性）出自討論 card-drawer-header-colors，該討論全文零提 worktree——worktree UI 是色彩審計之後才落地的，屬審計時序盲點。semantic-color-system 的 design/tasks/spec 亦零提 worktree。

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-08-05)

**Focus**: worktree 的三個 UI 落點（卡片標示／抽屜分支路徑／設定頁開關）中，哪些需要色彩處置
**Position**: 只有卡片標示需要處置——色彩審計時序上早於 worktree UI 落地，該處從未被三層規則檢驗：
- 卡片 GitBranch 圖示＝text-primary/60（ChangeCard.tsx:95），沿用鄰居「來源討論」圖示的慣例，但未經裁定
- 抽屜分支＋路徑繼承 meta 列容器的 text-muted-foreground（RichDetailDrawer.tsx:355），已符合「靜態一律中性」
- 設定頁開關用既有表單元件，無自訂色
- card-drawer-header-colors 討論與 semantic-color-system 的 design/tasks/spec 全部零提 worktree——確為審計盲點
- D2 守門只掃原生色階字面（sky-600 等），primary 是 token 不在掃描範圍；不主動補，此處永遠不會被機制抓到
**Ruled out**: 抽屜與設定頁納入調整範圍——兩處已合規，動它們是沒事找事
**Open**: 卡片標示的用色方向（中性化／保留主色記例外／給語意色）；SEMANTIC_TONE 是否加第五鍵；守門是否擴掃 primary；落地路徑（ingest 進 semantic-color-system vs 開新 change）

### Round 2 — assumptions (2026-08-05)

**Focus**: 卡片 worktree 標示的用色方向——中性化還是搶眼標示
**Position**: 使用者裁定要搶眼——標示的任務是掃視層一眼看出「這張卡目前由 worktree 在做」，方向轉為語意色，色相下輪定案：
- 中性化（前輪假設 2）被否決：worktree 是進行中的工作位置訊號，不是靜態 metadata
- 此裁定可不破壞三層規則：把「工作正於副本進行」讀成狀態，worktree 標示即歸「狀態=語意色」層，規則表不需新增角色
- 色相候選分析：sky（借用 inProgress 語意，與使用者「目前在做」的心智一致，守門經 tone.ts 引用天然合規）為推薦；orange（git 品牌色心智強，但 14px 下與同列 restale 的 amber 警示難辨）；fuchsia／indigo（與同列品質站章的 rose／violet 相鄰）
**Ruled out**: 中性化 text-muted-foreground——使用者否決，一眼辨識的功能需求優先；orange——與 amber 同列碰撞；fuchsia／indigo——與章色相鄰
**Open**: 色相定案（sky 待確認）；抽屜是否跟進上色或維持中性；SEMANTIC_TONE 是否加鍵；守門擴掃與落地路徑（ingest）

## Conclusion

**Decision**: 卡片 worktree 標示（ChangeCard.tsx:95 GitBranch 圖示）由 text-primary/60 改為引用 SEMANTIC_TONE.inProgress（sky-600／dark:sky-400），歸入三層規則「狀態＝語意色」層——worktree 掛著＝工作正於副本進行中。抽屜分支＋路徑維持 meta 列中性，設定頁零改動，SEMANTIC_TONE 不加鍵（直接引用 inProgress），D2 守門不擴掃 primary。
**Rationale**: 標示的功能是掃視層一眼看出「這張卡目前由 worktree 在做」——搶眼需求把它定性為狀態訊號而非靜態 metadata，恰好落在既有規則的語意色層，規則表零新增角色；sky 與同列鄰居（teal 來源討論、amber restale、violet／rose 章）在 14px 圖示下辨識清楚，經 tone.ts 引用天然通過守門。
**Rejected alternatives**: 中性化 text-muted-foreground（使用者否決——一眼辨識的功能優先）；orange git 品牌色（與同列 amber 警示僅半個色相之差，14px 難辨且誤讀為警示）；fuchsia／indigo（與品質站章 rose／violet 相鄰碰撞）；維持 primary/60（與來源討論圖示同色即不搶眼的病因，且違「主色＝連結／互動／進度」）；SEMANTIC_TONE 加 worktree 專屬鍵（單點使用不建抽象）。
**Deferred**: sky 的「進行中」語意在 worktree 已完工待 merge 時稍有誇大——接受，使用者心智模型為「未收尾即未完」；守門不掃 primary token 的殘留風險——主色角色越界仍靠審計抓，屬已接受的取捨。
**Capture to**: semantic-color-system 既有變更（design D3 處置清單＋desktop spec＋tasks）
**Next**: /speclink-ingest semantic-color-system

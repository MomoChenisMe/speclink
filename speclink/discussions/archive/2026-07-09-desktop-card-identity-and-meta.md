---
topic: 卡片/抽屜的身分與 meta（discuss 卡以檔名為題、change 卡 author 頭像＋tooltip、討論蓋 createdBy）
slug: desktop-card-identity-and-meta
status: promoted
promoted_to: desktop-card-identity
created: 2026-07-09
---

# Discussion: 卡片/抽屜的身分與 meta（discuss 卡以檔名為題、change 卡 author 頭像＋tooltip、討論蓋 createdBy）

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

桌面看板第三批 UI 差異中屬「卡片/抽屜身分與 metadata」的三項：(1) discuss 卡看不到也無法複製英文檔名；(2) change 卡缺抽屜有的 meta；(6) discuss 抽屜缺 change 那樣的 meta（誰發起）。

模式：assumptions（掃到 DiscussionColumn.tsx、ChangeCard.tsx、RichDetailDrawer.tsx、DiscussionDrawer.tsx、adapter.ts、newcmd.rs、util.rs）。

程式碼盤點：
- discuss 卡（DiscussionColumn DiscussionCard）顯示 topic＋狀態徽章＋輪數，無 slug、無複製；change 卡（ChangeCard）以 `change.name`（kebab）為題＋複製鈕——兩者身分呈現不對稱。
- 詞彙原則「工程詞 kebab-case/slug 不出現於使用者可見文案」，但有 config.yaml 頁簽的明文例外先例。
- change 卡刻意極簡（ChangeCard 註解）；relationship 指示（fromDiscussions、restale）用原生 `title`；author 頭像只在抽屜（RichDetailDrawer:236）。shadcn Tooltip 元件已存在（ui/tooltip.tsx）。
- 討論無 createdBy/started：DiscussionItem（adapter.ts:52）只有 slug/topic/status/rounds/created/promotedTo；discuss 檔 frontmatter 未蓋作者。change 的 createdBy 來自 `newcmd.rs:29 util::git_identity()`。started_at 是 change 實作階段概念，討論不適用。

介面深度：discuss createdBy 為跨層（core 蓋章→adapter 曝露→UI 顯示），但照抄既有 newcmd.rs git_identity pattern，非新 seam；其餘為純 UI。

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-07-09)

**Focus**: discuss 卡與 change 卡的身分呈現（slug 可見性、複製、卡片 meta）
**Position**: 兩卡身分對齊、卡片仍克制——
- discuss 卡改以檔名(slug)為標題、topic 降為卡身描述＋加複製 slug 鈕（比照 change 卡以 kebab 名為題＋複製）。
- 違反「slug 不出現於使用者文案」，但 change 卡本就顯示 kebab、且有 config.yaml 例外先例 → 記為受控例外（vocabulary drift）。
- change 卡維持極簡，只加 author 頭像（createdBy）；relationship 指示（來自討論、同源）由原生 title 改用 shadcn Tooltip hover 呈現（元件已存在）。不把抽屜整組 meta 搬上卡。
**Ruled out**: change 卡搬整組抽屜 meta（破壞卡片極簡、看板變雜）
**Open**: 討論的「誰發起」怎麼來（議題 6）

### Round 2 — assumptions (2026-07-09)

**Focus**: 討論要不要記「誰發起」以及怎麼記
**Position**: 記——引擎替討論蓋 createdBy：
- 使用者要：本機小團隊＋remote 都需知道討論由誰建立。
- 是 createdBy（誰發起），**不是** started_at（討論無開工階段，其生命週期 open→concluded→promoted 已由抽屜階梯呈現）。
- 實作照抄 change 的 `newcmd.rs:29 util::git_identity()`：`discuss new` 蓋 created_by，加進 DiscussionItem，抽屜（與卡片 author 頭像）顯示。
**Ruled out**: 給討論加 started_at（類別錯置，討論不「開工」）
**Open**: none

## Conclusion

**Decision**: (1) discuss 卡改以檔名(slug)為標題、topic 降為卡身次要描述、加複製 slug 鈕（比照 change 卡）。(2) change 卡加 author 頭像（createdBy）；「來自討論」「同源」指示由原生 title 改用 shadcn Tooltip；卡片其餘維持極簡（不搬抽屜整組 meta）。(6) 引擎替討論蓋 createdBy——`discuss new` 照 `newcmd.rs:29 util::git_identity()` 蓋 created_by，加進 DiscussionItem，抽屜/卡片顯示「誰發起」；不加 started_at。
**Rationale**: discuss 卡與 change 卡身分呈現對齊（皆以 kebab 檔名為題）＋slug 是 CLI 動詞的把手，值得可見可複製；卡片維持極簡以保看板可掃視；討論作者章服務本機小團隊與 remote 的「誰發起」，照抄既有 change 蓋章機制、低風險。
**Rejected alternatives**: 維持 discuss 卡以 topic 為題、slug 完全隱藏（與 change 卡不對稱、CLI 把手取不到）；change 卡搬整組抽屜 meta（破壞極簡）；給討論加 started_at（討論無開工概念，類別錯置）
**Deferred**: slug 當標題屬「slug 不出現於使用者文案」的受控例外，需於 openspec/LANGUAGE.md 記 vocabulary drift；author 頭像/tooltip 版式細節留 propose
**Capture to**: proposal（新變更）＋ openspec/LANGUAGE.md（vocabulary drift：discuss 卡 slug 為受控例外）
**Next**: /speclink-propose --from-discussion desktop-card-identity-and-meta

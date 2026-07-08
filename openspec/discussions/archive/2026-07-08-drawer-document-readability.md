---
topic: 討論與變更抽屜的文件版面結構
slug: drawer-document-readability
status: promoted
promoted_to: drawer-document-readability
created: 2026-07-08
---

# Discussion: 討論與變更抽屜的文件版面結構

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

使用者以三張截圖對照提出：speclink 討論抽屜「討論過程」分頁的輪呈現是文字牆，變更抽屜的提案／設計／任務／規格分頁同樣閱讀不易，希望對齊 Spectra 的文件呈現（區塊有邊界、欄位有標籤、留白有節奏）。前刀 desktop-reading-experience（字體打包、prose 16px、skipHtml、remark-breaks）已完成——字內排版已好，殘餘的是文件級版面結構。

模式：assumptions——相關程式碼充足（packages/ui 的 Markdown.tsx、DiscussionDrawer.tsx、RichDetailDrawer.tsx、TaskList.tsx；apps/desktop 的 index.css）。

相關變更：desktop-reading-experience（done，本題的前刀）、desktop-task-interactions（in-progress，任務分頁互動）、desktop-specs-view（in-progress，左導覽規格頁——與抽屜內規格分頁不同物）。Spectra 無本機原始碼，對齊以截圖為準。

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-07-08)

**Focus**: 文字牆殘餘成因的切分與 Spectra 式結構化的範圍
**Position**: 五項假設使用者全數確認：
- 成因拆兩層——內容側（add-round 的 Position 慣性寫成單行 500+ 字，remark-breaks 分得開行、分不開行內）＋呈現側（整個 `## Rounds` 丟單一 Markdown，輪與輪之間只有 h3 margin、無視覺分界，DiscussionDrawer.tsx:246）。
- 討論過程分頁改每輪一張卡片：切 CLI scaffold 固定格式 `### Round N — <mode> (<date>)`（仿 splitDiscussionSections 前例），卡頭輪次徽章＋mode chip＋日期，卡身把 **Focus**／**Position**／**Ruled out**／**Open** 拆成標籤欄位塊；非標準格式整篇 prose 退回。
- 變更抽屜四分頁共同修法＝文件容器與行寬上限（現況 `prose max-w-none` 貼邊全寬，Markdown.tsx:22，全螢幕 96vw 行長失控）；規格分頁另需去機器標記。
- 任務分頁不入此刀——TaskList 已結構化且 desktop-task-interactions 在途。
- add-round 模板小改為輔：Position 鼓勵列點多行；舊記錄不回改，靠 GUI 結構化改善。
- 規格分頁深度使用者裁定走 A 案：`## ADDED/MODIFIED Requirements` 等轉色標區段標題即止，requirement 原文照排。
**Ruled out**: 規格分頁 requirement 逐條卡片＋scenario 展開（Spectra 全套）——範圍膨脹，色標區段已除噪音主因；純 CSS 間距微調——做不出區塊邊界。
**Open**: 拆刀（一刀全包 vs GUI／skill 模板分刀）；轉出變更名稱。

## Conclusion

**Decision**: 一刀處理（drawer-document-readability），四項工作：
1. 討論過程分頁結構化——每輪一張卡片：切 scaffold `### Round N — <mode> (<date>)`，卡頭輪次徽章＋mode chip＋日期，卡身 Focus／Position／Ruled out／Open 標籤欄位塊；非標準格式整篇 prose 退回（與現行 sections fallback 同型）。
2. 抽屜 markdown 文件容器——行寬上限＋留白，取代 `prose max-w-none` 全寬貼邊；變更抽屜四分頁與討論抽屜共用。
3. 變更抽屜規格分頁去機器標記（A 案）——`## ADDED/MODIFIED/REMOVED/RENAMED Requirements` 轉色標區段標題，requirement 原文照排。
4. discuss skill add-round 模板小改——Position 鼓勵列點多行；三處同步（crates/speclink-core/assets、repo 技能實例、render golden 於乾淨樹再生）；舊記錄不回改。
**Rationale**: 前刀已把字排好，殘餘癥結全在文件級結構——區塊無邊界、欄位無標籤、機器標記直出、行寬無上限。GUI 結構化新舊記錄通吃，模板改動讓新記錄在卡片內也好讀，雙管齊下才根治。一刀不拆：三項 GUI 工作皆純前端小改，模板一項是文案級改動，分刀無獨立交付價值。
**Rejected alternatives**: 規格分頁 requirement 逐條卡片（範圍膨脹，色標區段已除噪音主因）；純 CSS 間距微調（做不出區塊邊界）；只改模板不改 GUI（舊記錄永遠是牆）；任務分頁入刀（已結構化且 desktop-task-interactions 在途）。
**Deferred**: 任務分頁若另有閱讀面問題（如群組標題視覺），具體化後併入 desktop-task-interactions；規格分頁 requirement 卡片化留待需要再加。
**Capture to**: proposal
**Next**: /speclink-propose --from-discussion drawer-document-readability

---
topic: 規格頁與已封存頁清單分頁與最新在前排序＋已封存頁雙節版面改善＋抽屜全螢幕閱讀寬度
slug: specs-archive-pagination
status: promoted
promoted_to: specs-archive-pagination
created: 2026-07-11
created_by: MomoChen <momochenisme@gmail.com>
---

# Discussion: 規格頁與已封存頁清單分頁與最新在前排序＋已封存頁雙節版面改善＋抽屜全螢幕閱讀寬度

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

desktop-ux-polish（已封存 2026-07-11）與 spec-archive-drawer（在途 0/22）之後的接續需求：規格頁與已封存頁的清單要能分頁瀏覽、以最新時間在前排序；已封存頁「變更／討論」兩節上下堆疊導致找封存討論要長捲動；追加第 4 題——抽屜全螢幕後內文仍維持 72ch 行寬靠左、右側大片留白。

模式：假設模式——偵察命中 SpecList.tsx、ArchivedList.tsx、apps/desktop/core 的 cache.rs／query.rs、speclink-core 的 discuss.rs／listing.rs，脈絡充足。

相關變更／討論：spec-archive-drawer（同檔案在途，本題排其後）、討論 spec-archive-drawer-ux（已轉出，本題為其後續）、drawer-document-readability（已封存，全螢幕行寬行為的決策出處）、board-card-anatomy（在途，同為 UI 佇列）。

現況要點：封存變更 40 筆（dated_name 升冪＝最舊在前）、封存討論 26 筆（檔名路徑升冪＝最舊在前）、規格 13 筆（capability 字母序）；清單資料已全量進記憶體，懶載入僅及文件內容。

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-07-11)

**Focus**: 界定四題範圍、確立現況根因，並對做法提出五項假設
**Position**: 四題皆屬前端呈現層、無新架構縫（深度檢查跳過），以獨立新變更排在 spec-archive-drawer 之後：
- 長捲動痛點的根因是排序：封存變更按 dated_name 升冪（cache.rs:97）、封存討論按檔名路徑升冪（discuss.rs:238）皆最舊在前；規格為字母序（listing.rs:128）
- 清單資料已全量進記憶體（store.ts loadAll 一次抓齊），分頁為純前端呈現，每頁約 20 筆、搜尋變更即跳回第 1 頁、單頁時隱藏控制列
- 排序改在前端桌面層，不動引擎——引擎排序直接反映在 CLI 輸出（回歸保護對象）
- 已封存頁雙節改為頁內子頁籤「變更／討論」＋筆數徽章，搜尋框共用、同時過濾兩節，各頁籤獨立分頁
- 排序鍵：封存變更＝封存日期新→舊；封存討論＝建立日期新→舊；規格＝mtime 新→舊（缺席排最後、字母序決勝）
- 第 4 題（全螢幕行寬）根因非 bug：drawer-document-readability design.md 明文「內容靠左、行寬不隨抽屜增長」（Markdown.tsx:24 的 max-w-[72ch]）；問題在靠左＋96vw 右側大片死區看起來像破版
**Ruled out**: 後端分頁 IPC——量級（40+26+13 筆）不需要、資料已在記憶體；引擎層改排序——攪動 CLI 輸出順序破壞回歸基線；已封存頁合併單一時間軸——變更卡與討論卡解剖不同、心智模型混亂；左右雙欄——窄視窗爆版
**Open**: 五項假設待使用者確認；全螢幕行寬的解法取捨（置中 vs 放寬行寬）；規格頁是否保留字母序；每頁筆數定值；「分頁」一詞與抽屜 tabs 撞名的詞彙處置

### Round 2 — assumptions (2026-07-11)

**Focus**: 使用者對五項假設與全螢幕解法的確認
**Position**: 五項假設全數成立，全螢幕採建議方案（維持 72ch 行寬、欄位置中）：
- 獨立新變更、排在 spec-archive-drawer 之後
- 排序在前端桌面層，引擎與 CLI 輸出不動
- 已封存頁雙節改子頁籤「變更／討論」＋筆數徽章、共用搜尋框
- 純前端分頁、每頁 20 筆、搜尋變更跳回第 1 頁、單頁隱藏控制列
- 排序鍵：封存變更＝封存日期新→舊、封存討論＝建立日期新→舊、規格＝mtime 新→舊（缺席排最後、字母序決勝）
- 全螢幕閱讀版面：行寬上限維持、改置中——對齊頁面清單 max-w-3xl mx-auto 既有慣例；新變更 design 明文推翻 drawer-document-readability 的「內容靠左」半句
**Ruled out**: 全螢幕文字撐滿——96vw 下行長逾 100 全形字，換行找行首困難，可讀性客觀變差
**Open**: 「分頁」（tabs）與 pagination 撞名的詞彙處置——結論一併裁定

## Conclusion

**Decision**: 以獨立新變更（排在 spec-archive-drawer 落地之後）完成四項清單與閱讀 UX 改善：
- 規格頁與已封存頁清單加純前端分頁——每頁 20 筆、單頁時隱藏控制列、搜尋字串變更即跳回第 1 頁
- 三清單改最新在前，排序在前端桌面層、引擎不動：封存變更＝封存日期新→舊、封存討論＝建立日期新→舊、規格＝mtime 新→舊（mtime 缺席排最後、字母序決勝）
- 已封存頁「變更／討論」雙節堆疊改為頁內子頁籤＋筆數徽章；搜尋框共用、同時過濾兩節，各頁籤獨立分頁
- 抽屜全螢幕閱讀版面：維持 72ch 行寬上限、欄位改置中（對齊頁面清單 max-w-3xl mx-auto 慣例）；新變更 design 明文推翻 drawer-document-readability 的「內容靠左」半句，行寬上限決策不變
**Rationale**: 長捲動痛點的根因是「最舊在前排序＋雙節堆疊」而非缺分頁本身，三刀（排序、子頁籤、分頁）合治；清單資料已全量在記憶體，分頁純前端最省；行寬上限是正確的可讀性決策，錯的只有靠左造成的單側死區。
**Rejected alternatives**: 後端分頁 IPC（量級 40+26+13 筆不需要、資料已在記憶體）；引擎層改排序（CLI 輸出為回歸保護對象）；已封存頁合併單一時間軸（變更卡與討論卡解剖不同、心智模型混亂）；左右雙欄（窄視窗爆版）；全螢幕文字撐滿（行長逾 100 全形字不可讀）。
**Deferred**: none
**Capture to**: proposal（轉為變更）＋ openspec/LANGUAGE.md（詞彙裁定：「分頁」保留給 tabs 語意；pagination 在 artifacts 散文中稱「換頁」，UI 文案僅用「上一頁／下一頁／第 N 頁」不另造名詞）
**Next**: /speclink-propose --from-discussion specs-archive-pagination

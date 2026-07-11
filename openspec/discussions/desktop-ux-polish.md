---
topic: Speclink desktop UI 微調（分析面板、驗證呈現、已轉出入口、slug 識別、搜尋強化、拖曳封存落點）
slug: desktop-ux-polish
status: promoted
promoted_to: desktop-ux-polish
created: 2026-07-11
created_by: MomoChen <momochenisme@gmail.com>
---

# Discussion: Speclink desktop UI 微調（分析面板、驗證呈現、已轉出入口、slug 識別、搜尋強化、拖曳封存落點）

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

使用者在開工 engine-typed-core 前，提出七點 desktop UI 微調：(1) 分析面板呈現看不懂、盼參考 Spectra 的摘要卡＋發現卡結構；(2) 分析結果開了關不掉；(3) 驗證通過只剩一行字近乎無資訊；(4) 已轉出討論的 ↗N 切換鈕不明顯；(5) 討論卡標題盼用 slug 且可複製（配合 --from-discussion）；(6) 搜尋功能與 UI 強化；(7) 拖曳時封存落點插入欄列壓縮全欄寬度。

模式：assumptions——偵察命中大量相關原始碼（packages/ui/src/components/AnalyzePanel.tsx、RichDetailDrawer.tsx、DiscussionColumn.tsx、KanbanBoard.tsx、search.ts、boardDnd.ts；apps/desktop/src/store.ts）。

相關既有變更：desktop-card-identity（2026-07-09，已封存）已讓討論中卡片用 slug 標題＋複製鈕並立 LANGUAGE.md 明文例外——本次第 5 點實為例外範圍太窄（promoted 細列與討論抽屜標題仍用 topic）；promoted-discussion-toggle（2026-07-09，已封存）是第 4 點互斥檢視設計的出處；desktop-verb-drawer-surface（2026-07-09，已封存）是第 1-3 點動詞面板的出處。

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-07-11)

**Focus**: 七點 UI 痛點的現況定位，與其中四項的方向裁定
**Position**: 三點方向明確直接定案、四點經選項裁定，全數收斂：
- (1) 分析面板改「維度摘要卡＋發現卡」兩層：引擎 AnalyzeReport 本就回傳 location、recommendation 與維度 status，現行 AnalyzePanel.tsx 全數丟棄只渲染 severity＋summary 平鋪清單——看不懂的根源；維度名改繁中（覆蓋度／一致性／模糊度／缺漏）
- (2) 關不掉＝缺 toggle：store.ts 的 drawerVerb 只在換 change／關抽屜時清空；修法為分析鈕 toggle＋面板右上 ×＋store 增 clear 動作
- (3) 驗證併入分析面板（使用者裁定）：移除獨立驗證鈕，「分析」一併跑 validate，面板頂部呈「結構驗證：通過／N 個錯誤」一列
- (4) 已轉出入口改欄底常駐收合區（使用者裁定）：取消互斥檢視，欄底「已轉出 N ▸」可展開衍生樹細列，與討論中同屏可見
- (5) slug 識別實為例外範圍太窄：討論中卡片已是 slug 標題＋複製鈕（desktop-card-identity），缺的是 promoted 細列（PromotedRow 刻意 topic 錨點）與討論抽屜標題（DiscussionDrawer topic 直出）兩處——擴到這兩處並同步更新 LANGUAGE.md 例外條目
- (6) 搜尋全面強化（使用者全選＋加碼）：搜尋列 UI（icon、清除鈕、快捷鍵、命中數）＋全文搜尋 artifacts＋命中高亮＋模糊比對＋篩選功能；全文搜尋需新引擎查詢／IPC，觸發介面深度檢查
- (7) 封存落點改浮層（使用者裁定）：絕對定位浮於看板右緣、不佔 flex 空間、欄寬零變動；且僅拖「變更卡」時浮現——現行 {dragging && <ArchiveDropZone />} 連拖討論卡也插入 140px flex 兄弟導致全欄 reflow，而討論卡根本不可封存拖放
**Ruled out**: 驗證「保留鈕＋輕量回饋」與「完全移除」（輸給併入分析——單一入口看全貌，且保住封存前自查價值）；已轉出「header 分段切換」與「維持互斥放大按鈕」（輸給欄底收合區——互斥檢視本身就是藏起另一面的根因）；封存落點「側欄已封存為落點」與「常駐保留空間」（輸給浮層——前者拖曳距離遠、後者常駐吃 140px）
**Open**: 篩選功能的具體維度（篩什麼？）；全文搜尋的 seam 切法（前端逐檔載 vs 新 IPC 一次查詢）；範圍是否拆刀（純前端六點 vs 含 IPC 的搜尋強化）

### Round 2 — assumptions (2026-07-11)

**Focus**: 篩選維度、全文搜尋的 seam 切法、與範圍拆刀
**Position**: 一刀全包，搜尋走新 IPC seam：
- 範圍：七點併入單一變更 desktop-ux-polish（使用者裁定）——單次 propose／verify／archive 循環的流程成本，優於拆刀省下的元件重複改動；接受任務數可能超過 15 條的 scope check 警告
- 篩選維度：建立者（createdBy）、建立時間（近 7 天／近 30 天／更早）、來源討論（fromDiscussion 鏈）——皆為現有卡片欄位，純前端
- 全文搜尋 seam（介面深度檢查）：speclink-desktop-core 新增 workspace 全文查詢、Tauri command 曝為單一 search_workspace(query)、前端 dataSource adapter 加一方法回傳命中卡片＋snippet；單 adapter 無疊層；隱藏「遍歷 openspec 樹、讀 artifacts、比對、彙整 snippet」整段行為；刪之則全文能力消失——非 pass-through
- fuzzy 限卡片名稱／slug 層，全文比對維持子字串——全文 fuzzy 成本高且命中難解釋
**Ruled out**: 兩刀／三刀拆法（使用者裁定一刀——單循環優先）；前端逐檔 loadDocument 掃描（N 卡 × 4 artifacts 的 IPC 風暴、遍歷邏輯抹進前端）；全文層 fuzzy（成本／雜訊）；任務進度篩選維度（使用者未選）
**Open**: （無——進入結論）

## Conclusion

**Decision**: 以單一變更 desktop-ux-polish 完成七點 desktop UI 微調：(1) 分析面板改「維度摘要卡＋發現卡」兩層——頂列四張繁中維度卡（覆蓋度／一致性／模糊度／缺漏，無問題綠／N 個問題琥珀），發現卡帶嚴重度徽章＋來源檔名（location）＋摘要＋建議行（recommendation）；(2) 分析鈕改 toggle＋面板右上 × 可關閉（store 增 clear 動作）；(3) 移除獨立驗證鈕，「分析」一併執行 validate，面板頂部呈「結構驗證：通過／N 個錯誤」列；(4) 已轉出討論入口改欄底常駐收合區「已轉出 N ▸」就地展開衍生樹，取消互斥檢視；(5) slug 標題＋複製鈕擴到 promoted 細列與討論抽屜標題（topic 降為副標），同步擴充 LANGUAGE.md 既有例外條目的適用範圍；(6) 搜尋整包強化——搜尋列 UI（icon、清除鈕、快捷鍵、命中數、空狀態）、fuzzy（限名稱／slug 層）、命中高亮＋全文命中 snippet、篩選 chips（建立者／建立時間／來源討論）、全文搜尋走 speclink-desktop-core 新查詢 IPC search_workspace(query)；(7) 封存落點改絕對定位浮層疊於看板右緣、不佔 flex 空間、僅拖變更卡時浮現。
**Rationale**: 分析面板「看不懂」的根源是引擎回傳的 location／recommendation／維度 status 被前端丟棄，補呈現即可對齊 Spectra 的可讀性而不必抄設計；範圍取捨上使用者裁定一刀全包——單次 propose／verify／archive 循環的流程成本優於拆刀省下的元件重複改動。
**Rejected alternatives**: 驗證「保留鈕＋輕量回饋」與「完全移除」（輸給併入分析——單一入口看全貌且保住封存前自查）；已轉出「header 分段切換」「維持互斥放大按鈕」（互斥檢視本身是藏起另一面的根因）；封存落點「側欄已封存為落點」（拖曳距離遠）「常駐保留空間」（常駐吃 140px）；全文搜尋「前端逐檔掃描」（IPC 風暴）；「全文層 fuzzy」（成本高、命中難解釋）；「兩刀／三刀拆法」（使用者裁定單循環優先）。
**Deferred**: 任務進度篩選維度（使用者未選）；搜尋觸發節奏（即時 vs debounce）與 snippet 呈現細節留給 design。
**Capture to**: proposal（新變更 desktop-ux-polish）；LANGUAGE.md（slug 例外條目範圍擴充——vocabulary drift：現行條文「僅限 discuss 卡標題與其複製鈕」需擴為含 promoted 細列與討論抽屜標題）
**Next**: /speclink-propose --from-discussion desktop-ux-polish

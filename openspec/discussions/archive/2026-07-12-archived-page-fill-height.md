---
topic: 已封存頁與規格頁高度填滿、分頁器固定底部
slug: archived-page-fill-height
status: promoted
promoted_to: archived-page-fill-height
created: 2026-07-12
created_by: MomoChen <momochenisme@gmail.com>
---

# Discussion: 已封存頁與規格頁高度填滿、分頁器固定底部

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

使用者附截圖指出：已封存頁必須捲到清單底部才能按到分頁器（上一頁／下一頁），希望頁面高度填滿、分頁器固定在底部常駐可見。採假設模式（codebase scout 找到 ArchivedList.tsx、ListPager.tsx、App.tsx、SpecList.tsx、KanbanBoard.tsx 共五個相關檔案，脈絡充足）。相關程式碼：App.tsx:312 對封存頁用 overflow-y-auto 整頁捲動；ListPager 排在 20 張卡片後的文件流末端；KanbanBoard.tsx:127 已有「填滿高度＋內部捲動」的既有模式可沿用。無進行中的變更或討論。

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-07-12)

**Focus**: 分頁器被推出視窗外的根因，與版面策略的選擇
**Position**: 採看板既有「填滿高度＋內部捲動」模式，分頁器固定底部常駐可見：
- 根因：App.tsx:312 對封存頁用 overflow-y-auto（整頁捲動），ListPager 排在 20 張卡片之後的文件流末端，必然落在摺疊線下
- ArchivedList 改為 h-full flex 直欄：搜尋框＋子頁籤固定頂部、卡片清單 flex-1 min-h-0 overflow-y-auto、ListPager 固定底部
- App.tsx 的 main 對封存頁也改 overflow-hidden，否則內部捲動容器高度不受限、改了等於沒改
- SpecList.tsx:162 有一模一樣的結構（ListPager 排清單末），比照辦理同一次改動處理
- 換頁後「捲回頂部」由 topRef.scrollIntoView()（ArchivedList.tsx:221）改為重置內部捲動容器的 scrollTop
- 兩個子頁籤各自保留獨立分頁器與頁碼（changeRawPage/discRawPage 本就獨立），不合併成共用底欄
**Ruled out**: position: sticky 底部分頁器——diff 較小但清單短時分頁器不沉底，且頁面捲動策略與看板分歧；合併跨頁籤共用底欄——需提升頁碼狀態並依 active tab 切換，複雜度增加而無收益
**Open**: 無——五項假設全數獲使用者確認

## Conclusion

**Decision**: 已封存頁與規格頁改為「填滿高度＋內部捲動」版面：搜尋框／子頁籤固定頂部、卡片清單於內部容器捲動、ListPager 固定底部常駐可見（不捲動即可換頁）；App.tsx 的 main 對這兩頁改 overflow-hidden；換頁「捲回頂部」改為重置內部捲動容器的 scrollTop；兩個子頁籤維持各自獨立的分頁器與頁碼。
**Rationale**: 分頁器排在整頁捲動的文件流末端，每頁 20 張卡片必然把它推出視窗外；看板已有同款填滿高度模式（KanbanBoard.tsx:127 欄內捲動），沿用可讓全 app 版面策略一致。
**Rejected alternatives**: position: sticky 底部分頁器——清單短時不沉底、與看板捲動策略分歧；只改封存頁不改規格頁——SpecList.tsx:162 同結構同病，兩清單頁行為不一致日後還得補改；跨頁籤共用底欄——需提升頁碼狀態，複雜度增加而無收益。
**Deferred**: none
**Capture to**: proposal
**Next**: /speclink-propose --from-discussion archived-page-fill-height

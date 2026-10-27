## Context

看板側的呈現 anatomy 已完備：變更卡標題＋whyExcerpt 描述（packages/ui/src/components/ChangeCard.tsx 的描述列）、討論卡 slug 標題＋topic 描述（DiscussionColumn.tsx，LANGUAGE.md 受控例外）、活躍變更抽屜四層標頭（RichDetailDrawer.tsx：標題＋複製鈕／狀態列／出身列／動作列）。已封存頁（ArchivedList.tsx、ArchivedDrawer.tsx）停在單行卡片與純文字抽屜標題。桌面清單 payload 已有「CLI 同形項上疊加呈現層輔助欄位」的先例（apps/desktop/core/src/query.rs 對規格清單疊 purposeExcerpt／modifiedAt）。抽屜互斥由 store 的 open* 函式群保證（apps/desktop/src/store.ts）。

## Goals / Non-Goals

**Goals**

- 任何入口開啟變更詳情抽屜／討論抽屜時，底層頁面一律回到看板。
- 已封存變更卡與討論卡改為「標題＋描述」雙行，與看板同構。
- 已封存抽屜標頭補標題複製鈕與出身列（建立者／建立日期／封存日期）。

**Non-Goals**

- 不改看板側任何卡片與抽屜（它們是被對齊的基準，不是對象）。
- 封存抽屜不補進度條與動詞動作列——封存是唯讀定格，無進度與動詞。
- 不動 LANGUAGE.md 的裁定（slug 為討論識別錨點維持不變，本 change 是其範圍的既有適用，不是擴充）。
- 不動系統匣本身（落頁修在 store 層，系統匣行為自動繼承）。

## Decisions

### D1 落頁修在 store 的 open* 函式，不是各入口

`openDetail`／`openDiscussion`（store.ts）在設定抽屜狀態的同一個 set 內補 `boardView: "board"`。理由：全部入口（系統匣 dispatch、討論抽屜跳衍生變更、同源變更互跳、封存前「去蓋章」）都收斂到這兩個函式，單一落點涵蓋全部；改在 tray dispatch 只修一條路徑（討論中已排除）。`openSpec` 與 `openArchived`（如有）不動——規格抽屜／封存抽屜的宿主頁本來就是規格頁／已封存頁，落回看板反而錯。

### D2 封存變更卡的描述資料走清單 payload 疊加，不新開 meta 查詢

`query.rs` 的封存清單為每個封存項疊加兩個選填欄位：`whyExcerpt`（封存 proposal.md 的 Why 首句，與看板變更卡同名同義）與 `created`（封存目錄 metadata 的建立日期）。理由：比照規格清單 `purposeExcerpt` 的既有先例；抽屜出身列需要的三項（建立者 createdBy、建立日期 created、封存日期 date）中兩項已在清單項上，補 `created` 後抽屜零新查詢——討論第一輪曾提「新增封存側 meta 查詢」，結論定案後以 payload 疊加取代（結論為準），少一支查詢動詞。不可讀／缺席時欄位缺席（不插 key），前端以缺席容錯，清單照常回傳。

### D3 封存討論卡雙行為純前端改動

`DiscussionItem` 已同時帶 slug 與 topic，`ArchivedList.tsx` 的封存討論卡改為 slug 標題（等寬強調＋複製鈕，沿用既有 CopyButton）＋topic 描述列即可，零後端改動。與看板討論卡（DiscussionColumn.tsx）同構。

### D4 封存抽屜標頭的層級：標題列＋出身列，僅此兩層

標題列＝現有 SheetTitle 補複製鈕（封存變更複製 datedName、封存討論複製 slug；沿用 RichDetailDrawer 的複製鈕樣式與 useCopied 回饋）。出身列＝建立者（首字母圓標＋名字，email 收 tooltip，沿用 displayName 慣例）、建立日期、封存日期，恆定單行溢出裁切——與活躍抽屜出身列同構。既有的改進標示、審查／驗證結局標示、來源討論 chips 位置不動。

## Risks / Trade-offs

- **落頁改動影響所有入口**：從看板自身開抽屜時 `boardView` 已是 "board"，重複 set 無害（zustand 淺比較不觸發多餘重繪的保證不依賴此——set 同值仍會通知，但 React 渲染結果相同）；風險集中在「從規格頁點討論抽屜的衍生變更」這類跨頁跳轉，行為改為落回看板——這正是需求本身。
- **Why 首句抽取對手寫 proposal 的容錯**：封存的 proposal.md 格式不一（三種模板、手寫變體），抽取規則沿用看板 whyExcerpt 的同一實作路徑；抽不到就缺席，卡片退回單行——與看板變更卡 null 缺席行為一致。

## Migration Plan

無資料遷移。純呈現層與清單 payload 選填欄位新增，向後相容（缺席欄位＝舊行為）。

## Open Questions

（無）

## Context

變更詳情抽屜(packages/ui/src/components/RichDetailDrawer.tsx)的 header 現為六層堆疊:標題列、metadata 列(建立者含 email、產生工具、相對時間、任務數、開工資訊、審查資訊)、來源討論列、進度條列、動作列、分頁列。來源討論籤(packages/ui/src/components/SourceDiscussionChip.tsx)以 topic 全句直出,長短不定導致排版兩種長相;已封存抽屜(packages/ui/src/components/ArchivedDrawer.tsx)共用同一顆籤。資料側 sourceDiscussions 已同時攜帶 slug 與 topic,引擎與 adapter 零改動。本設計承接討論 change-drawer-header-redesign 的結論,並落定其 Deferred 數值。

全案純前端呈現層:speclink-core、speclink-fs、speclink-host、CLI、server 皆零改動;speclink-desktop-core 無邏輯變更;apps/desktop 僅隨 @speclink/ui 套件更新。測試落點為 packages/ui 的 vitest+Testing Library 既有測試檔。

## Goals / Non-Goals

**Goals:**

- 變更詳情抽屜 header 收斂為四層固定結構,高度不隨資料量浮動。
- 來源討論籤 slug 直出、topic 降為提示;多筆來源討論在任何視窗寬度下長相一致。
- 溢出以「+N」數字籤+shadcn Popover 浮層承接,保住「點籤跳回討論」的路徑。
- 已封存抽屜的來源討論標記與變更詳情抽屜同構。

**Non-Goals:**

- 看板卡片、討論抽屜、規格抽屜、各分頁內文不動。
- 不動任何 Rust crate、CLI 輸出、server API、--json 欄位。
- 不引入 Popover 以外的新元件原語;不重做主題變數系統。
- 討論已排除方案不再考慮:topic 全文+更強截斷、收合區塊、固定拆兩行、流式折行、+N 原地展開、hover tooltip 列溢出清單。

## Decisions

**D1 四層結構的構成。** 標題列(變更名+複製鈕,維持現狀)/狀態列(進度條+百分比+審查章)/出身列(單行:頭像+建立者名字+產生工具+建立相對時間+開工資訊+「來自」slug 籤+「同源」籤)/動作列(維持現狀)。「N/N 任務」自 header 移除——任務分頁徽章與進度條已承載同一資訊,三處重複砍為兩處。取捨:曾考慮任務數留在狀態列,但與分頁徽章緊鄰重複,資訊零增量。

**D2 slug 籤與 topic 提示。** SourceDiscussionChip 的 props 由 topic+onClick 改為 slug+topic+onClick,籤面顯示 slug(等寬字型 font-mono,與討論抽屜標題、系統匣的 slug 直出慣例一致),topic 改以 shadcn Tooltip 呈現(與看板卡片關係指示的主題化提示同款,取代原生 title)。籤設寬度上限 max-w-[140px],超長 slug 截斷,Tooltip 內容為「slug+topic」兩行使截斷資訊可回取。

**D3 +N 切點:固定顯示 1 顆、第 2 顆起收 +N。**「來自」固定顯示清單第一顆(出身討論),其餘收進 +N 數字籤;「同源」比照(顯示第一顆,其餘收 +N)。理由:(1)與看板卡片「單一討論徽章以出身討論為代表」的既有規則對稱;(2)寬度預算——抽屜內容寬約 670px,出身列前綴(頭像+名字+工具+時間+開工)約 280px,「來自」與「同源」各以 標籤+140px 籤+28px +N 籤 計,兩組並存時合計約 668px,單行剛好成立;顯示 2 顆的方案在同源並存時必然溢出,且量測式切點會使同一變更在不同視窗寬度長相不同,違背排版一致的初衷。92% 的變更僅一筆來源討論,+N 在該情境完全缺席。

**D4 單行的硬保證。** 出身列容器採不折行 flex(whitespace-nowrap+overflow-hidden+min-w-0),固定顆數為主要防線,容器裁切為極端情境(超長名字)的兜底——任何資料組合都不得使出身列折行或撐寬抽屜。

**D5 +N 浮層。** 新增 shadcn Popover 原語(packages/ui/src/components/ui/popover.tsx,相依 @radix-ui/react-popover)。專案主題無 --popover 變數,浮層底色比照既有 select 原語用 bg-card。+N 籤為 Popover trigger(bg-muted 圓籤,aria-label 標明「其餘 N 份」語意);浮層內溢出項直列,每項兩行——slug 主行(font-mono)+topic 副行(text-muted-foreground、單行截斷),點擊項目跳至對應討論/變更抽屜並關閉浮層。浮層寬度上限 w-72,超長內容截斷。討論 Deferred 的「浮層內是否附 topic 副標」在此定案為附:浮層空間充足,補回籤面犧牲的 topic 可讀性。

**D6 審查章升狀態列。** 審查資訊(狀態詞+蓋章時間+審查者,inReview 僅狀態詞,REVIEW_TONE 四態配色)自 metadata 列移至進度條同列右側;reviewStatus 為 none 時僅進度條+百分比。內容與配色規則零變化,僅位置移動。

**D7 email 收進提示。** 建立者顯示頭像+名字(createdBy 去除尖括號 email 段;無尖括號時整串直出),完整 createdBy 以 Tooltip 呈現。開工資訊顯示「日期+開工」,開工者全名收進 Tooltip——開工者與建立者多數相同,名字重複直出是現況出身資訊過寬的主因之一。

**D8 兩抽屜共用來源討論列。**「標籤+第 1 顆籤+N 浮層」封裝為共用元件(與 SourceDiscussionChip 同檔輸出),RichDetailDrawer 與 ArchivedDrawer 皆改用之——共用消除兩處呈現分歧的可能,前案 drawer-source-chip-overflow「兩處抽屜採同一種處理」的原則延續。ArchivedDrawer 的其餘 header 結構(唯讀、無進度/審查/動作)不變。

**D9 LANGUAGE.md 範圍擴充。** openspec/LANGUAGE.md「slug 直出」明文例外的適用範圍清單,增列「變更詳情抽屜/已封存抽屜的來源討論籤與其溢出浮層」,註記本變更名與日期。i18n 標籤文字:「來自討論:」改「來自」、「同源:」改「同源」(zh),英文對應 From/Siblings——縮短前綴為單行讓位。

## Implementation Contract

**觀察行為:**

- 開啟任一變更詳情抽屜,header 依序為標題列、狀態列(進度條+百分比,審查狀態非 none 時同列呈現審查章)、出身列(單行)、動作列;header 不出現「N/N 任務」計數字樣。
- 出身列呈現:建立者頭像+名字(無 email 直出)、產生工具、建立相對時間、開工日期(有開工時)、「來自」+出身討論 slug 籤(有來源討論時)、「同源」+第一顆同源變更籤(有同源時)。
- 來源討論多於 1 筆時,第 2 筆起不直接渲染,出現「+N」籤(N=溢出數);點擊開啟浮層,列出其餘討論(slug 主行+topic 副行),點擊任一項開啟該討論抽屜;同源多於 1 筆時比照。
- 籤 hover 呈現主題化提示(slug+topic);出身列在任何資料組合下維持單行,抽屜無水平捲軸。
- 已封存變更抽屜的來源討論標記呈現與上述同構(slug 直出+固定顆數+N 浮層)。

**介面/資料形:**

- SourceDiscussionChip props 改為 slug+topic+onClick;新增共用的來源討論列元件(標籤+首籤+溢出浮層)供兩抽屜使用。皆為 @speclink/ui 套件內部介面,無對外契約。
- RichDetailDrawerProps、ArchivedDrawer props、adapter 型別零變化。
- 新檔 packages/ui/src/components/ui/popover.tsx(shadcn Popover);packages/ui/package.json 增列 @radix-ui/react-popover。

**失敗模式:**

- sourceDiscussions 缺席或空陣列:來源討論列整段缺席(現行為維持)。
- topic 缺失(僅 slug):籤照常渲染,提示僅含 slug,不報錯。
- 浮層開啟中按 Esc 或點擊外部:關閉浮層,不影響抽屜。

**驗收條件:**

- packages/ui/src/__tests__/richDrawer.test.tsx:籤面文字為 slug 且不含 topic 全文;多來源討論時僅首籤直接渲染、+N 籤呈現溢出數;點 +N 後浮層列出其餘討論且點擊觸發 onOpenDiscussion;header 無任務計數字樣;審查資訊列與進度條同列;email 不出現在可視文字。
- packages/ui/src/__tests__/archivedDrawer.test.tsx:封存抽屜的來源討論標記呈現 slug、多筆時 +N 浮層行為同上。
- 指令 npm test -w @speclink/ui 全綠;npm test -w apps/desktop 與 npm run build -w apps/desktop(vite 打包)通過。

**範圍邊界:** in scope=packages/ui 兩抽屜 header、SourceDiscussionChip、popover 原語、i18n 標籤、openspec/LANGUAGE.md 範圍擴充、packages/ui 既有測試改寫;out of scope=看板卡片、討論抽屜、規格抽屜、Rust crates、CLI、server、apps/desktop 自身程式碼、E2E harness。

## Risks / Trade-offs

- [既有測試斷言 topic 直出而失效] → 測試改寫屬本變更任務的一部分,先改測試錨定新契約再動元件(TDD 順序),不以放寬斷言矇混。
- [+N 浮層把 8% 情境的討論入口移到第二層,點擊成本+1] → 使用者於討論中明示裁定此取捨(恆定高度優先);浮層項附 topic 副行降低辨識成本。
- [font-mono 的 slug 在 140px 內截斷過多] → Tooltip 保 slug 全文+topic;浮層內 slug 不受 140px 限制。
- [@radix-ui/react-popover 新相依] → 與既有 radix 家族(dialog/select/tooltip)同源同版本線,鎖定於 packages/ui/package.json;無全域影響。
- [回歸對照] → CLI 人眼輸出與 --json 零變化,golden 與 CLI 測試不受影響;跨平台面(Windows/macOS/Linux)為純 web 呈現,無平台分支。

## Migration Plan

無資料遷移。一次性 UI 改版,舊位置資訊(任務數、email、topic 全文)皆有新承載處(分頁徽章/Tooltip/Tooltip+浮層),無使用者設定需搬移。

## Open Questions

(無——討論 Deferred 的顆數、籤寬、浮層排版已於 D3/D5 定案。)

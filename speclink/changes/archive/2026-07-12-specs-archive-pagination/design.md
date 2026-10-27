## Context

規格頁（SpecList）與已封存頁（ArchivedList）目前的清單順序全是最舊在前或字母序：封存變更清單由桌面 core 衍生快取按封存目錄名升冪回傳、封存討論按檔名路徑升冪、規格按名稱字母序。已封存頁「變更／討論」兩節上下堆疊，封存量（40 變更＋26 討論）持續成長，查找最近封存的討論需要長捲動。抽屜閱讀面依 drawer-document-readability 的決策鎖行寬上限且「內容靠左」，全螢幕（96vw）時右側形成大片死區。清單資料已由 store 一次全量載入記憶體，懶載入僅及文件內容。

本變更與在途變更 spec-archive-drawer 修改相同清單元件（SpecList、ArchivedList），依討論結論排在其落地之後動工；屆時清單卡片已是「點卡開抽屜」形態，且新增了規格抽屜（SpecDrawer）與封存抽屜（ArchivedDrawer）兩個共用閱讀面。

crate 邊界：本變更純屬 packages/ui 前端呈現層——speclink-core、speclink-cli、apps/desktop/core（Rust）與 apps/desktop 前端殼皆不動；無序列化、無設定欄位、無 git 互動。

## Goals / Non-Goals

**Goals:**

- 三清單最新在前：封存變更＝封存日期新→舊、封存討論＝建立日期新→舊、規格＝最後修改時間新→舊。
- 規格頁與已封存頁清單以每頁 20 筆換頁瀏覽，搜尋與換頁互動語意明確。
- 已封存頁雙節改頁內子頁籤，消除跨節長捲動。
- 抽屜閱讀欄置中：行寬上限不動，容器寬於上限時留白均分兩側。
- LANGUAGE.md 收錄「換頁」詞彙裁定。

**Non-Goals:**

- 後端換頁 IPC、清單 payload 欄位增減、引擎層排序改動（CLI 輸出為回歸保護對象）。
- 已封存頁合併時間軸、左右雙欄；取消行寬上限或逾 100 全形字的行長（上限「值」的調整不在此列——2026-07-12 使用者裁定 72ch→96ch，見 D4 修訂）。
- 看板的排序與換頁（board_rank 拖曳排序另有語意）；spec-archive-drawer 在途任務調整。

## Decisions

### D1：排序落在 packages/ui 清單元件內（呈現層）

各清單元件以 useMemo 對傳入的清單 props 排序後再過濾與換頁。排序鍵與決勝規則：

| 清單 | 主鍵 | 決勝 |
| ---- | ---- | ---- |
| 封存變更 | datedName 字串降冪（YYYY-MM-DD 前綴使字典序＝時間序） | 同日由字串降冪自然涵蓋 |
| 封存討論 | created 降冪 | 同日 slug 字母升冪 |
| 規格 | modifiedAt 降冪；不可得者一律排最後 | 名稱字母升冪 |

替代方案：桌面 core（Rust）排序——payload 雖屬 app 自有，但把呈現偏好下沉到資料層、且為純視覺需求動 Rust 與衍生快取，違反最小侵入；store.ts 排序——排序無跨元件共享需求，離換頁狀態（元件內）遠。呈現層排序 vitest（jsdom）可直接測。

### D2：自建 ListPager 受控元件，不採 shadcn Pagination

新增 packages/ui/src/components/ListPager.tsx：上一頁鈕＋「第 N／M 頁」字樣＋下一頁鈕，以既有 Button 原語組成；props 為 page／pageCount／onPage 的純受控形態，pageCount ≤ 1 時不渲染任何內容。每頁筆數為元件外的共用常數 20（PAGE_SIZE），頁碼狀態由各清單元件持有；頁碼一律以 min(page, pageCount) 鉗制派生，清單縮短（過濾、外部刷新）不會停在越界頁。

替代方案：shadcn Pagination——link/href 導向設計適合 URL 分頁的網站，桌面 app 為狀態式換頁，且頁碼列表在頁數多時還需另做省略邏輯；load-more／無限捲動——與「最新在前後找舊資料」的跳頁需求不合，越捲越長回到長捲動老路；虛擬捲動——引入依賴與複雜度，數十到數百筆量級不成比例。

### D3：已封存頁子頁籤以既有 Tabs 原語實作

搜尋框維持頁面頂部，其下為 Tabs：「變更」「討論」兩個 TabsTrigger（標籤沿用既有 i18n 字串），各帶過濾後命中數徽章——在任一子頁籤即可見另一節有幾筆命中。兩 TabsContent 各含自己的清單與 ListPager，頁碼互相獨立；搜尋字串變更時兩側頁碼皆回第 1 頁。預設落在「變更」子頁籤；子頁籤選擇存元件內 state，無跨視圖保留需求。archivedDiscussions 缺席（向後相容路徑）時不渲染子頁籤列，僅顯示變更清單（排序與換頁照常）。

替代方案：維持雙節堆疊＋節錨點連結——治標，清單本身仍無界成長；合併單一時間軸——已於討論否決（卡片解剖不同）。

### D4：閱讀欄置中落在共用置中容器，Markdown 行寬 class 與容器同值

packages/ui 匯出共用閱讀欄容器（置中 wrapper：寬度撐滿、max-width 與 Markdown 行寬上限同值、水平 margin auto），套在各抽屜分頁內容的捲動容器內側，包住整個分頁內容——SectionedDoc 的區段標籤、討論輪卡片、任務清單與內文同欄對齊置中。套用對象：RichDetailDrawer、DiscussionDrawer，以及 spec-archive-drawer 落地後的 SpecDrawer 與 ArchivedDrawer（實作時已存在，列入任務）。

**2026-07-12 修訂（使用者裁定）**：行寬上限值由 72ch 放寬為 96ch（≈48 全形字 @16px）——置中落地後實機檢視全螢幕留白仍嫌多。Markdown 元件自身的行寬 class 與共用容器同步改為 96ch（雙層同值），既有 72ch 測試斷言一併更新為 96ch；中文舒適行長帶（35–45 字）略為超出但仍遠低於 100 全形字硬上限，屬使用者明示的取捨。

本決策明文推翻 drawer-document-readability 的「內容靠左、容器保留一致的側向留白」半句：行寬上限維持該決策不動，僅將「靠左」改為「置中、留白均分兩側」，對齊頁面清單既有的定寬置中慣例。

替代方案：只在 Markdown 容器加置中——SectionedDoc 標籤、輪卡片等非 Markdown 元素仍貼左，欄位錯位；各抽屜各自硬編置中樣式——新抽屜易漏套用，共用容器一處定義。

### D5：「換頁」詞彙收錄 LANGUAGE.md

「分頁」在專案文案已被 tabs 語意佔用（抽屜的提案／設計／任務／規格分頁），pagination 若同用「分頁」則同詞兩義。裁定：artifacts 散文稱「換頁」；UI 文案不出現「換頁／分頁」名詞，僅用「上一頁」「下一頁」「第 N／M 頁」。i18n 新增 pager 前綴鍵（zh 與 en 同步）。

替代方案：pagination 也叫「分頁」——同詞兩義，規格與討論記錄會歧義；自造「頁碼列」——控制列本身無需命名，名詞越少越好。

## Implementation Contract

**可觀察行為：**

- 已封存頁載入後，「變更」子頁籤第 1 頁第一張卡是封存日期最新的變更；切到「討論」子頁籤，第一張卡是建立日期最新的封存討論，無需捲過變更清單。
- 規格頁第一張卡是最後修改時間最新的規格；modifiedAt 不可得的規格集中在清單尾端、彼此按名稱字母序。
- 清單（過濾後）超過 20 筆時清單底部出現換頁控制列（上一頁鈕、「第 N／M 頁」、下一頁鈕）；第 1 頁時上一頁鈕 disabled、末頁時下一頁鈕 disabled；20 筆以內無控制列。換頁後清單捲回頂部（清單頂 scrollIntoView）。
- 於任一頁輸入或修改搜尋字串，清單回到第 1 頁；已封存頁兩子頁籤的徽章即時顯示各自命中數。
- 任一抽屜（變更詳情、討論、規格、封存）在容器寬於行寬上限時（含全螢幕），內容欄水平置中、留白均分兩側；行寬上限 96ch（≈48 全形字 @16px；2026-07-12 由 72ch 放寬）。

**介面／資料形狀：**

- ListPager props：page（1 起算）、pageCount、onPage(next)；pageCount ≤ 1 渲染 null。PAGE_SIZE 常數 = 20，由 packages/ui 匯出供清單元件與測試共用。
- 清單 payload（SpecItem／ArchivedItem／DiscussionItem）欄位不增不減；無新 IPC、無 store 介面變更。
- i18n 新鍵：pager.prev、pager.next、pager.page（含 {n}／{m} 佔位），zh 與 en 兩份同步。

**失敗模式：**

- 排序鍵缺席（規格無 modifiedAt）不擲錯、不顯示錯誤——排最後、名稱決勝。
- 空清單與搜尋無結果沿用既有空狀態文案，換頁控制列缺席。

**驗收基準：**

- vitest（npm test -w packages/ui）：listPager.test.tsx（受控行為、單頁不渲染、disabled 邊界）；specList.test.tsx（mtime 排序含缺席案例、換頁、搜尋回第 1 頁）；archivedList.test.tsx（子頁籤與徽章、兩清單排序、獨立換頁、討論缺席相容）；richDrawer.test.tsx 與 discussionDrawer.test.tsx（置中容器 class 存在）。
- 真實視窗驗收（apply 尾聲）：全螢幕抽屜置中截圖、已封存頁子頁籤切換與換頁操作截圖。

**範圍邊界：** in scope＝packages/ui 清單與抽屜呈現、i18n、LANGUAGE.md；out of scope＝一切 Rust 與 apps/desktop 前端殼、清單 payload、CLI 輸出、看板。

## Risks / Trade-offs

- [本變更以 spec-archive-drawer 落地後的元件形態為基底，其在途實作若調整卡片／抽屜結構] → 開工前執行 drift 檢查；delta 規格的 MODIFIED 以其 delta 文字為基底並於 BEFORE 註記。
- [jsdom 測不出置中與換頁的視覺效果] → vitest 釘結構（class、渲染有無、順序），視覺以 release 真實視窗截圖驗收（機器備忘的 GUI 驗證程序）。
- [datedName 前綴非 YYYY-MM-DD 時字典序≠時間序] → 封存目錄名由引擎統一產生（archive 動詞），格式固定；測試以混合日期案例釘住排序。
- [規格頁改最新在前後，慣用字母序找規格的使用者受影響] → 既有搜尋列承接查找需求（討論已確認取捨）。

## Migration Plan

無資料遷移。實作順序：等 spec-archive-drawer 封存 → 對本變更跑 drift 確認 delta 假設 → apply。回滾即還原 packages/ui 檔案，無持久狀態。

## Open Questions

（無）

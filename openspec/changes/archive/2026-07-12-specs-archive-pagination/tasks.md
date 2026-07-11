## 1. ListPager 共用換頁元件（design D2：自建 ListPager 受控元件，不採 shadcn Pagination）

- [x] 1.1 撰寫失敗測試 packages/ui/src/__tests__/listPager.test.tsx（規格「清單最新在前與換頁瀏覽」的換頁控制列行為）：pageCount ≤ 1 時渲染 null；第 1 頁上一頁鈕 disabled、末頁下一頁鈕 disabled；點上一頁／下一頁以正確頁碼呼叫 onPage；頁數文案顯示「第 N／M 頁」（zh 訊息）。驗證：npm test -w packages/ui 紅燈（模組不存在）。
- [x] 1.2 實作 packages/ui/src/components/ListPager.tsx：受控元件 props { page, pageCount, onPage }，以既有 Button 原語組成上一頁鈕＋頁數字樣＋下一頁鈕；同檔匯出 PAGE_SIZE 常數 = 20；packages/ui/src/i18n.tsx 新增 pager.prev／pager.next／pager.page（含 {n}、{m} 佔位），zh 與 en 兩份同步。驗證：1.1 測試綠燈。

## 2. 規格頁排序與換頁（design D1：排序落在 packages/ui 清單元件內（呈現層）；規格「清單最新在前與換頁瀏覽」）

- [x] 2.1 撰寫失敗測試 packages/ui/src/__tests__/specList.test.tsx 新增「清單最新在前與換頁瀏覽」案例：規格依 modifiedAt 新→舊排序；modifiedAt 缺席者排最後且彼此依名稱字母升冪（案例：beta 今天、alpha 3 天前、zeta 與 delta 無 modifiedAt → 順序 beta、alpha、delta、zeta）；21 筆時第 1 頁僅 20 筆且出現換頁控制列，點下一頁顯示第 21 筆；13 筆時無換頁控制列；於第 2 頁修改搜尋字串後回到第 1 頁。驗證：npm test -w packages/ui 紅燈。
- [x] 2.2 實作 packages/ui/src/components/SpecList.tsx：useMemo 排序（modifiedAt 降冪、缺席殿後、名稱升冪決勝）→ 過濾 → 依 PAGE_SIZE 切頁；頁碼 state 以 min(page, pageCount) 鉗制派生；搜尋字串變更重設第 1 頁；換頁時清單頂 scrollIntoView；清單底部掛 ListPager。驗證：2.1 測試綠燈，既有 specList 測試不回歸。

## 3. 已封存頁子頁籤、排序與換頁（design D3：已封存頁子頁籤以既有 Tabs 原語實作；規格「已封存頁含討論節」）

- [x] 3.1 撰寫失敗測試 packages/ui/src/__tests__/archivedList.test.tsx 新增「已封存頁含討論節」與「清單最新在前與換頁瀏覽」案例：頁面呈現「變更」「討論」兩個子頁籤且預設顯示「變更」；子頁籤標籤帶過濾後筆數徽章（搜尋僅命中 3 筆討論、0 筆變更時，「變更」徽章 0＋無結果空狀態、「討論」徽章 3）；封存變更依 datedName 字典序降冪（2026-07-11、2026-07-08、2026-07-04 案例）；封存討論依 created 降冪、同日以 slug 升冪決勝；兩子頁籤頁碼互相獨立（變更翻到第 2 頁不影響討論頁碼）；搜尋字串變更後兩側皆回第 1 頁；archivedDiscussions 未提供時子頁籤列缺席、僅顯示變更清單。驗證：npm test -w packages/ui 紅燈。
- [x] 3.2 實作 packages/ui/src/components/ArchivedList.tsx：搜尋框下改 Tabs 原語呈現兩子頁籤（標籤沿用既有 archived.changesHeading／discussionsHeading 字串＋徽章）；兩節清單各自 useMemo 排序、依 PAGE_SIZE 切頁、各掛 ListPager；換頁捲回清單頂。驗證：3.1 測試綠燈，既有 archivedList 測試不回歸。

## 4. 抽屜閱讀欄置中（design D4：閱讀欄置中落在共用置中容器，Markdown 行寬 class 與容器同值；規格「markdown 文件內容行寬有上限」。註：4.x 完成當時上限值仍為 72ch，96ch 調整見第 6 組）

- [x] 4.1 撰寫失敗測試（規格「markdown 文件內容行寬有上限」的置中行為）：packages/ui/src/__tests__/richDrawer.test.tsx 與 discussionDrawer.test.tsx 斷言抽屜分頁內容的捲動容器內存在共用置中容器（行寬上限＋水平置中 class）且包住分頁全部內容（SectionedDoc 區段標籤與內文同欄）；packages/ui/src/__tests__/components.test.tsx 既有 Markdown 行寬 class 斷言維持通過。驗證：npm test -w packages/ui 紅燈。
- [x] 4.2 實作：packages/ui/src/components/Markdown.tsx 匯出共用置中容器 class 常數（寬度撐滿、max-width 與既有行寬上限同值、水平 margin auto），packages/ui/src/components/RichDetailDrawer.tsx 與 DiscussionDrawer.tsx 的分頁內容捲動容器內側套用，包住各分頁全部內容（含區段標籤、輪卡片、任務清單）；Markdown 容器自身行寬 class 不動。驗證：4.1 測試綠燈。
- [x] 4.3 對 spec-archive-drawer 落地後新增的規格抽屜與封存抽屜（packages/ui/src/components/SpecDrawer.tsx、ArchivedDrawer.tsx——開工時已存在，實際檔名以 drift 檢查現況為準）套用同款置中容器，先在其測試檔補置中容器斷言（紅燈）再套用（綠燈）。驗證：npm test -w packages/ui 全綠。

## 5. 詞彙與收尾驗證

- [x] 5.1 依 design D5：「換頁」詞彙收錄 LANGUAGE.md——於 openspec/LANGUAGE.md 收錄「換頁」條目：definition＝清單分批瀏覽（pagination），UI 文案僅用「上一頁／下一頁／第 N／M 頁」不出現名詞；avoid＝分頁（pagination 語意上）；why＝「分頁」已被抽屜 tabs 語意佔用，同詞兩義會歧義。驗證：條目格式與既有詞彙一致。
- [x] 5.2 全量驗證：npm test -w packages/ui 全綠；npm run build -w apps/desktop 成功（vite 建置無型別錯誤）。
- [x] 5.3 真實視窗驗收（操作前確認使用者未在使用螢幕）：啟動桌面 app 截圖確認——規格頁最新修改在前；已封存頁子頁籤切換直達討論清單、搜尋時兩徽章顯示各自命中數、換頁控制列翻頁並捲回頂部；變更抽屜全螢幕時內容欄置中、留白均分兩側。驗證：截圖逐項核對本清單行為。

## 6. 行寬上限放寬為 96ch（design D4 2026-07-12 修訂；規格「markdown 文件內容行寬有上限」的上限值調整）

- [x] 6.1 更新測試斷言（先紅）：packages/ui/src/__tests__/components.test.tsx 的 Markdown 行寬 class 斷言由 max-w-[72ch] 改 max-w-[96ch]；richDrawer.test.tsx、discussionDrawer.test.tsx、specDrawer.test.tsx、archivedDrawer.test.tsx 的置中容器斷言同步改 max-w-[96ch]。驗證：npm test -w packages/ui 紅燈（實作仍為 72ch）。
- [x] 6.2 實作：packages/ui/src/components/Markdown.tsx 的 Markdown prose 行寬 class 與 READING_COLUMN_CLS 由 72ch 同步改 96ch（雙層同值），註解的全形字換算一併更新（≈48 全形字 @16px）。驗證：6.1 測試綠燈；npm test -w packages/ui 全綠；npm run build -w apps/desktop 成功。
- [x] 6.3 真實視窗驗收（操作前確認使用者未在使用螢幕）：重建 release binary 後啟動桌面 app，變更抽屜全螢幕截圖確認——內容欄仍置中、每行約 48 全形字、兩側留白較 72ch 時明顯縮減。驗證：截圖核對。

## Context

第一刀 server-web-shell-alignment 封存後，server 網頁後台（apps/server-web）的殼、頁標題與登入這一族已是 desktop 的樣子，頁標題以下仍是舊零件：

- 三個列表頁（使用者、專案與儲存庫、憑證）用 server-web 自寫的 `DataList`（≥1024px 表格、更窄改卡片）與 `ListToolbar`（帶可見標籤的搜尋框與下拉，`h-9`）；沒有換頁。專案與儲存庫頁是一疊手刻卡片。
- 稽核紀錄頁同樣是 `ListToolbar` 加 `DataList`，底部只有「第 n / m 頁」與上一頁／下一頁，看不到總筆數、不能改每頁筆數；篩選與分頁由伺服器計算，回應只有 `entries` 與 `totalPages`。
- 使用者頁的待接受邀請是頁底另一塊框（`PendingInvitations`）。
- 總覽、系統、帳號三頁是手刻的區塊框（各自一個 `Section` 元件、`h2` 小標），帳號頁的清單是表格，撤銷鈕恆常顯示。

desktop 對應的樣子：規格頁與已封存頁用 packages/ui 的 `SearchField`（頁標題區右側）、`ListCard`（填滿主區、卡內捲動、卡底換頁列）、`ListRow`、`ListPager`／`usePaging`；已封存頁以 `TabsList variant="card"` 當列表卡頂端分頁；設定頁用 768px 置中欄加 `SettingsCard`／`SettingsRow`；閱讀卡頂端有 `DocTiles` 數字磚；狀態用 `StatusPill`。server-web 一個都沒用。

本變更落在三處：

- `apps/server-web`：七個頁面、i18n、測試；刪 `DataList`、`ListToolbar`。
- `packages/ui`：`ListRow`、`ListPager`、`SearchField`、`SettingsCard`、`SettingsRow`、`DocTiles`、`PageHeader`、`TabsTrigger` 各補一個向後相容的能力，入口補匯出 `ListPager`、`usePaging`；`ArchivedList` 改用 `TabsTrigger` 的新計數屬性（畫面不變）。
- `crates/host/speclink-server`：稽核 browser view model 多回一個 `total` 欄位。

engine crates、speclink-host、CLI、Protocol 都不動。稽核欄位只存在於 server 的 browser API（`/api/speclink/v1/web/admin/audit`），CLI 與 desktop 都不讀它，所以沒有 local／remote 雙路徑。視覺基準是討論 server-web-desktop-alignment 確認的原型：A 列總覽與使用者列表、B 列帳號頁、C 列 375px 窄螢幕；原型沒畫的專案與儲存庫、憑證、系統、稽核紀錄四頁依 D3–D9 的規則套用。

## Goals / Non-Goals

**Goals:**

- 列表頁與 desktop 規格頁同一種版面與零件：頁標題區右側搜尋、列表卡、共用列、共用換頁列。
- 設定型頁與 desktop 設定頁同一種版面：768px 置中欄、設定型卡與列。
- 「邀請中」與憑證的兩類改成列表卡頂端分頁，與 desktop 已封存頁同一種畫法。
- 稽核紀錄與其他列表用同一條換頁列，看得到總筆數、能改每頁筆數。
- 撤銷鈕（存取金鑰、裝置登入）滑過才出現。
- 375px 仍可完成全部操作、沒有整頁水平捲動。
- 共用元件只補能力、不另寫一份；desktop 呼叫端外觀與行為不變，唯一例外是 hover 才顯現的複製鈕改用共用顯隱 class 後，鍵盤聚焦時也看得到、觸控裝置上恆常顯示（見 D1 的 `REVEAL_ON_GROUP_HOVER`）。

**Non-Goals:**

- 右滑抽屜改詳情頁、建立類動作改對話框、DetailNav 搬家、刪 `DetailSheet`、一次性祕密值移進對話框：第三刀 server-web-detail-pages。本刀內點使用者列與專案列照舊開抽屜，邀請連結與金鑰明文照舊以頁面橫幅呈現。
- 「管理列表以抽屜承載建立與編輯」需求的抽屜條款：第三刀整條改寫。本刀只改它的列內容條款與情境。
- 使用者詳情頁、專案詳情頁的版面：第三刀。
- 每頁筆數跨重新整理保留：desktop 的記憶走 app 本機儲存，server-web 沒有對應需求，每次進頁都是預設 20 筆。
- 使用者、專案、憑證三個列表改為伺服器分頁：資料本來就整份載入，量級是一台 server 的帳號與憑證數，前端換頁足夠。
- 顏色、字型、theme token、網址、六個目的地：不動。

## Decisions

### D1 共用元件的補強（packages/ui）

全部是選配屬性，預設值等於現在的行為，desktop 呼叫端一行不改；唯一的行為改變來自共用顯隱 class：複製鈕在鍵盤聚焦時也看得到、觸控裝置上恆常顯示。

| 元件 | 新能力 | 預設 | 用在 |
| --- | --- | --- | --- |
| `ListRow` | `onClick` 改選配：不給時為靜態列——沒有 `role=button`、`tabIndex`、按鍵處理、hover 底色、手指游標與 ›；標題、描述、meta 版面不變 | 必給（現狀） | 憑證列、邀請列、窄螢幕稽核列 |
| `ListRow` | `mono?: boolean`：`false` 時標題改一般字（仍 14px semibold） | `true` | 使用者名稱、專案名稱、憑證名稱、稽核動作 |
| `ListRow` | `actions?: ReactNode`：靜態列 meta 之後的列尾槽；與 `onClick` 在型別上互斥（可點的列是 `role=button`，槽內再放按鈕就成了巢狀互動元素） | 無 | 撤銷鈕、撤回邀請 |
| `ListPager` | `compact?: boolean`：左側只留「第 a–b 筆，共 N 筆」，右側只留上一頁鈕、「目前頁 / 總頁數」與下一頁鈕（總頁數 > 1 才出現）；不渲染每頁筆數下拉、頁碼列、首末頁鈕與跳頁輸入 | `false` | server-web 窄螢幕 |
| 套件入口 | 匯出 `ListPager`、`usePaging`、`ListPagerProps` | — | server-web |
| `SearchField` | `label?: string`：輸入框的 `aria-label` | 無 | server-web 四個列表頁 |
| `SettingsCard` | `aside?: ReactNode`：小標同一列的右端槽（連結或 12px 灰字） | 無 | 總覽兩張卡的入口連結、帳號頁卡片的灰字說明 |
| `SettingsRow` | 根節點加 `group` class，讓列內按鈕能以 `group-hover:` 依列滑過顯示 | — | 帳號頁撤銷鈕 |
| `DocTiles` | `DocTile.link?: ReactElement`：呼叫端給一個不含內容的連結元素（例如 react-router 的 `<Link to=… />`），該格以它渲染，套同一組格子樣式並加 hover 底色與 2px focus ring | 無（div） | 總覽數字磚 |
| `PageHeader` | 根節點 `flex-wrap`；動作槽 `max-w-full flex-wrap`：標題與動作並排放不下時，動作槽換到標題下方一列，槽內控件再各自換行 | — | server-web 窄螢幕 |
| `TabsTrigger` | `count?: number`：標籤後的計數籤，樣式取自 `ArchivedList` 現有的計數籤 | 無 | 使用者頁、憑證頁分頁；`ArchivedList` 改用它（畫面不變） |
| `Table` | `wrapped?: boolean`：`false` 時不包外層橫向捲動容器——呼叫端自己就是捲動區時，外層的 `overflow-x-auto` 會自成捲動容器，sticky 表頭就黏在那層 | `true` | 稽核紀錄頁（D7） |
| `TableHeader` | `sticky?: boolean`：表頭固定在捲動區頂端並帶卡片底色（呼叫端不以 className 補底色） | `false` | 稽核紀錄頁（D7） |
| `Select` 模組 | 匯出 `SELECT_ALL`：「全部」選項的哨兵值（Radix Select 的 item 不接受空字串）；看板篩選改用它 | — | server-web 的 `FilterSelect`、`KanbanBoard` |
| 套件入口 | 匯出 `REVEAL_ON_GROUP_HOVER`：列滑過或鍵盤聚焦時出現、觸控裝置恆常顯示的顯隱 class；`CopyButton` 的 `REVEAL_ON_HOVER` 以它為基底（desktop 的複製鈕因此在鍵盤聚焦時也看得到） | — | server-web 的 `RevokeButton`、所有 hover 才顯現的複製鈕 |

`ListRow` 的靜態列仍把描述與 meta 截斷在一列；`actions` 只給靜態列，所以沒有「槽內點擊冒成開列」的情況要處理。

替代方案：server-web 自寫靜態列、連結版數字磚與計數籤——同一個介面元件兩份實作，違反 desktop-app「共用元件唯一來源」的精神，否決。為窄螢幕另做 `ListPagerCompact` 元件——兩個元件共用九成邏輯（範圍字串、鉗制），否決，改用屬性。`PageHeader` 以媒體查詢切版——desktop 視窗沒有最小寬度，換行本來就比溢出好，用 flex-wrap 不必分支。

### D2 列表頁版面

使用者、專案與儲存庫、憑證、稽核紀錄四頁的根節點改為 `flex h-full min-h-0 flex-col gap-4`（desktop `SpecList` 同值），由上而下：

1. `PageHeader`（`level={1}`）：動作槽依序為 `SearchField`（`label` 為「搜尋」，各頁自己的 placeholder）、該頁篩選（`Select`，觸發鈕 `h-8`、`aria-label` 為篩選名稱，「全部」項以共用的 `SELECT_ALL` 哨兵值對應空字串）、頁級動作（`data-tour="list-primary"` 照舊在主要按鈕上）。
2. 一次性祕密值橫幅（使用者頁的邀請連結；本刀不動，見 Non-Goals）。
3. 載入中與錯誤：沿用 `AdminLoading`／`AdminError`，放在列表卡位置。
4. `ListCard`：`header` 為分頁（D4、D6）或不給；`footer` 為 `ListPager`；`scrollRef` 給換頁後捲回頂部。

列表卡內容的三種狀態：

- 該類資料完全沒有：`EmptyState`（說明資料用途），不放建立鈕（建立鈕已常駐頁首；一頁只有一個主要動作）。
- 有資料但搜尋或篩選後沒有符合：`NoMatchState`（照舊）。
- 有符合：當頁的列。

`ListPager` 在 `total` 為 0 時自己不渲染，所以前兩種狀態卡底沒有換頁列。

窄螢幕（`useMediaQuery("(max-width: 1023px)")`，與殼同一個斷點）：`ListPager` 帶 `compact`；各頁的列依 D3–D7 的窄版規則；`PageHeader` 的動作槽自動換到標題下方。

替代方案：工具列放在頁標題區下方另一列——desktop 規格頁與已封存頁都放在頁標題區右側，否決；窄螢幕時由 flex-wrap 自然換到下方，與原型 C 列一致。

### D3 前端換頁

使用者、專案與儲存庫、憑證用 `usePaging(filtered, { onMove })`（不受控，預設 20 筆），`onMove` 把列表卡捲回頂部；搜尋字串、篩選或分頁切換時呼叫 `setPage(1)`。排序沿用 API 回傳的順序（本刀不改排序）。

### D4 使用者頁

- **分頁**：`Tabs` 包住 `ListCard`，`header` 為 `TabsList variant="card"` 加兩個 `TabsTrigger`：「使用者」（`count` 為使用者總數）與「邀請中」（`count` 為有效邀請總數）。計數是未經搜尋與篩選的總數。
- **分頁狀態在網址**：`useSearchParams` 的 `tab`——`pending` 為邀請中，其他值或缺席為使用者。切換分頁以 `setSearchParams(…, { replace: true })` 寫入（切回使用者時刪掉參數），不新增瀏覽紀錄；切換時頁碼回到 1、捲回頂部。
- **工具列**：搜尋框兩個分頁共用同一個字串（使用者比對顯示名稱與 email，邀請比對 email 與顯示名稱）；狀態篩選只在使用者分頁出現。兩個分頁各自一組 `usePaging`。
- **使用者列**（`ListRow`，`mono={false}`）：
  - `leading`：28px 圓形頭像字（顯示名稱第一個字；空字串退回 email 第一個字，沿用 `lib/userLabel` 的規則），有效者主色淡底主色字、停權者灰底灰字。
  - `title`：顯示名稱；`description`：email。
  - `meta`：管理員才有的 `StatusPill tone="neutral"`「管理員」、狀態 `StatusPill`（有效 success、已停權 warning）、「n 個專案」／「尚無專案」。
  - `onClick`：開細節抽屜（照舊）。
  - 窄螢幕：`description` 改為「email · 管理員」（非管理員只有 email），`meta` 只留狀態籤。
- **邀請分頁**：
  - 有邀請時，清單頂端一段灰字說明（「已寄出但尚未接受的邀請。受邀者接受後才會成為「使用者」分頁裡的一列。」），其下每筆一列。
  - 邀請列（靜態 `ListRow`）：`leading` 為灰底信封圖示頭像；`title` 為 email（等寬）；`description` 為「顯示名稱 · 角色 · 加入 a、b」（沒有專案時省略最後一段）；`meta` 為「到期 YYYY-MM-DD」；`actions` 為恆常顯示的「撤回邀請」（outline、sm，`aria-label` 沿用「撤回 <email> 的邀請」，確認框照舊）。
  - 沒有任何有效邀請：`EmptyState`「目前沒有待接受的邀請」加說明文字。
- 移除 `PendingInvitations` 區塊與 `userColumns`、列尾「檢視詳細資料」鈕（整列本身可聚焦、Enter 開抽屜）。

### D5 專案與儲存庫頁

- 搜尋比對照舊（專案名稱、代號、底下儲存庫的名稱與代號）。
- 專案列（`ListRow`，`mono={false}`）：`title` 為專案名稱，`description` 沿用「代號 <key> · n 個儲存庫」，`onClick` 開專案抽屜（照舊）。
- 完全沒有專案：`EmptyState`（照舊的標題與說明），拿掉其中的建立鈕。
- 移除列尾「檢視詳細資料」鈕。

### D6 憑證頁

- **分頁**：`ListCard` 的 `header` 為「存取金鑰」與「裝置」兩個分頁，`count` 各為該類總數；切換時頁碼回到 1。兩類完全沒有資料時照舊顯示整頁的 `EmptyState`（不出現分頁）。
- **工具列**：搜尋（存取金鑰比對前綴與名稱、裝置比對來源，照舊）、狀態篩選（有效／已撤銷，照舊）、「建立存取金鑰」（連到帳號頁，照舊）。
- **存取金鑰列**（靜態 `ListRow`，`mono={false}`）：`title` 為名稱；`description` 為「<前綴> · 使用者 <userId> · 建立 <日期> · 到期 <日期或永久>」（前綴與 userId 等寬）；`meta` 為狀態籤（有效 success、已撤銷 neutral）；`actions` 為有效者才有的 `RevokeButton`（D8）。
- **裝置列**：`title` 為來源；`description` 為「使用者 <userId> · 建立 <日期> · 最近更新 <日期>」；其餘同上。
- 窄螢幕：`description` 只留第一段（前綴或使用者），其餘不變。

### D7 稽核紀錄頁

- **工具列**：搜尋、動作篩選、來源篩選、起始日期、結束日期（兩個日期輸入之間一個灰色「–」，各自 `aria-label` 為「起始日期」「結束日期」）。日期輸入 `h-8` 與 `Select` 同圓角。
- **換頁**：頁碼與每頁筆數放在頁面狀態，呼叫 `getAdminAudit({ q, action, source, from, to, page, limit: pageSize })`。`ListPager` 以受控方式餵入：`page`、`pageCount = max(totalPages, 1)`、`total`（D9 新欄位）、`pageSize`、`onPage`、`onPageSize`。任一篩選或每頁筆數變更都把頁碼設回 1（沿用現有的重設寫法，函式改名 `refilter()`，`narrow` 留給窄螢幕旗標）。換頁與改筆數後捲回頂部。換頁或改篩選期間照舊顯示上一份回應（`useAsync` 重抓時會先清空 data），列表卡與換頁列不卸載，剛按的頁碼鈕留著、焦點不會掉回頁首；只在第一次載入時顯示 `AdminLoading`。重抓期間換頁列仍可按，可能點到新結果已不存在的頁（伺服器對越界頁碼回空頁），所以回應的總頁數比它抓取時的頁碼少時，這份空回應不顯示、也不留作「上一份」，頁碼拉回最後一頁並重抓一次（回應帶著抓取時的頁碼，頁碼拉回後、舊回應清掉前仍認得出它越界）。列表卡的捲動區與捲回頂部由 server-web 的 `useListScroll` 提供，四個列表頁共用。
- **寬螢幕**：列表卡內是共用 `Table`（`wrapped={false}`，見 D1），表頭用 `TableHeader sticky` 固定並帶卡片底色，五欄（時間、動作、對象、操作者、來源）照舊；來源改 `StatusPill tone="neutral"`。
- **窄螢幕**：每筆一列靜態 `ListRow`（`mono={false}`）：`leading` 為日期、`title` 為動作名稱、`description` 為「對象 · 操作者」、`meta` 為來源籤。
- 沒有符合事件：照舊的 `EmptyState`，工具列保留。
- 動作與來源的未知值照舊原樣呈現。

### D8 撤銷鈕滑過才出現

新增 `apps/server-web/src/components/RevokeButton.tsx`：`Button variant="outline" size="sm"`，文字「撤銷」，props 為 `label`（`aria-label`）、`disabled`、`onClick`。顯隱只用不透明度：

- 顯隱 class 取自 packages/ui 匯出的 `REVEAL_ON_GROUP_HOVER`（與 hover 才顯現的複製鈕同一個基底）。
- 平時 `opacity-0`；所在列滑過（`group-hover:`）或按鈕本身鍵盤聚焦（`focus-visible:`）時 `opacity-100`。
- 不具滑鼠移入能力的裝置（`@media (hover: none)`）恆常 `opacity-100`。
- 只改不透明度，不用 `hidden`／`display`：隱藏期間按鈕仍在 Tab 順序內、螢幕閱讀器讀得到。

用在帳號頁的存取金鑰與裝置登入列（`SettingsRow` 的 `control`）、憑證頁兩類列（`ListRow` 的 `actions`）。第三刀的使用者詳情頁憑證分頁會沿用它。「撤回邀請」不用它，恆常顯示（討論只裁定存取金鑰與裝置登入）。

替代方案：RevokeButton 自寫一份顯隱 class——packages/ui 的複製鈕已有同用途的 `REVEAL_ON_HOVER`（卡片名稱列、共用列、討論欄三處共用），另寫一份就是平行實作，否決；改為抽出共用基底 `REVEAL_ON_GROUP_HOVER`，兩者共用。元件本身留在 server-web（desktop 沒有撤銷鈕）。

### D9 稽核 view model 多回 `total`

`crates/host/speclink-server/src/admin.rs` 的 `WebAudit` 加 `total: u64`（序列化為 `total`），值為篩選後的事件總數——查詢層已算出 `page.total` 並用它算 `totalPages`，直接帶出即可。既有欄位 `entries`、`totalPages` 與所有參數不變；舊的呼叫端忽略多出的欄位。`apps/server-web/src/api/client.ts` 的 `AdminAudit` 型別加 `total: number`。

替代方案：前端只顯示「第 n / m 頁」——成為後台第三種換頁列樣式，否決。前端用 `totalPages × pageSize` 估總數——最後一頁不滿時數字錯，否決。

### D10 設定型頁版面

總覽、系統、帳號三頁的根節點改為 `mx-auto flex w-full max-w-[768px] flex-col gap-6`（desktop 設定頁同值），頁標題區在欄內。原本各頁私有的 `Section` 元件刪除，每個區塊改為 `SettingsCard`：

**總覽**

1. 歡迎區（`?welcome=1`）：`SettingsCard`「開始使用」，說明列加三列（伺服器網址、專案代號、儲存庫代號；值為等寬描述、`control` 為複製鈕）。
2. 儲存離線警示：照舊的 `role=alert` 警示框，移進欄內。
3. `DocTiles`，四格（每格 `link` 為 react-router `Link`）：
   - 有效使用者數，標籤「使用者 · 停權 n」→ `/admin/users`
   - 專案數，「專案 · 儲存庫 n」→ `/admin/registry`
   - 有效憑證數，「憑證 · 有效」→ `/admin/credentials`
   - 待啟用邀請數，「待啟用 · 邀請未使用」→ `/admin/users?tab=pending`
4. 「需要處理」卡（有待辦才渲染）：每則一列，`label` 為訊息，`control` 為 outline sm 的連結鈕；待接受邀請的連結改為 `/admin/users?tab=pending`（其他待辦照舊用伺服器給的目的地）。
5. 「系統健康」卡（`aside` 為「系統 ›」連到 `/admin/system`）：「儲存後端」列（`control` 為正常 success／異常 destructive 的狀態籤，異常時說明為可公開錯誤）、「資料結構版本」列（`control` 為等寬「v<n>」）。
6. 「最近活動」卡（`aside` 為「全部稽核 ›」連到 `/admin/audit`）：每筆一列，`label` 為「<動作名稱> <對象>」，說明為「<日期> · <操作者>」，沒有 control；沒有事件時一列灰字。

**系統**

- 「執行環境」卡：引擎版本、API 版本、識別資料結構版本、儲存驅動、契約版本、儲存等級、儲存能力，各一列，值為等寬 `control`。
- 「儲存狀態」卡：「儲存後端」狀態列（異常時說明為錯誤訊息），其下每個 outbox backlog 一列（`label` 為等寬「專案/儲存庫」，`control` 為待送出數）；沒有 backlog 時一列灰字。
- 「匯出」卡：每個 scope 一列（等寬「專案/儲存庫」，`control` 為下載鈕，照舊用原生 `<a href>`）；沒有 scope 時一列灰字。
- 「危險區」卡（圖示為紅色警示）：「資料結構遷移」一列，說明為現有警告文字（不可用時附加原因），`control` 為 outline「遷移」鈕；確認框照舊。
- 頁級動作「重看導覽」照舊。

**帳號**

順序依原型：我的專案、存取金鑰、裝置登入、登入工作階段。

- 「我的專案」（`aside` 灰字「由管理員授予」）：每個專案一列，`label` 為顯示名（缺席用代號），說明為等寬代號，`control` 為角色籤（neutral、等寬）；沒有隸屬時一列空狀態說明。
- 「存取金鑰」（`aside` 灰字「遠端工作流程以你的身分連線時使用」）：每把一列，`label` 為名稱，說明為「<前綴> · 到期 <日期或永久> · 最近使用 <日期或從未>」或「<前綴> · 撤銷於 <日期>」，`control` 為有效者的 `RevokeButton` 加狀態籤。
- 「裝置登入」（`aside` 灰字「桌面 app 或 CLI 授權後留下的憑證」）：每個一列，`label` 為來源，說明為「最近更新 <日期>」，`control` 同上。
- 「登入工作階段」：每個一列，`label` 為「建立 <日期>」，說明為「到期 <日期>」，`control` 為狀態籤。
- 各卡沒有資料時一列灰字（沿用現有的「尚無…」字串）。

替代方案：總覽沿用 4 欄格狀指標卡——與 desktop 數字磚兩種畫法，否決（討論已定 DocTiles 同款）。系統頁的 backlog 留表格——只有三欄、設定型卡的列已足夠，且與同頁其他卡一致。

### D11 i18n

zh-TW 與 en 同時增刪，兩種語言的 key 集合維持相等（i18n.test 守住）。

- 新增：分頁「使用者」（`users.tabUsers`）、專案數（`users.projectCount`「{n} 個專案」、`users.noProjectsShort`「尚無專案」）、邀請列說明（`users.pendingMeta`、`users.pendingMetaProjects`）、邀請空狀態標題（`users.pendingEmpty`）、四個列表頁的搜尋 placeholder（`users.searchPlaceholder`、`registry.searchPlaceholder`、`credentials.searchPlaceholder`、`audit.searchPlaceholder`）、憑證兩類列說明（`credentials.keyMeta`、`credentials.deviceMeta`）、帳號頁三張卡的灰字說明與列說明（`account.projectsAside`、`account.keysAside`、`account.devicesAside`、`account.keyMeta`、`account.keyRevokedMeta`、`account.deviceMeta`、`account.sessionCreated`、`account.sessionExpires`）、總覽健康卡列名（`overview.storeRow`、`overview.schemaRow`）。
- 改字：`users.pendingHint` 的「下方的使用者」改為「「使用者」分頁裡的一列」。
- 移除（因本刀不再使用）：`users.viewDetail`、`registry.viewDetail`、`users.colUser`、`users.colMemberships`、`audit.page`、`audit.prevPage`、`audit.nextPage`，以及隨 `DataList` 欄位與 `ListToolbar` 標籤一起孤兒化的 key（實作時以 grep 確認無其他使用者後才刪）。
- 換頁列文字與「頁面路徑」一樣來自 packages/ui 的共用字典（`pager.*`），server-web 不另寫。

## Implementation Contract

**行為（使用者看得到的）**

- 使用者頁：標題右側是「搜尋名稱或 email」全圓搜尋框、「全部狀態」下拉與「邀請使用者」；其下一張卡，卡頂「使用者 12｜邀請中 2」兩個分頁；每列是頭像字、名稱與 email、管理員籤、狀態籤、「n 個專案」、›；卡底「第 1–12 筆，共 12 筆」與「每頁 20 個」。點一列或 Tab 到該列按 Enter，右側抽屜照舊打開。切到「邀請中」，網址變成 `?tab=pending`，每列是 email、說明、到期日與「撤回邀請」。
- 專案與儲存庫頁：同一種卡，每列是專案名稱與「代號 x · n 個儲存庫」。
- 憑證頁：卡頂「存取金鑰 n｜裝置 n」；列不可點；滑鼠移到有效的一列，右端出現「撤銷」。
- 稽核紀錄頁：表格在卡內、表頭捲動時固定；卡底與其他列表同一條換頁列，左側看得到總筆數，可改每頁 20／50／100。
- 總覽：768px 置中；頂端四格數字磚，點「待啟用」直接看到邀請中分頁；其下需要處理、系統健康、最近活動三張卡。
- 系統、帳號：768px 置中的設定型卡；帳號頁的撤銷鈕滑過才出現。
- 375px：列表變成可換行的清單列，換頁列只剩範圍與「‹ 1 / 2 ›」；標題區的搜尋與篩選換到標題下方；沒有整頁水平捲動。

**介面與資料形狀**

- packages/ui 新屬性與匯出依 D1 表格；全部選配，預設等於現狀。
- 稽核 browser API 回應：`{ data: { entries: AuditEntry[], totalPages: number, total: number } }`，`total` 為篩選後總筆數（與 `page`、`limit` 無關，非負整數）；參數不變（`q`、`action`、`source`、`from`、`to`、`page`、`limit`）。
- 使用者頁網址：`/admin/users`（使用者分頁）、`/admin/users?tab=pending`（邀請中分頁）；其他 `tab` 值視為使用者分頁。
- server-web 新元件 `RevokeButton`（D8）；刪除 `DataList`、`ListToolbar`。
- i18n key 增刪依 D11。

**失敗模式**

- 稽核 API 失敗：照舊顯示 `AdminError` 與重試，工具列保留。
- 稽核回應缺 `total`（新 SPA 對上舊 server 的組合不會發生——SPA 內嵌於 server 一起出貨）：不另做相容分支。
- 搜尋使頁碼越界：`usePaging` 鉗制到末頁，不顯示空頁。
- `?tab=` 帶未知值：顯示使用者分頁，不報錯。
- 使用者顯示名稱為空：頭像字退回 email 第一個字（`lib/userLabel` 同一個函式）。

**驗收**

- `npm test -w apps/server-web` 全綠，含改寫後的：
  - users.test.tsx：使用者列的內容（頭像字、名稱、email、管理員籤、狀態籤、專案數）；整列以 Enter 開抽屜；兩個分頁的計數；`?tab=pending` 直達邀請中；切回使用者後網址無 `tab`；搜尋重回第 1 頁；25 位使用者時換頁列「第 1–20 筆，共 25 筆」與第 2 頁；沒有邀請時邀請中計數 0 與空狀態；撤回邀請確認框照舊。
  - registry.test.tsx：專案列與 Enter 開抽屜；空狀態沒有第二個建立鈕。
  - credentials.test.tsx：兩個分頁與計數；有效列有「撤銷 <名稱>」鈕、已撤銷列沒有；撤銷確認框照舊。
  - audit.test.tsx：呼叫參數含 `limit`；換頁列「第 a–b 筆，共 N 筆」取自回應 `total`；改每頁筆數回到第 1 頁；篩選後空狀態保留工具列。
  - overview.test.tsx：四格數字磚為連結且目的地正確（待啟用為 `/admin/users?tab=pending`）；需要處理、系統健康、最近活動三個 region 與入口照舊；無待辦時不渲染需要處理。
  - system.test.tsx、account.test.tsx：各卡為 region（名稱為小標）；帳號頁卡片順序；撤銷鈕帶 `opacity-0` 與 `group-hover:opacity-100` 的顯隱 class 且仍可由 `getByRole("button")` 找到；撤銷確認框照舊。
  - toolbar.test.tsx：稽核紀錄頁的搜尋框、兩個下拉與兩個日期輸入高度 class 相同；下拉是 combobox 而非原生 select；每個控件有無障礙名稱。
  - a11y.test.tsx：使用者頁、總覽、帳號頁無 axe 違規。
- `npm test -w packages/ui` 全綠，含：listRow.test.tsx（不給 `onClick` 時沒有 `role=button`、沒有 ›、點擊無作用；`mono={false}`；`actions` 與 `onClick` 型別互斥，靜態列的槽內按鈕照常可按）、listPager.test.tsx（`compact` 只渲染範圍、上一頁、「n / m」、下一頁）、settingsParts.test.tsx（`aside` 渲染在小標列；`SettingsRow` 根節點有 `group`）、detailShell.test.tsx（帶 `link` 的格渲染成該連結元素並含值與標籤）、pageHeader.test.tsx（根節點與動作槽可換行）、tabs.test.tsx（`count` 渲染計數籤）、archivedList.test.tsx 照舊通過；uiSingleSource 守門照舊通過。
- `npm test -w apps/desktop` 全綠（題目不改）。
- `cargo test -p speclink-server --test it admin::audit_filter` 全綠，既有各題加 `total` 斷言（篩選第 2 頁為 3、超出頁碼為 5、未知動作為 0）。
- `npm run build -w apps/server-web` 成功。
- 手動驗收：在瀏覽器以管理員走一次 1440px 與 375px 的七個頁面，對照原型 A／B／C 列；以一般成員開帳號頁。

**範圍邊界**

- 範圍內：D1–D11 列出的共用元件補強、七個頁面的清單與區塊、使用者頁分頁與網址參數、憑證頁分頁、撤銷鈕顯隱、稽核 `total` 欄位、i18n、測試、server-web-console 六條與 server-admin 一條需求。
- 範圍外：抽屜內的內容與行為（使用者細節、專案細節、邀請、建立專案、建立存取金鑰）、一次性祕密值橫幅、頁標題與殼（第一刀已定）、Tour 的步驟與文案、desktop 的任何畫面。

## Risks / Trade-offs

- [第三刀落地前，列表頁是新清單加舊抽屜] → 點列照舊開右側抽屜、建立照舊開抽屜，中間狀態可用；第三刀依序轉出。
- [撤銷鈕滑過才出現，降低可發現性] → 鍵盤聚焦時出現、觸控裝置恆常顯示、按鈕始終在 Tab 順序與無障礙樹內；有效狀態籤恆常可見，提示這一列可操作。
- [共用元件補強影響 desktop] → 全部是選配屬性、預設等於現狀，唯一例外是複製鈕改用共用顯隱 class 後鍵盤聚焦時也看得到（原本聚焦時仍透明，屬無障礙改善）；`ArchivedList` 改用計數屬性後 DOM 不變；`PageHeader` 只在放不下時換行。packages/ui 與 desktop 全套 vitest 都跑一次。
- [稽核 API 多一個欄位] → 只加不改，既有欄位與參數不變；Rust 測試補斷言。CLI 不讀這支 API，golden 與 CLI 測試不受影響。
- [前端換頁對大量使用者的效能] → 資料本來就整份載入並在前端篩選，換頁只減少渲染的列數，不增加負擔。
- [回歸對照] → 不碰 CLI 與 golden 檔；前端以 vitest 與 vite build 為準（前端 `tsc --noEmit` 基準本來就紅、不是守門）；Rust 只跑 speclink-server 的稽核測試。
- [跨平台] → 純瀏覽器畫面與一個 JSON 欄位；Windows／macOS／Linux 差異只在字型渲染。`@media (hover: none)` 在三平台的桌面瀏覽器皆為 false（有滑鼠），觸控筆電依瀏覽器回報而定，最差情況是按鈕恆常顯示。

## Migration Plan

隨 server 版本一起出貨（server 內嵌 server-web 的 build 產物，SPA 與 API 同版）。不需資料遷移；沒有新的瀏覽器儲存鍵。回滾就是回到前一版 server。

## Open Questions

（無）原型未畫的四頁已在 D5–D7、D10 定稿：專案與儲存庫、憑證、稽核紀錄套列表頁規則，系統套設定型頁規則。

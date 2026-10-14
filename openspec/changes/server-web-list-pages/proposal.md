## Summary

server 網頁後台（apps/server-web）的三個列表頁、稽核紀錄頁與三個設定型頁，改用 desktop 的共用清單與設定零件：使用者、專案與儲存庫、憑證換成列表卡加共用列與換頁列，「邀請中」改成清單卡頂端的分頁，稽核紀錄的表格放進同一種列表卡；總覽、系統與帳號改成 768px 置中的設定型卡與列，總覽頂端改為四格數字磚；存取金鑰與裝置登入的撤銷鈕只在滑鼠移入該列時出現。

## Motivation

目標使用者是架設 Speclink server 的管理員，以及透過 AI 代理跑遠端 SDD 流程、要到後台管理自己存取金鑰的成員（開發者／PO／PM）。使用情境是「連線 Server」之後的日常管理：邀請成員、建專案與儲存庫、查與撤銷憑證、看稽核紀錄、在帳號頁管自己的金鑰。

- 第一刀 server-web-shell-alignment 已把殼、頁標題與登入這一族改成 desktop 的樣子，但頁標題以下仍是 admin-console-redesign 的舊零件：server-web 自寫的 `DataList`（寬螢幕表格、窄螢幕卡片）與 `ListToolbar`（帶標籤的搜尋框與下拉）、各頁手刻的區塊框。desktop 的規格頁、已封存頁與設定頁早已用 packages/ui 的 `ListCard`、`ListRow`、`ListPager`、`SearchField`、`SettingsCard`、`SettingsRow`，server-web 一個都沒用。
- 使用者、專案與儲存庫、憑證三個列表沒有換頁，資料一多就整頁拉長；稽核紀錄只有「上一頁／下一頁」，看不到總筆數，也不能改每頁筆數。
- 使用者頁的待接受邀請是頁底另一塊框，與使用者清單長得不一樣；desktop 已封存頁以卡片頂端分頁區分「變更／討論」，同一種東西在兩個產品有兩種畫法。

本變更是討論 server-web-desktop-alignment 定案三刀中的第二刀（列表頁與設定型頁），依賴已封存的第一刀。第三刀 server-web-detail-pages（抽屜改詳情頁、建立類改對話框、刪 DetailSheet）在本刀封存後轉出。視覺基準是討論中確認的互動原型「server 後台對齊 desktop」的 A 列（總覽、使用者列表）、B 列（帳號頁）與 C 列（375px 窄螢幕）；原型沒畫的專案與儲存庫、憑證、系統、稽核紀錄四頁，依討論的延後項目在本刀 design 照已畫的畫板決定。

## Proposed Solution

- **列表頁（使用者、專案與儲存庫、憑證）**：版面與 desktop 規格頁相同——頁標題區右側是搜尋框、篩選下拉與頁級動作；其下一張列表卡填滿主區剩餘高度，列在卡內捲動，卡底是共用換頁列（每頁 20／50／100 筆，預設 20）。搜尋與換頁在前端進行（資料本來就整份載入）。
  - 使用者：卡片頂端分頁「使用者 n｜邀請中 n」。使用者列為頭像字、名稱與 email、管理員籤、狀態籤、所屬專案數；邀請列為受邀者 email、顯示名稱與角色與要加入的專案、到期日、「撤回邀請」。分頁狀態放在網址（`?tab=pending`），總覽的「待啟用」數字磚與「查看邀請」直接開到邀請中分頁。
  - 專案與儲存庫：專案列為名稱、專案代號、儲存庫數；點一列照舊開抽屜。
  - 憑證：卡片頂端分頁「存取金鑰 n｜裝置 n」；列不可點，列尾是狀態籤與滑過才出現的撤銷鈕。
- **稽核紀錄**：寬螢幕的表格放進列表卡（表頭固定、列在卡內捲動），卡底換成同一條共用換頁列；窄螢幕改成一列一筆的清單列。篩選與分頁照舊由伺服器計算，每頁筆數以 `limit` 參數送給伺服器。為了顯示「第 a–b 筆，共 N 筆」，稽核 view model 新增符合篩選的總筆數欄位 `total`（只加欄位，不改既有欄位）。
- **設定型頁（總覽、系統、帳號）**：內容改為 768px 置中欄，每個區塊是一張設定型卡（14px 小標、圓角卡、一列一筆）。
  - 總覽：頂端四格數字磚（與 desktop 閱讀卡的數字磚同款、每格可點進對應頁），其下「需要處理」「系統健康」「最近活動」三張卡；後兩張卡的標頭右側是前往系統頁與稽核紀錄頁的連結。
  - 系統：執行環境、儲存狀態、匯出、危險區四張卡，各值改為列。
  - 帳號：我的專案、存取金鑰、裝置登入、登入工作階段四張卡；存取金鑰與裝置登入的撤銷鈕滑過才出現。
- **撤銷鈕滑過才出現**：存取金鑰與裝置登入列（帳號頁、憑證頁）的撤銷鈕平時隱藏，滑鼠移入該列或鍵盤聚焦到按鈕時出現；沒有滑鼠的觸控裝置恆常顯示。「撤回邀請」不在此列，恆常顯示。
- **窄螢幕（<1024px）**：列表改為可換行的清單列（使用者列把管理員標記併入 email 那一行、隱藏專案數）；換頁列收成「第 a–b 筆，共 N 筆」與「‹ n / m ›」；頁標題區的搜尋與篩選換到標題下方一列。不出現整頁水平捲動。
- **共用元件（packages/ui）補強**，desktop 現有呼叫端外觀與行為不變：
  - `ListRow`：點擊改為選配（不給就是不可點的靜態列，沒有 hover 底色與 ›）、標題可改非等寬字、新增列尾動作槽。
  - `ListPager`：新增窄版（compact），並與 `usePaging` 一起由套件入口匯出。
  - `SearchField`：新增無障礙名稱。
  - `SettingsCard`：小標右側新增選配槽（放連結或灰字說明）；`SettingsRow` 支援列內「滑過才出現」的按鈕。
  - `DocTiles`：每格可帶一個連結元素，帶了就整格可點。
  - `PageHeader`：空間不足時動作槽換到標題下方一列。
  - `TabsTrigger`：標籤後可帶計數籤；desktop 已封存頁的分頁改用它（畫面不變）。
- **刪除** server-web 自有的 `DataList`、`ListToolbar`。
- **規格**：server-web-console 六條需求 MODIFIED、server-admin 一條需求 MODIFIED（見 Capabilities）。

相容性影響：只改 server 網頁後台與共用 UI 套件的畫面，加上稽核 view model 多一個欄位。網址、六個目的地、其餘 server API、CLI 人眼輸出與 `--json`、引擎全部不變；稽核 API 的既有欄位 `entries`、`totalPages` 與參數不變，只多 `total`，舊的呼叫端不受影響。desktop 的規格頁、已封存頁、設定頁與閱讀卡數字磚外觀與行為不變。抽屜（使用者細節、專案細節、邀請、建立專案、建立存取金鑰）與一次性祕密值的頁面橫幅照舊，留給第三刀。既有使用者不需遷移。

## Alternatives Considered

- 稽核紀錄改用卡片列、不留表格：稽核欄位多（時間、動作、對象、操作者、來源），寬螢幕用表格最好掃讀；討論已定「保留表格但放進同一種列表卡」。
- 稽核換頁列不顯示總筆數、只顯示「第 n / m 頁」：會成為後台第三種換頁列樣式；伺服器本來就算出總筆數，多回一個欄位成本很低。
- server-web 自寫靜態列與連結版數字磚：同一個介面元件兩份實作，違反 desktop-app「共用元件唯一來源」的精神。
- 沒有待接受邀請時隱藏「邀請中」分頁：卡片標頭會隨資料忽有忽無；分頁常駐、計數 0 時切過去顯示說明，管理員才知道邀請在哪裡看。
- 撤銷鈕恆常顯示：與 desktop「動作鈕滑過才出現」不一致，討論已裁定滑過才出現。

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `server-web-console`: 「管理列表提供搜尋、篩選、分頁與具引導的空狀態」列表頁改列表卡加共用換頁列、三個列表前端換頁、稽核換頁列顯示總筆數與每頁筆數；「使用者頁呈現尚未接受的邀請」改為清單卡頂端的「邀請中」分頁；「總覽提供可行動入口與待辦」指標卡改數字磚、區塊改設定型卡；「帳號頁呈現我的專案」改設定型卡；「共用設計系統維持高密度可存取體驗」工具列控件條款改為搜尋框加篩選、新增列表頁與設定型頁兩種版面與撤銷鈕滑過才出現；「管理列表以抽屜承載建立與編輯」只改列內容的條款與情境（使用者列改為名稱、email、管理員籤、狀態與專案數）。最後兩條不在討論結論的四條之內：前者是 propose 期發現工具列與版面條款會隨本刀失準而補上，後者是使用者列欄位依原型改變後該條情境的字面會失準而補上，抽屜本身的條款由第三刀整條改寫。
- `server-admin`: 「管理 browser API 提供最小且完整的頁面 view model」稽核 view model 額外回傳符合篩選的總筆數。

## Impact

- Affected specs: server-web-console（6 條 MODIFIED）、server-admin（1 條 MODIFIED）
- Affected app／package／crate: `apps/server-web`（後台 SPA）、`packages/ui`（共用 UI 元件）、`crates/host/speclink-server`（稽核 view model 多一個欄位）；`apps/desktop` 程式碼不改（只隨共用元件的向後相容補強一起跑測試）；speclink-core、speclink-cli、speclink-host 其餘部分不受影響
- Affected code:
  - New: `apps/server-web/src/components/RevokeButton.tsx`、`apps/server-web/src/components/FilterSelect.tsx`（頁標題區的篩選下拉，承接 `ListToolbar` 刪除後的「全部」哨兵值作法）
  - Modified: `apps/server-web/src/pages/admin/UsersPage.tsx`、`apps/server-web/src/pages/admin/RegistryPage.tsx`、`apps/server-web/src/pages/admin/CredentialsPage.tsx`、`apps/server-web/src/pages/admin/AuditPage.tsx`、`apps/server-web/src/pages/admin/OverviewPage.tsx`、`apps/server-web/src/pages/admin/SystemPage.tsx`、`apps/server-web/src/pages/AccountPage.tsx`、`apps/server-web/src/api/client.ts`、`apps/server-web/src/i18n/messages.ts`、`apps/server-web/src/__tests__/users.test.tsx`、`apps/server-web/src/__tests__/registry.test.tsx`、`apps/server-web/src/__tests__/credentials.test.tsx`、`apps/server-web/src/__tests__/audit.test.tsx`、`apps/server-web/src/__tests__/overview.test.tsx`、`apps/server-web/src/__tests__/system.test.tsx`、`apps/server-web/src/__tests__/account.test.tsx`、`apps/server-web/src/__tests__/toolbar.test.tsx`、`apps/server-web/src/__tests__/wording.test.tsx`、`apps/server-web/src/__tests__/a11y.test.tsx`、`apps/server-web/src/__tests__/helpers/adminHarness.tsx`、`packages/ui/src/index.ts`、`packages/ui/src/components/ListRow.tsx`、`packages/ui/src/components/ListPager.tsx`、`packages/ui/src/components/SearchField.tsx`、`packages/ui/src/components/DetailShell.tsx`、`packages/ui/src/components/ui/settings.tsx`、`packages/ui/src/components/ui/page-header.tsx`、`packages/ui/src/components/ui/tabs.tsx`、`packages/ui/src/components/ArchivedList.tsx`、`packages/ui/src/__tests__/listRow.test.tsx`、`packages/ui/src/__tests__/listPager.test.tsx`、`packages/ui/src/__tests__/settingsParts.test.tsx`、`packages/ui/src/__tests__/detailShell.test.tsx`、`packages/ui/src/__tests__/pageHeader.test.tsx`、`packages/ui/src/__tests__/tabs.test.tsx`、`crates/host/speclink-server/src/admin.rs`、`crates/host/speclink-server/tests/it/admin/audit_filter.rs`
  - Removed: `apps/server-web/src/components/DataList.tsx`、`apps/server-web/src/components/ListToolbar.tsx`

---
topic: server 網頁後台（server-web）的 UI／UX 調整成與已全面改版的 desktop 一致的設計風格
slug: server-web-desktop-alignment
status: promoted
created: 2026-10-13
created_by: MomoChen <momochenisme@gmail.com>
promoted_to: server-web-shell-alignment, server-web-list-pages, server-web-detail-pages
---

# Discussion: server 網頁後台（server-web）的 UI／UX 調整成與已全面改版的 desktop 一致的設計風格

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

desktop 已依 desktop-ui-redesign 全面改版（左側標題列、詳情頁取代抽屜、PageHeader、設定零件、列式清單與換頁），server 網頁後台（apps/server-web）的殼、版面與互動仍停在 admin-console-redesign 的樣子，使用者要求兩邊的設計風格與 UI／UX 一致。未經 grill 階段：需求有具體參考對象（desktop 現況與 desktop-ui-redesign 的二十張畫板原型），可直接列假設。相關規格為 server-web-console（主體）與 desktop-app（參考）；進行中變更 clickable-cursor-hover-rule 的 delta 觸及 clickable-cursor-hover-rule: server-web-console（ADDED「網頁後台可點元素顯示手指游標」）。
Prior discussions: admin-console-redesign, desktop-ui-redesign, web-service-navigation-redesign

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-10-13)

**Focus**: server-web 對齊 desktop 的範圍與五個方向（範圍、殼、頁面版面與元件、抽屜、密度）
**Position**: 五條假設全數確認：server-web 全面對齊 desktop 的殼、版面、元件與互動，顏色與資訊架構不動。
- 決策地圖：1 範圍 → 2 殼 → 3 頁面版面與元件 → 4 抽屜改詳情頁 → 5 密度 → 6 分刀（待定）
- 範圍：管理殼、六個管理頁、帳號頁、登入／setup／邀請／啟用四個流程頁；六個目的地與網址不變；顏色字型不動——兩邊 index.css 已 import 同一份 packages/ui/src/theme.css（admin-console-redesign 否決換色票的理由仍成立）
- 殼：頂部橫貫 header（ConsoleLayout.tsx）改成 desktop 形狀——側欄頂端 48px 放品牌、主區頂列放麵包屑（後台 / 使用者）、email／語言／登出收進側欄底部帳號選單（dropdown-menu 原語）；不做 56px 圖示列（後台無多專案切換）；一般成員也有只含品牌＋帳號選單的窄側欄，殼只有一種形狀；HeaderAccount.tsx「不引入 dropdown 原語」的理由已因 packages/ui 已有 dropdown-menu 而失效
- 頁面版面：頁標題改 PageHeader（24px 一般字重＋灰字說明＋右側主要動作）；使用者、專案與儲存庫、憑證三個列表頁以 ListCard＋ListRow＋ListPager＋SearchField 取代 server-web 自有的 DataList＋ListToolbar（全寬、卡內捲動、底部換頁列）；稽核紀錄欄多保留表格但放進同一種 ListCard、同列高與細線；帳號、系統、總覽為 768px 置中＋SettingsCard／SettingsRow；四個流程頁對齊 desktop 零專案空狀態與新增專案對話框的卡；證據為 server-web 目前零使用 PageHeader、Settings*、ListRow／ListCard／ListPager、SearchField、DropdownMenu、StatusPill
- 互動：右滑抽屜（DetailSheet）改為詳情頁——主區換成該筆資料的頁、有自己的網址、麵包屑「後台 / 使用者 / <email>」、內容用設定型卡與列；邀請、建立專案、建立存取金鑰改置中對話框（同 desktop 新增專案）；破壞性動作確認框不變；重開 admin-console-redesign 的抽屜裁定，因其理由「與 desktop 同語彙」已反轉——desktop-ui-redesign 讓 Sheet 自桌面退場，server-web 成為唯一抽屜使用者
- 密度：字級與控件高度與 desktop 相同（外殼 11–13px、閱讀內文 15px、按鈕 28–36px）；改寫 server-web-console「正文 ≥16px、互動目標 ≥44×44px」——無任何測試檢查、共用 Button 本為 h-9／h-7、全站只有 header email 連結有 min-h-11；重開 web-service-navigation-redesign「44px 觸控目標為驗收條件」，理由是承諾從未被共用元件遵守且一致性需要同一套控件；鍵盤操作、focus ring、4.5:1 對比、skip link 保留
- 受影響規格（server-web-console）：「全部 browser route 由單一 SPA 提供可發現導覽」（帳號入口、一般成員側欄）、「共用設計系統維持高密度可存取體驗」（16px／44px）、「管理列表以抽屜承載建立與編輯」（整條改寫）
**Ruled out**: 保留抽屜只換外觀——兩個產品「點一列」反應不同，正是要消掉的差異；列表保留表格——清單外觀仍兩套；做 56px 圖示列——後台無多專案，圖示列會是空欄；保留 44px／16px——server-web 控件要比 desktop 大一號，一致只能做一半；一般成員維持無側欄——殼會有兩種形狀
**Open**: 窄螢幕（<1024px）導覽是否保留 Sheet；詳情頁的網址形狀與規格路由清單；首次導覽（Tour）步驟是否受殼改動影響；是否先做互動原型當視覺基準；分刀

### Round 2 — interview (2026-10-13)

**Focus**: propose 前是否先做互動原型當各刀的視覺基準；順帶以程式碼查清三個事實節點
**Position**: 先做互動原型再分刀與 propose，約 7 張畫板，追加在 desktop 原型同一份 artifact。
- 畫板：殼＋總覽（管理員）、使用者列表、使用者詳情頁、邀請對話框、帳號頁（一般成員窄側欄）、登入頁、375px 窄螢幕
- 理由：desktop-ui-redesign 與 detail-content-redesign 皆先以原型逐面確認、各刀 design 以原型為準；這次有三樣 desktop 沒有現成樣子——無圖示列的殼、側欄底部帳號選單、以設定卡排的使用者詳情頁
- 事實（查程式碼定案，不需使用者裁決）：窄螢幕（<1024px）導覽保留 Sheet——desktop 無窄螢幕模式可對齊，導覽抽屜與要拿掉的資料詳情抽屜是兩回事，web-service-navigation-redesign 亦定行動版 Sheet；Tour 每步指向側欄六個目的地的 data-tour，帳號移位不影響；詳情頁網址 /admin/users/<id>、/admin/registry/<代號> 落在規格既有的 /admin/* 內，路由清單不改
- 抽屜盤點：DetailSheet 共 6 處用途——使用者細節、專案細節改詳情頁；邀請使用者、建立專案、建立存取金鑰改對話框；憑證頁只有確認框不動
- 測試影響：users.test.tsx（49 處）、registry.test.tsx（23 處）、account.test.tsx（13 處）提及對話框或抽屜，需隨之改寫
**Ruled out**: 不做原型直接 propose——畫面要到 apply 後才看得到，改動成本較高
**Open**: 原型確認後的修正；分刀

### Round 3 — interview (2026-10-13)

**Focus**: 原型的畫板範圍——使用者指出缺少登入相關設計
**Position**: 原型補齊登入這一族：登入頁原已在 B 列，另補初始設定、接受邀請、裝置登入三頁，並給登入頁加錯誤狀態；共十張 server 畫板，追加在 desktop 原型畫布最下方。
- 原型位置：https://claude.ai/artifact/1QhDWmQi6muFFQnHTqqHa3 ，A 列殼＋總覽、使用者列表、使用者詳情頁；B 列邀請對話框、帳號頁（一般成員）、登入；C 列 375px 窄螢幕、初始設定、接受邀請、裝置登入
- 登入這一族同一個畫法：置中橫式字標（同 desktop 零專案空狀態）＋一張對話框式卡片（12px 圓角、15px 標題＋灰字說明、頁尾細線上方右側主要鈕，同 desktop 新增專案對話框）；錯誤以 role=alert 緊貼欄位；連結無效以琥珀圖示＋標題＋說明
- 初始設定沿用新增專案對話框的「步驟 n / 2」＋步驟條；裝置登入分輸入代碼、代碼無效、確認（大字代碼＋核准後的身分）、已核准／已拒絕四態；桌面 app 與 CLI 皆走裝置代碼（crates/adapters/speclink-cli/src/verbs/connection.rs 帶 user_code），說明文字寫兩者
- 原型裡五個超出第 1 輪假設、待使用者裁定的設計：「邀請中」改成卡片標頭分頁；一次性邀請連結在對話框內顯示（改寫規格「成功即關閉、頁面橫幅回饋」）；使用者詳情頁用 768px 單欄、保留四個分頁、管理員改開關；一般成員 56px 窄側欄；撤銷鈕 hover 才出現
**Open**: 原型十張的修正與五個自訂設計的裁定；分刀

### Round 4 — interview (2026-10-13)

**Focus**: 後台左上角的品牌標示用哪一套 Logo
**Position**: 後台側欄頂端改用有文字的橫式 Logo（Wordmark，高 20px），取代「裸標記＋Speclink 純文字」；一般成員的側欄因此改為與管理員同寬的 200px。
- 適用：管理員與一般成員的側欄頂端、375px 窄螢幕導覽 Sheet 的頂端；登入這一族本來就是置中 40px 橫式 Logo，不變
- 刻意偏離 desktop-ui-redesign 的品牌擺放規則（Wordmark 只在零專案空狀態與「關於」、圖示列頂用裸標記）：desktop 左上角已有專案名作為脈絡、品牌由圖示列的裸標記承擔；後台沒有圖示列也沒有專案名，左上角就是品牌本身；現行 ConsoleLayout.tsx 的 header 也已是 <Wordmark className="h-5" />
- 一般成員：橫式 Logo 寬約 75px，放不進 56px 窄欄，改為 200px 側欄——頂端橫式 Logo、中間只有「帳號」一項（作用中）、底部帳號選單；殼與管理員同一種形狀
**Ruled out**: 一般成員保留 56px 窄欄、只在那裡用裸標記——同一個後台出現兩套左上角品牌，與使用者裁定不符；一般成員拿掉側欄、Logo 放主區頂列——殼又變回兩種形狀（第 1 輪已否決）
**Open**: 原型十張的修正與五個自訂設計的裁定（「邀請中」分頁、對話框內顯示邀請連結、詳情頁單欄、撤銷鈕 hover 才出現；一般成員側欄已由本輪改為 200px）；分刀

### Round 5 — interview (2026-10-13)

**Focus**: 原型十張與剩下四個自訂設計是否成立
**Position**: 使用者確認原型十張全數成立，剩下四個自訂設計全部採用。
- 「邀請中」從頁底第二塊改為清單卡頂端的卡片標頭分頁（使用者 n｜邀請中 n），同 desktop 已封存頁以分頁區分變更／討論
- 一次性邀請連結在對話框內顯示：送出後對話框換成「邀請已建立」＋綠系成功框（連結＋複製）＋「完成」，取代「成功即關閉、頁面橫幅回饋」；改寫 server-web-console「管理列表以抽屜承載建立與編輯」與「後台狀態徽章語意色」的揭示橫幅字面
- 使用者詳情頁 768px 單欄：標題列（22px 名稱＋狀態籤、右側停權＋⋯）、灰字 meta、卡片標頭分頁（概要／成員資格／憑證／稽核）＋設定型卡與列；管理員旗標由勾選框改為開關列；不套 desktop 三欄文件樹——使用者資料不是文件
- 撤銷鈕 hover 才出現（帳號頁與詳情頁的存取金鑰、裝置登入列），同 desktop「動作鈕 hover 才出現」
- 原型即為各刀 design 的視覺基準：https://claude.ai/artifact/1QhDWmQi6muFFQnHTqqHa3 最底「server 後台對齊 desktop」A／B／C 三列
**Open**: 分刀

### Round 6 — interview (2026-10-13)

**Focus**: 分刀
**Position**: 分三刀依序落地：cut 1 殼與登入流程 → cut 2 列表頁與設定型頁 → cut 3 詳情頁與對話框。
- cut 1 `server-web-shell-alignment`：ConsoleLayout（側欄頂端橫式 Logo、主區麵包屑、側欄底部帳號選單、一般成員 200px 側欄、窄螢幕 Sheet 導覽保留）、全頁 PageHeader、FocusLayout 四頁置中字標＋對話框式卡片
- cut 2 `server-web-list-pages`（依賴 cut 1）：使用者／專案與儲存庫／憑證換 ListCard＋ListRow＋ListPager＋SearchField、「邀請中」分頁、稽核表格進 ListCard、總覽數字磚、系統與帳號換 Settings 零件、撤銷鈕 hover、刪 DataList 與 ListToolbar
- cut 3 `server-web-detail-pages`（依賴 cut 2，最後一刀）：使用者與專案詳情頁（/admin/users/<id>、/admin/registry/<代號>）＋ ‹ n／m ›、建立類改對話框、DetailNav 搬進 packages/ui、刪 DetailSheet
- 對話框用 packages/ui 既有的 AlertDialog（desktop 新增專案對話框 WorkspaceChooser.tsx 即用它），不新增原語
- 順序理由：cut 2 與 cut 3 都大改 UsersPage.tsx 與 RegistryPage.tsx，不能並行；cut 1 先定殼與 PageHeader
- 前置：先封存 clickable-cursor-hover-rule（status done，ADDED「網頁後台可點元素顯示手指游標」於 server-web-console，需求名不重疊）
**Ruled out**: 只切兩刀——頁面刀要改六頁加三個大測試檔，品質關卡每輪過重；按頁面切——殼仍須先行，DetailNav 搬遷、DataList 刪除等共用工作被拆散到不同刀

## Conclusion

**Decision**: server 網頁後台（apps/server-web）全面改用 desktop 的設計語言——殼、頁面版面、元件與互動都對齊 desktop，顏色字型與資訊架構不動，以互動原型為視覺基準，分三刀依序落地。
- 範圍：管理殼、六個管理頁（總覽、使用者、專案與儲存庫、憑證、系統、稽核紀錄）、帳號頁、登入／初始設定／接受邀請／裝置登入四個流程頁；六個目的地與網址不變；顏色字型不動——兩邊 index.css 已 import 同一份 packages/ui/src/theme.css
- 殼：拿掉頂部橫貫 header；側欄 200px，頂端 48px 放有文字的橫式 Logo（Wordmark，高 20px）；主區頂列 48px 放麵包屑（後台 / 使用者 / <名稱>）；email、語言、登出收進側欄底部的帳號選單（頭像＋名稱＋email，用 dropdown-menu 原語，選單項：帳號、介面語言、登出）；不做 56px 圖示列；一般成員也有同寬側欄，頂端橫式 Logo、中間只有「帳號」一項、底部帳號選單，殼只有一種形狀；窄於 1024px 時側欄收成左上「開啟導覽」鈕，導覽以 Sheet 從左側滑出
- 品牌擺放刻意偏離 desktop 規則（desktop 的 Wordmark 只在零專案空狀態與關於、左上角用裸標記）：後台沒有圖示列也沒有專案名，左上角就是品牌；現行 ConsoleLayout.tsx 已用 Wordmark
- 頁面版面：全部頁標題改 PageHeader（24px 一般字重＋灰字說明＋右側頁級動作）；列表頁（使用者、專案與儲存庫、憑證）全寬、ListCard＋ListRow＋ListPager＋SearchField，卡內捲動、底部換頁列，與 desktop 規格頁同款；稽核紀錄欄位多，保留表格但放進同一種 ListCard、同列高與細線；總覽、系統、帳號為 768px 置中＋SettingsCard／SettingsRow；總覽四格數字磚（DocTiles 同款、可點進對應頁）＋需要處理、系統健康、最近活動三張設定型卡
- 使用者頁的「邀請中」改為清單卡頂端的卡片標頭分頁（使用者 n｜邀請中 n），同 desktop 已封存頁以分頁區分變更／討論
- 互動：右滑抽屜（DetailSheet）改為詳情頁——點一列主區換成該筆資料的頁，有自己的網址（/admin/users/<id>、/admin/registry/<代號>，落在規格既有的 /admin/* 內），麵包屑右端 ‹ n／m ›；使用者詳情頁 768px 單欄：22px 名稱＋狀態籤、右側「停權」＋⋯、灰字 meta、卡片標頭分頁（概要／成員資格／憑證／稽核）＋設定型卡與列，管理員旗標改開關列；不套 desktop 三欄文件樹——使用者資料不是文件
- 建立類動作（邀請使用者、建立專案、建立存取金鑰）改置中對話框，用 packages/ui 既有的 AlertDialog（desktop 新增專案對話框同款），不新增原語；一次性祕密值（邀請連結、存取金鑰明文）在對話框內顯示——送出後換成成功狀態＋綠系成功框（值＋複製）＋「完成」；破壞性動作確認框不變
- 撤銷鈕（存取金鑰、裝置登入列）hover 才出現，同 desktop「動作鈕 hover 才出現」
- 登入這一族（登入、初始設定、接受邀請、裝置登入）：置中 40px 橫式 Logo＋一張對話框式卡片（12px 圓角、15px 標題＋灰字說明、頁尾細線上方右側主要鈕），同 desktop 零專案空狀態與新增專案對話框；初始設定加「步驟 n / 2」＋步驟條；錯誤以 role=alert 緊貼欄位；連結無效以琥珀圖示＋標題＋說明；裝置登入四態（輸入代碼、確認時大字代碼＋核准後的身分、已核准、已拒絕），說明寫「桌面 app 或 CLI」（CLI 的 verbs/connection.rs 同用 user_code）
- 密度與無障礙：字級與控件高度同 desktop（外殼 11–13px、閱讀內文 15px、按鈕 28–36px）；改寫 server-web-console「正文 ≥16px、互動目標 ≥44×44px」——無測試檢查、共用 Button 本為 h-9／h-7；鍵盤操作、skip link、focus ring、4.5:1 對比、aria-label、reduced-motion 保留
- 不受影響（查證）：首次導覽 Tour 每步指向側欄六個目的地的 data-tour，帳號移位不影響
- 互動原型（十張畫板）：https://claude.ai/artifact/1QhDWmQi6muFFQnHTqqHa3 最底「server 後台對齊 desktop」A 列殼＋總覽、使用者列表、使用者詳情頁；B 列邀請對話框、帳號頁（一般成員）、登入；C 列 375px 窄螢幕、初始設定、接受邀請、裝置登入；各刀 design 以它為視覺基準
- 前置：先封存 clickable-cursor-hover-rule（同改 server-web-console，ADDED「網頁後台可點元素顯示手指游標」，需求名不重疊）
- **cut 1 `server-web-shell-alignment`**: 殼、頁標題與登入這一族
  - ConsoleLayout：側欄頂端橫式 Logo、主區麵包屑、側欄底部帳號選單（dropdown-menu）、一般成員 200px 側欄只含「帳號」、窄螢幕 Sheet 導覽保留
  - 全部頁面標題換 PageHeader
  - FocusLayout：登入、初始設定、接受邀請、裝置登入改置中字標＋對話框式卡片，初始設定加步驟條
  - server-web-console MODIFIED：「全部 browser route 由單一 SPA 提供可發現導覽」（帳號入口改側欄底部選單、一般成員有側欄）、「共用設計系統維持高密度可存取體驗」（16px／44px 改寫為 desktop 密度、header 數值條款改為殼的新結構）
- **cut 2 `server-web-list-pages`**: 列表頁與設定型頁換共用元件；依賴 cut 1
  - 使用者、專案與儲存庫、憑證換 ListCard＋ListRow＋ListPager＋SearchField；「邀請中」改分頁；稽核紀錄表格進 ListCard
  - 總覽數字磚＋設定型卡；系統、帳號換 SettingsCard／SettingsRow；撤銷鈕 hover 才出現
  - 刪除 server-web 自有的 DataList、ListToolbar
  - server-web-console MODIFIED：「管理列表提供搜尋、篩選、分頁與具引導的空狀態」「使用者頁呈現尚未接受的邀請」「總覽提供可行動入口與待辦」「帳號頁呈現我的專案」
- **cut 3 `server-web-detail-pages`**: 抽屜改詳情頁、建立類改對話框；依賴 cut 2；最後一刀
  - 使用者詳情頁、專案詳情頁（/admin/users/<id>、/admin/registry/<代號>）＋ ‹ n／m ›
  - 邀請使用者、建立專案、建立存取金鑰改 AlertDialog 對話框；邀請連結與金鑰明文在對話框內顯示
  - desktop 的 DetailNav 搬進 packages/ui 兩邊共用（第二個消費者；守門測試禁止同名元件各寫一份）
  - 刪除 DetailSheet；Sheet 只剩窄螢幕導覽一個用途
  - server-web-console：「管理列表以抽屜承載建立與編輯」整條改寫（含名稱，RENAMED 或 REMOVED＋ADDED 由 propose 期決定）、「不可變識別欄位唯讀且更名為顯式動作」「後台狀態徽章語意色」（揭示改在對話框內）MODIFIED
  - 測試：users.test.tsx（49 處）、registry.test.tsx（23 處）、account.test.tsx（13 處）的抽屜斷言改寫
**Rationale**: 兩邊的色票與字型早已同源，「不一致」的實體是殼、版面、元件與互動模式；desktop-ui-redesign 讓 Sheet 自桌面退場後，server-web 成了唯一還用抽屜的地方，admin-console-redesign 當初選抽屜的理由「與 desktop 同語彙」已反轉，改為詳情頁加對話框才是同一套心智模型；所需元件（PageHeader、Settings 零件、ListCard 家族、SearchField、DropdownMenu、AlertDialog、DocTiles 樣式）packages/ui 都已有，server-web 一個都沒用，所以工作是換用而非新建，只有 DetailNav 需要搬家；依「殼 → 列表 → 詳情」切刀是因為後兩刀都大改同兩個頁面檔，必須排先後，而殼與 PageHeader 是後兩刀的地基。
**Rejected alternatives**:
- 換色票或調間距 — 兩邊 theme 同源，視覺 token 無可換之物（admin-console-redesign 舊裁定仍成立）
- 保留抽屜只換外觀 — 兩個產品「點一列」的反應不同，正是要消掉的差異
- 列表全部保留表格 — 清單外觀仍是兩套（稽核紀錄因欄位多例外，但表格放進 ListCard）
- 做 desktop 的 56px 圖示列 — 後台沒有多專案切換，圖示列會是空欄
- 保留「正文 ≥16px、互動目標 ≥44px」 — server-web 控件要比 desktop 大一號，一致只做一半；且共用 Button 從未遵守
- 一般成員維持無側欄 — 殼會有兩種形狀
- 一般成員用 56px 窄欄＋裸標記 — 同一個後台出現兩套左上角品牌，與使用者裁定不符
- 側欄頂端用裸標記＋純文字 — 使用者裁定用有文字的橫式 Logo
- 使用者詳情頁套 desktop 三欄文件樹 — 使用者資料不是文件，沒有可排成樹的東西
- 新增 Dialog 原語承載建立表單 — packages/ui 的 AlertDialog 已足夠，desktop 新增專案對話框即用它
- 不做原型直接 propose — 畫面要到 apply 後才看得到，改動成本高
- 只切兩刀 — 頁面刀要改六頁加三個大測試檔，品質關卡每輪過重
- 按頁面切刀 — 殼仍須先行，DetailNav 搬遷與 DataList 刪除等共用工作會被拆散
**Deferred**:
- 專案與儲存庫、憑證、系統、稽核紀錄四頁的逐頁畫面 — 原型只畫了使用者頁與帳號頁，其餘頁套同一套列表／設定型規則，各刀 design 期照已畫的畫板決定
- 專案詳情頁的分頁內容（儲存庫、成員）與更名動作的呈現 — cut 3 design 期以使用者詳情頁為範本決定
- 首次導覽 Tour 浮層的外觀 — 步驟目標不受影響，外觀隨 theme token 走，cut 1 實作時確認
**Capture to**: proposal、design、spec（server-web-console MODIFIED／RENAMED）、tasks；LANGUAGE.md 詞彙漂移——server-web-console 需求名與內文的「抽屜」是「詳情頁」的 avoid 同義詞，cut 3 改寫時一併收斂
**Next**: 先封存 clickable-cursor-hover-rule；再 /speclink-propose --from-discussion server-web-desktop-alignment 建 cut 1 `server-web-shell-alignment`；cut 2、cut 3 依序在前一刀封存後轉出，cut 3 帶 --last

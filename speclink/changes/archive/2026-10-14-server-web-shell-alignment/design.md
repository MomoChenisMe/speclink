## Context

server 網頁後台（apps/server-web）目前的殼由 ConsoleLayout 組成：頂部 48px 橫貫 header（左側橫式 Logo，右側 HeaderAccount 的 email 連結、LocaleSwitch 下拉、LogoutButton），下方左側 200px 管理導覽（只有管理員有），右側主內容。登入、初始設定、接受邀請、裝置登入由 FocusLayout 承載：左上角小 Logo，下方置中一張 `Card`，頁標題 24px 粗體。六個管理頁與帳號頁各自手寫 `<h1 className="text-2xl font-semibold">`。

desktop 已改成另一套樣子：左側標題列與主區頂列各 48px、主區頂列放麵包屑（`apps/desktop/src/components/TitleBar.tsx` 的 MainTitleBar）、頁標題用共用 `PageHeader`、對話框是 12px 圓角卡加「步驟 n / total」步驟條（WorkspaceChooser）、零專案空狀態是置中 40px 橫式 Logo。

本變更落在三處，全部是前端：

- `apps/server-web`：殼、頁標題、登入這一族四頁、i18n 文案與測試。
- `packages/ui`：新增 `Breadcrumbs`；`PageHeader`、`SheetContent`、dropdown-menu 原語各補一個能力。
- `apps/desktop`：MainTitleBar 改用共用 `Breadcrumbs`、WorkspaceChooser 的步驟列改用共用 `StepProgress`，外觀與行為不變（麵包屑按鈕多了鍵盤聚焦時的 focus ring）。

server crates、engine crates、CLI、server API 都不動，所以 crate 邊界、Host／Protocol 契約、local／remote 雙路徑都不涉及。視覺基準是討論 server-web-desktop-alignment 確認的原型（A 列殼＋總覽、B 列帳號頁與登入、C 列 375px 窄螢幕、初始設定、接受邀請、裝置登入）。

## Goals / Non-Goals

**Goals:**

- 後台殼與 desktop 同一種形狀：側欄頂端橫式 Logo、主區頂列麵包屑、帳號資訊收進側欄底部帳號選單；管理員與一般成員共用一種殼。
- 全部頁標題用共用 `PageHeader`，字級、字重、說明與動作槽與 desktop 相同。
- 登入這一族四頁與 desktop 對話框同一個畫法。
- 規格裡已失準的條款（header、16px／44px、header 上的語言切換）改寫成新殼的樣子。
- 新出現的共用介面元件（麵包屑）只有一份實作。

**Non-Goals:**

- 列表頁的清單外觀（DataList、ListToolbar 換 ListCard 家族）、總覽數字磚、系統與帳號頁換設定零件、撤銷鈕滑過才出現：第二刀 server-web-list-pages。
- 右滑抽屜改詳情頁、建立類動作改對話框、DetailNav 搬家、刪 DetailSheet：第三刀 server-web-detail-pages。本刀內 DetailSheet 照舊從右側滑出。
- 頁面內容寬度（768px 置中的設定型版面）：第二刀。本刀各頁內容維持現有寬度，只換標題區。
- 首次導覽（Tour）的步驟與浮層外觀：步驟目標（側欄六個目的地的 `data-tour`、`list-primary`）不受影響，浮層外觀隨 theme token 走，實作時確認不跑版即可。
- 顏色、字型、theme token：兩邊同源，不動。
- 網址、六個目的地、server API：不動。

## Decisions

### D1 殼改為側欄加主欄，兩種角色同一種形狀

ConsoleLayout 改成兩欄：

- **側欄**（`aside`，200px，`bg-sidebar`，右側細線）：
  - 頂端 48px 列：`Wordmark`（`h-5`），下緣細線。
  - 中段：`nav`，`aria-label` 為「管理導覽」（管理員）或「導覽」（一般成員）。管理員放 `AdminNav`；一般成員放只有「帳號」一項的導覽（`AdminNav.tsx` 新增 `MemberNav`，同一個 `NavItem`，在 `/account` 時作用中）。
  - 底部：`AccountMenu`（D2）。
- **主欄**：頂端 48px 列放麵包屑（D4），下方是 `main#main-content`，只有它捲動，內距 `p-5`（desktop 看板同值）。

拿掉 `header`（`role=banner`）。skip link 照舊是第一個可聚焦元素。管理員在 `/account` 時側欄照舊整條呈現且無項目高亮。

替代方案：保留 header 只把帳號移進去——不是 desktop 的形狀，否決。做 56px 圖示列——後台沒有多專案切換，圖示列會是空的，否決（討論已否決）。

### D2 帳號選單取代 HeaderAccount、LocaleSwitch、LogoutButton

新增 `apps/server-web/src/components/AccountMenu.tsx`：

- **觸發鈕**：佔側欄底部全寬。左側 28px 圓形頭像（顯示名稱第一個字，主色淡底主色字），中間兩行（顯示名稱 13px、email 11px 灰字、超長截斷），右側上下箭頭圖示。`aria-label` 為「帳號選單：<顯示名稱>」。
- **選單內容**（`DropdownMenuContent`，向上展開、與觸發鈕左緣對齊）：
  1. 標頭：顯示名稱、「<email> · 管理員」或「<email> · 成員」（`DropdownMenuLabel`）。
  2. 分隔線。
  3. 「帳號」：導向 `/account`。
  4. 「介面語言」群組標籤，下接三個單選項：繁體中文、English、跟隨系統（D3）。語言名稱維持母語寫法（沿用 LocaleSwitch 的理由）。
  5. 分隔線。
  6. 「登出」：沿用 LogoutButton 的流程——撤銷 session、`refresh()`、導向 server 回傳的 destination；失敗時導向 `/login`；請求期間此項停用。

三個舊元件刪除；HeaderAccount 註解裡「兩個項目不值得引入 dropdown 原語」的前提已不成立（packages/ui 已有 dropdown-menu，desktop 也在用）。

### D3 介面語言以選單內單選項呈現

dropdown-menu 原語補 `DropdownMenuRadioGroup` 與 `DropdownMenuRadioItem`（Radix 同名元件的薄包裝，項目樣式沿用 `menuItemClass`，選中項左側顯示勾號），由 `packages/ui/src/index.ts` 匯出。單選項的角色是 `menuitemradio`，`aria-checked` 由 Radix 處理，螢幕閱讀器能讀出目前的語言。

替代方案：做成子選單（原型畫了「介面語言 中文 ›」）——要多一組子選單原語，且只有三個選項，直接攤在選單內少一層操作。用一般選單項加勾號圖示——`menuitem` 不能帶 `aria-checked`，讀不出目前選哪個。兩者都否決。

### D4 麵包屑搬進共用元件庫

新增 `packages/ui/src/components/Breadcrumbs.tsx`，匯出 `Breadcrumbs` 元件與 `Crumb` 型別（`{ label: string; onClick?: () => void; mono?: boolean }`，與 desktop 現有 `Crumb` 相同）。渲染內容取自 MainTitleBar 現有的麵包屑：

- `nav`（`aria-label` 為共用字典的「頁面路徑」／「Breadcrumb」）內一列 12px 灰字，項與項之間是半透明的「/」。
- 最後一項為前景色；有 `onClick` 的項是按鈕、滑過變前景色、鍵盤聚焦時顯示 2px focus ring（共用 ring token）；`mono` 的項用等寬字型；超長截斷。

desktop 的 MainTitleBar 改為渲染 `<Breadcrumbs crumbs={crumbs} />`，`Crumb` 型別改由 `@speclink/ui` 匯入（`apps/desktop/src/App.tsx` 跟著改）。desktop 的外觀、拖曳區與 Windows 視窗鈕不變。

server-web 的麵包屑由 ConsoleLayout 依目前路徑產生：

| 路徑 | 麵包屑 |
| --- | --- |
| `/admin` | 後台 / 總覽 |
| `/admin/users` | 後台 / 使用者 |
| `/admin/registry` | 後台 / 專案與儲存庫 |
| `/admin/credentials` | 後台 / 憑證 |
| `/admin/system` | 後台 / 系統 |
| `/admin/audit` | 後台 / 稽核紀錄 |
| `/account` | 帳號 |

「後台」一律可點，點了導向 `/admin`。目的地名稱沿用 `nav.*` 字串。第三刀加詳情頁時再往後接一項。

替代方案：server-web 自寫一份——同一個介面元件兩份實作，違反 desktop-app「共用元件唯一來源」的精神，否決。讓 Crumb 支援 `href` 渲染成連結——desktop 用不到、多一種分支；後台麵包屑只有「後台」一項可點，按鈕加 `navigate` 足夠，否決。

### D5 PageHeader 加標題層級屬性

`PageHeader` 加 `level?: 1 | 2`，預設 2（desktop 現有呼叫端不變），決定標題渲染成 `h1` 或 `h2`；樣式不變。server-web 的頁標題用 `level={1}`：後台沒有外層 app 標題，頁標題就是文件的第一層標題（規格要求連續 heading；測試以 `heading level 1` 找頁標題）。

各頁標題區：

| 頁 | 標題 | 說明 | 動作槽 |
| --- | --- | --- | --- |
| 總覽 | 總覽 | 這台 server 的使用者、專案與憑證概況；點數字前往對應頁。 | — |
| 使用者 | 使用者 | 能登入這台 server 的帳號。 | 邀請使用者（`data-tour="list-primary"`） |
| 專案與儲存庫 | 專案與儲存庫 | 遠端工作流程的範圍：專案與其底下的儲存庫。 | 建立專案（`data-tour="list-primary"`） |
| 憑證 | 憑證 | 所有使用者的存取金鑰與裝置登入；撤銷後立即失效。 | 建立存取金鑰（連到帳號頁） |
| 系統 | 系統 | 執行環境、儲存狀態、匯出與資料結構遷移。 | 重看導覽（outline、sm） |
| 稽核紀錄 | 稽核紀錄 | 每一筆管理動作的紀錄；只能檢視，不能修改。 | — |
| 帳號 | 帳號 | <顯示名稱>（<email>）——你的專案、存取金鑰與登入紀錄。 | 建立存取金鑰 |

英文字串同時新增，兩種語言的 key 集合維持相等。使用者頁說明的「點一列開詳情頁」由第三刀補上（本刀點一列仍開抽屜）。

替代方案：server-web 頁標題改成 `h2`——頁面會沒有第一層標題，違反連續 heading，否決。新增 `as` 任意標籤屬性——只需要兩種層級，否決。

### D6 窄螢幕 Sheet 從左側滑出，兩種角色都有

`SheetContent` 加 `side?: "left" | "right"`，預設 `"right"`（DetailSheet 照舊）。`"left"` 時改為貼左、右側細線、從左滑入滑出；寬度與 reduced-motion 處理不變。

寬度小於 1024px 時（沿用 `useMediaQuery("(max-width: 1023px)")`）：

- 不渲染側欄。
- 主欄頂列在麵包屑左側放「開啟導覽」鈕（ghost、icon，`aria-label` 為「開啟導覽」，選單圖示）。
- Sheet 從左側滑出、寬 280px，內容與側欄相同：頂端 `Wordmark`、導覽（管理員六個目的地／一般成員一項）、底部 `AccountMenu`。`SheetTitle` 為「導覽」（視覺隱藏）。
- 點導覽項後關閉 Sheet；關閉後焦點回到觸發鈕（Radix 預設）。

一般成員也有這個鈕：殼只有一種形狀。

### D7 登入這一族的卡片骨架

FocusLayout 改為：

- 整頁 `min-h-screen`，內容垂直與水平置中、四周 24px 內距；寬 400px、上限 100%。
- 由上而下：`Wordmark`（`h-10`，置中）、卡片、語言切換。
- 卡片：`Card size="nested"`（12px 圓角，不覆蓋圓角與陰影），內距 20px，子元素間距 16px。
- 語言切換：卡片下方一列 12px 灰字，「中文 · English」兩個按鈕，目前語言為前景色且 `aria-pressed="true"`，點了寫入語言偏好（與帳號選單同一個儲存鍵）。

FocusLayout.tsx 另匯出三個小元件，四頁共用：

- `FocusHeader({ title, description, step? })`：`h1`（15px、medium）加灰字說明；給 `step`（`{ current, total, label }`）時上方多一列「<label> · 步驟 n / total」（11px 大寫字距灰字）與步驟條（每段 `h-1 w-8` 圓角，已到的段為主色）。這一列是 `packages/ui` 的共用元件 `StepProgress`，desktop WorkspaceChooser 改用同一個元件。
- `FocusFooter({ children })`：上緣細線、上內距 14px、右對齊的按鈕列。
- `FocusNotice({ title, body })`：連結無效與無權限用。琥珀色圓形圖示（`SEMANTIC_TONE` 的 warning 色）、15px 標題、灰字說明，整塊 `role="alert"`。

各頁內容：

- **登入**：標題「登入」，說明「用管理員給你的帳號登入 <主機名>。」（主機名取 `window.location.host`，等寬字）。欄位電子郵件、密碼。頁尾「登入」。
- **初始設定**：步驟 1「建立管理員帳號」，說明「這台 server 的第一個帳號；之後用它邀請其他人、建立專案。」，欄位電子郵件、顯示名稱、密碼，頁尾「建立管理員」。步驟 2「建立第一組專案與儲存庫」，說明「專案是遠端工作流程的範圍單位；完成後會直接進入後台，並顯示連線所需的資訊。」，四個欄位，兩個代號欄的標籤只放「專案代號」「儲存庫代號」，下方灰字「建立後不可變更。」（`Field` 的 `hint`），頁尾「建立專案與儲存庫」。專案與儲存庫頁的兩個代號欄（建立專案抽屜的專案代號、專案抽屜內新增儲存庫表單的儲存庫代號）改用同一種寫法，同一條限制只有一種說法。步驟列的 label 為「初始設定」。連結無效用 `FocusNotice`。
- **接受邀請**：標題「接受邀請」，說明「為 <email> 設定登入密碼；建立後會直接登入。」。密碼欄上方兩列唯讀資訊（電子郵件、伺服器），以 `dl` 呈現。頁尾「建立帳號」。連結無效用 `FocusNotice`。
- **裝置登入**：
  - 輸入代碼：標題「裝置登入」，說明「輸入裝置上顯示的代碼以核准登入。代碼在 Speclink 桌面 app 或 CLI 的登入畫面上。」，欄位「裝置代碼」，錯誤緊貼欄位下方，頁尾「下一步」。
  - 確認：標題「核准這個裝置？」，說明「代碼 <code> 的裝置要求以你的身分登入。只核准你自己剛才發起的登入。」；中間一塊淡底框，大字等寬代碼（22px）加一行「核准後以 <顯示名稱>（<email>）的身分登入」；頁尾「拒絕」（outline）與「核准」。
  - 已核准／已拒絕：標題「裝置登入」，結果句（`role=status`）加返回 app 的提示，沒有頁尾按鈕。
- **無權限**（一般成員開 `/admin`）：`FocusNotice`「沒有權限」。
- 載入中：卡片內一行灰字 `role=status`。

錯誤處理不變：欄位錯誤經 `Field` 以 `role=alert` 緊貼欄位下方；非欄位錯誤緊貼頁尾上方。提交中停用主要鈕。

### D8 規格密度與殼條款改寫

server-web-console 三條需求 MODIFIED（見 delta）。要點：

- 「共用設計系統維持高密度可存取體驗」：
  - 「header 高度、側欄寬度與主內容內距與 Desktop 相同」改為「側欄頂列與主區頂列高度、側欄寬度與主內容內距」。
  - 「header 與側欄恆常可見」改為「側欄與主區頂列」。
  - 「至少 44×44px 互動目標、至少 16px 正文」改為 desktop 密度：外殼文字 11–13px、閱讀內文 15px、按鈕高 28–36px；其餘無障礙條款原文保留。
  - 窄螢幕 Sheet 改為兩種角色都有、從左側滑出。
- 「全部 browser route 由單一 SPA 提供可發現導覽」：帳號入口改為側欄底部帳號選單；一般成員有側欄、只有帳號一項。
- 「介面語言支援中文與英文」：語言切換改在帳號選單與登入這一族卡片下方。

16px／44px 的改寫重開了 web-service-navigation-redesign 的「44px 觸控目標為驗收條件」：該承諾從未被共用元件遵守，且一致性需要同一套控件（討論 server-web-desktop-alignment 第 1 輪）。

## Implementation Contract

**行為（使用者看得到的）**

- 管理員登入後：左側 200px 側欄，頂端橫式 Logo，中段六個目的地分兩組，底部是自己的頭像字、名稱與 email；主區頂端一列「後台 / <目的地>」；沒有頂部橫條。
- 點側欄底部：向上展開選單，頂端是名稱與「email · 管理員」，選單項帳號、介面語言三選一（目前語言帶勾）、登出。選英文後介面立即變英文，重新整理後維持。
- 一般成員登入後：同一種側欄，中段只有「帳號」一項且為作用中；主區頂端一列「帳號」；底部選單標頭為「email · 成員」。
- 視窗窄於 1024px：側欄消失，主區頂列左端有「開啟導覽」鈕，點了從左側滑出含 Logo、導覽與帳號選單的面板。
- 每頁頂端是 24px 一般字重標題、灰字說明，右側是該頁主要動作。
- 登入、初始設定、接受邀請、裝置登入：畫面正中一個大 Logo，下方一張卡，卡下方「中文 · English」。

**介面與資料形狀**

- `packages/ui` 新匯出：`Breadcrumbs`、`Crumb`（型別）、`StepProgress`、`DropdownMenuRadioGroup`、`DropdownMenuRadioItem`。
- `PageHeader` 新屬性 `level?: 1 | 2`（預設 2）；`SheetContent` 新屬性 `side?: "left" | "right"`（預設 `"right"`）。
- `packages/ui` 共用字典新 key：麵包屑 `nav` 的 `aria-label`（`breadcrumbs.label`：zh-TW「頁面路徑」、en「Breadcrumb」）、步驟列文字（`steps.progress`：zh-TW「步驟 {current} / {total}」、en「Step {current} of {total}」，取代 desktop 的 `chooser.step`）。
- server-web 新 i18n key（zh-TW 與 en 同時新增）：帳號選單觸發鈕 aria-label（`shell.accountMenu`）、成員導覽與窄螢幕 Sheet 標題（`shell.nav`）、麵包屑根（`shell.console`）；角色字沿用既有的 `users.roleAdmin`／`users.roleMember`（值即「管理員」「成員」）。七個頁標題說明（帳號頁為 `account.description`，取代 `account.identity`）、登入說明、初始設定步驟標籤與兩段說明、代號欄共用的 `field.projectKey`／`field.repoKey`／`field.keyImmutable`（取代 `registry.projectKeyLabel`／`registry.repoKeyLabel`）、接受邀請的伺服器欄名稱、裝置登入確認標題與身分句、「開啟導覽」沿用 `shell.openNav`。不再使用的 key（`shell.menu`、`setup.title`、`registry.projectKeyLabel`、`registry.repoKeyLabel`、`account.identity`）隨之移除，兩種語言同步。
- 語言選項（值、選單用名稱、切換鈕用短名）只有一份清單，帳號選單與登入卡片下方共用；顯示名稱為空時退回 email 的規則只有一個函式，帳號選單、裝置身分句與帳號頁說明共用，代入使用者資料時值以字面文字放入。
- server API、網址、session 資料形狀不變（帳號選單用現有 `session.user` 的 `display`、`email`、`admin`）。

**失敗模式**

- 登出請求失敗：照舊導向 `/login`，不留在半登出狀態的畫面。
- `session.user.display` 為空字串：頭像字退回 email 第一個字；名稱列改顯示 email。
- 麵包屑遇到未列在表內的 `/admin/*` 路徑（例如舊書籤 `/admin/data` 在導向前的一瞬間）：只顯示「後台」。

**驗收**

- `npm test -w apps/server-web` 全綠，含改寫後的：
  - admin-console-shell.test.tsx：側欄頂端有共用字標；側欄恰有六個目的地且不含帳號；一般成員有側欄且只有帳號一項；帳號選單含帳號、三個語言單選項、登出；不存在 `banner` landmark；`/admin/users` 的麵包屑為「後台 / 使用者」且點「後台」回總覽；768px 時「開啟導覽」開出含六個目的地的 Sheet。
  - i18n.test.tsx：語言切換改由帳號選單的單選項操作；登入頁卡片下方的「English」可切換語言。
  - app.test.tsx、setup.test.tsx、invite.test.tsx、activate.test.tsx：頁標題與步驟文字的斷言改成新字串；裝置確認狀態顯示大字代碼與「核准後以 … 的身分登入」。
  - a11y.test.tsx：登入頁與管理殼無 axe 違規（兩個 `nav` landmark 各有不同的 `aria-label`）。
- `npm test -w packages/ui` 全綠，含新增 breadcrumbs.test.tsx（項目、分隔、最後一項前景色、`onClick` 項為按鈕、`aria-label`）、stepProgress.test.tsx（「[前綴 · ]步驟 n / total」與步驟條）、pageHeader.test.tsx 新增 `level={1}` 渲染 `h1`、menus.test.tsx 新增單選項的 `menuitemradio` 與 `aria-checked`。uiSingleSource 守門照舊通過（server-web 與 desktop 不再自行定義 `Crumb` 以外的同名元件；`Crumb` 是型別，不在守門範圍，但 desktop 也改為匯入共用型別）。
- `npm test -w apps/desktop` 全綠，其中 titleBar.test.tsx（麵包屑外觀與點擊行為不變）與 workspaceChooser.test.tsx（步驟列文字「步驟 n / 4」改由共用字典提供後不變）題目不改。
- `npm run build -w apps/server-web` 成功。
- 手動驗收：在瀏覽器以管理員與一般成員各走一次 1440px 與 375px，對照原型 A／B／C 列。

**範圍邊界**

- 範圍內：D1–D8 列出的殼、頁標題、四個流程頁、無權限頁、三個共用原語補強、麵包屑與步驟列搬進共用元件庫、i18n、測試、server-web-console 三條需求。
- 範圍外：各頁標題區以下的內容（清單、表格、卡片、抽屜）一律不動，唯一例外是專案與儲存庫頁的兩個代號欄（建立專案抽屜的專案代號、專案抽屜內新增儲存庫表單的儲存庫代號）改用灰字說明（與初始設定同一種寫法）；Tour 的步驟與文案不動；desktop 除麵包屑與 WorkspaceChooser 步驟列換用共用元件外不動。

## Risks / Trade-offs

- [第二、三刀尚未落地前，頁面是新殼加舊內容] → 標題區與殼已是新樣子，下方仍是 DataList 與抽屜；兩刀依序轉出，中間狀態可用、不會壞，只是不完全一致。
- [帳號選單把登出藏進一層選單] → 選單觸發鈕常駐側欄底部並顯示自己的名稱與 email，比原本右上角的 email 連結更容易辨認「我是誰」；登出只多一次點擊，與 desktop 和常見後台一致。
- [移除 16px／44px 承諾降低觸控友善度] → 窄螢幕版仍保留完整功能與鍵盤操作；共用按鈕本來就不是 44px，改寫只是讓規格說真話。
- [麵包屑與步驟列搬家動到 desktop] → 只換渲染來源、不換外觀（麵包屑按鈕只多鍵盤聚焦時的 focus ring）；titleBar.test.tsx 與 workspaceChooser.test.tsx 守住現有行為，desktop 全套測試跟著跑一次。
- [回歸對照] → 本變更不碰 CLI 與 golden 檔，`cargo test` 與 CLI golden 不受影響；前端以 vitest 與 vite build 為準（前端 `tsc --noEmit` 基準本來就紅、不是守門）。
- [跨平台] → 純瀏覽器畫面，Windows／macOS／Linux 差異只在字型渲染；共用 theme 已處理字型堆疊。desktop 麵包屑在三平台的拖曳區設定不變。

## Migration Plan

純前端改版，隨 server 版本一起出貨（server 內嵌 server-web 的 build 產物）。不需資料遷移；語言偏好沿用同一個瀏覽器儲存鍵。回滾就是回到前一版 server。

## Open Questions

（無）原型未畫的四頁（專案與儲存庫、憑證、系統、稽核紀錄）本刀只換標題區，說明文字已在 D5 定稿；其內容版面由第二刀決定。

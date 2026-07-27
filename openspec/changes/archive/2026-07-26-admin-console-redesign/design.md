## Context

`web-service-navigation-redesign` 已把 Speclink Server 的 browser 面換成 Vite + React + TypeScript SPA（`apps/server-web`），並復用 `packages/ui` 的 shadcn/ui 原語與共用 semantic theme。該變更解決的是「端點必須手打 URL」，未處理版面組成與互動模式。

現況三個結構性事實決定本設計的形狀：

1. **視覺 token 已同源**——`apps/server-web/src/index.css` 與 `apps/desktop/src/index.css` 皆匯入 `packages/ui/src/theme.css`。因此「與 Desktop 風格一致」的工作不在色票，而在版面數值、殼結構與互動語彙。
2. **`packages/ui` 已匯出 Sheet 與 AlertDialog，但沒有 Dialog**。Desktop 的細節與動作本來就在右側 Sheet（`RichDetailDrawer`），所以「編輯進抽屜」是零新相依且與 Desktop 同語彙的路徑。
3. **管理面的資料全部經 `/api/speclink/v1/web` same-origin JSON API**，由 `crates/speclink-server/src/admin.rs` 組裝 per-page view model。頁面合併與新增區塊會連動 view model，而非只是前端重排。

限制：本變更不得觸碰認證、授權、CSRF、secret 邊界與 audit 來源標記；那些契約由 `server-admin`、`server-identity` 既有需求釘死，本設計只重排呈現與 view model 形狀。

## Goals / Non-Goals

**Goals:**

- 管理員在任何頁面（含帳號頁）都保有全站導覽，不需依賴瀏覽器上一頁。
- 列表頁成為主體：建立、邀請與編輯退到抽屜，表格列不含輸入控制項。
- 消除 Store 健康的兩份真相：資料操作與系統狀態合併為單一目的地與單一 view model。
- 總覽可行動：指標可點入對應頁，並呈現待辦、系統健康摘要與最近稽核。
- 版面數值與 Desktop 殼一致（header 高度、側欄寬度、主內容內距）。
- 使用者可見文案清除工程詞，新詞條寫入 `openspec/LANGUAGE.md`。
- 表單控件全部出自同一套 shadcn/ui 原語，高度與 focus 行為一致。
- 管理員第一次進入管理面時有分步導覽，之後不再打擾。
- Server 管理面與 Desktop 一樣支援 zh-TW 與 en。

**Non-Goals:**

- 不變更任何認證、授權、session、CSRF 或 secret 邊界。原本一併排除「不新增 domain action」，但 2026-07-25 走查後決定納入撤回邀請——邀請一旦寄錯人或寄錯權限，介面上沒有任何止血手段（見下方決策）。除此之外仍不新增 domain action。
- 不變更 CLI 子指令、旗標、stdin 契約、exit code 或 `--json` 輸出。
- 不變更 bearer API 與 Client Protocol。
- 不把殼或導覽元件抽到 `packages/ui` 共用。
- 不新增 Radix **dropdown menu** 相依（header 帳號入口仍是連結加按鈕，不是選單）。Radix **Select** 是另一回事——它取代的是既有的原生 `<select>`，見下方決策。
- 不在 SPA 內加入 changes、specs 或 discussions 的檢視與編輯。
- 不修 Desktop 抽屜長標題造成水平捲軸的缺陷（獨立變更處理）。
- 不加入深色模式切換，或本次未列出的新管理功能。
- 導覽教學不做多步驟分支、不做進度回報、不存伺服器端狀態。
- 多語系不做語言的第三種選項、不翻譯 audit 的動作字串與 store 錯誤訊息（那是伺服器的原樣資料）。

## Decisions

### 以單一 ConsoleLayout 承載管理員與一般成員，依角色裁切側欄

`/account` 目前一律套用沒有側欄的 `AccountLayout`，管理員從側欄的「帳號」進入後即失去全部導覽。改為單一 `ConsoleLayout`：header 恆常呈現，側欄僅在 session 帶 admin 旗標時渲染。管理員在 `/account` 時側欄整條保留但無項目高亮（因帳號已不是側欄目的地）；一般成員完全不渲染側欄，視覺結果與現行帳號殼等價。`AccountLayout` 移除，`FocusLayout`（登入、初始設定、邀請、裝置核准）維持不變——那些流程刻意不該有導覽。

側欄可見性以伺服器回傳的 session 角色決定，沿用既有 `home` 與 admin 旗標，前端不自行推導安全敏感結果；非管理員即使手打 `/admin/*` 仍由既有 gate 回 403。

替代方案：(a) 在帳號頁加返回按鈕——治標，管理員在該頁仍失去其餘六個目的地；(b) 保留兩個 layout 各自維護導覽——同一份側欄語彙兩份實作，正是目前 `AdminNav` 與 Desktop `NavItem` 已經漂移的成因。

### 建立與編輯一律以 Sheet 抽屜承載，破壞性動作維持 AlertDialog

列表頁的常駐表單全部移入右滑 Sheet：`＋ 邀請使用者`、`＋ 建立專案`、`＋ 新增儲存庫`、`＋ 加入專案`，以及使用者與專案的細節編輯。表格列只呈現資料，整列可點開細節抽屜。停權、撤銷憑證、移除成員資格、刪除專案與資料結構遷移維持既有 AlertDialog 確認，符合 `server-web-console` 既有的「破壞性操作送出前顯示確切對象並要求確認」需求。

抽屜實作以 `apps/server-web/src/components/DetailSheet.tsx` 收斂共用結構（標題列、關閉鈕、可選分頁、動作列、內容捲動區），內部使用 `packages/ui` 既有的 Sheet 與 Tabs 原語。窄螢幕時抽屜改為全寬。

替代方案：(a) 新增 Dialog 原語承載建立表單——`packages/ui` 目前沒有 Dialog，引入後 Server 面出現「彈窗建立、抽屜檢視」兩套心智模型，而 Desktop 只有抽屜一套；(b) 保留常駐表單只調整間距——問題在資訊層級不在密度。

### 系統頁合併為單一 view model，資料操作目的地移除

「資料操作」（Store 狀態、Scope 匯出、遷移）與「系統狀態」（引擎與 API 版本、Store 驅動／契約／等級／能力／健康、Outbox backlog）各自輸出一份 Store 健康描述。合併為 `/admin/system` 單一目的地，區段為執行環境、儲存狀態、匯出、危險區。

view model 在 `crates/speclink-server/src/admin.rs` 合併為一支：既有兩個 handler 的欄位併入同一個 struct，欄位維持 `#[serde(rename)]` 的 camelCase 輸出。前端不同時呼叫兩支 API——那會讓「每頁一個最小且完整 view model」的既有需求形同虛設，且兩支回應的 Store 健康可能在同一畫面上互相矛盾。

替代方案：前端保留兩支呼叫僅合併呈現——省下 Rust 改動，但把「兩份真相」從兩個頁面搬到同一個頁面，沒有解決問題。

### 稽核篩選與分頁在伺服器端執行

稽核紀錄新增動作、來源、時間區間篩選與分頁。篩選與分頁參數由 query string 傳入 browser API，由伺服器套用後回傳當頁資料與總頁數；前端只呈現。稽核事件會隨營運時間單調增長，前端全量載入再篩選會讓頁面隨資料量線性劣化，且與既有「每個 route 表示 loading、success、empty」的狀態契約衝突。

替代方案：前端全量載入後在記憶體篩選——實作最短，但在事件累積後不可用，且違反 view model「最小且完整」的既有需求。

### 帳號入口放在 header，以電子郵件連結加登出按鈕呈現

「帳號」自側欄移除，改由 header 右上呈現當前使用者的電子郵件（連結至 `/account`，在該頁高亮）與登出按鈕。一併解決現行 header 只有一顆登出、看不出登入身分的問題。

不採用下拉選單：`packages/ui` 沒有 dropdown 原語，為兩個項目引入 `@radix-ui/react-dropdown-menu` 不划算，且下拉會把兩個常用動作各多藏一次點擊。

替代方案：新增 dropdown 原語做使用者選單——項目增長到三個以上時再考慮，現在屬於過度設計。

### 不把殼與導覽抽到 packages/ui 共用

Desktop 與 Server 的 header 內容本質不同（workspace 分頁與新增 workspace 對比身分與登出），共用殼只會是轉發版面的空層——刪掉它兩端各留自己的殼，沒有任何行為會壞。一致性改以兩項明確手段維持：沿用已同源的 `packages/ui/src/theme.css`，以及在本設計固定側欄寬度、header 高度與主內容內距三個數值採用 Desktop 殼的值。

`NavItem` 的 active／inactive class 矩陣確實有共用價值（那正是目前唯一實際漂移的部分），但它跨越 Tauri 與 web 兩個組建邊界只為省下十餘行樣式，本次不抽，列為 Open Questions。

替代方案：抽 `AppShell` 至 `packages/ui`——未通過刪除測試；抽 `NavItem`——價值真實但非本次必要，留待重複出現第三次再處理。

### 不可變代號以唯讀樣式呈現，更名採顯式編輯模式

專案與儲存庫代號建立後不可變更，目前卻與可改的名稱共用同一種輸入框樣式並常駐可編輯，會誤導管理員以為代號可改。代號改以唯讀文字呈現並標註「建立後不可變更」；名稱改為預設唯讀，按下「更名」才出現輸入框與確認／取消。

替代方案：把代號輸入框設為 disabled——仍是輸入框外觀，暗示「某些條件下可改」，語意錯誤。

### 總覽 view model 增列待辦、系統健康摘要與最近稽核

總覽由六個純數字改為四張可點入對應頁的指標卡（使用者、專案、憑證、待啟用），並新增三個區塊：需要處理（無事項時整塊不渲染，不留空殼）、系統健康摘要（連往系統頁）、最近活動（連往稽核頁）。對應 view model 增列這三組資料，資料來源沿用既有 identity、registry、store 與 audit 查詢，不新增 domain action。

替代方案：前端分別呼叫使用者、憑證、系統、稽核四支 API 組出總覽——首屏四次往返，且與「每頁一個 view model」的既有需求相違。

### 文案收斂與 LANGUAGE.md 同步在同一個變更內完成

工程詞退場與 `openspec/LANGUAGE.md` 新增詞條同批進行，避免詞彙定義落後於介面。涉及的皆為尚未收錄的新詞條，不改動既有正典詞條，因此不影響歷史 artifacts。

替代方案：先改介面、之後補詞彙表——歷史經驗是「之後」不會到來，介面與詞彙表會長期分歧。

### 下拉選單改用 Radix Select，`NativeSelect` 移除

`packages/ui` 的 `NativeSelect` 是 shadcn 樣式包在原生 `<select>` 外面。它與同套 `Input`（`h-9`）高度不同（`h-8`），展開後的選單完全由作業系統繪製，focus ring、選中態與圓角都跟不上 theme——工具列把搜尋框與狀態篩選並排時一眼看得出參差。

改為 `@radix-ui/react-select` 實作的 `Select`，Desktop 與 Server 兩端一起換、`NativeSelect` 自匯出移除。留兩套下拉會讓「這個頁面該用哪一種」變成每次都要判斷的問題，而兩者的鍵盤與樣式行為並不等價。

代價明確承認：Radix Select 在 jsdom 需要 `hasPointerCapture`、`releasePointerCapture` 與 `scrollIntoView` 的 stub（放進兩個 app 既有的 `vitest.setup.ts`），且既有以 `selectOptions` 驅動原生 select 的測試要改寫為點開選單再點選項。

替代方案：(a) 只換 Server 面——codebase 同時存在兩種下拉寫法，Desktop 的設定頁與看板永遠是另一套；(b) 只把 `NativeSelect` 高度改成 `h-9`——解決了對齊，沒解決展開樣式與 theme 脫節。

### 撤回邀請以刪除邀請實作，是本變更唯一新增的 domain action

邀請連結一旦交出去就在對方手上，寄錯人或寄錯權限時介面上原本沒有任何止血手段——只能等它過期。這是唯一足以推翻「不新增 domain action」這條 Non-Goal 的缺口，因此納入。

實作為刪除該筆邀請與其 memberships，而非加一個 `revoked_at` 欄位：新增欄位要動 identity schema 與 migration，而未接受的邀請除了稽核之外沒有歷史價值。`consumed_at` 的語意固定是「已被接受」，借用它會讓接受路徑的判準跟著失真。刪除與稽核寫入在同一個交易內，與其他 `admin_*` 動作同構。

只認未接受的邀請：已接受的那筆背後已經有真實帳號，該走停權而不是回收邀請，因此回 `NotFound`（HTTP 404）。稽核事件 `invitation-revoked` 以受邀者 email 為 subject，token 與其 hash 不進稽核。

介面用詞為「撤回邀請」而非「取消邀請」：確認框裡的取消鈕代表「放棄這個操作」，兩者並排會分不清哪顆是哪顆。

替代方案：(a) 標記 `revoked_at`——要 schema migration，換來的只是保留一筆沒人會查的死資料；(b) 不做，等邀請自然過期——寄錯權限的連結會在有效期內一直可用。

### 待啟用邀請併入使用者 view model，而非新增一支 API

受邀者接受前沒有 user row，因此使用者頁查不到剛邀請的人——而總覽的「待啟用」指標正是連往這一頁。識別資料層已有 `count_pending_invitations`，改為同一組「未使用且未過期」判準再加一支 `list_pending_invitations`，由既有的 `/admin/users` view model 一併回傳。

不另開 `/admin/invitations`：那會讓使用者頁首屏兩次往返，且與「每頁一個最小且完整 view model」的既有需求相違。

前端不把待啟用者混進使用者表格：他們不能被停權、沒有憑證也沒有細節抽屜可開，混列會讓整列可點的語彙出現例外。改為列表下方的獨立區塊，無邀請時整塊不渲染。

替代方案：(a) 前端另打一支邀請 API——多一次往返；(b) 混列並在列上標示待啟用——列的可點語彙出現例外，且 `DataList` 要為此長出 per-row 的可選性。

### modal 容器內的浮層 portal 進容器自身

Radix Select 的選單預設 portal 到 `document.body`。在 Sheet（modal Dialog）內部，body 是 focus trap 的外面：Dialog 的 FocusScope 把焦點拉回 content，Select 把焦點送去它 portal 出去的選單，兩邊對同一次 focus 事件互推——在 jsdom 直接爆堆疊，瀏覽器則是焦點行為難以預期。

改為由 `SheetContent` 以 context 提供自己的 content 節點（`packages/ui/src/components/ui/portal-container.tsx`），`SelectContent` 在有 provider 時 portal 進去，沒有時維持 body。代價是抽屜內的選單受抽屜的 `overflow-y-auto` 影響，過長清單會在抽屜內捲動而非覆蓋到抽屜外——對窄抽屜內的短清單可以接受。

替代方案：(a) 一律不 portal——工具列的下拉會被 `<main>` 的 overflow 裁切；(b) 把 Sheet 改成非 modal——失去 focus trap，違反既有的鍵盤與無障礙契約。

### 首次導覽以疊層分步呈現，狀態存在瀏覽器

管理員第一次開啟 `/admin` 時啟動分步導覽，依序指向側欄的六個目的地與列表頁的 primary action，說明「列表為主體、編輯進抽屜」這個貫穿全站的語彙。任一步可略過；走完或略過即記為看過，之後不再自動出現，並在系統頁保留「重看導覽」入口。

「看過了」是瀏覽器端偏好而非伺服器真相——它不影響任何授權或資料，存 `localStorage` 即可（鍵 `speclink.tourSeen`）。存伺服器要新增 identity 欄位、migration 與 API，為了一個一次性提示不值得。代價是換瀏覽器會再看到一次；對一個「教你怎麼用」的提示，這個代價可以接受。

不引入 `driver.js`：它自行接管 DOM 與捲動，與 Radix 的 focus trap、既有的 `useFocusMain` 會互相搶焦點。改以既有的 Radix `Popover`／絕對定位疊層自製——導覽只有「高亮一個目標、講一句話、上一步／下一步／略過」，不值得為此背一個會動 DOM 的相依。

替代方案：(a) 引入 `driver.js`——相依小但焦點行為衝突難查；(b) 改做一頁式的「開始使用」說明頁——讀完仍要自己對應到畫面上的哪一塊，正是導覽要消除的那一步。

### 多語系沿用 Desktop 的機制，locale helper 提升為共用

Desktop 已有完整的一套：`packages/ui` 的 `I18nProvider`／`useI18n`／`MESSAGES`，加上 `apps/desktop/src/i18n/locale.ts` 的偏好解析（明示偏好優先、`null` 跟隨系統、存 `localStorage` 鍵 `speclink.uiLocale`）與 `apps/desktop/src/i18n/messages.ts` 的 app 級字典。Server 面照抄同一個形狀：新增 `apps/server-web/src/i18n/messages.ts`，`App.tsx` 以解析後的 locale 掛 `I18nProvider`。

`locale.ts` 的四個函式與 app 無關，且即將有第二個消費者——提升到 `packages/ui/src/locale.ts` 共用，Desktop 改為 re-export 以免動到它既有的 import 面。抄第三份是這類工具漂移的起點。

語言切換放在 header（與電子郵件連結、登出同一列），三選：中文／English／跟隨系統。

既有的 `wording.test.tsx` 斷言的是中文字面，多語系後那些字面只在 zh-TW 成立——該測試改為「在 zh-TW 下」斷言，並增列「en 下不出現中文」的對照，兩種語言的 key 集合相等由字典測試保證（與 Desktop 的 `i18n.test` 同款）。

替代方案：(a) 引入 `react-i18next`——字串規模與 Desktop 同級（數百條），既有自製方案已證明夠用，多一個執行期相依不划算；(b) 只翻譯管理面、帳號頁維持中文——同一個殼裡兩種語言，比全中文更糟。

## Implementation Contract

#### 行為

- 管理員登入後，於任一管理頁面與 `/account` 皆可見完整側欄；`/account` 時側欄無任何項目高亮，header 的電子郵件連結呈現高亮。
- 一般成員登入後於 `/account` 不見側欄，僅有 header（電子郵件連結與登出）與內容。
- 側欄提供六個目的地：總覽、使用者、專案與儲存庫、憑證、系統、稽核紀錄。`/admin/data` 不再是目的地。
- 使用者頁、專案與儲存庫頁、憑證頁、稽核頁的表格或卡片列不含任何輸入控制項；點整列開啟細節抽屜。
- 建立與邀請動作以抽屜開啟；提交成功後抽屜關閉、列表更新並以 `aria-live=polite` 回饋；提交失敗保留輸入與原頁資料。
- 專案與儲存庫代號在介面上不可編輯；名稱預設唯讀，按「更名」後才可輸入。
- 總覽四張指標卡皆可點入對應頁；「需要處理」在無事項時整塊不渲染。
- 系統頁單頁呈現執行環境、儲存狀態、匯出與危險區；Store 健康在整個介面只出現一處權威來源。
- 稽核頁支援動作、來源與時間區間篩選與分頁，篩選結果由伺服器計算。
- 小於 1024px 時表格轉為卡片列且頁面不產生水平捲動，抽屜改為全寬。
- 殼本身不捲動：只有主內容區捲，header 與側欄恆常留在畫面上。
- 介面上所有下拉選單皆為 `packages/ui` 的 `Select`；同一列工具列中的搜尋框、下拉與日期欄高度、內距、圓角與陰影相同。下拉選單展開後為不透明底色，且在抽屜內可正常開啟與選取。
- 邀請時挑選要加入的專案以下拉逐一挑選，已選項目列在下方可逐一移除；不為每個專案渲染常駐勾選框。
- 帳號頁的建立存取金鑰與管理列表頁同構：頁首單一 primary action 開抽屜，成功後抽屜關閉並在頁面呈現一次性明文與複製動作。
- 憑證頁的建立存取金鑰入口常駐頁首，不因清單非空而消失。
- 邀請送出後以受邀者可直接開啟的連結回饋（`{origin}/invite/{token}`）並附複製動作，不呈現裸 token。
- 使用者頁在使用者列表之外另列仍有效的邀請，含受邀者、角色、要加入的專案與到期時間；無邀請時整塊不渲染。
- 每筆待啟用邀請提供撤回動作，經 AlertDialog 指名受邀者確認後才送出；撤回後該連結立即失效。
- 管理員第一次開啟 `/admin` 時自動啟動導覽；每一步指向一個實際存在的畫面元素並附一句說明，說明卡片依該元素的位置擺放（優先右側，放不下改下方或上方，座標夾回視窗內），提供上一步／下一步／略過。走完或略過後重新整理不再自動啟動；系統頁提供「重看導覽」。
- header 提供介面語言切換（中文／English／跟隨系統）；切換即時生效並持久化，重新整理後維持。
- 未設定語言偏好時跟隨瀏覽器語言：`zh` 開頭為 zh-TW，其餘為 en。

#### 介面與資料形狀

- browser API 路徑與 envelope 沿用既有契約：成功 `{data: T}`、錯誤 `{error:{code,message,fieldErrors?}}`，欄位 camelCase（Rust struct 以 `#[serde(rename)]` 輸出）。
- 系統頁 view model：由原「資料操作」與「系統狀態」兩個 view model 的欄位合併為單一 struct，涵蓋引擎與 API 版本、識別資料結構版本、儲存後端驅動／契約版本／等級／能力／健康、待送佇列、可匯出範圍清單與遷移可用性。
- 總覽 view model：既有計數欄位保留，增列待辦項目清單、儲存健康摘要與最近稽核事件清單。
- 稽核 view model：接受動作、來源、時間區間與頁碼參數，回傳當頁事件與總頁數。
- 使用者 view model：增列建立時間（`createdAt`，RFC3339）與待啟用邀請清單（`pending`，每筆含 `id`／`email`／`display`／`admin`／`memberships`／`createdAt`／`expiresAt`，一律不含 token 或其 hash）。列表需呈現建立日期，而識別資料層原本未把 `users.created_at` 讀進 `User`；此欄位僅供顯示，不參與任何認證或授權判斷。
- 上述皆為新增或合併欄位，既有欄位名稱不重新命名，維持 serde 反序列化的向後相容。
- 不新增 CLI 子指令、旗標或 exit code；不變更 bearer API 與 Client Protocol 的任何型別。
- `@speclink/ui` 匯出 `Select`／`SelectTrigger`／`SelectValue`／`SelectContent`／`SelectItem`（shadcn 命名），並移除 `NativeSelect` 匯出。
- 浮層底色使用 `bg-card`：這份 theme 沒有 `--popover`，用上游 shadcn 的 `bg-popover` 會產出解析不到值的宣告而讓選單透明。
- `@speclink/ui` 匯出 `UiLocale`、`LocalePreference`、`detectSystemLocale`、`readLocalePreference`、`writeLocalePreference`、`resolveUiLocale`；`apps/desktop/src/i18n/locale.ts` 改為自該處 re-export，其對外簽名不變。
- `apps/server-web/src/i18n/messages.ts` 匯出 `APP_MESSAGES: Record<UiLocale, Record<string, string>>`；zh-TW 與 en 的 key 集合必須完全相等。
- 導覽的「看過了」狀態：`localStorage` 鍵 `speclink.tourSeen`，值為 `"1"`；缺鍵或任何其他值都視為未看過。語言偏好沿用既有鍵 `speclink.uiLocale`。
- 導覽與語言切換都不呼叫任何 browser API，不新增 view model 欄位。

#### 失敗模式

- 未登入存取受保護 route：沿用既有導向登入頁並帶白名單 `returnTo` 的行為。
- 非管理員存取 `/admin/*`：沿用既有 403，SPA 呈現無權限狀態，不降級導向 `/account`。
- Store 不健康：系統頁與總覽的健康摘要明確標示降級狀態，身分管理功能仍可用（沿用既有「Store 不健康降級」需求）。
- 抽屜內提交失敗：抽屜保持開啟、保留非祕密輸入、錯誤置於對應欄位附近並以 `role=alert` 宣告。
- 稽核篩選無結果：呈現 empty 狀態而非空白表格。
- 稽核參數不合法：頁碼小於 1 或時間區間起始晚於結束回 400 `invalid_argument`；頁碼超出總頁數不是錯誤，回空事件清單與正確總頁數。
- 合併後的系統 view model 取得失敗：整頁呈現 unexpected error 與重試入口，不部分渲染陳舊資料。
- 導覽某一步的目標元素不存在（例如該角色看不到側欄）：跳過該步繼續，不中斷整個導覽、不丟例外。
- 目標貼邊或視窗過小導致說明卡片放不下：座標夾回視窗內，寧可略微偏移也不讓卡片跑出畫面。
- `localStorage` 不可用（隱私模式或被停用）：導覽照常運作、語言切換照常生效，只是不持久化——讀寫一律以 try 包住，失敗視為「未設定」。
- 訊息 key 在字典中缺漏：`useI18n` 的既有行為是回傳 key 本身，不丟例外也不回退到另一語言。

#### 驗收標準

- `npm test -w apps/server-web` 全綠，且測試涵蓋：管理員於 `/account` 仍見側欄、一般成員於 `/account` 不見側欄、側欄恰有六個目的地、`/admin/data` 不再可由導覽到達、列表列不含輸入控制項、抽屜開關與提交失敗保留輸入、代號欄位不可編輯、總覽無待辦時不渲染該區塊。
- `cargo test -p speclink-server` 全綠，且測試涵蓋：系統 view model 單次回應即含執行環境與儲存與匯出與遷移四組資料、總覽 view model 含待辦與健康摘要與最近稽核、稽核 view model 依參數篩選與分頁、既有權限與 secret exclusion 測試未退化。
- `npm run test:all` 與 `cargo test --workspace` 全綠，確認 CLI 人眼輸出與 `--json` 回歸對照未受影響。
- 以真實瀏覽器在 375、768、1024、1440px 檢視六個管理目的地與帳號頁，確認無水平捲動、鍵盤可完成主要流程、focus ring 可見。
- `openspec/LANGUAGE.md` 收錄本變更列出的全部新詞條，且介面上不再出現對應的工程詞。
- `npm test -w packages/ui` 全綠，且測試涵蓋：`Select` 可由鍵盤開啟與選取並回報選中值、`NativeSelect` 已不再匯出。
- `npm test -w apps/desktop` 全綠，證明 `Select` 替換與 locale helper 提升未改變 Desktop 行為。
- `npm test -w apps/server-web` 全綠，且測試涵蓋：首次進入 `/admin` 自動啟動導覽、略過後重新整理不再啟動、「重看導覽」可再次啟動、目標元素缺席時該步被跳過；語言切換後管理面文案改為英文、重新整理後維持、未設定偏好時跟隨 `navigator.language`；zh-TW 與 en 字典 key 集合相等。
- 以真實瀏覽器確認：整頁不捲動而主內容區捲動、工具列控件高度一致、導覽疊層不遮住它正在指的元素。

#### 範圍邊界

**In scope**：`apps/server-web` 的殼、導覽、七個頁面（其中兩頁合併為一頁）與帳號頁的版面與互動，含其 `index.css` 的 reduced-motion 守門；`crates/speclink-server` 的 `admin.rs` view model 組裝，以及撤回邀請與待啟用清單所需的 `identity.rs`／`identity_sqlite.rs`／`audit.rs` 改動，與 `assets.rs` 的 browser route allowlist 調整；`openspec/LANGUAGE.md` 的新詞條；`packages/ui` 的 `Select` 原語、modal 浮層的 portal container 與 locale helper 提升；`apps/desktop` 因這些提升而必要的替換與 import 調整。

**Out of scope**：認證與授權邏輯、撤回邀請以外的 domain action、audit 來源標記、CLI、bearer API、Client Protocol、`packages/ui` 的殼抽取、Desktop 的版面與功能變更（含抽屜水平捲軸缺陷）、SPA 資產內嵌與 CSP 等交付邊界、未登入流程（登入／初始設定／邀請／裝置核准）的導覽教學。

## Risks / Trade-offs

- **[排序相依：`server-web-console` 正典規格尚未存在]** → 該 capability 由 `web-service-navigation-redesign` 建立，該變更尚餘終驗未封存。本變更的 delta 以其為基準；實作與封存須排在其後，否則 MODIFIED 找不到基準需求。
- **[合併 view model 可能牽動既有 server 測試]** → 合併只是欄位聚合，既有欄位名稱不重新命名；先以 RED 測試釘住新形狀，再刪除舊 handler，避免中途兩支並存。
- **[表格列改為整列可點，可能與列內殘留的連結或按鈕搶焦點]** → 契約明訂列內不得含輸入控制項；憑證頁的撤銷入口置於列尾的明確動作按鈕，並確保鍵盤先到列本身再到動作。
- **[抽屜承載表單後，鍵盤與螢幕閱讀器路徑改變]** → 沿用 Radix Sheet 的 focus trap 與關閉後歸還 focus；驗收明列鍵盤走查。
- **[文案大量改動可能打破既有前端測試的文字查詢]** → 文案收斂與各頁改造放在同一批任務內完成，測試與文案同步更新，不留跨任務的紅燈期。
- **[跨平台]** → 本變更只動 Server 的 browser 面與 `speclink-server`，不涉及 git 互動、檔案系統版面或 OS 專屬 API，無跨平台差異風險；Desktop 不受影響。
- **[回歸對照]** → CLI 人眼輸出與 `--json` 完全不動，parity 對照不受影響；browser API 僅供同源 SPA 使用並隨 binary 一同交付，無外部使用者需要遷移。

- **[Radix Select 替換牽動兩個 app 的既有測試]** → 以 `selectOptions` 驅動原生 select 的測試會全數失效。先在兩個 app 的 `vitest.setup.ts` 補齊 jsdom stub，再逐檔改寫查詢方式；`npm test -w apps/desktop` 全綠即為未退化的證據。
- **[多語系會讓既有的中文字面測試失效]** → `wording.test.tsx` 改為明示在 zh-TW 下斷言，並增列 en 對照；兩語言 key 集合相等由字典測試釘住，避免只翻一半。
- **[modal 內的浮層 portal 目標]** → Select 選單改 portal 進 Sheet content 後，抽屜的 `overflow` 會裁切過長清單。抽屜內的下拉都是短清單（專案、角色），且 `packages/ui` 有一支測試釘住「抽屜內的下拉可開啟並選取」。
- **[導覽疊層與 Radix focus trap 互搶焦點]** → 導覽只在沒有任何 Sheet／AlertDialog 開啟時啟動，且自身不使用 focus trap；驗收明列「導覽進行中仍可用鍵盤離開」。

## Open Questions

- `NavItem` 是否值得抽到 `packages/ui` 共用？目前 Desktop 與 Server 各有一份 active／inactive class 矩陣，已實際漂移。本次不抽（跨組建邊界只為省十餘行樣式）；待第三處出現相同語彙時重新評估。

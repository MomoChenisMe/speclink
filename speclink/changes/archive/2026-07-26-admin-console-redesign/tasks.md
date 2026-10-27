## 1. 主控台殼、導覽與帳號入口

- [x] 1.1 RED：先寫失敗測試釘住「全部 browser route 由單一 SPA 提供可發現導覽」與「共用設計系統維持高密度可存取體驗」在殼層的新契約——管理員於 `/account` 仍見完整側欄且無項目高亮、一般成員於 `/account` 不渲染側欄、側欄恰有總覽／使用者／專案與儲存庫／憑證／系統／稽核紀錄六個目的地且不含資料操作與帳號、header 呈現當前使用者電子郵件連結與登出、768px 時 Sheet 內同為六個目的地。測試放在 `apps/server-web/src/__tests__/`，以 `npm test -w apps/server-web -- admin account` 執行，預期全部因現行 `AdminLayout` 與 `AccountLayout` 行為而失敗。 <!-- speclink-task:tsk_01KYBTV8PW7AA08J6MC2J1WAYJ -->
- [x] 1.2 GREEN／REFACTOR：依 design 決策「以單一 ConsoleLayout 承載管理員與一般成員，依角色裁切側欄」、「帳號入口放在 header，以電子郵件連結加登出按鈕呈現」與「不把殼與導覽抽到 packages/ui 共用」，在 `apps/server-web/src/layouts/ConsoleLayout.tsx`、`apps/server-web/src/components/HeaderAccount.tsx`、`apps/server-web/src/components/AdminNav.tsx` 與 `apps/server-web/src/routes/AppRoutes.tsx` 實作單一殼與六目的地導覽，移除 `apps/server-web/src/layouts/AccountLayout.tsx`，並把 header 高度、側欄寬度與主內容內距改為 Desktop 殼相同數值；側欄可見性只依伺服器回傳的 admin 旗標決定，不在前端推導。以 1.1 測試轉綠、既有 `npm test -w apps/server-web` 全綠驗證，並確認非管理員存取 `/admin/*` 仍為 403。 <!-- speclink-task:tsk_01KYBTV8PWF5RVQXHQQEK520N8 -->

## 2. 抽屜基座與使用者頁

- [x] 2.1 RED：先寫失敗測試釘住「管理列表以抽屜承載建立與編輯」——使用者列表列只呈現使用者、狀態、角色、成員資格與建立日期且不含下拉或提交按鈕、點整列開啟含概要／成員資格／憑證／稽核的細節抽屜、邀請欄位只在按下邀請動作後出現於抽屜、抽屜提交收到含 `fieldErrors` 的 400 時保持開啟並保留已輸入的電子郵件與顯示名稱且錯誤以 `role=alert` 宣告、停權與撤銷仍走 AlertDialog 確認。以 `npm test -w apps/server-web -- users` 執行，預期因現行常駐邀請表單與列內控制項而失敗。 <!-- speclink-task:tsk_01KYBTV8PWK2T5PAF2TAHNBTW3 -->
- [x] 2.2 GREEN／REFACTOR：依 design 決策「建立與編輯一律以 Sheet 抽屜承載，破壞性動作維持 AlertDialog」，新增 `apps/server-web/src/components/DetailSheet.tsx` 收斂抽屜結構（標題列、關閉、分頁、動作列、內容捲動區，窄螢幕全寬），改寫 `apps/server-web/src/pages/admin/UsersPage.tsx` 使列表無輸入控制項、細節與邀請皆由抽屜承載，成員資格於抽屜內新增與移除。以 2.1 測試轉綠與 `npm test -w apps/server-web` 全綠驗證，並手動確認抽屜關閉後 focus 回到觸發元素。 <!-- speclink-task:tsk_01KYBTV8PWRKVP9JRTQWCV9EW4 -->

## 3. 專案與儲存庫頁

- [x] 3.1 RED：先寫失敗測試釘住「不可變識別欄位唯讀且更名為顯式動作」——專案與儲存庫代號以唯讀文字呈現且畫面上不存在可編輯代號的輸入框、名稱預設為唯讀文字、按下更名後才出現輸入框與確認及取消、建立專案與新增儲存庫皆由抽屜承載而非常駐表單。以 `npm test -w apps/server-web -- registry` 執行，預期因現行常駐更名輸入框與常駐建立表單而失敗。 <!-- speclink-task:tsk_01KYBTV8PW9RHJ082WQVPH71J7 -->
- [x] 3.2 GREEN／REFACTOR：依 design 決策「不可變代號以唯讀樣式呈現，更名採顯式編輯模式」改寫 `apps/server-web/src/pages/admin/RegistryPage.tsx` 為卡片列表加細節抽屜，代號標示建立後不可變更，名稱採顯式編輯模式，建立與新增動作復用 `DetailSheet`。以 3.1 測試轉綠與 `npm test -w apps/server-web` 全綠驗證。 <!-- speclink-task:tsk_01KYBTV8PWQZG6JPMQZE1PDSAG -->

## 4. 憑證頁、稽核頁與清單能力

- [x] 4.1 RED：先寫失敗測試釘住「管理列表提供搜尋、篩選、分頁與具引導的空狀態」與「稽核篩選與分頁在伺服器端執行」——憑證頁以存取金鑰與裝置兩個分頁呈現並提供關鍵字搜尋與狀態篩選、稽核頁提供關鍵字與動作與來源與時間區間篩選及分頁控制項、SPA 以篩選與頁碼參數呼叫 browser API 且只呈現伺服器回傳的當頁事件與總頁數、篩選無結果時呈現空狀態並保留篩選控制項、空憑證頁說明用途並提供建立存取金鑰的 primary action。同時在 `crates/speclink-server/tests/` 寫失敗測試斷言 audit view model 依關鍵字／動作／來源／時間區間／頁碼參數於伺服器端篩選與分頁，回應為 camelCase 且未符合篩選的事件不出現，並涵蓋參數邊界：頁碼小於 1 與時間區間起始晚於結束回 400 `invalid_argument`、頁碼超出總頁數回空清單與正確總頁數、未知動作名稱回空清單與總頁數 0、全部篩選省略回第一頁全部事件。以 `npm test -w apps/server-web -- credentials audit` 與 `cargo test -p speclink-server` 執行，預期全部失敗。 <!-- speclink-task:tsk_01KYBTV8PW0C8JPB8JPWSCK84K -->
- [x] 4.2 GREEN／REFACTOR：在 `crates/speclink-server/src/admin.rs` 實作 audit view model 的伺服器端篩選與分頁參數，並改寫 `apps/server-web/src/pages/admin/CredentialsPage.tsx`、`apps/server-web/src/pages/admin/AuditPage.tsx`、`apps/server-web/src/api/client.ts` 與新增 `apps/server-web/src/components/ListToolbar.tsx`、`apps/server-web/src/components/EmptyState.tsx`；憑證撤銷入口置於列尾明確動作按鈕，鍵盤順序先到列本身再到動作。以 4.1 測試轉綠、`cargo test -p speclink-server` 與 `npm test -w apps/server-web` 全綠驗證，並確認既有 secret exclusion 與權限測試未退化。 <!-- speclink-task:tsk_01KYBTV8PWP6KVJ5H71FQQ1FH3 -->

## 5. 系統頁合併與 view model

- [x] 5.1 RED：先寫失敗測試釘住「管理 browser API 提供最小且完整的頁面 view model」的合併形狀——system view model 單次回應同時含引擎與 API 版本、identity schema version、store 驅動與契約版本與等級與能力與 health、outbox backlog、可匯出 scope 清單與遷移可用性，且不再存在獨立的 data view model；斷言 `--json` 風格的 payload 欄位存在、camelCase 命名與型別，既有欄位名稱未被重新命名。前端同步寫失敗測試斷言側欄不含資料操作目的地、系統頁單頁呈現執行環境／儲存狀態／匯出／危險區、系統 view model 取得失敗時整頁呈現錯誤與重試入口而非部分渲染。以 `cargo test -p speclink-server` 與 `npm test -w apps/server-web -- system` 執行，預期失敗。 <!-- speclink-task:tsk_01KYBTV8PWG6Z7CGMSRN7HQT9K -->
- [x] 5.2 GREEN／REFACTOR：依 design 決策「系統頁合併為單一 view model，資料操作目的地移除」在 `crates/speclink-server/src/admin.rs` 將 data 與 system 兩個 handler 的欄位聚合為單一 struct 後刪除舊 handler，並改寫 `apps/server-web/src/pages/admin/SystemPage.tsx`、移除 `apps/server-web/src/pages/admin/DataPage.tsx` 與 `apps/server-web/src/pages/admin/AdminSection.tsx` 內的資料操作 route；遷移維持 AlertDialog 確認。以 5.1 測試轉綠、`cargo test -p speclink-server` 與 `cargo test --workspace` 全綠驗證，確認 CLI 人眼輸出與 `--json` 回歸對照未受影響。 <!-- speclink-task:tsk_01KYBTV8PWDFDGQ629MPK5BGM0 -->

## 6. 總覽頁與 view model

- [x] 6.1 RED：先寫失敗測試釘住「總覽提供可行動入口與待辦」與 overview view model 的新欄位——四張指標卡（使用者、專案、憑證、待啟用）皆可點入對應目的地、識別資料結構版本呈現於系統健康摘要而非獨立指標卡、無待處理事項時整個待辦區塊不渲染、無有效憑證時待辦呈現該事項並提供建立存取金鑰入口、系統健康摘要與最近活動各自提供前往系統與稽核目的地的入口；後端斷言 overview 回傳待啟用邀請數、待處理事項清單（含類型與對應目的地）與最近稽核事件清單，欄位為 camelCase 且不含任何祕密值。以 `npm test -w apps/server-web -- overview` 與 `cargo test -p speclink-server` 執行，預期失敗。 <!-- speclink-task:tsk_01KYBTV8PW1KF93XY4SD5RSY79 -->
- [x] 6.2 GREEN／REFACTOR：依 design 決策「總覽 view model 增列待辦、系統健康摘要與最近稽核」在 `crates/speclink-server/src/admin.rs` 擴充 overview view model（沿用既有 identity、registry、store 與 audit 查詢，不新增 domain action），並改寫 `apps/server-web/src/pages/admin/OverviewPage.tsx` 為四張可點指標卡加待辦、系統健康、最近活動三個區塊。以 6.1 測試轉綠與 `cargo test -p speclink-server`、`npm test -w apps/server-web` 全綠驗證，並確認 Store 不健康時 overview 仍回 `storeHealthy: false` 與可公開錯誤且 users 與 credentials 管理可用。 <!-- speclink-task:tsk_01KYBTV8PWKHJBAX02TZYDR11B -->

## 7. 文案收斂與詞彙表

- [x] 7.1 RED：先寫失敗測試斷言使用者可見文案不再出現「建立 project」、「Project key」、「Repo key」、「Personal Access Tokens」、「PAT」、「Web Sessions」、「Schema 版本」、「Outbox backlog」等工程詞，且對應中文詞（建立專案、專案代號、儲存庫代號、存取金鑰、登入工作階段、資料結構版本、待送佇列）出現於六個管理目的地與帳號頁。以 `npm test -w apps/server-web` 執行，預期失敗。 <!-- speclink-task:tsk_01KYBTV8PWQJZ9VFMCSVHWMKDB -->
- [x] 7.2 GREEN／REFACTOR：依 design 決策「文案收斂與 LANGUAGE.md 同步在同一個變更內完成」改寫 `apps/server-web` 各頁與抽屜的使用者可見文案，並在 `openspec/LANGUAGE.md` 新增對應詞條（definition／avoid／why），不改動既有正典詞條。以 7.1 測試轉綠、`npm test -w apps/server-web` 全綠與逐條審視 `openspec/LANGUAGE.md` 內容驗證。 <!-- speclink-task:tsk_01KYBTV8PWB6ZZNKAEP26N3AS1 -->

## 8. 原範圍終驗

- [x] 8.1 AUDIT／終驗：對本次調整的 browser API view model 參數處理（稽核篩選與分頁的輸入邊界、合併後 system view model 的 payload 內容）套用 `speclink instructions --skill audit` 的 sharp-edges 檢查清單並修正 Critical 與 High 項目，確認回應不含 PAT hash、PAT plaintext、password hash、refresh credential、setup token 或 invite token；接著執行 `npm run test:all` 與 `cargo test --workspace`，並以真實瀏覽器在 375、768、1024、1440px 走訪六個管理目的地與帳號頁，逐項確認無整頁水平捲動、鍵盤可完成邀請與建立專案與撤銷憑證三項流程、focus ring 可見、`prefers-reduced-motion` 下無轉場動畫，全部留下綠燈證據。 <!-- speclink-task:tsk_01KYBTV8PWMT1MXKCPC0PXQCPJ -->

## 9. Select 原語替換

- [x] 9.1 RED：先寫失敗測試釘住「共用設計系統維持高密度可存取體驗」新增的控件契約。在 `packages/ui/src/__tests__/` 斷言 `Select` 可由鍵盤開啟（Enter／Space）、以方向鍵移動、Enter 選取後回報選中值，且 `@speclink/ui` 不再匯出 `NativeSelect`；在 `apps/server-web/src/__tests__/` 斷言使用者頁與憑證頁的狀態篩選是 `role=combobox` 的 shadcn Select 而非原生 `select`，並斷言工具列的搜尋輸入、狀態下拉與日期輸入三者的高度 class 相同。以 `npm test -w packages/ui` 與 `npm test -w apps/server-web` 執行，預期失敗。 <!-- speclink-task:tsk_01KYCSRPH2P0VH7ZKAGW08PTY5 -->
- [x] 9.2 GREEN／REFACTOR：依 design 決策「下拉選單改用 Radix Select，`NativeSelect` 移除」，在 `packages/ui` 加入 `@radix-ui/react-select` 相依、把 `packages/ui/src/components/ui/select.tsx` 改寫為 shadcn `Select`／`SelectTrigger`／`SelectValue`／`SelectContent`／`SelectItem` 並更新 `packages/ui/src/index.ts` 的匯出（移除 `NativeSelect`），在 `packages/ui/vitest.setup.ts`、`apps/desktop/vitest.setup.ts` 與 `apps/server-web/vitest.setup.ts` 補上 `hasPointerCapture`／`releasePointerCapture`／`scrollIntoView` 的 jsdom stub；改寫 `packages/ui/src/components/KanbanBoard.tsx`、`apps/desktop/src/views/ProjectSettingsView.tsx`、`apps/server-web/src/components/ListToolbar.tsx` 與 `apps/server-web/src/pages/admin/UsersPage.tsx` 的全部下拉，並把既有以 `selectOptions` 驅動原生 select 的測試改寫為點開選單再點選項。以 9.1 測試轉綠與 `npm test -w packages/ui`、`npm test -w apps/desktop`、`npm test -w apps/server-web` 三者全綠驗證。 <!-- speclink-task:tsk_01KYCTBS93Z1FVWYZ34E135KWB -->

## 10. 首次導覽

- [x] 10.1 RED：先寫失敗測試釘住「首次進入提供可略過的分步導覽」——尚未檢視過時開啟 `/admin` 自動啟動導覽且第一步指向側欄的總覽並提供下一步與略過、按略過後重新整理不再自動啟動、系統頁的「重看導覽」可再次啟動、某一步的目標元素不存在時跳過該步而非中斷、導覽進行中按 Escape 可離開。測試以 `localStorage` 的 `speclink.tourSeen` 控制初始狀態，並涵蓋 `localStorage` 讀寫丟例外時導覽仍可運作。以 `npm test -w apps/server-web -- tour` 執行，預期失敗。 <!-- speclink-task:tsk_01KYCTH44REJ6ZF3QZWTYSNCMK -->
- [x] 10.2 GREEN／REFACTOR：依 design 決策「首次導覽以疊層分步呈現，狀態存在瀏覽器」，新增 `apps/server-web/src/lib/tourSeen.ts`（讀寫 `speclink.tourSeen`，任何例外一律視為未看過）與 `apps/server-web/src/components/Tour.tsx`（高亮目標、一句說明、上一步／下一步／略過，不使用 focus trap，且只在沒有 Sheet 或 AlertDialog 開啟時啟動），掛進 `apps/server-web/src/layouts/ConsoleLayout.tsx`，並在 `apps/server-web/src/pages/admin/SystemPage.tsx` 加入「重看導覽」入口。不新增任何前端相依。以 10.1 測試轉綠與 `npm test -w apps/server-web` 全綠驗證。 <!-- speclink-task:tsk_01KYCTNXF5CCBZ12PYBB0YWVJF -->

## 11. 介面多語系

- [x] 11.1 RED：先寫失敗測試釘住「介面語言支援中文與英文」——未設定偏好且 `navigator.language` 為 `en-US` 時管理面文案為英文、為 `zh-TW` 時為中文；header 的語言切換選英文後管理面文案改為英文且重新整理後維持；`APP_MESSAGES` 的 zh-TW 與 en key 集合完全相等（缺漏即紅燈）。同時把既有的 `apps/server-web/src/__tests__/wording.test.tsx` 改為明示在 zh-TW 下斷言中文正典詞，並增列 en 下不出現該批中文詞的對照。以 `npm test -w apps/server-web` 執行，預期失敗。 <!-- speclink-task:tsk_01KYCTTP9ZCSVA8Q3MQ3J9GNGW -->
- [x] 11.2 GREEN／REFACTOR：依 design 決策「多語系沿用 Desktop 的機制，locale helper 提升為共用」，把 `apps/desktop/src/i18n/locale.ts` 的 `UiLocale`／`LocalePreference`／`detectSystemLocale`／`readLocalePreference`／`writeLocalePreference`／`resolveUiLocale` 提升到 `packages/ui/src/locale.ts` 並自 `packages/ui/src/index.ts` 匯出，Desktop 該檔改為 re-export 以維持既有 import 面不變；新增 `apps/server-web/src/i18n/messages.ts` 收齊 server-web 全部使用者可見文案的 zh-TW 與 en 兩份字典，把 `apps/server-web` 各頁、抽屜、工具列、空狀態與導覽的硬編文案改為 `useI18n()` 的 `t(key)`，`apps/server-web/src/App.tsx` 以解析後的 locale 掛 `I18nProvider`，並新增 `apps/server-web/src/components/LocaleSwitch.tsx` 放進 header。以 11.1 測試轉綠與 `npm test -w packages/ui`、`npm test -w apps/desktop`、`npm test -w apps/server-web` 三者全綠驗證。 <!-- speclink-task:tsk_01KYCVG7H7MPPXKYRC9W48J0PJ -->

## 12. 全範圍終驗

- [x] 12.1 終驗：執行 `npm run test:all` 與 `cargo test --workspace` 並確認全綠；以真實瀏覽器在 375、768、1024、1440px 走訪六個管理目的地與帳號頁，逐項確認殼不整頁捲動而主內容區可捲、工具列控件高度一致、下拉展開後的選單套用 theme、首次進入的導覽可完成與略過且不遮住它正在指的元素、語言切換為英文後六個目的地與帳號頁皆無殘留中文、切回中文亦然，全部留下綠燈證據。 <!-- speclink-task:tsk_01KYEBT58YY74B3VQ9G5PVX19F -->

## 13. 走查回饋修正

- [x] 13.1 RED：先寫失敗測試釘住 2026-07-25 走查回饋的六項。在 `packages/ui/src/__tests__/theme.test.ts` 斷言元件用到的每個 `bg-*` 語意色都在 `theme.css` 有對應 `--color-*`（掃描時略過註解）；在 `packages/ui/src/__tests__/select.test.tsx` 斷言 `SelectTrigger` 與 `Input` 共用 `h-9`／`px-3`／`rounded-md`／`shadow-sm`；新增 `packages/ui/src/__tests__/selectInSheet.test.tsx` 斷言 modal Sheet 內的 Select 可開啟選單並選取；在 `apps/server-web/src/__tests__/tour.test.tsx` 斷言導覽卡片帶依目標算出的 inline `top`／`left` 而非固定在畫面底部；在 `apps/server-web/src/__tests__/users.test.tsx` 斷言邀請抽屜的加入專案是 `role=combobox` 且已選專案可逐一移除、送出時 `memberships` 正確；在 `apps/server-web/src/__tests__/account.test.tsx` 斷言建立欄位不常駐頁面、按 primary action 後於抽屜出現、成功後抽屜關閉且明文附複製鈕；在 `apps/server-web/src/__tests__/credentials.test.tsx` 斷言已有憑證時頁首仍有指向 `/account` 的建立入口。以 `npm test -w packages/ui` 與 `npm test -w apps/server-web` 執行，預期失敗。 <!-- speclink-task:tsk_01KYCYR1P4ECMJWQQF5J5K9890 -->
- [x] 13.2 GREEN／REFACTOR：依 design 決策「modal 容器內的浮層 portal 進容器自身」新增 `packages/ui/src/components/ui/portal-container.tsx`，由 `SheetContent` 提供自身節點、`SelectContent` 消費；把 `SelectContent` 的 `bg-popover` 改為 `bg-card`、`SelectTrigger` 對齊 `Input` 的內距與陰影；改寫 `apps/server-web/src/components/Tour.tsx` 讓說明卡片依目標 rect 定位並夾回視窗內；改寫 `apps/server-web/src/pages/admin/UsersPage.tsx` 的邀請抽屜為下拉挑選加已選清單（挑選器綁對不到選項的哨符值，不用 key 重掛）；改寫 `apps/server-web/src/pages/AccountPage.tsx` 為頁首 primary action 開 `CreateKeySheet`，明文回饋加複製鈕；在 `apps/server-web/src/pages/admin/CredentialsPage.tsx` 頁首加入常駐的建立存取金鑰入口並移除空狀態內重複的 action。以 13.1 測試轉綠與 `npm test -w packages/ui`、`npm test -w apps/desktop`、`npm test -w apps/server-web` 三者全綠驗證。 <!-- speclink-task:tsk_01KYCYR1V38K4RGCG2QGH7MAWA -->

## 14. 邀請可見性與邀請連結

- [x] 14.1 RED：先寫失敗測試釘住「使用者頁呈現尚未接受的邀請」與邀請連結。在 `crates/speclink-server/tests/admin_users_view.rs` 斷言 `/admin/users` 的 view model 帶 `pending` 陣列（含 `email`／`display`／`admin`／`memberships`／`createdAt`／`expiresAt`）、已接受的邀請離開該清單且受邀者成為使用者、過期邀請不列入、回應不含 token 或 `tokenHash`；在 `apps/server-web/src/__tests__/users.test.tsx` 斷言待啟用邀請列在使用者表格之外的「邀請中」區塊、無邀請時該區塊不渲染、邀請成功後的回饋是可複製的 `/invite/<token>` 連結而非裸 token。以 `cargo test -p speclink-server --test admin_users_view` 與 `npm test -w apps/server-web -- users` 執行，預期失敗。 <!-- speclink-task:tsk_01KYCZQXFA04MH84XGZM88NYEX -->
- [x] 14.2 GREEN／REFACTOR：依 design 決策「待啟用邀請併入使用者 view model，而非新增一支 API」，在 `crates/speclink-server/src/identity.rs` 為 `Invitation` 增列 `created_at` 並新增 `list_pending_invitations`，於 `crates/speclink-server/src/identity_sqlite.rs` 以與 `count_pending_invitations` 相同的「未使用且未過期」判準實作，於 `crates/speclink-server/src/admin.rs` 的 `WebUsers` 增列 `pending: Vec<WebPendingInvitation>`；前端在 `apps/server-web/src/api/client.ts` 增列 `AdminPendingInvitation` 型別，於 `apps/server-web/src/pages/admin/UsersPage.tsx` 新增「邀請中」區塊並把邀請回饋改為 `{origin}/invite/{token}` 連結，並把複製鈕抽為 `apps/server-web/src/components/CopyButton.tsx` 供帳號頁與使用者頁共用。以 14.1 測試轉綠、`cargo test -p speclink-server` 與 `npm test -w apps/server-web` 全綠驗證。 <!-- speclink-task:tsk_01KYCZQXKBA0HC124K9ESBMPQZ -->

## 15. 撤回邀請

- [x] 15.1 RED：先寫失敗測試釘住「使用者頁呈現尚未接受的邀請」新增的撤回動作。在 `crates/speclink-server/tests/admin_users_view.rs` 斷言 `POST /admin/users/invitations/{id}/revoke` 回 200 後該筆離開 `pending`、`find_valid_invitation` 查不到該 token 且 `accept_invitation` 失敗、留下一筆 subject 為受邀者 email 的 `invitation-revoked` 稽核事件、未知 id 與已接受的邀請皆回 404；在 `apps/server-web/src/__tests__/users.test.tsx` 斷言撤回鈕先開 AlertDialog 且對話框指名受邀者、確認後才以正確 id 呼叫 `adminRevokeInvitation`。以 `cargo test -p speclink-server --test admin_users_view` 與 `npm test -w apps/server-web -- users` 執行，預期失敗。 <!-- speclink-task:tsk_01KYEBGA1QKH3SF750G5W3CSQA -->
- [x] 15.2 GREEN／REFACTOR：依 design 決策「撤回邀請以刪除邀請實作，是本變更唯一新增的 domain action」，在 `crates/speclink-server/src/audit.rs` 新增 `AuditAction::InvitationRevoked`（字串 `invitation-revoked`），於 `crates/speclink-server/src/identity.rs` 新增 `admin_revoke_invitation`，於 `crates/speclink-server/src/identity_sqlite.rs` 於單一交易內刪除該筆邀請與其 memberships 並寫入稽核（未接受者才認，其餘回 `NotFound`），於 `crates/speclink-server/src/admin.rs` 新增 `web_admin_revoke_invitation` 與 `/admin/users/invitations/{id}/revoke` route；前端在 `apps/server-web/src/api/client.ts` 新增 `adminRevokeInvitation`，於 `apps/server-web/src/pages/admin/UsersPage.tsx` 的邀請中區塊每筆加入撤回動作並復用既有 AlertDialog 確認流程，字典補上撤回相關文案與 `audit.action.invitation-revoked`，稽核頁的動作篩選補上該選項。以 15.1 測試轉綠、`cargo test -p speclink-server` 與 `npm test -w apps/server-web` 全綠驗證。 <!-- speclink-task:tsk_01KYEBGA8HFXQKCZEGYWSJXE07 -->

## 16. reduced-motion 守門

- [x] 16.1 RED／GREEN：終驗發現需求「共用設計系統維持高密度可存取體驗」的 Scenario「reduced motion 停用轉場」在 `apps/server-web` 沒有任何實作——原始碼與三份 CSS 皆無 `prefers-reduced-motion` 或 `motion-reduce:`，而 Tailwind 的 `transition-colors` 在 reduced-motion 下照常轉場。新增 `apps/server-web/src/__tests__/reducedMotion.test.ts` 斷言 `apps/server-web/src/index.css` 帶涵蓋 `*`／`*::before`／`*::after` 的 `@media (prefers-reduced-motion: reduce)` 守門且壓掉 animation 與 transition 時長（預期先失敗），再於該 CSS 加入守門使其轉綠。時長寫 0.01ms 而非 0：時長為零時 `transitionend`／`animationend` 不觸發，依賴這些事件收尾的元件會卡在中間狀態。以 `npm test -w apps/server-web` 全綠驗證。 <!-- speclink-task:tsk_01KYECFBX5A3426J5RB4WCYT0X -->

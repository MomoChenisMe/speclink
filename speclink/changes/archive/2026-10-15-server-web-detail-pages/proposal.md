## Summary

server 網頁後台（apps/server-web）拿掉右側滑出的抽屜：點使用者或專案的一列，主內容區換成該筆資料自己的詳情頁（有自己的網址、麵包屑三段、頂列右端「‹ n / m ›」）；邀請使用者、建立專案、新增儲存庫、建立存取金鑰與更名改成畫面中央的對話框，邀請連結與金鑰明文直接顯示在對話框裡。

## Motivation

目標使用者是架設 Speclink server 的管理員，以及透過 AI 代理跑遠端 SDD 流程、要到後台管理自己存取金鑰的成員（開發者／PO／PM）。使用情境是「連線 Server」之後的日常管理：查看與調整某位使用者的成員資格、管理員身分與憑證，為專案新增儲存庫或更名，邀請新成員，在帳號頁建立自己的存取金鑰。

- desktop 已依 desktop-ui-redesign 讓抽屜從桌面退場：點一列就換成佔滿主區的詳情頁、麵包屑右端有 ‹ n／m ›、建立類動作用置中對話框。server-web 是全產品唯一還在用資料抽屜（`DetailSheet`）的地方，兩個產品「點一列」的反應不同。
- 抽屜沒有網址：管理員不能把某位使用者的頁面加書籤或貼給同事，重新整理後抽屜就不見了。
- 一次性祕密值（邀請連結、存取金鑰明文）現在是在抽屜關掉之後，於頁面頂端出現一條橫幅；送出的地方與拿到結果的地方不同。
- 成員資格的「移除」現在按下就生效，沒有確認；規格「管理列表以抽屜承載建立與編輯」早已要求移除成員資格先以確認框確認。

本變更是討論 server-web-desktop-alignment 定案三刀中的第三刀（詳情頁與對話框），也是最後一刀，依賴已封存的第一刀 server-web-shell-alignment 與第二刀 server-web-list-pages。視覺基準是討論中確認的互動原型「server 後台對齊 desktop」的 A 列使用者詳情頁與 B 列邀請對話框；原型沒畫的專案詳情頁，依討論的延後項目，在本刀 design 以使用者詳情頁為範本決定。

## Proposed Solution

- **詳情頁**：網址為 `/admin/users/<使用者 id>` 與 `/admin/registry/<專案代號>`，可直接開啟與重新整理；對不到資料時顯示「找不到」說明與回到列表的按鈕。版面為 768px 置中欄：頁標題是該筆的名稱（使用者名稱後接狀態籤）、灰字 meta、右側頁級動作；其下是卡片標頭分頁加設定型列。
  - 使用者詳情頁：頁級動作為「停權」（停權中為「重新啟用」）。分頁為概要（電子郵件＋複製、狀態、建立日期、管理員開關）、成員資格（每列可就地改角色、滑過才出現的移除鈕，末列為「加入專案」）、憑證（存取金鑰與裝置登入，滑過才出現的撤銷鈕）、稽核。
  - 專案詳情頁：頁級動作為「更名」；meta 標示專案代號建立後不可變更。分頁為儲存庫（每列可更名，末列為「新增儲存庫」）與成員（唯讀）。
- **頂列**：詳情頁的麵包屑為「後台 / <目的地> / <名稱>」，目的地可點回列表；右端「‹ n / m ›」在列表目前呈現的順序（含搜尋與篩選）內切換上一筆、下一筆，到頭停用、不循環，切換不新增瀏覽紀錄。回到列表時，搜尋、篩選與頁碼都保留。
- **對話框**：邀請使用者、建立專案、新增儲存庫、建立存取金鑰、更名專案與儲存庫，改用 packages/ui 既有的 `AlertDialog`（與 desktop 新增專案對話框同款），不新增原語。送出失敗時對話框保持開啟、保留輸入、錯誤緊貼欄位。邀請與建立金鑰送出成功後，對話框換成「已建立」狀態：綠系成功框裡是邀請連結或金鑰明文加複製鈕，下方是「完成」；頁面頂端不再出現橫幅。
- **共用元件（packages/ui）**，desktop 外觀與行為不變：
  - desktop 的 `DetailNav`（‹ n / m ›）搬進 packages/ui，兩邊共用；它的三個顯示字串一起搬進共用字典。desktop 的返回鈕與返回快捷鍵判斷留在 desktop，檔案改名為 `DetailBack.tsx`。
  - `AlertDialogContent` 比照抽屜，提供「浮層掛載點」：對話框裡的下拉選單改掛進對話框自己，避免焦點在對話框與選單之間互搶。
  - `PageHeader` 新增選配的 `badge` 槽，放在標題文字後面（使用者詳情頁的狀態籤）。
- **刪除** `DetailSheet`；之後 Sheet 只剩窄螢幕導覽一個用途。
- **相依（apply 期發現）**：packages/ui 的 Radix 元件各自釘住不同版本的內部套件，對話框裡開著下拉時按 Esc 會連對話框一起關掉、之後整頁點不動（舊抽屜同樣如此）。11 個 `@radix-ui` 套件一起升到 `^` 範圍內的最新版，讓它們共用同一份內部套件；只改 `package-lock.json`。
- **規格**：server-web-console 的「管理列表以抽屜承載建立與編輯」整條移除，改由新需求「管理列表以詳情頁與對話框承載檢視與建立」承接（引擎不允許同一條需求同時 RENAMED 又 MODIFIED，所以用 REMOVED＋ADDED）；另 MODIFIED 四條（見 Capabilities）。需求名與內文裡的「抽屜」一併換成「詳情頁」或「對話框」，收斂 LANGUAGE.md 的詞彙漂移（「抽屜」是「詳情頁」的 avoid 同義詞）；正式規格 Purpose 裡的「抽屜式建立編輯」直接改字（delta 改不到既有 capability 的 Purpose）。

相容性影響：只改 server 網頁後台與共用 UI 套件的畫面。server API、CLI 人眼輸出與 `--json`、引擎全部不變；六個目的地的網址不變，只多出兩種詳情頁網址（落在既有的 `/admin/*` 內，伺服器已對它們回傳 SPA）。desktop 的詳情頁導覽、返回鈕與快捷鍵外觀與行為不變；desktop 既有的對話框若內含下拉或選單，浮層改掛進對話框內，畫面相同。既有使用者不需遷移。

## Alternatives Considered

- 規格改名用 RENAMED 加 MODIFIED：引擎的封存守門把「同一個需求名在兩個操作區段出現」視為衝突並拒絕封存，所以改用 REMOVED 舊名加 ADDED 新名。
- 詳情頁只在網址帶 id、列表狀態不保留（點回列表就回到第 1 頁、搜尋清空）：從抽屜改成換頁後，「看完一筆回到剛才的位置」會退步；把列表狀態上移到使用者與專案兩個區段的父元件，列表與詳情頁共用同一份資料與狀態，不必把搜尋與頁碼寫進網址。
- ‹ n / m › 每按一次就新增一筆瀏覽紀錄：瀏覽器上一頁會一筆一筆倒退，回不到列表；改為取代目前紀錄，上一頁直接回列表（與第二刀 `?tab=pending` 切分頁的作法一致）。
- 新增 Dialog 原語承載表單：討論已否決，packages/ui 的 AlertDialog 已足夠（點對話框外不關閉，反而避免誤觸丟失輸入）。
- 使用者詳情頁照原型放「⋯」更多動作鈕：目前沒有停權以外的次要動作，空選單只是雜訊；之後有次要動作再加。
- 頁標題用原型的 22px：規格要求每頁標題用共用頁標題元件（24px），詳情頁沿用它，只多一個狀態籤槽。

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `server-web-console`: 「管理列表以抽屜承載建立與編輯」REMOVED，由 ADDED「管理列表以詳情頁與對話框承載檢視與建立」承接（詳情頁網址與版面、‹ n / m ›、回列表保留狀態、建立類對話框、祕密值在對話框內一次性呈現、成員資格就地改角色與移除確認）；MODIFIED「全部 browser route 由單一 SPA 提供可發現導覽」（詳情頁麵包屑三段）、「共用設計系統維持高密度可存取體驗」（下拉在對話框內可正常開啟、滑過才出現的按鈕擴及詳情頁的憑證與成員資格列）、「不可變識別欄位唯讀且更名為顯式動作」（專案詳情頁與更名對話框）、「後台狀態徽章語意色」（揭示橫幅改為對話框內的成功框）。其中「全部 browser route…」與「共用設計系統…」不在討論結論列出的三條之內，是 propose 期發現麵包屑與下拉條款會隨本刀失準而補上。

## Impact

- Affected specs: server-web-console（1 條 REMOVED、1 條 ADDED、4 條 MODIFIED）
- Affected app／package: `apps/server-web`（後台 SPA）、`packages/ui`（共用 UI 元件：DetailNav 搬入、AlertDialog 浮層掛載點、PageHeader 狀態籤槽、Card 接在分頁列下方的 `attached` 變體）、`apps/desktop`（只改 import 位置與返回鈕檔名，外觀與行為不變）；speclink-core、speclink-cli、speclink-host、speclink-server 不受影響
- Affected code:
  - New: `packages/ui/src/components/DetailNav.tsx`、`packages/ui/src/__tests__/detailNav.test.tsx`、`packages/ui/src/__tests__/selectInDialog.test.tsx`、`apps/desktop/src/components/DetailBack.tsx`、`apps/desktop/src/__tests__/detailBack.test.tsx`、`apps/server-web/src/layouts/DetailTitle.tsx`、`apps/server-web/src/components/FormDialog.tsx`、`apps/server-web/src/lib/fmtDate.ts`、`apps/server-web/src/pages/admin/detail.tsx`、`apps/server-web/src/pages/admin/UserDetailPage.tsx`、`apps/server-web/src/pages/admin/ProjectDetailPage.tsx`、`apps/server-web/src/__tests__/userDetail.test.tsx`、`apps/server-web/src/__tests__/projectDetail.test.tsx`
  - Modified: `package-lock.json`（packages/ui 的 `@radix-ui` 套件一起升版，見 design D4）、`openspec/specs/server-web-console/spec.md`（只改 Purpose 一句的「抽屜式」字面；delta 改不到既有 capability 的 Purpose）、`packages/ui/src/index.ts`、`packages/ui/src/i18n.tsx`、`packages/ui/src/components/ui/alert-dialog.tsx`、`packages/ui/src/components/ui/page-header.tsx`、`packages/ui/src/components/ui/card.tsx`、`packages/ui/src/components/ui/tabs.tsx`（註解）、`packages/ui/src/components/ListCard.tsx`、`packages/ui/src/__tests__/pageHeader.test.tsx`、`packages/ui/src/__tests__/ui.test.tsx`、`apps/desktop/src/App.tsx`、`apps/desktop/src/i18n/messages.ts`、`apps/server-web/src/layouts/ConsoleLayout.tsx`、`apps/server-web/src/pages/admin/AdminSection.tsx`、`apps/server-web/src/pages/admin/UsersPage.tsx`、`apps/server-web/src/pages/admin/RegistryPage.tsx`、`apps/server-web/src/pages/admin/states.tsx`、`apps/server-web/src/pages/admin/AuditPage.tsx`（改用共用的留著上一份回應）、`apps/server-web/src/pages/admin/CredentialsPage.tsx`（改用共用的日期格式）、`apps/server-web/src/lib/useAsync.ts`、`apps/server-web/src/pages/AccountPage.tsx`、`apps/server-web/src/i18n/messages.ts`、`apps/server-web/src/__tests__/users.test.tsx`、`apps/server-web/src/__tests__/registry.test.tsx`、`apps/server-web/src/__tests__/account.test.tsx`、`apps/server-web/src/__tests__/wording.test.tsx`、`apps/server-web/src/__tests__/toolbar.test.tsx`、`apps/server-web/src/__tests__/a11y.test.tsx`、`apps/server-web/src/__tests__/admin-console-shell.test.tsx`、`apps/server-web/src/__tests__/helpers/adminHarness.tsx`
  - Removed: `apps/server-web/src/components/DetailSheet.tsx`、`apps/desktop/src/components/DetailNav.tsx`、`apps/desktop/src/__tests__/detailNav.test.tsx`

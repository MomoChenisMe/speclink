## Context

server-web-desktop-alignment 定案三刀，本刀是最後一刀。前兩刀已封存：第一刀把殼（側欄、主區頂列麵包屑、帳號選單）與登入這一族改成 desktop 的樣子；第二刀把列表頁換成 `ListCard`＋`ListRow`＋`ListPager`＋`SearchField`、設定型頁換成 `SettingsCard`／`SettingsRow`，並刻意把抽屜留給本刀。

現況（apps/server-web）：
- `components/DetailSheet.tsx` 是 Sheet 抽屜的共用殼，被六處使用：使用者細節（`UsersPage.tsx` 的 `UserDetailSheet`）、邀請（`InviteSheet`）、專案細節（`RegistryPage.tsx` 的 `ProjectSheet`）、建立專案（`CreateProjectSheet`）、建立存取金鑰（`AccountPage.tsx` 的 `CreateKeySheet`）；新增儲存庫是專案抽屜內展開的表單（`CreateRepoForm`），更名是抽屜內就地展開的表單（`RenameAction`）。
- 邀請連結與金鑰明文在抽屜關閉後，由頁面頂端的綠系橫幅以 `aria-live=polite` 呈現。
- 使用者資料只來自 `getAdminUsers()` 一支整份清單（沒有單筆端點）；憑證與稽核在抽屜開啟時才讀（`getAdminCredentials()`、`getAdminAudit()`），再在前端裁切成該使用者的項目。專案成員由 `getAdminUsers()` 反查。
- 成員資格移除按下即送出，沒有確認框——規格「管理列表以抽屜承載建立與編輯」本來就要求移除成員資格先確認，這是既有落差。
- 伺服器的成員資格寫入是 upsert（同一使用者、同一專案重寫角色），所以「就地改角色」直接呼叫既有的 `adminSetMembership(id, { projectKey, role, member: true })` 即可，不需改 server。
- `AdminSection.tsx` 以 react-router 的 `<Routes>` 掛六個目的地；`ConsoleLayout.tsx` 的 `useCrumbs` 以 `end: true` 比對目的地，`/admin/users/<id>` 目前只會顯示「後台」。
- desktop 的 ‹ n／m › 是 `apps/desktop/src/components/DetailNav.tsx` 的 `DetailNav`，同檔還有返回鈕 `DetailBack` 與返回快捷鍵判斷 `isBackShortcut`；它的三個字串（上一個、下一個、第 n 個，共 m 個）在 desktop 的 app 字典。
- packages/ui 的 `SheetContent` 會把自己的 content 節點經 `PortalContainerProvider` 提供出去，讓內部的 Select、DropdownMenu、Popover 浮層掛進抽屜裡；`AlertDialogContent` 沒有這一層。對話框裡放 Select 時，選單會掛到 body、落在對話框的焦點鎖外面，兩邊互推焦點（`portal-container.tsx` 的註解記載了這個堆疊爆掉的問題）。

視覺基準：互動原型「server 後台對齊 desktop」A 列的使用者詳情頁、B 列的邀請對話框（線上原型在帳號更換後讀不到，本刀以本機副本 ServerUserDetail 與 ServerInvite 兩張畫板為準）。專案詳情頁原型沒畫，依討論延後項目以使用者詳情頁為範本。

## Goals / Non-Goals

**Goals:**

- 使用者與專案的抽屜改成有網址的詳情頁（768px 置中欄、頁標題、卡片標頭分頁、設定型列），頂列麵包屑三段、右端 ‹ n／m ›。
- 從詳情頁回到列表時，列表的搜尋、篩選、分頁與頁碼保持原樣。
- 邀請使用者、建立專案、新增儲存庫、建立存取金鑰、更名專案與儲存庫改成置中對話框；一次性祕密值在同一個對話框內以「已建立」狀態呈現，頁面頂端不再有橫幅。
- desktop 的 `DetailNav` 搬進 packages/ui，兩邊共用；`AlertDialogContent` 提供浮層掛載點。
- 刪除 `DetailSheet`；Sheet 只剩窄螢幕導覽。
- 補上成員資格移除的確認框（既有規格要求、實作未做）。
- 規格裡的「抽屜」收斂成「詳情頁」與「對話框」，含 server-web-console 正式規格的 Purpose 一句。

**Non-Goals:**

- 不改 server API、view model 或 speclink-server 任何程式；不新增單筆讀取端點（詳情頁沿用整份清單）。
- 使用者詳情頁的稽核分頁資料來源不變：仍讀稽核紀錄第一頁再於前端裁切為該使用者的事件（`actorId` 等於使用者 id，或 `subject` 等於使用者 id），不新增依操作者篩選的伺服器參數。稽核紀錄頁才是完整查詢的地方。
- 不做 desktop 的返回鈕、Esc／⌘[／⌥↑↓ 快捷鍵與觸控板滑動返回——瀏覽器本身有上一頁；server-web 只搬 ‹ n／m ›。
- 不做使用者詳情頁原型裡的「⋯」更多動作鈕：目前沒有停權以外的次要動作。
- 不把清單列改成 `<a href>`（中鍵開新分頁）；清單列維持共用 `ListRow` 的按鈕語意。
- 不改「邀請中」分頁的邀請列（仍是靜態列、恆常顯示撤回）；不改憑證頁（只有確認框，本來就沒有抽屜）。
- 不新增 Dialog 原語（討論已否決），建立類對話框一律用 `AlertDialog`。
- 邀請對話框保留現有的顯示名稱欄與管理員選項（原型沒畫，但拿掉會改變邀請能力，不在本刀範圍）。

## Decisions

### D1 路由：使用者與專案各自成為一個區段，列表狀態上移

`AdminSection` 的 `users` 與 `registry` 兩條路由改為 `users/*` 與 `registry/*`，分別掛 `UsersSection`（`UsersPage.tsx`）與 `RegistrySection`（`RegistryPage.tsx`）。區段元件持有：
- 資料：`getAdminUsers()`（使用者區段另讀 `getAdminRegistry()` 供專案名稱與加入專案下拉）、`getAdminRegistry()`（專案區段）。
- 列表狀態：搜尋字串、狀態篩選、`usePaging` 的頁碼與每頁筆數，以及由這些算出的 `visible`（列表目前呈現的項目）。
- 內層 `<Routes>`：index 為列表、`:id`（使用者）或 `:key`（專案）為詳情頁，其他路徑導回該列表。

react-router 在 `/admin/users` 與 `/admin/users/<id>` 之間切換時，`users/*` 這條路由的元素不變，`UsersSection` 不會卸載，所以列表狀態自然保留；列表與詳情頁共用同一份資料，詳情頁的變更呼叫區段的 `reload` 後列表也是新的。直接開啟詳情頁網址時區段狀態是初值（無搜尋、無篩選），`visible` 就是全部項目，正好是規格要求的「未經搜尋與篩選的全部項目」。

`useAsync` 的 `reload` 會先把 `data` 清成 null 再重抓；區段若照抄列表頁的 `{data && …}`，每次在詳情頁變更（改角色、撤銷）都會讓詳情頁卸載重掛、分頁跳回第一個。區段因此比照第二刀稽核頁的作法，留著最近一次成功的回應繼續渲染（`useState` 存上一份、`fresh ?? last`），只有第一次載入顯示 `AdminLoading`、載入失敗顯示 `AdminError`。不改 `useAsync` 本身——其他頁面依賴它重抓時清空的行為。

`?tab=pending` 仍只在列表上；詳情頁只能從「使用者」分頁進入（邀請列不可點），回列表時落在使用者分頁。列表卡的捲動位置不保留（列表重新掛載時捲回頂部），頁碼保留。

替代方案：搜尋、篩選、頁碼寫進網址查詢參數——網址變長，且第二刀已定「每頁筆數與頁碼不跨重新整理保留」；或用模組層級的變數暫存——在測試間會漏狀態。上移到區段元件最直接。

### D2 頂列：麵包屑第三段與 ‹ n／m › 的唯一落點

新檔 `apps/server-web/src/layouts/DetailTitle.tsx` 是詳情頁標題列的唯一實作落點，兩個詳情頁都只透過它接頂列：
- `DetailTitleProvider`：`ConsoleLayout` 持有 `label`（字串或 null）與一個插槽節點（頂列 `Breadcrumbs` 之後、`ml-auto` 的空 `div`，以 callback ref 取得），經 context 提供 `setLabel` 與插槽。
- `DetailTitle({ label, index, total, onStep })`：掛載時與 `label` 改變時 `setLabel(label)`，卸載時清回 null；把共用 `DetailNav` 以 portal 送進插槽。插槽尚未掛上時不渲染。

`ConsoleLayout` 的 `useCrumbs` 改以各目的地自己的 `end` 比對（總覽 `end: true`、其他 `end: false`），所以 `/admin/users/<id>` 也找得到「使用者」這個目的地。有 `label` 時麵包屑為「後台 / <目的地> / <label>」，「<目的地>」帶 `onClick` 導向目的地網址（區段內導覽、列表狀態保留），`label` 不可點；沒有 `label` 時維持兩段、目的地不可點。側欄高亮本來就以 `end: false` 比對，詳情頁時「使用者」或「專案與儲存庫」維持高亮。

‹ › 的 `onStep` 以 `navigate(<相鄰項目的網址>, { replace: true })` 切換，不新增瀏覽紀錄；瀏覽器上一頁直接回列表。`index` 為目前項目在區段 `visible` 中的位置，找不到為 -1（`DetailNav` 整組缺席）。

替代方案：直接沿用 packages/ui 的 `DetailActionsSlot`／`DetailActions`——它在內容前插一條分隔線，是給「‹ n／m › 之後的變更層動作」用的，語意不同；label 與 nav 放進 `ConsoleLayout` 的 state——ReactNode 進 state 會隨每次渲染換身分而造成迴圈，所以只有字串進 state、nav 走 portal。

### D3 `DetailNav` 搬進 packages/ui

- 新檔 `packages/ui/src/components/DetailNav.tsx`：內容與 desktop 現版相同（`index`／`total`／`onStep`，到頭停用、不循環，`index < 0` 或 `total === 0` 時回 null，序位文字等寬 11px）。由 `packages/ui/src/index.ts` 匯出 `DetailNav`。
- `detail.prev`、`detail.next`、`detail.position` 三個字串搬進 packages/ui 的內建字典（`packages/ui/src/i18n.tsx`，zh-TW 與 en 兩語系），自 desktop app 字典刪除。文案不變。
- desktop：`apps/desktop/src/components/DetailNav.tsx` 刪掉 `DetailNav` 後只剩 `DetailBack` 與 `isBackShortcut`，檔案改名為 `DetailBack.tsx`；`App.tsx` 改從 `@speclink/ui` 取 `DetailNav`、從 `./components/DetailBack` 取另外兩個。
- 測試：`apps/desktop/src/__tests__/detailNav.test.tsx` 的 `DetailNav` 案例搬到 `packages/ui/src/__tests__/detailNav.test.tsx`（以 packages/ui 內建字典驗字串），`DetailBack` 與 `isBackShortcut` 案例改名留在 `apps/desktop/src/__tests__/detailBack.test.tsx`。`uiSingleSource` 守門測試在搬家後必須綠（desktop 與 server-web 都不得再定義 `DetailNav`）。

### D4 對話框殼：`FormDialog`、`SecretDone` 與 AlertDialog 浮層掛載點

- packages/ui `AlertDialogContent` 比照 `SheetContent`：以 callback ref 取得 content 節點，children 包進 `PortalContainerProvider`。對話框內的 Select、DropdownMenu、Popover 浮層因此掛進對話框自己，焦點留在同一個焦點鎖內。新測試 `packages/ui/src/__tests__/selectInDialog.test.tsx` 釘住「AlertDialog 內的 Select 可開啟、可選取、選單節點在對話框內」。
- 新檔 `apps/server-web/src/components/FormDialog.tsx`（取代 `DetailSheet`）：
  - `FormDialog({ open, onOpenChange, title, description?, children })`：`AlertDialog`＋`AlertDialogContent`（寬 520px、上限為視窗寬減 2rem）＋`AlertDialogHeader`（15px 標題、13px 灰字說明）＋內容。沒有說明時 content 的 `aria-describedby` 設為 undefined。
  - `FormDialogFooter({ children })`：頁尾細線上方右側排列（`AlertDialogFooter`＋`border-t border-border pt-3.5`，與第一刀專注流程卡的頁尾同一畫法）。
  - `SecretDone({ title, body, value, copyLabel, note?, onDone })`：已建立狀態——左側綠系圓形勾號＋15px 標題與灰字說明；其下 `role=status`＋`aria-live=polite` 的綠系成功框（`SEMANTIC_SURFACE.success`、12px 圓角），內為等寬字的值與 `CopyButton`；可選的 12px 灰字補充；頁尾只有主要鈕「完成」。
- AlertDialog 原生行為：Esc 與取消關閉、點遮罩不關閉、關閉後焦點回到開啟它的按鈕。表單元件在 `open` 為 false 時回 null，下次開啟一律是空白表單。
- 對話框內下拉的 Esc（apply 期發現）：packages/ui 的 Radix 元件原本各自釘住不同版本的內部套件（dismissable-layer、focus-scope 各有多份），Select 與 AlertDialog 都以為自己在最上層——下拉開著時按 Esc 會同時關掉下拉與對話框，且 body 的 pointer-events 停在 none，整頁點不動（舊抽屜同樣如此）。只在 `AlertDialogContent` 擋 Esc 行不通（擋住對話框就連下拉也關不掉），根治是讓兩邊共用同一份：以 `npm update -w packages/ui` 把 11 個 `@radix-ui` 套件一起升到 `^` 範圍內的最新版，只改 `package-lock.json`（package.json 範圍不變、不加 overrides），內部套件收斂為單一份。`selectInDialog.test.tsx` 釘住「下拉開著時 Esc 只收起下拉、再按才關對話框、頁面恢復可點」。

### D5 使用者詳情頁（新檔 `UserDetailPage.tsx`）

由 `UsersSection` 的 `:id` 路由渲染，接收區段的資料、`visible`、專案清單與 `reload`。
- 版面：`mx-auto flex w-full max-w-[768px] flex-col gap-6`（與第二刀設定型頁同一寬度）。
- 頁標題：`PageHeader`（level 1）＋ D8 的 `badge` 放狀態籤（有效為綠系、停權為琥珀系）；說明沿用 `users.detailMeta`（「email · 角色 · 建立於 日期」）；動作為「停權」（outline，`canSuspend` 為 false 時停用，按下開確認框）或停權中的「重新啟用」（直接送出）。
- `DetailTitle`：label 為使用者名稱（`userLabel`），序列為區段 `visible`。
- 分頁：`Tabs`＋`TabsList variant="card"`（窄螢幕時 `overflow-x-auto`，卡片標頭內水平捲動）＋去上圓角與上框線的 `Card`（與 `ListCard` 同一接法）；分頁內容為 `SettingsRow` 列。分頁狀態是元件 state，‹ › 切換使用者時保留所在分頁。
  - 概要：電子郵件（等寬＋`CopyButton`）、狀態（狀態籤）、建立（日期）、管理員（`Switch`，說明「可以管理使用者、專案與系統；這台 server 至少要保留一位管理員。」；`admin && !canRemoveAdmin` 時停用，切換即送 `adminSetAdminFlag`）。
  - 成員資格（計數＝成員資格筆數）：每列標題為專案名稱（registry 查不到時用代號）、說明為等寬代號；右側為角色 `Select`（editor／reader，`aria-label`「<代號> 的角色」，變更即送 `adminSetMembership(..., member: true)`）與移除圖示鈕（`REVEAL_ON_GROUP_HOVER`，按下開確認框指名使用者與代號，確認後送 `member: false`）。末列「加入專案」：專案 `Select`（只列尚未加入的專案；全都加入時停用並以 placeholder 說明）、角色 `Select`（預設 editor）、「加入」鈕（未選專案時停用）。寬螢幕控制項在列右側，窄螢幕放到 `SettingsRow` 的 children（標題下方整列、可換行）。沒有任何成員資格時以 `CardNote` 說明。
  - 憑證（計數＝存取金鑰＋裝置登入筆數）：詳情頁掛載與 id 改變時讀 `getAdminCredentials()` 並裁切成該使用者。存取金鑰列：名稱；說明「前綴 · 存取金鑰 · 到期 · 最近使用」（已撤銷改為撤銷日期）；右側 `RevokeButton`（僅有效項目）＋`RevocationPill`。裝置登入列：來源；說明「裝置登入 · 最近更新 日期」。撤銷開確認框，確認後送 `adminRevokeToken`／`adminRevokeFamily` 並重讀憑證；重讀期間同 D1 留著上一份成功回應，分頁計數不閃掉。
  - 稽核：開啟該分頁時讀 `getAdminAudit()`（資料來源見 Non-Goals）。每列為等寬 12px 日期、`auditLabel` 動作名稱、操作者名稱（以使用者清單查 `actorId`，查不到原樣呈現）。
- 找不到：資料已載入但 id 對不到時，以 `EmptyState` 說明找不到這位使用者，附「回到使用者列表」按鈕；不渲染 `DetailTitle`（頂列沒有 ‹ n／m ›，麵包屑兩段）。
- 所有變更沿用現行 `runNow`／`runConfirmed` 模式（busy 期間停用控制項，結束後 `reload`）。

### D6 專案詳情頁（新檔 `ProjectDetailPage.tsx`）與更名

由 `RegistrySection` 的 `:key` 路由渲染。
- 頁標題：`PageHeader`，title 為專案名稱，說明「代號 <key> · 建立後不可變更」，動作為「更名」（outline，開更名對話框）。
- `DetailTitle`：label 為專案名稱，序列為區段 `visible`。
- 分頁（同 D5 的卡片標頭分頁）：
  - 儲存庫（計數）：每列標題為儲存庫名稱、說明「代號 <key> · 建立後不可變更」、右側「更名」鈕（`aria-label`「更名 <名稱>」，開更名對話框）；末列「新增儲存庫」（說明「代號建立後不可變更；名稱之後可以更名。」）右側「新增儲存庫」鈕開對話框。沒有儲存庫時以 `CardNote` 說明，末列照常出現。
  - 成員（計數）：詳情頁掛載時讀 `getAdminUsers()` 反查成員；每列標題為顯示名稱、說明為電子郵件、右側為等寬中性角色籤。唯讀、不可點。
- 更名對話框（`ProjectDetailPage.tsx` 內的 `RenameDialog`，專案與儲存庫共用）：標題「更名專案」或「更名儲存庫」，說明「代號 <key> 不會改變。」，一個預填目前名稱的欄位，頁尾「取消」「確認」。空白名稱時確認鈕停用。失敗保持開啟、保留輸入、`role=alert`；成功關閉並 `reload`，頁標題與麵包屑隨即換成新名稱。
- 找不到：同 D5，說明找不到這個專案並附「回到專案列表」。
- 刪除 `RenameAction`、`CreateRepoForm`、`ProjectSheet`、`ProjectMembers`（舊抽屜內容）；邏輯搬進詳情頁與對話框。

### D7 建立類對話框

都以 D4 的 `FormDialog` 承載、表單欄位沿用 `components/Field.tsx`、錯誤沿用 `readFormError`：
- 邀請使用者（`UsersPage.tsx` 內的 `InviteDialog`）：說明「受邀者會拿到一個連結，開啟後自己設定密碼；連結只在送出後顯示這一次。」；欄位為電子郵件、顯示名稱、加入專案（沿用現行的哨兵值挑選器：選一個就加入已選清單；已選的專案列在圓角細框內，每列專案名稱＋等寬代號＋移除圖示鈕，已選者不再出現在下拉）、管理員（`Switch` 列，與詳情頁同一畫法）。送出成功後同一個對話框換成 `SecretDone`：標題「邀請已建立」、說明「把這個連結給 <email>。它只顯示這一次，關掉對話框之後就看不到了。」、值為 `<origin>/invite/<token>`、複製鈕「複製連結」、補充「受邀者接受前，可在「使用者 › 邀請中」撤回這個邀請。」；成功當下即 `reload`，邀請中計數隨之加一。頁面頂端的邀請連結橫幅刪除。
- 建立專案（`RegistryPage.tsx` 內的 `CreateProjectDialog`）：欄位為專案代號（附「建立後不可變更」提示）與名稱；成功關閉並 `reload`。
- 新增儲存庫（`ProjectDetailPage.tsx` 內的 `CreateRepoDialog`）：欄位為儲存庫代號（附同一提示）與名稱；成功關閉並 `reload`。
- 建立存取金鑰（`AccountPage.tsx` 內的 `CreateKeyDialog`）：欄位為名稱與到期；成功後換成 `SecretDone`：標題「金鑰已建立」、說明「這把金鑰只顯示這一次，關掉對話框之後就看不到了。」、值為明文金鑰、複製鈕「複製」；成功當下即 `reload`。頁面頂端的明文金鑰橫幅刪除。

### D8 `PageHeader` 的 `badge` 槽

packages/ui `PageHeader` 新增選配 `badge?: ReactNode`：放在標題文字之後、與標題同一列垂直置中（`flex items-center gap-2`），標題仍是同一個 `h1`／`h2` 元素且字級不變（24px）。不給 `badge` 時 DOM 與現行相同，desktop 呼叫端不受影響。原型的 22px 名稱改用共用的 24px，以維持規格「每頁標題使用共用頁標題元件」。

### D9 規格寫法

- 「管理列表以抽屜承載建立與編輯」用 REMOVED＋ADDED（新名「管理列表以詳情頁與對話框承載檢視與建立」）：封存守門把同一需求名同時出現在 RENAMED 與 MODIFIED 視為衝突並拒絕。
- MODIFIED 四條：「全部 browser route…」（詳情頁麵包屑三段、側欄高亮）、「共用設計系統…」（下拉在對話框內可正常開啟、滑過才出現的按鈕擴及詳情頁）、「不可變識別欄位…」（更名改對話框）、「後台狀態徽章語意色」（揭示橫幅改對話框內成功框）。改名的情境以 `REMOVED-SCENARIO` 註解宣告。
- 規格原文「提交成功 SHALL … 以 aria-live=polite 回饋」實際只有祕密值做到；新需求把 aria-live 限定在祕密值成功框，其餘建立成功以「關閉對話框並立即更新列表或詳情頁」描述，與實作一致。
- server-web-console 正式規格的 Purpose 有「管理列表的抽屜式建立編輯」：delta 改不到既有 capability 的 Purpose，依 specs 指引直接改 `openspec/specs/server-web-console/spec.md` 的 Purpose 一句為「管理列表的詳情頁與對話框式檢視建立」。

### D10 i18n

server-web 字典（zh-TW 與 en 同步）：
- 新增：詳情頁（找不到的標題、說明與回列表按鈕；管理員開關說明；加入專案列的標題與說明；角色下拉的無障礙名稱；移除成員資格確認框的標題與說明；憑證列的說明格式；專案詳情頁的說明格式）、對話框（邀請、建立存取金鑰的說明；兩個已建立狀態的標題、說明與補充；「完成」；「複製連結」；更名專案與更名儲存庫的標題與說明；新增儲存庫的說明）。
- 刪除：因本刀孤兒化的 key（頁面橫幅的 `users.inviteLinkNotice`、`account.newKeyNotice`，舊抽屜專用而詳情頁不再使用的字串，例如 `users.addProject` 的「＋ 加入專案」）。刪除前以 grep 確認全 app 已無引用。
- 不出現「抽屜」字樣。

### D11 窄螢幕（<1024px）

詳情頁 768px 欄在窄螢幕即為全寬（主內容區 `p-5`）；`PageHeader` 動作槽沿用第二刀的換行規則；卡片標頭分頁在卡內水平捲動；加入專案列的控制項換到標題下方一列；對話框寬度上限為視窗寬減 2rem。全部不造成整頁水平捲動。

## Implementation Contract

**行為（管理員與成員看得到的）**
- 使用者頁點一列（或 Tab 到該列按 Enter）→ 網址變 `/admin/users/<id>`，主內容區是該使用者的詳情頁，沒有側邊滑出的面板；專案頁同理為 `/admin/registry/<key>`。
- 詳情頁網址可直接開啟與重新整理；對不到資料時顯示找不到說明與回列表按鈕。
- 詳情頁頂列麵包屑「後台 / <目的地> / <名稱>」，目的地可點回列表；右端「‹ n / m ›」依列表目前呈現順序切換、到頭停用、不新增瀏覽紀錄。回列表後搜尋、篩選、頁碼不變。
- 邀請使用者、建立專案、新增儲存庫、建立存取金鑰、更名皆為置中對話框；Esc 與取消關閉、點遮罩不關閉；失敗保留輸入且錯誤緊貼欄位。
- 邀請與建立金鑰成功後同一個對話框顯示已建立狀態（綠系成功框＋複製＋完成），頁面頂端沒有橫幅。
- 使用者詳情頁的成員資格可就地改角色、移除需確認、末列可加入專案；管理員以開關切換；憑證可撤銷（需確認）。
- desktop 詳情頁的 ‹ n／m ›、返回鈕與快捷鍵外觀與行為不變。

**介面／資料形狀**
- packages/ui 新匯出 `DetailNav({ index: number; total: number; onStep: (delta: -1 | 1) => void })`；`PageHeader` 新增選配 `badge?: ReactNode`；`AlertDialogContent` 對外 props 不變，內部多提供浮層掛載點。
- server-web 新元件：`DetailTitleProvider`、`DetailTitle({ label: string; index: number; total: number; onStep })`（`layouts/DetailTitle.tsx`）；`FormDialog`、`FormDialogFooter`、`SecretDone`（`components/FormDialog.tsx`）；`UserDetailPage`、`ProjectDetailPage`。
- 路由：`/admin/users/*`、`/admin/registry/*` 由區段元件承接；server API 呼叫與參數完全不變。

**失敗模式**
- 詳情頁資料載入失敗：沿用 `AdminError`（重試）。
- 對話框送出失敗：對話框保持開啟、非祕密輸入保留、欄位錯誤以 `role=alert` 緊貼欄位，非欄位錯誤在表單底部。
- 角色變更、管理員開關、加入或移除成員資格、撤銷失敗：沿用現行作法——busy 結束後 `reload`，畫面回到伺服器真相。

**驗收**
- `npm test -w packages/ui`：`detailNav.test.tsx`、`selectInDialog.test.tsx`、`pageHeader.test.tsx`（badge）、`uiSingleSource.test.ts` 綠。
- `npm test -w apps/desktop -- detailBack messages App`：返回鈕、字典與詳情頁導覽的既有案例綠。
- `npm test -w apps/server-web`：`userDetail.test.tsx`、`projectDetail.test.tsx`、`users.test.tsx`、`registry.test.tsx`、`account.test.tsx`、`admin-console-shell.test.tsx`、`wording.test.tsx`、`a11y.test.tsx` 綠，且全套無紅。
- `npm run build -w apps/server-web` 成功。
- 以 grep 確認 `apps/server-web/src` 不再有 `DetailSheet`；`apps/desktop/src` 不再定義 `DetailNav`。
- 瀏覽器實機驗收（[M] 任務）。

**範圍**
- 內：上列 apps/server-web、packages/ui 三處（DetailNav、AlertDialogContent、PageHeader）、apps/desktop 的 import 與檔名、server-web-console 規格（含 Purpose 一句）。
- 外：server API、speclink-server、CLI、引擎；憑證頁、稽核紀錄頁、總覽、系統頁的版面；desktop 任何畫面。

## Risks / Trade-offs

- [AlertDialog 加浮層掛載點影響 desktop 既有對話框] → 掛載點只改「浮層掛在哪個節點」，不改外觀；desktop 的對話框若內含 DropdownMenu 或 Popover，改掛進對話框後反而與焦點鎖一致。跑 desktop 對話框相關測試（`npm test -w apps/desktop`）確認不紅。
- [對話框是 `alertdialog` 角色，不是 `dialog`] → 討論已定用 AlertDialog；測試改以 `alertdialog` 角色查詢。螢幕閱讀器會以較強語氣宣告開啟，點遮罩不關閉則避免誤觸丟失輸入。
- [區段元件持有狀態後，詳情頁的變更會讓列表重載] → 這正是要的（列表與詳情頁一致）；`useAsync` 重抓時會清空 `data`，區段以 D1 的「留著上一份成功回應」渲染，詳情頁不閃空白、不跳回第一個分頁。測試要釘住「改角色後仍停在成員資格分頁」。
- [稽核分頁只看得到最近一頁事件] → 既有限制、列入 Non-Goals；稽核紀錄頁可完整查詢。
- [測試改寫量大：users.test.tsx、registry.test.tsx、account.test.tsx 的抽屜斷言] → 詳情頁的案例搬到新檔 `userDetail.test.tsx`、`projectDetail.test.tsx`，列表檔只留列表與對話框；回歸對照以「每個被刪的抽屜案例都在新檔有對應案例」檢查。
- [desktop `DetailNav` 字串搬家] → 文案不變；`messages.test.ts` 兩語系 key 集合相等的檢查涵蓋刪除，`detailNav.test.tsx` 在 packages/ui 以內建字典驗字串。
- [Radix 一起升版（D4）] → 小版本之間的行為差異可能影響 desktop 與 server-web 的對話框、選單與分頁；以 `npm test -w packages/ui`、`npm test -w apps/desktop`、`npm test -w apps/server-web` 全套與 `npm run build -w apps/server-web` 把關，瀏覽器實機驗收（[M]）再看一次。
- [跨平台] → 本刀只改前端與共用 UI 套件，沒有路徑或檔案系統行為；CI 的 Windows 前端測試照常跑同一套 vitest。

## Migration Plan

純前端改版，伺服器與資料不變。部署新版 server（內含新 SPA）即生效；舊版書籤（`/admin/users`、`/admin/registry`）照常可用。回退即部署前一版。

## Open Questions

（無）

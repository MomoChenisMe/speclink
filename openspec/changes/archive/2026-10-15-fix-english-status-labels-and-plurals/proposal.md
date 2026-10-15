## Problem

桌面 app 與 server 後台切到英文介面時，有兩個看得到的錯：

- **詳情頁「狀態」列的標籤與數值重疊**：規格詳情頁的「Requirements」「Provenance」標籤蓋到右邊的數字，畫面讀成「Requirem2ts」「Provenance1」。變更、討論與已封存詳情頁共用同一個狀態列，較長的英文單字標籤都有同樣風險。
- **英文計數一律用複數**：數量是 1 時仍顯示「1 scenarios」「1 rounds」「1 tasks」「1 days ago」「1 users have not activated yet.」。用 `{n}` 代入的有 35 個（`packages/ui` 共用字典 32 個、server 後台的 app 字典 3 個）；另有 11 個用 `{count}` 或 `{m}` 代入（`packages/ui` 5 個、桌面 app 字典 6 個），以及 4 個寫成「(s)」的（例如「1 issue(s)」），共 50 個。

受影響的是用英文介面的使用者，也包括 README 英文截圖的讀者：`docs/assets/screenshots/en/desktop-spec.png` 就拍到了重疊與「1 scenarios」。2026-10-15 重拍文件截圖時發現。

## Root Cause

- 狀態列的列元件 `StatusRow`（`packages/ui/src/components/DetailHeader.tsx`）把標籤欄寫死為 `w-14`（56px）。繁中標籤最多四個字，剛好放得下；英文單字標籤（例如 Requirements，約 75px）無法折行，只能溢出到數值上。
- 共用 i18n 只有 `t(key)` 一種查詢，呼叫端以 `t(key).replace("{n}", …)` 代入數量，沒有依數量選單複數的機制；英文字串只寫了複數形。

## Proposed Solution

- **狀態列對齊**：「狀態」列容器改成兩欄格線，各列以 subgrid 共用同一個標籤欄。標籤欄寬度取該頁最長標籤的寬度，下限 56px（繁中版面不變）、上限 112px，超過上限才在字間折行。
- **單複數**：共用 i18n 新增 `tn(key, n)`：介面語言為英文且字典有 `<key>.one` 時，數量 1 用單數形；其餘情況用原本的 `<key>`，並代入 `{n}`。英文字典為這 50 個訊息補上 `.one` 單數形；`{count}`、`{m}` 在兩種語言都統一改成 `{n}`，「(s)」改成正常的複數形；呼叫端改用 `tn`。繁中不需要單數形。不在 React 元件裡的輔助函式（摘要、相對時間）改收一個翻譯物件 `Translator`，測試用 `translatorFor` 取得同一套規則。
- **防回流**：新增守門測試：英文字典中「`{n}` 之後接複數名詞」的訊息必須有 `.one`；三個字典的中英鍵集合相等測試把 `.one` 視為英文專屬的單數形，不列入比對，但要求同一語言有對應的基礎鍵。
- 正式規格新增三條需求：桌面詳情頁狀態列不重疊、桌面與後台的英文計數依數量用單複數。

## Non-Goals

- 不重拍文件截圖：`docs/assets/screenshots/` 的重拍歸 docs-tray-and-mod-screenshots 變更（它依賴本變更）。
- 不處理英文以外語言的複數規則，也不引入 ICU MessageFormat 或 react-i18next 這類執行期相依。
- 不改「Wave {n}」「Round {n}」「{n}/{m}」這類序數或分數字串，它們不是名詞計數。
- 不改狀態列的列內容、順序與資料來源，只改標籤欄的寬度規則。
- 不改繁中文案。

## Success Criteria

- 英文介面的規格詳情頁，狀態列「Requirements 2」「Provenance 1」「Last updated …」各自標籤與數值分開，數值欄左緣對齊；繁中介面的狀態列標籤欄仍是 56px。
- 數量為 1 時顯示「1 scenario」「1 round」「1 issue」「1 rule」「1 user has not activated yet.」；數量為 0 或 2 以上時維持複數。差 1 天的相對時間照舊顯示「Yesterday」。
- `npm test -w packages/ui`、`npm test -w apps/desktop`、`npm test -w apps/server-web` 全部通過，含新的 `tn` 測試、狀態列結構測試與單數形守門測試。
- `node --test scripts/*.test.mjs scripts/*/*.test.mjs` 通過。

## Impact

- Affected specs: `desktop-app`（新增：詳情頁狀態列標籤對齊且不與數值重疊）、`desktop-config`（新增：英文計數文案依數量用單複數）、`server-web-console`（新增：後台英文計數文案依數量用單複數）
- Affected apps／packages：`packages/ui`（共用 i18n 與狀態列元件）、`apps/desktop`、`apps/server-web`（呼叫端與字典）。crate、CLI、`--json` 輸出與設定欄位都不受影響。
- Affected code:
  - Modified:
    - `packages/ui/src/i18n.tsx`
    - `packages/ui/src/components/DetailHeader.tsx`
    - `packages/ui/src/components/DetailShell.tsx`
    - `packages/ui/src/__tests__/i18n.test.tsx`
    - `apps/server-web/src/i18n/messages.ts`
    - `apps/server-web/src/__tests__/i18n.test.tsx`
    - `apps/desktop/src/__tests__/messages.test.ts`
    - `packages/ui/src/index.ts`
    - `packages/ui/src/time.ts`
    - `apps/desktop/src/i18n/messages.ts`
    - `packages/ui/src` 底下代入上述 50 個計數訊息的元件（約 40 處，含 `packages/ui/src/components/AnalyzePanel.tsx`、`packages/ui/src/components/RevertBlockedDialog.tsx`、`packages/ui/src/components/TicketView.tsx`）、`apps/desktop/src` 的 4 個元件（`apps/desktop/src/components/AssetNoticeCard.tsx`、`apps/desktop/src/components/MigrationDialog.tsx`、`apps/desktop/src/views/project-settings/RulesPage.tsx`、`apps/desktop/src/views/project-settings/ReviewRulesPage.tsx`）與 `apps/server-web/src` 的 3 個頁面（`apps/server-web/src/pages/admin/OverviewPage.tsx`、`apps/server-web/src/pages/admin/RegistryPage.tsx`、`apps/server-web/src/pages/admin/UsersPage.tsx`）
    - 改用共用字典檢查的既有測試：`apps/desktop/src/__tests__/tabs.test.ts`、`apps/desktop/src/__tests__/App.test.tsx`；補情境測試的：`apps/server-web/src/__tests__/registry.test.tsx`、`apps/server-web/src/__tests__/overview.test.tsx`、`packages/ui/src/__tests__/discussionColumn.test.tsx`、`packages/ui/src/__tests__/specDetailPage.test.tsx`
  - New:
    - `packages/ui/src/__tests__/detailStatusLayout.test.tsx`
    - `packages/ui/src/__tests__/time.test.ts`

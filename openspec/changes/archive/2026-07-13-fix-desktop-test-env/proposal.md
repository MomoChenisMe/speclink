## Problem

`apps/desktop` 的 vitest 測試套件在乾淨環境下有 42 個測試失敗：`src/__tests__/workspace.test.ts`（15 個，分頁列與開啟專案的本機狀態持久化）與 `src/__tests__/App.test.tsx`（27 個）於 `beforeEach` 呼叫 `localStorage.clear()` 時拋 `TypeError: Cannot read properties of undefined (reading 'clear')`。因為這是 `npm run test:all` 串接鏈中 `npm test -w apps/desktop` 這一步，失敗會使整條 `&&` 鏈在此中止——`test:all` 對任何 change 都不再是可信的綠燈 gate，貢獻者必須以 stash 對照才能區分「自己弄壞的」與「本來就壞的」。

此紅燈長期被遮掩：`packages/ui` 用同一套 jsdom 且 280 測試全過，但 ui 測試從不觸及 `localStorage`，因此掩蓋了 desktop 套件的失敗。

## Root Cause

vitest 3.2.6 的 jsdom 測試環境在 `apps/desktop` 這個 workspace 沒有把 `localStorage`／`sessionStorage` 掛到 window（也未成為測試全域）。已逐項排除其他成因：

- 環境有載入：探測顯示 `typeof window` 與 `typeof document` 皆為 `object`，僅 `localStorage` 為 `undefined`。
- 非 opaque-origin 問題：探測顯示 `window.location.href` 為有效的 `http://localhost:3000/`，而非 `about:blank`。
- 非 jsdom 本體缺陷：以 node 直接 `new JSDOM('', { url })` 建立的 window，其 `localStorage` 為 `object`——jsdom 25.0.1 本體有提供，是 vitest 環境的接線沒暴露它。
- 設定缺口：`apps/desktop` 宣告 `environment: "jsdom"`，卻未把 `jsdom` 列為自身 devDependency（靠 `packages/ui` 的 hoisting），且無 setupFiles 補齊瀏覽器全域。

此問題完全由鎖定的套件版本（vitest 3.2.6、jsdom 25.0.1）與 `apps/desktop` 的最小 vitest 設定決定，故 repo 全域可重現、非本機因素；CI 同樣會中。與 `drift-client-server-split` 無關（已於乾淨樹重現、確認）。

（實作期發現的第二個既有缺陷）補齊儲存全域、讓 `App.test.tsx` 得以實際執行後，暴露出一個原本被遮蔽的生命週期漏洞：`store.ts` 的看板搜尋去抖以 200ms `setTimeout` 排程 `dataSource.searchWorkspace(...).then(...)`，但**未在擁有此 store 的元件卸載時取消**。`App.test.tsx` 有測試在搜尋框輸入後未清空即結束，漏出的 timer 偶爾在測試環境拆除後才觸發，對已失效的 mock 回傳呼叫 `.then` → 未處理例外使 vitest 以非零退出（約 2–5% 的跑），儘管所有斷言通過。此為真實的去抖生命週期漏洞（去抖不應在擁有它的元件卸載後開火），先前因 `App.test.tsx` 全數卡在 `localStorage.clear()` 而從未走到搜尋路徑，故被遮住。

## Proposed Solution

以最小外科手術補齊 desktop 測試環境的瀏覽器儲存全域：

- 新增 vitest setup 檔（`apps/desktop/vitest.setup.ts`）：以一個真實 jsdom `Window` 的 `Storage` 實例，在測試啟動前將 `localStorage` 與 `sessionStorage` 掛到 `globalThis`（並確保每個測試檔起始為乾淨狀態）。此 polyfill 路徑已以一次性探測驗證可讓 desktop 現有測試的 `localStorage.setItem`／`getItem`／`clear` 全數通過。
- 於 `apps/desktop/vitest.config.ts` 的 `test` 區塊加入 `setupFiles` 指向該檔。
- 把 `jsdom` 加入 `apps/desktop` 的 `package.json` devDependencies（版本對齊現行 `25.0.1`），使 setup 檔的 jsdom 解析不再依賴 hoisting——修正「宣告 jsdom 環境卻不直接依賴 jsdom」的既有衛生缺口。
- 於 `store.ts` 補上搜尋去抖的生命週期清理：新增 `disposeSearch()` 取消在途 debounce timer 並前進序號作廢在途回填，並在 `App.tsx` 既有的 `[useStore]` 生命週期 effect 卸載時呼叫——杜絕漏出的 timer 在 store 卸載後才開火所致的未處理例外，使 exit 0 為決定性結果。

除上述搜尋去抖的生命週期清理外，不改動其他 desktop 產品行為、不改既有測試斷言、不更動 DOM 實作（維持 jsdom，不切換 happy-dom）。

## Non-Goals

- 不切換 DOM 實作（不改用 happy-dom 或其他環境）——維持 jsdom，只補齊缺失的儲存全域。
- 不升級 vitest 或 jsdom 版本——以設定與 setup 修正，避免牽動其他 workspace 的環境行為。
- 除為讓套件穩定綠燈所需的搜尋去抖生命週期清理（`store.ts` 的 `disposeSearch` ＋ `App.tsx` 卸載時呼叫）外，不擴及與此紅燈無關的其他產品行為、不修改既有測試斷言。
- 不觸及 `packages/ui`、`crates/speclink-node` 或桌面 GUI 的真實視窗驗證（jsdom 本就測不出 pointer／拖曳互動，那屬另一種驗證手段）。

## Success Criteria

- `npm test -w apps/desktop` 於乾淨環境全綠：`workspace.test.ts`（15）、`App.test.tsx`（27）與其餘既有測試皆通過，無 `Cannot read properties of undefined` 例外。
- `npm run test:all` 不再於 `apps/desktop` 步驟中止，整條鏈可跑到底（含後續 `crates/speclink-node` 步驟）。
- `npm test -w apps/desktop` 的 exit 0 為**決定性**（連續重複執行 ≥20 次不再偶發非零）；測試執行期間無任何未處理例外（含 Web Storage 未定義、以及在途搜尋去抖於測試卸載後觸發所致的 uncaught 例外）。
- 於 desktop vitest 環境中，`typeof localStorage` 與 `typeof sessionStorage` 皆為 `object`，且 `setItem`／`getItem`／`clear` 語意正確、測試間互不殘留。
- 不引入新的失敗；`packages/ui` 與 `crates/speclink-node` 的既有測試維持全綠。

## Impact

- Affected specs: `delivery-baseline`（新增一條「桌面測試套件於乾淨環境全綠」的交付基準需求）。
- Affected code:
  - New: apps/desktop/vitest.setup.ts
  - Modified: apps/desktop/vitest.config.ts、apps/desktop/package.json、package-lock.json、apps/desktop/src/store.ts、apps/desktop/src/App.tsx、apps/desktop/src/__tests__/store.test.ts
  - Removed: 無

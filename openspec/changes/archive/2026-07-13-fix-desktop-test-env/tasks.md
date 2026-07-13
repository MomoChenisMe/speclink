## 1. 補齊 Web Storage 全域（需求「桌面測試套件於乾淨環境全綠」）

- [x] 1.1 於 `apps/desktop/package.json` 的 devDependencies 加入 `jsdom`（對齊現行 `25.0.1`），使測試環境宣稱的 jsdom 成為直接依賴而非依賴 hoisting（滿足需求「直接宣告 jsdom 為 devDependency」）；更新 `package-lock.json` 使 `npm ci` 同步（無 "Missing from lock file"）。驗證：`npm ci --prefix apps/desktop`（或 workspace 等效）以 exit 0 完成。
- [x] 1.2 新增 `apps/desktop/vitest.setup.ts`：以一個真實 jsdom `Window` 的 `Storage`，在測試啟動前將 `localStorage` 與 `sessionStorage` 掛到 `globalThis`，並於每個測試檔起始清空，確保測試檔間狀態不殘留——即需求「測試環境提供 Web Storage 全域」的實作。（jsdom 25 本體提供 `window.localStorage`，vitest 3.2 的 jsdom 環境未暴露它，此 setup 補上該全域。）
- [x] 1.3 於 `apps/desktop/vitest.config.ts` 的 `test` 區塊加入 `setupFiles` 指向 1.2 的 setup 檔，使其於所有 desktop 測試前執行。不更動既有的 `environment: "jsdom"` 與 `globals: true`。

## 2. 修復搜尋去抖生命週期漏洞（使需求「桌面測試套件於乾淨環境全綠」的 exit 0 為決定性）

- [x] 2.1 （TDD 紅）於 `apps/desktop/src/__tests__/store.test.ts` 新增測試：`setBoardQuery` 排程 200ms 去抖後呼叫 `disposeSearch()`，`vi.advanceTimersByTimeAsync(300)` 後 `searchWorkspace` 未被呼叫（漏出的 timer 已取消、在途回填作廢）。此測試先失敗（`disposeSearch` 尚未存在）。
- [x] 2.2 （TDD 綠）於 `apps/desktop/src/store.ts` 的 `AppState` 與 `createAppStore` 回傳新增 `disposeSearch()`：清 `searchTimer`（若在途則 `clearTimeout` 並置 `null`）並前進 `searchSeq` 作廢在途回填；於 `apps/desktop/src/App.tsx` 既有的 `[useStore]` 生命週期 effect 之 cleanup 卸載時呼叫 `disposeSearch()`。使 2.1 轉綠、根除在途去抖於卸載後觸發的未處理例外。

## 3. 驗證需求「桌面測試套件於乾淨環境全綠」的 Scenario

- [x] 3.1 驗證需求「桌面測試套件於乾淨環境全綠」的 Scenario「乾淨環境 desktop 測試全綠」與「測試環境提供 Web Storage 全域」：`npm test -w apps/desktop` 以 exit 0 **決定性**完成（連續重複執行 ≥20 次無偶發非零），`src/__tests__/workspace.test.ts`（15）、`src/__tests__/App.test.tsx`（27）與其餘既有測試全數通過，無 `Cannot read properties of undefined (reading 'clear'/'setItem'/'then')` 等未處理例外；`localStorage`／`sessionStorage` 的 `setItem`／`getItem`／`clear` 語意正確。
- [x] 3.2 驗證 Scenario「test:all 貫穿 desktop 步驟不中止」：`npm run test:all` 不再於 `apps/desktop` 步驟中止，續行至 `crates/speclink-node` 步驟；`packages/ui`（280）與 `crates/speclink-node` 既有測試維持全綠，無新增失敗。
- [x] 3.3 確認外科手術邊界：`git diff --stat` 涵蓋 `apps/desktop/vitest.setup.ts`（新增）、`apps/desktop/vitest.config.ts`、`apps/desktop/package.json`、`package-lock.json`，以及本次去抖生命週期修復的 `apps/desktop/src/store.ts`、`apps/desktop/src/App.tsx`、`apps/desktop/src/__tests__/store.test.ts`（新增測試）；未修改任何既有測試斷言、未擴及與此紅燈無關的其他產品行為。

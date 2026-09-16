## 1. 主內容區改為 flex 直欄（apps/desktop）

- [ ] 1.1 落實規格「指令檔過期提示」的「提示 SHALL 只佔用自身高度」約束：主內容區 main 改為 flex 直欄，技能檔提示的包裹層加 shrink-0；提示存在時，看板、規格頁、手冊頁、已封存頁的根節點（各自已帶 h-full min-h-0）縮到 main 扣除提示後的剩餘高度，底緣貼齊 main 底緣、不再被 overflow-hidden 裁切；設定頁與專案設定頁的 main 仍為 overflow-y-auto 整頁捲動，提示與內容一起捲。手冊視圖的提示包裹層維持既有的 px-5 pt-5 自補內距。影響路徑：apps/desktop/src/App.tsx。驗證：任務 1.2 的新測試與既有 describe「main content scroll containment（主內容區捲動約束）」全部通過（`npm test -w @speclink/desktop`）。 <!-- speclink-task:tsk_01M2M39Y9XQV61QFQ31SDSP3FY -->
- [ ] 1.2 在 apps/desktop/src/__tests__/App.test.tsx 的 describe「main content scroll containment（主內容區捲動約束）」新增一條測試：store 帶 assetPrompt（kind 為 stale、fileCount 為 3）渲染看板時，main 的 className 同時含 flex、flex-col 與 overflow-hidden，data-testid 為 asset-prompt 的元素其父層 className 含 shrink-0；切到設定頁時 main 的 className 含 overflow-y-auto 且不含 overflow-hidden。驗證：`npm test -w @speclink/desktop` 綠，且該測試在任務 1.1 之前執行會紅（先寫測試再改 class 可確認）。 <!-- speclink-task:tsk_01M2M39Y9XMPCYVRES1WREVXRH -->

## 2. 實機確認與收尾

- [ ] [M] 2.1 以 `npm run dev -w @speclink/desktop` 與 `npm run tauri -w @speclink/desktop -- dev`（或安裝版）開啟一個技能檔為舊版的本地專案，讓技能檔提示出現；若版本更新通知列未同時出現，把視窗高度縮到約 700px 模擬同等高度。逐頁確認：看板「提案中」欄捲到底時最後一張卡連同進度列完整可見；規格頁與已封存頁的換頁控制列可見；手冊頁三欄底緣貼齊主內容區底緣；專案設定頁提示與內容一起整頁捲動。驗證：以上四項肉眼確認皆成立，並回報一張看板截圖。 <!-- speclink-task:tsk_01M2M39Y9XVZ79B79CNH49JHTP -->
- [ ] 2.2 執行 scripts 全組守門確認本次沒有動到詞彙、對照與連結面：`node --test scripts/*.test.mjs scripts/*/*.test.mjs` 全綠；同時再跑一次 `npm test -w @speclink/desktop` 確認任務 1 的測試仍綠。影響路徑：無新增檔案；只確認 apps/desktop/src/App.tsx 與 apps/desktop/src/__tests__/App.test.tsx 兩檔在 git status 中。 <!-- speclink-task:tsk_01M2M39Y9XHM623DP6VA03SME7 -->

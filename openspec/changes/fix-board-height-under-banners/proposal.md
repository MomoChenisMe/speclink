## Problem

桌面 app（apps/desktop）在專案分頁出現「技能檔提示」（指令檔過期、缺失或較新）時，看板、規格頁、手冊頁與已封存頁的內容區底部被視窗底緣裁掉。看板每一欄的內部捲軸捲到底時，最後一張卡片仍在畫面外；同時出現「有新版本」橫幅時畫面更矮、問題更明顯。目標使用者是透過桌面 app 操作看板的開發者；情境是升版後首次開啟舊專案（提示必出現）。

## Root Cause

主內容區 main 是一般區塊排版（非 flex 直欄）並帶 overflow-hidden。技能檔提示掛在 main 內頂部佔掉一段高度後，四個視圖的根節點仍以 h-full 取 main 的 100% 高度，被提示往下推的那截落在 main 可視範圍外、被 overflow-hidden 裁掉。版本橫幅在最外層 h-screen flex 直欄內、shrink-0，main 隨之縮小，不是元兇。

## Proposed Solution

只修容器：把 main 改成 flex 直欄，技能檔提示的包裹層 shrink-0，四個視圖的根節點（已各自帶 min-h-0）在直欄內自動縮到「main 扣掉提示」的剩餘高度。各視圖元件、各欄內部捲軸、提示的位置與捲動釘選、版本橫幅全部不動。desktop-app 規格「指令檔過期提示」補一個 scenario，釘住「提示存在時內容區不被裁切」。

## Non-Goals

- 不把技能檔提示搬到 header 上方與版本橫幅並排：違反規格「分頁內容頂部、per 專案、非阻斷」的承諾。
- 不合併或縮小同時出現的兩條橫幅：使用者明示不要，畫面變矮是合理佈局不是裁切。
- 不只修 KanbanBoard 根節點：規格頁、手冊頁、已封存頁同樣受影響，修容器一次到位。
- 不動 packages/ui 的元件：server-web 共用同一批元件，其版面不在本次範圍。
- 不處理「專案設定頁的提示包裹層可能讓捲動釘選失效」：那是 2026-09-02 手冊頁變更加入包裹層後的另一個問題，本次只記錄不修。

## Success Criteria

- 技能檔提示與版本橫幅同時出現、提案中欄有 3 張卡時，欄內捲到底可完整看到第 3 張卡的進度列，不被視窗底緣裁掉。
- 規格頁與已封存頁在提示存在時，清單底部的換頁控制列完整可見。
- 手冊頁在提示存在時，三欄底緣貼齊 main 底緣，不被裁掉。
- 設定頁與專案設定頁行為不變：整頁縱向捲動。
- App.test.tsx 新增斷言：store 帶 assetPrompt 時 main 的 class 含 flex 與 flex-col，提示包裹層含 shrink-0；既有「主內容區捲動約束」測試維持綠。
- 相容性影響：無 CLI、無 --json、無設定欄位、無技能檔變動；影響的 app 只有 apps/desktop。

## Impact

- Affected specs: desktop-app（「指令檔過期提示」補 scenario，Requirement 正文不變）
- Affected code:
  - Modified: apps/desktop/src/App.tsx（main 的 class 與提示包裹層 class）
  - Modified: apps/desktop/src/__tests__/App.test.tsx（新增版面斷言）
  - New: （無）
  - Removed: （無）
- 掃描既有規格：desktop-app「指令檔過期提示」「指令檔過期提示捲動釘選」「桌面自動更新」「清單最新在前與換頁瀏覽」相關；皆未承諾提示存在時內容區的高度，故補 scenario 而非改 Requirement。
- 來源討論：board-height-under-banners

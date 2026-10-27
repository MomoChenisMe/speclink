## 1. 重編號引擎側（design D2 重編號語意；design D3 重編號落點）

- [x] 1.1 紅：於 apps/desktop/core/src/manage.rs 撰寫重編號純函式與 move_task_at 的失敗測試——組內移動後前綴依「群組編號.組內序」重寫（spec Example「組內移動重編號」的甲乙丙值逐列斷言）、跨群組搬移取得新群組編號且兩群組各自重排、無「數字.數字」前綴的任務行逐字元保留、標題無數字前綴之群組其下任務不重編號、群組標題與非 checkbox 行逐字元不變、set_task_done 勾選不觸發重編號、越界 from/to 維持既有 Err 且檔案不動。驗證：cargo test -p speclink-desktop-core 出現預期紅燈。
- [x] 1.2 綠：實作重編號純函式（吃行陣列回傳改寫後行陣列），move_task_at 搬行成功後套用、一次寫回。驗證：1.1 測試全綠、cargo test -p speclink-desktop-core 全綠。
- [x] 1.3 對重編號的前綴比對與參數處理跑 sharp-edges 快查（Scoundrel/Lazy/Confused 三視角：惡意前綴樣式、空群組、零任務檔），逐項記錄結論，發現的尖銳邊以紅綠循環修正。驗證：結論記錄於實作對話、cargo test 全綠。

## 2. 拖放介面（design D1 把手拖曳；design D4 onReorder 收斂；design D5 寫回重載）

- [x] 2.1 紅：撰寫 packages/ui 失敗測試，涵蓋 spec 需求「任務清單拖放排序與自動重編號」的 jsdom 可驗部分——每任務列渲染 ⠿ 把手（aria-label「拖曳任務 N」）、上下箭頭按鈕不存在、readOnly 不渲染把手、TaskListProps 的 onReorder(from, to) 接線（以元件回呼直接觸發斷言）、RichDetailDrawer 拖放落點轉發 onMoveTask(change, from, to) 且 resolve 後重讀 tasks.md。驗證：npm test -w packages/ui 出現預期紅燈。
- [x] 2.2 綠：TaskList 改用 @dnd-kit/sortable（verticalListSortingStrategy、listeners 只綁把手、PointerSensor distance 8＋KeyboardSensor、DragOverlay 浮起列）、移除上下箭頭與 onMove、新增 onReorder；RichDetailDrawer 刪除逐格 handleMove 改為直接轉發 from/to；packages/ui/package.json 加 @dnd-kit/sortable。驗證：2.1 測試全綠、npm test -w packages/ui 與 -w apps/desktop 全綠。

## 3. 整合驗證

- [x] 3.1 全套自動化：cargo test -p speclink-desktop-core、npm test -w packages/ui、npm test -w apps/desktop 全綠；git diff 確認 speclink-core／speclink-cli／SpeclinkDataSource 介面零變更。驗證：全部通過。
- [x] 3.2 真實視窗驗證（cargo build --release -p speclink-desktop 前先關閉執行中 exe；操作前確認使用者沒在使用螢幕）：實拖 ⠿ 把手完成組內與跨群組搬移→畫面順序與編號即時正確、tasks.md diff 驗證重編號符合 spec 各 Scenario；在核取方塊上點擊（8px 內）→勾選切換無拖曳；封存展開的任務分頁無把手；長清單拖曳跨捲動不破版。驗證：每項有截圖或觀察記錄。

## 4. 組界修正（design D6 標題入讓位序列與組首落點；design D7 moveTask 側別）

- [x] 4.1 紅：於 apps/desktop/core/src/manage.rs 撰寫 move_task_at 側別的失敗測試——before=Some(true) 且目標為相鄰群組組首時，任務插於目標行之前、成為該群組第一個任務並重編號為組首（spec Example「標題落點成組首」的乙丙丁值逐列斷言）；before=None 維持方向推斷（既有測試不變）；before=Some(false) 明確插於目標行之後；越界 from/to 帶 before 仍回 Err 且檔案不動。驗證：cargo test -p speclink-desktop-core 出現預期紅燈。
- [x] 4.2 綠：move_task_at 簽名加 before: Option<bool>；apps/desktop/src-tauri/src/lib.rs 的 move_task command、apps/desktop/src/adapter/tauriDataSource.ts、packages/ui/src/adapter.ts 的 SpeclinkDataSource.moveTask 對應加可選參數（省略時行為與修訂前一致）。驗證：4.1 測試全綠、既有 Rust 與 npm 測試不破。
- [x] 4.3 紅：撰寫 packages/ui 失敗測試——TaskList 的 sortable 序列含群組標題項（不可拖）、onDragEnd 遇標題落點時以 onReorder(from, 組首任務 ordinal, true) 觸發（元件回呼直測或 handler 單元測試）、over=空群組標題不觸發 onReorder、RichDetailDrawer 把 before 轉發至 onMoveTask 第四參數。驗證：npm test -w packages/ui 出現預期紅燈。
- [x] 4.4 綠：TaskList 群組標題以 useSortable disabled 項加入 SortableContext（id 與任務 ordinal 不衝突）、onDragEnd 解析標題落點→（組首 ordinal, before=true）、RichDetailDrawer 與 App.tsx 傳遞側別。驗證：4.3 測試全綠、npm test -w packages/ui 與 -w apps/desktop 全綠。
- [x] 4.5 真實視窗驗證（cargo build --release -p speclink-desktop 前先關閉執行中 exe；操作前確認使用者沒在使用螢幕）：重現原報告操作——拖 1.6 向群組 2：讓位過程 2.1 的視覺不穿越「## 2」標題（拖曳中截圖）；放開於標題上→檔案 diff 顯示該任務成為 2.1 組首；放開於原 2.1 任務上→成為 2.2；核取方塊點擊與組內拖放回歸。驗證：每項有截圖或 tasks.md diff 記錄，行為與 spec 各 Scenario 一致。
- [x] 4.6 紅：resolveDropTarget 雙向標題落點的失敗測試——active 在標題下方（組首自己或更深）拖到該標題→解析為（上一群組末任務 ordinal, before=false）；上一側無任務（標題前是檔首或另一標題）→ null；既有「上方來→組首 before=true」不變（spec Example「標題落點雙向」兩列逐列斷言）。驗證：npm test -w packages/ui 出現預期紅燈。
- [x] 4.7 綠：resolveDropTarget 依 active 與標題的相對位置雙向解析（items 索引比較），既有測試不破。驗證：4.6 測試全綠、npm test -w packages/ui 全綠。
- [x] 4.8 真實視窗驗證（cargo build --release -p speclink-desktop 前先關閉執行中 exe；操作前確認使用者沒在使用螢幕）：重現回報操作——把任務拖到「## 2」標題成為 2.1 後，再拖回同一標題→回到群組 1 末位（tasks.md diff 佐證雙向）；組首落點與組內拖放回歸。驗證：diff 與截圖記錄。


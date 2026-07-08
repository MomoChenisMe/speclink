## 1. 批次寫回動詞（apps/desktop/core → src-tauri，TDD）

- [x] 1.1 撰寫批次函式測試（紅）：於 apps/desktop/core/src/manage.rs 的 #[cfg(test)] 模組新增案例，對應規格「任務分頁提供批次操作工具列」的寫回語意——①done=true 將全部任務標為完成且僅單次寫檔 ②done=false 全部取消勾選、不蓋開工章、不記 touched ③未開工變更 done=true 蓋開工章一次 ④目標狀態已達成時冪等成功不改檔。驗證：cargo test -p speclink-desktop-core --lib 新案例全數失敗（紅）
- [x] 1.2 實作批次函式與指令（綠，design D1：批次動詞單指令雙用）：apps/desktop/core/src/manage.rs 實作一次讀檔一次寫回的批次函式，apps/desktop/src-tauri/src/lib.rs 新增並註冊 set_all_tasks 指令。驗證：cargo test -p speclink-desktop-core --lib 全綠且既有測試無退化

## 2. 前端資料介面（TDD）

- [x] 2.1 擴充 SpeclinkDataSource（紅→綠）：apps/desktop/src/__tests__/tauriDataSource.test.ts 先斷言 setAllTasks(change, done) 以正確參數 invoke set_all_tasks（紅）；再於 packages/ui/src/adapter.ts 的 SpeclinkDataSource 介面與 apps/desktop/src/adapter/tauriDataSource.ts 補實作（綠）。驗證：npm test -w apps/desktop 全綠

## 3. 任務工具列（packages/ui，TDD）

- [x] 3.1 撰寫工具列測試（紅）：packages/ui/src/__tests__/taskList.test.tsx 新增案例，對應規格「任務分頁提供批次操作工具列」——①三鍵渲染 ②全部已完成／重置觸發對應回呼 ③全部任務完成時「全部已完成」與「下一個未完成」disabled ④readOnly 不渲染工具列 ⑤「下一個未完成」與 n 鍵使第一個未完成任務捲入可視（jsdom mock scrollIntoView）並帶高亮 class。驗證：npm test -w packages/ui 新案例全數失敗（紅）
- [x] 3.2 實作工具列（綠，design D2：下一個未完成純前端）：packages/ui/src/components/TaskList.tsx 清單頂部渲染工具列、接批次回呼與前端定位高亮；packages/ui/src/components/RichDetailDrawer.tsx 與 apps/desktop/src/App.tsx 接 setAllTasks 寫回（批次執行期間工具列與清單短暫 disabled）。驗證：npm test -w packages/ui 與 npm test -w apps/desktop 全綠

## 4. 勾選樂觀更新（packages/ui，TDD）

- [x] 4.1 撰寫樂觀更新測試（紅）：packages/ui/src/__tests__/richDrawer.test.tsx 新增案例，對應規格「勾選任務即時回饋」——①勾選後寫回 promise 未完成時 checkbox 已呈勾選態 ②寫回 reject 時回滾原狀態並顯示單行錯誤 ③寫回進行中清單其餘 checkbox 仍可勾選（無 pointer-events 鎖）。驗證：npm test -w packages/ui 新案例全數失敗（紅）
- [x] 4.2 實作樂觀更新（綠，design D3：樂觀更新落在抽屜層 tasksMd＋D4：busy 指標鎖移除、批次操作例外）：packages/ui/src/components/RichDetailDrawer.tsx 的勾選處理改為本地翻轉 tasksMd 對應 checkbox 標記後再發寫回、失敗還原並顯示錯誤；packages/ui/src/components/TaskList.tsx 移除單發勾選的 busy 指標鎖、批次操作維持短暫 disabled 例外（拖曳讓路與世代重載讓路機制保留）。驗證：npm test -w packages/ui 全綠且既有測試無退化

## 5. 整合驗證（真實視窗）

- [x] 5.1 真實視窗驗證互動：關閉執行中的 exe 後 cargo build --release -p speclink-desktop，啟動 app 開變更抽屜任務分頁實測——勾選即時反映無可感知延遲、連續勾選不互鎖、「全部已完成」後進度 100% 且 tasks.md 全 [x]、「重置任務」後 0% 且無開工章新增、「下一個未完成」與 n 鍵捲動高亮、封存檢視無工具列（操作前先確認使用者未在使用螢幕）。驗證：逐項對照規格「任務分頁提供批次操作工具列」「勾選任務即時回饋」的場景皆符合，npm test -w packages/ui、npm test -w apps/desktop、cargo test -p speclink-desktop-core --lib 全綠

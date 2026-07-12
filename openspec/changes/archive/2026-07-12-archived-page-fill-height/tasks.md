## 1. 已封存頁填滿高度版面（spec「清單最新在前與換頁瀏覽」）

- [x] 1.1 於 packages/ui/src/__tests__/archivedList.test.tsx 撰寫版面結構測試（先紅）：斷言 ArchivedList 根容器為填滿高度的 flex 直欄（含 h-full）；卡片清單容器帶 overflow-y-auto 的內部捲動樣式；ListPager 渲染於清單捲動容器之外、作為直欄末端手足（DOM 上換頁控制列不在捲動容器節點內）；點下一頁後內部捲動容器的 scrollTop 被重置為 0。驗證：npm test -w packages/ui 顯示新測試失敗、其餘既有測試通過
- [x] 1.2 實作 packages/ui/src/components/ArchivedList.tsx：根容器改 h-full flex 直欄，搜尋框與子頁籤列固定頂部，卡片清單容器改 flex-1 min-h-0 overflow-y-auto，ListPager 移出捲動容器固定於直欄底部（兩子頁籤各自保留獨立 ListPager 與頁碼）；換頁「捲回頂部」由 topRef.scrollIntoView() 改為重置內部捲動容器捲動位置。驗證：npm test -w packages/ui 全綠（含 1.1 新測試與既有 archivedList、listPager 測試）

## 2. 規格頁比照填滿高度版面

- [x] 2.1 於 packages/ui/src/__tests__/specList.test.tsx 撰寫同款版面結構測試（先紅）：斷言 SpecList 根容器填滿高度、清單容器內部捲動、ListPager 位於捲動容器之外、換頁後捲動位置歸零。驗證：npm test -w packages/ui 顯示新測試失敗、其餘通過
- [x] 2.2 實作 packages/ui/src/components/SpecList.tsx：套用與 ArchivedList 相同的 h-full flex 直欄＋內部捲動＋ListPager 沉底結構，換頁捲回頂部改為重置內部捲動容器。驗證：npm test -w packages/ui 全綠

## 3. 桌面殼主內容區高度約束

- [x] 3.1 於 apps/desktop/src/__tests__/App.test.tsx 撰寫測試（先紅）：boardView 為已封存頁或規格頁時，主內容區（main）帶 overflow-hidden 而非 overflow-y-auto，使內部捲動容器高度受視窗約束；設定頁維持整頁捲動（overflow-y-auto）不受影響。驗證：npm test -w apps/desktop 顯示新測試失敗、其餘通過
- [x] 3.2 實作 apps/desktop/src/App.tsx：主內容區的捲動類別由「僅看板 overflow-hidden、其餘 overflow-y-auto」改為「看板、規格頁、已封存頁皆 overflow-hidden；設定頁維持 overflow-y-auto」。驗證：npm test -w apps/desktop 全綠

## 4. 建置與真實視窗驗證

- [x] 4.1 前端與桌面殼建置通過：npm run build -w apps/desktop 成功產出 dist，cargo build --release -p speclink-desktop 編譯成功（建置前確認無執行中的桌面 app 進程）。驗證：兩指令 exit code 0
- [x] 4.2 真實視窗手動驗證（jsdom 測不出實際版面）：啟動 release 版桌面 app 開啟含超過 20 筆封存變更的專案，斷言——已封存頁不捲動即可看到並點擊底部換頁控制列；捲動卡片清單時搜尋框、子頁籤列與換頁控制列固定不動；點下一頁清單回到頂部；「討論」子頁籤與規格頁行為相同；清單不足一頁時無換頁控制列且版面無異常。驗證：逐項截圖確認符合 spec「清單最新在前與換頁瀏覽」的換頁控制列常駐行為

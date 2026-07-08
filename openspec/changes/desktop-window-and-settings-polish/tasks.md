## 1. 視窗預設（規格「視窗預設尺寸與置中」）

- [ ] 1.1 落實規格需求「視窗預設尺寸與置中」——修改 apps/desktop/src-tauri/tauri.conf.json 視窗項為 1440×900 並開啟置中：交付行為＝啟動視窗邏輯尺寸 1440×900 且於主螢幕置中、不記憶上次狀態。驗證：npm run build -w apps/desktop 後 cargo build --release -p speclink-desktop（先關閉執行中的 exe），啟動 release exe 以真實視窗量測尺寸與螢幕位置並截圖

## 2. 側欄設定沉底（規格「側欄導覽結構」）

- [ ] 2.1 對應規格需求「側欄導覽結構」——於 apps/desktop/src/__tests__/App.test.tsx 撰寫側欄結構測試：設定導覽項為側欄最末子元素且與頂部三項（變更、規格、已封存依序）之間存在彈性區隔；設定項的切頁與高亮語意不變。驗證：npm test -w apps/desktop 新斷言失敗（紅）
- [ ] 2.2 於 apps/desktop/src/App.tsx 調整側欄佈局使設定項沉底（彈性空間推底，不加新視覺元素，design D5）。驗證：npm test -w apps/desktop 2.1 斷言轉綠且既有導覽測試無退化

## 3. 設定頁三頁簽重構（規格「設定頁圖形化讀寫兩層設定」「設定頁編輯專案說明與產出規則」）

- [ ] 3.1 對應規格需求「設定頁圖形化讀寫兩層設定」——於 apps/desktop/src/__tests__/settingsView.test.tsx 撰寫頁簽組織測試：頁簽依序 config.yaml、.speclink.yaml、本機設定且預設落在 config.yaml；各簽卡片歸屬（config.yaml＝專案說明、產出規則、產出政策；.speclink.yaml＝AI 工具；本機設定＝介面語言）；config.yaml 與 .speclink.yaml 簽首等寬字檔案路徑註記、本機設定簽「僅存於此裝置」註記；任一層解析失敗時對應頁簽標籤帶警示點、該簽表單停用、本機設定簽不受影響。驗證：npm test -w apps/desktop 新斷言失敗（紅）
- [ ] 3.2 對應規格需求「設定頁編輯專案說明與產出規則」——於 apps/desktop/src/__tests__/settingsView.test.tsx 撰寫拆卡編輯態測試：專案說明卡與產出規則卡各持編輯態（一卡編輯中另一卡唯讀可用）、取消僅還原本卡、各卡儲存僅寫對應鍵（僅存產出規則時 context 逐字元不變）；既有寫入語意測試（未觸及鍵保留、清空移除鍵、行序即寫入順序、固定鍵分節、保留字元加引號）逐案保留、僅更新呈現層選擇器。驗證：npm test -w apps/desktop 新斷言失敗（紅）
- [ ] 3.3 重構 apps/desktop/src/views/SettingsView.tsx 為三頁簽結構（design D1–D3）並於 apps/desktop/src/i18n/messages.ts 補齊新鍵（本機設定簽標籤、產出政策／AI 工具／介面語言等卡標題，zh-TW 與 en 雙語齊備；檔名頁簽標籤為字面常數不進字典）：交付行為＝3.1 與 3.2 描述的頁簽組織與獨立編輯行為，且所有檔案寫入效果與既有規格場景一致。驗證：npm test -w apps/desktop 全綠（含 messages 鍵集合相等測試）
- [ ] 3.4 重構收整：僅於卡片框架程式碼實際重複時抽出共用片段，無重複則記錄不動。驗證：npm test -w apps/desktop 維持全綠

## 4. 詞彙例外與真實視窗驗證

- [ ] 4.1 於 openspec/LANGUAGE.md 記錄明文例外：檔名（config.yaml、.speclink.yaml）得作為設定頁頁簽標籤，註明與「工程詞不進使用者文案」原則的刻意抵觸與討論出處（desktop-window-and-settings-polish）。驗證：speclink language show 輸出呈現該例外
- [ ] 4.2 真實視窗驗證全案（jsdom 測不出的部分；操作前確認使用者未在使用螢幕）：啟動 release exe 截圖確認①視窗 1440×900 置中②側欄設定沉底③三頁簽實點切換、專案說明卡進出編輯、警示點於改壞 config.yaml 後顯現（驗畢還原檔案）。驗證：各項截圖逐一檢視斷言

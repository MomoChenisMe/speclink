## 1. 修改時間資料（apps/desktop/core → src-tauri，TDD）

- [x] 1.1 撰寫清單查詢 mtime 測試（紅）：於 apps/desktop/core/src/query.rs 的 #[cfg(test)] 模組新增案例，對應規格「桌面 app 呈現 change 與 spec 的清單與內容」的呈現層輔助欄位——規格清單查詢對每個 spec 回傳 modifiedAt（自 spec.md 檔案系統 mtime 衍生的 YYYY-MM-DD 字串）、mtime 不可得時欄位缺席（design D2：修改時間走清單查詢單趟帶回）。驗證：cargo test -p speclink-desktop-core --lib 新案例失敗（紅）
- [x] 1.2 實作 mtime 欄位（綠）：apps/desktop/core/src/query.rs 以標準庫 metadata 讀取衍生 modifiedAt（Option、serde camelCase），apps/desktop/src-tauri/src/lib.rs 的 list_specs payload 隨之帶出。驗證：cargo test -p speclink-desktop-core --lib 全綠且既有測試無退化

## 2. 前端型別與 wiring 基礎（TDD）

- [x] 2.1 SpecItem 型別擴充（紅→綠）：apps/desktop/src/__tests__/tauriDataSource.test.ts 先斷言 listSpecs 回傳項含 modifiedAt 可選欄位（紅）；再於 packages/ui/src/adapter.ts 的 SpecItem 增 modifiedAt?: string | null 並確認 apps/desktop/src/adapter/tauriDataSource.ts 型別通過（綠）。驗證：npm test -w apps/desktop 全綠
- [x] 2.2 主視圖新增 specs 態（紅→綠）：apps/desktop/src/__tests__/store.test.ts 先斷言 boardView 可切至 specs 並保留 specs 清單狀態（紅）；再於 apps/desktop/src/store.ts 擴充 BoardView 聯集（綠）。驗證：npm test -w apps/desktop 全綠

## 3. SpecList 元件（packages/ui，TDD）

- [x] 3.1 撰寫 SpecList 測試（紅）：新增 packages/ui/src/__tests__/specList.test.tsx，對應規格「規格頁提供清單、搜尋與展開檢視」——①清單渲染各卡名稱與相對修改時間（今天／昨天／N 天前；modifiedAt 缺席時該行不渲染）②搜尋子字串過濾（大小寫不敏感）與清空還原 ③無結果與無 spec 各顯示空狀態 ④點標題展開才呼叫內容載入（mock loadDocument 斷言呼叫時機）、首次展開呈載入態、再點縮合 ⑤複製名稱鈕寫入剪貼簿並顯示回饋。驗證：npm test -w packages/ui 新案例全數失敗（紅）
- [x] 3.2 實作 SpecList（綠，design D1：SpecList 元件進 packages/ui＋D3：搜尋僅名稱過濾＋D4：展開延遲載入）：新增 packages/ui/src/components/SpecList.tsx（props 注入 specs 清單與 loadDocument，展開內容留元件狀態、refreshGen 遞增時清空快取），packages/ui/src/index.ts 匯出；相對時間邏輯自 RichDetailDrawer 的 relativeDays 抽為共用。驗證：npm test -w packages/ui 全綠且既有測試無退化

## 4. App 接線

- [x] 4.1 導覽接上規格頁（紅→綠）：apps/desktop/src/__tests__/App.test.tsx 先斷言點導覽「規格」後主內容出現規格頁清單且導覽項 active（紅）；再於 apps/desktop/src/App.tsx 給「規格」NavItem 接 onClick 切 boardView、主內容渲染 SpecList 並注入 store.specs 與 dataSource.getSpecDocument、傳入 refreshGen（綠）。驗證：npm test -w apps/desktop 全綠

## 5. 整合驗證（真實視窗）

- [x] 5.1 真實視窗驗證規格頁：關閉執行中的 exe 後 cargo build --release -p speclink-desktop，啟動 app 實測——點「規格」進頁、卡片列出全部正典 spec 與相對修改時間、搜尋過濾與清空、展開顯示全文（載入態→渲染）、縮合、複製名稱回饋、外部改 spec 檔後清單與內容更新（操作前先確認使用者未在使用螢幕）。驗證：逐項對照規格「規格頁提供清單、搜尋與展開檢視」場景皆符合，npm test -w packages/ui、npm test -w apps/desktop、cargo test -p speclink-desktop-core --lib 全綠

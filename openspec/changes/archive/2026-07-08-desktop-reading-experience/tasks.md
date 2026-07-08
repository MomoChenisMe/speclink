## 1. markdown 渲染行為（packages/ui，TDD）

- [x] 1.1 撰寫 Markdown 渲染測試（紅）：於 packages/ui/src/__tests__/components.test.tsx 新增案例，對應規格「raw HTML 不以原文呈現」與「markdown 內容保留文件結構呈現」——①來源兩行文字以單一換行分隔時，渲染結果分行呈現（斷言換行節點存在）②來源含 HTML 註解時，註解文字不出現於渲染結果 ③code fence 內的 HTML 標籤原文保留 ④渲染容器帶 prose 系 class。驗證：npm test -w packages/ui 新增案例全數失敗（紅），既有案例不受影響
- [x] 1.2 實作 Markdown 渲染行為（綠，design D4：渲染端過濾與換行）：packages/ui 新增 remark-breaks 依賴（packages/ui/package.json），packages/ui/src/components/Markdown.tsx 開 skipHtml、掛 remark-breaks、容器 class 由 markdown 改為 prose 系＋自有掛鉤 class。驗證：npm test -w packages/ui 1.1 案例轉綠且無既有測試退化
- [x] 1.3 任務清單字級對齊（紅→綠，design D5：任務字級對齊）：packages/ui/src/__tests__/taskList.test.tsx 先斷言任務文字與群組標題帶 16px 級距的 class（紅），再改 packages/ui/src/components/TaskList.tsx 任務文字 13px→16px、群組標題同步放大一級（綠）。驗證：npm test -w packages/ui 全綠

## 2. 樣式基盤與字體（apps/desktop）

- [x] 2.1 排版接管（design D1：typography 接管排版＋D2：prose token 映射）：apps/desktop 新增 @tailwindcss/typography devDependency（apps/desktop/package.json），apps/desktop/src/index.css 以 @plugin 啟用插件、將 --tw-prose-body／headings／links／bold／code／quotes／hr／th-borders 等變數映射到既有 oklch token，手工 .markdown 樣式縮為薄覆寫——保留 teal code chip、GFM 任務清單 checkbox 懸掛縮排與完成刪除線、表格橫向捲動。行為對應規格「markdown 內容保留文件結構呈現」：清單顯示符號與編號、段落間距可辨、基準字級 16px、深淺主題色彩皆取自 token。驗證：npm run build -w apps/desktop 成功；npm test -w apps/desktop 全綠
- [x] 2.2 字體打包（design D3：字體打包）：apps/desktop 新增 @fontsource-variable/noto-sans-tc 依賴（apps/desktop/package.json），apps/desktop/src/main.tsx 單一 import 可變字重樣式（woff2 檔族涵蓋全字重），apps/desktop/src/index.css 的 body font-family 改為 Noto Sans TC Variable 第一優先（Noto Sans TC、Segoe UI、system-ui 後備），等寬字體宣告不動。行為對應規格「介面文字以打包的 Noto Sans TC 呈現」：未安裝字體的機器以打包字體呈現介面與內容文字，無對外字體請求。驗證：npm run build -w apps/desktop 成功，且 dist 產物內含 noto-sans-tc 字體資產（檢視建置輸出清單）

## 3. 整合驗證（真實視窗）

- [x] 3.1 真實視窗驗證排版與字體：關閉執行中的 exe 後 cargo build --release -p speclink-desktop，啟動 app 開啟變更抽屜（提案／任務分頁）與討論抽屜（討論過程分頁）截圖檢視——逐項對照規格「介面文字以打包的 Noto Sans TC 呈現」「markdown 內容保留文件結構呈現」「raw HTML 不以原文呈現」的場景：列表符號與編號顯示、段落間距可辨、內文與任務文字 16px、字形為 Noto Sans TC、scaffold 註解消失；切換系統深色偏好再截圖確認排版一致（操作前先確認使用者未在使用螢幕）。驗證：截圖逐項對照三需求場景皆符合
- [x] 3.2 既有文件回歸抽查：以真實視窗對照 2-3 份既有變更與已封存文件的渲染（含英文段落與表格內容），確認 remark-breaks 未在非預期處產生換行、表格仍可橫向捲動。驗證：截圖內容審視無非預期呈現差異

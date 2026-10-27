## 1. Checkbox 原語與任務清單（packages/ui，TDD）

- [x] 1.1 撰寫勾選框主題化測試（紅）：packages/ui/src/__tests__/taskList.test.tsx 新增案例，對應規格「表單控制項與按鈕以主題化元件呈現」——①任務勾選框為 checkbox 角色的非 input 元素（aria-checked 反映完成態）②空白鍵切換觸發與點擊相同的 onToggle ③readOnly 時不可互動。驗證：npm test -w packages/ui 新案例全數失敗（紅），既有案例不受影響
- [x] 1.2 升級 Checkbox 原語並換用（綠，design D1：Checkbox 升級為 Radix 原語）：packages/ui 新增 @radix-ui/react-checkbox 依賴（packages/ui/package.json），重寫 packages/ui/src/components/ui/checkbox.tsx 為 Radix 版 shadcn 實作（主題邊框空框／主色底勾選圖示／disabled 半透明），packages/ui/src/components/TaskList.tsx 的任務列勾選框換用（aria-label「任務 N」、checked、disabled 條件、onCheckedChange 轉接不變），既有測試的 HTMLInputElement 斷言改為 role／aria-checked。驗證：npm test -w packages/ui 全綠且拖放、工具列、樂觀更新案例無退化

## 2. 初始化對話框（apps/desktop，TDD）

- [x] 2.1 工具多選換用 Checkbox（紅→綠）：apps/desktop/src/__tests__/App.test.tsx 先斷言初始化對話框的 claude／codex 選項為 checkbox 角色且可獨立切換、預設勾選 claude（紅）；再於 apps/desktop/src/App.tsx 換用 ui 勾選原語（綠）。驗證：npm test -w apps/desktop 全綠

## 3. Textarea 原語與設定頁（TDD）

- [x] 3.1 新建 Textarea 並換用（紅→綠，design D2：Textarea 為樣式化原生、無新依賴）：apps/desktop/src/__tests__/settingsView.test.tsx 先斷言專案說明與產出規則兩處多行輸入帶主題化 border／focus class（紅）；再新增 packages/ui/src/components/ui/textarea.tsx、packages/ui/src/index.ts 匯出，apps/desktop/src/views/SettingsView.tsx 兩處換用且受控值與儲存行為不變（綠）。驗證：npm test -w apps/desktop 全綠

## 4. 按鈕統一（design D3：按鈕收斂到既有變體、視覺近似不重設計）

- [x] 4.1 packages/ui 活路徑按鈕換變體：packages/ui/src/components/TaskList.tsx（工具列三鍵與拖曳把手——把手保留 attributes／listeners 展開）、RichDetailDrawer.tsx（複製名稱、全螢幕、來源討論與同源 chip）、ChangeCard.tsx、DiscussionColumn.tsx、ArchivedList.tsx 的 raw button 改用 ui 按鈕變體（icon 鈕 ghost＋icon 尺寸、文字鈕 ghost／outline＋sm），無障礙名稱、onClick、disabled 條件逐一不變。驗證：npm test -w packages/ui 全綠（既有 getByRole 查詢與拖放案例不改語意）
- [x] 4.2 apps/desktop 活路徑按鈕換變體：apps/desktop/src/App.tsx（NavItem、頂欄開啟專案、空狀態）、apps/desktop/src/components/ProjectTabs.tsx（分頁、關閉、新增）、apps/desktop/src/views/SettingsView.tsx 的 raw button 改用 ui 按鈕變體，行為與名稱不變。驗證：npm test -w apps/desktop 全綠；npm run build -w apps/desktop 成功
- [x] 4.3 收斂確認：以 grep 檢視 packages/ui/src/components 與 apps/desktop/src 的活路徑檔案不再出現 raw button 元素（死元件 ChangeBoard、ChangeList、ChangeListItem、DocumentTree、DocumentViewer 除外——明列於報告；markdown 渲染的 GFM 唯讀 checkbox 維持 CSS 樣式不在替換之列，見 design D4：GFM 唯讀 checkbox 維持 CSS）。驗證：grep 清單僅剩死元件與 ui 原語自身

## 5. 整合驗證（真實視窗）

- [x] 5.1 真實視窗驗證主題化控制項：關閉執行中的 exe 後 npm run build -w apps/desktop 與 cargo build --release -p speclink-desktop，啟動 app 逐項對照規格「表單控制項與按鈕以主題化元件呈現」場景——任務勾選框勾／未勾外觀（主色底＋勾圖示、非 OS 原生）、勾選與空白鍵切換照常寫回、初始化對話框多選、設定頁多行輸入主題樣式、Tab 聚焦按鈕 focus 環一致；切系統深色偏好複驗；拖曳把手實拖一次確認不退化（操作前先確認使用者未在使用螢幕）。驗證：截圖逐項符合，npm test -w packages/ui、npm test -w apps/desktop 全綠

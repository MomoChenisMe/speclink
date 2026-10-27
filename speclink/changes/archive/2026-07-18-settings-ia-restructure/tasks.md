## 1. 路由與導覽（TDD：先紅後綠）

- [x] 1.1 紅：`apps/desktop/src/__tests__/App.test.tsx` 新增側欄五導覽項案例——頂部依序「變更、規格、已封存、專案設定」、底部「設定」，點「專案設定」切頁且高亮；零分頁時點「設定」渲染應用程式設定頁（非空狀態頁）、點「專案設定」渲染空狀態引導頁（需求「側欄導覽結構」的零分頁行為）。此時應全數失敗 <!-- speclink-task:tsk_01KXTVAQ62KJ1JQCMQC9E8AC01 -->
- [x] 1.2 綠：`apps/desktop/src/store.ts` 的 `boardView` union 增加 `"project-settings"`；`apps/desktop/src/App.tsx` 側欄加入「專案設定」導覽項（`app.navProjectSettings` i18n 鍵，zh「專案設定」／en「Project Settings」，兩語系鍵集合相等）、`settings` 分支改為不依賴 activeSession 且先於零分頁 EmptyState 分支（design 決策 1：專案設定入側欄頂部群組、應用程式設定沉底；決策 2：路由狀態擴充而非新機制）。1.1 案例轉綠，`npm test -w apps/desktop` 全綠 <!-- speclink-task:tsk_01KXTVAQ62FTJKKFPHD4AH5FA8 -->

## 2. 頁面拆分（TDD：先紅後綠）

- [x] 2.1 紅：新增 `apps/desktop/src/__tests__/appSettingsView.test.tsx`（本機設定簽含介面語言卡與「僅存於此裝置」註記、伺服器簽渲染 ServersPanel、簽序本機設定→伺服器且預設本機設定）與 `apps/desktop/src/__tests__/projectSettingsView.test.tsx`（簽序 config.yaml→.speclink.yaml 預設 config.yaml、簽首等寬字路徑註記、`notice` 有值時整頁單一說明卡〔`data-testid="settings-unavailable"`〕且不發出 settings 讀取）——對應需求「設定頁圖形化讀寫兩層設定」的兩頁分工與「伺服器管理最小面」的頁籤位置，及 design 決策 4：簽序與預設簽。此時應全數失敗 <!-- speclink-task:tsk_01KXTVAQ62SHKRFJKW0XT4H63N -->
- [x] 2.2 綠：新增 `apps/desktop/src/views/AppSettingsView.tsx`（本機設定＋伺服器兩簽，收 localePref／onLocalePrefChange／trayPanelError／servers，內容自現行 SettingsView 對應區塊搬移）與 `apps/desktop/src/views/ProjectSettingsView.tsx`（config.yaml＋.speclink.yaml 兩簽，收 settings 與 `notice?: string`，含三卡、寫入驗證、解析失敗簽級警示，notice 有值即整頁說明卡並跳過 settings 讀取）——design 決策 3：檔案拆分與命名。2.1 案例轉綠 <!-- speclink-task:tsk_01KXTVAQ621J7PZH8Q162NY2FA -->
- [x] 2.3 既有 `settingsView.test.tsx` 案例依所屬頁面搬移至 2.1 兩檔（設定卡讀寫、解析失敗警示、tools 同步等案例逐一保留），四簽提示卡案例改寫為單卡斷言；刪除 `apps/desktop/src/views/SettingsView.tsx` 與 `apps/desktop/src/__tests__/settingsView.test.tsx`，App.tsx 改接兩個新檢視（remote 分頁以 `activeSession.locator.kind === "remote"` 傳 `notice`，沿用 `remote.settingsUnavailable` 文案；`workspaceSettingsNotice` prop 與其分支移除；design 決策 5：remote 專案設定呈現）。`npm test -w apps/desktop` 全綠且無殘留 SettingsView 引用（rg SettingsView 為零筆） <!-- speclink-task:tsk_01KXTVAQ624JMMDMK98WFRB02V -->

## 3. 收尾驗證

- [x] 3.1 回歸：`npm test -w apps/desktop` 與 `npm test -w packages/ui` 全綠；`npx tsc --noEmit`（apps/desktop）無新增錯誤（HEAD 既有錯誤不計） <!-- speclink-task:tsk_01KXTVAQ627TSVBMBRQJD6H07M -->
- [x] 3.2 手動 GUI 驗證（真實視窗）：（a）零分頁啟動 app → 側欄「設定」→ 伺服器簽開啟 remote workspace 成功、切至看板出現新分頁；（b）local 分頁「專案設定」三卡讀寫如常、改壞 config.yaml 後對應簽帶警示點；（c）remote 分頁「專案設定」整頁單一說明卡；（d）介面語言三選於應用程式設定頁即時生效 <!-- speclink-task:tsk_01KXTVAQ62DW04S42GH9Q7K2QN -->

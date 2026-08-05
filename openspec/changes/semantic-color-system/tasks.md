<!-- 依專案 TDD 慣例:守門測試先收緊(紅燈=現存違規清單),各表面批次「先改斷言、再實作」逐批轉綠。 -->

## 1. 語意色常數表與守門

- [x] 1.1 新增 packages/ui/src/tone.ts:SEMANTIC_TONE(inProgress=sky、success=emerald、warning=amber -600/dark 變體、danger=destructive)與 SEMANTIC_SURFACE(對應 border+bg 淡色組),表頭註記三紅分工(訊息與按鈕=destructive、品質站章=rose、delta=red);自 packages/ui/src/index.ts 匯出——契約:兩 app 可 import 取色;驗證:npm test -w @speclink/ui 全綠(消費落地由後續任務驗)。 <!-- speclink-task:tsk_01KZ5P0WAAZCJR5VRN5MM2WR2G -->
- [x] 1.2 [測試先行] packages/ui/src/__tests__/theme.test.ts 守門收緊(規格「介面狀態語意色分層」的守門場景):掃描 packages/ui/src、apps/desktop/src、apps/server-web/src 的 .ts/.tsx(排除 __tests__/dist),原生語意色階字面僅允許白名單檔(tone.ts、reviewStyle.tsx、DeltaBadges.tsx、stage.ts),違規失敗並列出檔名+class——契約:紅燈即現存違規清單(TrayPanel/App/ProjectTabs/RemoteWorkspaceRecovery/RemoteConflictDialog/InstructionUpdatePrompt/兩設定頁的 amber、ServersPanel teal-700 與 red-600、connectionLogin red-600、ChangeCard amber-500 等);驗證:該測試紅、其餘綠。 <!-- speclink-task:tsk_01KZ5P0WAABDSJS19PNXR965SX -->

## 2. packages/ui 修正批

- [x] 2.1 [測試先行] 更新 packages/ui 斷言錨定新契約:AnalyzePanel Warning 徽章=amber(同檔維度摘要同語意同色)、「驗證通過/零問題」=中性;tooltip 氣泡=bg-foreground text-background;HighlightText 搜尋高亮=琥珀 mark;討論抽屜輪次籤=中性;五處頭像=bg-muted;ChangeCard restale=amber-600+dark 變體;ChangeCard worktree 標示=sky(kanban.test.tsx 的 worktree marker 案例加色彩斷言:圖示帶 SEMANTIC_TONE.inProgress、不帶 text-primary/60);ReviewArchiveDialog「照樣帶走」=destructive——契約:新斷言紅燈;驗證:npm test -w @speclink/ui 顯示上述案例失敗。 <!-- speclink-task:tsk_01KZ5P0WAAHVV5K8BPYDCMHNKY -->
- [x] 2.2 實作 packages/ui 修正批(AnalyzePanel、ui/tooltip.tsx、HighlightText、DiscussionDrawer 輪次籤、DiscussionColumn 欄頭圖示/色條/計數收斂至 stage.ts 或中性、ReviewArchiveDialog、ChangeCard/RichDetailDrawer/DiscussionColumn/ArchivedList/DiscussionDrawer 頭像、ChangeCard worktree 標示由 text-primary/60 改 SEMANTIC_TONE.inProgress——抽屜分支+路徑維持中性不動),語意色一律 import tone.ts——契約:任務 2.1 轉綠、守門測試對 packages/ui 檔案零違規;驗證:npm test -w @speclink/ui 全綠(theme 守門除 desktop/server-web 殘留)。 <!-- speclink-task:tsk_01KZ5P0WAA6SW6VZXEK7Q84CTC -->

## 3. desktop 殼層修正批

- [x] 3.1 [測試先行+實作] 系統匣面板(apps/desktop/src/panel/TrayPanel.tsx):討論/已轉出分區標題圖示與計數徽章轉中性(生命週期三分區維持 STAGE_* 階梯)、recovery 卡依狀態分色(restoring=sky、error=destructive)、作用中非 ready 分頁改「主色外框表選取+列內語意色表狀態」、根層漸層轉中性、stale 列「重新登入」鈕改中性 outline——契約:規格「生命週期階梯與互動回饋豁免」場景成立;驗證:npm test -w apps/desktop 對應斷言先紅後綠。 <!-- speclink-task:tsk_01KZ5P0WAAQCZMSSSGSW152RC1 -->
- [x] 3.2 [測試先行+實作] 對話框與設定面(MigrationDialog 成功=emerald/進行=sky/eyebrow 中性;WorkspaceChooser 步驟標籤與 notice 中性;UpdateBanner 與 InstructionUpdatePrompt 底色中性+圖示語意色;ProjectTabs error=destructive/spinner=sky/ready 圖示中性;RemoteWorkspaceRecovery 還原=sky/失敗=destructive/needs-reauth 維持 amber;AppSettingsView 與 ProjectSettingsView 錯誤=destructive、有新版=sky、停用選項中性、政策衝突=amber 邊框;App.tsx stale 鈕中性 outline;RemoteConflictDialog 兩選項語彙統一)——契約:規格「錯誤紅/進行藍/成功綠」三場景成立;驗證:npm test -w apps/desktop 全綠。 <!-- speclink-task:tsk_01KZ5P0WAAFFWX8VGHS1XFDB50 -->
- [x] 3.3 ServersPanel 與 connectionLogin 的原生色階改 token(text-teal-700 已登入改中性、三處 text-red-600 改 text-destructive)——契約:守門測試對 apps/desktop 零違規;驗證:npx vitest run src/__tests__/theme.test.ts --root packages/ui 綠(desktop 部分)。 <!-- speclink-task:tsk_01KZ5P0WAA33CKSD470TAAKP36 -->

## 4. web 後台修正批

- [x] 4.1 [測試先行+實作] 狀態徽章語意色(OverviewPage/SystemPage 儲存健康、UsersPage 成員狀態與抽屜內憑證、CredentialsPage、AccountPage 工作階段)與揭示橫幅改 emerald(AccountPage PAT、UsersPage 邀請連結)——契約:規格「後台狀態徽章語意色」四場景成立、守門對 apps/server-web 零違規;驗證:npm test -w apps/server-web 全綠、npm run build -w apps/server-web 通過。 <!-- speclink-task:tsk_01KZ5P0WAAWPARTWZXCDH1QD2F -->

## 5. 死碼清理

- [x] 5.1 移除 ChangeBoard.tsx/ChangeList.tsx/ChangeListItem.tsx/DetailDrawer.tsx、index.ts 對應匯出(含 ListView 型別,移除前 grep 確認零消費端,有消費端則保留型別並記錄)、changeListItem.test.tsx 整檔、components.test.tsx 的 ChangeBoard 區塊、kanban.test.tsx 的 DetailDrawer 區塊——契約:@speclink/ui 匯出面無四元件、全套測試不因缺檔失敗;驗證:npm test -w @speclink/ui 全綠、grep 全 repo 無殘留 import。 <!-- speclink-task:tsk_01KZ5P0WAA27HGKP71R9VW2DSE -->

## 6. 整體驗證

- [ ] 6.1 全套驗證與手動走查:npm test -w @speclink/ui、npm test -w apps/desktop、npm test -w apps/server-web 全綠;npm run build -w apps/desktop 與 npm run build -w apps/server-web 通過;手動於深淺主題走查桌面 app(含系統匣 vibrancy 底的對比)與後台各狀態徽章——契約:design Implementation Contract 觀察行為清單全數成立;驗證:逐條核對。 <!-- speclink-task:tsk_01KZ5P0WAA4DDMAQMG8XBH1M56 -->

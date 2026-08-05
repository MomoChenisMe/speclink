## Why

討論 card-drawer-header-colors 的三路全域審計(共用 UI 套件/desktop 殼層/web 後台)證實:三層色彩角色規則——teal 主色=連結/互動/進度、語意色=狀態、中性=靜態 metadata——只被實作了一半。sky(進行中)/emerald(成功)/rose(未通過)在 desktop 殼層與 web 後台幾乎零使用,teal 與琥珀承包過載:「進行中」用 teal 轉圈、「成功」用 teal、「錯誤」大量塗琥珀而失去警示層級;後台的健康/啟用停權/有效撤銷狀態徽章全被壓成兩種灰,同頁出現「上方紅色橫幅、下方灰徽章」講同一件事的自相矛盾。另有 4 處繞過主題 token 直用 Tailwind 原生色階,而 packages/ui 守門測試的白名單明文放行原生色階,是漂移能長期全綠的原因。

目標使用者是透過 AI 代理跑 SDD 的開發者、PO 與 PM;使用情境為看板、變更/討論/已封存抽屜、系統匣面板、設定頁與 Speclink Server Web Console 的狀態辨識——涵蓋全 workflow 階段的檢視面。

## What Changes

- 新增 packages/ui 語意色常數表(單一來源,比照 reviewStyle/stage.ts 的 TS 常數慣例):進行中=sky、成功/新增=emerald、警示=amber(統一 -600 與深色變體階梯)、錯誤與危險=destructive token;表內註記三紅分工(錯誤訊息與危險按鈕=destructive、品質站未通過章=rose、delta 刪除=red)。
- packages/ui 違規修正:AnalyzePanel 的 Warning 徽章由 primary 改 amber、「驗證通過/零問題」改中性;tooltip 氣泡由實心主色改反色中性(深底白字),解除與「已就緒」徽章撞色;搜尋高亮(HighlightText)由 primary 改琥珀 mark;討論抽屜輪次籤轉中性;DiscussionColumn 欄頭圖示/頂部色條/計數徽章收斂至 stage.ts 單一來源或轉中性(消除硬編平行 teal 階);ReviewArchiveDialog「照樣帶走」按鈕由借用狀態色改 destructive;建立者頭像全域中性化(看板卡片、變更詳情抽屜、討論卡、封存卡、討論抽屜五處 bg-primary 改 bg-muted);看板卡片 restale 指示補深色變體並統一琥珀階;看板卡片 worktree 標示由主色 60%(text-primary/60)改語意色 sky——worktree 掛著=工作正於副本進行中,歸「進行中」狀態,抽屜的分支+路徑維持 meta 列中性(討論 worktree-color-semantics 補位:worktree UI 晚於三路審計落地,未經三層規則檢驗)。
- desktop 殼層修正:系統匣面板「討論/已轉出」分區還原中性(不再借穿生命週期徽章樣式)、recovery 卡依狀態分色(還原中=sky、錯誤=destructive)、作用中分頁的「選取」以主色外框表達(不再與琥珀警示混淆)、根層裝飾漸層轉中性;遷移對話框(成功=emerald、進行中=sky、eyebrow 小標中性);WorkspaceChooser 步驟標籤與 notice 轉中性(步驟進度條維持主色);UpdateBanner 與 InstructionUpdatePrompt 底色轉中性、圖示改對應語意色;ProjectTabs(error=destructive、還原 spinner=sky、遠端 ready 圖示中性);RemoteWorkspaceRecovery(還原中=sky、access-denied/not-found/unknown=destructive、needs-reauth 維持 amber);設定頁(更新錯誤與設定檔解析錯誤=destructive、有新版=sky、停用 locale 選項=中性、政策衝突面板=amber 邊框);stale 橫幅「重新登入」按鈕(主視窗與面板)由琥珀改中性 outline;RemoteConflictDialog 兩選項按鈕語彙統一;ServersPanel 與 connectionLogin 的原生色階改 token(text-teal-700 已登入改中性、三處 text-red-600 改 text-destructive)。
- web 後台修正:儲存健康、成員啟用/停權、PAT 與裝置憑證有效/已撤銷等狀態徽章補語意色(正常/啟用/有效=emerald、異常=destructive、停權=amber、已撤銷=中性且與有效可區辨);PAT 揭示與邀請連結揭示橫幅由 primary 改 emerald(成功語意)。
- 守門收緊:packages/ui 主題守門測試的原生色階白名單改為「僅集中常數檔可使用原生語意色階字面」,掃描範圍涵蓋 packages/ui 與兩個 app 的元件原始碼,阻止新的漂移。
- 死碼清理:ChangeBoard/ChangeList/ChangeListItem/DetailDrawer 四個無消費端元件、index.ts 對應匯出、專屬測試(changeListItem.test.tsx 整檔,components/kanban 測試中的對應區塊)。
- 明文不動:stage.ts 生命週期 teal 深淺階梯(看板欄與系統匣共用,屬「進度」角色合法用法)、複製成功勾號的 primary(互動即時回饋)、看板卡片討論徽章 text-primary/60(連結語意)。

## Non-Goals

- 品質站蓋章換紫由 review-stamp-violet 承載;驗證章 tone 由 verify-station-parity 以 ingest 釘定——本變更不觸碰 reviewStyle.tsx,避免與前者同檔互踩。
- 同源籤改 teal 已由 change-drawer-header-redesign 的共用籤元件達成,不重複處理。
- 不動 theme.css 的 token 值,不新增 CSS 變數(討論裁定落地機制為 TS 常數表)。
- 已封存討論/變更的歷史文案與截圖不回改。
- 不重做任何版面結構——本變更純換色與死碼清理,元件版面零移動。

## Capabilities

### New Capabilities

(無)

### Modified Capabilities

- `desktop-app`: 新增「介面狀態語意色分層」約束——狀態呈現一律語意色(進行中=藍,含看板卡片 worktree 標示、成功=綠、警示=琥珀、錯誤=紅),主色不得表達狀態,錯誤不得以琥珀呈現;生命週期階梯與互動回饋的主色用法明文豁免。
- `server-web-console`: 新增「後台狀態徽章語意色」約束——健康/啟用停權/有效撤銷徽章依語意上色,揭示橫幅為成功語意。

## Impact

- Affected specs: `desktop-app`、`server-web-console`
- Affected code:
  - New: packages/ui/src/tone.ts
  - Modified: packages/ui/src/components/AnalyzePanel.tsx、packages/ui/src/components/ui/tooltip.tsx、packages/ui/src/components/HighlightText.tsx、packages/ui/src/components/DiscussionDrawer.tsx、packages/ui/src/components/DiscussionColumn.tsx、packages/ui/src/components/ReviewArchiveDialog.tsx、packages/ui/src/components/ChangeCard.tsx、packages/ui/src/components/RichDetailDrawer.tsx、packages/ui/src/components/ArchivedList.tsx、packages/ui/src/index.ts、packages/ui/src/__tests__/theme.test.ts、packages/ui/src/__tests__/components.test.tsx、packages/ui/src/__tests__/kanban.test.tsx、apps/desktop/src/panel/TrayPanel.tsx、apps/desktop/src/App.tsx、apps/desktop/src/components/MigrationDialog.tsx、apps/desktop/src/components/WorkspaceChooser.tsx、apps/desktop/src/components/UpdateBanner.tsx、apps/desktop/src/components/InstructionUpdatePrompt.tsx、apps/desktop/src/components/ProjectTabs.tsx、apps/desktop/src/components/RemoteWorkspaceRecovery.tsx、apps/desktop/src/components/RemoteConflictDialog.tsx、apps/desktop/src/components/ServersPanel.tsx、apps/desktop/src/components/connectionLogin.tsx、apps/desktop/src/views/AppSettingsView.tsx、apps/desktop/src/views/ProjectSettingsView.tsx、apps/server-web/src/pages/AccountPage.tsx、apps/server-web/src/pages/admin/UsersPage.tsx、apps/server-web/src/pages/admin/OverviewPage.tsx、apps/server-web/src/pages/admin/SystemPage.tsx、apps/server-web/src/pages/admin/CredentialsPage.tsx(另含各表面對應測試檔的斷言更新)
  - Removed: packages/ui/src/components/ChangeBoard.tsx、packages/ui/src/components/ChangeList.tsx、packages/ui/src/components/ChangeListItem.tsx、packages/ui/src/components/DetailDrawer.tsx、packages/ui/src/__tests__/changeListItem.test.tsx
- 影響的 app/套件:packages/ui、apps/desktop、apps/server-web(皆呈現層);Rust crates、CLI、server API 零改動。
- 相容性影響:純視覺;CLI 人眼輸出與 --json 零變化;@speclink/ui 對外匯出面縮減(移除四個無消費端元件的匯出,兩個 app 皆未引用,無遷移需求)。

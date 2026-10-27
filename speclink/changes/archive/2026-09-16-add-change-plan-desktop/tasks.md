## 1. 桌面 core：清單順序、欄位與守門（D1、D2、D3、D8）

- [x] 1.1 紅燈：在 apps/desktop/core/src/query.rs 測試模組以 testfixture 建三個 change（b 的 depends_on 含 a、c 與 a 重疊）並斷言「變更清單的排程欄位」：`list_changes_at` 的 changes 順序為 plan 配置順序、每項含 wave／blockedBy／dependsOn／overlaps 四鍵（camelCase）、頂層 planError 為 null；成環夾具（a、b 互依）時順序退回 board 排序、四鍵缺席、planError 為成環訊息；壞 meta 項照舊列出且四鍵缺席。驗證：`cargo test -p speclink-desktop-core query::` 先紅。 <!-- speclink-task:tsk_01M2J22N3HZTC5Z5QSS71ST2CM -->
- [x] 1.2 綠燈：`list_changes_at` 於 overlay 後的 store 呼叫 `speclink_core::plan::compute`，依結果排序並疊四欄位與 planError；`manage.rs` 的 `change_stage` 改呼叫 `speclink_core::model::stage`（D8）。驗證：1.1 全綠；`cargo test -p speclink-desktop-core` 既有 reorder 測試仍綠。 <!-- speclink-task:tsk_01M2J22N3HDB3FVYBPHP6B1KXY -->
- [x] 1.3 紅燈→綠燈：「欄內拖排以中點 rank 單檔寫回」的依賴檢查——測試 c depends_on a、同欄、`reorder_card_at` 把 c 拖到 a 前時回 Err 且 c 的 meta `board_rank` 不變（補章寫入允許存在）；重疊夥伴翻轉成功。實作：`reorder_change` 於補章後、寫被拖卡前呼叫 `speclink_core::plan::check_rank_move(store, id, &key)`，被拒以 Display 文字回 Err。驗證：`cargo test -p speclink-desktop-core manage::` 全綠。 <!-- speclink-task:tsk_01M2J22N3HND13JYF04Z5ZE2SR -->
- [x] 1.4 紅燈→綠燈：`manage::set_depends_at(root, change, on, remove) -> Result<(), String>` 單行委派到 `speclink_core::plan::set_depends`；測試寫入後 meta 含 `depends_on: a`、守門失敗回 Err 且 meta 不變。驗證：`cargo test -p speclink-desktop-core manage::` 全綠。 <!-- speclink-task:tsk_01M2J22N3H44SQKE17G616W6DC -->

## 2. Tauri 殼與資料源（D6）

- [x] 2.1 apps/desktop/src-tauri/src/lib.rs 新增 `set_change_depends(root, change, on, remove)` command（spawn_blocking 單行委派）並註冊到 generate_handler；packages/ui/src/adapter.ts 的 `SpeclinkDataSource` 加 `setDepends`，`ChangeItem` 加 wave／blockedBy／dependsOn／overlaps 四個選填欄位；apps/desktop/src/adapter/tauriDataSource.ts 委派 invoke、remoteDataSource.ts 拋「不可用」；apps/desktop/src/session.ts 的 capability 加 `setDepends`（local true、remote false）；store.ts 加 `setDepends` action（成功刷新、失敗單行錯誤）。驗證：新增 apps/desktop/src/__tests__/planDataSource.test.ts 斷言 tauri 資料源以正確參數呼叫 `set_change_depends`、remote 資料源拒絕、capability 旗標兩值；`npm test -w apps/desktop` 全綠。 <!-- speclink-task:tsk_01M2J22N3H4SCXKK2VGFH3P9K7 -->

## 3. 看板卡片與拖排（D3、D4、D5）

- [x] 3.1 紅燈：新增 packages/ui/src/__tests__/planBadge.test.tsx 覆蓋「看板卡片的波次與阻擋標示」：wave=1 無阻擋只有圓圈「1」章、wave=2 且 blockedBy 兩項出現「等 2 項」灰章與 tooltip「等待：add-a、add-b」、缺 wave 時兩章缺席；在 packages/ui/src/__tests__/stage.test.ts 補 `planWave`／`planBlockedBy` 的缺欄位邊界。驗證：`npm test -w packages/ui` 先紅。 <!-- speclink-task:tsk_01M2J22N3HV1ZQCXQGTVCDYVG4 -->
- [x] 3.2 綠燈：packages/ui/src/stage.ts 加 `planWave`／`planBlockedBy` 單一入口；ChangeCard.tsx 於驗證章後渲染波次章與阻擋章；packages/ui/src/i18n.tsx 加 card.wave、card.blocked、card.blockedTitle（tw／en）。驗證：3.1 全綠。 <!-- speclink-task:tsk_01M2J22N3HQ9D7VT72R2Y9ZPN3 -->
- [x] 3.3 紅燈→綠燈：「拖排時不合法落點灰化」——packages/ui/src/boardDnd.ts 新增純函式 `invalidDropTargets(column, deps, activeId)`（只看 dependsOn，不看 overlaps），測試三個規格 scenario（拖依賴者時前置及其上方灰化、拖前置時依賴者及其下方灰化、重疊夥伴不灰化）；KanbanBoard.tsx 於拖動中對不合法目標加 `data-drop-invalid="true"` 與降透明度，dragEnd 命中不合法目標不呼叫 onReorder。驗證：packages/ui/src/__tests__/kanban.test.tsx 新增案例斷言屬性存在與 onReorder 未被呼叫；`npm test -w packages/ui` 全綠。 <!-- speclink-task:tsk_01M2J22N3HW90Y3RMA3EE1T5NA -->
- [x] 3.4 紅燈→綠燈：「依賴成環時看板提示」——KanbanBoard 接受 `planError` prop，非 null 時於 reorderUnavailableReason 同位置渲染 tw「依賴成環：」＋原文；App.tsx 自 payload 傳入；i18n 加 board.planCycle。驗證：kanban.test.tsx 斷言提示列存在／不存在；`npm test -w packages/ui` 全綠。 <!-- speclink-task:tsk_01M2J22N3HTAKA2RKXG4CFXN1A -->
- [x] 3.5 依「看板卡片順序以 board_rank 欄位為真相」的新語意改寫 kanban.test.tsx 中依賴「缺 rank 置頂」或修改時間回退序的既有案例（看板只按階段過濾、順序來自 payload），確認卡片順序測試改為斷言 payload 順序被保留。驗證：`npm test -w packages/ui` 全綠。 <!-- speclink-task:tsk_01M2J22N3HY19JFHAC41K3TPXT -->

- [x] 3.6 紅燈→綠燈：依 D4 改版「看板卡片的波次與阻擋標示」——改寫 packages/ui/src/__tests__/planBadge.test.tsx：波次章落在進度列（`data-progress` 列內）最左、tooltip 無阻擋「第 1 波」／有阻擋「第 2 波 · 等待：add-a、add-b」、被擋卡 `[data-change]` 帶 `data-blocked="true"` 與 `opacity-60`、標題列無「等 N 項」文字、缺 wave 時無章不變淡；ChangeCard.tsx 把波次章自標題列搬到進度列並套整卡變淡；packages/ui/src/i18n.tsx 移除 card.blocked（tw／en）。驗證：`npm test -w packages/ui` 全綠。 <!-- speclink-task:tsk_01M2M5BJTJ9WAPJ4WRNWD9XSJD -->
- [x] 3.7 重建安裝本機 desktop（`scripts/desktop/desktop-install.mjs --install`）供 6.2 核對改版後的卡片。驗證：`~/.local/bin/speclink --version` 為 engine v1.36.0，且 apps/desktop/dist 的 bundle 含「等待：」而不含「等 {n} 項」。 <!-- speclink-task:tsk_01M2M5BJTKHJWB4CZZ69AYW2Z9 -->

## 4. 詳情抽屜的排程分頁（D6）

- [x] 4.1 紅燈：新增 packages/ui/src/__tests__/planTab.test.tsx 覆蓋「詳情抽屜的排程分頁」四個 scenario：四段內容、選擇下拉新增觸發 setDepends(change, [name], false)、移除鈕觸發 setDepends(change, [name], true)、capability false 時無編輯控制項、缺 wave 時只顯示一句說明。驗證：`npm test -w packages/ui` 先紅。 <!-- speclink-task:tsk_01M2J22N3HWCR5QVNY37FKFVSE -->
- [x] 4.2 綠燈：RichDetailDrawer.tsx 於規格分頁後加 `value="plan"` 分頁與四段內容，編輯控制項受 capability 開關；i18n 加 drawer.tab.plan、plan.wave、plan.onlyOne、plan.depends、plan.overlaps、plan.blocked、plan.canStart、plan.unavailable（tw／en）；失敗訊息走既有單行錯誤路徑。驗證：4.1 全綠。 <!-- speclink-task:tsk_01M2J22N3H64Y2KFPYXMDF0Q4V -->

## 5. 系統匣（D7）

- [x] 5.1 紅燈→綠燈：「變更列的波次標示與順序同源」——apps/desktop/src/__tests__/trayPanel.test.tsx 斷言列首數字、被擋列降透明度與 tooltip、缺 wave 時列不變；apps/desktop/src/__tests__/tray.test.ts 斷言 buildTrayModel 標籤前綴「N· 」且無 wave 時不加。實作 TrayPanel.tsx 與 tray.ts。驗證：`npm test -w apps/desktop` 全綠。 <!-- speclink-task:tsk_01M2J22N3HPD3HJKFC4TQZH5K9 -->

## 6. 收尾

- [x] 6.1 全面回歸：`cargo test -p speclink-desktop-core`、`npm test -w packages/ui`、`npm test -w apps/desktop` 三組全綠；`speclink list --json` 於含 depends_on 的 change 下輸出不含 wave 等四欄與 planError；`node --test scripts/*.test.mjs scripts/*/*.test.mjs` 全綠。 <!-- speclink-task:tsk_01M2J22N3HKNY40PPA42W5A5SZ -->
- [x] [M] 6.2 於桌面 app 實際操作：看板三欄依 plan 順序、卡片波次章與阻擋章可見、拖動時前置卡灰化且放下無效、排程分頁可新增與移除前置、系統匣面板列首有數字。驗證：人工核對上述五項。 <!-- speclink-task:tsk_01M2J22N3H1VH5BSR26NECDG8P -->

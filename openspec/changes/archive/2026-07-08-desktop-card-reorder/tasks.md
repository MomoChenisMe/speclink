## 1. speclink-core：rank 讀寫原語歸 speclink-core（design D4）

- [x] 1.1 紅：撰寫「meta 寫入路徑對 board_rank 互不破壞」的失敗測試——ChangeMetadata 讀取含 board_rank 的 .openspec.yaml 不失敗；rank 寫回原語保留既有欄位（created_*、started_*）逐位元組不變；開工標記寫回保留既有 board_rank；討論 frontmatter 的 rank 讀取與寫回同構斷言（crates/speclink-core/src/model.rs、crates/speclink-core/src/inprogress.rs、crates/speclink-core/src/discuss.rs 的 #[cfg(test)]）。驗證：cargo test -p speclink-core 出現預期紅燈
- [x] 1.2 綠：實作 board_rank 選配欄位與變更／討論兩側的 rank 讀寫原語（經 Store trait，沿 conclude／promote 的既有 frontmatter 改寫機制；design「D4: rank 讀寫原語歸 speclink-core，排序與中點演算歸桌面 core」）。驗證：1.1 測試全綠
- [x] 1.3 紅：撰寫「board_rank 不進 CLI 輸出且既有輸出逐位元不變」的測試——對含 board_rank 的 fixture，speclink list --json 與 speclink discuss list --json 的 payload 不含 rank 欄位、輸出與移除全部 board_rank 後逐位元一致（crates/speclink-core/src/listing.rs、crates/speclink-core/src/discuss.rs 測試模組）。驗證：cargo test 紅（或釘住現狀）
- [x] 1.4 綠：以 serde skip 將 rank 欄位排除於 CLI 序列化，1.3 測試轉綠。驗證：cargo test -p speclink-core 與 -p speclink-cli 全綠，parity／color 回歸對照維持通過（CLI 輸出路徑未動）

## 2. 桌面 core：rank 演算與排序（design D1、D2、D3）

- [x] 2.1 紅：撰寫 design「D1: rank 採字串型 fractional key，不用浮點」的性質測試——midpoint(a, b) 對任意 a < b 回傳嚴格介於其間的小寫字母鍵、無縫隙時延長鍵長且不改寫鄰居、批次派發回傳嚴格遞增且兩兩留有可再分縫隙的鍵列（apps/desktop/core/src/rank.rs 新模組的 #[cfg(test)]）。驗證：cargo test -p speclink-desktop-core 紅
- [x] 2.2 綠：實作字串中點與批次派發演算。驗證：2.1 性質測試全綠
- [x] 2.3 紅：撰寫「看板卡片順序以 board_rank 欄位為真相」的排序測試——依 design「D2: 排序語意——rank 升冪、缺值置頂、同值以名稱決斷」：缺 board_rank 的卡置欄頂維持回退序（變更卡＝修改時間、討論卡＝slug）、具值卡依字典序升冪、同值以名稱／slug 決斷（apps/desktop/core/src/query.rs 測試模組）。驗證：cargo test -p speclink-desktop-core 紅
- [x] 2.4 綠：list_changes_at 與 list_discussions 查詢改依（有無 rank, rank, 回退序）複合鍵排序，payload 欄位形狀不變。驗證：2.3 測試全綠且既有 query 測試不破
- [x] 2.5 紅：撰寫「欄內拖排以中點 rank 單檔寫回」與「欄內存在缺 rank 卡時整欄補章」的失敗測試——穩態下只改被拖卡 meta 檔且其餘內容逐位元組不變；欄內有缺 rank 卡時整欄依顯示序補章後套用移動、不波及他欄；鄰居於寫回前被封存／刪除時以現存鄰居重導或落欄頂／欄底不損壞 meta（apps/desktop/core/src/manage.rs 測試模組）。驗證：cargo test -p speclink-desktop-core 紅
- [x] 2.6 綠：實作 reorder 寫回——依 design「D3: 首次拖排時整欄補章，穩態單檔寫入」，經 1.2 的 speclink-core 原語落檔。驗證：2.5 測試全綠、cargo test 全 workspace 綠

## 3. reorder command 以鄰居識別碼表達落點（design D5）

- [x] 3.1 紅：撰寫 design「D5: reorder command 以鄰居識別碼表達落點」的契約測試——reorder_card 參數 kind（change | discussion）、id、prevId／nextId（null＝欄頂／欄底）；tauriDataSource 新方法以正確參數 invoke；store 的 reorder 動作失敗時錯誤浮上 verbResult 且觸發 refresh（apps/desktop/src/__tests__/tauriDataSource.test.ts、apps/desktop/core/src/manage.rs）。驗證：npm test -w apps/desktop 與 cargo test 紅
- [x] 3.2 綠：註冊 reorder_card command（apps/desktop/src-tauri/src/lib.rs）、擴充 SpeclinkDataSource 介面（packages/ui/src/adapter.ts）、實作 tauriDataSource 方法與 store 動作（apps/desktop/src/adapter/tauriDataSource.ts、apps/desktop/src/store.ts）；對新 command 的參數處理套 sharp-edges 審視（speclink instructions --skill audit：id 非法值、prevId／nextId 指涉不存在卡的靜默行為）。驗證：3.1 測試全綠

## 4. 看板 UI 沿任務列既有拖排模式（design D6）

- [x] 4.1 紅：撰寫看板拖排互動測試——三個變更欄與討論欄各掛 SortableContext；dragEnd 於同欄內解析正確 prevId／nextId 呼叫 onReorder；「跨欄拖曳不改變變更階段」：跨欄放開不呼叫 onReorder（彈回、零寫入）、封存落點仍走 onArchive、位移門檻內按放開啟詳情不觸發拖排；PointerSensor activationConstraint distance 8 釘住（packages/ui/src/__tests__/kanban.test.tsx、packages/ui/src/__tests__/discussionColumn.test.tsx）。驗證：npm test -w packages/ui 紅
- [x] 4.2 綠：依 design「D6: 看板 UI 沿任務列既有拖排模式——欄內 SortableContext、跨欄彈回、封存落點保留」實作：KanbanBoard 三欄與 DiscussionColumn 改為欄內 sortable（verticalListSortingStrategy＋DragOverlay＋onDragActiveChange 手勢讓路），拖排卡片補 aria-label 的 i18n 鍵（packages/ui/src/components/KanbanBoard.tsx、packages/ui/src/components/DiscussionColumn.tsx、packages/ui 訊息表）。驗證：npm test -w packages/ui 與 npm test -w apps/desktop 全綠

## 5. 端到端驗證

- [x] 5.1 建置 release（npm run build -w apps/desktop、cargo build --release -p speclink-desktop，建置前關閉執行中 exe）後依 CLAUDE.md 真實視窗流程實測：實拖一張變更卡到同欄新位置（操作前確認使用者未使用螢幕），截圖確認落位；重啟 app 順序不變；git status 只含被拖卡的 .openspec.yaml、diff 僅 board_rank 一行。驗證：截圖與 git diff 人工核對
- [x] 5.2 真實視窗補測三情境：首次拖排整欄補章（該欄各卡 meta 皆獲 board_rank、欄序＝視覺序）；跨欄放開彈回且 git 工作樹零變更；拖到封存落點走既有確認流程。討論欄實拖一張討論卡確認 frontmatter 單檔寫回。驗證：截圖與 git status 人工核對

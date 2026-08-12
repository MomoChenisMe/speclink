## 1. skeleton 基元與佔位組件（packages/ui）

- [x] 1.1 新增 shadcn Skeleton 基元於 packages/ui/src/components/ui/skeleton.tsx（圓角灰塊＋pulse 動畫，prefers-reduced-motion 下停用動畫）。先寫 render 測試（packages/ui/src/__tests__/ 下新檔）斷言基元 class 結構與 reduced-motion 停用，再實作至綠。驗證：npm test -w @speclink/ui 通過。 <!-- speclink-task:tsk_01KZSPGSH37JEW3ET53CM2D73R -->
- [x] 1.2 以 Skeleton 基元組合三種佔位組件並自 packages/ui 匯出：看板佔位卡（名稱條＋摘要條的卡形）、面板佔位列（單行列形）、DocSkeleton（標題條＋數行內文條），皆標記 aria-busy。先寫測試斷言三組件的渲染結構與 aria-busy 屬性，再實作。檔案：packages/ui/src/components/skeletons.tsx（新檔）、packages/ui/src/index.ts（匯出）。驗證：npm test -w @speclink/ui 通過。 <!-- speclink-task:tsk_01KZSPGSH4P8FDT4B7AG3ZBKW5 -->

## 2. A 段：分頁切換中 spinner（apps/desktop）

- [x] 2.1 store 新增 pendingTabKey: string | null（初值 null）：activateTab 本地路徑於 openProject probe 前設為目標分頁 key，enterProject 完成（activeKey 翻轉）或 probe 失敗（寫入 tabErrors）時清為 null；remote 路徑不寫入。先寫單元測試斷言成功與失敗兩條生命週期（含失敗時 tabErrors 行為照舊），再實作。檔案：apps/desktop/src/store.ts 與 apps/desktop/src/__tests__/ 下對應測試。驗證：npm test -w @speclink/desktop 通過。 <!-- speclink-task:tsk_01KZSPGSH49D1T9AQMS38CMB5P -->
- [x] 2.2 主視窗分頁列消費 pendingTabKey，落實 desktop-app spec「分頁切換中即時回饋」：目標分頁渲染 spinner，帶切換中語意的 aria-label（新增 i18n 鍵，中英文案皆補）。先寫測試斷言 pending 時 spinner 出現、翻頁後消失、probe 失敗時消失且錯誤呈現照舊，再實作。檔案：apps/desktop/src/components/ProjectTabs.tsx、apps/desktop/src/i18n/messages.ts。驗證：npm test -w @speclink/desktop 通過。 <!-- speclink-task:tsk_01KZSPGSH42AR3KD28FA44GMT2 -->

## 3. B 段：看板首訪 skeleton

- [x] 3.1 KanbanBoard 新增可選 loading prop，落實 desktop-app spec「看板首訪以 skeleton 佔位」：true 時各欄（含討論欄）欄名照常、卡片區渲染佔位卡且不渲染空態文案；App.tsx 以活躍快照的 loaded 為 false 接線。先寫測試斷言 loading true 渲染佔位卡、false 渲染資料、載入完成的空 workspace 顯示既有空態，再實作。檔案：packages/ui/src/components/KanbanBoard.tsx、packages/ui/src/components/DiscussionColumn.tsx、apps/desktop/src/App.tsx。驗證：npm test -w @speclink/ui 與 npm test -w @speclink/desktop 通過。 <!-- speclink-task:tsk_01KZSPGSH409RGDKGC8KTS3GEK -->

## 4. tray 面板：spinner 與 skeleton 同源呈現

- [x] 4.1 TraySnapshot 新增 pendingTabKey 與 workspaceLoaded 欄位，buildTraySnapshot 自 store state 導出（workspaceLoaded＝活躍快照的 loaded）。先寫單元測試斷言兩欄位於切換中／首訪／已訪三種 state 下的導出值，再實作。檔案：apps/desktop/src/tray.ts。驗證：npm test -w @speclink/desktop 通過。 <!-- speclink-task:tsk_01KZSPGSH4MMNN51ACAH9RE2CD -->
- [x] 4.2 TrayPanel 消費新欄位，落實 tray-status-menu spec「面板分頁切換中回饋」與「面板分區首訪 skeleton」：分頁條於 pendingTabKey 相符的分頁顯示 spinner；workspaceLoaded 為 false 時分區內容渲染佔位列、分區標題照常；既有 hideWorkspaceData（remote 復原遮蔽）優先於 skeleton。先寫測試斷言三個情境（pending spinner、首訪 skeleton、遮蔽時無 skeleton），再實作。檔案：apps/desktop/src/panel/TrayPanel.tsx。驗證：npm test -w @speclink/desktop 通過。 <!-- speclink-task:tsk_01KZSPGSH437ZKT08BHAVHM58D -->

## 5. 抽屜文件三態統一與 skeleton

- [x] 5.1 變更抽屜四個文件分頁（提案、設計、任務、規格差異）統一消費文件三態，落實 desktop-app spec「抽屜文件載入以 skeleton 呈現」：undefined 渲染 DocSkeleton、null 顯示該分頁既有空態文案、字串走既有內容渲染；empty 參數移除「載入中」文字借用。先寫測試逐分頁斷言三態，再實作。檔案：packages/ui/src/components/RichDetailDrawer.tsx。驗證：npm test -w @speclink/ui 通過。 <!-- speclink-task:tsk_01KZSPGSH46VS2H50T0D52CYG0 -->
- [x] 5.2 規格、討論、已封存三個抽屜同款三態處理（含已封存抽屜的討論頁）。先寫測試斷言各抽屜 undefined 渲染 DocSkeleton、null 顯示空態，再實作。檔案：packages/ui/src/components/SpecDrawer.tsx、packages/ui/src/components/DiscussionDrawer.tsx、packages/ui/src/components/ArchivedDrawer.tsx。驗證：npm test -w @speclink/ui 通過。 <!-- speclink-task:tsk_01KZSPGSH4CZ7THF72EMRE9H6G -->
- [x] 5.3 清理因本變更孤兒化的「載入中」i18n 鍵：grep 確認 packages/ui/src/i18n.tsx 與 apps/desktop/src/i18n/messages.ts 中該類鍵零消費者後移除；仍有消費者的鍵保留。驗證：grep 零引用，npm test -w @speclink/ui 與 npm test -w @speclink/desktop 通過。 <!-- speclink-task:tsk_01KZSPGSH4RDCTSWE34V0PPDRV -->

## 6. 收尾驗證

- [x] 6.1 受影響面測試全綠且無既有測試語意變動：npm test -w @speclink/ui 與 npm test -w @speclink/desktop 全數通過，diff 檢視確認未修改任何既有測試的斷言語意（僅允許新增測試與必要的 fixture 擴充）。 <!-- speclink-task:tsk_01KZSPGSH4YECB0M2B2M9YXDVH -->
- [ ] [M] 6.2 手動驗收 design 的 Behavior 契約：以 tray 切換至一個 agent 正在操作的忙碌 repo workspace——目標分頁 spinner 立即出現且期間 UI 可互動；首訪 workspace 看板與面板出現 skeleton、載入完成換真資料；切回已訪 workspace 全程不閃 skeleton；開啟變更抽屜各分頁確認載入中為文件 skeleton、不再出現「沒有文件」假空態；系統設定開啟減少動態效果後 skeleton 為靜態灰塊。 <!-- speclink-task:tsk_01KZSPGSH4BPHVG6THN17V6E5N -->

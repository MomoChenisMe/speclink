## 1. store：在途計數與導出旗標

- [x] 1.1 store 內部新增 inflightRefreshes: Map<string, number> 與導出旗標 loadingActive: boolean（＝activeKey 的在途計數 > 0）：refresh() 於第一個 await 前對 sourceKey 計數 +1，settle（成功、失敗、世代過期）於單一 finally 遞減、歸零刪 key，計數或 activeKey 變動時重算 loadingActive。刪除 refreshing 布林、refresh() 出口的 activeKey 與世代守衛。先改寫既有「整批載入的進行中旗標」「整批載入旗標的記帳邊界」測試群的斷言對象為 loadingActive（五類情境語意不變：關閉在途分頁、跨 workspace 不互清、同 key 重疊不早收、翻頁與標記同批、不接載入的入口不標），跑紅後實作至綠。檔案：apps/desktop/src/store.ts、apps/desktop/src/__tests__/store.test.ts。驗證：npm test -w @speclink/desktop 通過。 <!-- speclink-task:tsk_01KZT5YDDFY5RH5BXY6PRGBQNK -->
- [x] 1.2 workspaceActivationState 恢復單參數：刪除 willRefresh 參數與 refreshing 欄位寫入，六個翻頁呼叫端同步改回單參數；「翻頁與載入中標記同批」測試改斷言 loadingActive（翻頁入口同步接 refresh 的計數 +1 使同批成立）。先跑既有測試確認紅、再實作。檔案：apps/desktop/src/store.ts、apps/desktop/src/__tests__/store.test.ts。驗證：npm test -w @speclink/desktop 通過。 <!-- speclink-task:tsk_01KZT5YDDG879QPTQP9PBGQN25 -->
- [x] 1.3 WorkspaceSnapshot 新增 loadFailed: boolean（初值 false）：整批載入失敗且該發為現任世代時設 true、成功載入設 false，隨快照存續。先寫測試斷言設定／清除／過期世代不寫入／切走切回仍在，再實作。檔案：apps/desktop/src/store.ts、apps/desktop/src/__tests__/store.test.ts。驗證：npm test -w @speclink/desktop 通過。 <!-- speclink-task:tsk_01KZT5YDDGFMADS9ET9JCN49DH -->

## 2. tray：單欄導出與即時推送

- [x] 2.1 TraySnapshot 刪 workspaceLoaded／workspaceRefreshing、增 workspaceLoading: boolean（＝loadingActive && !loaded）與 workspaceLoadFailed: boolean；surfaceKey 改 [pendingTabKey, activeKey, workspaceLoading]。先改寫 tray 既有欄位導出與即推測試、補「失敗當下 workspaceLoading 翻 false 即時推送」案，跑紅後實作。檔案：apps/desktop/src/tray.ts、apps/desktop/src/__tests__/tray.test.ts。驗證：npm test -w @speclink/desktop 通過。 <!-- speclink-task:tsk_01KZT5YDDGM7YEWYS3TXFNM65Q -->
- [x] 2.2 TrayPanel 消費單欄：分區骨架條件改 pendingTabKey !== null || workspaceLoading（面板不再自行組合旗標）；落實 tray-status-menu spec「面板分區載入失敗終態」——workspaceLoadFailed 時分區顯示失敗提示列（非空態、非骨架），activeRecovery 遮蔽維持優先。先改寫 trayPanel 測試 fixtures 並補失敗終態與遮蔽優先兩案，跑紅後實作。檔案：apps/desktop/src/panel/TrayPanel.tsx、apps/desktop/src/__tests__/trayPanel.test.tsx、apps/desktop/src/i18n/messages.ts（失敗提示鍵，中英）。驗證：npm test -w @speclink/desktop 通過。 <!-- speclink-task:tsk_01KZT5YDDGT9GZHTGFAJ88XJYB -->

## 3. 看板：失敗終態呈現

- [x] 3.1 KanbanBoard 與 DiscussionColumn 增可選 loadFailed prop，落實 desktop-app spec「首訪載入失敗終態呈現」：loadFailed 且非 loading 時卡片區顯示載入失敗提示文案（i18n 新鍵）、不顯示空態文案；App.tsx 接線 loadFailed={!s.loaded && 活躍快照.loadFailed}、骨架條件改 (s.pendingTabKey !== null || s.loadingActive) && !s.loaded。先寫 kanban 測試斷言失敗提示／空態互斥與 loading 優先，跑紅後實作。檔案：packages/ui/src/components/KanbanBoard.tsx、packages/ui/src/components/DiscussionColumn.tsx、packages/ui/src/i18n.tsx、apps/desktop/src/App.tsx、packages/ui/src/__tests__/kanban.test.tsx。驗證：npm test -w @speclink/ui 與 npm test -w @speclink/desktop 通過。 <!-- speclink-task:tsk_01KZT5YDDGPP6Z91787QSS9JK3 -->

## 4. 收尾

- [x] 4.1 全 repo grep 確認 refreshing／willRefresh／workspaceRefreshing／workspaceLoaded 識別字零殘留（含註解與 design 引文之外的程式碼），孤兒化的測試 helper 與 i18n 鍵一併清除。驗證：grep 零命中，npm test -w @speclink/ui 與 npm test -w @speclink/desktop 全綠。 <!-- speclink-task:tsk_01KZT5YDDGWG2FAC5ARAJVN8FN -->
- [x] 4.2 同步 desktop-loading-skeleton-ux 既有正典描述：本案 archive 時 tray-status-menu「面板分區首訪 skeleton」的「載入中狀態 SHALL 經 TraySnapshot 導出」語意不變，無需 delta；確認本案 design D3 已載明取代 desktop-loading-skeleton-ux design D5 去抖例外。驗證：speclink validate desktop-refreshing-inflight-set 通過。 <!-- speclink-task:tsk_01KZT5YDDG4TZRPY38ES3S3KJC -->
- [ ] [M] 4.3 手動驗收 design 的 Behavior 契約：斷網（或指向不可達的 remote）後首訪一個 workspace——看板與 tray 面板骨架收掉並顯示載入失敗提示、與空 workspace 呈現可區分；恢復連線後重切分頁或等 watcher 重載，提示消失顯示真資料；已訪 workspace 重載失敗仍靜默沿用舊快照。 <!-- speclink-task:tsk_01KZT5YDDGKYW6N66XK27F62A4 -->

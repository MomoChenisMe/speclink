## Context

現況零件：remote runtime 已有 needs_reauth 狀態（401→refresh→重試→仍失敗即置位，之後全部操作拒絕；install_token 即復原）；event manager 為 per-connection 退避 worker（斷流→ETag 比對→重連續訂），但無任何狀態信號發往前端；store 有 tabErrors 與 failure toast，reload 失敗路徑對列表的處理不一（部分吞錯、部分可能清空）；archive 確認對話（AlertDialog）本地與 remote 已共用同一路徑；capability 停用管線（remote-data-source 刀）已是 UI 停用的既成機制。缺：離線偵測、stale 呈現、寫入 offline 即拒、重新認證入口與原地恢復編排。

## Goals / Non-Goals

**Goals:**

- roadmap gate 落地：stale/offline snapshot 只能讀、無隱性 local write queue。
- 離線與 needs-reauth 皆有明確呈現與出口；恢復全自動、不需使用者手動重整。
- 好天氣路徑零改動；本地 session 零影響。

**Non-Goals:**

- 不做離線寫入佇列或衝突合併（明確反目標——gate 禁止隱性佇列，顯式佇列也不做）；不做跨重啟的 snapshot 持久化（stale 內容僅存記憶體，重啟後回到 handshake 重驗路徑）；不改 event manager 的收斂演算法（重用）；不動 server；不做通知中心／歷史（橫幅即時狀態而已）。

## Decisions

### 決策 1：connection 狀態機在 Rust、單一真相、事件廣播

狀態 online｜offline｜needs-reauth 由 remote runtime 判定：任何請求的傳輸層失敗使連續失敗計數遞增、達閾值（常數，測試可注入）即 offline；SSE worker 每輪退避中 sync-state 亦失敗同計；任一請求成功或 worker 收斂成功即歸零並回 online。needs-reauth 沿既有置位、優先於 offline 呈現。轉換時以 Tauri 事件 remote-connection-state（payload：connectionId、state、message）廣播；TS 不自行推斷狀態——單一真相在 Rust。替代案「TS 端以請求錯誤推斷」被否：多視圖各自推斷必然分歧。

### 決策 2：stale 唯讀＝保留最後內容＋capability 疊加 offline mask

offline／needs-reauth 期間：(a) store 的 reload 失敗 SHALL 保留既有清單與文件內容（顯式保障——修正現行可能清空的路徑），分頁呈現 stale 橫幅與 cloud-off 圖示；(b) UI 寫入 affordance 停用重用 capability 停用管線——session 的有效 capability＝handshake capability 疊加 offline mask（全寫入為假），同一條停用 UI 路徑、零新機制；(c) Rust 端全部寫入命令在 offline 即拒（與 needs-reauth 同語意）、讀取命令放行嘗試（成功即回 online）。寫入被拒即回錯誤——不排隊、不重試、不暫存，恢復後 server 端無離線期間寫入痕跡是可測斷言。

### 決策 3：恢復＝worker 收斂事件驅動，不新造機制

worker 重連成功（含 ETag 比對與 Last-Event-ID／reset 分流——既有決策不動）→ runtime 回 online → 廣播狀態＋發全量失效通知 → store 全量重查、清 stale。使用者不需任何操作。輪詢心跳在 offline 期間由 worker 既有退避迴圈承擔——不加額外計時器。

### 決策 4：重新認證入口與原地恢復編排

needs-reauth 橫幅帶「重新登入」：開應用程式設定頁伺服器簽並聚焦該連線的登入（重用 device login／PAT 全流程）。登入成功（install_token 復原 needs-reauth）後自動：對該 connection 的全部 remote sessions 逐一 re-handshake（失敗者維持錯誤呈現）→ 全量重查 → event worker 重啟。分頁全程存在、內容全程可讀（stale）；SHALL NOT 出現退回 local mode 或分頁消失的任何路徑。撤銷情境（server 端撤 device family）即此流程的實走驗證。

### 決策 5：destructive 一致化＝檢核與措辭，不改機制

archive 確認對話已同路徑：remote 分頁時描述文字補充「將寫入 server 上的 scope（Project/Repo 名）」；deleteChange 於 remote 維持停用（既有 capability 斷言不變）；offline 期間 archive 亦被 mask 停用（決策 2 自然涵蓋）。不新增確認機制。

## Implementation Contract

- Rust 測試（注入失敗序列與閾值）：連續失敗達閾值轉 offline 並廣播、單次成功歸零回 online、needs-reauth 優先呈現、offline 期間寫入命令即拒且讀取放行、worker 收斂後 online＋全量失效通知。整合測試（in-process server 可停起）：殺 server → offline → 重啟 → 自動回 online 且 server 端無離線期間寫入。
- vitest（remoteResilience.test.tsx，假事件源）：offline 事件 → 橫幅與 cloud-off、清單保留不清空、寫入 affordance 全停用（capability mask）；online 事件 → 重查、stale 清除；needs-reauth → 橫幅與重新登入入口、登入成功 → re-handshake 重查 worker 重啟的編排呼叫序；本地分頁全程不受影響。
- GUI 鐵律手動（remote-dev-harness；操作前確認使用者未在使用螢幕）：npm run dev 開 remote 分頁 → 殺 server：橫幅現身、看板仍可讀、勾任務被拒不排隊 → 重啟 server：自動恢復、stale 清除、期間 server 無寫入痕跡 → 於 server /account 撤 device family：needs-reauth 橫幅 → 重新登入 → 分頁原地復活 → 全程分頁不消失、本地分頁如常。
- 回歸：cargo test --workspace、npm test -w apps/desktop、npm test -w packages/ui、cargo build --release -p speclink-desktop 全綠。

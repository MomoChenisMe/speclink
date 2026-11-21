## MODIFIED Requirements

### Requirement: 離線狀態機單一真相且明確呈現
<!-- BEFORE: 只有連不上 Server 計入離線的連續失敗；Server 以 500 以上回應時不會進入 offline，離線訊息固定、不含 Server 的原因；進入 offline 之後只有成功的請求才回 online -->

connection 的 online｜offline｜needs-reauth 狀態 SHALL 由 Rust 端 runtime 單一判定並以事件廣播（connectionId、狀態、訊息）：請求連續失敗達閾值或事件 worker 退避中 sync-state 亦失敗即 offline，任一請求成功或 worker 收斂成功即回 online；Server 以低於 500 的狀態碼回應（例如 403、404、409）同樣表示 Server 已在服務——連續失敗的計數 SHALL 歸零，connection 為 offline 時 SHALL 回 online 並廣播，該回應的錯誤 SHALL 照常回給發出請求的一方；401 走既有的換發與 needs-reauth 流程，不適用這一條。計入失敗的 SHALL 是連不上 Server 與 Server 以 500 以上的狀態碼回應這兩種，其他有狀態碼的回應（401、403、404、409 等）SHALL NOT 計入；進入 offline 時廣播的訊息，在最後一次失敗帶有 Server 回報的原因（錯誤信封的 reason 為 unavailable 且 message 不是空字串）時 SHALL 含該原因的原文，否則 SHALL 為固定的離線說明；needs-reauth SHALL 優先於 offline 呈現。remote 分頁 SHALL 於 offline 與 needs-reauth 各自呈現分頁層級的明確狀態（橫幅與 cloud 狀態圖示）；TS 層 SHALL NOT 自行推斷連線狀態。好天氣路徑（無失敗）SHALL 零改動。

#### Scenario: server 不可達轉為離線呈現

- **WHEN** remote 分頁開啟期間 server 程序被終止，後續請求連續失敗達閾值
- **THEN** 該連線廣播 offline，分頁呈現離線橫幅與 cloud-off 圖示；本地分頁不受影響

#### Scenario: Server 持續回 503 轉為離線並顯示原因

- **WHEN** remote 分頁開啟期間 Server 對後續請求都回應 503，信封的 reason 為 unavailable、message 為「系統維護中」，連續失敗達閾值
- **THEN** 該連線廣播 offline，分頁呈現離線橫幅，橫幅的說明含「系統維護中」；Server 恢復回應 200 之後自動回 online

#### Scenario: 單次 500 以上的回應不轉為離線

- **WHEN** remote 分頁開啟期間 Server 對一個請求回應 503，下一個請求成功
- **THEN** 連線維持 online，分頁沒有呈現離線橫幅

#### Scenario: 離線期間 Server 以 403 回應即回 online

- **WHEN** connection 已是 offline，之後 Server 恢復服務，但對這個使用者的輪詢回應 403（使用者在離線期間被移出專案）
- **THEN** 該連線廣播 online，離線橫幅消失；分頁依「存取被拒的狀態跟著分頁」改為呈現存取被拒，不停在離線

#### Scenario: 低於 500 的回應把連續失敗歸零

- **WHEN** connection 為 online，Server 依序回應 503、503、404、503、503
- **THEN** 連線維持 online：404 之後連續失敗從零重新計算，後面兩次 503 未達閾值

## ADDED Requirements

### Requirement: 存取被拒的狀態跟著分頁

remote 分頁的背景輪詢（對 sync-state 的查詢）收到 403 時，Desktop SHALL 把該分頁標為「存取被拒」。這個狀態 SHALL 以分頁的 locator（connection、project、repo）為單位，由 Rust 端的事件 worker 單一判定並以事件廣播（locator key、是否被拒、Server 回報的原因）；TS 層 SHALL NOT 由查詢失敗自行推斷。同一 connection 上其他 project 或 repo 的分頁 SHALL NOT 受影響。其他請求（讀取文件、寫入）的 403 SHALL NOT 觸發這個狀態。

狀態期間：已載入的看板與文件內容 SHALL 保留可讀，查詢失敗 SHALL NOT 清空既有內容；全部寫入操作 SHALL 於 UI 停用並說明原因，Rust 端的寫入命令 SHALL 立即拒絕，SHALL NOT 排隊、暫存或延後重放；切換專案、開啟詳情頁、搜尋與複製 SHALL 照常可用。

呈現 SHALL 依 desktop-app「介面狀態語意色分層」以錯誤的紅系與警示三角形圖示出現在三處：看板上方的橫幅（與離線橫幅同一個位置；標題為「沒有存取權限」，說明為 Server 回報的原因原文接上「目前顯示最後成功載入的內容」的提示，保留 stale 標示，SHALL NOT 有按鈕）、專案圖示列的狀態點（錯誤色，無障礙名稱含「存取遭拒」）、系統匣（依 tray-status-menu）。標題與提示 SHALL 跟隨介面語言（zh-TW 與 en）；Server 回報的原因 SHALL 原樣顯示，SHALL NOT 翻譯。Server 的回應沒有可讀的原因時，說明 SHALL 改用固定的「目前帳號無法存取這個專案」文案。

下一次輪詢成功時 SHALL 全自動復原：狀態解除、橫幅與狀態點消失、寫入操作恢復、看板重新查詢，SHALL NOT 要求使用者手動重整。connection 同時為 needs-reauth 或 offline 時，SHALL 依 needs-reauth、offline、存取被拒的優先序只呈現一種。沒有 session 時（app 重啟後第一次握手就被拒）SHALL 維持既有的復原頁，本需求不適用。

#### Scenario: 看板開著時帳號被停用

- **WHEN** remote 分頁已載入看板，之後 Server 對該分頁的輪詢回應 403，信封的 message 為「帳號已停用」
- **THEN** 在一個輪詢間隔（最長 30 秒）內，看板上方出現紅色橫幅，標題為「沒有存取權限」、說明含「帳號已停用」與「目前顯示最後成功載入的內容」；專案圖示列該專案的狀態點為錯誤色；看板的卡片仍是最後成功載入的內容

#### Scenario: 存取被拒期間寫入被拒

- **WHEN** 分頁為存取被拒時使用者檢視變更的任務清單並嘗試勾選任務
- **THEN** 勾選操作被停用並顯示「目前沒有存取權限」的原因；即使繞過 UI 呼叫寫入命令亦立即被拒，Server 沒有收到該寫入

#### Scenario: 恢復存取後自動復原

- **WHEN** 分頁為存取被拒，之後 Server 對輪詢回應 200
- **THEN** 橫幅與狀態點消失、寫入操作恢復可用、看板重新查詢並呈現最新內容，全程無使用者操作

#### Scenario: 同一 connection 的其他專案不受影響

- **WHEN** 同一 connection 開著 project alpha 與 project beta 兩個分頁，Server 只對 alpha 的輪詢回應 403
- **THEN** alpha 的分頁呈現存取被拒；beta 的分頁維持正常、寫入操作可用

#### Scenario: 推送串流被 Server 關閉後偵測到被拒

- **WHEN** 分頁以推送串流接收更新，Server 因使用者失去權限而關閉串流，重新訂閱與接著的輪詢都回應 403
- **THEN** 分頁呈現存取被拒，與只用輪詢時相同

#### Scenario: Server 沒有說明原因

- **WHEN** Server 對輪詢回應 403，本文不是可解析的錯誤信封
- **THEN** 分頁呈現存取被拒，橫幅的說明為固定的「目前帳號無法存取這個專案」文案

#### Scenario: 多種狀態並存時只呈現一種

- **WHEN** 分頁為存取被拒，之後同一 connection 因連續失敗進入 offline
- **THEN** 分頁改為呈現離線橫幅，不同時呈現兩條橫幅；connection 回 online 而分頁仍被拒時，改回呈現存取被拒

##### Example: 多種狀態並存時呈現哪一種

| connection 狀態 | 分頁是否存取被拒 | 呈現 |
| --------------- | ---------------- | ---- |
| online | 否 | 正常，沒有橫幅 |
| online | 是 | 存取被拒（紅色橫幅） |
| offline | 是 | 離線（琥珀色橫幅） |
| needs-reauth | 是 | 需要重新登入（琥珀色橫幅與重新登入鈕） |
| offline | 否 | 離線（琥珀色橫幅） |

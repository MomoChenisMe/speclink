## ADDED Requirements

### Requirement: 離線狀態機單一真相且明確呈現

connection 的 online｜offline｜needs-reauth 狀態 SHALL 由 Rust 端 runtime 單一判定並以事件廣播（connectionId、狀態、訊息）：請求連續失敗達閾值或事件 worker 退避中 sync-state 亦失敗即 offline，任一請求成功或 worker 收斂成功即回 online；needs-reauth SHALL 優先於 offline 呈現。remote 分頁 SHALL 於 offline 與 needs-reauth 各自呈現分頁層級的明確狀態（橫幅與 cloud 狀態圖示）；TS 層 SHALL NOT 自行推斷連線狀態。好天氣路徑（無失敗）SHALL 零改動。

#### Scenario: server 不可達轉為離線呈現

- **WHEN** remote 分頁開啟期間 server 程序被終止，後續請求連續失敗達閾值
- **THEN** 該連線廣播 offline，分頁呈現離線橫幅與 cloud-off 圖示；本地分頁不受影響

### Requirement: 最後 snapshot 唯讀且寫入即拒無佇列

offline 或 needs-reauth 期間：已載入的看板與文件內容 SHALL 保留可讀並標示 stale——查詢失敗 SHALL NOT 清空既有內容；全部寫入操作（任務勾選、動詞、artifact 寫回、policy 儲存）SHALL 於 UI 停用（capability 疊加離線遮罩）且 Rust 端命令 SHALL 立即拒絕——SHALL NOT 排隊、暫存或延後重放，恢復後 server 端 SHALL 不存在離線期間的任何寫入。讀取命令 SHALL 放行嘗試（成功即促成回 online）。

#### Scenario: 離線期間看板可讀寫入被拒

- **WHEN** 連線 offline 時使用者檢視看板並嘗試勾選任務
- **THEN** 看板呈現最後成功載入的內容附 stale 標示，勾選操作被停用；即使繞過 UI 呼叫寫入命令亦立即被拒，server 恢復後查無該寫入

### Requirement: 恢復自動收斂並清除 stale

server 恢復可達後 SHALL 全自動：事件 worker 以既有 Polling 加 ETag 收斂機制重連，runtime 回 online 並發全量失效通知，store 全量重查後清除 stale 標示——SHALL NOT 要求使用者手動重整或任何操作。

#### Scenario: server 重啟後自動復原

- **WHEN** offline 期間另一 client 於同 scope 建立新 change，隨後 server 恢復
- **THEN** 分頁自動回 online、stale 標示消失，看板含恢復期間的新 change，全程無使用者操作

### Requirement: 重新認證原地復活不退 local

needs-reauth 時分頁橫幅 SHALL 提供重新登入入口（重用既有 device login／PAT 流程）；登入成功後 SHALL 自動對該 connection 的全部 remote sessions 重走 handshake、全量重查並重啟事件 worker——session 與分頁 SHALL 原地恢復。全程分頁 SHALL NOT 消失、SHALL NOT 退回 local mode；期間內容維持 stale 唯讀。

#### Scenario: 撤銷 device family 後原地復活

- **WHEN** server 端撤銷該裝置的 device family，使用者於 needs-reauth 橫幅重新登入並完成授權
- **THEN** 分頁未曾消失，登入後自動 re-handshake 與重查，看板回到可讀寫狀態

### Requirement: remote 破壞性操作確認一致

remote 分頁的 archive 確認對話 SHALL 沿用與本地相同的確認路徑，描述 SHALL 指出將寫入 server 上的 scope（Project/Repo 名）；deleteChange 於 remote SHALL 維持停用；offline 期間 archive SHALL 隨寫入遮罩停用。

#### Scenario: remote archive 確認指出 scope

- **WHEN** 於 remote 分頁對就緒的 change 觸發 archive
- **THEN** 確認對話呈現且描述含該 Project/Repo 名；確認後寫入 server，取消則無任何變更

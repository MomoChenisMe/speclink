## MODIFIED Requirements

### Requirement: 重新認證原地復活不退 local
<!-- BEFORE: needs-reauth 只規範主視窗橫幅入口與既有 session 的原地復活，未涵蓋無 session 復原頁及 Tray。 -->

needs-reauth 時，已建立 session 的 remote 分頁橫幅與尚未建立 session 的 remote 復原頁 SHALL 提供重新登入入口（重用既有 device login／PAT 流程）；macOS Tray Panel 與原生 Tray 選單 SHALL 由同一狀態提供對應重新登入動作。使用者明確選取重新登入 SHALL 顯示主視窗、進入應用程式設定的伺服器頁並聚焦該 connection；僅顯示 needs-reauth 狀態或執行不需登入 UI 的 retry SHALL NOT 自動喚起主視窗。

登入成功後 SHALL 自動對該 connection 的全部 remote sessions 與無 session 復原分頁重走 handshake、全量重查並重啟事件 worker：既有 session 與分頁 SHALL 原地恢復，無 session 分頁 SHALL 於同一 locator key 建立 session。全程分頁 SHALL NOT 消失、SHALL NOT 退回 local mode；既有 session 期間內容維持 stale 唯讀，無 session 分頁 SHALL 維持復原頁且 SHALL NOT 顯示偽造 stale 內容。

#### Scenario: 撤銷 device family 後原地復活

- **WHEN** server 端撤銷該裝置的 device family，使用者於 needs-reauth 橫幅或復原頁選擇重新登入並完成授權
- **THEN** 分頁未曾消失，登入後自動 re-handshake 與重查，看板回到可讀寫狀態，Tray 同步回到 ready

#### Scenario: Tray 顯示 needs-reauth 但不自動奪焦

- **WHEN** background remote workspace 進入 needs-reauth，使用者開啟 macOS Panel 或原生 Tray 選單但尚未選擇重新登入
- **THEN** Tray 顯示需要登入的狀態與動作，主視窗的顯示、Space 與焦點維持不變

#### Scenario: 從 Tray 明確選擇重新登入

- **WHEN** 使用者於 macOS Panel 或原生 Tray 選單的 needs-reauth workspace 選擇重新登入
- **THEN** 主視窗顯示並取得焦點，切至伺服器設定且聚焦對應 connection，登入成功後該 workspace 原地恢復

## ADDED Requirements

### Requirement: stale snapshot 與無 session 復原頁依 session 存在性分流

Desktop SHALL 以可用 WorkspaceSession 是否存在裁決 remote 壞天氣呈現：session 已存在而 connection 為 offline 或 needs-reauth 時 SHALL 保留最後成功資料並標示 stale；app 重啟或恢復 handshake 尚未成功、因此無 session 時 SHALL 呈 restoring／error 復原頁，SHALL NOT 讀取上一個 workspace 資料、將空集合標為 stale 或建立離線資料副本。兩條路徑 SHALL 共用 Rust 提供的連線／錯誤真相，TS SHALL NOT 由查詢失敗次數自行推斷 offline。

#### Scenario: 已建立 session 離線保留最後內容

- **WHEN** remote workspace 已成功載入看板後 server 中斷並由 runtime 判定 offline
- **THEN** 主視窗與 macOS Panel 標示 offline／stale 並保留最後成功內容，寫入維持停用，既有 worker 繼續自動收斂

#### Scenario: 重啟後 handshake 失敗不偽造 stale

- **WHEN** app 重啟還原 remote locator，但第一次 handshake 因 server 不可達而失敗且尚無 session
- **THEN** 主視窗與 Tray 呈 error 復原 UI，不呈現上一個 workspace 的變更／討論，也不標示任何空集合為 stale

#### Scenario: server 恢復後兩條路徑各自收斂

- **WHEN** server 恢復可達
- **THEN** 已建立 session 的 offline 路徑由既有 worker 自動回 online；無 session error 路徑於 retry 或登入恢復編排成功後建立 session，兩者均清除對應壞天氣呈現

## MODIFIED Requirements

### Requirement: 每個 session 自帶 dataSource 且 Rust 側無 current-root 全域

<!-- BEFORE: 以 openspec/ 與 .openspec.yaml 描述規格資料夾與變更資料檔 -->

每個 WorkspaceSession SHALL 攜帶自己的 dataSource／settings／events；App SHALL NOT 注入單一全域 DataSource。每個 session 的最後成功讀取內容 SHALL 依 locator key 獨立歸屬，主內容在 activeKey 改變後 SHALL 只呈現新 active session 自己的最後成功內容；若該 session 尚無成功內容，SHALL 呈現屬於該 session 的載入、恢復或空白狀態，SHALL NOT 保留上一個 active session 的清單、搜尋結果或詳情。非同步讀取完成時 SHALL 以發出請求時的 session 身分結算；結果可更新來源 session 的最後成功內容，但 SHALL NOT 覆寫另一個目前 active session 的主內容或詳情。

local session 的 dataSource 與 settings SHALL 將 root 綁入閉包，使每一支 Tauri command 呼叫皆顯式攜帶 root 參數並直通 desktop-core 的帶路徑函式；Rust 側 SHALL NOT 保有 current-root 可變全域，專案探測命令 SHALL 為純探測、對同一路徑重複呼叫冪等且無任何全域副作用。分頁切換後，前一分頁尚未完成的呼叫 SHALL 仍以其原 root 結算，SHALL NOT 落在新分頁的 root 上。

#### Scenario: in-flight 呼叫不受切換影響

- **WHEN** 分頁 A 的清單查詢尚未回應時使用者切到分頁 B，B 的內容先載入完成，之後 A 的查詢才回應
- **THEN** A 的查詢仍以 A 的 root 結算並僅更新 A 的最後成功內容，主內容持續顯示 B 的資料，A 的回應 SHALL NOT 覆寫 B 的清單、搜尋結果或詳情

#### Scenario: 切到 needs-reauth 分頁不殘留前一工作區

- **WHEN** 分頁 A 已顯示看板，使用者切到已有 session 但載入失敗且狀態為 `needs-reauth` 的分頁 B
- **THEN** 主內容只顯示 B 自己最後成功的 stale snapshot；若 B 從未成功載入則顯示 B 的安全恢復狀態，兩種情況均不顯示 A 的變更、規格、討論、搜尋結果或詳情

#### Scenario: 返回分頁顯示各自最後成功內容

- **WHEN** 分頁 A 與 B 都曾成功載入不同內容，使用者在兩者之間切換且其中一次背景重查失敗
- **THEN** 每次切換均立即顯示目標分頁自己的最後成功內容，失敗不以另一分頁的內容取代目標分頁

#### Scenario: 設定讀寫落在正確專案

- **WHEN** 分頁列有 A、B 兩專案且活躍為 B，使用者於設定頁修改 Workflow 欄位
- **THEN** 寫入落在 B 的 speclink/config.yaml，A 的設定檔內容不變

### Requirement: watcher 顯式跟隨活躍 session 且事件攜帶 root

<!-- BEFORE: 以 openspec/ 與 .openspec.yaml 描述規格資料夾與變更資料檔 -->

檔案監看 SHALL 維持單一實例並跟隨活躍 session：重掛 SHALL 由顯式的監看命令觸發（探測命令 SHALL NOT 附帶重掛副作用）；workspace-changed 事件 SHALL 攜帶被監看的 root，session 的事件來源 SHALL 以自身 locator 過濾後才觸發重載。監看不可用時僅失去自動刷新、app 照常運作的既有語意 SHALL 保留。外部寫者修改活躍專案 speclink/ 後看板秒級自動更新的既有行為 SHALL 不變。

#### Scenario: 外部變更僅觸發活躍 session 重載

- **WHEN** 活躍分頁為 A 時，外部寫者修改 A 的 speclink/ 下文件
- **THEN** A 的看板數秒內自動更新；事件 payload 為 A 的 root，非活躍分頁不因此發出任何查詢

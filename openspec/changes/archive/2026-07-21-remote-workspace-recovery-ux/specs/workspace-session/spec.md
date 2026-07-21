## ADDED Requirements

### Requirement: 可選取的 remote 復原分頁與 session 邊界

已存在於分頁列的 remote workspace 在尚無可用 WorkspaceSession 時，Desktop SHALL 允許該分頁成為作用中 navigation destination，並以 locator key 記錄 activeKey；handshake 進行中 SHALL 呈 restoring，失敗 SHALL 呈 error 復原頁。此狀態下 workspace 資料操作 SHALL 視為無 active session 而不執行，主內容 SHALL NOT 顯示上一個分頁的資料或偽造 stale snapshot。restoring／error 為不持久化的執行期狀態；關閉分頁 SHALL 同時清除，retry 成功 SHALL 於同一 locator key 建立 session 並清除而不新增分頁。

#### Scenario: handshake 失敗仍選取該分頁

- **WHEN** local 分頁作用中，使用者點擊一個持久化但尚無 session 的 remote 分頁，而 handshake 失敗
- **THEN** remote 分頁成為作用中且顯示 error 復原頁，local 分頁資料不再出現在主內容，remote 分頁未消失

#### Scenario: retry 成功原地建立 session

- **WHEN** 作用中的 remote error 分頁再次執行 retry 且 handshake 成功
- **THEN** 同一分頁原地取得 session 並顯示 server 資料，restoring／error 清除，分頁列不新增重複項目

#### Scenario: 較舊 handshake 不搶回作用中分頁

- **WHEN** remote 分頁 A 的 handshake 尚未完成時使用者切至分頁 B，之後 A 的 handshake 才成功或失敗
- **THEN** A 的結果只更新 A 的 session 或 recovery 狀態，activeKey 維持 B

#### Scenario: 同分頁只接受最新 retry 結果

- **WHEN** 同一 remote error 分頁先後觸發兩次 retry，第二次成功後第一個較舊請求才失敗
- **THEN** 該分頁維持第二次成功建立的 session，較舊失敗 SHALL NOT 覆蓋成 error

#### Scenario: local 分頁切換維持既有行為

- **WHEN** 使用者在兩個有效 local 分頁之間切換
- **THEN** active session、watcher、看板資料與持久化 activeKey 依既有流程切換，不建立 remote recovery 狀態

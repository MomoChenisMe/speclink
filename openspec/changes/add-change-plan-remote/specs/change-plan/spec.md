## Purpose

change 層的執行順序：以看板排序鍵與建立日期為基底、以宣告依賴與 delta capability 重疊做拓樸修正，算出波次（同波可並行）與每個 change 的阻擋清單；涵蓋 `plan` 查詢動詞、`change depends` 寫入動詞、rank 移動的依賴檢查，以及 apply 與 archive 技能消費 plan 的規定。看板卡片 rank 的持久化屬 board-card-order，任務層並行不在此範圍。

## ADDED Requirements

### Requirement: plan 與 change depends 的 remote 臂

remote 模式下 `speclink plan` SHALL 呼叫 server 的 GET /plan 並把回應轉回引擎的 plan 報告後以 fs 模式同一渲染輸出，人眼與 --json 的 stdout 形狀 SHALL 與 fs 模式對同一內容一致；server 回 409（成環，reason refused）時 SHALL 以非零 exit code 結束、stderr 為 server 轉發的引擎訊息 `dependency cycle: …`、stdout 為空。remote 模式下 `speclink change depends` SHALL 呼叫 POST /changes/{name}/depends，成功時 stdout 與 fs 模式同一行 `✓ …`（--json 同形），server 的 404 與守門 409 SHALL 以 server 轉發的引擎單行訊息印於 stderr、非零 exit code。兩動詞在 remote 模式 SHALL NOT 讀寫本機 store。remote 拖排的宣告依賴檢查 SHALL 以引擎的純序列檢查函式對拖放後同階段的名稱序列與 plan 回應的 dependsOn 圖執行，只計涉及被拖卡的配對，違反時 SHALL NOT 發出 PUT /board-order、SHALL 以與 local 相同的單行訊息呈現；plan 不可得（舊 server、成環、請求失敗）時 SHALL 略過此檢查。

#### Scenario: remote plan 同形

- **WHEN** 於 remote 模式執行 speclink plan --json，server 回兩個 change 的 plan
- **THEN** stdout 的 JSON 鍵與 fs 模式一致（waves、changes、next、skipped），exit code 為 0，未讀取本機 openspec/

#### Scenario: remote 成環

- **WHEN** 於 remote 模式執行 speclink plan，server 回 HTTP 409、reason 為 refused、message 為 `dependency cycle: add-a -> add-b -> add-a`
- **THEN** exit code 非零，stderr 含 `dependency cycle: add-a -> add-b -> add-a`，stdout 為空

#### Scenario: remote 寫入前置

- **WHEN** 於 remote 模式執行 speclink change depends add-b --on add-a
- **THEN** 對 server 發出 POST /changes/add-b/depends、body `{"on":["add-a"],"remove":false}`，stdout 為 `✓ add-b depends on: add-a`

#### Scenario: remote 拖排違反依賴不發 PUT

- **WHEN** remote 分頁中 add-b 的 dependsOn 含 add-a，使用者把 add-b 拖到 add-a 之前放開
- **THEN** 桌面顯示單行錯誤 `cannot move 'add-b' there: it depends on add-a`、未發出 PUT /board-order、看板刷新回 add-a 在前

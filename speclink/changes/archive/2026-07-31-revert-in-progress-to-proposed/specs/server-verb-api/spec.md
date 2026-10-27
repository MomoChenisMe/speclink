## ADDED Requirements

### Requirement: in-progress 標記移除端點與加入端點成鏡像

server SHALL 提供 DELETE /changes/{name}/in-progress 端點,與既有 POST /changes/{name}/in-progress 同資源、反向語意,並循既有動詞端點的認證與 binding 規則。守門 SHALL 由引擎同一裁決點執行,與本地 CLI 行為一致:零工作痕跡(已勾任務數為 0 且 touched 記錄兩清單皆空)時移除 started_* 三欄位,HTTP 200 回 Ack 並發佈變更退回事件(事件經 SSE 以 invalidation hint 流動);對未開工的 change SHALL 冪等成功——HTTP 200、零寫入、不 commit、不發事件。有工作痕跡時 SHALL 回 HTTP 409,error payload SHALL 含 camelCase 證據欄位:checkedTasks(數字,已勾任務數)與 touchedFiles(字串陣列,touched 記錄檔案清單聯集去重);不存在的 change SHALL 回 HTTP 404。既有 POST /changes/{name}/in-progress 端點的行為與回應 SHALL 不變。

#### Scenario: 零痕跡變更移除成功並發事件

- **WHEN** 認證通過的呼叫者對一個零工作痕跡的進行中 change 發 DELETE /changes/{name}/in-progress
- **THEN** HTTP 200 回 Ack,該 change 的 started_* 欄位消失且其餘 meta 內容不變,SSE 串流出現對應的 invalidation hint 事件

#### Scenario: 未開工變更冪等成功且不發事件

- **WHEN** 對一個從未開工的 change 發 DELETE /changes/{name}/in-progress
- **THEN** HTTP 200 回 Ack,無任何寫入與 commit,SSE 串流不出現新事件

#### Scenario: 有工作痕跡時回 409 與結構化證據

- **WHEN** 對一個已勾 2 個任務且 touched 記錄含 src/a.rs 的 change 發 DELETE /changes/{name}/in-progress
- **THEN** HTTP 409,error payload 的 checkedTasks 為 2、touchedFiles 為 ["src/a.rs"],該 change 的 meta 與 touched 記錄皆不變

##### Example: 證據欄位形狀

- **GIVEN** 一個 change 已勾任務數 3,touched 記錄檔案清單為 src/x.rs 與 docs/y.md
- **WHEN** 對其發 DELETE /changes/{name}/in-progress
- **THEN** 409 的 error payload 含 "checkedTasks": 3 與 "touchedFiles": ["src/x.rs", "docs/y.md"]

#### Scenario: 不存在的 change 回 404

- **WHEN** 對不存在的 change 名稱發 DELETE /changes/{name}/in-progress
- **THEN** HTTP 404,無任何寫入

## MODIFIED Requirements

### Requirement: plan 唯讀衍生查詢端點

<!-- BEFORE: DTO 七欄；壞 board 時順序寫「依 created 升冪」 -->

server SHALL 提供 GET /plan（scope 層級）：經 Command gateway 執行與本地相同的引擎 plan 計算，rank 來源 SHALL 為該 scope 的 board resource 文件（缺席、無法解析或非物件時視為全員缺 rank），回應為 typed DTO（waves、changes、next、skipped，欄位與 CLI plan --json 同形，changes 每項九鍵含 `requirementOverlap` 與 `archiveAfter`，自引擎報告逐欄搬運）附 scope ETag。端點 SHALL 對 reader 與 editor 皆可用，SHALL NOT 產生任何寫入或事件發布。宣告依賴成環時 SHALL 回 HTTP 409、reason 為 refused（沿用封閉 error reason registry，SHALL NOT 新增 reason 值）、message 為引擎的成環訊息（含成環路徑）；成環以外的引擎一般錯誤 SHALL 維持既有映射。同一 scope 內容與同一 rank 圖下，回傳的配置順序、波次、requirementOverlap 與 archiveAfter SHALL 與本地 fs 模式對同一內容的結果一致。

#### Scenario: reader 可讀 plan

- **WHEN** 以 reader role 憑證呼叫 GET /plan，scope 內 add-b 的 depends_on 含 add-a
- **THEN** HTTP 200，changes 內 add-a 在 add-b 之前、add-b 的 blockedBy 為 ["add-a"]，每項含 requirementOverlap 與 archiveAfter 鍵，附 scope ETag，revision 不前進

#### Scenario: rank 來自 board resource

- **WHEN** board resource 的 changes 段為 {"add-b":"b","add-a":"f"}，兩者無依賴且不重疊
- **THEN** GET /plan 的 changes 順序為 add-b、add-a（rank 字典序），波次皆為 1

#### Scenario: 壞 board 內容視為空圖

- **WHEN** board resource 內容為非 JSON 文字
- **THEN** GET /plan 仍 HTTP 200，順序依 change-plan 規定的缺 rank 順序（全員缺 rank；無人依賴且 task 數相同時即 created 升冪），不回錯誤

#### Scenario: 成環回 409

- **WHEN** add-a 與 add-b 互相 depends_on，呼叫 GET /plan
- **THEN** HTTP 409，reason 為 refused，message 為 `dependency cycle: add-a -> add-b -> add-a`，revision 不前進

#### Scenario: 同 requirement 重疊帶 archiveAfter

- **WHEN** scope 內 add-a 與 add-b 皆 MODIFIED 同一 capability 的同一 requirement，無依賴、無 rank、add-a 配置在前
- **THEN** GET /plan 的 add-b 項 archiveAfter 為 ["add-a"]、requirementOverlap 含 change 為 add-a 且 conflict 為 false 的一項，兩者 wave 皆為 1

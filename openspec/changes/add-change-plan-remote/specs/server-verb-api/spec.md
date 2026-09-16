## ADDED Requirements

### Requirement: plan 唯讀衍生查詢端點

server SHALL 提供 GET /plan（scope 層級）：經 Command gateway 執行與本地相同的引擎 plan 計算，rank 來源 SHALL 為該 scope 的 board resource 文件（缺席、無法解析或非物件時視為全員缺 rank），回應為 typed DTO（waves、changes、next、skipped，欄位與 CLI plan --json 同形）附 scope ETag。端點 SHALL 對 reader 與 editor 皆可用，SHALL NOT 產生任何寫入或事件發布。宣告依賴成環時 SHALL 回 HTTP 409、reason 為 dependency-cycle、訊息含成環路徑。同一 scope 內容與同一 rank 圖下，回傳的配置順序與波次 SHALL 與本地 fs 模式對同一內容的結果一致。

#### Scenario: reader 可讀 plan

- **WHEN** 以 reader role 憑證呼叫 GET /plan，scope 內 add-b 的 depends_on 含 add-a
- **THEN** HTTP 200，changes 內 add-a 在 add-b 之前、add-b 的 blockedBy 為 ["add-a"]，附 scope ETag，revision 不前進

#### Scenario: rank 來自 board resource

- **WHEN** board resource 的 changes 段為 {"add-b":"b","add-a":"f"}，兩者無依賴且不重疊
- **THEN** GET /plan 的 changes 順序為 add-b、add-a（rank 字典序），波次皆為 1

#### Scenario: 壞 board 內容視為空圖

- **WHEN** board resource 內容為非 JSON 文字
- **THEN** GET /plan 仍 HTTP 200，順序依 created 升冪（全員缺 rank），不回錯誤

#### Scenario: 成環回 409

- **WHEN** add-a 與 add-b 互相 depends_on
- **THEN** HTTP 409，reason 為 dependency-cycle，訊息含 `add-a -> add-b -> add-a`

### Requirement: 變更依賴寫入端點

server SHALL 提供 POST /changes/{name}/depends，body 為 { on: string[], remove: boolean }，經 Command gateway 直通引擎的依賴寫入命令，editor role 限定（reader 回 403）。change 不存在 SHALL 回 404；自依賴 SHALL 回 409 reason depends-self；任一 on 名稱不存在或已封存 SHALL 回 409 reason depends-target-invalid；非 remove 且加邊後成環 SHALL 回 409 reason dependency-cycle；守門失敗 SHALL 零寫入。實際改動 meta 時 SHALL 發布領域事件、scope revision 前進並發布 invalidate；冪等無改動時 SHALL HTTP 200、零寫入、零事件、revision 不前進。回應 SHALL 為 { change: string, dependsOn: string[] }（寫入後完整清單）附 scope ETag。

#### Scenario: editor 寫入前置

- **WHEN** editor 對 add-b 呼叫 POST /changes/add-b/depends，body { on: ["add-a"], remove: false }
- **THEN** HTTP 200、body 為 { change: "add-b", dependsOn: ["add-a"] }，add-b 的 meta 多一行 depends_on: add-a、其餘逐字元保留，事件發布且 revision 前進

#### Scenario: reader 被拒

- **WHEN** reader 呼叫同一端點
- **THEN** HTTP 403，meta 不變

#### Scenario: 守門失敗 409

- **WHEN** 分別以 on 指向自己、指向已封存的名稱、會成環的邊呼叫端點
- **THEN** 三次皆 HTTP 409，reason 分別為 depends-self、depends-target-invalid、dependency-cycle，meta 皆不變、revision 不前進

#### Scenario: 冪等重加

- **WHEN** 對已含 add-a 的 add-b 再次以 on ["add-a"] 呼叫
- **THEN** HTTP 200、dependsOn 為 ["add-a"]，零寫入、零事件、revision 不前進

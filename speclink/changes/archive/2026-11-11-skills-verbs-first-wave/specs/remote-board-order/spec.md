## MODIFIED Requirements

### Requirement: board resource 為 scope 單文件且 server 不解析
<!-- BEFORE: 唯一的例外是 GET /plan；寬鬆讀取時任一段型別錯整份視為全員缺 rank；排序路由不存在 -->

remote 看板順序 SHALL 儲存於 scope 層級的獨立 board resource 文件（DocumentId 的 board order 種類），內容為 JSON 兩段圖（changes：變更名→rank、discussions：slug→rank）。server SHALL 提供 GET /board-order（回內容與 scope ETag；文件缺席為正常態、內容回 null）與 PUT /board-order（If-Match CAS、全文覆寫、editor role 限定、超過大小上限拒絕）；這兩個端點 SHALL 視內容為不透明文本、SHALL NOT 解析或校驗其語意。例外有兩個：GET /plan 端點（見 server-verb-api）與排序請求 POST /changes/{name}/rank（見 client-protocol「排序請求與回應」）。兩者 SHALL 以同一個寬鬆規則逐段讀取該文件：整份缺席、無法解析或不是物件時兩段皆視為空；某一段缺席或型別錯時只有那一段視為空，另一段照常讀取；SHALL NOT 因內容壞掉而失敗。GET /plan SHALL 以讀到的 changes 段作為 rank 來源，SHALL NOT 回寫或修剪文件。排序請求經引擎命令整份寫回時，changes 段 SHALL 只保留作用中變更的條目再併入這次的順序鍵，discussions 段 SHALL 照讀到的內容帶回，並以這次讀到的版本做 CAS。PUT 成功 SHALL 於 commit 後發布 invalidate 使訂閱端重讀。拖排 SHALL NOT 觸碰任何變更卡 meta 或討論 frontmatter。

#### Scenario: 缺席為正常態

- **WHEN** 對從未拖排過的 scope 呼叫 GET /board-order
- **THEN** HTTP 200、內容為 null 並附 scope ETag，看板照回退序渲染

#### Scenario: CAS 過期拒絕

- **WHEN** 以過期 If-Match 呼叫 PUT /board-order
- **THEN** HTTP 409 且 reason 機器可判為 revision 衝突，文件內容不變

#### Scenario: reader 不可寫

- **WHEN** 以 reader role 憑證呼叫 PUT /board-order
- **THEN** HTTP 403 且 reason 機器可判為權限不足

#### Scenario: 拖排不動卡片文件

- **WHEN** editor 於 remote 分頁完成一次卡片拖排
- **THEN** 只有 board resource 文件產生新 revision，被拖卡的 meta／frontmatter 內容與 revision 皆不變

#### Scenario: plan 端點寬鬆讀取

- **WHEN** board resource 內容為 `{"changes":{"add-a":"n"},"discussions":{}}` 且另一次為非 JSON 文字，各呼叫一次 GET /plan
- **THEN** 前者的 plan 以 add-a 的 rank n 參與排序；後者仍 HTTP 200 並視為全員缺 rank；兩次 board resource 的內容與 revision 皆不變

#### Scenario: 一段壞掉不影響另一段

- **WHEN** board resource 內容為 `{"changes":{"add-a":"n"},"discussions":7}`，呼叫 GET /plan
- **THEN** plan 以 add-a 的 rank n 參與排序；board resource 的內容與 revision 不變

#### Scenario: 排序請求修剪孤兒條目並帶回討論那一段

- **WHEN** board resource 內容為 `{"changes":{"add-a":"n","gone":"c"},"discussions":{"topic":"f"}}`，gone 不是作用中的變更，add-a 與 add-c 同為提案中，對 changes/add-c/rank 送出 `{"anchor":"add-a","position":"before"}`
- **THEN** 新的 board resource 的 changes 段只有 add-a 與 add-c 兩個條目，add-a 仍為 n、add-c 字典序小於 n；discussions 段仍為 `{"topic":"f"}`

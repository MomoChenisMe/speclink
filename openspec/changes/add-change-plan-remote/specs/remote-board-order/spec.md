## MODIFIED Requirements

### Requirement: board resource 為 scope 單文件且 server 不解析

remote 看板順序 SHALL 儲存於 scope 層級的獨立 board resource 文件（DocumentId 的 board order 種類），內容為 JSON 兩段圖（changes：變更名→rank、discussions：slug→rank）。server SHALL 提供 GET /board-order（回內容與 scope ETag；文件缺席為正常態、內容回 null）與 PUT /board-order（If-Match CAS、全文覆寫、editor role 限定、超過大小上限拒絕）；這兩個端點 SHALL 視內容為不透明文本、SHALL NOT 解析或校驗其語意。唯一的例外是 GET /plan 端點（見 server-verb-api）：它 SHALL 以寬鬆方式讀取該文件的 changes 段作為 rank 來源——缺席、無法解析、非物件或缺 changes 段一律視為全員缺 rank，SHALL NOT 因內容壞掉而失敗、SHALL NOT 回寫或修剪文件。PUT 成功 SHALL 於 commit 後發布 invalidate 使訂閱端重讀。拖排 SHALL NOT 觸碰任何變更卡 meta 或討論 frontmatter。

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

### Requirement: remote 排序 overlay 與本地語意同構

remote 清單 SHALL 於桌面 Rust 側合併排序後回傳，UI 元件 SHALL NOT 另做排序。變更卡 SHALL 以 GET /plan 回傳的配置順序排列（plan 的 rank 來源即同一份 board resource，宣告依賴修正與本地同構）；plan 不可得（舊 server、連線失敗）或成環時 SHALL 退回 board resource overlay：具 rank 卡依 rank 位元組字典序升冪、缺 rank 卡排在同欄具 rank 卡之前並維持 server 回傳序、rank 相同以變更名字典序決斷。討論卡 SHALL 維持 board resource overlay：具 rank 依字典序升冪、缺 rank 置頂維持 server 回傳序、同值以 slug 決斷。scope 無 board resource 且 plan 不可得時清單順序 SHALL 與本能力交付前完全一致。

#### Scenario: 依 rank 升冪且缺值置頂

- **WHEN** remote 看板討論欄內存在具 rank 與缺 rank 的討論卡，或 plan 不可得時變更欄內存在具 rank 與缺 rank 的變更卡
- **THEN** 缺 rank 卡依 server 回傳序排欄頂，其後接具 rank 卡依字典序升冪

#### Scenario: 變更卡依 plan 配置順序

- **WHEN** server 的 GET /plan 回 changes 順序為 add-b、add-a（add-a 的 depends_on 含 add-b）而 board resource 的 rank 使 add-a 在前
- **THEN** remote 看板提案中欄的順序為 add-b、add-a

#### Scenario: 無 board resource 時行為不變

- **WHEN** scope 從未拖排（無 board resource）、server 無 plan 端點，且開啟 remote 分頁
- **THEN** 看板各欄順序與本能力交付前的 remote 分頁完全一致

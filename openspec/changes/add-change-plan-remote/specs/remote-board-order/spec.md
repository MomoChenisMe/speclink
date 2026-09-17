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

### Requirement: 拖排寫回以全文 CAS 與一次重試收斂

remote 拖排 SHALL 依序：取當下清單、board resource 與 plan（plan 請求失敗視為不可得）→ 依當前顯示序（plan 可得時為 plan 配置序，否則為 board resource overlay 序）推導被拖卡所在欄成員（畫面上同欄的全部卡，含 plan 未配置而排在欄尾的壞 meta 卡——它照常參與補章與落點計算）→ 欄內有缺 rank 卡、或欄內 rank 序與顯示序不一致時，依顯示序整欄補章（等距鍵只寫入 board resource）→ 以落點鄰居中點鍵更新被拖卡條目（消失的鄰居視為開放端、鄰居現值逆序時棄上界保底）→ plan 可得時執行宣告依賴檢查（見 change-plan「plan 與 change depends 的 remote 臂」；plan 未配置的卡不列入檢查序列），違反即停止、SHALL NOT 發出 PUT → 修剪不在現行清單的條目 → PUT 全文帶 If-Match。全員具 rank 且 rank 序與顯示序一致時 SHALL 只改被拖卡的條目。本需求的步驟在桌面的拖排寫回層執行；看板前端的不合法落點灰化另依 desktop-app「拖排時不合法落點灰化」判定。收到 409 SHALL 重讀重算後重試恰一次；重試仍失敗 SHALL 以單行錯誤呈現並刷新至 server 現況，SHALL NOT 保留未落檔的假象順序。

#### Scenario: 穩態拖排落位並共享

- **WHEN** editor 把卡拖到兩鄰居之間放開且 PUT 成功
- **THEN** 該欄新序跨重啟持久，另一台已連線 client 的看板數秒內反映同序

#### Scenario: 409 重讀後落位

- **WHEN** 拖排 PUT 因他人剛寫入而 409
- **THEN** 桌面重讀 board resource 重算中點後重試一次成功，最終順序含兩人的變動

#### Scenario: 重試仍敗不留假象

- **WHEN** 重試的 PUT 仍回 409
- **THEN** 呈現單行錯誤，看板刷新後顯示 server 現況順序

#### Scenario: rank 序與顯示序不一致時整欄重派

- **WHEN** 提案中欄 add-c 的 depends_on 含 add-a、board resource 中 add-c 的 rank 小於 add-a（plan 顯示序 add-a、add-c），editor 把 add-x 拖到 add-a 與 add-c 之間放開
- **THEN** PUT 的全文中該欄依顯示序重派 rank，add-x 的新鍵介於 add-a 與 add-c 之間，刷新後顯示 add-a、add-x、add-c

#### Scenario: plan 略過的前置不擋拖排寫回

- **WHEN** 提案中欄顯示 add-a、add-b、add-x（add-x 的 meta 損壞而被 plan 略過、排在欄尾），add-b 的 depends_on 含 add-x，桌面收到把 add-b 移到 add-a 之前的拖排寫回請求（prevId 為 null、nextId 為 add-a）
- **THEN** PUT 成功，全文中 add-b 的新鍵小於 add-a，不回依賴錯誤

#### Scenario: 拖到欄尾的略過卡之後落在欄底

- **WHEN** 提案中欄顯示 add-a、add-b、add-x（add-x 被 plan 略過、排在欄尾），editor 把 add-a 拖到 add-x 之後放開
- **THEN** PUT 的全文中 add-a 的新鍵大於 add-b 與 add-x，刷新後 add-a 排在 add-b 之後

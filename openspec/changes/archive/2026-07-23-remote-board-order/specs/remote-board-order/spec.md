## ADDED Requirements

### Requirement: board resource 為 scope 單文件且 server 不解析

remote 看板順序 SHALL 儲存於 scope 層級的獨立 board resource 文件（DocumentId 的 board order 種類），內容為 JSON 兩段圖（changes：變更名→rank、discussions：slug→rank）。server SHALL 提供 GET /board-order（回內容與 scope ETag；文件缺席為正常態、內容回 null）與 PUT /board-order（If-Match CAS、全文覆寫、editor role 限定、超過大小上限拒絕）；server SHALL 視內容為不透明文本、SHALL NOT 解析或校驗其語意。PUT 成功 SHALL 於 commit 後發布 invalidate 使訂閱端重讀。拖排 SHALL NOT 觸碰任何變更卡 meta 或討論 frontmatter。

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

---
### Requirement: remote 排序 overlay 與本地語意同構

remote 清單 SHALL 於桌面 Rust 側以 board resource 的 rank 合併排序後回傳：具 rank 卡依 rank 位元組字典序升冪、缺 rank 卡排在同欄具 rank 卡之前並維持 server 回傳序、rank 相同以變更名／討論 slug 字典序決斷；UI 元件 SHALL NOT 另做排序。scope 無 board resource 時清單順序 SHALL 與本能力交付前完全一致。

#### Scenario: 依 rank 升冪且缺值置頂

- **WHEN** remote 看板同一欄內存在具 rank 與缺 rank 的卡片
- **THEN** 缺 rank 卡依 server 回傳序排欄頂，其後接具 rank 卡依字典序升冪，與本地同構

#### Scenario: 無 board resource 時行為不變

- **WHEN** scope 從未拖排（無 board resource）且開啟 remote 分頁
- **THEN** 看板各欄順序與本能力交付前的 remote 分頁完全一致

---
### Requirement: 拖排寫回以全文 CAS 與一次重試收斂

remote 拖排 SHALL 依序：取當下清單與 board resource → 推導被拖卡所在欄成員 → 欄內有缺 rank 卡時依當前顯示序整欄補章（等距鍵只寫入 board resource）→ 以落點鄰居中點鍵更新被拖卡條目（消失的鄰居視為開放端、鄰居現值逆序時棄上界保底）→ 修剪不在現行清單的條目 → PUT 全文帶 If-Match。收到 409 SHALL 重讀重算後重試恰一次；重試仍失敗 SHALL 以單行錯誤呈現並刷新至 server 現況，SHALL NOT 保留未落檔的假象順序。

#### Scenario: 穩態拖排落位並共享

- **WHEN** editor 把卡拖到兩鄰居之間放開且 PUT 成功
- **THEN** 該欄新序跨重啟持久，另一台已連線 client 的看板數秒內反映同序

#### Scenario: 409 重讀後落位

- **WHEN** 拖排 PUT 因他人剛寫入而 409
- **THEN** 桌面重讀 board resource 重算中點後重試一次成功，最終順序含兩人的變動

#### Scenario: 重試仍敗不留假象

- **WHEN** 重試的 PUT 仍回 409
- **THEN** 呈現單行錯誤，看板刷新後顯示 server 現況順序

---
### Requirement: 損壞容錯與孤兒條目修剪

board resource 內容非法（無法解析為預期 JSON 形狀）時，桌面 SHALL 視為全員缺 rank 以回退序渲染、SHALL NOT 使看板失效；下一次成功的拖排 PUT SHALL 重建合法文件。排序時查無對應卡的 rank 條目 SHALL 被忽略；每次 PUT 重寫全文時 SHALL 修剪不在現行清單中的條目。

#### Scenario: 壞文件退回回退序

- **WHEN** board resource 內容為非法 JSON 且開啟 remote 分頁
- **THEN** 看板以回退序正常渲染無錯誤彈窗；此後第一次拖排成功後文件恢復合法且該欄新序生效

#### Scenario: 已封存卡的條目被修剪

- **WHEN** 某變更封存後 editor 執行任一次拖排
- **THEN** PUT 寫回的全文不含該已封存變更的條目

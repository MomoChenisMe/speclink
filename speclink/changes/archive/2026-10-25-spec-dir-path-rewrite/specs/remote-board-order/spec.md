## MODIFIED Requirements

### Requirement: 拖排寫回以全文 CAS 與一次重試收斂

<!-- BEFORE: 以 openspec/ 與 .openspec.yaml 描述規格資料夾與變更資料檔 -->

remote 拖排 SHALL 依序：取當下清單、board resource 與 plan（只在拖變更卡時取，且在 board resource 之後讀，使 plan 所依據的 board 狀態不早於 If-Match 的 revision；請求失敗視為不可得）→ 依當前顯示序（plan 可得時為 plan 配置序，否則為 board resource overlay 序）推導被拖卡所在欄成員（畫面上同欄的全部卡，含 plan 未配置而排在欄尾的壞 meta 卡——它照常參與補章與落點計算；被拖卡本身為 plan 略過的卡時 SHALL 以與本地相同的單行錯誤 `invalid speclink/changes/<name>/.meta.yaml: <原因>` 拒絕、SHALL NOT 發出 PUT）→ 欄內有缺 rank 卡、或欄內 rank 序與顯示序不一致時，依顯示序整欄補章（等距鍵只寫入 board resource）→ 以落點鄰居中點鍵更新被拖卡條目（消失的鄰居視為開放端、鄰居現值逆序時棄上界保底）→ plan 可得時執行宣告依賴檢查（見 change-plan「plan 與 change depends 的 remote 臂」；plan 未配置的卡不列入檢查序列），違反即停止、SHALL NOT 發出 PUT → 修剪不在現行清單的條目 → PUT 全文帶 If-Match。全員具 rank 且 rank 序與顯示序一致時 SHALL 只改被拖卡的條目。本需求的步驟在桌面的拖排寫回層執行；看板前端的不合法落點灰化另依 desktop-app「拖排時不合法落點灰化」判定。收到 409 SHALL 重讀重算後重試恰一次；重試仍失敗 SHALL 以單行錯誤呈現並刷新至 server 現況，SHALL NOT 保留未落檔的假象順序。

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

#### Scenario: 拖動 plan 略過的卡被拒

- **WHEN** 提案中欄顯示 add-a、add-x（add-x 的 meta 損壞而被 plan 略過，原因為 bad yaml），editor 把 add-x 拖到 add-a 之前放開
- **THEN** 顯示單行錯誤 `invalid speclink/changes/add-x/.meta.yaml: bad yaml`，未發出 PUT /board-order

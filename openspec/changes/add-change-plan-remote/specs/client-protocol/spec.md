## ADDED Requirements

### Requirement: plan 回應 payload

protocol SHALL 以 Rust 型別定義 GET /plan 的回應：`waves`（陣列，每項 `index` 整數與 `changes` 字串陣列）、`changes`（陣列，每項 `name`、`wave`、`stage`、`dependsOn`、`overlaps`（每項 `change` 與 `capabilities`）、`blockedBy`、`ready`）、`next`（字串或 null）、`skipped`（每項 `change` 與 `reason`）。欄位一律 camelCase，陣列欄位缺席 SHALL 讀作空陣列。remote client SHALL 提供型別化的 plan 呼叫；CLI remote 模式 SHALL 把回應轉回引擎的 plan 報告型別後共用 fs 模式的渲染，SHALL NOT 另寫第二份渲染。

#### Scenario: 回應反序列化

- **WHEN** client 收到 `{"waves":[{"index":1,"changes":["a"]}],"changes":[{"name":"a","wave":1,"stage":"proposed","dependsOn":[],"overlaps":[],"blockedBy":[],"ready":true}],"next":"a","skipped":[]}`
- **THEN** 反序列化成功，next 為 "a"，changes[0].ready 為 true

#### Scenario: 缺陣列欄位容忍

- **WHEN** client 收到的某 change 項缺 overlaps 鍵
- **THEN** 反序列化成功且 overlaps 為空陣列

### Requirement: 依賴寫入請求與回應

protocol SHALL 定義 POST /changes/{name}/depends 的請求 `{ on: string[], remove: boolean }` 與回應 `{ change: string, dependsOn: string[] }`，camelCase。remote client SHALL 提供型別化的 set_depends 呼叫；409 回應的 reason（depends-self、depends-target-invalid、dependency-cycle）SHALL 進入標準 error reason registry，CLI 與桌面 SHALL 以引擎同款單行訊息呈現。

#### Scenario: 請求序列化

- **WHEN** client 以 on ["add-a","add-c"]、remove false 呼叫 set_depends
- **THEN** 送出的 body 為 `{"on":["add-a","add-c"],"remove":false}`

#### Scenario: 409 reason 可判讀

- **WHEN** server 回 409 且 reason 為 dependency-cycle
- **THEN** client 的錯誤型別可機器判別該 reason，訊息含成環路徑

### Requirement: remote 變更清單的排程欄位

桌面 remote 資料源的變更清單 SHALL 以 GET /plan 的配置順序排列變更項（討論項維持 board resource overlay），並在每項疊 `wave`、`blockedBy`、`dependsOn`、`overlaps` 四欄，頂層 `planError`（字串或 null）；四欄 SHALL 為桌面側組裝，SHALL NOT 改動 server list 端點的回應與 ChangeSummary 型別。plan 端點回 409 dependency-cycle 時順序 SHALL 退回 board overlay、四欄缺席、planError 為訊息；plan 端點回 404（舊 server）或連線失敗時 SHALL 退回 board overlay、四欄缺席、planError 為 null。

#### Scenario: remote 清單帶排程欄位

- **WHEN** remote 分頁載入，server 的 plan 回 add-a 在 add-b 之前且 add-b blockedBy ["add-a"]
- **THEN** 桌面清單 add-a 在 add-b 之前，add-b 項含 wave=2 與 blockedBy=["add-a"]，看板顯示波次章與阻擋章

#### Scenario: 舊 server 退回

- **WHEN** server 對 GET /plan 回 404
- **THEN** 清單順序與本需求引入前一致、各項無四欄、planError 為 null、看板無章且無提示列

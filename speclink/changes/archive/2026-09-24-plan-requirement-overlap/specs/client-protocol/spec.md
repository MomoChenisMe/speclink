## MODIFIED Requirements

### Requirement: plan 回應 payload

<!-- BEFORE: changes 每項七鍵，無 requirementOverlap 與 archiveAfter -->

protocol SHALL 以 Rust 型別定義 GET /plan 的回應：`waves`（陣列，每項 `index` 整數與 `changes` 字串陣列）、`changes`（陣列，每項 `name`、`wave`、`stage`、`dependsOn`、`overlaps`（每項 `change` 與 `capabilities`）、`blockedBy`、`ready`、`requirementOverlap`（每項 `change`、`capability`、`requirement`、`ownOperation`、`otherOperation`、`conflict`）、`archiveAfter`）、`next`（字串或 null）、`skipped`（每項 `change` 與 `reason`）。欄位一律 camelCase，陣列欄位缺席 SHALL 讀作空陣列（含 `requirementOverlap` 與 `archiveAfter`，讓新 client 讀舊 server）。remote client SHALL 提供型別化的 plan 呼叫；CLI remote 模式 SHALL 把回應轉回引擎的 plan 報告型別後共用 fs 模式的渲染，轉換 SHALL 逐欄搬運九鍵、SHALL NOT 以空值填任何欄位，SHALL NOT 另寫第二份渲染；`stage` 為 proposed／in-progress／ready 以外的值、或 `ownOperation`／`otherOperation` 為 ADDED／MODIFIED／REMOVED／RENAMED 以外的值時，轉換 SHALL 失敗並回報錯誤，SHALL NOT 猜測；`waves` 列出 `changes` 沒有的名稱時轉換同樣 SHALL 失敗並回報錯誤，SHALL NOT 使 CLI 崩潰。JSON Schema 匯出 SHALL 隨 Rust 型別更新。

#### Scenario: 回應反序列化

- **WHEN** client 收到 `{"waves":[{"index":1,"changes":["a"]}],"changes":[{"name":"a","wave":1,"stage":"proposed","dependsOn":[],"overlaps":[],"blockedBy":[],"ready":true,"requirementOverlap":[{"change":"b","capability":"desktop-app","requirement":"看板與任務","ownOperation":"MODIFIED","otherOperation":"MODIFIED","conflict":false}],"archiveAfter":["b"]}],"next":"a","skipped":[]}`
- **THEN** 反序列化成功，next 為 "a"，changes[0].ready 為 true，changes[0].requirementOverlap[0].conflict 為 false，changes[0].archiveAfter 為 ["b"]

#### Scenario: 缺陣列欄位容忍

- **WHEN** client 收到 `{"waves":[],"changes":[{"name":"a","wave":1,"stage":"proposed","dependsOn":[],"blockedBy":[],"ready":true}],"next":"a","skipped":[]}`（change 項缺 overlaps、requirementOverlap、archiveAfter 鍵）
- **THEN** 反序列化成功且 changes[0].overlaps、requirementOverlap、archiveAfter 皆為空陣列

#### Scenario: wave 成員不在 changes 時轉換失敗

- **WHEN** client 收到 `{"waves":[{"index":1,"changes":["add-a"]}],"next":"add-a"}`（缺 changes 鍵）並轉回引擎的 plan 報告
- **THEN** 轉換回報錯誤、訊息含 `add-a`，不崩潰

#### Scenario: 轉回引擎報告逐欄搬運

- **WHEN** client 收到 changes[0] 帶一項 requirementOverlap 與 archiveAfter ["b"] 的回應並轉回引擎的 plan 報告
- **THEN** 引擎報告的 changes[0].requirement_overlap 有一項且 archive_after 為 ["b"]

#### Scenario: 不認得的操作轉換失敗

- **WHEN** client 收到的 requirementOverlap 項 ownOperation 為 `MOVED` 並轉回引擎的 plan 報告
- **THEN** 轉換回報錯誤、訊息含 `MOVED`，不崩潰

## MODIFIED Requirements

### Requirement: 過期判定單源共用

drift 的 Specs 維度、bulk archive 的 readiness 預檢、單筆 archive 的合併守門與 change 驗證（`speclink validate`）SHALL 共用同一過期判定實作；四處對同一 delta 的過期認定 SHALL 一致，drift、bulk 預檢與 validate 的 reason 文案 SHALL 表述拒絕語意（archive 將拒絕，而非跳過）。

<!-- REMOVED-SCENARIO: 三處判定一致 -->

#### Scenario: 四處判定一致

- **WHEN** 同一 change 的 delta 含一條過期 MODIFIED，分別執行 speclink validate、speclink drift、bulk archive 預檢與單筆 speclink archive
- **THEN** 四處皆認定該操作過期：validate 列為 error、drift 列為 spec assumption、bulk 預檢列為未就緒、單筆 archive 拒絕，且四者指向同一 capability 與需求名

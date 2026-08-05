## MODIFIED Requirements

### Requirement: 驗證收尾迴圈

<!-- BEFORE: 零 findings 一律記錄空 discovery round 並由主線執行 verify stamp，無 quality 時序區分（基準：verify-station-parity 併入正典後的條文） -->

技能 SHALL 於主線接手 fork 報告後進行 remediation triage；修正一律回主線依專案 TDD 慣例執行，fork 不得修改任何檔案。Discovery 有 findings 時 SHALL 詢問三選項——修正後重驗／接受現狀蓋章（`verify stamp --accept`）／先不蓋章結束；零 findings 時，單站直接呼叫 SHALL 記錄空 discovery round 並由主線執行 `verify stamp`；於 quality 時序中（由 /speclink-quality 依序呼叫時）SHALL 記錄空 discovery round 後改以「先不蓋章」結束，蓋章延至 quality 的複驗階段。

令 Bn 為第 n 輪 triage 後「未接受且要求修正」的必修集合。每輪 validation 寫入工單後 SHALL 依下列規則收尾：

- Bn 為空且沒有 accepted findings：執行 `verify stamp`，結果為 passed clean
- Bn 為空且仍有 accepted findings：等待使用者明示 `verify stamp --accept`，結果為 passed with reservations
- `0 < |Bn| < |Bn-1|`：允許使用者再次選擇修正後驗收、接受現狀或先不蓋章
- `|Bn| >= |Bn-1|`：立即以 failed 結束自動迴圈，保留工單、不蓋章且不得自動再試

blocking set 的縮小只決定能否繼續自動修正，SHALL NOT 被描述為品質分數或通過。技能 SHALL NOT 設固定最大輪數；每次允許續跑都必須嚴格下降。互動詢問於不支援選單工具的環境 SHALL 以純文字詢問並等待回覆。

#### Scenario: 乾淨首輪蓋章

- **WHEN** 單站直接呼叫且 discovery 的三維度皆零 findings
- **THEN** 技能記錄零 findings 的 discovery round，執行 `verify stamp` 並回報 passed clean

#### Scenario: quality 時序中乾淨首輪先不蓋章

- **WHEN** 於 quality 時序中 discovery 的三維度皆零 findings
- **THEN** 技能記錄零 findings 的 discovery round，以「先不蓋章」結束，驗證工單與 verify snapshot 保留，不執行 `verify stamp`

#### Scenario: 有進展時允許再驗收

- **WHEN** 上輪有兩筆必修，validation 後剩一筆未解且沒有直接 regression
- **THEN** 技能記錄新輪並允許再次選擇修正，且不得宣稱已通過

#### Scenario: 第一個無進展輪立即停止

- **WHEN** 上輪有一筆必修，validation 後同一筆仍未解
- **THEN** 技能記錄該輪後回報 failed，保留工單、不蓋章且不再啟動 validation

#### Scenario: 只剩保留事項

- **WHEN** validation 已解決所有必修但末輪仍帶 accepted findings
- **THEN** 技能等待使用者明示 `verify stamp --accept`，不再為 accepted findings 啟動 validation

#### Scenario: 先不蓋章離場

- **WHEN** 使用者於可選擇節點選擇「先不蓋章」
- **THEN** 技能結束，驗證工單與 verify snapshot 保留，metadata 不變

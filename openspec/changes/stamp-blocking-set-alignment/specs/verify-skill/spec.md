## MODIFIED Requirements

### Requirement: 驗證收尾迴圈

<!-- BEFORE: Discovery 有任何 findings 即詢問三選項；Bn 為空且無 accepted findings 才蓋章；SUGGESTION 也逼修或逼問 --accept -->

技能 SHALL 於主線接手 fork 報告後進行 remediation triage；修正一律回主線依專案 TDD 慣例執行，fork 不得修改任何檔案。triage 的阻斷分界 SHALL 落在嚴重度：必修＝CRITICAL／WARNING 級；可裁事項 SHALL 一律以 SUGGESTION 級記錄、SHALL NOT 以 WARNING 級入單，且 SHALL NOT 進入接受機制——SUGGESTION 級 findings 不擋蓋章、無需任何人批准。

Discovery 有必修 findings 時 SHALL 詢問三選項——修正後重驗／接受現狀蓋章（`verify stamp --accept`）／先不蓋章結束；零 findings 或僅 SUGGESTION 級 findings 時，單站直接呼叫 SHALL 記錄該 discovery round 並由主線執行 `verify stamp`（乾淨蓋章）；於 quality 時序中（由 /speclink-quality 依序呼叫時）SHALL 記錄該 discovery round 後改以「先不蓋章」結束，且必修集合淨空的 validation 輪亦 SHALL 以「先不蓋章」結束，蓋章一律延至 quality 的收尾補蓋；惟編排方明示本次呼叫為收尾補蓋時，此禁蓋例外 SHALL NOT 適用——該呼叫中必修淨空的輪即蓋。收尾補蓋呼叫 SHALL 於入口即分流：末輪必修淨空且凍結點後內容無移動時，跳過檢查 pass 直接執行 `verify stamp`、不另落空輪；有移動時於同一呼叫內先跑 validation 輪至必修淨空再蓋，SHALL NOT 對未驗證的移動直接補蓋。

令 Bn 為第 n 輪 triage 後「未接受且要求修正」的必修集合。每輪 validation 寫入工單後 SHALL 依下列規則收尾：

- Bn 為空且沒有 accepted 必修 findings：執行 `verify stamp`，結果為 passed clean——末輪殘留的 SUGGESTION 級 findings 不擋章、無需批准
- Bn 為空且仍有 accepted 必修 findings：等待使用者明示 `verify stamp --accept`，結果為 passed with reservations
- `0 < |Bn| < |Bn-1|`：允許使用者再次選擇修正後驗收、接受現狀或先不蓋章
- `|Bn| >= |Bn-1|`：立即以 failed 結束自動迴圈，保留工單、不蓋章且不得自動再試

blocking set 的縮小只決定能否繼續自動修正，SHALL NOT 被描述為品質分數或通過。技能 SHALL NOT 設固定最大輪數；每次允許續跑都必須嚴格下降。互動詢問於不支援選單工具的環境 SHALL 以純文字詢問並等待回覆。

#### Scenario: 乾淨首輪蓋章

- **WHEN** 單站直接呼叫且 discovery 的三維度皆零 findings
- **THEN** 技能記錄零 findings 的 discovery round，執行 `verify stamp` 並回報 passed clean

#### Scenario: 僅 SUGGESTION 首輪直接蓋章

- **WHEN** 單站直接呼叫且 discovery 僅記錄 SUGGESTION 級 findings、無任何必修
- **THEN** 技能記錄該 discovery round 後不發三選項詢問，直接執行 `verify stamp` 並回報 passed clean，報告列出 SUGGESTION 清單

#### Scenario: quality 時序中乾淨首輪先不蓋章

- **WHEN** 於 quality 時序中 discovery 的三維度皆零 findings
- **THEN** 技能記錄零 findings 的 discovery round，以「先不蓋章」結束，驗證工單與 verify snapshot 保留，不執行 `verify stamp`

#### Scenario: quality 時序中複驗淨空仍先不蓋章

- **WHEN** 於 quality 時序中（本次呼叫非收尾補蓋）validation 輪後必修集合為空且無 accepted 必修 findings
- **THEN** 技能記錄該輪後以「先不蓋章」結束，不執行 `verify stamp`，蓋章延至 quality 的收尾補蓋

#### Scenario: quality 收尾補蓋於入口分流

- **WHEN** 收尾補蓋呼叫進入 verify 站且末輪必修淨空、凍結點後內容無移動
- **THEN** 技能跳過檢查 pass 直接執行 `verify stamp`，不另落空輪

#### Scenario: quality 收尾補蓋前內容再移動則先驗後蓋

- **WHEN** quality 收尾補蓋時凍結點後仍有內容移動
- **THEN** 技能於同一呼叫內先執行 validation 輪，必修淨空後即執行 `verify stamp`（收尾補蓋呼叫不受禁蓋例外攔截），不對未驗證的移動直接補蓋

#### Scenario: 有進展時允許再驗收

- **WHEN** 上輪有兩筆必修，validation 後剩一筆未解且沒有直接 regression
- **THEN** 技能記錄新輪並允許再次選擇修正，且不得宣稱已通過

#### Scenario: 第一個無進展輪立即停止

- **WHEN** 上輪有一筆必修，validation 後同一筆仍未解
- **THEN** 技能記錄該輪後回報 failed，保留工單、不蓋章且不再啟動 validation

#### Scenario: 只剩保留事項

- **WHEN** validation 已解決所有必修但末輪仍帶 accepted 必修 findings
- **THEN** 技能等待使用者明示 `verify stamp --accept`，不再為 accepted findings 啟動 validation

#### Scenario: 先不蓋章離場

- **WHEN** 使用者於可選擇節點選擇「先不蓋章」
- **THEN** 技能結束，驗證工單與 verify snapshot 保留，metadata 不變

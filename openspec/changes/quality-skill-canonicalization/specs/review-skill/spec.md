## MODIFIED Requirements

### Requirement: 審查後的迴圈與收尾

<!-- BEFORE: 零 findings 的 discovery 一律當場自動蓋章（乾淨首輪自動蓋章），無 quality 時序區分 -->

Discovery 呈現與 triage 後，技能 SHALL 沿既有三選項讓使用者選擇：修正後重審／接受現狀蓋章／先不蓋章。修正 SHALL 一律由主線依專案 TDD 慣例執行，sub-agent 不得修改檔案；修正後 SHALL 先通過「修復迴圈的驗證門」，再開始 validation。於 quality 時序中（由 /speclink-quality 依序呼叫時），零 findings 的 discovery SHALL NOT 當場蓋章，SHALL 改走既有「先不蓋章」離場，蓋章延至 quality 的複驗階段；單站直接呼叫時行為不變。

每輪 validation 後，技能 SHALL 以未接受的必修集合 Bn 與上輪 Bn-1 比較：

- Bn 為空且沒有 accepted findings 時 SHALL 執行 review stamp，結果為 passed clean
- Bn 為空且仍有 accepted findings 時 SHALL 推薦使用者明示 review stamp --accept，結果為 passed with reservations
- Bn 非空且數量嚴格小於 Bn-1 時 SHALL 允許使用者再次選擇修正後驗收、接受現狀或先不蓋章
- Bn 數量大於或等於 Bn-1 時 SHALL 在記錄本輪後立即以 failed 結束自動迴圈，保留工單、不蓋章且不得自動再試

blocking set 的縮小只決定能否繼續自動修正，SHALL NOT 被描述為品質分數或通過。技能 SHALL NOT 設固定最大輪數；每次允許續跑都必須嚴格下降。互動工具不可用時 SHALL 以純文字詢問並等待回覆。

#### Scenario: 乾淨首輪自動蓋章

- **WHEN** 單站直接呼叫且 discovery 的兩軸皆零 findings
- **THEN** 技能記錄零 findings 的 discovery round，執行 review stamp 並回報 passed clean

#### Scenario: quality 時序中乾淨首輪先不蓋章

- **WHEN** 於 quality 時序中 discovery 的兩軸皆零 findings
- **THEN** 技能記錄零 findings 的 discovery round，以「先不蓋章」離場，工單與 host-local snapshot 保留，不執行 review stamp

#### Scenario: 有進展時允許再驗收

- **WHEN** 上輪有兩筆必修，validation 後剩一筆未解且沒有直接 regression
- **THEN** 技能記錄新輪並允許再次選擇修正，SHALL NOT 宣稱已通過

#### Scenario: 第一個無進展輪立即停止

- **WHEN** 上輪有一筆必修，validation 後同一筆仍未解
- **THEN** 技能記錄該輪後回報 failed，保留工單、不蓋章且不再派出 sub-agent

#### Scenario: 只剩保留事項

- **WHEN** validation 已解決所有必修但末輪仍帶 accepted findings
- **THEN** 技能推薦 review stamp --accept，不再為 accepted items 啟動 validation

#### Scenario: 先不蓋章離場

- **WHEN** 使用者在可選擇的節點選擇先不蓋章
- **THEN** 技能結束，工單與 host-local snapshot 保留，metadata 不變

### Requirement: 審查技能的生成與正典化

<!-- BEFORE: workflow 行為不含 quality 入口的版本（discuss? → propose → apply ⇄ ingest → (review? ∥ verify?) → archive） -->

`speclink update` SHALL 生成 `/speclink-review` 技能檔至 claude 與 codex 兩工具的技能目錄，內容以引擎內的正典模板為準（golden 對照涵蓋）。同次更新 SHALL 將生成之 CLAUDE.md／AGENTS.md 的 workflow 行改為含品質關卡與並行品質站的版本（`discuss? → propose → apply ⇄ ingest → (quality? | review? ∥ verify?) → archive`），並於技能使用清單加入審查站的觸發時機（實作完成、封存之前、由使用者判斷是否執行）。

#### Scenario: 技能檔生成

- **WHEN** 於已啟用 speclink 的專案執行 `speclink update`
- **THEN** claude 與 codex 的技能目錄各出現 speclink-review 技能檔，且內容與 golden 對照一致

#### Scenario: workflow 行更新

- **WHEN** `speclink update` 完成後讀取生成的 CLAUDE.md
- **THEN** workflow 行含 `(quality? | review? ∥ verify?)` 且技能清單含審查站條目

## MODIFIED Requirements

### Requirement: 審查後的迴圈與收尾

<!-- BEFORE: 零 findings 的 discovery 一律當場自動蓋章（乾淨首輪自動蓋章），無 quality 時序區分 -->

Discovery 呈現與 triage 後，技能 SHALL 沿既有三選項讓使用者選擇：修正後重審／接受現狀蓋章／先不蓋章。修正 SHALL 一律由主線依專案 TDD 慣例執行，sub-agent 不得修改檔案；修正後 SHALL 先通過「修復迴圈的驗證門」，再開始 validation。於 quality 時序中（由 /speclink-quality 依序呼叫時），零 findings 的 discovery 與必修集合淨空的 validation 輪皆 SHALL NOT 當場蓋章，SHALL 改走既有「先不蓋章」離場，蓋章延至 quality 的收尾補蓋；惟編排方明示本次呼叫為收尾補蓋時，此禁蓋例外 SHALL NOT 適用——該呼叫中淨空的輪即蓋。quality 收尾補蓋 SHALL 區分乾淨末輪的來源：外部守門失敗留下者沿既有路徑直接重試 stamp；quality 時序刻意留下者 SHALL 僅於收尾補蓋呼叫中蓋章——該呼叫先以 review scope 確認凍結點後內容未再移動，無移動即重試 stamp，有移動則於同一呼叫內先跑 validation 輪至必修淨空再蓋，SHALL NOT 對未驗證的移動直接補蓋；同一乾淨末輪若由非收尾補蓋的呼叫進入，無論凍結點後有無移動皆 SHALL NOT 蓋章。單站直接呼叫時行為不變。

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

#### Scenario: quality 時序中複驗淨空仍先不蓋章

- **WHEN** 於 quality 時序中（本次呼叫非收尾補蓋）validation 輪後必修集合為空且無 accepted findings
- **THEN** 技能記錄該輪後以「先不蓋章」離場，不執行 review stamp，蓋章延至 quality 的收尾補蓋

#### Scenario: 非收尾呼叫進入乾淨末輪不蓋章

- **WHEN** quality 時序中非收尾補蓋的呼叫進入已存在的乾淨未蓋章末輪，且 review scope 顯示凍結點後無內容移動
- **THEN** 技能回報無新內容可判並結束，不執行 review stamp、不動工單

#### Scenario: quality 收尾補蓋前內容再移動則先驗後蓋

- **WHEN** quality 收尾補蓋時 review scope 顯示乾淨末輪凍結點後仍有內容移動
- **THEN** 技能於同一呼叫內先執行 validation 輪，必修淨空後即執行 review stamp（收尾補蓋呼叫不受禁蓋例外攔截），不對未驗證的移動直接補蓋

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

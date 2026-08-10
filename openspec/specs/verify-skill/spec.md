# verify-skill Specification

## Purpose

TBD - created by archiving change 'verify-station-parity'. Update Purpose after archive.

## Requirements

### Requirement: 驗證技能的工單落地

verify 技能 SHALL 於檢查段（fork）結束時依任務完成度分流：任務全數完成時先取得 frozen verify scope，再以 `verify add-round` 將相同 phase、patch hash、Scope 與 findings 寫入驗證工單並於報告中告知；任務未全數完成時維持對話報告（進度盤點），不呼叫 `verify scope`、不落工單。技能模板 SHALL 於 claude 與 codex 兩工具正典化生成，golden 對照涵蓋。Completeness／Correctness／Coherence 三維度的檢查內容與 CRITICAL／WARNING／SUGGESTION 分級 SHALL 不因本需求改變。

#### Scenario: 成品驗證落 structured 工單

- **WHEN** 對任務全數完成的 change 執行 verify 技能，resolved scope 為 discovery 且檢查產出 findings
- **THEN** 檢查段執行 `verify add-round` 成功，Round 1 記錄相同的 Phase／Patch／Scope，報告說明工單已建立與輪次

#### Scenario: 中途盤點不落工單

- **WHEN** 對任務 3/5 的 change 執行 verify 技能
- **THEN** 技能輸出對話報告（含未完成任務），不執行 `verify scope` 或 `verify add-round`，change 目錄無 `verify.md`

#### Scenario: 技能模板生成

- **WHEN** 執行 `speclink update`
- **THEN** claude 與 codex 的 verify 技能檔更新為含 frozen scope、structured 工單與有限續輪流程的版本，且與 golden 對照一致


<!-- @trace
source: verify-station-parity
updated: 2026-08-06
-->

---
### Requirement: 驗證續輪只驗收修正

任務全數完成且無工單時，技能 SHALL 執行唯一一次 discovery：讀取全部 change artifacts，並以 frozen change patch 及判斷直接影響所需的呼叫端／測試作程式碼證據，完整執行 Completeness／Correctness／Coherence。已有 structured 工單時 SHALL 執行 validation，只將上輪未解 findings、accepted 清單、remediation patch 與必要脈絡交給 fork；逐筆判定原 finding 已解／未解，並只回報 remediation patch 直接引入的 regression。

未解 finding SHALL 以原文寫入新輪；已解 finding SHALL 從新輪移除。Validation SHALL NOT 重新掃描整份 change、finding 所在整檔或未修改區域，也 SHALL NOT 新增未修改區域的 smell、SUGGESTION 或既存問題。legacy 工單缺少可對應的 phase／patch snapshot 時 SHALL fail closed，保留工單並等待使用者明示 discard 後重新 discovery，不得以 touched 整檔替代。

#### Scenario: 首輪完整驗證 frozen change

- **WHEN** change 有 12 項 requirements，而 frozen discovery patch 只含 touched 檔案的三個 hunks
- **THEN** fork 對 12 項 requirements 執行完整三維度驗證，程式碼 discovery 面限定於該三個 hunks及必要脈絡，不掃描任意 workspace 既存問題

#### Scenario: 續輪只驗收未解 finding 與修正 patch

- **WHEN** Round 1 有兩筆未解 findings，remediation patch 只修改其中一檔並新增一個呼叫端
- **THEN** validation 只判定兩筆原 findings 與該 patch 的直接 regression，不重新執行整份 change discovery

#### Scenario: 未解 finding 保留原文

- **WHEN** validation 判定一筆上輪 finding 仍未解
- **THEN** 新 round 以原 severity、path 與 text 寫回該 finding，不以改寫文字假裝 blocking set 已縮小

#### Scenario: legacy 工單缺 snapshot 時 fail closed

- **WHEN** 工單有 findings但 lastRound.patchHash 為 null，且 Host 沒有可對應 snapshot
- **THEN** 技能保留工單、說明無法精確重建 remediation delta，等待使用者明示 discard 後重新 discovery，不重驗 touched 整檔


<!-- @trace
source: verify-station-parity
updated: 2026-08-06
-->

---
### Requirement: 驗證收尾迴圈

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


<!-- @trace
source: stamp-blocking-set-alignment
updated: 2026-08-10
-->

---
### Requirement: 驗證續輪重大晚發問題的安全退出

Validation 偶然看見與 remediation patch 無關的新問題時，技能 SHALL NOT 將它加入目前 findings 或重新開啟 discovery。只有問題同時具有現實觸發路徑、重現方式／失敗測試／明確 invariant 破壞之一，且影響安全、資料損失或錯誤行為時，技能 SHALL 以 scope changed／failed 結束本站，保留原工單且不蓋章，並建議另開 discovery 或衍生 change。證據不足或不達門檻的事項 SHALL 僅列為後續提示，不阻斷目前 validation。

#### Scenario: 無關 smell 不加入續輪

- **WHEN** validation 期間注意到未修改鄰檔的 possible Duplicated Code
- **THEN** 該事項不寫入目前 round、不改變 blocking set，也不觸發新的 discovery

#### Scenario: 有證據的資料損失問題終止本站

- **WHEN** validation 期間發現與 remediation patch 無關但有失敗測試可重現的資料損失
- **THEN** 技能回報 scope changed／failed、保留工單且不蓋章，建議另開 discovery 或衍生 change

<!-- @trace
source: verify-station-parity
updated: 2026-08-06
-->
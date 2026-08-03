## MODIFIED Requirements

### Requirement: 審查流程的技能行為
<!-- BEFORE: 每一輪都以 Scope 檔案集合派出 Standards 與 Correctness discovery，續輪仍能在整檔發現新事項。 -->

技能文件 SHALL 指示主線 orchestrator 依序執行：(1) 選定 change；(2) 守門自檢，任務未全數完成即停止；(3) 呼叫 review scope 取得 frozen patch——無工單時 phase=discovery，有工單時 phase=validation；needsInput 時 SHALL 等待使用者提供可信 base、hash-pinned hunk selection 或隔離 worktree，不得以 touched 整檔替代；(4) 讀取 change artifacts 作判準脈絡；(5) 依 phase 執行以下分流；(6) 並列呈現結果與 remediation triage；(7) 以 review add-round 寫入相同 phase、patchHash、Scope 與 findings。

Discovery SHALL 將同一 frozen patch 平行交給 Standards（repo 慣例文件＋smell baseline，repo 文件優先）與 Correctness（bug hunting）兩個 read-only sub-agent，各以 400 字內回報並以 CRITICAL／WARNING／SUGGESTION 分級。兩軸 SHALL 只以 change hunks 與判斷直接影響所需的呼叫端、測試為審查面；兩份報告 SHALL 原樣並列，不合併、不跨軸重排。Spec compliance SHALL NOT 在審查站裁決。

Validation SHALL 只把上輪未解 findings、accepted 清單、remediation patch 與必要脈絡交給對應 axes；sub-agent SHALL 逐筆判定原 finding 已解／未解，並只回報 remediation patch 直接引入的 regression。未解 finding SHALL 由主線以原文寫入新輪；已解 finding SHALL 從新輪 findings 移除；未修改區域的新 smell、SUGGESTION 或既存問題 SHALL NOT 加入。

artifacts 稀薄時 sub-agent SHALL 僅憑 code 與測試判斷，不臆造需求。locale SHALL 沿用既有「審查產出的語言綁定」契約；phase、patchHash、severity、axis prefix 與 path 保持英文 token。

#### Scenario: 任務未完成即停

- **WHEN** 對任務 3/5 的 change 執行 speclink-review
- **THEN** 技能停止並說明審查站要求任務全數完成，不呼叫 review scope、不派出 sub-agent、不寫工單

#### Scenario: 首輪只審 frozen change hunks

- **WHEN** touchedFiles 含一份 300 行檔案，而 resolved discovery patch 只含其中兩個 hunks
- **THEN** Standards 與 Correctness 都收到相同兩個 hunks及必要上下文，不把其餘未修改內容當 discovery 面

#### Scenario: 續輪只驗收上輪 findings 與 remediation patch

- **WHEN** Round 1 有兩筆未解 findings，修正 patch 只改其中一檔並新增一個呼叫端
- **THEN** validation 只判定兩筆原 finding 與該 patch 的直接 regression，不重新掃描整個 finding 檔案或 change

#### Scenario: 末輪零 findings 時重試蓋章而非重審

- **WHEN** 工單末輪 findings 為空但先前 stamp 因外部守門失敗而留下工單
- **THEN** 技能在守門恢復後直接重試 review stamp，不派出新的 discovery 或 validation

#### Scenario: legacy 工單缺 snapshot 時 fail closed

- **WHEN** 工單有 findings但 lastRound.patchHash 為 null，且 host 沒有可對應 snapshot
- **THEN** 技能說明無法精確重建 remediation delta，保留工單並等待使用者明示 discard 後重新 discovery，不得重審整檔

### Requirement: 審查後的迴圈與收尾
<!-- BEFORE: 每輪有 findings 都詢問相同三選項，沒有無進展終止條件。 -->

Discovery 呈現與 triage 後，技能 SHALL 沿既有三選項讓使用者選擇：修正後重審／接受現狀蓋章／先不蓋章。修正 SHALL 一律由主線依專案 TDD 慣例執行，sub-agent 不得修改檔案；修正後 SHALL 先通過「修復迴圈的驗證門」，再開始 validation。

每輪 validation 後，技能 SHALL 以未接受的必修集合 Bn 與上輪 Bn-1 比較：

- Bn 為空且沒有 accepted findings 時 SHALL 執行 review stamp，結果為 passed clean
- Bn 為空且仍有 accepted findings 時 SHALL 推薦使用者明示 review stamp --accept，結果為 passed with reservations
- Bn 非空且數量嚴格小於 Bn-1 時 SHALL 允許使用者再次選擇修正後驗收、接受現狀或先不蓋章
- Bn 數量大於或等於 Bn-1 時 SHALL 在記錄本輪後立即以 failed 結束自動迴圈，保留工單、不蓋章且不得自動再試

blocking set 的縮小只決定能否繼續自動修正，SHALL NOT 被描述為品質分數或通過。技能 SHALL NOT 設固定最大輪數；每次允許續跑都必須嚴格下降。互動工具不可用時 SHALL 以純文字詢問並等待回覆。

#### Scenario: 乾淨首輪自動蓋章

- **WHEN** discovery 的兩軸皆零 findings
- **THEN** 技能記錄零 findings 的 discovery round，執行 review stamp 並回報 passed clean

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

## ADDED Requirements

### Requirement: 續輪重大晚發問題的安全退出

Validation 偶然看見與 remediation patch 無關的新問題時，技能 SHALL NOT 將它加入目前 findings 或重新開啟 discovery。只有問題同時具有現實觸發路徑、重現方式／失敗測試／明確 invariant 破壞之一，且影響安全、資料損失或錯誤行為時，技能 SHALL 以 scope changed／failed 結束本站，保留原工單且不蓋章，並建議另開 discovery 或衍生 change。證據不足或不達門檻的事項 SHALL 僅列為後續提示，不阻斷目前 validation。

#### Scenario: 無關 smell 不加入續輪

- **WHEN** validation 期間注意到未修改鄰檔的 possible Duplicated Code
- **THEN** 該事項不寫入目前 round、不改變 blocking set，也不觸發新的 discovery

#### Scenario: 有證據的資料損失問題終止本站

- **WHEN** validation 期間發現與 remediation patch 無關但有失敗測試可重現的資料損失
- **THEN** 技能回報 scope changed／failed、保留工單且不蓋章，建議另開 discovery 或衍生 change

## ADDED Requirements

### Requirement: task done 寫入逐任務 evidence

task done 成功且本次有可歸屬的 touched files（未被先前任務認領的新髒檔）時，SHALL 寫入逐任務證據記錄：task stable ID、actor（來自 ExecutionContext 的顯示身分）、repo（binding key）、head commit、touched files、basis digests（spec、tasks、policy 三項）與 recordedAt（UTC）；無新髒檔時 SHALL 沿現行 touched 語意不新增任何記錄。記錄格式 SHALL 帶版本標記（v2）；讀取端 SHALL 相容無版本標記的既有格式（v1 檔案清單語意不變）；既有消費者（commit 檔案歸屬、archive trace）依賴的檔案清單語意 SHALL 保留。task undone SHALL NOT 寫入或修改任何證據記錄。

#### Scenario: 完成任務後證據齊全

- **WHEN** 於有 git 身分與髒檔的 workspace 執行 speclink task done tsk_ 某任務
- **THEN** 該 change 的 touched 記錄含該任務 entry：taskId 為該 tsk_ ID、actor 與 repo 非空、headCommit 為當前 HEAD、touchedFiles 含髒檔、basisDigests 三項齊備、recordedAt 為 UTC 時間戳

#### Scenario: 舊格式記錄可讀

- **WHEN** 對帶有既有 v1 touched 記錄的 change 執行 archive
- **THEN** trace 檔案清單自 v1 記錄聚合成功，行為與現行一致、無錯誤

### Requirement: VerifyBundle 固定驗證基準

Host SHALL 提供 VerifyBundle 產生：對指定 change 回報 change 名、任務 stable ID 清單、spec digest、tasks digest、policy digest 與產生時間；同一 workspace 狀態下重複產生 SHALL 得到相同的三項 basis digest。

#### Scenario: bundle 基準可重現

- **WHEN** 對同一 change 連續產生兩份 VerifyBundle
- **THEN** 兩份的 spec、tasks、policy digest 逐項相同；修改任一 delta spec 後再產生，spec digest 改變

### Requirement: evidence 的 stale 判定

Host SHALL 提供 evidence 對 VerifyBundle 的 stale 判定：任一 basis digest 不符時 SHALL 判 stale 並列出全部不符項，SHALL NOT 靜默接受混用基準的證據；全部相符時判有效。

#### Scenario: 基準改變即 stale

- **WHEN** 以某 bundle 基準記錄的 evidence，在 tasks.md 被修改後對新 bundle 執行 stale 判定
- **THEN** 判定為 stale 且列出 tasks digest 不符；spec 與 policy 未變時不列入不符項

### Requirement: archive trace 由 evidence 建立

archive 注入的 trace 檔案清單 SHALL 由該 change 的 evidence 記錄聚合建立（v2 逐任務 entries 或 v1 檔案清單），注入的輸出格式 SHALL 與現行逐位元一致。Host SHALL 提供 archive gate 的 evidence 檢查函式（任務全數勾選、evidence 存在且未 stale 則通過，否則回帶原因的拒絕）；本地 archive SHALL NOT 強制該檢查（強制點屬遠端 Host）。

#### Scenario: trace 輸出格式凍結

- **WHEN** 對含 v2 evidence 的 change 執行 speclink archive
- **THEN** 封存後正典規格的 trace 區塊格式與以現行 touched 記錄產生者逐位元同構（相同檔案清單時內容一致）

#### Scenario: gate 檢查函式回報原因

- **WHEN** 對 evidence 已 stale 的 change 呼叫 archive gate 檢查函式
- **THEN** 回拒絕並指出 stale 的 basis 項；本地 speclink archive 不受該函式阻擋、行為不變

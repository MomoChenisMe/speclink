## MODIFIED Requirements

### Requirement: task done 寫入逐任務 evidence

task done 成功且本次有可歸屬的 touched files（未被先前任務認領的新髒檔）時，SHALL 寫入逐任務證據記錄：task stable ID、actor（來自 ExecutionContext 的顯示身分）、repo（binding key）、head commit、touched files 與 recordedAt（UTC）；無新髒檔時 SHALL 沿現行 touched 語意不新增任何記錄。記錄 SHALL 寫入 change 目錄的 `.evidence.json`，隨 change 的提交、封存與廢棄同生命週期；讀取端 SHALL 於 change 目錄記錄缺席時回退舊路徑 `.speclink/touched/<change>.json`（僅供讀取，永不作為寫入目標），寫入 SHALL 一律落 change 目錄。記錄寫入新位置後、以及 change 封存或廢棄時，舊路徑檔案 SHALL 被移除——其內容已由回退讀取帶入新位置，殘留檔會被日後重用同一名稱的 change 誤讀為自己的記錄。記錄格式 SHALL 帶版本標記（v2）；讀取端 SHALL 相容無版本標記的既有格式（v1 檔案清單語意不變），並 SHALL 忽略既有記錄中的未知欄位（含先前版本寫入的 basisDigests）；commit 檔案歸屬依賴的檔案清單語意 SHALL 保留。entry SHALL NOT 含 basis digests——帳僅存有讀者的歷史事實。task undone SHALL NOT 寫入或修改任何證據記錄。

#### Scenario: 完成任務後證據齊全

- **WHEN** 於有 git 身分與髒檔的 workspace 執行 speclink task done tsk_ 某任務
- **THEN** 該 change 目錄的 `.evidence.json` 含該任務 entry：taskId 為該 tsk_ ID、actor 與 repo 非空、headCommit 為當前 HEAD、touchedFiles 含髒檔、recordedAt 為 UTC 時間戳；entry 無 basisDigests 欄位

#### Scenario: 舊格式記錄可讀

- **WHEN** change 目錄無 `.evidence.json` 而舊路徑存在 v1 或含 basisDigests 的 v2 記錄，執行任何讀取端（drift、commit 檔案歸屬）
- **THEN** 讀取回退舊路徑成功、v1 檔案清單語意不變、v2 的未知欄位被忽略且 all_files 不變；下一次 task done 寫入後新位置成為讀取來源且舊路徑檔案不再存在

#### Scenario: 證據隨 change 生命週期移動

- **WHEN** 對含 `.evidence.json` 的 change 執行封存或 discard
- **THEN** 封存後記錄位於封存 change 目錄內；discard 後記錄隨目錄一併消失；兩者皆不留下舊路徑的孤兒檔

## REMOVED Requirements

### Requirement: archive trace 由 evidence 建立

**Reason**: trace 不再承載檔案清單，「由 evidence 記錄聚合 trace 檔案清單」的規範對象消失；Host gate 檢查函式隨帳瘦身一併退場（見同批 REMOVED）。
**Migration**: 由本 delta 新增的「archive trace 注入與零證據提示」承接 trace 形狀；檔案歸屬的耐久保存改由 evidence 記錄隨 change 目錄移動承接。

### Requirement: VerifyBundle 固定驗證基準

**Reason**: 討論 evidence-gate-false-blocks 裁決帳瘦身——entry 不再記 basis digests，bundle 的比對對象消失；全 repo 亦無生產呼叫端。遠端 Phase 2 應由 server 自記自判，不以本機自報指紋為地基。
**Migration**: 模組刪除、無承接。drift 的現場基準計算（current_basis_digests）非本需求範疇，保留不動。

### Requirement: evidence 的 stale 判定

**Reason**: 同上——比對的兩端（entry 的 basis digests 與 VerifyBundle）皆退場，判定失去對象。實證（討論探針）顯示其在本機場景誤擋正常流程且無有效補救路徑。
**Migration**: 模組刪除、無承接。

## ADDED Requirements

### Requirement: archive trace 注入與零證據提示

archive 於 ADDED／MODIFIED 物化正典需求時 SHALL 一律注入 trace 區塊，內容 SHALL 僅含 source（change 名）與 updated（封存日期）兩欄；SHALL NOT 含檔案清單，SHALL NOT 掃描工作樹髒檔產生任何 trace 內容。archive SHALL NOT 因 evidence 缺席或內容拒絕封存。本機 fs 模式下，change 無任何 v2 evidence entry 時 CLI SHALL 於 stderr 印恰一行提示（點名 change 名、說明無任務證據記錄），exit code 與封存結果 SHALL 不受影響；有任一 entry 時 SHALL NOT 印出提示。引擎 SHALL 以結構化事實（ArchiveOutcome 的 evidence_recorded）回報有無證據，呈現歸呼叫端。遠端 store 模式不在本需求範圍：證據的記錄與提示應由 server 端自記自判（Phase 2 另行設計），wire 契約 SHALL NOT 為此攜帶欄位。

#### Scenario: trace 兩欄一律注入

- **WHEN** 對含 v2 evidence 的 change 執行 speclink archive
- **THEN** 封存後正典的 trace 區塊僅含 source 與 updated 兩行，無 code 清單；evidence 記錄本身不受注入影響、隨目錄移入封存區

#### Scenario: 零證據照常封存並提示

- **WHEN** 對無任何 v2 entry 的 change 執行 speclink archive
- **THEN** 封存成功、exit code 0，stderr 恰一行提示點名該 change 無任務證據記錄；正典 trace 照樣注入 source 與 updated 兩欄

#### Scenario: 有證據時一字不印

- **WHEN** 對含任一 v2 entry 的 change 執行 speclink archive
- **THEN** 封存成功且 stderr 無任何 evidence 相關提示

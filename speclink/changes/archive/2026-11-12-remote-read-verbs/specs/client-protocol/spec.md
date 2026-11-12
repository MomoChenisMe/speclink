## ADDED Requirements

### Requirement: 溯源回應 payload

Client Protocol SHALL 定義溯源端點的回應型別：欄位與 `speclink trace --json` 的輸出一對一，名稱為 camelCase，型別相同；它 SHALL 在 protocol 的單一定義處宣告。typed client SHALL 提供取得溯源回應的方法，並 SHALL 能把回應轉回引擎的溯源報告，使 CLI 以與 fs 模式相同的輸出函式呈現。evidence 端點的回應 SHALL 增列選填的 `archivedAs`（字串，缺席時不序列化）與選填的 `change`（字串，這份記錄所屬變更自己的名稱；以引擎命令回答的 server 一律會帶）。用戶端把 evidence 回應轉回 `evidence show` 的結果時 SHALL 以回應的 `change` 為變更名稱；回應沒有 `change` 時 SHALL 回報錯誤（訊息說明 server 不支援 `evidence show`），SHALL NOT 把它當成「沒有記錄」——只讀進行中變更的舊版 server 對已封存或不存在的名稱都回空集合。既有直接讀取 evidence 端點的用途（remote 蓋章）不受這個檢查影響。typed client SHALL 提供帶嚴格重疊旗標取得 plan 回應的方法，回應沿用既有的 plan 回應 payload，並增列選填的 `strictOverlap`（布林）：只有以嚴格重疊計算出的計畫才為 `true`，其餘缺席且不序列化。這個欄位是線上的標記，SHALL NOT 出現在 `speclink plan --json` 的輸出。typed client 要求嚴格重疊而回應沒有這個標記時 SHALL 回報錯誤（訊息說明 server 不支援 `plan --strict-overlap`），SHALL NOT 把一般的計畫當成嚴格重疊的結果交回——不認得這個查詢參數的舊版 server 會忽略它、照樣回一般的計畫。

#### Scenario: 溯源回應轉回報告後輸出相同

- **WHEN** 把引擎對某 capability 的溯源報告轉成溯源回應、序列化、解析、再轉回報告，並以輸出函式產生 --json
- **THEN** 得到的 JSON 與直接由原報告產生的相同

#### Scenario: archivedAs 缺席不序列化

- **WHEN** 序列化一份進行中變更的 evidence 回應
- **THEN** JSON 沒有 archivedAs 鍵；舊版用戶端解析含 archivedAs 的回應時不報錯

#### Scenario: evidence 回應沒有 change

- **WHEN** 於 remote 模式執行 speclink evidence show demo，server 的回應是 `{"entries":[]}`（沒有 change 鍵）
- **THEN** exit code 非零，stderr 說明這個 server 不支援 evidence show，stdout 為空

#### Scenario: 嚴格重疊的回應帶標記

- **WHEN** 以 typed client 的嚴格重疊方法向支援它的 server 取得 plan
- **THEN** 請求是帶 strictOverlap=true 的 GET /plan，回應的 strictOverlap 為 true；一般的 plan 回應沒有 strictOverlap 鍵

#### Scenario: 舊版 server 的回應沒有標記

- **WHEN** 以嚴格重疊方法向一個不認得 strictOverlap 的 server 取得 plan，server 回了一般的計畫（沒有 strictOverlap 鍵）
- **THEN** typed client 回報錯誤，訊息說明這個 server 不支援 plan --strict-overlap；CLI 於 remote 模式以非零 exit code 結束、stdout 為空

### Requirement: 變更清單的最後修改時間欄位

變更清單回應的每個項目 SHALL 增列選填的 `modifiedAt`（字串，RFC 3339 的 UTC 時間）：該變更底下的文件最後一次被修改的時間，取自儲存回報的最後修改時間。儲存答不出時 SHALL 缺席且不序列化，SHALL NOT 以 0 或空字串代替。既有的 `created` 欄位不變。CLI 於 remote 模式 SHALL 把 `created` 與 `modifiedAt` 帶進 `speclink list --json` 的輸出，欄位名與 fs 模式相同。舊版用戶端遇到這個欄位 SHALL 照常運作。

#### Scenario: 清單項帶出最後修改時間

- **WHEN** 儲存回報變更 demo 底下文件的最後修改時間是 2026-11-12T03:00:00Z，請求變更清單
- **THEN** demo 的項目含 modifiedAt `2026-11-12T03:00:00Z`

#### Scenario: 儲存答不出時缺席

- **WHEN** 儲存對每份文件回報的最後修改時間是 0，請求變更清單
- **THEN** 每個項目都沒有 modifiedAt 鍵

#### Scenario: 舊版用戶端不受影響

- **WHEN** 以不認得 modifiedAt 的型別解析含該欄位的清單回應
- **THEN** 解析成功，其餘欄位的值不變

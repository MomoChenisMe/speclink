## ADDED Requirements

### Requirement: 自由欄位的線上欄位與請求

變更清單的項目、單筆變更的回應、已封存清單的項目與討論資訊 SHALL 各增列選填的 `fields`（物件，字串對字串；空的時候不序列化）。建立變更、建立討論與轉出變更的請求 SHALL 各增列選填的 `fields`。Client Protocol SHALL 定義兩個寫入請求：對專案底下 changes/<name>/fields 與 discussions/<slug>/fields 的 PUT，payload 為 `set`（物件，選填）與 `unset`（字串陣列，選填），兩者至少一個不是空的；成功回應為 `{ "fields": object }`（寫入後的全部欄位）並帶新的範圍版本。握手的 capabilities SHALL 增列選填的布林 `fields`，Server 支援自由欄位時為 true。不認得這些欄位的舊版用戶端 SHALL 照常解析回應。用戶端在建立請求帶了 `fields` 而握手沒有宣告支援時 SHALL 警告使用者欄位沒有被保存，建立本身 SHALL 視為成功。這些型別 SHALL 在 protocol 的單一定義處宣告。

#### Scenario: 回應帶出欄位

- **WHEN** Server 上的 demo 有兩個自由欄位，請求變更清單與 demo 的單筆回應
- **THEN** 兩個回應中 demo 的 fields 都是這兩組

#### Scenario: 空的不序列化

- **WHEN** 序列化一個沒有自由欄位的清單項
- **THEN** JSON 沒有 fields 鍵

#### Scenario: 建立請求帶欄位

- **WHEN** 送出含 `"fields":{"ticket":"PROJ-1"}` 的建立變更請求
- **THEN** 建立成功；之後的單筆回應含該欄位

#### Scenario: 寫入請求

- **WHEN** 對 changes/demo/fields 送出 `{"set":{"a":"1"},"unset":["b"]}`
- **THEN** 回應的 fields 含 a、不含 b，並帶新的範圍版本

#### Scenario: set 與 unset 都是空的

- **WHEN** 送出 `{}`
- **THEN** 回應的 reason 為 invalid_argument

#### Scenario: 舊版用戶端解析新回應

- **WHEN** 以不認得 fields 的型別解析含 fields 的清單回應
- **THEN** 解析成功，其餘欄位的值不變

#### Scenario: 舊 Server 不保存欄位時警告

- **WHEN** 握手沒有宣告 fields 的 Server 上，執行 speclink new change demo --field ticket=PROJ-1
- **THEN** demo 建立成功，exit code 為 0，stderr 有一行說明這台 Server 沒有保存欄位

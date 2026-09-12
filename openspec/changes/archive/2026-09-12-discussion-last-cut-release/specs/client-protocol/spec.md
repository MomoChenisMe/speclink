## ADDED Requirements

### Requirement: 轉出請求攜帶最後一刀旗標

討論轉出端點的請求型別（promote 請求）、建立變更的請求型別，以及 link 與 seal 共用的綁定請求型別 SHALL 各增選填 last 欄位（camelCase 布林、serde default、缺席即 false、值為 false 時不序列化）——true 表示本次轉出的是結論規劃的最後一刀，引擎於累加 promoted_to 的同一次寫入解除記錄的 hold。回應型別 SHALL 逐位元不變（轉出不報告 hold）。typed client 的 promote、建立變更與 seal 方法 SHALL 各接受 last 布林參數並填入請求；link 方法 SHALL NOT 填此欄位（link 從不改動討論記錄）。舊 server 收到未知鍵時忽略，效果等同未帶 last；舊 client 不送即 false。

#### Scenario: 三個請求的 last 缺席即 false 且 false 不出鍵

- **WHEN** 以不含 last 的 JSON 分別反序列化 promote 請求、建立變更請求與綁定請求；再各序列化一筆 last 為 false 與一筆 last 為 true 的請求
- **THEN** 三個反序列化結果的 last 皆為 false；序列化後 last 為 false 的 JSON 無 last 鍵、last 為 true 的 JSON 含 last: true，其餘欄位與本變更前同形

#### Scenario: typed client 填入 last

- **WHEN** 以 last 為 true 呼叫 typed client 的 promote 方法、建立變更方法（帶 from_discussion）與 seal 方法
- **THEN** 三個請求 body 皆含 last: true；以 last 為 false 呼叫時 body 無 last 鍵、與本變更前逐位元一致；link 方法的 body 與本變更前逐位元一致

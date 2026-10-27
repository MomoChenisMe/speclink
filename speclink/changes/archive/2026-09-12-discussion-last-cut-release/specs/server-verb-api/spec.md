## ADDED Requirements

### Requirement: 轉出端點轉傳最後一刀旗標

POST /discussions/{slug}/promote、POST /changes（帶 fromDiscussion）與 POST /discussions/{slug}/seal SHALL 將請求的 last 欄位直通引擎對應的轉出命令（缺席即 false，行為與本變更前完全相同）；POST /discussions/{slug}/link SHALL 忽略 last、討論記錄逐位元不變。判準與旗標解除 SHALL 由引擎執行，server 路由 SHALL NOT 重複實作、SHALL NOT 重讀記錄。last 為 true 時對帶 hold 的討論轉出後，GET /discussions 該筆的 hold SHALL 為 false，其後最後一個轉出變更以封存端點封存時討論 SHALL 移入封存清單。既有回應欄位、狀態碼與錯誤語意 SHALL 維持不變；三端點皆為 unit of work，成功寫入時 scope revision 前進、事件照引擎 outcome 發布。

#### Scenario: promote 端點帶 last 解除 hold

- **WHEN** 對 frontmatter 帶 hold: true 的討論以 last: true 呼叫 POST /discussions/{slug}/promote；再對另一份帶 hold 的討論以不含 last 的請求呼叫同端點
- **THEN** 兩者 HTTP 200 且回應與本變更前同形；GET /discussions 中前者 hold 為 false、後者 hold 為 true；兩者 promotedTo 皆累加新變更名；scope revision 前進

#### Scenario: seal 端點帶 last 解除 hold 而 link 忽略

- **WHEN** 對帶 hold 的討論先以 last: true 呼叫 POST /discussions/{slug}/link，再以 last: true 呼叫 POST /discussions/{slug}/seal
- **THEN** link 後討論記錄逐位元不變（GET /discussions 該筆 hold 仍為 true）；seal 後該筆 hold 為 false 且標記 promoted；兩者 HTTP 200

#### Scenario: 帶 last 轉出後最後一個封存收走討論

- **WHEN** 對帶 hold 且結論已寫的討論以 last: true 呼叫 promote 端點建立唯一的轉出變更，再以封存端點封存該變更
- **THEN** 封存回應的隨行封存清單列該討論；GET /discussions 不再列它、封存清單列它

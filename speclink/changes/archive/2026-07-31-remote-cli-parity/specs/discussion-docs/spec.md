## ADDED Requirements

### Requirement: 討論動詞於 remote 模式與本機同語意

remote 模式下 speclink discuss new 的 --slug 覆寫、speclink discuss discard（含 --force）、speclink discuss link 與 speclink discuss seal SHALL 可用，語意與 fs 模式一致。slug 的 ASCII kebab-case 驗證與 discard 的有輪 guard SHALL 由引擎於 server 端執行（單一事實來源），CLI 與 server 路由 SHALL NOT 重複實作驗證邏輯；引擎拒絕 SHALL 映射為語義化錯誤訊息與非零 exit code。未升級的舊 server 對新動詞回應找不到端點時，CLI SHALL 呈現語義化錯誤訊息並以非零 exit code 結束，SHALL NOT panic。

#### Scenario: remote 帶 --slug 以中文主題建立討論

- **WHEN** 於 remote 模式執行 speclink discuss new 並給定中文主題「看板搜尋列」與 --slug board-search-bar
- **THEN** server 端建立 slug 為 board-search-bar 的討論記錄，topic 保留中文原文；stdout 顯示建立訊息含 board-search-bar，--json 的 slug 欄位為 board-search-bar

#### Scenario: remote 非法 --slug 被拒且 server 不落檔

- **WHEN** 於 remote 模式執行 speclink discuss new 並帶 --slug「中文slug」
- **THEN** exit code 非 0，stderr 說明 slug 格式要求，server 端未建立任何討論記錄

#### Scenario: remote discard 的輪數 guard 與本機一致

- **WHEN** 於 remote 模式對 server 上一筆 0 輪討論執行 speclink discuss discard，再對一筆已有 2 輪的討論執行同指令（無 --force）
- **THEN** 0 輪記錄被刪除且 exit code 為 0；2 輪記錄保留、exit code 非 0 且 stderr 提示需 --force；對 2 輪記錄帶 --force 重試則刪除成功

#### Scenario: remote link 鑄鏈可經 show 觀察

- **WHEN** 於 remote 模式執行 speclink discuss link 某已結論討論與某既有 change，隨後執行 speclink show 該 change --json
- **THEN** link 指令 exit code 為 0，show 的 payload 中 from_discussion 鏈含該討論 slug，與 fs 模式同欄位形狀

#### Scenario: remote seal 標記已轉出

- **WHEN** 於 remote 模式對已 link 的討論執行 speclink discuss seal
- **THEN** exit code 為 0，該討論於 server 端標記為已轉出（promoted），speclink discuss list --json 反映該狀態

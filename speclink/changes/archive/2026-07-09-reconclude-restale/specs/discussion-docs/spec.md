## ADDED Requirements

### Requirement: 討論重新結論標記已反映變更待重新反映

speclink discuss conclude 寫入結論後 SHALL 檢查討論記錄 frontmatter 的 promoted_to：非空時 SHALL 對其中每個變更名判存活——僅 openspec/changes/<name>/ 存在（active）者納入蓋章，僅存在於 openspec/changes/archive/ 者 SHALL 跳過。對每個納入的 active 變更，其 meta 檔（openspec/changes/<name>/.openspec.yaml）的 restale_from 欄位 SHALL 以逗號累加本討論 slug——尚無該欄位時增寫為單值、已有其他值時於尾端累加、已含本 slug 時 SHALL 冪等不改該檔；既有 meta 欄位 SHALL 逐字保留。判存活的鍵 SHALL 為 promoted_to 非空（曾被反映），SHALL NOT 綁 status 欄位值。promoted_to 為空、或其項全為已歸檔變更時，SHALL NOT 寫入任何變更 meta。討論記錄的 Context、Rounds 區 SHALL 逐位元不變（僅 Conclusion 區依既有 conclude 行為改寫）。成功時 stdout SHALL 於既有結論訊息後，另報告被標記待重新反映的 active 變更清單（無則不報告）；帶 --json 時 payload SHALL 含被標記變更名的陣列。本行為為 Speclink 自有延伸，不在 Spectra 對照範圍；未觸發蓋章時（promoted_to 空）既有 conclude 的人眼與 --json 輸出 SHALL 逐位元不變。

#### Scenario: 重新結論已反映討論蓋章其 active 變更

- **WHEN** 對 promoted_to 含一個 active 變更名的討論執行 speclink discuss conclude
- **THEN** 該變更 meta 的 restale_from 累加本討論 slug；討論記錄除 Conclusion 區外逐位元不變；stdout 報告該變更被標記待重新反映

#### Scenario: 蓋章跳過已歸檔變更

- **WHEN** 對 promoted_to 同時含一個 active 變更與一個已歸檔變更的討論執行 speclink discuss conclude
- **THEN** 僅 active 變更 meta 的 restale_from 累加本 slug；已歸檔變更目錄下任何檔案逐位元不變

#### Scenario: promoted_to 空的結論不蓋章

- **WHEN** 對 promoted_to 為空（尚未 seal）的討論執行 speclink discuss conclude
- **THEN** 不寫入任何變更 meta；既有 conclude 的人眼與 --json 輸出逐位元不變

#### Scenario: 重複重新結論為冪等

- **WHEN** 對已因先前結論而使某 active 變更 restale_from 含本 slug 的討論再次執行 speclink discuss conclude
- **THEN** 該變更 meta 逐位元不變（restale_from 已含本 slug 不重複累加）

##### Example: 蓋章後的變更 meta

- **GIVEN** 討論 alpha-search 的 promoted_to 含 active 變更 cut-a，cut-a 的 meta 無 restale_from
- **WHEN** 執行 speclink discuss conclude alpha-search 帶新結論
- **THEN** cut-a 的 meta 增寫 restale_from: alpha-search

---
### Requirement: seal 清除變更的 restale 旗標

speclink discuss seal 通過既有守衛並標記討論 promoted 後 SHALL 額外自目標變更 meta 檔的 restale_from 逗號清單移除本討論 slug：清單移除後仍有值時保留其餘值、變空時 SHALL 移除 restale_from 行；本 slug 不在清單（或無該欄位）時 SHALL 冪等不改該檔。清除 SHALL 僅動 restale_from 欄位，變更 meta 其餘欄位與討論記錄 SHALL 逐位元不變。此清除使 re-conclude → re-ingest → seal 成閉環：seal 作為誠實的「內容落地」動作，清掉對應該討論的過期標記。既有 seal 的守衛、promoted 標記、輸出與冪等行為 SHALL 不變。

#### Scenario: seal 清除對應 slug 的 restale 旗標

- **WHEN** 對 restale_from 含本討論 slug 的目標變更執行 speclink discuss seal
- **THEN** 該變更 meta 的 restale_from 移除本 slug；其餘 restale_from 值與所有其他欄位逐位元不變；討論如常標記 promoted

#### Scenario: restale_from 變空移除整行

- **WHEN** seal 移除的 slug 是目標變更 restale_from 的唯一值
- **THEN** 該變更 meta 的 restale_from 行消失；其他欄位逐位元不變

#### Scenario: 無對應旗標時清除為冪等

- **WHEN** 對 restale_from 不含本 slug（或無該欄位）的目標變更執行 speclink discuss seal
- **THEN** 變更 meta 逐位元不變（除既有 seal 的 from_discussion／promoted 相關行為外）

##### Example: 多討論過期時 per-slug 清除

- **GIVEN** 變更 cut-a 的 meta 含 restale_from: alpha-search, beta-cache
- **WHEN** 執行 speclink discuss seal alpha-search cut-a
- **THEN** cut-a 的 restale_from 變為 beta-cache（beta-cache 仍待其各自 re-seal）

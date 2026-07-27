## MODIFIED Requirements

### Requirement: 討論以 link 動詞併入既有變更

speclink discuss link SHALL 接受兩個位置參數（討論 slug 與既有變更名），鑄造變更側連結：變更 meta 檔（openspec/changes/<change>/.openspec.yaml）的 from_discussion 欄位 SHALL 為逗號分隔清單——尚無該欄位時增寫為單值；已指向其他討論時 SHALL 於既有值尾端以逗號累加本 slug、既有值保留不覆蓋；清單已含本 slug 時 SHALL 為冪等成功不改檔。討論記錄（openspec/discussions/<slug>.md）SHALL 逐位元不變——link SHALL NOT 標記 status: promoted、SHALL NOT 寫 promoted_to；「已轉出」的標記職責移交 speclink discuss seal。open 與 concluded 狀態的討論皆 SHALL 可併入；已封存討論 SHALL NOT 可併入。指令不吃 stdin，旗標僅 --json。成功時 exit code 0，stdout 輸出單行成功訊息（含討論 slug 與變更名；--no-color 下無 ANSI 色彩）；帶 --json 時輸出含 slug 與 change 欄位（camelCase）的 payload。守衛失敗（討論不存在、討論已封存、變更不存在）時 SHALL 以非零 exit code 結束、stderr 說明原因，且兩側檔案 SHALL 逐位元不變。同一組合重跑 SHALL 為冪等成功：exit code 0 且兩側檔案內容不變。變更封存時，其 from_discussion 清單中的每份討論 SHALL 各自檢查：無其他在途變更的 from_discussion 清單引用該討論時，既有自動封存機制 SHALL 將該記錄移入 openspec/discussions/archive/；僅單一來源討論之變更，其封存的人眼輸出 SHALL 與變更前逐位元一致。本指令為 Speclink 自有延伸；既有指令的人眼與 --json 輸出 SHALL 逐位元不變。

#### Scenario: 成功併入既有變更

- **WHEN** 執行 speclink discuss link 並給定一份未封存討論的 slug 與一個尚無 from_discussion 的既有變更名
- **THEN** exit code 0；stdout 單行成功訊息含該 slug 與變更名；openspec/changes/<change>/.openspec.yaml 增寫 from_discussion: <slug>；討論記錄逐位元不變（status 與 promoted_to 皆不變）；帶 --json 時 payload 的 slug 與 change 欄位分別為兩參數值

#### Scenario: link 不改討論記錄

- **WHEN** 對任一狀態（open、concluded、或已由 seal 標記 promoted）的討論執行 speclink discuss link 指向既有變更
- **THEN** exit code 0；變更 meta 的 from_discussion 累加該 slug；討論記錄 frontmatter 與內文逐位元不變

#### Scenario: 出身自討論的變更再併入新討論

- **WHEN** 對 meta 已有 from_discussion 指向其他討論的變更執行 speclink discuss link 給定另一份討論的 slug
- **THEN** exit code 0；變更 meta 的 from_discussion 於既有值尾端累加本 slug、既有值保留；本討論與先前連結的討論記錄皆逐位元不變

##### Example: 累加後的 meta 欄位

- **GIVEN** 變更 cut-a 的 meta 含 from_discussion: alpha-search
- **WHEN** 執行 speclink discuss link beta-cache cut-a
- **THEN** cut-a 的 meta 欄位為 from_discussion: alpha-search, beta-cache；alpha-search 的記錄逐位元不變

#### Scenario: 同一組合重跑為冪等

- **WHEN** 對已互相連結的同一組討論與變更再次執行 speclink discuss link
- **THEN** exit code 0；變更 meta 檔與討論記錄內容逐位元不變

#### Scenario: 守衛拒絕且不落檔

- **WHEN** 執行 speclink discuss link 且命中任一守衛：討論不存在、討論已封存、變更不存在
- **THEN** 指令以非零 exit code 結束，stderr 說明原因，openspec/changes/ 與 openspec/discussions/ 下任何檔案逐位元不變

#### Scenario: 併入後隨變更自動封存

- **WHEN** 已 link 的變更執行封存，且無其他在途變更引用同一討論
- **THEN** 討論記錄自動移入 openspec/discussions/archive/，與 promote 型討論的既有封存行為一致

#### Scenario: 多來源討論的變更封存逐一共行

- **WHEN** 封存 from_discussion 清單含兩份討論的變更
- **THEN** 清單中每份討論各自檢查存活引用：無其他在途變更引用者移入 openspec/discussions/archive/、人眼輸出逐討論各一行共行訊息；仍被引用者維持在途不動

### Requirement: 內容落地以 seal 動詞標記已轉出

speclink discuss seal SHALL 接受兩個位置參數（討論 slug 與變更名）。前置守衛 SHALL 全數通過方可寫入：討論 SHALL 存在且未封存、變更 SHALL 存在、且變更 meta 的 from_discussion 清單 SHALL 含該 slug（鏈須先由 link／promote／new change 鑄妥）——任一不滿足時 SHALL 以非零 exit code 結束、stderr 說明原因，且兩側檔案逐位元不變。守衛通過時：討論記錄 frontmatter 的 status SHALL 標記 promoted（由 open 或 concluded 轉入），promoted_to SHALL 以逗號累加該變更名、既有值保留不覆蓋。指令不吃 stdin，旗標僅 --json 與 --no-color。成功時 exit code 0、stdout 單行成功訊息（--no-color 下無 ANSI 色彩）；帶 --json 時輸出含 slug 與 change 欄位（camelCase）的 payload。同一組合重跑 SHALL 為冪等成功：promoted_to 已含該變更名時不改檔、exit code 0。本指令為 Speclink 自有延伸。

#### Scenario: 成功封印標記已轉出

- **WHEN** 對一份 concluded 討論執行 speclink discuss seal，且目標變更 meta 的 from_discussion 已含該 slug
- **THEN** exit code 0；討論 frontmatter 變為 status: promoted 且 promoted_to 含該變更名；stdout 單行成功訊息；帶 --json 時 payload 的 slug 與 change 欄位分別為兩參數值

#### Scenario: 鏈未鑄妥守衛拒絕

- **WHEN** 執行 speclink discuss seal 但目標變更 meta 的 from_discussion 不含該 slug
- **THEN** 指令以非零 exit code 結束、stderr 說明鏈未存在；討論記錄與變更 meta 皆逐位元不變

#### Scenario: 重跑封印為冪等

- **WHEN** 對 promoted_to 已含該變更名的討論再次執行 speclink discuss seal
- **THEN** exit code 0；討論記錄逐位元不變

##### Example: seal 守衛一覽

| 情境 | 結果 |
| ---- | ---- |
| slug 無對應討論記錄 | 拒絕：討論不存在 |
| slug 僅存在於 discussions/archive/ | 拒絕：討論已封存 |
| 變更名無對應目錄 | 拒絕：變更不存在 |
| 變更 meta 的 from_discussion 未含該 slug | 拒絕：鏈未鑄妥 |
| promoted_to 已含該變更名 | 冪等成功，不改檔 |

### Requirement: 討論重新結論標記已反映變更待重新反映

speclink discuss conclude 寫入結論後 SHALL 檢查討論記錄 frontmatter 的 promoted_to：非空時 SHALL 對其中每個變更名判存活——僅 openspec/changes/<name>/ 存在（active）者納入蓋章，僅存在於 openspec/changes/archive/ 者 SHALL 跳過。對每個納入的 active 變更，其 meta 檔（openspec/changes/<name>/.openspec.yaml）的 restale_from 欄位 SHALL 以逗號累加本討論 slug——尚無該欄位時增寫為單值、已有其他值時於尾端累加、已含本 slug 時 SHALL 冪等不改該檔；既有 meta 欄位 SHALL 逐字保留。判存活的鍵 SHALL 為 promoted_to 非空（曾被反映），SHALL NOT 綁 status 欄位值。promoted_to 為空、或其項全為已歸檔變更時，SHALL NOT 寫入任何變更 meta。討論記錄的 Context、Rounds 區 SHALL 逐位元不變（僅 Conclusion 區依既有 conclude 行為改寫）。成功時 stdout SHALL 於既有結論訊息後，另報告被標記待重新反映的 active 變更清單（無則不報告）；帶 --json 時 payload SHALL 含被標記變更名的陣列。本行為為 Speclink 自有延伸；未觸發蓋章時（promoted_to 空）既有 conclude 的人眼與 --json 輸出 SHALL 逐位元不變。

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

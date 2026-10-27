## MODIFIED Requirements

### Requirement: 桌面 app 提供動詞操作面

桌面 app SHALL 讓使用者對選定 change 執行 status、validate、analyze、archive，並對專案執行 list、show，全部經內嵌 core 執行。動詞的可觀察結果（成功資料、失敗訊息與失敗語意）SHALL 與對應 CLI 指令一致；失敗時 app SHALL 於 UI 呈現 core 的錯誤訊息，SHALL NOT 靜默吞掉失敗。validate 與 analyze 的結果 SHALL 呈現於該 change 的詳情抽屜內，而非僅視窗頂列狀態列：validate SHALL 於動作列近處以通過或失敗呈現（失敗附首則錯誤訊息）；analyze SHALL 以 Coverage、Consistency、Ambiguity、Gaps 四維度面板呈現，各維度顯示發現數與逐條發現項（嚴重度與訊息對應 speclink analyze 的 --json 輸出）。視窗頂列狀態列 SHALL 保留供看板全域操作（刪除、封存、拖排失敗）之結果訊息。

#### Scenario: 於抽屜內執行 validate 呈現結果

- **WHEN** 使用者於某 change 的詳情抽屜觸發 validate
- **THEN** 抽屜內於動作列近處呈現與 speclink validate 一致的通過或失敗結果，失敗時顯示其錯誤訊息

#### Scenario: 於抽屜內執行 analyze 呈現四維度發現項

- **WHEN** 使用者於某 change 的詳情抽屜觸發 analyze
- **THEN** 抽屜內以 Coverage、Consistency、Ambiguity、Gaps 四維度呈現各維度發現數與逐條發現項，其嚴重度與訊息對應 speclink analyze 的 --json 輸出

#### Scenario: archive 前置未滿足時失敗顯示

- **WHEN** 使用者對尚未滿足歸檔前置的 change 觸發 archive
- **THEN** app 呈現 core 回報的失敗訊息，不將該 change 標為已歸檔

### Requirement: 討論抽屜檢視與轉出變更

點擊討論卡或細列 SHALL 開啟討論抽屜。抽屜標題下方 SHALL 呈現生命週期階梯「討論中 → 已結論 → 轉出變更」且現站可辨識。分頁 SHALL 依序為：結論、討論過程 N、背景、衍生變更——前三者呈現記錄文件對應區段（區段缺失或格式非預期時 SHALL 整篇以單一檢視退回而非報錯）；記錄切分成功且結論區段非空時 SHALL 預設開啟「結論」分頁，結論為空時預設「背景」。衍生變更分頁 SHALL 列出各子變更現況與跳轉，且 SHALL 為唯讀——SHALL NOT 提供「轉為變更」或「再轉出一個變更」動作。concluded 卡的封存動詞 SHALL 經確認後將討論移入封存。GUI SHALL NOT 提供 conclude、add-round、new、discard、轉為變更（promote）——討論的推進、結論撰寫與轉出變更屬 agent 與 CLI。來自討論的變更卡 SHALL 帶討論徽章，其詳情抽屜 SHALL 顯示來源討論與同源變更清單並可互跳。

#### Scenario: 有結論的討論預設開啟結論分頁

- **WHEN** 使用者開啟一筆已結論（結論區段非空）討論的抽屜
- **THEN** 抽屜顯示分頁 結論／討論過程 N／背景／衍生變更，且預設呈現結論內容；階梯顯示「已結論」為現站

#### Scenario: 衍生變更分頁唯讀且無轉出動作

- **WHEN** 使用者開啟一筆已結論或已轉出討論的抽屜衍生變更分頁
- **THEN** 分頁列出各子變更現況與跳轉按鈕，但不呈現「轉為變更」或「再轉出一個變更」按鈕

#### Scenario: GUI 不提供轉出等寫入動詞

- **WHEN** 使用者檢視任一討論抽屜或討論卡
- **THEN** 介面不提供 conclude、add-round、轉為變更等寫入動作，轉出變更改由 CLI 或 agent 執行

#### Scenario: 同源 change 互跳

- **WHEN** 使用者開啟一個 from_discussion 非空的變更詳情抽屜
- **THEN** 抽屜顯示來源討論 topic 與同源變更清單，點擊同源項可開啟該變更的詳情

### Requirement: 討論於看板第 0 欄兩級呈現

看板 SHALL 於最左新增「討論」欄，依討論狀態兩級呈現：status 為 open 或 concluded 的討論 SHALL 為全尺寸卡（顯示 topic、輪數與狀態）——open 卡為唯讀，concluded 卡 SHALL 提供「封存」動詞（「轉為變更」動詞已自 GUI 撤除，轉出改由 CLI 或 agent）；status 為 promoted 的討論 SHALL 收合於欄底「已轉出變更的討論」群組的細列——細列 SHALL 以討論 topic 為首行（slug SHALL NOT 出現於看板），其下每個 promoted_to 子變更 SHALL 以樹狀前綴（末列 └、其餘 ├）逐列列出名稱與階段標示。子變更的階段 SHALL 由其於清單中的存在性派生：active 清單命中依看板欄位規則、封存清單命中為已封存、兩者皆無 SHALL 標示為已刪除且討論維持已轉出不回退。封存的討論 SHALL NOT 出現於此欄。輪數文案 SHALL 使用「N 輪」。

#### Scenario: 進行中與已結論討論的全卡呈現

- **WHEN** openspec/discussions/ 下存在一筆 status: open（3 輪）與一筆 status: concluded 的討論
- **THEN** 討論欄顯示兩張全卡：open 卡呈現 topic 與「3 輪」且無動詞按鈕，concluded 卡帶「封存」按鈕且無「轉為變更」按鈕

#### Scenario: 已轉出討論收合為衍生樹細列

- **WHEN** 一筆 topic 為「桌面即時刷新與封存瀏覽」的討論，其 promoted_to 含兩個變更，其一在 active 清單（有任務未開工）、其一已在封存清單
- **THEN** 該討論不以全卡呈現，而在「已轉出變更的討論」群組顯示一列：首行為該 topic，其下兩列樹狀子項——前者帶 ├ 前綴標示提案中、後者帶 └ 前綴標示已封存

##### Example: chip 階段派生矩陣

| promoted_to 子變更的所在 | 階段標示 |
| ------------------------ | -------- |
| active 清單，無 started、0/24 | 提案中 |
| active 清單，有 started、13/24 | 進行中 |
| active 清單，24/24 | 已就緒 |
| 封存清單（dated name 尾碼命中） | 已封存 |
| 兩清單皆無 | 已刪除（討論維持已轉出） |

#### Scenario: 外部推進輪次後欄自動更新

- **WHEN** 桌面 app 執行中，於外部以 CLI 對某 open 討論 add-round
- **THEN** 數秒內該討論卡的輪數自動更新，無需任何 app 內操作

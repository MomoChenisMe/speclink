## RENAMED Requirements

- FROM: `### Requirement: 討論抽屜檢視與 GUI 促轉`
- TO: `### Requirement: 討論抽屜檢視與轉出變更`

## MODIFIED Requirements

### Requirement: 討論於看板第 0 欄兩級呈現

看板 SHALL 於最左新增「討論」欄，依討論狀態兩級呈現：status 為 open 或 concluded 的討論 SHALL 為全尺寸卡（顯示 topic、輪數與狀態）——open 卡為唯讀，concluded 卡 SHALL 提供「轉為變更」與「封存」動詞；status 為 promoted 的討論 SHALL 收合於欄底「已轉出變更的討論」群組的細列——細列 SHALL 以討論 topic 為首行（slug SHALL NOT 出現於看板），其下每個 promoted_to 子變更 SHALL 以樹狀前綴（末列 └、其餘 ├）逐列列出名稱與階段標示。子變更的階段 SHALL 由其於清單中的存在性派生：active 清單命中依看板欄位規則、封存清單命中為已封存、兩者皆無 SHALL 標示為已刪除且討論維持已轉出不回退。封存的討論 SHALL NOT 出現於此欄。輪數文案 SHALL 使用「N 輪」。

#### Scenario: 進行中與已結論討論的全卡呈現

- **WHEN** openspec/discussions/ 下存在一筆 status: open（3 輪）與一筆 status: concluded 的討論
- **THEN** 討論欄顯示兩張全卡：open 卡呈現 topic 與「3 輪」且無動詞按鈕，concluded 卡帶「轉為變更」與「封存」按鈕

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

### Requirement: 討論抽屜檢視與 GUI 促轉

點擊討論卡或細列 SHALL 開啟討論抽屜。抽屜標題下方 SHALL 呈現生命週期階梯「討論中 → 已結論 → 轉出變更」且現站可辨識。分頁 SHALL 依序為：結論、討論過程 N、背景、衍生變更——前三者呈現記錄文件對應區段（區段缺失或格式非預期時 SHALL 整篇以單一檢視退回而非報錯）；記錄切分成功且結論區段非空時 SHALL 預設開啟「結論」分頁，結論為空時預設「背景」。衍生變更分頁 SHALL 列出各子變更現況與跳轉，並於 concluded 與 promoted 狀態提供轉出動詞——尚未轉出時按鈕文字為「轉為變更」、已轉出過為「再轉出一個變更」。轉為變更 SHALL 經確認後建立新變更——其 meta 含 from_discussion、proposal 以討論結論預填——並使新卡現身提案中欄、討論的 promoted_to 累積該名稱；確認框說明 SHALL 以使用者語言描述後果（新增變更卡、提案以結論開頭、討論移入已轉出區），SHALL NOT 出現 from_discussion、kebab-case 等工程詞，名稱輸入說明 SHALL 為「英文小寫，字間用 -」。concluded 卡的封存動詞 SHALL 經確認後將討論移入封存。轉為變更失敗（同名變更已存在、討論已封存等）SHALL 顯示單行錯誤且看板不變。GUI SHALL NOT 提供 conclude、add-round、new、discard——討論的推進與結論撰寫屬 agent 與 CLI。來自討論的變更卡 SHALL 帶討論徽章，其詳情抽屜 SHALL 顯示來源討論與同源變更清單並可互跳。

#### Scenario: 有結論的討論預設開啟結論分頁

- **WHEN** 使用者開啟一筆已結論（結論區段非空）討論的抽屜
- **THEN** 抽屜顯示分頁 結論／討論過程 N／背景／衍生變更，且預設呈現結論內容；階梯顯示「已結論」為現站

#### Scenario: GUI 轉為變更建立變更

- **WHEN** 使用者於已結論討論卡按「轉為變更」並確認
- **THEN** 新變更出現於提案中欄、其 .openspec.yaml 含 from_discussion、proposal.md 以結論預填；討論改於「已轉出變更的討論」群組以細列呈現且 promoted_to 含新變更名

#### Scenario: 再轉出一個變更（扇出第二刀）

- **WHEN** 使用者於已轉出討論的抽屜衍生變更分頁按「再轉出一個變更」、輸入新名稱並確認
- **THEN** 第二個變更建立並現身提案中欄，細列樹狀子項增加對應一列，promoted_to 累積兩個名稱

#### Scenario: 轉為變更失敗浮出錯誤

- **WHEN** 轉出的變更名與既有 active 變更同名
- **THEN** 前端顯示單行錯誤訊息，看板與討論記錄皆不變

#### Scenario: 同源 change 互跳

- **WHEN** 使用者開啟一個 from_discussion 非空的變更詳情抽屜
- **THEN** 抽屜顯示來源討論 topic 與同源變更清單，點擊同源項可開啟該變更的詳情

### Requirement: 已封存頁含討論節

已封存頁 SHALL 分「變更」與「討論」兩節：變更節維持既有展開列；討論節 SHALL 列出封存討論（日期＋topic）並可展開唯讀檢視記錄內容，SHALL NOT 提供任何寫入動詞。搜尋框 SHALL 同時過濾兩節。隨最後一個子變更歸檔而自動封存的討論、與經 GUI 或 CLI 手動封存的討論 SHALL 一致地出現於此節。展開檢視的區段標題 SHALL 使用「背景」「討論過程」「結論」。

#### Scenario: 封存討論唯讀展開

- **WHEN** 使用者於已封存頁討論節點擊一筆封存討論
- **THEN** 列展開顯示記錄內容（背景、討論過程、結論），無轉為變更、封存或任何寫入按鈕

#### Scenario: 自動封存的討論現身討論節

- **WHEN** 某已轉出討論的最後一個子變更被歸檔（觸發引擎的討論自動封存）且看板更新
- **THEN** 該討論自看板討論欄消失，已封存頁討論節出現該筆，搜尋其 topic 可命中

## ADDED Requirements

### Requirement: 討論於看板第 0 欄兩級呈現

看板 SHALL 於最左新增「討論」欄，依討論狀態兩級呈現：status 為 open 或 concluded 的討論 SHALL 為全尺寸卡（顯示 topic、回合數與狀態）——open 卡為唯讀，concluded 卡 SHALL 提供促轉與歸檔動詞；status 為 promoted 的討論 SHALL 以欄底收合細列呈現，列出每個 promoted_to 子 change 的名稱與階段標示。子 change 的階段 SHALL 由其於清單中的存在性派生：active 清單命中依看板欄位規則、封存清單命中為已封存、兩者皆無 SHALL 標示為已刪除且討論維持已促轉不回退。封存的討論 SHALL NOT 出現於此欄。

#### Scenario: 進行中與已結論討論的全卡呈現

- **WHEN** openspec/discussions/ 下存在一筆 status: open（3 回合）與一筆 status: concluded 的討論
- **THEN** 討論欄顯示兩張全卡：open 卡呈現 topic 與回合數且無動詞按鈕，concluded 卡帶促轉與歸檔按鈕

#### Scenario: 已促轉討論降為細列

- **WHEN** 一筆討論的 promoted_to 含兩個 change，其一在 active 清單（有任務未開工）、其一已在封存清單
- **THEN** 該討論不以全卡呈現，而在欄底細列列出兩個 chip：前者標示提案中、後者標示已封存

##### Example: chip 階段派生矩陣

| promoted_to 子 change 的所在 | chip 標示 |
| ---------------------------- | --------- |
| active 清單，無 started、0/24 | 提案中 |
| active 清單，有 started、13/24 | 進行中 |
| active 清單，24/24 | 已就緒 |
| 封存清單（dated name 尾碼命中） | 已封存 |
| 兩清單皆無 | 已刪除（討論維持已促轉） |

#### Scenario: 外部推進回合後欄自動更新

- **WHEN** 桌面 app 執行中，於外部以 CLI 對某 open 討論 add-round
- **THEN** 數秒內該討論卡的回合數自動更新，無需任何 app 內操作

### Requirement: 討論抽屜檢視與 GUI 促轉

點擊討論卡或細列 SHALL 開啟討論抽屜，含脈絡、回合、結論、促轉四分頁——前三者呈現記錄文件對應區段（區段缺失或格式非預期時 SHALL 整篇以單一檢視退回而非報錯），促轉分頁 SHALL 列出各子 change 現況與跳轉，並於 concluded 與 promoted 狀態提供「再促轉」。促轉（含再促轉）SHALL 經確認後建立新 change——其 meta 含 from_discussion、proposal 以討論結論預填——並使新卡現身提案中欄、討論的 promoted_to 累積該名稱。concluded 卡的歸檔動詞 SHALL 經確認後將討論移入封存。促轉失敗（同名 change 已存在、討論已封存等）SHALL 顯示單行錯誤且看板不變。GUI SHALL NOT 提供 conclude、add-round、new、discard——討論的推進與結論撰寫屬 agent 與 CLI。來自討論的 change 卡 SHALL 帶討論徽章，其詳情抽屜 SHALL 顯示來源討論與同源 change 清單並可互跳。

#### Scenario: GUI 促轉建立 change

- **WHEN** 使用者於已結論討論卡按促轉並確認
- **THEN** 新 change 出現於提案中欄、其 .openspec.yaml 含 from_discussion、proposal.md 以結論預填；討論改以細列呈現且 promoted_to 含新 change 名

#### Scenario: 再促轉扇出第二刀

- **WHEN** 使用者於已促轉討論的抽屜促轉分頁按再促轉並確認
- **THEN** 第二個 change 建立並現身提案中欄，細列增加對應 chip，promoted_to 累積兩個名稱

#### Scenario: 促轉失敗浮出錯誤

- **WHEN** 促轉衍生的 change 名與既有 active change 同名
- **THEN** 前端顯示單行錯誤訊息，看板與討論記錄皆不變

#### Scenario: 同源 change 互跳

- **WHEN** 使用者開啟一個 from_discussion 非空的 change 詳情抽屜
- **THEN** 抽屜顯示來源討論 topic 與同源 change 清單，點擊同源項可開啟該 change 的詳情

### Requirement: 已封存頁含討論節

已封存頁 SHALL 分「變更」與「討論」兩節：變更節維持既有展開列；討論節 SHALL 列出封存討論（日期＋topic）並可展開唯讀檢視記錄內容，SHALL NOT 提供任何寫入動詞。搜尋框 SHALL 同時過濾兩節。隨最後一個子 change 歸檔而自動歸檔的討論、與經 GUI 或 CLI 手動歸檔的討論 SHALL 一致地出現於此節。

#### Scenario: 封存討論唯讀展開

- **WHEN** 使用者於已封存頁討論節點擊一筆封存討論
- **THEN** 列展開顯示記錄內容（脈絡、回合、結論），無促轉、歸檔或任何寫入按鈕

#### Scenario: 自動歸檔的討論現身討論節

- **WHEN** 某已促轉討論的最後一個子 change 被歸檔（觸發引擎的討論自動歸檔）且看板更新
- **THEN** 該討論自看板討論欄消失，已封存頁討論節出現該筆，搜尋其 topic 可命中

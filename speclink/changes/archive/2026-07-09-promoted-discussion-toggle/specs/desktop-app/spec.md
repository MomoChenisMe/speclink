## MODIFIED Requirements

### Requirement: 討論於看板第 0 欄兩級呈現

看板 SHALL 於最左新增「討論」欄，依討論狀態兩級呈現：status 為 open 或 concluded 的討論 SHALL 為全尺寸卡（顯示 topic、輪數與狀態）——open 卡為唯讀，concluded 卡 SHALL 提供「封存」動詞（「轉為變更」動詞已自 GUI 撤除，轉出改由 CLI 或 agent）。討論欄的計數徽章 SHALL 只計 active 討論（open 與 concluded），SHALL NOT 計入 promoted。status 為 promoted 的討論 SHALL 預設隱藏且不佔欄體空間；當存在至少一筆 promoted 討論時，欄 header SHALL 提供一個「顯示已轉出」開關，呈 ↗ 圖示與 promoted 計數，點按 SHALL 切換 promoted 群組顯示；無任何 promoted 討論時該開關 SHALL 缺席。開關開啟時，promoted 討論 SHALL 於欄底「已轉出變更的討論」群組以細列呈現——細列 SHALL 以討論 topic 為首行（slug SHALL NOT 出現於看板），其下每個 promoted_to 子變更 SHALL 以樹狀前綴（末列 └、其餘 ├）逐列列出名稱與階段 chip。子變更的階段 SHALL 由其於清單中的存在性派生：active 清單命中依看板欄位規則、封存清單命中為已封存、兩者皆無 SHALL 標示為已刪除且討論維持已轉出不回退。階段 chip SHALL 以看板階段配色呈現：提案中、進行中、已就緒各對應該階段欄的 teal 濃度，已封存為中性色，已刪除為 destructive 色加刪除線。當討論欄無任何 active 討論但存在 promoted 討論時，欄體 SHALL NOT 顯示「尚無討論」空狀態。封存的討論 SHALL NOT 出現於此欄。輪數文案 SHALL 使用「N 輪」。

#### Scenario: 進行中與已結論討論的全卡呈現

- **WHEN** openspec/discussions/ 下存在一筆 status: open（3 輪）與一筆 status: concluded 的討論、無任何 promoted 討論
- **THEN** 討論欄顯示兩張全卡：open 卡呈現 topic 與「3 輪」且無動詞按鈕，concluded 卡帶「封存」按鈕且無「轉為變更」按鈕；欄計數徽章顯示 2，header 無「顯示已轉出」開關

#### Scenario: 已轉出討論預設隱藏並經 header 開關顯示

- **WHEN** 討論欄存在 promoted 討論、且「顯示已轉出」開關為預設關閉狀態
- **THEN** promoted 討論不佔欄體空間，header 呈現帶 promoted 計數的 ↗ 開關；點按開關後 promoted 討論於欄底「已轉出變更的討論」群組以衍生樹細列顯示，再點按即收起

#### Scenario: 無已轉出討論時開關缺席

- **WHEN** 討論欄無任何 promoted 討論
- **THEN** header 不顯示「顯示已轉出」開關

#### Scenario: 僅有已轉出討論時欄體不顯空狀態

- **WHEN** 討論欄無任何 active（open 或 concluded）討論、但存在至少一筆 promoted 討論
- **THEN** 欄體不顯示「尚無討論」文案，header 的 ↗ 開關傳達存在已轉出討論

#### Scenario: 已轉出細列的子變更樹與階段 chip 配色

- **WHEN** 開關開啟，一筆 promoted 討論的 promoted_to 含一個在 active 清單（提案中）與一個已在封存清單的子變更
- **THEN** 該討論列首行為 topic，其下兩列樹狀子項——前者帶 ├ 前綴與「提案中」chip（呈提案中欄的 teal 濃度）、後者帶 └ 前綴與「已封存」chip（中性色）

##### Example: chip 階段派生與配色矩陣

| promoted_to 子變更的所在 | 階段標示 | chip 配色 |
| ------------------------ | -------- | --------- |
| active 清單，無 started、0/24 | 提案中 | 提案中欄的 teal 濃度 |
| active 清單，有 started、13/24 | 進行中 | 進行中欄的 teal 濃度 |
| active 清單，24/24 | 已就緒 | 已就緒欄的 teal |
| 封存清單（dated name 尾碼命中） | 已封存 | 中性色 |
| 兩清單皆無 | 已刪除（討論維持已轉出） | destructive 加刪除線 |

#### Scenario: 外部推進輪次後欄自動更新

- **WHEN** 桌面 app 執行中，於外部以 CLI 對某 open 討論 add-round
- **THEN** 數秒內該討論卡的輪數自動更新，無需任何 app 內操作

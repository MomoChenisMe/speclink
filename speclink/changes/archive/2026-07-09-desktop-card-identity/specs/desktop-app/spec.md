## MODIFIED Requirements

### Requirement: 討論於看板第 0 欄兩級呈現

看板 SHALL 於最左新增「討論」欄，並以 header 的「顯示已轉出」開關在「討論中」與「已轉出」兩檢視間互斥切換（預設為討論中檢視）。

討論中檢視 SHALL 顯示 status 為 open 或 concluded 的討論為全尺寸卡——卡 SHALL 以 slug（檔名）為標題、topic 為卡身描述，並顯示輪數、狀態與建立者（createdBy，缺席時省略），且帶複製 slug 鈕（slug 為題屬 openspec/LANGUAGE.md 明載的受控例外，僅限 discuss 卡標題）；open 卡為唯讀，concluded 卡 SHALL 提供「封存」動詞（「轉為變更」動詞已自 GUI 撤除，轉出改由 CLI 或 agent）。

當存在至少一筆 promoted 討論時，欄 header SHALL 提供「顯示已轉出」開關，呈 ↗ 圖示與 promoted 計數；無任何 promoted 討論時該開關 SHALL 缺席。點按開關 SHALL 切換至已轉出檢視：欄標題由「討論」換為「已轉出討論」，只顯示 promoted 討論、討論中卡暫時隱藏；再點按即切回討論中檢視。

已轉出檢視中，promoted 討論 SHALL 自欄頂由上而下以細列呈現——細列 SHALL 以討論 topic 為首行（promoted 細列不顯 slug），其下每個 promoted_to 子變更 SHALL 以樹狀前綴（末列 └、其餘 ├）逐列列出名稱與階段 chip。子變更的階段 SHALL 由其於清單中的存在性派生：active 清單命中依看板欄位規則、封存清單命中為已封存、兩者皆無 SHALL 標示為已刪除且討論維持已轉出不回退。階段 chip SHALL 以看板階段配色呈現：提案中、進行中、已就緒各對應該階段欄的 teal 濃度，已封存為中性色，已刪除為 destructive 色加刪除線。

討論欄的計數徽章 SHALL 隨當前檢視顯示數量：討論中檢視顯 active（open 與 concluded）數、已轉出檢視顯 promoted 數。當討論欄無任何 active 討論但存在 promoted 討論時，討論中檢視 SHALL NOT 顯示「尚無討論」空狀態（改由 header 開關傳達）。封存的討論 SHALL NOT 出現於此欄。輪數文案 SHALL 使用「N 輪」。

#### Scenario: 進行中與已結論討論的全卡呈現

- **WHEN** openspec/discussions/ 下存在一筆 status: open（3 輪、frontmatter 含 created_by）與一筆 status: concluded 的討論、無任何 promoted 討論
- **THEN** 討論欄（討論中檢視）顯示兩張全卡，各以 slug 為標題、topic 為描述、顯示「3 輪」與建立者、並帶複製 slug 鈕；open 卡無動詞按鈕，concluded 卡帶「封存」按鈕且無「轉為變更」按鈕；欄計數徽章顯示 2，header 無「顯示已轉出」開關

#### Scenario: 複製討論 slug

- **WHEN** 點討論全卡的複製鈕
- **THEN** 該討論的 slug 寫入剪貼簿並短暫顯示已複製回饋

#### Scenario: 已轉出檢視經 header 開關互斥切換並換欄標題

- **WHEN** 討論欄存在 promoted 討論、且「顯示已轉出」開關為預設關閉狀態（討論中檢視）
- **THEN** 只顯示討論中全卡、promoted 隱藏且不佔空間，header 呈現帶 promoted 計數的 ↗ 開關；點按開關後切至已轉出檢視——欄標題換為「已轉出討論」、討論中卡隱藏、promoted 討論自欄頂以衍生樹細列顯示、計數徽章改顯 promoted 數；再點按即切回討論中檢視

#### Scenario: 無已轉出討論時開關缺席

- **WHEN** 討論欄無任何 promoted 討論
- **THEN** header 不顯示「顯示已轉出」開關

#### Scenario: 僅有已轉出討論時討論中檢視不顯空狀態

- **WHEN** 討論欄無任何 active（open 或 concluded）討論、但存在至少一筆 promoted 討論
- **THEN** 討論中檢視不顯示「尚無討論」文案，header 的 ↗ 開關傳達存在已轉出討論

#### Scenario: 已轉出細列的子變更樹與階段 chip 配色

- **WHEN** 切至已轉出檢視，一筆 promoted 討論的 promoted_to 含一個在 active 清單（提案中）與一個已在封存清單的子變更
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

## ADDED Requirements

### Requirement: 看板變更卡呈現建立者與關係提示

看板變更卡 SHALL 顯示建立者（createdBy）頭像——以建立者首字母圓標呈現，meta 無 created_by 時省略。變更卡的關係指示（來自討論、待重新反映）SHALL 於 hover 以主題化提示（shadcn Tooltip，取代原生 title）呈現對應資訊：來自討論指示於卡片來自討論時 SHALL 列出全部來源討論，待重新反映指示於卡片帶 restale 旗標時 SHALL 列出待重新反映的來源；無對應關係時該指示 SHALL 缺席。提示內容 SHALL 與原生 title 一致（改以主題化樣式呈現）。

#### Scenario: 變更卡顯示建立者頭像

- **WHEN** 看板呈現一個 meta 含 created_by 的變更卡
- **THEN** 卡片顯示該建立者的首字母圓標頭像

#### Scenario: 無建立者時省略頭像

- **WHEN** 變更卡的 meta 無 created_by
- **THEN** 卡片不顯示建立者頭像

#### Scenario: 關係指示以主題化 hover 提示呈現

- **WHEN** 使用者 hover 一個來自討論之變更卡的來自討論指示
- **THEN** 以主題化提示（shadcn Tooltip）列出全部來源討論，取代原生 title 呈現

## MODIFIED Requirements

### Requirement: 討論列表

選單討論區 SHALL 分兩個分區呈現 active 討論：「討論」分區列引擎不會自動收走的討論——含討論中（尚未轉出變更）、已轉出但尚無結論（promoted 且 concluded 為 false）、以及已轉出且已有結論但保留在途（promoted 且 concluded 為 true 且 hold 為 true）者；其後「已轉出」分區列已轉出且已有結論且未保留（promoted 且 concluded 為 true 且 hold 非 true）的討論，無此類討論時 SHALL NOT 顯示該分區。分區判準 SHALL 由快照導出的單一布林承載，面板與原生選單 SHALL NOT 各自計算。討論清單資料的 concluded 欄位缺席（向後相容路徑）時 SHALL 退回既有判準——promoted 一律列於「已轉出」分區；hold 欄位缺席時 SHALL 視同未保留、SHALL NOT 把缺席當成 true。兩分區的每則討論 SHALL 呈現為子選單：父項標籤 SHALL 為該討論的 slug（識別錨點直出）；子選單 SHALL 依序含 topic 描述行（disabled、不可選取）、「開啟此討論」、「複製 slug」；保留在途的討論 SHALL NOT 帶額外標籤。選取「開啟此討論」SHALL 顯示主視窗並取得焦點、且開啟該討論。選取「複製 slug」SHALL 將該討論的 slug 寫入系統剪貼簿，主視窗隱藏或無焦點時 SHALL 仍寫入成功。無討論中討論時「討論」分區 SHALL 顯示「討論 0」。

#### Scenario: 討論以 slug 為題、topic 為描述

- **WHEN** 存在 active 討論（slug 為 board-search-bar、topic 為「看板搜尋列」）
- **THEN** 討論區該項父標籤為「board-search-bar」，展開子選單首行為灰字「看板搜尋列」且不可選取

#### Scenario: 開啟某討論

- **WHEN** 使用者展開某討論子選單並選取「開啟此討論」
- **THEN** 主視窗顯示於前景並取得焦點，且開啟該討論

#### Scenario: 複製討論 slug

- **WHEN** 使用者展開某討論子選單並選取「複製 slug」
- **THEN** 系統剪貼簿內容等於該討論的 slug

#### Scenario: 已轉出討論列於已轉出分區

- **WHEN** 存在討論中的討論與已轉出且已有結論且無 hold 的討論各一
- **THEN** 討論中的討論列於「討論」分區、已轉出且已有結論且無 hold 的列於「已轉出」分區，兩者子選單結構相同（topic 描述行、開啟此討論、複製 slug）

#### Scenario: 已轉出但尚無結論的討論列於討論分區

- **WHEN** 存在一筆已轉出但尚無結論（promoted 且 concluded 為 false）的討論
- **THEN** 該討論列於「討論」分區而非「已轉出」分區；若無其他已轉出且已有結論的討論，選單不出現「已轉出」分區標題

#### Scenario: 保留中的已轉出討論列於討論分區

- **WHEN** 存在一筆已轉出、已有結論且 hold 為 true 的討論，與一筆已轉出、已有結論且 hold 為 false 的討論
- **THEN** 前者列於「討論」分區並計入「討論 N」、子選單結構與其他討論相同且無額外標籤；後者列於「已轉出」分區

#### Scenario: 無已轉出討論時不顯示該分區

- **WHEN** 目前沒有任何已轉出且已有結論且無 hold 的討論
- **THEN** 選單不出現「已轉出」分區標題

#### Scenario: 無討論時顯示零

- **WHEN** 目前沒有任何討論中的討論
- **THEN** 「討論」分區顯示「討論 0」

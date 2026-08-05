## ADDED Requirements

### Requirement: 看板討論卡片的改進標示

kind 為 improve 的討論卡片 SHALL 於卡片上渲染行內小章(不增加文字列、維持卡片極簡),小章 SHALL 附 tooltip 顯示狀態詞——文案取自 i18n 詞條,tw 使用共用詞彙的正典詞「改進討論」、en 使用對應英文詞條,未設定語言時依應用預設語言。一般討論(無 kind)SHALL NOT 渲染任何新增元素。標示 SHALL 隨 kind 恆定:已轉出變更與已封存側的改進討論 SHALL 維持同一標示,SHALL NOT 隨生命週期狀態變化或消失。

#### Scenario: 改進討論卡片

- **WHEN** 看板顯示一筆 kind 為 improve 的討論(語言 tw)
- **THEN** 該卡片出現改進小章,tooltip 為「改進討論」

#### Scenario: 一般討論卡片無標示

- **WHEN** 看板顯示一筆無 kind 欄位的討論
- **THEN** 卡片不出現改進小章,與本變更引入前的渲染一致

#### Scenario: 已封存的改進討論維持標示

- **WHEN** 已封存頁顯示一筆 kind 為 improve 的已封存討論
- **THEN** 該筆項目仍顯示改進小章,tooltip 詞彙與看板側一致

### Requirement: 討論抽屜的改進標示

討論抽屜 SHALL 於 kind 為 improve 時顯示改進標示,文案與卡片 tooltip 使用同一 i18n 詞條;一般討論的抽屜 SHALL NOT 顯示該標示。

#### Scenario: 改進討論抽屜

- **WHEN** 開啟 kind 為 improve 的討論抽屜(語言 tw)
- **THEN** 抽屜顯示「改進討論」標示

#### Scenario: 一般討論抽屜無標示

- **WHEN** 開啟無 kind 欄位的討論抽屜
- **THEN** 抽屜不顯示改進標示,與本變更引入前的渲染一致

## ADDED Requirements

### Requirement: 看板卡片的審查標示

change 卡片 SHALL 依審查狀態渲染行內小章（不增加文字列、維持卡片極簡）：`none` 無標示；`inReview` 顯示「審查中」章；`reviewed` 顯示「已審查」章；`reviewedStale` 顯示降級樣式章。章 SHALL 附 tooltip 顯示狀態詞；文案取自 i18n 詞條——tw 使用共用詞彙的正典詞（審查中／已審查／已審查·其後有變動），en 使用對應英文詞條，未設定語言時依應用預設語言。

#### Scenario: 已審查卡片

- **WHEN** 看板載入且某 change 的協定資料為 `reviewStatus: "reviewed"`
- **THEN** 該卡片名稱旁出現審查章，tooltip 為「已審查」（tw）

#### Scenario: 降級章

- **WHEN** 協定資料為 `reviewStatus: "reviewedStale"`
- **THEN** 卡片章以降級樣式呈現，tooltip 為「已審查·其後有變動」（tw）

#### Scenario: 無審查痕跡

- **WHEN** 協定資料為 `reviewStatus: "none"`
- **THEN** 卡片無任何審查相關元素

### Requirement: 詳情抽屜的審查資訊列

change 詳情抽屜 SHALL 於審查狀態非 `none` 時顯示審查資訊列：狀態詞、蓋章時間與審查者（`inReview` 時僅顯示狀態詞）。狀態為 `none` 時 SHALL 不渲染該區塊。

#### Scenario: 已審查抽屜

- **WHEN** 開啟 `reviewStatus: "reviewed"` 的 change 抽屜
- **THEN** 資訊列顯示「已審查」與 reviewedAt／reviewedBy 內容

#### Scenario: 審查中抽屜

- **WHEN** 開啟 `reviewStatus: "inReview"` 的 change 抽屜
- **THEN** 資訊列顯示「審查中」，無時間與審查者

### Requirement: 已封存側的審查標示

已封存頁的清單與抽屜 SHALL 依封存時的審查結局標示：帶章者顯示「已審查」；帶化石工單而無章者顯示「曾審查未通過」；皆無者無標示。

#### Scenario: 曾審查未通過

- **WHEN** 已封存 change 的目錄含 review.md 而 metadata 無 reviewed 欄位
- **THEN** 已封存清單項與抽屜顯示「曾審查未通過」標示

### Requirement: 封存入口的未結工單三選項

desktop 的封存入口（卡片封存鈕、抽屜封存動作、拖曳封存）SHALL 於目標 change 為 `inReview` 時彈出三選項對話框：前往完成蓋章（導引執行審查收尾）、放棄審查（等同 review discard 後封存）、照樣帶走（等同 `--carry-review` 封存，並說明將永久顯示「曾審查未通過」）。非 `inReview` 的封存行為 SHALL 維持現行不變。

#### Scenario: 封存審查中的 change

- **WHEN** 對 `reviewStatus: "inReview"` 的 change 觸發封存
- **THEN** 出現三選項對話框，未選擇前不執行封存

#### Scenario: 選擇照樣帶走

- **WHEN** 於對話框選擇「照樣帶走」
- **THEN** change 封存成功且已封存側顯示「曾審查未通過」

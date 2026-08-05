## ADDED Requirements

### Requirement: 看板卡片的驗證標示

change 卡片 SHALL 依驗證狀態渲染第二顆行內小章，與審查章並排且順序固定（審查章在前、驗證章在後）：`none` 無標示；`inVerify` 顯示「驗證中」章；`verified` 顯示「已驗證」章；`verifiedStale` 顯示降級樣式章。tooltip 文案取自 i18n 詞條——tw 使用正典詞（驗證中／已驗證／已驗證·其後有變動），en 使用對應詞條。驗證章各狀態的配色 SHALL 與審查章對應狀態同值（色承載狀態、圖示形狀承載站別），SHALL NOT 另建配色；驗證章圖示 SHALL 採盾牌形系，與審查章的徽章形系可區辨。

#### Scenario: 兩章並排

- **WHEN** 某 change 的協定資料為 `reviewStatus: "reviewed"` 且 `verifyStatus: "verified"`
- **THEN** 卡片依序顯示審查章與驗證章，各自 tooltip 正確

#### Scenario: 兩站同狀態同色、異形可辨

- **WHEN** 某 change 同時為 `reviewStatus: "reviewed"` 與 `verifyStatus: "verified"`
- **THEN** 兩章配色同值（皆為品質站蓋章色），審查章為徽章形、驗證章為盾牌形，站別可一眼區辨

#### Scenario: 僅驗證章

- **WHEN** 協定資料為 `reviewStatus: "none"` 且 `verifyStatus: "verifiedStale"`
- **THEN** 卡片僅顯示降級樣式的驗證章，tooltip 為「已驗證·其後有變動」（tw）

### Requirement: 詳情抽屜的驗證資訊列

change 詳情抽屜 SHALL 於驗證狀態非 `none` 時顯示驗證資訊列（狀態詞、蓋章時間與驗證者；`inVerify` 僅狀態詞），與審查資訊列同構並列；狀態為 `none` 時不渲染。

#### Scenario: 已驗證抽屜

- **WHEN** 開啟 `verifyStatus: "verified"` 的 change 抽屜
- **THEN** 驗證資訊列顯示「已驗證」與 verifiedAt／verifiedBy 內容

### Requirement: 已封存側的驗證標示

已封存頁的清單與抽屜 SHALL 依封存時的驗證結局標示：帶章者「已驗證」；帶化石驗證工單而無章者「曾驗證未通過」；皆無者無標示。與審查結局標示可並存。

#### Scenario: 曾驗證未通過

- **WHEN** 已封存 change 的目錄含 verify.md 而 metadata 無 verified 欄位
- **THEN** 已封存清單項與抽屜顯示「曾驗證未通過」標示

### Requirement: 封存入口三選項擴及驗證工單

desktop 封存入口 SHALL 於目標 change 存在未結驗證工單時彈三選項對話框（前往完成驗證蓋章／放棄驗證／照樣帶走並警示永久標示）；review 與 verify 工單並存時 SHALL 讓使用者對兩種工單分別處置後才執行封存。

#### Scenario: 封存驗證中的 change

- **WHEN** 對 `verifyStatus: "inVerify"` 的 change 觸發封存
- **THEN** 出現驗證三選項對話框，未選擇前不執行封存

#### Scenario: 雙工單並存的封存

- **WHEN** 對同時為 `inReview` 與 `inVerify` 的 change 觸發封存
- **THEN** 對話流程涵蓋兩種工單的處置選擇，全部選定後才封存

## MODIFIED Requirements

### Requirement: 看板欄位由生命週期標記驅動

看板欄位 SHALL 依下列優先序判定：任務全完成（總數大於 0 且完成數等於總數）＝已就緒；meta 含 started_at 或任務完成數大於 0＝進行中；其餘＝提案中。剛完成 propose（有任務、全未勾、未標記開工）的 change SHALL 顯示於提案中欄。詳情抽屜 SHALL 於 meta 含 started_at 時顯示開工者與開工日（started_by、started_at）；meta 無 started_at 時 SHALL NOT 顯示開工資訊——即使該 change 因任務進度列於進行中欄（派生管顯示，歸屬缺席維持缺席）。此判定 SHALL 對 fs 與 remote 資料來源一致：remote 清單 payload 的 startedAt SHALL 進入看板與系統匣的欄位推導，SHALL NOT 以任務完成數替代開工判定（完成數大於 0 的 fallback 保留，涵蓋繞過工具的寫入路徑）。

#### Scenario: 未開工的 change 留在提案中

- **WHEN** 某 change 的 tasks.md 含 28 項任務全未勾、meta 無 started_at
- **THEN** 看板將其顯示於「提案中」欄，卡片任務數為 0/28

#### Scenario: 標記開工後移入進行中

- **WHEN** 對上述 change 執行 speclink in-progress add 後看板更新
- **THEN** 該卡片移至「進行中」欄，抽屜標頭顯示開工者與開工日

#### Scenario: 無章而有任務進度列於進行中

- **WHEN** 某 change 的 meta 無 started_at，其 tasks.md 經任意途徑（如編輯器直接修改後 git pull 或本機儲存）成為 3/28 已勾，看板刷新
- **THEN** 該卡片顯示於「進行中」欄，詳情抽屜不顯示開工者與開工日

#### Scenario: remote 已開工零進度列於進行中

- **WHEN** remote 連線的 change 清單中某 change 帶 startedAt 且任務進度為 0/15，看板與系統匣刷新
- **THEN** 該卡片顯示於「進行中」欄、系統匣列於進行中分區——不因完成數為 0 而落回提案中

##### Example: 欄位判定矩陣

| meta started_at | 任務進度 | 看板欄 |
| --------------- | -------- | ------ |
| 無 | 0 任務 | 提案中 |
| 無 | 0/28 | 提案中 |
| 無 | 3/28 | 進行中（抽屜無開工資訊） |
| 有 | 0/28 | 進行中 |
| 有 | 13/28 | 進行中 |
| 無 | 28/28 | 已就緒（全完成優先） |
| 有 | 28/28 | 已就緒 |

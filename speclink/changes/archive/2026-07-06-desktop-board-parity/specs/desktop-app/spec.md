## ADDED Requirements

### Requirement: 外部變更即時反映

桌面 app SHALL 監看目前專案的 openspec/ 目錄樹：app 之外的寫者（CLI、agent、手動編輯器）修改其下任何文件後，看板、詳情抽屜與已封存頁 SHALL 在短時間內（秒級）自動更新呈現，SHALL NOT 要求重啟或任何 app 內操作。app 內操作（勾任務、動詞）後的即時反映 SHALL 維持既有行為。監看不可用時（如檔案系統權限）app SHALL 照常提供其餘功能——僅失去自動刷新，SHALL NOT 崩潰或反覆彈出錯誤。

#### Scenario: 外部勾選任務後看板自動更新

- **WHEN** 桌面 app 執行中，於外部終端執行 speclink task done 勾掉某 change 的一項任務
- **THEN** 數秒內該 change 的看板卡片任務數與進度條更新，抽屜若開啟亦同步，全程無任何 app 內操作

#### Scenario: 外部新增與歸檔反映到清單

- **WHEN** 於外部以 CLI 建立新 change，隨後將另一 change 歸檔
- **THEN** 數秒內看板出現新 change 卡片，被歸檔者自看板消失並出現於已封存頁

#### Scenario: 監看不可用時功能照常

- **WHEN** 檔案監看因環境因素無法建立
- **THEN** app 啟動與所有查詢、操作照常運作，錯誤僅記錄於日誌，畫面無錯誤彈窗堆疊

### Requirement: 已封存變更可展開檢視

已封存頁的每列 SHALL 顯示日期、名稱與任務數徽章，且 SHALL 可展開為唯讀檢視——至少含提案、設計、任務、規格分頁，內容來自封存目錄的實體文件。檢視 SHALL 為唯讀：SHALL NOT 提供任務勾選、動詞執行或任何寫入操作。所請求的文件不存在時對應分頁 SHALL 顯示空狀態而非錯誤。

#### Scenario: 展開封存列檢視文件

- **WHEN** 使用者於已封存頁點擊一個含完整 artifacts 的封存列
- **THEN** 列展開顯示提案／設計／任務／規格分頁，各分頁呈現封存目錄內對應文件的內容，任務分頁的核取方塊不可點擊

#### Scenario: 徽章顯示任務計數

- **WHEN** 已封存頁載入一個 tasks.md 為 48 項全勾的封存變更
- **THEN** 該列顯示 48/48 徽章；無 tasks.md 的封存變更不顯示徽章

#### Scenario: 缺件文件顯示空狀態

- **WHEN** 使用者展開一個無 design.md 的封存變更並切至設計分頁
- **THEN** 分頁顯示空狀態文字，無錯誤彈窗，其餘分頁照常可用

### Requirement: 看板欄位由生命週期標記驅動

看板欄位 SHALL 依下列優先序判定：任務全完成（總數大於 0 且完成數等於總數）＝已就緒；meta 含 started_at＝進行中；其餘＝提案中。剛完成 propose（有任務、未標記開工）的 change SHALL 顯示於提案中欄。詳情抽屜 SHALL 於 change 已開工時顯示開工者與開工日（started_by、started_at），未開工時不顯示該資訊。

#### Scenario: 未開工的 change 留在提案中

- **WHEN** 某 change 的 tasks.md 含 28 項任務全未勾、meta 無 started_at
- **THEN** 看板將其顯示於「提案中」欄，卡片任務數為 0/28

#### Scenario: 標記開工後移入進行中

- **WHEN** 對上述 change 執行 speclink in-progress add 後看板更新
- **THEN** 該卡片移至「進行中」欄，抽屜標頭顯示開工者與開工日

##### Example: 欄位判定矩陣

| meta started_at | 任務進度 | 看板欄 |
| --------------- | -------- | ------ |
| 無 | 0 任務 | 提案中 |
| 無 | 0/28 | 提案中 |
| 有 | 13/28 | 進行中 |
| 無 | 28/28 | 已就緒（全完成優先） |
| 有 | 28/28 | 已就緒 |

## ADDED Requirements

### Requirement: 已封存頁的流程成本摘要

已封存頁 SHALL 在頁標題區與列表卡之間顯示「流程成本」摘要，以全部已封存變更（不含已封存討論，且不受頁內搜尋影響）的流程成本欄位計算，最多四項：

- 文件行數：`docLines` 的中位數與 P90
- 任務數：`tasksTotal` 的中位數
- 建立到封存：`leadDays` 的中位數（天）；有 `leadHours` 樣本時，同一項另顯示 `leadHours` 的中位數（小時）
- 品質關卡輪數：`reviewRounds` 與 `verifyRounds` 各自的中位數，分別標為審查與驗證

每個數字 SHALL 標出樣本數，也就是帶有該欄位的封存變更筆數。樣本數為 0 的數字 SHALL NOT 顯示；一項內所有數字都沒有樣本時，該項 SHALL NOT 顯示；沒有任何封存變更帶 `docLines`、`leadDays`、`leadHours`、`reviewRounds` 或 `verifyRounds` 時（包含沒有任何封存變更，以及清單只帶 `tasksTotal`、不帶這些欄位的遠端工作區），整個摘要 SHALL NOT 顯示，即使任務數有樣本。中位數 SHALL 為由小到大排序後的中間值，筆數為偶數時取中間兩值的平均；P90 SHALL 為由小到大第 ⌈0.9 × n⌉ 個值。數字 SHALL 以最多一位小數顯示，整數不帶小數點。摘要文案 SHALL 跟隨 UI 語言：繁中為「流程成本」「文件行數」「任務數」「建立到封存」「品質關卡輪數」「審查」「驗證」「中位數」「P90」「天」「小時」「筆」；英文為 "Process cost"、"Doc lines"、"Tasks"、"Created to archived"、"Quality gate rounds"、"Review"、"Verify"、"median"、"P90"、"days"、"hours"、"changes"。摘要 SHALL 為唯讀，SHALL NOT 提供任何寫入動作。

#### Scenario: 顯示中位數、P90 與樣本數

- **WHEN** 本機工作區有 3 筆封存變更，`docLines` 分別為 100、200、400，`tasksTotal` 分別為 4、6、9，`leadDays` 分別為 0、0、3
- **THEN** 已封存頁的流程成本摘要顯示文件行數中位數 200、P90 400、3 筆，任務數中位數 6、3 筆，建立到封存中位數 0 天、3 筆

#### Scenario: 偶數筆取中間兩值平均

- **WHEN** 本機工作區有 2 筆封存變更，`leadDays` 分別為 0 與 1，`leadHours` 分別為 6 與 30.5
- **THEN** 建立到封存顯示中位數 0.5 天、2 筆，以及中位數 18.3 小時、2 筆

#### Scenario: 舊記錄沒有輪數時不顯示該項

- **WHEN** 所有封存變更都沒有 `reviewRounds` 與 `verifyRounds`
- **THEN** 流程成本摘要不顯示品質關卡輪數這一項，其他有樣本的項目照常顯示

#### Scenario: 沒有資料時整個摘要不出現

- **WHEN** 已封存頁沒有任何封存變更，或處於清單只帶 `tasksTotal`、不帶流程成本欄位的遠端工作區
- **THEN** 頁標題區與列表卡之間沒有流程成本摘要，列表卡與分頁照常顯示

#### Scenario: 搜尋不改變摘要數字

- **WHEN** 使用者在已封存頁的搜尋框輸入只命中 1 筆封存變更的字串
- **THEN** 列表只顯示命中的那一筆，流程成本摘要的數字與樣本數維持全部封存變更的計算結果

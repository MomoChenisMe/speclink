## ADDED Requirements

### Requirement: engine 動詞經橋接於 TeamStore 上執行

Host SHALL 提供 engine-over-TeamStore 執行橋接：以 TeamStore snapshot 供應 engine 命令層的讀取視圖、把變更型動詞的寫入捕捉為 UnitOfWork staged ops、成功時連同領域事件經 Host 的 commit 組合路徑原子提交。同一動詞對語意相同的內容分別經本地 fs seam 與經橋接執行，typed outcome、錯誤分類與領域事件 SHALL 一致；TeamStore 的 revision_conflict SHALL 映射為命令層錯誤且保留 expected/actual 詳情。橋接 SHALL NOT 分叉 engine 命令層的動詞語意，發現的檔案系統暗依賴 SHALL 修在橋接視圖。

#### Scenario: 雙路徑 outcome 一致

- **WHEN** 對含相同 change 內容的 fs workspace 與 TeamStore scope 分別執行同一查詢動詞與同一變更型動詞
- **THEN** 兩路徑的 typed outcome 結構相等、變更型動詞回報相同種類的領域事件；失敗情境（如 not_found）的錯誤碼相同

#### Scenario: 橋接寫入原子落店

- **WHEN** 經橋接執行 task done 成功
- **THEN** 任務勾選後的文件內容、revision 遞增與 task-completed 事件記錄在同一 commit 內可見；commit 前 store 無任何中間狀態

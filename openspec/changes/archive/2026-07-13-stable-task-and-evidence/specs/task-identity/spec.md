## ADDED Requirements

### Requirement: 任務行內嵌不可變 stable ID

Engine 產出 tasks artifact 時 SHALL 為每個任務行指派 stable ID：tsk_ 前綴加 ULID，以 speclink-task 標記的 HTML 註解內嵌於該行尾。ID SHALL 不可變：任務重排、改寫描述或增刪其他任務 SHALL NOT 改變既有任務的 ID；SHALL NOT 以內容 hash 或 ordinal 作永久身分。

#### Scenario: 產出的 tasks.md 全檔帶 ID

- **WHEN** 經 new artifact tasks 寫入含五個任務的 tasks.md
- **THEN** 五個任務行尾各帶一個唯一的 speclink-task 註解（tsk_ 前綴），任務描述文字不變

#### Scenario: 重排不改 ID

- **WHEN** 將 tasks.md 內兩個帶 ID 的任務對調位置後讀取任務清單
- **THEN** 兩任務的 stable ID 與對調前相同，僅顯示序數互換

### Requirement: task done 對無 ID 目標行單行補章

task done 遇目標任務行無 stable ID 時，SHALL 於同一次寫入對該行補指派 ID：除勾選標記與該行行尾註解外，tasks.md 其餘內容 SHALL 逐位元不變。task undone SHALL NOT 補章。既有無 ID 的 tasks.md SHALL NOT 被任何背景程序全檔改寫。

#### Scenario: 補章只動目標行

- **WHEN** 對全檔皆無 ID 的 tasks.md 執行 speclink task done 3
- **THEN** 僅第 3 個任務行變更（勾選加行尾 ID 註解）；其餘行逐位元不變；再次讀取時該任務可以新 ID 定址

### Requirement: 定址接受 ordinal 與 stable ID 雙值域

task done 與 task undone 的 task-id 參數 SHALL 接受兩種值域：純數字走既有 ordinal 定址（人眼與 --json 輸出、exit code、錯誤訊息與現行逐位元一致）；tsk_ 前綴走 stable ID 查找，查無此 ID 時 SHALL 回與 ordinal 超界對稱的錯誤形狀。其餘值 SHALL 沿現行非法 task id 錯誤。重排 tasks.md 後 stable ID 定址 SHALL 仍命中原任務。

#### Scenario: stable ID 在重排後仍命中

- **WHEN** 記下第 2 個任務的 tsk_ ID、將其移到清單末尾後執行 speclink task done 該 ID
- **THEN** 被勾選的是原任務（描述相同）；以原 ordinal 2 定址則命中的是移入該位置的別的任務

#### Scenario: 數字值域輸出凍結

- **WHEN** 對同一 workspace 於本變更前後執行 speclink task done 3 --change demo --json
- **THEN** stdout、stderr 與 exit code 逐位元一致

### Requirement: 重複 stable ID 使 task 動詞拒絕

tasks.md 內出現重複的 stable ID 時，task done 與 task undone SHALL 拒絕並於錯誤訊息點名重複的 ID 值，SHALL NOT 靜默選取任一筆，且 SHALL NOT 寫入任何檔案。

#### Scenario: 重複 ID 拒絕

- **WHEN** tasks.md 兩個任務行帶相同的 tsk_ ID，對該 ID 執行 speclink task done
- **THEN** 以非零 exit code 結束，stderr 點名重複的 ID；tasks.md 逐位元不變

### Requirement: 任務事件載荷攜 stable ID

task-completed 與 task-uncompleted 領域事件的任務識別 SHALL 為 stable ID 字串（task done 對無 ID 任務以該次補章後的 ID 入載荷；task undone 對無 ID 任務以序數字串入載荷）。事件契約沿 command-runtime 的 experimental 標示。

#### Scenario: 完成事件攜 ID

- **WHEN** 經命令層對帶 ID 的任務執行 task done
- **THEN** 執行結果附帶恰一筆 task-completed 事件，任務識別為該 tsk_ ID

### Requirement: UI 剝離 ID 註解並以 stable ID 操作

任務清單呈現 SHALL 剝離 speclink-task 註解（使用者可見文字與無註解時相同）；清單項 SHALL 以 stable ID 作呈現 key（無 ID 舊檔退回 ordinal key）；勾選操作 SHALL 以 stable ID 定址（無 ID 任務走 ordinal 相容路徑）；樂觀就地改寫 SHALL 保留行尾註解原文。

#### Scenario: 桌面顯示無標記且勾選命中

- **WHEN** 桌面載入帶 ID 註解的 tasks.md 並勾選其中一項
- **THEN** 清單顯示的任務文字不含註解；勾選請求攜該任務的 tsk_ ID；tasks.md 該行翻轉且註解原文保留

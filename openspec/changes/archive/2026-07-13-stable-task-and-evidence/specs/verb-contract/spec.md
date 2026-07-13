## MODIFIED Requirements

### Requirement: 任務取消勾選動詞

<!-- BEFORE: task-id 值域僅接受數字（「task id 非數字」即錯），無 stable ID 定址 -->

CLI SHALL 提供 speclink task undone <task-id>（旗標 --change <name> 與 --json，無 stdin），把已勾選的任務翻回未勾選。task-id SHALL 接受兩種值域：純數字（ordinal 定址，行為與輸出與現行逐位元一致）與 tsk_ 前綴的 stable ID（依 task-identity 能力查找）。取消勾選 SHALL 為純狀態翻轉：SHALL NOT 寫入 touched 記錄、SHALL NOT 變更 change 的開工標記、SHALL NOT 補指派 stable ID。省略 --change 時的變更解析規則 SHALL 與 task done 相同。成功時 exit code 為 0；任務已是未勾選、task id 既非數字亦非 tsk_ 前綴、task id 超界或查無此 stable ID、tasks.md 不存在時 SHALL 以非 0 exit code 結束、stderr 輸出語義化訊息（形狀與 task done 的對應錯誤對稱）且無任何檔案效果。remote 模式下本動詞的人眼與 --json 輸出形狀 SHALL 與 fs 模式一致。本動詞為對 Spectra 2.3.1 的刻意延伸（Spectra 無此動詞），SHALL NOT 改變既有指令的 parity 基線。

#### Scenario: 取消已勾選的任務

- **WHEN** 對 tasks.md 中第 3 個任務已勾選的 change 執行 speclink task undone 3 --change demo
- **THEN** tasks.md 僅該任務由 [x] 變回 [ ]（縮排與 bullet 風格保留、其餘內容不變），stdout 顯示成功訊息（--no-color 下無 ANSI 序列），exit code 為 0，且 .speclink/ 下無新增 touched 記錄

#### Scenario: 以 stable ID 取消勾選

- **WHEN** 對帶 tsk_ ID 且已勾選的任務執行 speclink task undone 該 ID --change demo
- **THEN** tasks.md 僅該任務由 [x] 變回 [ ]（行尾 ID 註解原文保留），exit code 為 0；--json 形狀與數字值域一致

#### Scenario: --json 輸出形狀與 task done 對稱

- **WHEN** 執行 speclink task undone 3 --change demo --json
- **THEN** stdout 為 compact 單行 JSON，鍵依序為 change、status、task_desc、task_id，status 值為 undone，exit code 為 0

##### Example: 取消第 3 個任務的 payload

- **GIVEN** change demo 的 tasks.md 第 3 個任務為已勾選的「1.3 Third」
- **WHEN** 執行 speclink task undone 3 --change demo --json
- **THEN** stdout 為 {"change":"demo","status":"undone","task_desc":"1.3 Third","task_id":"3"}

#### Scenario: 任務已是未勾選

- **WHEN** 對未勾選的任務執行 speclink task undone
- **THEN** exit code 非 0，stderr 單行訊息說明該任務已是未完成狀態，tasks.md 與 .speclink/ 無任何變更

#### Scenario: tasks.md 不存在

- **WHEN** 對沒有 tasks.md 的 change 執行 speclink task undone 1
- **THEN** exit code 非 0，stderr 訊息指出該 change 的 tasks.md 不存在（與 task done 的同情境訊息一致）

#### Scenario: remote 模式輸出形狀與 fs 一致

- **WHEN** 於 remote 模式（server 回應成功）執行 speclink task undone 3 --change demo --json
- **THEN** stdout 的 JSON 鍵名與鍵序與 fs 模式完全一致，exit code 為 0

#### Scenario: 取消勾選不回滾開工標記

- **WHEN** 對已有開工標記的 change 執行 speclink task undone 取消其唯一已勾選的任務
- **THEN** 該 change 的開工標記維持原值，touched 記錄內容不減少

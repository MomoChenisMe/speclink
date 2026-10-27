## ADDED Requirements

### Requirement: in-progress 標記可自 change meta 移除(零工作痕跡守門)

執行 speclink in-progress remove 某 change 後,系統 SHALL 僅在該 change 為零工作痕跡——tasks.md 的已勾任務數為 0(tasks.md 不存在視為 0)且 touched 記錄的 v1 與 v2 兩清單皆空(記錄檔不存在視為空)——時,自該 change 的 .openspec.yaml 移除 started_at、started_by、started_with 三欄位;其餘欄位與內容 SHALL 逐字保留,SHALL NOT 重新序列化整份文件。子指令不接旗標、不讀 stdin。成功移除 SHALL exit 0 並於 stdout 印出移除確認;對未開工(無任何 started_* 欄位)的 change SHALL 冪等成功——exit 0、零檔案寫入。已勾任務數 > 0 或 touched 記錄非空時 SHALL 拒絕:exit 非 0,stderr SHALL 列出已勾任務數與 touched 記錄的檔案清單(兩清單聯集、去重)及出路說明(已勾任務可取消後重試;touched 需以人工或 agent 判斷處理),且 SHALL NOT 修改任何檔案。指名不存在的 change SHALL exit 非 0 並於 stderr 報找不到——此行為與 in-progress add 對未知名稱的靜默成功刻意不對稱(add 受遷移前 parity 凍結,remove 為新動詞、修正動作打錯名字必須明確報錯)。change meta 損毀無法解析時 SHALL fail-closed 報錯且不動任何檔案。本指令 SHALL NOT 提供任何強制旗標或機械清理已勾任務/touched 記錄的路徑,SHALL NOT 影響 speclink in-progress add 的既有輸出與行為。

#### Scenario: 零痕跡的進行中變更成功退回

- **WHEN** 一個 change 曾執行 in-progress add(meta 含 started_at 與 started_by),tasks.md 無任何已勾任務且無 touched 記錄,對其執行 speclink in-progress remove
- **THEN** exit 0,stdout 印出移除確認;meta 的 started_at、started_by、started_with 消失,schema、created_*、from_discussion 等其餘欄位逐字不變

#### Scenario: 已勾任務時拒絕退回

- **WHEN** 一個進行中的 change 其 tasks.md 有 2 個已勾任務,對其執行 speclink in-progress remove
- **THEN** exit 非 0,stderr 說明有 2 個已勾任務並提示取消勾選後可重試;meta 與 tasks.md 皆不變

#### Scenario: touched 記錄非空時拒絕退回並列出檔案

- **WHEN** 一個進行中的 change 已勾任務數為 0,但 touched 記錄含檔案,對其執行 speclink in-progress remove
- **THEN** exit 非 0,stderr 列出 touched 記錄的檔案清單並說明需以人工或 agent 判斷處理;meta 與 touched 記錄皆不變

##### Example: 證據清單為兩版記錄的聯集去重

- **GIVEN** touched 記錄 v1 清單含 src/a.rs 與 src/b.ts,v2 清單含 src/b.ts 與 src/c.rs
- **WHEN** 對該 change 執行 speclink in-progress remove
- **THEN** stderr 的檔案清單恰為 src/a.rs、src/b.ts、src/c.rs 三項,無重複

#### Scenario: 未開工的變更冪等成功

- **WHEN** 對一個 meta 無任何 started_* 欄位的 change 執行 speclink in-progress remove
- **THEN** exit 0,不寫入任何檔案

#### Scenario: 不存在的變更明確報錯

- **WHEN** 對不存在的 change 名稱執行 speclink in-progress remove
- **THEN** exit 非 0,stderr 報找不到該 change

#### Scenario: meta 損毀時 fail-closed

- **WHEN** 一個 change 的 .openspec.yaml 無法解析,對其執行 speclink in-progress remove
- **THEN** exit 非 0 報錯,不修改任何檔案

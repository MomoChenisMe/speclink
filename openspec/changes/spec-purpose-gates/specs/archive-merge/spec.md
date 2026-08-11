## MODIFIED Requirements

### Requirement: 新 capability 的 Purpose 自 delta 帶入

<!-- BEFORE: delta 未提供 Purpose 時沿用佔位骨架（靜默寫入 TBD 佔位句） -->

建立新正典規格時，引擎 SHALL 於 delta 檔含 Purpose 區段（需求操作區塊之外的獨立段落）時將其內容複製為新正典的 Purpose。delta 的 Purpose 不合格——缺 Purpose 區段、內容為空、或 trim 後不足 50 字元（合格判準與 change 驗證共用單一定義）——時，單筆封存 SHALL 拒絕並回報該 capability 與不合格原因（維持零檔案效果的 fail-closed 語意），SHALL NOT 寫入佔位骨架放行。既有 capability 的正典 Purpose SHALL NOT 被 delta 的 Purpose 區段改動，且既有 capability 的 delta 帶 Purpose SHALL NOT 構成封存拒絕理由（忽略不報錯）。skip_specs 封存不觸發此守門。

#### Scenario: delta 提供 Purpose

- **WHEN** 新 capability 的 delta 檔頂部含合格 Purpose 區段，封存通過
- **THEN** 新建正典規格的 Purpose 為該區段內容，非占位文字

#### Scenario: 既有正典 Purpose 不受 delta 影響

- **WHEN** 既有 capability 的 delta 檔含 Purpose 區段，封存通過
- **THEN** 該 capability 正典的 Purpose 維持原樣

#### Scenario: 新 capability 缺 Purpose 封存被拒

- **WHEN** 某 change 的 delta 新開一個正典尚無的 capability 且 delta 檔無 Purpose 區段，執行單筆封存
- **THEN** 封存拒絕、非零收尾，stderr 指出該 capability 與缺 Purpose 的不合格原因；change 目錄與正典規格零檔案變動

#### Scenario: 新 capability 的 Purpose 過短封存被拒

- **WHEN** 新開 capability 的 delta 檔含 Purpose 區段但內容 trim 後不足 50 字元，執行單筆封存
- **THEN** 封存拒絕並回報不足門檻的不合格原因；零檔案變動

## MODIFIED Requirements

### Requirement: 封存合併 fail-closed 守門

<!-- BEFORE: 拒絕情形封閉列舉為 (1)–(6)，補救動線只有 drift → ingest 一套 -->

封存套用 delta 至正典時，引擎 SHALL 於以下任一情形拒絕封存：（1）ADDED 需求名已存在於正典；（2）MODIFIED／REMOVED／RENAMED 的來源需求名不存在於正典；（3）同一需求名出現在同一 delta 的多個操作區段（含 RENAMED 的 FROM／TO 與其他區段互撞）；（4）RENAMED 目標名已存在於正典；（5）MODIFIED 區塊缺正典既有 scenario 且未附刪除聲明；（6）正典不存在的 capability 出現 ADDED 以外的操作；（7）正典尚無該 capability 且 delta 的 Purpose 不合格（缺 `## Purpose` 區段、內容為空、或不足最低字元門檻）。拒絕 SHALL 聚合全部違規一次回報，每條列明 capability、操作、需求名與原因，並附補救動線指引——(1)–(6) 的過期類指向先執行 drift、再以 ingest 更新 delta；(7) 的 Purpose 類指向補寫 `## Purpose` 區段並以 validate 取得完整指引，SHALL NOT 對 Purpose 類僅給 drift → ingest 動線。此守門為 correctness 級：SHALL NOT 提供任何旁路旗標；--no-validate SHALL 維持只略過文件驗證、不解鎖合併守門；--skip-specs SHALL 維持整段跳過規格套用的既有語意。

#### Scenario: 過期 ADDED 被拒絕

- **WHEN** delta 的 ADDED 需求名已存在於正典規格，執行 speclink archive
- **THEN** 封存以非零 exit code 拒絕，錯誤列明該 capability、ADDED、需求名與「已存在於正典」原因，並附 drift → ingest 補救指引

#### Scenario: 缺目標的 MODIFIED 被拒絕

- **WHEN** delta 的 MODIFIED 來源需求名不存在於正典，執行 speclink archive
- **THEN** 封存拒絕並點名該需求；REMOVED 與 RENAMED 缺來源時同樣拒絕

#### Scenario: 多區段互撞被拒絕

- **WHEN** 同一需求名同時出現在 delta 的兩個操作區段（例如 MODIFIED 與 REMOVED，或 RENAMED 的 FROM 與 REMOVED）
- **THEN** 封存拒絕並列明互撞的操作組合

#### Scenario: 新 capability 僅接受 ADDED

- **WHEN** 正典尚不存在該 capability，而 delta 含 MODIFIED、REMOVED 或 RENAMED 操作
- **THEN** 封存拒絕；現行「正典不存在時 MODIFIED 物化成新規格」的行為不再發生

#### Scenario: 違規聚合一次回報

- **WHEN** 同一 change 的 delta 含多條違規（跨 capability 或跨操作）
- **THEN** 單次執行即回報全部違規清單，而非僅首條

#### Scenario: no-validate 不解鎖守門

- **WHEN** 對含違規 delta 的 change 執行 speclink archive --no-validate
- **THEN** 文件驗證被略過但合併守門照常拒絕

#### Scenario: 新 capability 缺 Purpose 的違規呈現三處一致

- **WHEN** 新開 capability 的 delta 缺合格 Purpose，分別執行 speclink drift、批次 speclink archive --all 與單筆 speclink archive
- **THEN** 三處回報同一違規（PURPOSE／`## Purpose`／帶「archive would refuse it」語意的同一原因字串）；批次預檢的略過原因點名缺 `## Purpose` 的 capability；單筆封存的補救指引指向補寫 `## Purpose` 與 validate、而非 drift → ingest；drift 的主建議指向 validate 的 Purpose 指引、而非 ingest

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

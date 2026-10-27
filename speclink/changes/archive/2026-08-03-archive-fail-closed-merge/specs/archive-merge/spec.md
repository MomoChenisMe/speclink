## Purpose

封存時 delta 併入正典規格的合併引擎語意：fail-closed 守門清單、兩階段合併計畫與新 capability 的 Purpose 帶入。正典規格是現況唯一真相，本 capability 保證任何進入正典的寫入都經過完整驗證，過期或自相矛盾的 delta 在零檔案效果下被拒絕。

## ADDED Requirements

### Requirement: 封存合併 fail-closed 守門

封存套用 delta 至正典時，引擎 SHALL 於以下任一情形拒絕封存：（1）ADDED 需求名已存在於正典；（2）MODIFIED／REMOVED／RENAMED 的來源需求名不存在於正典；（3）同一需求名出現在同一 delta 的多個操作區段（含 RENAMED 的 FROM／TO 與其他區段互撞）；（4）RENAMED 目標名已存在於正典；（5）MODIFIED 區塊缺正典既有 scenario 且未附刪除聲明；（6）正典不存在的 capability 出現 ADDED 以外的操作。拒絕 SHALL 聚合全部違規一次回報，每條列明 capability、操作、需求名與原因，並附補救動線指引（先執行 drift、再以 ingest 更新 delta）。此守門為 correctness 級：SHALL NOT 提供任何旁路旗標；--no-validate SHALL 維持只略過文件驗證、不解鎖合併守門；--skip-specs SHALL 維持整段跳過規格套用的既有語意。

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

### Requirement: 兩階段合併計畫與零半套寫入

封存 SHALL 先讀取全部 capability 的 delta 與正典、完成全部驗證並產生合併計畫，全數通過後才進入寫入階段；驗證階段的任何違規 SHALL 使封存在零檔案效果下結束——正典未動、無新 snapshot、change 目錄不移動。寫入階段 SHALL 依「全部 snapshot 備份 → 全部正典寫回 → change 移入封存區」的順序執行，使寫入中途的 I/O 失敗必可由已落地的 snapshot 與 Git 恢復。

#### Scenario: 任一 capability 違規則全部不寫

- **WHEN** change 涉及兩個 capability，其一 delta 合法、另一含違規，執行 speclink archive
- **THEN** 兩個正典規格皆未被修改、無任何 snapshot 產生、change 仍在進行區原位

#### Scenario: snapshot 先於正典寫入

- **WHEN** 全部驗證通過進入寫入階段
- **THEN** 所有受影響正典的封存前備份先寫入 snapshot 目錄，之後才寫回正典並移動 change

### Requirement: MODIFIED 的 scenario 保全與明示刪除聲明

MODIFIED 套用前，引擎 SHALL 驗證正典目標需求的每個 scenario 名稱皆出現在 delta 區塊中，或以刪除聲明註解（MODIFIED 區塊內一行一個的 REMOVED-SCENARIO 註解，格式與既有 BEFORE 審閱註解同層）明示放棄；名稱比對 SHALL 與需求名同語意（trim 後完全相等），解析前 SHALL 將 CRLF 正規化為 LF。缺 scenario 且未聲明 SHALL 拒絕並逐條點名遺失的 scenario 名。聲明註解 SHALL 於寫入正典前剝除。

#### Scenario: 漏抄 scenario 被拒絕並點名

- **WHEN** 正典目標需求含 scenario「逾時重試」與「離線佇列」，delta 的 MODIFIED 區塊僅含「逾時重試」且無刪除聲明
- **THEN** 封存拒絕，錯誤點名遺失的「離線佇列」

#### Scenario: 明示聲明後允許刪除

- **WHEN** delta 的 MODIFIED 區塊含「逾時重試」與一行 REMOVED-SCENARIO 註解聲明放棄「離線佇列」
- **THEN** 封存通過，合併後的正典需求含「逾時重試」、不含「離線佇列」、也不含聲明註解本身

### Requirement: 新 capability 的 Purpose 自 delta 帶入

建立新正典規格時，引擎 SHALL 於 delta 檔含 Purpose 區段（需求操作區塊之外的獨立段落）時將其內容複製為新正典的 Purpose；delta 未提供 Purpose 時 SHALL 沿用現行占位骨架。既有 capability 的正典 Purpose SHALL NOT 被 delta 的 Purpose 區段改動。

#### Scenario: delta 提供 Purpose

- **WHEN** 新 capability 的 delta 檔頂部含 Purpose 區段，封存通過
- **THEN** 新建正典規格的 Purpose 為該區段內容，非占位文字

#### Scenario: 既有正典 Purpose 不受 delta 影響

- **WHEN** 既有 capability 的 delta 檔含 Purpose 區段，封存通過
- **THEN** 該 capability 正典的 Purpose 維持原樣

### Requirement: 過期判定單源共用

drift 的 Specs 維度、bulk archive 的 readiness 預檢與單筆 archive 的合併守門 SHALL 共用同一過期判定實作；三處對同一 delta 的過期認定 SHALL 一致，drift 與 bulk 預檢的 reason 文案 SHALL 表述拒絕語意（archive 將拒絕，而非跳過）。

#### Scenario: 三處判定一致

- **WHEN** 同一 change 的 delta 含一條過期 MODIFIED，分別執行 speclink drift、bulk archive 預檢與單筆 speclink archive
- **THEN** 三處皆認定該操作過期：drift 列為 spec assumption、bulk 預檢列為未就緒、單筆 archive 拒絕，且三者指向同一 capability 與需求名

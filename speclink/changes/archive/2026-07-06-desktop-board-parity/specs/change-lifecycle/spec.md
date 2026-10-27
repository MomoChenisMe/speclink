## ADDED Requirements

### Requirement: in-progress 標記真相存於 change meta

in-progress 標記 SHALL 以 change 目錄的 .openspec.yaml 為唯一真相：執行 speclink in-progress add 某 change 後，該檔 SHALL 含 started_at（ISO 日期）；started_by 與 started_with SHALL 依 created_by／created_with 的同一身分機制寫入——呼叫端可歸屬者寫入、不可歸屬者缺席（CLI 以 git 身分供 started_by；CLI 現無 agent 識別來源，started_with 缺席，寫入縫留在引擎函式的 agent 參數，desktop／remote 通道屆時供給）。既有欄位（schema、created_*、from_discussion 等）SHALL 原樣保留。指令 SHALL NOT 建立 .git/speclink-app/ 目錄或其下任何檔案。重複執行 SHALL 冪等——已有 started_* 時欄位值不變。本需求為 parity 敏感：指令的 stdout、stderr 與 exit code SHALL 與遷移前的行為位元級一致（首次與重複執行皆然）；對不存在 change 的行為 SHALL 維持遷移前實測基線——靜默成功（無輸出、exit 0），且 SHALL NOT 寫入任何檔案。

#### Scenario: 標記後 meta 含三站中的開工欄位

- **WHEN** 對含 created_* 欄位的 change 執行 speclink in-progress add 該 change
- **THEN** 該 change 的 .openspec.yaml 新增 started_at 與 started_by（git 身分可得時），created_* 與 schema 欄位逐字元保留，且 .git/speclink-app/ 未被建立；經引擎函式帶 agent 識別呼叫時另含 started_with

#### Scenario: 重複標記冪等

- **WHEN** 對已含 started_* 的 change 再次執行 speclink in-progress add
- **THEN** 三欄位值與首次標記後完全相同（保留首次開工蓋章），stdout 與 exit code 與首次執行一致

#### Scenario: 不存在的 change 行為不變

- **WHEN** 對不存在的 change 名執行 speclink in-progress add
- **THEN** 與遷移前版本一致地靜默成功——無輸出、exit 0（遷移前實測基線：名稱不驗證），且無任何檔案被寫入（無 meta 變動、無 .git/speclink-app/）

### Requirement: 歸檔保留完整生命週期歸屬

speclink archive 一個已標記開工的 change 後，封存目錄的 .openspec.yaml SHALL 同時含 created／created_by（建立站）、started_at／started_by／started_with（開工站）與 archived_at／archived_by（歸檔站）——started_* 欄位 SHALL NOT 於歸檔時被剝除或改寫。

#### Scenario: 歸檔後三站欄位並存

- **WHEN** 對 meta 含 created_* 與 started_* 的 change 執行 speclink archive
- **THEN** changes/archive/ 下該 change 的 .openspec.yaml 同時含三站全部欄位，started_* 的值與歸檔前逐字元一致

### Requirement: meta 新欄位向後相容

change meta 的解析 SHALL 對缺少 started_* 欄位的既有檔案維持既有行為：所有讀取 meta 的指令與查詢 SHALL 正常運作、該 change 視為未開工，SHALL NOT 產生任何警告或錯誤。

#### Scenario: 舊 meta 檔正常解析且視為未開工

- **WHEN** 對 meta 僅含 schema 與 created_* 欄位（無 started_*）的 change 執行 speclink list --json 與 speclink status --change 該 change
- **THEN** 兩指令輸出與遷移前版本位元級一致，exit code 為 0，無警告

## ADDED Requirements

### Requirement: 線上變更回應的關卡欄位

變更清單回應的每個項目與單筆變更的回應 SHALL 各增列選填的 `review` 與 `verify` 物件（camelCase，缺席時不序列化）：`state`（字串：`none`、`inProgress`、`stamped`）；state 為 `stamped` 時另有 `stampedAt`（字串）、`rounds`（整數）、`tasksTotal`（整數），以及有值才出現的 `stampedBy`、`stampedWith`、`commit`（字串）。單筆變更的回應在 `stamped` 時 SHALL 另含 `scope`（陣列，每項 `path` 與 `hash`）；清單的項目 SHALL NOT 含 `scope`。判定 SHALL 為：該關卡的工單存在 → `inProgress`；否則 metadata 有完整的章 → `stamped`；否則 → `none`。Server SHALL NOT 回報「蓋章後內容是否變動」：這個判定 SHALL 由有工作樹的用戶端以 `tasksTotal` 與 `scope` 自行完成。已封存清單的每個項目 SHALL 增列選填的 `reviewOutcome` 與 `verifyOutcome`（字串：`none`、`stamped`、`notPassed`——封存的文件集含該關卡的工單而沒有章）。變更清單的項目 SHALL 增列選填的 `branch` 與 `landedCommit`；單筆變更的回應 SHALL 增列選填的 `branch`、`landedCommit`、`landedAt`、`landedBy`。用戶端 SHALL 把不認得的 state 值讀作未知而不報錯。這些欄位 SHALL 全部是選填：不認得它們的舊版用戶端 SHALL 照常解析回應，舊版 Server 的回應沒有它們時新版用戶端 SHALL 把關卡讀作沒有資料。CLI 於 remote 模式 SHALL NOT 把 `review`、`verify`、`branch`、`landedCommit` 帶進 `speclink list --json`。

#### Scenario: 審查中的清單項

- **WHEN** 變更 demo 有審查工單而沒有章，請求變更清單
- **THEN** demo 的項目含 `"review":{"state":"inProgress"}`，沒有 stampedAt

#### Scenario: 已蓋章的清單項不帶 scope

- **WHEN** demo 的審查已蓋章（3 輪、12 個任務、帶 commit），請求變更清單
- **THEN** demo 的 review 含 state `stamped`、stampedAt、stampedBy、rounds 3、tasksTotal 12 與 commit，沒有 scope 鍵

#### Scenario: 單筆回應帶 scope

- **WHEN** 請求 demo 的單筆變更回應
- **THEN** review 另含 scope 陣列，每項有 path 與 hash，筆數等於蓋章時記錄的檔案數

#### Scenario: 兩個關卡各自獨立

- **WHEN** demo 的審查已蓋章而驗證有未結的工單，請求變更清單
- **THEN** review 的 state 是 `stamped`，verify 的 state 是 `inProgress`

#### Scenario: 已封存的結局

- **WHEN** 一個變更帶著未結的驗證工單被封存，另一個兩關都蓋章後封存，請求已封存清單
- **THEN** 前者的 verifyOutcome 是 `notPassed`，後者的 reviewOutcome 與 verifyOutcome 都是 `stamped`

#### Scenario: 舊版用戶端解析新回應

- **WHEN** 以新增這些欄位之前的清單項與單筆回應型別，解析含全部新欄位的回應
- **THEN** 解析成功，既有欄位的值不變

#### Scenario: 新版用戶端解析舊回應

- **WHEN** 新版用戶端解析沒有任何新欄位的清單回應
- **THEN** 解析成功，每個項目的關卡讀作沒有資料

#### Scenario: 不認得的 state

- **WHEN** 用戶端解析 `"review":{"state":"paused"}`
- **THEN** 解析成功，state 讀作未知

#### Scenario: CLI 的 list --json 不帶新欄位

- **WHEN** 於 remote 模式對含已蓋章變更的 Server 執行 speclink list --json
- **THEN** 輸出沒有 review、verify、branch、landedCommit 鍵，形狀與 fs 模式相同

### Requirement: 落地回報的請求與回應

Client Protocol SHALL 定義落地回報：對專案底下 changes/<name>/landed 的 POST。請求 payload：`commit`（字串，必填）與選填的 `branch`（字串）。成功回應：`change`、`commit` 與有值才出現的 `branch`，並帶新的範圍版本。這是寫入請求，SHALL 受與其他寫入相同的權限檢查。commit 或 branch 的形狀不合 SHALL 以 invalid_argument 拒絕；變更不存在 SHALL 以 not_found 拒絕。蓋章的請求 SHALL 增列選填的 `headCommit`（字串），開工標記的請求 SHALL 增列選填的 `branch`（字串）；兩者缺席時 Server SHALL 照既有行為處理。這些型別 SHALL 在 protocol 的單一定義處宣告。

#### Scenario: 回報成功

- **WHEN** 對 changes/demo/landed 送出 `{"commit":"0123456789abcdef0123456789abcdef01234567","branch":"speclink/demo"}`
- **THEN** 回應是 200，payload 含 change、commit 與 branch，並帶新的範圍版本

#### Scenario: 沒有寫入權限

- **WHEN** 只有讀取權限的成員送出落地回報
- **THEN** 回應是權限錯誤；metadata 不變

#### Scenario: commit 形狀不合

- **WHEN** 送出 `{"commit":"zzz"}`
- **THEN** 回應的 reason 為 invalid_argument

#### Scenario: 蓋章請求帶 headCommit

- **WHEN** 蓋章的請求含 `"headCommit":"0123456789ab"`，蓋章成功後請求該變更的單筆回應
- **THEN** 該關卡的 commit 是 `0123456789ab`

#### Scenario: 蓋章請求沒有 headCommit

- **WHEN** 舊版用戶端送出不含 headCommit 的蓋章請求
- **THEN** 蓋章照常成功，該關卡沒有 commit 欄位

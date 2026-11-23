## ADDED Requirements

### Requirement: 已封存清單的 delta capability 欄位

已封存清單回應的每個項目 SHALL 增列選填的 `deltaCapabilities`（camelCase、字串陣列、帶序列化預設值、空陣列時不序列化）：封存的文件裡有 delta spec 的 capability 名稱，依名稱遞增。它的個數 SHALL 等於同一個項目的 `specCount`。既有欄位的名稱、型別與值 SHALL 不變。不認得這個欄位的舊版用戶端 SHALL 照常解析回應；舊版 Server 的回應沒有它時，新版用戶端 SHALL 把它讀作空陣列，SHALL NOT 因此失敗，也 SHALL NOT 為了補上它而對每個項目另外發請求。

#### Scenario: 項目帶出 capability 名稱

- **WHEN** 已封存變更 `2026-11-22-add-two` 對 auth 與 billing 各有一份 delta spec，請求已封存清單
- **THEN** 這一項含 `"deltaCapabilities":["auth","billing"]`，`specCount` 是 2

#### Scenario: 沒有 delta spec 的項目不帶這個欄位

- **WHEN** 已封存變更 `2026-11-10-old-one` 沒有任何 delta spec，請求已封存清單
- **THEN** 這一項沒有 `deltaCapabilities` 鍵，`specCount` 是 0

#### Scenario: 新舊版本互相解析

- **WHEN** 以新增這個欄位之前的項目型別解析含 `deltaCapabilities` 的回應，再以新版的型別解析沒有這個欄位的回應
- **THEN** 兩次解析都成功；後者的 `deltaCapabilities` 是空陣列，既有欄位的值不變

## MODIFIED Requirements

### Requirement: 討論資訊 payload 增選填 promotedTo 欄位

<!-- BEFORE: promotedTo 只由 server 在 route 邊緣組裝，引擎的討論輸出不得有這個欄位 -->

DiscussionInfo SHALL 增選填 promotedTo 欄位（camelCase、serde default、空清單即省略），值為該討論已轉出／已併入的 change 名稱清單，順序沿 frontmatter promoted_to 的累加順序。引擎的討論輸出（`discuss list --json` 的每一項與 `discuss show --json` 的 `info`）SHALL 以同名的 `promotedTo` 鍵帶出同一份清單；線上格式的值與順序 SHALL 與它相同。舊 server 不送時 client SHALL 以空清單容錯，SHALL NOT 據此推論討論未轉出以外的狀態。

#### Scenario: promotedTo 序列化與缺席容錯

- **WHEN** 序列化一筆 promotedTo 含兩個 change 名的 DiscussionInfo，再序列化一筆空清單的
- **THEN** 前者 JSON 含 promotedTo 且順序保持；後者無 promotedTo 鍵；反序列化無此鍵的舊 payload 不失敗且值為空清單

#### Scenario: 線上格式與動詞輸出同值

- **WHEN** 討論 two 依序轉出 add-two 與 add-three，分別讀取討論清單的回應，以及連到同一台 Server 執行 `speclink discuss list --json`
- **THEN** 兩邊 two 的 `promotedTo` 都是 `["add-two", "add-three"]`

### Requirement: 線上變更回應的關卡欄位

<!-- BEFORE: 關卡物件在 inProgress 時只有 state，rounds 只在 stamped 時出現 -->

變更清單回應的每個項目與單筆變更的回應 SHALL 各增列選填的 `review` 與 `verify` 物件（camelCase，缺席時不序列化）：`state`（字串：`none`、`inProgress`、`stamped`）；state 為 `stamped` 時另有 `stampedAt`（字串）、`rounds`（整數）、`tasksTotal`（整數），以及有值才出現的 `stampedBy`、`stampedWith`、`commit`（字串）；state 為 `inProgress` 時 SHALL 另有選填的 `rounds`（整數）：工單目前的輪數，工單無法解析時 SHALL 不出現。單筆變更的回應在 `stamped` 時 SHALL 另含 `scope`（陣列，每項 `path` 與 `hash`）；清單的項目 SHALL NOT 含 `scope`。判定 SHALL 為：該關卡的工單存在 → `inProgress`；否則 metadata 有完整的章 → `stamped`；否則 → `none`。Server SHALL NOT 回報「蓋章後內容是否變動」：這個判定 SHALL 由有工作樹的用戶端以 `tasksTotal` 與 `scope` 自行完成。已封存清單的每個項目 SHALL 增列選填的 `reviewOutcome` 與 `verifyOutcome`（字串：`none`、`stamped`、`notPassed`）。結局的判定 SHALL 與進行中的變更同一個順序：封存的文件集含該關卡的工單 → `notPassed`；否則 metadata 有章 → `stamped`；否則 → `none`。工單與章並存時（蓋章後重開工單，再帶著未結的工單封存）SHALL 是 `notPassed`：重開的工單取代了舊章。變更清單的項目 SHALL 增列選填的 `branch` 與 `landedCommit`；單筆變更的回應 SHALL 增列選填的 `branch`、`landedCommit`、`landedAt`、`landedBy`。用戶端 SHALL 把不認得的 state 值讀作未知而不報錯。這些欄位 SHALL 全部是選填：不認得它們的舊版用戶端 SHALL 照常解析回應，舊版 Server 的回應沒有它們時新版用戶端 SHALL 把關卡讀作沒有資料。CLI 於 remote 模式 SHALL NOT 把 `review`、`verify`、`branch`、`landedCommit` 帶進 `speclink list --json`。

#### Scenario: 審查中的清單項

- **WHEN** 變更 demo 有兩輪的審查工單而沒有章，請求變更清單
- **THEN** demo 的項目含 `"review":{"state":"inProgress","rounds":2}`，沒有 stampedAt

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

#### Scenario: 蓋章後重開工單再封存

- **WHEN** 一個變更的審查蓋章後又開了新的一輪審查（審查工單存在，章還在 metadata），帶著這張未結的工單被封存，請求已封存清單
- **THEN** 它的 reviewOutcome 是 `notPassed`

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

#### Scenario: 單筆回應的進行中關卡也帶輪數

- **WHEN** 變更 demo 有三輪的驗證工單而沒有章，請求 demo 的單筆變更回應
- **THEN** verify 是 `{"state":"inProgress","rounds":3}`

#### Scenario: 新版用戶端解析沒有輪數的進行中關卡

- **WHEN** 新版用戶端解析 `"review":{"state":"inProgress"}`
- **THEN** 解析成功，state 是 `inProgress`，輪數讀作沒有資料

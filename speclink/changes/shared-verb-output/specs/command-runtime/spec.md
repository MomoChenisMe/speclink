## ADDED Requirements

### Requirement: 動詞的輸出只有一份

每個經命令執行層的動詞，把結果轉成輸出的定義 SHALL 只有一份，CLI 與 Node SDK dispatch SHALL 共用它。這份定義 SHALL 是不產生副作用的函式：SHALL NOT 直接寫 stdout 或 stderr，SHALL NOT 讀取終端機狀態、環境變數或程序層級的顏色開關；是否上色與是否輸出 --json SHALL 由呼叫端以參數指定。CLI 的本機模式與遠端模式 SHALL 經同一份定義產生輸出。CLI 每個動詞在人眼、--json、--no-color 與強制上色四種形式下的 stdout、stderr 與 exit code SHALL 維持位元級一致。有 --json 形式的動詞，Node SDK dispatch 回傳的結構化物件 SHALL 與 CLI --json 輸出解析後的內容相同。

#### Scenario: CLI 的輸出不變

- **WHEN** 於這個變更前後，在同一份 workspace 內容上對同一個動詞以人眼、--json、--no-color 三種形式執行
- **THEN** stdout、stderr 逐位元一致，exit code 相同

#### Scenario: dispatch 與 CLI 的 --json 同源

- **WHEN** 對同一個 workspace 分別執行 speclink validate demo --json 與 engine.dispatch(['validate', 'demo'])
- **THEN** dispatch 解析出的物件與 CLI 的 stdout 經 JSON 解析後的值相等

#### Scenario: 輸出不帶色碼除非呼叫端要求

- **WHEN** 呼叫端以不上色的參數取得 list 的人眼輸出
- **THEN** 回傳的文字不含任何 ANSI 色碼，內容與 speclink list --no-color 的 stdout 相同

#### Scenario: 輸出的定義不直接印出

- **WHEN** 在沒有終端機的程序裡取得任一動詞的輸出
- **THEN** 該程序的 stdout 與 stderr 沒有被寫入任何內容，輸出以回傳值交給呼叫端

#### Scenario: 兩種模式同一份輸出

- **WHEN** 對語意相同的內容分別以本機模式與遠端模式執行 speclink discuss list
- **THEN** 兩者的 stdout 逐位元一致

## MODIFIED Requirements

### Requirement: workflow-config show 動詞

<!-- BEFORE: show 顯示政策五欄、context 與 rules；--json 欄位為 locale、specLocale、tdd、audit、worktree、context、rules -->

CLI SHALL 提供 speclink workflow-config show（旗標 --json 與 --no-color，無 stdin），顯示 openspec/config.yaml 的正典內容：政策五欄（locale、spec_locale、tdd、audit、worktree——未設定的欄位標示為未設定與其預設語意）、context（有無與行數）、rules（各 artifact 節的條數）與 review（預設清單模式與三個嚴重度清單的條數）。show SHALL 顯示正典值，SHALL NOT 應用環境變數的覆寫（有效值的三層解析屬 instructions payload 職責）。帶 --json 時 SHALL 輸出 camelCase payload：locale、specLocale（未設定為 null）、tdd、audit、worktree（布林）、context（字串或 null）、rules（artifact id 對規則字串陣列的物件）、review（物件：baseline 為字串 "extend" 或 "replace"，critical、warning、suggestion 為規則字串陣列；檔案未寫 review 區塊時為 baseline "extend" 與三個空陣列）。人眼輸出 SHALL 於 rules 行之後、以與其他行相同的標籤欄寬印出 review 行：baseline 為 extend（含未設定）且三個清單皆空時值為 none，否則值為「<baseline>, critical <n>, warning <n>, suggestion <n>」。fs 模式讀取 openspec/config.yaml；remote 模式 SHALL 經既有連線讀取 server 端 config 文件，人眼與 --json 輸出形狀與 fs 模式一致。config 文件無法解析時 SHALL 沿用既有 fail-closed 行為：非零 exit code、stderr 指出檔案與解析原因。本動詞為周邊設定動詞（與 config 同類），SHALL NOT 進入命令執行層。成功 exit code 0、輸出至 stdout；--no-color 下無 ANSI 色彩。本次修訂為刻意變更：人眼輸出於 rules 行之後新增 review 一行、--json 新增 review 欄位；其餘各行與既有欄位 SHALL 維持既有位元級輸出。

#### Scenario: fs 模式顯示正典值

- **WHEN** openspec/config.yaml 含 locale: tw、tdd: true 與 context，執行 speclink workflow-config show
- **THEN** stdout 顯示 locale 為 tw、tdd 開啟、audit 未設定（預設關閉）、worktree 未設定（預設關閉）、context 存在；exit code 0

#### Scenario: --json payload 形狀

- **WHEN** 執行 speclink workflow-config show --json
- **THEN** stdout 為 JSON，欄位 locale、specLocale、tdd、audit、worktree、context、rules、review 一律 camelCase；未設定的 specLocale 為 null、未設定的 tdd 為 false、未設定的 worktree 為 false、未寫 review 區塊時 review 為 baseline "extend" 與 critical、warning、suggestion 三個空陣列

#### Scenario: show 不應用環境變數覆寫

- **WHEN** 設定 SPECLINK_TDD=false 且 openspec/config.yaml 含 tdd: true，執行 speclink workflow-config show --json
- **THEN** payload 的 tdd 為 true（正典值，非解析後有效值）

#### Scenario: remote 模式輸出形狀一致

- **WHEN** 於 remote 綁定的 workspace 執行 speclink workflow-config show --json
- **THEN** stdout 的 JSON 欄位名與 fs 模式完全一致，值來自 server 端 config 文件

#### Scenario: 壞 config fail-closed

- **WHEN** openspec/config.yaml 含 YAML 語法錯誤，執行 speclink workflow-config show
- **THEN** exit code 非 0，stderr 指出該檔與解析原因，stdout 無 payload

#### Scenario: 人眼輸出的 review 行

- **WHEN** 執行 speclink workflow-config show --no-color
- **THEN** stdout 於 rules 行之後恰有一行 review，標籤欄寬與 rules 行相同，值依 review 區塊內容而定；其餘各行與本次修訂前逐位元一致

##### Example: review 行的值

| config.yaml 的 review 區塊 | review 行的值 |
| -------------------------- | ------------- |
| 未寫 review | none |
| critical 兩條、warning 一條、suggestion 一條，未寫 baseline | extend, critical 2, warning 1, suggestion 1 |
| baseline: replace、三個清單皆空 | replace, critical 0, warning 0, suggestion 0 |

#### Scenario: --json 帶出 review 內容

- **WHEN** openspec/config.yaml 含 review 區塊 baseline: replace 與 critical 一條「所有 SQL 一律用參數化查詢」，執行 speclink workflow-config show --json
- **THEN** payload 的 review 為 {"baseline":"replace","critical":["所有 SQL 一律用參數化查詢"],"warning":[],"suggestion":[]}；exit code 0

## ADDED Requirements

### Requirement: config 的 review 區塊與嚴格解析

openspec/config.yaml SHALL 支援頂層鍵 review，供審查關卡的專案規則使用。子鍵限四個：baseline（值為 extend 或 replace，大小寫敏感；省略等同 extend）與 critical、warning、suggestion（各為規則字串清單；省略為空清單）。review 鍵缺席等同 baseline extend 與三個空清單。review 不屬政策欄位：SHALL NOT 有環境變數覆寫，SHALL NOT 讀取 .speclink.yaml 的同名鍵。

review 區塊出現未知子鍵、baseline 值不是 extend 或 replace、或任一清單不是字串陣列時，引擎 SHALL 將整份 config 視為無法解析並沿用既有 fail-closed：讀取 config 的指令（含 workflow-config show、instructions）以非零 exit code 結束，stderr 指出 openspec/config.yaml 與解析原因，原因 SHALL 點名出錯的鍵或值；SHALL NOT 退回預設值。remote 寫入時 server SHALL 以同一引擎解析拒收，回應 invalid_config，store 中文件與 revision 皆不變。

#### Scenario: 未寫 review 時為預設

- **WHEN** openspec/config.yaml 沒有 review 鍵，執行 speclink workflow-config show --json
- **THEN** exit code 0，payload 的 review 為 baseline "extend" 與三個空陣列

#### Scenario: 子鍵打錯即 fail-closed

- **WHEN** openspec/config.yaml 的 review 區塊含子鍵 critcal（critical 拼錯），執行 speclink workflow-config show
- **THEN** exit code 非 0，stderr 指出 openspec/config.yaml 且原因含 critcal；stdout 無輸出

##### Example: 嚴格解析的拒收面

| review 區塊內容 | 結果 |
| --------------- | ---- |
| critcal: [x] | 拒收，原因點名 critcal |
| baseline: Replace | 拒收，原因點名 baseline 的值 |
| critical: {a: b} | 拒收，原因指出 critical 不是字串陣列 |
| baseline: replace、critical: [x] | 正常讀取 |

#### Scenario: 其他指令同樣 fail-closed

- **WHEN** openspec/config.yaml 的 review 區塊 baseline 值為 keep，執行 speclink instructions proposal --change <name> --json
- **THEN** exit code 非 0，stderr 指出 openspec/config.yaml 與 baseline 的值；不輸出以預設值組成的 payload

#### Scenario: remote 寫入以同一解析拒收

- **WHEN** client 以正確的 expected revision 向 server 提交 review 區塊含子鍵 critcal 的 config 內容
- **THEN** server 回應 invalid_config 並指出解析原因，store 中文件內容與 revision 皆未改變

### Requirement: workflow-config review 寫入

CLI SHALL 提供兩個寫入子指令，皆支援 --dry-run 與 --no-color：

- speclink workflow-config review <critical|warning|suggestion> --stdin：以 stdin 整段代換該清單——一行一條、頭尾空白去除、空行忽略；stdin 為空或只有空白時 SHALL 移除該清單鍵。未帶 --stdin 時 SHALL 以非零 exit code 於 stderr 說明用法，且無檔案效果。
- speclink workflow-config review baseline <extend|replace>：寫 replace 時 SHALL 寫入 baseline: replace；寫 extend 時 SHALL 移除 baseline 鍵（省略即預設）。

寫入後 baseline 鍵缺席且三個清單皆空時，SHALL 移除整個 review 鍵。第一個參數不是 critical、warning、suggestion、baseline 之一，或 baseline 的值不是 extend、replace 時，SHALL 以非零 exit code 拒絕且無檔案效果。寫入 SHALL 僅代換 review 區塊——config.yaml 的其他鍵、註解與空行逐位元保留，review 區塊內的註解不保留；以 YAML 保留字元（反引號、at 符號等）開頭的條目 SHALL 自動加引號，讀回時條目值逐字元一致。--dry-run 行為、remote 模式的讀-改-寫與版本衝突處理、離線與認證失效、壞檔 fail-closed SHALL 與 workflow-config set 政策欄位寫入一致。成功 exit code 0、stdout 單行成功訊息；--no-color 下無 ANSI 色彩。

#### Scenario: 清單整段代換

- **WHEN** config.yaml 的 review.critical 原含一條，執行 speclink workflow-config review critical --stdin 並經 stdin 提供兩行規則與一個空行
- **THEN** exit code 0；review.critical 恰為該兩條（原條目被代換），review 的其他子鍵與 config.yaml 其他鍵逐位元不變；stdout 單行成功訊息

##### Example: 寫入效果與成功訊息

| 操作前 review 區塊 | 指令 | 寫入後 review 區塊 | stdout |
| ------------------ | ---- | ------------------ | ------ |
| 無 review 鍵 | review warning --stdin（一行） | warning 一條 | ✓ review.warning = 1 entries |
| warning 一條 | review warning --stdin（空） | review 鍵被移除 | ✓ review.warning removed |
| critical 一條 | review baseline replace | baseline: replace、critical 一條 | ✓ review.baseline = replace |
| baseline: replace、critical 一條 | review baseline extend | critical 一條（無 baseline 鍵） | ✓ review.baseline = extend (default) |
| baseline: replace、三清單皆空 | review baseline extend | review 鍵被移除 | ✓ review.baseline = extend (default) |

#### Scenario: 未帶 --stdin 說明用法

- **WHEN** 執行 speclink workflow-config review critical（無 --stdin）
- **THEN** exit code 非 0，stderr 說明需以 --stdin 提供內容；openspec/config.yaml 逐位元不變

#### Scenario: 不認得的清單名或 baseline 值

- **WHEN** 分別執行 speclink workflow-config review blocker --stdin 與 speclink workflow-config review baseline keep
- **THEN** 兩者 exit code 皆非 0，stderr 列出合法值；openspec/config.yaml 逐位元不變

#### Scenario: dry-run 印 diff

- **WHEN** 執行 speclink workflow-config review suggestion --stdin --dry-run 並提供一行規則
- **THEN** exit code 0；stdout 為 unified diff，只顯示 review 區塊的變更；openspec/config.yaml 逐位元不變

#### Scenario: 保留字元開頭的條目自動加引號

- **WHEN** 執行 speclink workflow-config review warning --stdin 並提供一行以 @ 開頭的規則
- **THEN** exit code 0；寫出的 config.yaml 可被解析，speclink workflow-config show --json 的 review.warning 含與輸入逐字元一致的該條目

#### Scenario: 壞檔拒寫

- **WHEN** openspec/config.yaml 含 YAML 語法錯誤，執行 speclink workflow-config review critical --stdin
- **THEN** exit code 非 0，stderr 指出該檔與解析原因並說明拒寫；檔案逐位元不變

#### Scenario: remote 寫入經版本檢查

- **WHEN** 於 remote 模式執行 speclink workflow-config review critical --stdin 且無並行改寫
- **THEN** exit code 0；server 端 config 的 review.critical 更新為 stdin 內容；後續 speclink workflow-config show --json 讀回同一內容

#### Scenario: remote 版本衝突

- **WHEN** 於 remote 模式執行 speclink workflow-config review baseline replace，而讀取後到寫入前另一個 client 已改寫 config
- **THEN** exit code 非 0，stderr 說明版本衝突並提示重跑指令；server 端 config 維持另一個 client 寫入的內容

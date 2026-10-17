## MODIFIED Requirements

### Requirement: workflow-config show 動詞
<!-- BEFORE: show 只輸出既有五項政策、context、rules 與 review，沒有安全政策摘要。 -->

CLI SHALL 提供 speclink workflow-config show（旗標 --json 與 --no-color，無 stdin），顯示 openspec/config.yaml 的正典內容：政策五欄（locale、spec_locale、tdd、audit、worktree——未設定的欄位標示為未設定與其預設語意）、context（有無與行數）、rules（各 artifact 節的條數）與 review（預設清單模式與三個嚴重度清單的條數）。show SHALL 顯示正典值，SHALL NOT 應用環境變數的覆寫（有效值的三層解析屬 instructions payload 職責）。帶 --json 時 SHALL 輸出 camelCase payload：locale、specLocale（未設定為 null）、tdd、audit、worktree（布林）、context（字串或 null）、rules（artifact id 對規則字串陣列的物件）、review（物件：baseline 為字串 "extend" 或 "replace"，critical、warning、suggestion 為規則字串陣列；檔案未寫 review 區塊時為 baseline "extend" 與三個空陣列）。人眼輸出 SHALL 於 rules 行之後、以與其他行相同的標籤欄寬印出 review 行：baseline 為 extend（含未設定）且三個清單皆空時值為 none，否則值為「<baseline>, critical <n>, warning <n>, suggestion <n>」。fs 模式讀取 openspec/config.yaml；remote 模式 SHALL 經既有連線讀取 server 端 config 文件，人眼與 --json 輸出形狀與 fs 模式一致。config 文件無法解析時 SHALL 沿用既有 fail-closed 行為：非零 exit code、stderr 指出檔案與解析原因。本動詞為周邊設定動詞（與 config 同類），SHALL NOT 進入命令執行層。成功 exit code 0、輸出至 stdout；--no-color 下無 ANSI 色彩。本次安全審查修訂 SHALL 保留 review 行與欄位，刻意於 review 行之後新增 security_verification 摘要行與 JSON securityVerification 物件；除此之外的各行與既有欄位 SHALL 維持既有位元級輸出。安全物件 SHALL 含 enabled:boolean、scope:string、standards:object、reviews:object、disabledRules:string[]、excludeDirs:string[]，其預設與值域依安全政策區塊需求；四標準皆含 enabled:boolean 與 version:string|null，ASVS 另含 level:string，reviews 含 spec／code／apiBoundary 布林。

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
- **THEN** stdout 於 rules 行之後恰有一行 review，標籤欄寬與 rules 行相同，值依 review 區塊內容而定；除新增 security_verification 摘要行外，其餘各行與本次修訂前逐位元一致

##### Example: review 行的值

| config.yaml 的 review 區塊 | review 行的值 |
| -------------------------- | ------------- |
| 未寫 review | none |
| critical 兩條、warning 一條、suggestion 一條，未寫 baseline | extend, critical 2, warning 1, suggestion 1 |
| baseline: replace、三個清單皆空 | replace, critical 0, warning 0, suggestion 0 |

#### Scenario: --json 帶出 review 內容

- **WHEN** openspec/config.yaml 含 review 區塊 baseline: replace 與 critical 一條「所有 SQL 一律用參數化查詢」，執行 speclink workflow-config show --json
- **THEN** payload 的 review 為 {"baseline":"replace","critical":["所有 SQL 一律用參數化查詢"],"warning":[],"suggestion":[]}；exit code 0

未設定 locale 與 spec_locale 的人眼標示 SHALL 為 unset (system language)；JSON 仍為 null，SHALL NOT 把偵測值寫回設定。官方新增的 review 行與 JSON 欄位 SHALL 保留。

#### Scenario: 舊設定與安全摘要

- **WHEN** 未寫安全政策，執行 workflow-config show --json 及 --no-color 人眼輸出
- **THEN** JSON securityVerification.enabled=false、scope=off、四標準停用、reviews 三項 true、兩清單空；人眼在 review 後有 security_verification: disabled，舊行內容不變

#### Scenario: 僅一個標準且保留版本

- **WHEN** enabled=true、只啟用 Top 10 2025、ASVS 停用，執行 show --json
- **THEN** securityVerification.standards.owaspTop10 為 enabled=true、version="2025"，ASVS 停用不影響 Top 10，Local／Remote JSON 同形


### Requirement: instructions apply payload 的有效政策欄位
<!-- BEFORE: apply instructions 只帶 tdd／audit 有效政策，未提供安全政策。 -->

speclink instructions apply --change <名> --json 的 payload SHALL 含 tdd 與 audit 兩個布林欄位（camelCase 欄位名 tdd、audit），值 SHALL 為三層解析後的有效政策值（與既有 locale 欄位同一解析入口）。人眼輸出 SHALL 維持既有位元級輸出（不新增顯示行）。fs 模式與 remote 模式的 --json payload 形狀 SHALL 一致；remote 模式的值由 server 端以同一解析入口計算。

remote 模式下 server 回應缺 tdd 或 audit 欄位（版本偏斜：舊 server）時，指令 SHALL 以非零 exit code 失敗且 stderr 說明回應無法解析，SHALL NOT 以預設值補欄位繼續執行——缺欄位默認關閉會靜默停用 TDD 紀律，fail closed 為刻意設計（沿寫碼進度欄位的既有先例）。

#### Scenario: fs 模式 payload 帶有效值

- **WHEN** openspec/config.yaml 設定 tdd: true 且未設定 audit，執行 speclink instructions apply --change 某 change --json
- **THEN** payload 含 "tdd": true 與 "audit": false，exit code 為 0

#### Scenario: 環境變數覆寫反映於 payload

- **WHEN** openspec/config.yaml 設定 tdd: true，設定環境變數 SPECLINK_TDD=false，執行 speclink instructions apply --change 某 change --json
- **THEN** payload 含 "tdd": false

##### Example: tdd 有效值進 payload

| SPECLINK_TDD | openspec/config.yaml | payload 的 tdd |
| ------------ | -------------------- | -------------- |
| 未設定       | tdd: true            | true           |
| 未設定       | 未設定               | false          |
| false        | tdd: true            | false          |
| true         | 未設定               | true           |

#### Scenario: remote 模式形狀一致

- **WHEN** 於 remote 綁定的 workspace 執行 speclink instructions apply --change 某 change --json
- **THEN** payload 含 tdd 與 audit 布林欄位，欄位名與 fs 模式完全一致，值來自 server 端 config 文件的同一三層解析

#### Scenario: 舊 server 缺欄位 fail closed

- **WHEN** remote 模式下 server 回應的 apply instructions payload 不含 tdd 欄位，執行 speclink instructions apply --change 某 change --json
- **THEN** exit code 非 0，stderr 說明回應無法解析，stdout 不輸出 payload

apply 的 JSON payload SHALL 另含 securityVerification 物件，內容為有效政策的 Agent 摘要，SHALL 含 enabled:boolean、scope:string、standards:[{standardId:string,version:string,level:string|null}]、reviews:string[]、executionMode:string、dataAvailability:string、requirements:string[]。standards 只列選用標準，reviews 只列啟用審查；此摘要 SHALL 不與 workflow-config show 的完整政策物件混用，且不包含 disabledRules／excludeDirs。缺設定時 enabled=false；tdd／audit／locale 原有解析與錯誤語意保持。proposal／design／tasks instructions SHALL 亦提供相同欄位供適用要求提示。人眼輸出不額外列政策，不因停用安全政策呼叫掃描。新 client 遇舊 server 未提供安全政策欄位時 SHALL 明示該 server 不支援安全審查；SHALL NOT 擅自以 disabled 掩蓋未知政策，原有非安全流程 SHALL 仍按既有欄位運作。

#### Scenario: 安全政策與 audit 獨立

- **WHEN** 安全主開關啟用、audit=false，取得 apply instructions --json
- **THEN** securityVerification.enabled=true、audit=false，各自保持，不將 audit 強制開啟

#### Scenario: 舊 server 沒有安全能力

- **WHEN** server 的 instructions 沒有 securityVerification 而使用者請求安全審查
- **THEN** 顯示需升級 server 的安全能力缺口，不假稱安全已停用或已通過，既有 TDD／audit 欄位不改值


## ADDED Requirements

### Requirement: 安全政策區塊預設與獨立選項

security_verification SHALL 為單一政策來源，缺席／null 為 enabled=false、scope=off、四標準停用、ASVS level=L1、reviews.spec/code/api_boundary=true、disabled_rules/exclude_dirs 為空。主開關與 scope off SHALL 不同義；標準 SHALL 獨立，Top 10 限2021／2025、API限2019／2023、ASVS限4.0.3／5.0.0與L1／L2／L3，CWE選已安裝版本字串。停用 SHALL 保留已選版本及子設定；啟用標準缺版本、錯型別、非法enum SHALL 報 invalid_config，不退預設。未知既有欄位 SHALL 保留，不當作可執行功能；不新增環境覆寫或 .speclink.yaml 政策來源。

#### Scenario: 各標準互不影響

- **WHEN** 先設定 Top 10 2025 與 ASVS 5.0.0 L2，再只停用 ASVS
- **THEN** Top 10 仍啟用2025，ASVS版本及L2保留但不執行，tdd／audit／worktree值不變

#### Scenario: 非法安全政策

- **WHEN** enabled 是字串、scope=desktop、ASVS level=L4 或啟用標準沒有版本
- **THEN** 載入失敗並指出欄位，不能轉為 disabled 或執行審查

### Requirement: 安全政策部分更新與保值

CLI SHALL 提供 workflow-config security --stdin [--dry-run] [--no-color]，stdin為至多64KiB單一YAML部分更新mapping。空白／非mapping／未知輸入鍵／不合法葉值 SHALL 拒絕且原文不動；缺--stdin exit1，語法錯誤exit2。只改指定葉節點，未觸及內容（包含原文件未知安全子鍵、頂層鍵、註解、tdd／audit／worktree）SHALL 保留。複雜alias／merge／重複鍵無法安全改寫 SHALL 拒寫而非重建整份。成功stdout一行摘要exit0，--dry-run stdout逐行diff且不寫入，錯誤stderr exit1；不新增--json。Remote SHALL 共用改寫、editor授權及expected revision CAS，衝突／離線／401不排隊、不覆蓋。

#### Scenario: 修改一個葉節點保留未知內容

- **WHEN** 原文件含未知 future_option 與其註解，stdin只提供 enabled:false
- **THEN** 只改主開關，future_option、註解、已選版本及其他政策仍原樣保留，重新開啟可還原子設定

#### Scenario: dry-run 與失敗原檔不變

- **WHEN** 對合法patch執行--dry-run，或提供65,537bytes／未知鍵／錯誤型別
- **THEN** 前者只有實際差異且檔案不變，後者stderr錯誤exit1、沒有diff與檔案效果

#### Scenario: Remote 並行修改

- **WHEN** 另一人於讀取與提交間更新政策
- **THEN** exit1並提示重讀重試，不覆蓋他人資料；Desktop保留使用者輸入並走現有逐欄對照

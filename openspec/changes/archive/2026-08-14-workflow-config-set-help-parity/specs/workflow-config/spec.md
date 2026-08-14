## MODIFIED Requirements

### Requirement: workflow-config set 政策欄位寫入

<!-- BEFORE: key 限 locale、spec_locale、tdd、audit 四者、正典插入序只列四鍵，且未規定 CLI help 文字須與接受鍵集合一致 -->

CLI SHALL 提供 speclink workflow-config set <key> <value>（旗標 --dry-run 與 --no-color，無 stdin），寫入政策欄位：key SHALL 限 locale、spec_locale、tdd、audit、worktree 五者，其他 key SHALL 以非零 exit code 拒絕且無檔案效果；tdd、audit 與 worktree 的 value SHALL 僅接受 true 或 false，其他值 SHALL 以非零 exit code 拒絕。locale 的 value SHALL 僅接受語系代碼 tw、ja、en，spec_locale 的 value SHALL 僅接受 tw、ja、en、auto；比對 SHALL 大小寫敏感（TW、Auto 均非法）；空字串（移除鍵、回到未設定＝預設）SHALL 仍合法；其他值——含顯示名稱字串（如「繁體中文」）——SHALL 以非零 exit code 拒絕，stderr SHALL 指出欄位名、收到的值與合法代碼集合，且無任何檔案效果；帶 --dry-run 時非法值 SHALL 同樣拒絕且 SHALL NOT 印出 diff。

set 的 CLI help 文字 SHALL 與實際接受的鍵集合一致：speclink workflow-config set --help 的子指令說明所列的政策鍵 SHALL 為前述五鍵，且逐字等同未知 key 錯誤訊息所列的鍵集合；<value> 參數說明 SHALL 指出僅接受 true 或 false 的三個鍵（tdd、audit、worktree）。speclink workflow-config --help 子指令一覽中 set 一列的說明 SHALL 與 set --help 的子指令說明同源同字面。help 輸出無 --json 面，--no-color 亦 SHALL NOT 改變其內容。此 help 說明行由四鍵改為五鍵屬刻意變更（既有字面漏列 worktree，與接受鍵集合不符）；set 的成功訊息、錯誤訊息、diff 輸出、輸出去向與 exit code SHALL 維持既有位元級輸出。

寫入 SHALL 為文字層 read-modify-write：僅變動目標鍵所在的行或區塊，其餘行——含註解行、空行、未知頂層鍵與 schema 鍵下方的使用者自加內容——SHALL 逐位元保留（模板註解不再喪失）。tdd、audit 或 worktree 設為 false、locale 或 spec_locale 設為空字串時 SHALL 移除該鍵行（維持未設定＝預設語意），SHALL NOT 一併刪除其上方的註解行。新增原先不存在的政策鍵時 SHALL 依正典序（locale、spec_locale、tdd、audit、worktree）以連續區塊插於 schema 鍵行之後，區塊與前後內容行之間 SHALL 各恰一空行（相鄰處原已有空行時不重複補）；schema 鍵不存在時 SHALL 插於檔案最頂端。既有政策鍵 SHALL 原位改值、SHALL NOT 移動位置。寫入結果 SHALL 於落檔前重新解析並與目標狀態逐鍵等值比對，不等值 SHALL fail-closed 拒絕寫入且原檔逐位元不變。

帶 --dry-run 時 SHALL 於 stdout 印出變更前後的 unified diff、SHALL NOT 寫入任何檔案、exit code 0；無變更時 diff 為空。fs 模式寫入 openspec/config.yaml；remote 模式 SHALL 先經連線讀取 server 端 config 現行內容與版本，套用同一改寫、寫回時附帶讀得的版本——server 端版本已前進（他人並行改寫）時 SHALL 以非零 exit code 失敗、stderr 提示重新執行，SHALL NOT 覆蓋他人寫入；版本識別 SHALL NOT 出現在指令介面。連線離線或認證失效時 SHALL 以非零 exit code 失敗並輸出語義化訊息，SHALL NOT 暫存或排隊寫入。config 文件無法解析時 SHALL fail-closed 拒絕寫入（重寫壞檔會毀掉使用者內容）。成功 exit code 0、stdout 單行成功訊息。

#### Scenario: 設定 locale 保留其他鍵

- **WHEN** openspec/config.yaml 含 schema、tdd: true 與 context，執行 speclink workflow-config set locale tw
- **THEN** exit code 0；config.yaml 的 locale 為 tw，schema、tdd、context 的值不變

#### Scenario: 缺鍵插於 schema 之下且空行區隔

- **WHEN** config.yaml 依序含 schema、context、rules 而無任何政策鍵，執行 speclink workflow-config set locale tw
- **THEN** exit code 0；locale: tw 行位於 schema 鍵行之後、context 之前，與前後內容行之間各恰一空行；context 與 rules 的所有行逐位元不變

#### Scenario: 註解與空行逐位元保留

- **WHEN** config.yaml 含模板註解與使用者自加的註解行，執行任一合法的 set 寫入
- **THEN** exit code 0；除目標鍵所在行與插入區塊外，檔案所有行（含全部註解與空行）逐位元不變

#### Scenario: 檔尾既有鍵原位改值不搬家

- **WHEN** config.yaml 的 locale 鍵位於檔案最尾（rules 之後），執行 speclink workflow-config set locale ja
- **THEN** exit code 0；locale 行仍在檔尾原位、值改為 ja，其餘行逐位元不變

#### Scenario: schema 缺席時插於檔案最頂端

- **WHEN** config.yaml 無 schema 鍵且無政策鍵，執行 speclink workflow-config set tdd true
- **THEN** exit code 0；tdd: true 位於檔案最頂端，與後續內容之間恰一空行

#### Scenario: 內部改寫驗證失敗拒絕寫入

- **WHEN** 文字層手術產出的結果重新解析後與目標狀態不等值（防呆觸發）
- **THEN** exit code 非 0，stderr 單行錯誤指明內部改寫驗證失敗；openspec/config.yaml 逐位元不變

#### Scenario: 未知 key 拒絕

- **WHEN** 執行 speclink workflow-config set theme dark
- **THEN** exit code 非 0，stderr 指出 key 限 locale、spec_locale、tdd、audit、worktree；openspec/config.yaml 逐位元不變

#### Scenario: set --help 列出全部政策鍵

- **WHEN** 執行 speclink workflow-config set --help
- **THEN** exit code 0；stdout 的子指令說明所列政策鍵為 locale、spec_locale、tdd、audit、worktree，與未知 key 錯誤訊息所列的鍵集合逐字相同

#### Scenario: set --help 標明布林鍵的合法值

- **WHEN** 執行 speclink workflow-config set --help
- **THEN** exit code 0；stdout 的 <value> 參數說明指出 tdd、audit 與 worktree 僅接受 true 或 false

#### Scenario: 父層 help 的 set 說明同源

- **WHEN** 執行 speclink workflow-config --help
- **THEN** exit code 0；子指令一覽中 set 一列的說明與 speclink workflow-config set --help 的子指令說明字面相同（同樣列出五個政策鍵）

#### Scenario: 非法布林值拒絕

- **WHEN** 執行 speclink workflow-config set tdd yes
- **THEN** exit code 非 0，stderr 指出 tdd 僅接受 true 或 false；無任何檔案效果

#### Scenario: 非法 locale 值拒絕

- **WHEN** 執行 speclink workflow-config set locale 繁體中文
- **THEN** exit code 非 0，stderr 指出 locale 欄位、收到的值「繁體中文」與合法代碼 tw、ja、en；openspec/config.yaml 逐位元不變

#### Scenario: 非法值帶 dry-run 同樣拒絕

- **WHEN** 執行 speclink workflow-config set spec_locale 繁體中文 --dry-run
- **THEN** exit code 非 0，stdout 無 diff 輸出，stderr 列出 spec_locale 的合法代碼（tw、ja、en、auto）；無任何檔案效果

##### Example: locale 值域判定

| key | value | 結果 |
| --- | ----- | ---- |
| locale | tw | 成功 |
| locale | 空字串 | 成功（移除鍵） |
| locale | 繁體中文 | 拒絕 |
| locale | TW | 拒絕（大小寫敏感） |
| spec_locale | auto | 成功 |
| spec_locale | zh-Hant | 拒絕 |

#### Scenario: 設 false 移除鍵

- **WHEN** openspec/config.yaml 含 audit: true，執行 speclink workflow-config set audit false
- **THEN** exit code 0；config.yaml 不含 audit 鍵（未設定＝預設關閉），該鍵上方的註解行仍在

#### Scenario: dry-run 印 diff 不落檔

- **WHEN** 執行 speclink workflow-config set locale ja --dry-run
- **THEN** exit code 0；stdout 為 unified diff 且僅含 locale 相關行的變更（不含其他行的重排）；openspec/config.yaml 逐位元不變

#### Scenario: remote 版本衝突提示重跑

- **WHEN** 於 remote 模式執行 speclink workflow-config set tdd true，且 server 端 config 在讀取後、寫回前已被他人改寫
- **THEN** exit code 非 0，stderr 說明設定已被他人更新、請重新執行；server 端內容維持他人寫入的版本

#### Scenario: 壞 config 拒絕寫入

- **WHEN** openspec/config.yaml 含 YAML 語法錯誤，執行 speclink workflow-config set locale tw
- **THEN** exit code 非 0，stderr 指出該檔無法解析、寫入已拒絕；檔案逐位元不變

### Requirement: workflow-config show 動詞

<!-- BEFORE: 需求文字寫「政策四欄（locale、spec_locale、tdd、audit）」，--json payload 欄位清單亦只列 locale、specLocale、tdd、audit、context、rules，未含 worktree -->

CLI SHALL 提供 speclink workflow-config show（旗標 --json 與 --no-color，無 stdin），顯示 openspec/config.yaml 的正典內容：政策五欄（locale、spec_locale、tdd、audit、worktree——未設定的欄位標示為未設定與其預設語意）、context（有無與行數）與 rules（各 artifact 節的條數）。show SHALL 顯示正典值，SHALL NOT 應用環境變數或 .speclink.yaml 舊鍵的覆寫（有效值的四層解析屬 instructions payload 職責）。帶 --json 時 SHALL 輸出 camelCase payload：locale、specLocale（未設定為 null）、tdd、audit、worktree（布林）、context（字串或 null）、rules（artifact id 對規則字串陣列的物件）。fs 模式讀取 openspec/config.yaml；remote 模式 SHALL 經既有連線讀取 server 端 config 文件，人眼與 --json 輸出形狀與 fs 模式一致。config 文件無法解析時 SHALL 沿用既有 fail-closed 行為：非零 exit code、stderr 指出檔案與解析原因。本動詞為周邊設定動詞（與 config 同類），SHALL NOT 進入命令執行層。成功 exit code 0、輸出至 stdout；--no-color 下無 ANSI 色彩。本次修訂為正典文字校正：人眼與 --json 兩條輸出的實際內容早已含 worktree 欄位，SHALL 維持既有位元級輸出，SHALL NOT 因本次修訂而改變。

#### Scenario: fs 模式顯示正典值

- **WHEN** openspec/config.yaml 含 locale: tw、tdd: true 與 context，執行 speclink workflow-config show
- **THEN** stdout 顯示 locale 為 tw、tdd 開啟、audit 未設定（預設關閉）、worktree 未設定（預設關閉）、context 存在；exit code 0

#### Scenario: --json payload 形狀

- **WHEN** 執行 speclink workflow-config show --json
- **THEN** stdout 為 JSON，欄位 locale、specLocale、tdd、audit、worktree、context、rules 一律 camelCase；未設定的 specLocale 為 null、未設定的 tdd 為 false、未設定的 worktree 為 false

#### Scenario: show 不應用環境變數覆寫

- **WHEN** 設定 SPECLINK_TDD=false 且 openspec/config.yaml 含 tdd: true，執行 speclink workflow-config show --json
- **THEN** payload 的 tdd 為 true（正典值，非解析後有效值）

#### Scenario: remote 模式輸出形狀一致

- **WHEN** 於 remote 綁定的 workspace 執行 speclink workflow-config show --json
- **THEN** stdout 的 JSON 欄位名與 fs 模式完全一致，值來自 server 端 config 文件

#### Scenario: 壞 config fail-closed

- **WHEN** openspec/config.yaml 含 YAML 語法錯誤，執行 speclink workflow-config show
- **THEN** exit code 非 0，stderr 指出該檔與解析原因，stdout 無 payload

### Requirement: init 範本的政策寫入位置

<!-- BEFORE: 範本示例區只規定 locale、spec_locale、tdd、audit 四鍵，且未規定覆寫提示行須列出 SPECLINK_* 環境變數名 -->

speclink init SHALL 將政策欄位的示例寫入 openspec/config.yaml 範本（locale、spec_locale、tdd、audit、worktree 的註解示例區），該範本 SHALL 於覆寫提示行列出五個對應的環境變數名（SPECLINK_LOCALE、SPECLINK_SPEC_LOCALE、SPECLINK_TDD、SPECLINK_AUDIT、SPECLINK_WORKTREE），且 .speclink.yaml 範本 SHALL NOT 含任何政策鍵（僅 tools 與 spec_dir 相關內容）。本次修訂為正典文字校正：生成的範本內容早已含 worktree 與 SPECLINK_WORKTREE，SHALL 維持既有位元級輸出，SHALL NOT 因本次修訂而改變。

#### Scenario: 新專案初始化的範本內容

- **WHEN** 於空目錄執行 speclink init
- **THEN** 生成的 openspec/config.yaml 含 locale、spec_locale、tdd、audit、worktree 的註解示例，且覆寫提示行同時列出 SPECLINK_LOCALE、SPECLINK_SPEC_LOCALE、SPECLINK_TDD、SPECLINK_AUDIT、SPECLINK_WORKTREE；生成的 .speclink.yaml 不含此五鍵；exit code 為 0

#### Scenario: 既有專案不受範本變更影響

- **WHEN** 於已初始化且 .speclink.yaml 含舊政策鍵的專案執行 speclink update
- **THEN** 兩個設定檔內容不被改寫（update 不觸碰設定檔），僅 stderr 出現 deprecation 警告

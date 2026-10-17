# workflow-config Specification

## Purpose

工作流政策設定的語意：locale、spec_locale、tdd、audit、worktree 五個欄位以 openspec/config.yaml 為正典，有效值依環境變數 ＞ .speclink.yaml 舊鍵 ＞ 正典檔 ＞ 內建預設四層解析，並由 workflow-config show／set 讀寫政策欄位與 context、rules。本 capability 保證政策值有唯一可預期的解析結果、舊鍵命中時出 deprecation 警告，正典檔無法解析時 fail closed 而非退回預設。

## Requirements

### Requirement: init 範本的政策寫入位置

speclink init SHALL 將政策欄位的示例寫入 openspec/config.yaml 範本（locale、spec_locale、tdd、audit、worktree 的註解示例區），該範本 SHALL 於覆寫提示行列出五個對應的環境變數名（SPECLINK_LOCALE、SPECLINK_SPEC_LOCALE、SPECLINK_TDD、SPECLINK_AUDIT、SPECLINK_WORKTREE），且 .speclink.yaml 範本 SHALL NOT 含任何政策鍵（僅 tools 與 spec_dir 相關內容）。語言預設的註解 SHALL 明示 locale 與 spec_locale 未設定時使用系統語系，spec_locale 的舊 auto 語意維持跟隨 locale；此註解由 English 改為 system language 是刻意輸出變更，其餘範本結構 SHALL 保留。

#### Scenario: 新專案初始化的範本內容

- **WHEN** 於空目錄執行 speclink init
- **THEN** 生成的 openspec/config.yaml 含 locale、spec_locale、tdd、audit、worktree 的註解示例，且覆寫提示行同時列出 SPECLINK_LOCALE、SPECLINK_SPEC_LOCALE、SPECLINK_TDD、SPECLINK_AUDIT、SPECLINK_WORKTREE；生成的 .speclink.yaml 不含此五鍵；exit code 為 0

#### Scenario: 既有專案不受範本變更影響

- **WHEN** 於已初始化且 .speclink.yaml 含舊政策鍵的專案執行 speclink update
- **THEN** 兩個設定檔內容不被改寫（update 不觸碰設定檔），stderr 無任何 deprecation 警告

#### Scenario: 新專案語言預設說明

- **WHEN** 於空目錄執行 speclink init --tools copilot
- **THEN** exit code 為 0，stdout 為既有初始化摘要；config.yaml 的 locale 與 spec_locale 註解預設均為 system language，未寫入任何實際語言鍵；.speclink.yaml 只記錄工具與綁定設定


<!-- @trace
source: copilot-project-tool
updated: 2026-10-10T08:18:29+08:00
-->

---
### Requirement: workflow-config show 動詞

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


<!-- @trace
source: add-owasp-cwe-preliminary-review
updated: 2026-10-17T16:55:59+08:00
-->

---
### Requirement: workflow-config set 政策欄位寫入

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


<!-- @trace
source: workflow-config-set-help-parity
updated: 2026-08-14
-->

---
### Requirement: workflow-config context 與 rules 寫入

CLI SHALL 提供 speclink workflow-config context --stdin 與 speclink workflow-config rules <artifact> --stdin（各支援 --dry-run 與 --no-color）：context SHALL 以 stdin 全文設定 config.yaml 的 context 鍵，內容僅空白時 SHALL 移除該鍵；rules SHALL 整節代換指定 artifact 的規則清單——stdin 一行一條、空行忽略，stdin 為空時 SHALL 移除該 artifact 節；artifact SHALL 限目前 schema 的 artifact id（如 proposal、specs、design、tasks），未知 id SHALL 以非零 exit code 拒絕且無檔案效果。兩個子指令未帶 --stdin 時 SHALL 以非零 exit code 說明用法。寫入的保留語意、--dry-run 行為、remote 模式的讀-改-寫與版本衝突處理、離線與認證失效、壞檔 fail-closed，SHALL 與 workflow-config set 政策欄位寫入一致。成功 exit code 0、stdout 單行成功訊息。

#### Scenario: context 設定多行內容

- **WHEN** 執行 speclink workflow-config context --stdin 並經 stdin 提供多行專案說明
- **THEN** exit code 0；config.yaml 的 context 為該多行內容（YAML block scalar），政策四欄與 rules 不變

#### Scenario: 空白 stdin 移除 context

- **WHEN** config.yaml 含 context，執行 speclink workflow-config context --stdin 且 stdin 僅含空白
- **THEN** exit code 0；config.yaml 不含 context 鍵

#### Scenario: rules 整節代換

- **WHEN** 執行 speclink workflow-config rules design --stdin 並經 stdin 提供三行規則
- **THEN** exit code 0；config.yaml 的 rules.design 恰為該三條（原節整份代換），其他 artifact 節不變

#### Scenario: 未知 artifact 拒絕

- **WHEN** 執行 speclink workflow-config rules blueprint --stdin
- **THEN** exit code 非 0，stderr 指出 artifact id 不在目前 schema；無任何檔案效果

#### Scenario: rules dry-run 印 diff

- **WHEN** 執行 speclink workflow-config rules tasks --stdin --dry-run 並提供新規則
- **THEN** exit code 0；stdout 為 unified diff 顯示 rules.tasks 節的變更；openspec/config.yaml 逐位元不變

#### Scenario: remote context 寫入經版本檢查

- **WHEN** 於 remote 模式執行 speclink workflow-config context --stdin 且無並行改寫
- **THEN** exit code 0；server 端 config 的 context 更新為 stdin 內容；後續 workflow-config show 讀回同一內容

<!-- @trace
source: workflow-config-verb-and-skill
updated: 2026-07-27
code:
  - crates/speclink-cli/src/commands.rs
  - crates/speclink-cli/src/main.rs
  - crates/speclink-cli/src/remote_commands.rs
  - crates/speclink-cli/tests/workflow_config.rs
  - crates/speclink-core/assets/skills/config.md
  - crates/speclink-core/src/skills.rs
  - crates/speclink-core/tests/golden/claude.snapshot.md
  - crates/speclink-core/tests/golden/codex.snapshot.md
  - crates/speclink-core/tests/golden/neutral-cli.snapshot.md
  - crates/speclink-core/tests/golden/neutral-tool-call.snapshot.md
  - docs/configuration.md
  - docs/configuration.zh-TW.md
-->

---
### Requirement: worktree 欄位寫入的技能同步與關閉擋下

speclink workflow-config set worktree <value> SHALL 於 config 寫入成功後同步技能足跡（生成與清理範圍等同 speclink update）。由 true 改 false 時，若 local workspace 存在活躍 linked worktree（分支慣例 speclink/<change名> 映射到未封存 change），SHALL 拒絕寫入：exit code 非 0，stderr 逐列每個活躍 worktree 的 change 名、分支與路徑並提示先收尾（worktree-merge），openspec/config.yaml SHALL 維持位元組不變、技能足跡不動。git 不可用時視為無活躍 worktree（fail-open，沿用 discovery 慣例）。config 寫入成功但技能同步失敗時，寫入仍成立（config 為正典），SHALL 於 stderr 回報同步錯誤並提示重跑 speclink update 重建足跡。workflow-config show 的人眼與 --json 輸出形狀不變（worktree 欄位既已存在，此處為維持既有輸出的相容性聲明）。

#### Scenario: set true 寫入並注入技能

- **WHEN** 於無活躍 worktree 的專案執行 speclink workflow-config set worktree true
- **THEN** exit code 0，openspec/config.yaml 的 worktree 鍵為 true，各工具 skills 目錄出現兩顆 worktree 技能，stdout 確認寫入與同步結果

#### Scenario: set false 無活躍 worktree 時寫入並清理

- **WHEN** worktree: true 且無活躍 linked worktree 的專案執行 speclink workflow-config set worktree false
- **THEN** exit code 0，config 鍵改為 false，兩顆 worktree 技能目錄被移除

#### Scenario: set false 遇活躍 worktree 拒絕

- **WHEN** 存在分支 speclink/add-auth 的 linked worktree 且 add-auth 為未封存 change 時執行 speclink workflow-config set worktree false
- **THEN** exit code 非 0，stderr 列出 add-auth 的 change 名、分支 speclink/add-auth 與 worktree 路徑並提示先收尾，openspec/config.yaml 位元組不變，技能足跡不動

#### Scenario: 技能同步失敗時 config 寫入仍成立

- **WHEN** set worktree true 的 config 寫入成功、技能足跡生成因檔案系統故障失敗
- **THEN** openspec/config.yaml 的 worktree 鍵已為 true，exit code 非 0 且 stderr 報同步錯誤並提示重跑 speclink update

<!-- @trace
source: worktree-toggle-and-guards
updated: 2026-08-05
-->

---
### Requirement: 工作流政策的正典歸屬與三層解析順序

工作流政策欄位（locale、spec_locale、tdd、audit、worktree）的正典值 SHALL 儲存於 openspec/config.yaml（經儲存介面讀取）。有效值 SHALL 依下列順序解析，先命中者勝：SPECLINK_LOCALE／SPECLINK_SPEC_LOCALE／SPECLINK_TDD／SPECLINK_AUDIT／SPECLINK_WORKTREE 環境變數 ＞ openspec/config.yaml ＞ 預設（locale 與 spec_locale 未設定時各自採用執行 Speclink 的作業系統語言，tdd 與 audit 與 worktree＝false）。.speclink.yaml 的同名鍵 SHALL 一律不生效且不產生任何警告（五鍵一致）。布林環境變數僅接受 true 或 false，其他值 SHALL 視為未設定並落到下一層。

workflow-config set SHALL 接受 worktree 鍵（值 true 或 false，非法值的錯誤行為與既有政策鍵一致：非零 exit code、stderr 說明）；workflow-config show 的人眼輸出與 --json payload SHALL 呈現 worktree 欄位（camelCase 欄位名 worktree，布林）。

openspec/config.yaml 檔案存在但無法解析（YAML 語法錯誤或型別不符）時，讀取政策的指令 SHALL 以非零 exit code 失敗，stderr SHALL 指出該檔的 workspace 相對路徑與解析原因；SHALL NOT 以內建預設或解析順序中其他層的值繼續執行。檔案不存在時 SHALL 沿用內建預設。此 fail-closed 行為為刻意設計。

相對前身「四層解析」的刻意變更：.speclink.yaml 舊鍵層移除後，含政策鍵的 .speclink.yaml 由「鍵生效＋stderr 一行 deprecation 警告」改為「鍵不生效、stderr 無警告」；其他已明確設定的政策值與 JSON 結構 SHALL 保留；本次追加將未設定語言由固定英文改為系統語言，屬刻意行為變更。

系統語言的中文 tag（zh，含 zh-TW、zh_Hant_TW.UTF-8）SHALL 對應 tw，日文 tag（ja）SHALL 對應 ja；英文、其他語言、C locale 或無法取得語言時 SHALL 使用 en。語言段比對 SHALL 不分大小寫。locale 與 spec_locale SHALL 各自取預設，未設定的 spec_locale SHALL NOT 因 locale 明確設定而改為跟隨它；舊 spec_locale:auto SHALL 維持跟隨有效 locale。明確的 en SHALL 覆蓋非英文系統預設。

Local 模式 SHALL 使用執行 CLI／Host 的本機語言；Remote 模式 SHALL 使用執行命令的 server 語言，維持既有 server 政策邊界，不以 client 的系統語言或本機覆寫代替團隊政策。App 介面語言偏好 SHALL NOT 參與工作流語言解析。偵測值 SHALL 只參與有效政策計算，SHALL NOT 寫回 config 或新增政策鍵；unsupported／偵測失敗 SHALL 靜默落 en，配置壞檔仍 SHALL fail-closed。

#### Scenario: 正典值生效

- **WHEN** openspec/config.yaml 設定 tdd: true，執行 speclink instructions tasks --change 某 change --json
- **THEN** payload 反映 tdd 開關為開啟（tasks 指引含 TDD 紀律內容），stderr 無任何警告

#### Scenario: .speclink.yaml 政策鍵一律不生效

- **WHEN** .speclink.yaml 設定 locale: tw 與 tdd: true 而 openspec/config.yaml 設定 locale: ja 且未設定 tdd，執行 speclink instructions proposal --change 某 change --json
- **THEN** payload 的 locale 欄位為 Japanese (日本語)、tdd 有效值為 false（內建預設），stderr 無任何警告，exit code 為 0

#### Scenario: 環境變數覆寫正典值

- **WHEN** 設定環境變數 SPECLINK_TDD=false，而 openspec/config.yaml 設定 tdd: true，執行 speclink instructions tasks --change 某 change --json
- **THEN** payload 反映 tdd 開關為關閉

#### Scenario: 非法布林環境變數落到下一層

- **WHEN** 設定 SPECLINK_AUDIT=yes（非法值），openspec/config.yaml 設定 audit: true，執行任一讀取政策的指令
- **THEN** 有效 audit 值為 true（環境變數被忽略，不輸出錯誤）

#### Scenario: 壞 config.yaml 一律 fail-closed

- **WHEN** openspec/config.yaml 含 YAML 語法錯誤，執行 speclink instructions tasks --change 某 change --json
- **THEN** exit code 非 0，stderr 指出 openspec/config.yaml 與解析原因，stdout 不輸出 instructions payload

#### Scenario: 環境變數不得繞過壞檔

- **WHEN** openspec/config.yaml 含 YAML 語法錯誤，且設定 SPECLINK_TDD=true，執行 speclink instructions tasks --change 某 change --json
- **THEN** 指令仍以非零 exit code 失敗（環境變數不使壞檔被忽略）

#### Scenario: 缺檔沿用內建預設

- **WHEN** openspec/config.yaml 不存在，執行 speclink instructions proposal --change 某 change --json
- **THEN** payload 以內建預設政策生成，exit code 為 0

#### Scenario: worktree 欄位寫入與呈現

- **WHEN** 執行 speclink workflow-config set worktree true 後執行 speclink workflow-config show --json
- **THEN** set 以 exit code 0 結束且 openspec/config.yaml 含 worktree: true；show 的 payload 含 "worktree": true

#### Scenario: worktree 非法值報錯

- **WHEN** 執行 speclink workflow-config set worktree yes
- **THEN** exit code 非 0，stderr 說明合法值為 true 或 false，openspec/config.yaml 未被改動

#### Scenario: SPECLINK_WORKTREE 覆寫檔案值

- **WHEN** openspec/config.yaml 設定 worktree: false，設定環境變數 SPECLINK_WORKTREE=true，執行讀取政策的指令
- **THEN** 有效 worktree 值為 true

#### Scenario: 兩個語言未設定時採用系統語系

- **WHEN** 系統語言為 zh-TW，config 未設定 locale、spec_locale，且無相應環境覆寫，執行 speclink instructions proposal 與 specs --change 某 change --json
- **THEN** exit code 均為 0；proposal 的 locale:string 為 Traditional Chinese (繁體中文)，specs 指引明示繁中及保持英文結構標記；stderr 無新警告，config 的兩鍵仍未設定

##### Example: 系統語言對應

| 系統語言 | artifact 語言 | spec 散文語言 |
| --- | --- | --- |
| zh-TW | 繁體中文 | 繁體中文 |
| zh_Hant_TW.UTF-8 | 繁體中文 | 繁體中文 |
| ja-JP | 日本語 | 日本語 |
| en-US | English | English |
| de-DE | English | English |
| C 或取不到語言 | English | English |

#### Scenario: 明確語言優先於系統預設

- **WHEN** 系統語言為 zh-TW，config 明確設定 locale:en、spec_locale:en，執行 proposal 與 specs 的 instructions --json
- **THEN** 兩者採英文；再以 SPECLINK_LOCALE=ja、SPECLINK_SPEC_LOCALE=ja 執行 Local 指令時，有效語言均改為日文；JSON 欄位名、型別及 config 檔案不變

#### Scenario: 未設定 spec 語言獨立於 locale

- **WHEN** 系統語言為 zh-TW、config locale:ja 而 spec_locale 未設定，執行 proposal 與 specs 的 instructions --json
- **THEN** artifact 為日文、spec 散文為繁中；若原 config 改為 spec_locale:auto，spec 散文才跟隨 locale 為日文

#### Scenario: 遠端使用執行 server 的語言

- **WHEN** Remote Store 的 config 未設定語言，server 系統語言為 ja-JP，client 系統語言為 zh-TW，經既有連線取得 proposal 與 specs 指引
- **THEN** 有效 artifact 及 spec 語言均為日文，不採 client 語言；canonical config 的語言仍未設定；既有 revision／離線／認證失效回應保持原契約


<!-- @trace
source: copilot-project-tool
updated: 2026-10-10T08:18:29+08:00
-->

---
### Requirement: instructions apply payload 的有效政策欄位

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


<!-- @trace
source: add-owasp-cwe-preliminary-review
updated: 2026-10-17T16:55:59+08:00
-->

---
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

<!-- @trace
source: review-rules-config
updated: 2026-10-14T16:37:17+08:00
-->

---
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

<!-- @trace
source: review-rules-config
updated: 2026-10-14T16:37:17+08:00
-->

---
### Requirement: 審查規則寫入後的技能檔過期提示

speclink workflow-config review <critical|warning|suggestion> --stdin 與 speclink workflow-config review baseline <extend|replace> 實際寫入成功後，CLI SHALL 以技能檔過期探測（workspace-tools「技能檔過期探測」）檢查本機工作區的技能檔：探測整體不是較新、也不是無法判定，且逐工具結果中至少一個工具判為過期時，CLI SHALL 於 stderr 輸出恰一行以 `speclink: warning: ` 開頭、不帶 ANSI 色彩的提示——內容 SHALL 列出每個過期工具的名稱與其技能版號、引擎現版號，說明舊版審查技能可能不套用審查規則，並指示執行 speclink update。stdout SHALL 維持既有的單行成功訊息，exit code SHALL 為 0。

以下情況 SHALL NOT 輸出該提示：所有工具的技能檔為現版或整組缺失；探測整體為較新（工作區檔案領先引擎，update 會降級）或無法判定；帶 --dry-run（沒有寫入，也 SHALL NOT 探測）；workflow-config set、context、rules 的寫入（它們由 CLI 在指令注入時讀取，技能檔新舊不影響生效）。remote 模式寫入成功後 SHALL 以本機工作區——解析 store 模式時找到的工作區——的技能檔判定，條件與本機模式相同。探測失敗 SHALL NOT 改變寫入結果與 exit code；提示只是診斷，寫入不回滾。

#### Scenario: 技能檔過期時寫入後提示

- **GIVEN** .speclink.yaml 的 tools 為 claude，.claude/skills/ 下技能檔的 frontmatter 版號舊於引擎現版
- **WHEN** 執行 speclink workflow-config review warning --stdin 並提供一行規則
- **THEN** exit code 0；review.warning 已寫入；stdout 恰為單行成功訊息；stderr 恰有一行以 `speclink: warning: ` 開頭的提示，含 claude、該技能版號、引擎現版號與 speclink update

##### Example: 提示行

| 工作區技能檔 | 引擎現版 | stderr 提示行 |
| ------------ | -------- | ------------- |
| claude v1.44.0 | v1.45.0 | speclink: warning: skill files are older than this engine (claude v1.44.0; engine v1.45.0) — an older review skill may not apply these rules; run `speclink update` |
| claude v1.44.0、codex v1.43.0 | v1.45.0 | speclink: warning: skill files are older than this engine (claude v1.44.0, codex v1.43.0; engine v1.45.0) — an older review skill may not apply these rules; run `speclink update` |

#### Scenario: 一個工具缺失、另一個過期時仍提示過期的工具

- **GIVEN** tools 為 claude 與 codex，.claude/skills/ 技能版號舊於現版，.agents/skills/ 下無任何 speclink- 技能檔（探測整體為缺失）
- **WHEN** 執行 speclink workflow-config review baseline replace
- **THEN** exit code 0；stderr 的提示列出 claude 與其技能版號，不列 codex

#### Scenario: 技能檔現版或整組缺失時不提示

- **WHEN** 分別在技能檔為現版、以及 tools 宣告 claude 但 .claude/skills/ 下無任何 speclink- 技能檔的工作區，執行 speclink workflow-config review critical --stdin 並提供一行規則
- **THEN** 兩者 exit code 0、stdout 單行成功訊息；stderr 都不含 `speclink: warning: skill files are older` 提示

#### Scenario: 工作區較新時不提示

- **GIVEN** 工作區技能檔版號新於引擎現版
- **WHEN** 執行 speclink workflow-config review suggestion --stdin 並提供一行規則
- **THEN** exit code 0；stderr 不含過期提示

#### Scenario: dry-run 與其他寫入子指令不提示

- **GIVEN** 工作區技能檔版號舊於引擎現版
- **WHEN** 分別執行 speclink workflow-config review warning --stdin --dry-run、speclink workflow-config set tdd true、speclink workflow-config rules tasks --stdin
- **THEN** 三者 stderr 都不含過期提示；dry-run 的 openspec/config.yaml 逐位元不變

#### Scenario: remote 模式以本機工作區判定

- **GIVEN** remote 模式的工作區，.speclink.yaml 的 tools 含 claude，本機 .claude/skills/ 技能版號舊於引擎現版
- **WHEN** 執行 speclink workflow-config review critical --stdin 並寫入 server 成功
- **THEN** exit code 0；stdout 單行成功訊息；stderr 含列出 claude 的過期提示

<!-- @trace
source: workflow-config-write-feedback
updated: 2026-10-14T21:40:15+08:00
-->

---
### Requirement: dry-run 差異只標示實際改動的行

workflow-config 各寫入子指令（set、context、rules、review）帶 --dry-run 時於 stdout 印出的 unified diff SHALL 為單一 hunk：以 `--- a/<label>`、`+++ b/<label>` 兩行開頭，接一行 `@@ -<舊側起點,行數> +<新側起點,行數> @@` 標頭，hunk 前後各帶至多三行 context。hunk 內的行 SHALL 依新舊文字的最長共同子序列逐行對齊：對齊上的行（未改動且相對順序不變）SHALL 以 context 行（前綴一個空白）呈現，SHALL NOT 以成對的 `-` 與 `+` 出現；其餘只在舊文字一側的行 SHALL 以 `-` 開頭，只在新文字一側的行 SHALL 以 `+` 開頭——被刪除、新增或移動位置的行才帶 `-`／`+`；同一位置同時有刪除與新增時，`-` 行 SHALL 先於 `+` 行。標頭的舊側行數 SHALL 等於 hunk 內 context 行數加 `-` 行數，新側行數 SHALL 等於 context 行數加 `+` 行數。新舊文字相同時 SHALL 輸出空字串。只有單一連續改動時，輸出 SHALL 與只標示該段改動的單一 hunk 相同。--dry-run 的其他行為（不寫檔、exit code 0、本機與 remote 模式共用同一改寫）不變。

#### Scenario: 兩處改動之間的未改行以 context 呈現

- **GIVEN** openspec/config.yaml 的 rules.tasks 依序為三條規則「甲」「乙」「丙」
- **WHEN** 執行 speclink workflow-config rules tasks --stdin --dry-run，stdin 依序為「甲二」「乙」「丙」「丁」
- **THEN** exit code 0；stdout 的 diff 中「乙」「丙」兩條只以 context 行出現；`-` 行恰為「甲」那一條，`+` 行恰為「甲二」與「丁」兩條；openspec/config.yaml 逐位元不變

##### Example: rules.tasks 的差異

- **GIVEN** rules.tasks 為 `- 甲`、`- 乙`、`- 丙` 三行
- **WHEN** 以「甲二、乙、丙、丁」整節代換並帶 --dry-run
- **THEN** hunk 內依序為 `-  - 甲`、`+  - 甲二`、`   - 乙`、`   - 丙`、`+  - 丁`，其前後為至多三行未改的 context

#### Scenario: hunk 標頭行數與內容一致

- **WHEN** 對任一寫入子指令帶 --dry-run 產生非空 diff
- **THEN** @@ 標頭的舊側行數等於 hunk 內以空白或 `-` 開頭的行數，新側行數等於以空白或 `+` 開頭的行數

#### Scenario: 單一連續改動的輸出不變

- **WHEN** 執行 speclink workflow-config set locale ja --dry-run（只改 locale 一行）
- **THEN** stdout 的 diff 只有一行 `-` 與一行 `+`，其餘為 context，與改版前輸出位元級相同

<!-- @trace
source: workflow-config-write-feedback
updated: 2026-10-14T21:40:15+08:00
-->

---
### Requirement: 語言預設的技能說明

生成的 config、baseline、propose、ingest 技能及內建 specs 指引 SHALL 描述未設定語言使用系統語系，適用 Claude、Codex／Copilot 及 Custom 共用資產；SHALL NOT 再將未設定 spec_locale 說成固定英文。明確語言設定與舊 auto 的後端語意 SHALL 保留；結構標記及 SHALL/MUST 關鍵字 SHALL 維持英文。生成本文的刻意變更 SHALL 與資產版本、golden 及 assets.lock 同步。

#### Scenario: 各工具語言指引一致

- **WHEN** 為 claude、codex、copilot 或 custom 生成 config、baseline、propose、ingest 技能，或取得內建 specs 指引
- **THEN** 內文明示未設定使用系統語言，既有合法明確代碼及 legacy auto 語意保留，結構標記保持英文；各工具採原生目錄與呼叫語法，語言政策一致

<!-- @trace
source: copilot-project-tool
updated: 2026-10-10T08:18:29+08:00
-->

---
### Requirement: workflow-config languages 只讀查詢

CLI SHALL 提供 speclink workflow-config languages（旗標 --json 與 --no-color，無 stdin）。此查詢 SHALL 將正典 locale／spec_locale 與執行主機 OS 預設解析，不套用 SPECLINK_*，以供 Baseline 在沒有 change 時取得具體語言。JSON SHALL 只含 locale:string（語言顯示名稱）及 specLocale:string（tw／ja／en）；人眼 SHALL 顯示兩個語言值。查詢 SHALL NOT 建立 change、寫入 config 或改變 revision。show 的正典 null 與 JSON 形狀 SHALL 保留。

Remote SHALL 沿用既有 GET /config 的可選 languages 物件，其值由 Server 解析，SHALL NOT 由 Client 環境或 OS 決定。舊回應缺少此欄仍可供原 show 使用；languages 查詢 SHALL 非零結束、stdout 空白、stderr 指示升級 Server，SHALL NOT 猜測 Server 語言。壞 config／離線／認證失效 SHALL 沿用原錯誤。成功 exit 0、stdout 資料；--no-color SHALL 無 ANSI。

#### Scenario: 本機只讀解析且不需要變更

- **WHEN** 本機 config 未設定語言、spec_locale:auto 或明確 locale:tw／spec_locale:ja，且無 change，執行 languages --json
- **THEN** JSON 採正典值與 OS 預設，不受 SPECLINK_* 影響；兩鍵皆有具體語言，config 與檔案樹不被修改；壞 config 時非零退出且無 stdout payload

#### Scenario: 遠端 Baseline 使用 Server 語言

- **WHEN** Server 系統語言為 ja-JP、Client 為 zh-TW 且帶繁中環境覆寫，Server config 的語言皆未設定或 spec_locale:auto，執行 languages --json
- **THEN** 回傳 locale 為 Japanese (日本語)、specLocale 為 ja；不傳送 config 寫入，不建立本機 openspec，Baseline 以此資料產出日文散文

#### Scenario: 舊 Server 語言資料缺席

- **WHEN** 舊 GET /config 回應未含 languages，執行 languages --json
- **THEN** 非零 exit code、stdout 空白、stderr 提示 upgrade；原 workflow-config show --json 仍維持原輸出契約

#### Scenario: 未知舊語言值僅阻擋具體語言查詢

- **WHEN** 既有 config 含 spec_locale:zh-Hant，或 locale:zh-Hant／JA 等未知值，執行 Local 或 Remote languages --json
- **THEN** 非零 exit code、stdout 空白、stderr 指出語言欄位的合法代碼，config／revision 不變；show 與設定頁仍可讀取原未知值，GET /config 保留原文件及 revision、但不提供不合法的 languages；若 Server metadata 本身含不合法 specLocale，Client 在輸出前亦拒絕它

<!-- @trace
source: copilot-project-tool
updated: 2026-10-10T08:18:29+08:00
-->

---
### Requirement: 安全政策區塊預設與獨立選項

security_verification SHALL 為單一政策來源，缺席／null 為 enabled=false、scope=off、四標準停用、ASVS level=L1、reviews.spec/code/api_boundary=true、disabled_rules/exclude_dirs 為空。主開關與 scope off SHALL 不同義；標準 SHALL 獨立，Top 10 限2021／2025、API限2019／2023、ASVS限4.0.3／5.0.0與L1／L2／L3，CWE選已安裝版本字串。停用 SHALL 保留已選版本及子設定；啟用標準缺版本、錯型別、非法enum SHALL 報 invalid_config，不退預設。未知既有欄位 SHALL 保留，不當作可執行功能；不新增環境覆寫或 .speclink.yaml 政策來源。

#### Scenario: 各標準互不影響

- **WHEN** 先設定 Top 10 2025 與 ASVS 5.0.0 L2，再只停用 ASVS
- **THEN** Top 10 仍啟用2025，ASVS版本及L2保留但不執行，tdd／audit／worktree值不變

#### Scenario: 非法安全政策

- **WHEN** enabled 是字串、scope=desktop、ASVS level=L4 或啟用標準沒有版本
- **THEN** 載入失敗並指出欄位，不能轉為 disabled 或執行審查

<!-- @trace
source: add-owasp-cwe-preliminary-review
updated: 2026-10-17T16:55:59+08:00
-->

---
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

<!-- @trace
source: add-owasp-cwe-preliminary-review
updated: 2026-10-17T16:55:59+08:00
-->
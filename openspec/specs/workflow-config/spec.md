# workflow-config Specification

## Purpose

工作流政策設定的語意：locale、spec_locale、tdd、audit、worktree 五個欄位以 openspec/config.yaml 為正典，有效值依環境變數 ＞ .speclink.yaml 舊鍵 ＞ 正典檔 ＞ 內建預設四層解析，並由 workflow-config show／set 讀寫政策欄位與 context、rules。本 capability 保證政策值有唯一可預期的解析結果、舊鍵命中時出 deprecation 警告，正典檔無法解析時 fail closed 而非退回預設。

## Requirements

### Requirement: init 範本的政策寫入位置

speclink init SHALL 將政策欄位的示例寫入 openspec/config.yaml 範本（locale、spec_locale、tdd、audit、worktree 的註解示例區），該範本 SHALL 於覆寫提示行列出五個對應的環境變數名（SPECLINK_LOCALE、SPECLINK_SPEC_LOCALE、SPECLINK_TDD、SPECLINK_AUDIT、SPECLINK_WORKTREE），且 .speclink.yaml 範本 SHALL NOT 含任何政策鍵（僅 tools 與 spec_dir 相關內容）。範本內容 SHALL 維持既有位元級輸出，SHALL NOT 因本次修訂而改變；本次修訂的刻意變更僅在 scenario 層：update 遇含舊政策鍵的 .speclink.yaml 不再輸出 deprecation 警告。

#### Scenario: 新專案初始化的範本內容

- **WHEN** 於空目錄執行 speclink init
- **THEN** 生成的 openspec/config.yaml 含 locale、spec_locale、tdd、audit、worktree 的註解示例，且覆寫提示行同時列出 SPECLINK_LOCALE、SPECLINK_SPEC_LOCALE、SPECLINK_TDD、SPECLINK_AUDIT、SPECLINK_WORKTREE；生成的 .speclink.yaml 不含此五鍵；exit code 為 0

#### Scenario: 既有專案不受範本變更影響

- **WHEN** 於已初始化且 .speclink.yaml 含舊政策鍵的專案執行 speclink update
- **THEN** 兩個設定檔內容不被改寫（update 不觸碰設定檔），stderr 無任何 deprecation 警告


<!-- @trace
source: tdd-switch-apply-wiring
updated: 2026-08-23
-->

---
### Requirement: workflow-config show 動詞

CLI SHALL 提供 speclink workflow-config show（旗標 --json 與 --no-color，無 stdin），顯示 openspec/config.yaml 的正典內容：政策五欄（locale、spec_locale、tdd、audit、worktree——未設定的欄位標示為未設定與其預設語意）、context（有無與行數）與 rules（各 artifact 節的條數）。show SHALL 顯示正典值，SHALL NOT 應用環境變數的覆寫（有效值的三層解析屬 instructions payload 職責）。帶 --json 時 SHALL 輸出 camelCase payload：locale、specLocale（未設定為 null）、tdd、audit、worktree（布林）、context（字串或 null）、rules（artifact id 對規則字串陣列的物件）。fs 模式讀取 openspec/config.yaml；remote 模式 SHALL 經既有連線讀取 server 端 config 文件，人眼與 --json 輸出形狀與 fs 模式一致。config 文件無法解析時 SHALL 沿用既有 fail-closed 行為：非零 exit code、stderr 指出檔案與解析原因。本動詞為周邊設定動詞（與 config 同類），SHALL NOT 進入命令執行層。成功 exit code 0、輸出至 stdout；--no-color 下無 ANSI 色彩。本次修訂為正典文字校正（「四層解析」改「三層解析」並移除 .speclink.yaml 舊鍵字句），show 的人眼與 --json 輸出 SHALL 維持既有位元級輸出，SHALL NOT 因本次修訂而改變。

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


<!-- @trace
source: tdd-switch-apply-wiring
updated: 2026-08-23
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

工作流政策欄位（locale、spec_locale、tdd、audit、worktree）的正典值 SHALL 儲存於 openspec/config.yaml（經儲存介面讀取）。有效值 SHALL 依下列順序解析，先命中者勝：SPECLINK_LOCALE／SPECLINK_SPEC_LOCALE／SPECLINK_TDD／SPECLINK_AUDIT／SPECLINK_WORKTREE 環境變數 ＞ openspec/config.yaml ＞ 內建預設（locale 未設定＝English、tdd 與 audit 與 worktree＝false）。.speclink.yaml 的同名鍵 SHALL 一律不生效且不產生任何警告（五鍵一致）。布林環境變數僅接受 true 或 false，其他值 SHALL 視為未設定並落到下一層。

workflow-config set SHALL 接受 worktree 鍵（值 true 或 false，非法值的錯誤行為與既有政策鍵一致：非零 exit code、stderr 說明）；workflow-config show 的人眼輸出與 --json payload SHALL 呈現 worktree 欄位（camelCase 欄位名 worktree，布林）。

openspec/config.yaml 檔案存在但無法解析（YAML 語法錯誤或型別不符）時，讀取政策的指令 SHALL 以非零 exit code 失敗，stderr SHALL 指出該檔的 workspace 相對路徑與解析原因；SHALL NOT 以內建預設或解析順序中其他層的值繼續執行。檔案不存在時 SHALL 沿用內建預設。此 fail-closed 行為為刻意設計。

相對前身「四層解析」的刻意變更：.speclink.yaml 舊鍵層移除後，含政策鍵的 .speclink.yaml 由「鍵生效＋stderr 一行 deprecation 警告」改為「鍵不生效、stderr 無警告」；其餘層的解析結果與所有輸出 SHALL 維持既有位元級輸出。

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

<!-- @trace
source: tdd-switch-apply-wiring
updated: 2026-08-23
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

<!-- @trace
source: tdd-switch-apply-wiring
updated: 2026-08-23
-->
## REMOVED Requirements

### Requirement: 工作流政策的正典歸屬與四層解析順序

**Reason**: .speclink.yaml 舊政策鍵相容層移除（第一個正式版發布時即無舊鍵使用者，相容層無保護對象），解析層數入名故整條除役。
**Migration**: 由「工作流政策的正典歸屬與三層解析順序」承接全部語意（僅拿掉 .speclink.yaml 舊鍵層）；含舊鍵的 .speclink.yaml 需將鍵搬入 openspec/config.yaml。

### Requirement: 舊政策鍵的 deprecation 警告

**Reason**: 相容層移除後無舊鍵可警告；警告機制整組除役。
**Migration**: 無需遷移——第一個正式版起即無含舊政策鍵的專案；.speclink.yaml 的政策鍵改為一律不生效且無警告。

## ADDED Requirements

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

## MODIFIED Requirements

### Requirement: init 範本的政策寫入位置

speclink init SHALL 將政策欄位的示例寫入 openspec/config.yaml 範本（locale、spec_locale、tdd、audit、worktree 的註解示例區），該範本 SHALL 於覆寫提示行列出五個對應的環境變數名（SPECLINK_LOCALE、SPECLINK_SPEC_LOCALE、SPECLINK_TDD、SPECLINK_AUDIT、SPECLINK_WORKTREE），且 .speclink.yaml 範本 SHALL NOT 含任何政策鍵（僅 tools 與 spec_dir 相關內容）。範本內容 SHALL 維持既有位元級輸出，SHALL NOT 因本次修訂而改變；本次修訂的刻意變更僅在 scenario 層：update 遇含舊政策鍵的 .speclink.yaml 不再輸出 deprecation 警告。

#### Scenario: 新專案初始化的範本內容

- **WHEN** 於空目錄執行 speclink init
- **THEN** 生成的 openspec/config.yaml 含 locale、spec_locale、tdd、audit、worktree 的註解示例，且覆寫提示行同時列出 SPECLINK_LOCALE、SPECLINK_SPEC_LOCALE、SPECLINK_TDD、SPECLINK_AUDIT、SPECLINK_WORKTREE；生成的 .speclink.yaml 不含此五鍵；exit code 為 0

#### Scenario: 既有專案不受範本變更影響

- **WHEN** 於已初始化且 .speclink.yaml 含舊政策鍵的專案執行 speclink update
- **THEN** 兩個設定檔內容不被改寫（update 不觸碰設定檔），stderr 無任何 deprecation 警告

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

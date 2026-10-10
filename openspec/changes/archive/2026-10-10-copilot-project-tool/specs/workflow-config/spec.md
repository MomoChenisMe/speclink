## MODIFIED Requirements

### Requirement: init 範本的政策寫入位置
<!-- BEFORE: 語言範本註解寫固定 English，其他範本結構與政策歸屬相同。 -->

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

### Requirement: workflow-config show 動詞
<!-- BEFORE: show 人眼對未設定語言顯示 English，canonical JSON 仍為 null。 -->

CLI SHALL 提供 speclink workflow-config show（旗標 --json 與 --no-color，無 stdin），顯示 openspec/config.yaml 的正典內容：政策五欄（locale、spec_locale、tdd、audit、worktree——未設定的欄位標示為未設定與其預設語意）、context（有無與行數）與 rules（各 artifact 節的條數）。show SHALL 顯示正典值，SHALL NOT 應用環境變數的覆寫（有效值的三層解析屬 instructions payload 職責）。帶 --json 時 SHALL 輸出 camelCase payload：locale、specLocale（未設定為 null）、tdd、audit、worktree（布林）、context（字串或 null）、rules（artifact id 對規則字串陣列的物件）。fs 模式讀取 openspec/config.yaml；remote 模式 SHALL 經既有連線讀取 server 端 config 文件，人眼與 --json 輸出形狀與 fs 模式一致。config 文件無法解析時 SHALL 沿用既有 fail-closed 行為：非零 exit code、stderr 指出檔案與解析原因。本動詞為周邊設定動詞（與 config 同類），SHALL NOT 進入命令執行層。成功 exit code 0、輸出至 stdout；--no-color 下無 ANSI 色彩。未設定 locale 與 spec_locale 的人眼標示 SHALL 為 unset (system language)，取代舊 English 預設字句；其他人眼格式與 --json 形狀 SHALL 保留。

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

#### Scenario: 未設定語言的人眼與 JSON 呈現

- **WHEN** config 文件未設定 locale、spec_locale，分別執行 speclink workflow-config show --no-color 與 speclink workflow-config show --json
- **THEN** 兩次 exit code 為 0；人眼 stdout 的兩個欄位均顯示 unset (system language) 且無 ANSI；JSON stdout 的 locale、specLocale 均為 null，SHALL NOT 把偵測到的系統語言當作設定值寫入 payload 或 config

### Requirement: 工作流政策的正典歸屬與三層解析順序
<!-- BEFORE: 未設定語言固定為英文；明確政策與壞檔拒絕順序維持。 -->

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

## ADDED Requirements

### Requirement: 語言預設的技能說明

生成的 config、baseline、propose、ingest 技能及內建 specs 指引 SHALL 描述未設定語言使用系統語系，適用 Claude、Codex／Copilot 及 Custom 共用資產；SHALL NOT 再將未設定 spec_locale 說成固定英文。明確語言設定與舊 auto 的後端語意 SHALL 保留；結構標記及 SHALL/MUST 關鍵字 SHALL 維持英文。生成本文的刻意變更 SHALL 與資產版本、golden 及 assets.lock 同步。

#### Scenario: 各工具語言指引一致

- **WHEN** 為 claude、codex、copilot 或 custom 生成 config、baseline、propose、ingest 技能，或取得內建 specs 指引
- **THEN** 內文明示未設定使用系統語言，既有合法明確代碼及 legacy auto 語意保留，結構標記保持英文；Codex／Copilot 仍逐位元共用同一份本文


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

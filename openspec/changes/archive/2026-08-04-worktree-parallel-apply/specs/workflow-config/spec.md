## MODIFIED Requirements

### Requirement: 工作流政策的正典歸屬與四層解析順序

工作流政策欄位（locale、spec_locale、tdd、audit、worktree）的正典值 SHALL 儲存於 openspec/config.yaml（經儲存介面讀取）。有效值 SHALL 依下列順序解析，先命中者勝：SPECLINK_LOCALE／SPECLINK_SPEC_LOCALE／SPECLINK_TDD／SPECLINK_AUDIT／SPECLINK_WORKTREE 環境變數 ＞ .speclink.yaml 的同名舊鍵（相容層，僅限 locale、spec_locale、tdd、audit 四鍵——worktree 無歷史舊鍵，.speclink.yaml 的 worktree 鍵 SHALL 不生效且不產生警告）＞ openspec/config.yaml ＞ 內建預設（locale 未設定＝English、tdd 與 audit 與 worktree＝false）。布林環境變數僅接受 true 或 false，其他值 SHALL 視為未設定並落到下一層。

workflow-config set SHALL 接受 worktree 鍵（值 true 或 false，非法值的錯誤行為與既有政策鍵一致：非零 exit code、stderr 說明）；workflow-config show 的人眼輸出與 --json payload SHALL 呈現 worktree 欄位（camelCase 欄位名 worktree，布林）。

openspec/config.yaml 檔案存在但無法解析（YAML 語法錯誤或型別不符）時，讀取政策的指令 SHALL 以非零 exit code 失敗，stderr SHALL 指出該檔的 workspace 相對路徑與解析原因；SHALL NOT 以內建預設或解析順序中其他層的值繼續執行。檔案不存在時 SHALL 沿用內建預設。此 fail-closed 行為為刻意設計。

#### Scenario: 正典值生效

- **WHEN** .speclink.yaml 不含政策鍵、openspec/config.yaml 設定 tdd: true，執行 speclink instructions tasks --change 某 change --json
- **THEN** payload 反映 tdd 開關為開啟（tasks 指引含 TDD 紀律內容），stderr 無 deprecation 警告

#### Scenario: 舊鍵相容層勝過正典值

- **WHEN** .speclink.yaml 設定 locale: tw 且 openspec/config.yaml 設定 locale: ja，執行 speclink instructions proposal --change 某 change --json
- **THEN** payload 的 locale 欄位為 Traditional Chinese (繁體中文)，且 stderr 出現一行 deprecation 警告

#### Scenario: 環境變數覆寫一切

- **WHEN** 設定環境變數 SPECLINK_TDD=false，而 .speclink.yaml 與 openspec/config.yaml 均設定 tdd: true，執行 speclink instructions tasks --change 某 change --json
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

#### Scenario: .speclink.yaml 的 worktree 鍵不生效

- **WHEN** .speclink.yaml 設定 worktree: true 而 openspec/config.yaml 未設定 worktree，執行讀取政策的指令
- **THEN** 有效 worktree 值為 false（內建預設），stderr 無 deprecation 警告

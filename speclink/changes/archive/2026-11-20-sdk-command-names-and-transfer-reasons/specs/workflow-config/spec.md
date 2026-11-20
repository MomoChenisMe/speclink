## ADDED Requirements

### Requirement: workflow-config set 的說明寫出空值的作用

`speclink workflow-config set --help` 的輸出 SHALL 在 `<VALUE>` 的說明裡寫出：空字串會把 `locale` 或 `spec_locale` 設回未設定。這句說明 SHALL 是英文，含 `an empty value unsets locale or spec_locale` 這段文字；說明文字不隨 `locale` 是 `tw`、`ja`、`en` 或未設定而改變。`--help` SHALL 照舊印到 stdout、exit code 0、不讀 stdin；`--no-color` 下的文字相同。

本需求只改說明文字。`workflow-config set` 的行為 SHALL 維持「workflow-config set 政策欄位寫入」的規定：`locale` 或 `spec_locale` 的值是空字串時移除那個鍵；`tdd`、`audit`、`worktree` 不接受空字串。

#### Scenario: 說明寫出空值的作用

- **WHEN** 執行 `speclink workflow-config set --help`
- **THEN** exit code 是 0；stdout 的 `<VALUE>` 說明含 `an empty value unsets locale or spec_locale`，並照舊含 `tdd/audit/worktree take true or false`

#### Scenario: 空值照舊移除語言的鍵

- **WHEN** 專案的 `locale` 是 `tw`，執行 `speclink workflow-config set locale ''`，之後執行 `speclink workflow-config show --json`
- **THEN** 兩次的 exit code 都是 0；show 的 `locale` 是 `null`

#### Scenario: 布林欄位不接受空值

- **WHEN** 執行 `speclink workflow-config set tdd ''`
- **THEN** exit code 不是 0，錯誤碼是 `invalid_argv`，設定檔沒有被改動

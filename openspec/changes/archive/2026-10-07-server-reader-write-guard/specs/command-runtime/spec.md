## MODIFIED Requirements

### Requirement: 穩定錯誤碼註冊表

命令層 SHALL 以封閉的錯誤碼集合分類失敗：invalid_argv（參數不合法）、not_found（主體不存在）、invalid_config（設定檔或 change metadata 檔存在但無法解析）、refused（前置條件拒絕，須 --force 或先完成前置動作）、error（其餘失敗）。呼叫者送入的內容不合格式（寫入 artifact 時缺必要段落或任務 checkbox、delta spec 沒有操作區段、品質關卡的輪內容不合格式）SHALL 歸類為 invalid_argv，SHALL NOT 落入 error。同一失敗情境的錯誤碼 SHALL NOT 因入口而異；錯誤的語意訊息文字 SHALL 沿用現行 CLI 訊息。

#### Scenario: 需 --force 的拒絕

- **WHEN** 對已記錄開工的 change 執行 speclink discard（未帶 --force）
- **THEN** 指令以非零 exit code 拒絕、不刪除任何檔案，stderr 為現行拒絕訊息（此情境在命令層歸類為 refused）

#### Scenario: 錯誤碼跨入口穩定

- **WHEN** 以相同的非法參數組合分別經 CLI 與 dispatch 執行同一動詞
- **THEN** dispatch 錯誤碼為 invalid_argv，CLI 以非零 exit code 輸出同語意訊息

#### Scenario: 內容格式錯誤歸類為 invalid_argv

- **WHEN** 以沒有任何 `- [ ]` 的內容寫入某 change 的 tasks artifact，分別經 CLI（`speclink new artifact tasks --change <name> --stdin`）、dispatch 與 remote server 執行
- **THEN** 三者都拒絕且不寫入檔案：CLI 以非零 exit code 在 stderr 輸出 `Error: Tasks must contain at least one checkbox (- [ ])`（與本需求生效前逐字相同）；dispatch 錯誤碼為 invalid_argv；server 回 HTTP 400、reason 為 invalid_argument，message 與 CLI 訊息相同

##### Example: 失敗情境對應錯誤碼

| 情境 | 錯誤碼 |
| --- | --- |
| status 指到不存在的 change | not_found |
| discard 已開工的 change 未帶 --force | refused |
| discuss discard 已有 rounds 未帶 --force | refused |
| .speclink.yaml 存在但 YAML 解析失敗 | invalid_config |
| openspec/config.yaml 存在但 YAML 解析失敗 | invalid_config |
| 某 change 的 .openspec.yaml 存在但 YAML 解析失敗 | invalid_config |
| dispatch 收到未支援的動詞 | invalid_argv |
| 寫入的 tasks 沒有任何 `- [ ]` | invalid_argv |
| 寫入的 proposal 缺 `## Why`、`## Problem` 與 `## Summary` | invalid_argv |
| 審查輪內容缺 `**Scope**:` 行 | invalid_argv |

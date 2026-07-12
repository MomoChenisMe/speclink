## ADDED Requirements

### Requirement: change metadata 損壞的跨入口處置

`.openspec.yaml` 存在但 YAML 解析失敗的 change SHALL 標為 invalid：list SHALL 照常列出全部 change，該 change 的 `--json` 項目 SHALL 附選填欄位 metaError（值為解析原因）、人眼輸出 SHALL 於該行附 invalid 標記，其餘 change 的輸出 SHALL NOT 受影響。需要 metadata 語意的單一 change 動詞（查詢：status、instructions、validate、analyze、drift、artifact cat；與全部變更型動詞）SHALL 拒絕並停止，錯誤訊息 SHALL 指出該 metadata 檔的 workspace 相對路徑與解析原因，錯誤碼 SHALL 為 invalid_config 且 SHALL NOT 因入口而異。檔案不存在或欄位缺席 SHALL 維持既有預設行為。

#### Scenario: list 對壞 metadata 標 invalid 而不失效

- **WHEN** workspace 含兩個 change 且其一 `.openspec.yaml` 為壞 YAML，執行 speclink list --json
- **THEN** exit code 0；清單含全部兩個 change；壞檔項目帶 metaError 欄位；有效項目無 metaError 欄位且內容與無壞檔時一致

#### Scenario: 單一 change 查詢對壞 metadata fail closed

- **WHEN** 對 `.openspec.yaml` 為壞 YAML 的 change 執行 speclink status --change 該 change
- **THEN** 以非零 exit code 結束；stderr 指出該檔的 workspace 相對路徑與解析原因；此情境於命令層歸類為 invalid_config

#### Scenario: dispatch 與 CLI 對壞 metadata 的錯誤分類一致

- **WHEN** 同一壞 metadata 的 workspace 分別經 CLI 與 engine.dispatch(['status', '--change', 該 change]) 執行
- **THEN** dispatch 以 Error 拒絕、code 為 invalid_config，message 與 CLI 訊息文字相同

## MODIFIED Requirements

### Requirement: 穩定錯誤碼註冊表

<!-- BEFORE: invalid_config 僅描述為「設定檔存在但無法解析」，對應表無 .openspec.yaml 情境 -->

命令層 SHALL 以封閉的錯誤碼集合分類失敗：invalid_argv（參數不合法）、not_found（主體不存在）、invalid_config（設定檔或 change metadata 檔存在但無法解析）、refused（前置條件拒絕，須 --force 或先完成前置動作）、error（其餘失敗）。同一失敗情境的錯誤碼 SHALL NOT 因入口而異；錯誤的語意訊息文字 SHALL 沿用現行 CLI 訊息。

#### Scenario: 需 --force 的拒絕

- **WHEN** 對已記錄開工的 change 執行 speclink discard（未帶 --force）
- **THEN** 指令以非零 exit code 拒絕、不刪除任何檔案，stderr 為現行拒絕訊息（此情境在命令層歸類為 refused）

#### Scenario: 錯誤碼跨入口穩定

- **WHEN** 以相同的非法參數組合分別經 CLI 與 dispatch 執行同一動詞
- **THEN** dispatch 錯誤碼為 invalid_argv，CLI 以非零 exit code 輸出同語意訊息

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

## ADDED Requirements

### Requirement: 變更清單的寫碼進度欄位

desktop 協定的 change 清單項 SHALL 增列 `codeTotal`/`codeComplete`/`codeRemaining` 三欄(寫碼任務的總數/完成數/剩餘數;`[M]` 手動測試任務不計),計數 SHALL 取自引擎任務雙組計數的同一入口——與品質站守門及失效判定的任務錨同源,SHALL NOT 於呈現層另行過濾。欄位命名 SHALL 與 instructions apply payload 的寫碼進度欄位一致。CLI `speclink list --json` SHALL NOT 包含此三欄;remote 變更摘要 payload SHALL NOT 增列(待手測標示為 local-only,沿審查狀態欄位的先例)。

#### Scenario: 清單項帶寫碼進度

- **WHEN** desktop 載入變更清單且某 change 有 9 個已勾寫碼任務與 1 個未勾 `[M]` 任務
- **THEN** 該清單項含 codeTotal=9、codeComplete=9、codeRemaining=0,既有欄位(completedTasks=9、totalTasks=10 等)不變

#### Scenario: CLI 清單不含寫碼進度欄位

- **WHEN** 執行 speclink list --json
- **THEN** change 項不含 codeTotal/codeComplete/codeRemaining,輸出與本需求引入前逐位元一致

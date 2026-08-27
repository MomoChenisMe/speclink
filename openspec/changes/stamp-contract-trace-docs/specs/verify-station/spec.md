## MODIFIED Requirements

### Requirement: 驗證蓋章守門與蓋章效果

系統 SHALL 提供 `speclink verify stamp <change> [--accept]`,守門與審查站同一條:寫碼任務全數完成(manual-task-marker 預測子)＋工單末輪零未解必修 findings。必修 SHALL 以嚴重度界定:CRITICAL 與 WARNING 級為必修、擋乾淨蓋章;SUGGESTION 級 SHALL NOT 擋章——末輪僅含 SUGGESTION 級 findings 時蓋章照常放行。`--accept` SHALL 僅豁免必修條件。通過時 SHALL 於同一原子寫入內:將 `verified_at`/`verified_by`/`verified_with`/`verified_tasks_total`(蓋章時全任務總數,含 `[M]` 任務)/`verified_scope` 寫入 `.openspec.yaml` 並刪除 `verify.md`,不得出現「章已寫而工單仍在」的中間狀態;canonical mutation 成功後 SHALL 依「驗證 frozen scope 與續輪 snapshot」清理 verify snapshots。工單刪除後其文字於 fs 模式僅存於 git 歷史;remote 模式的 store 不保留已刪文件內容,蓋章後工單文字不可回讀。

#### Scenario: 寫碼任務未完成即拒絕蓋章

- **WHEN** 對寫碼任務 4/5 的 change 執行 `verify stamp`
- **THEN** exit code 非零,stderr 說明寫碼任務未全數完成,metadata 與工單皆不變

#### Scenario: 僅餘手動任務可蓋章

- **WHEN** 寫碼任務全勾、一個 `[M]` 任務未勾且驗證工單末輪 findings 為空時執行 `verify stamp`
- **THEN** exit code 0,五個 verified 欄位寫入且 `verify.md` 刪除

#### Scenario: 末輪有未解 findings 且未帶 --accept

- **WHEN** 驗證工單末輪含至少一筆 CRITICAL 或 WARNING 級 findings 時執行 `verify stamp`
- **THEN** exit code 非零,stderr 點名未解必修數並提示 `--accept` 或先修正重驗

#### Scenario: 僅 SUGGESTION 的末輪乾淨蓋章

- **WHEN** 寫碼任務全數完成且驗證工單末輪僅含 SUGGESTION 級 findings 時執行 `verify stamp`(無 `--accept`)
- **THEN** exit code 0,五個 verified 欄位寫入且 `verify.md` 刪除;fs 模式下 SUGGESTION 紀錄留在工單的 git 歷史,remote 模式下工單文字不保留

#### Scenario: 乾淨蓋章

- **WHEN** 寫碼任務全數完成且末輪 findings 為空時執行 `verify stamp`
- **THEN** exit code 0,`.openspec.yaml` 含五個 verified 欄位且 `verify.md` 不存在

##### Example: 蓋章寫入的任務錨

- **GIVEN** change 有 8 個任務,其中 7 個寫碼任務全數勾選、1 個 `[M]` 任務未勾,驗證工單 Round 1 的 findings 為空
- **WHEN** `verify stamp` 成功
- **THEN** `.openspec.yaml` 內 `verified_tasks_total` 為 8(全任務總數)

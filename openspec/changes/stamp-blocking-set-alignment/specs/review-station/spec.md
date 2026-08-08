## MODIFIED Requirements

### Requirement: 蓋章守門與蓋章效果

<!-- BEFORE: 守門條件 (2) 為「工單末輪零未解 findings」（不分嚴重度，SUGGESTION 也擋章），--accept 豁免全部 findings -->

系統 SHALL 提供 `speclink review stamp <change> [--accept]`。守門條件：(1) change 的任務全數完成；(2) 工單末輪零未解必修 findings。必修 SHALL 以嚴重度界定：CRITICAL 與 WARNING 級為必修、擋乾淨蓋章；SUGGESTION 級 SHALL NOT 擋章——末輪僅含 SUGGESTION 級 findings 時蓋章照常放行。`--accept` SHALL 僅豁免條件 (2) 的必修部分。守門通過時系統 SHALL 於同一原子寫入內：將 `reviewed_at`／`reviewed_by`／`reviewed_with`／`reviewed_tasks_total`（蓋章時任務總數）／`reviewed_scope`（指紋清單）寫入該 change 的 `.openspec.yaml`，並刪除 `review.md`。不得出現「章已寫入而工單仍存在」的中間狀態。

#### Scenario: 任務未全完成即拒絕

- **WHEN** change 的任務為 4/5 完成時執行 `review stamp`
- **THEN** exit code 非零，stderr 說明任務未全數完成，metadata 與工單皆不變

#### Scenario: 末輪有未解 findings 且未帶 --accept

- **WHEN** 工單末輪含至少一筆 CRITICAL 或 WARNING 級 findings 且執行 `review stamp`（無 `--accept`）
- **THEN** exit code 非零，stderr 點名未解必修數並提示 `--accept` 或先修正重審

#### Scenario: 僅 SUGGESTION 的末輪乾淨蓋章

- **WHEN** 任務全數完成且工單末輪僅含 SUGGESTION 級 findings 時執行 `review stamp`（無 `--accept`）
- **THEN** exit code 0，五個 reviewed 欄位寫入且 `review.md` 刪除，SUGGESTION 紀錄留在工單的 git 歷史

#### Scenario: 帶保留蓋章

- **WHEN** 工單末輪含必修 findings 且執行 `review stamp --accept`
- **THEN** exit code 0，章寫入且工單刪除

#### Scenario: 乾淨蓋章

- **WHEN** 任務 5/5 完成且工單末輪 findings 為空時執行 `review stamp`
- **THEN** exit code 0，`.openspec.yaml` 含五個 reviewed 欄位且 `review.md` 不存在

##### Example: 蓋章寫入的任務錨

- **GIVEN** change 有 5 個任務全數勾選，工單 Round 2 的 findings 為空
- **WHEN** `review stamp` 成功
- **THEN** `.openspec.yaml` 內 `reviewed_tasks_total` 為 5

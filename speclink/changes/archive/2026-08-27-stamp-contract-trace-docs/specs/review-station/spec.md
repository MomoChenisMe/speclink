## MODIFIED Requirements

### Requirement: 蓋章守門與蓋章效果

系統 SHALL 提供 `speclink review stamp <change> [--accept]`。守門條件:(1) change 的寫碼任務全數完成(非 `[M]` 任務全數勾選;手動測試任務不計,見 manual-task-marker 的寫碼任務全完成預測子);(2) 工單末輪零未解必修 findings。必修 SHALL 以嚴重度界定:CRITICAL 與 WARNING 級為必修、擋乾淨蓋章;SUGGESTION 級 SHALL NOT 擋章——末輪僅含 SUGGESTION 級 findings 時蓋章照常放行。`--accept` SHALL 僅豁免條件 (2) 的必修部分。守門通過時系統 SHALL 於同一原子寫入內:將 `reviewed_at`/`reviewed_by`/`reviewed_with`/`reviewed_tasks_total`(蓋章時全任務總數,含 `[M]` 任務)/`reviewed_scope`(指紋清單)寫入該 change 的 `.openspec.yaml`,並刪除 `review.md`。不得出現「章已寫入而工單仍存在」的中間狀態。工單刪除後其文字於 fs 模式僅存於 git 歷史;remote 模式的 store 不保留已刪文件內容,蓋章後工單文字不可回讀。

#### Scenario: 寫碼任務未全完成即拒絕

- **WHEN** change 的寫碼任務為 4/5 完成時執行 `review stamp`
- **THEN** exit code 非零,stderr 說明寫碼任務未全數完成並列計數(4/5),metadata 與工單皆不變

#### Scenario: 僅餘手動任務可蓋章

- **WHEN** change 的寫碼任務 4/4 全勾、一個 `[M]` 任務未勾,工單末輪 findings 為空時執行 `review stamp`
- **THEN** exit code 0,五個 reviewed 欄位寫入且 `review.md` 刪除

#### Scenario: 末輪有未解 findings 且未帶 --accept

- **WHEN** 工單末輪含至少一筆 CRITICAL 或 WARNING 級 findings 且執行 `review stamp`(無 `--accept`)
- **THEN** exit code 非零,stderr 點名未解必修數並提示 `--accept` 或先修正重審

#### Scenario: 僅 SUGGESTION 的末輪乾淨蓋章

- **WHEN** 寫碼任務全數完成且工單末輪僅含 SUGGESTION 級 findings 時執行 `review stamp`(無 `--accept`)
- **THEN** exit code 0,五個 reviewed 欄位寫入且 `review.md` 刪除;fs 模式下 SUGGESTION 紀錄留在工單的 git 歷史,remote 模式下工單文字不保留

#### Scenario: 帶保留蓋章

- **WHEN** 工單末輪含必修 findings 且執行 `review stamp --accept`
- **THEN** exit code 0,章寫入且工單刪除

#### Scenario: 乾淨蓋章

- **WHEN** 任務 5/5 完成且工單末輪 findings 為空時執行 `review stamp`
- **THEN** exit code 0,`.openspec.yaml` 含五個 reviewed 欄位且 `review.md` 不存在

##### Example: 蓋章寫入的任務錨

- **GIVEN** change 有 5 個任務,其中 4 個寫碼任務全數勾選、1 個 `[M]` 任務未勾,工單 Round 2 的 findings 為空
- **WHEN** `review stamp` 成功
- **THEN** `.openspec.yaml` 內 `reviewed_tasks_total` 為 5(全任務總數,含未勾的 `[M]` 任務)

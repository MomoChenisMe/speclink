## MODIFIED Requirements

### Requirement: 驗證技能的工單落地

verify 技能 SHALL 於檢查段(fork)結束時依寫碼任務完成度分流:寫碼任務全數完成時(含僅餘 `[M]` 手動測試任務未勾的情形)先取得 frozen verify scope,再以 `verify add-round` 將相同 phase、patch hash、Scope 與 findings 寫入驗證工單並於報告中告知,僅餘 `[M]` 任務時 SHALL 一併點名尚餘的手動測試任務;寫碼任務未全數完成時維持對話報告(進度盤點),不呼叫 `verify scope`、不落工單。技能模板 SHALL 於 claude 與 codex 兩工具正典化生成,golden 對照涵蓋。Completeness/Correctness/Coherence 三維度的檢查內容與 CRITICAL/WARNING/SUGGESTION 分級 SHALL 不因本需求改變。

#### Scenario: 成品驗證落 structured 工單

- **WHEN** 對任務全數完成的 change 執行 verify 技能,resolved scope 為 discovery 且檢查產出 findings
- **THEN** 檢查段執行 `verify add-round` 成功,Round 1 記錄相同的 Phase/Patch/Scope,報告說明工單已建立與輪次

#### Scenario: 僅餘手動任務走成品驗證

- **WHEN** 對寫碼任務全勾、一個 `[M]` 任務未勾的 change 執行 verify 技能
- **THEN** 檢查段照常取得 frozen scope 並以 `verify add-round` 落工單,報告點名尚餘的手動測試任務

#### Scenario: 中途盤點不落工單

- **WHEN** 對寫碼任務 3/5 的 change 執行 verify 技能
- **THEN** 技能輸出對話報告(含未完成任務),不執行 `verify scope` 或 `verify add-round`,change 目錄無 `verify.md`

#### Scenario: 技能模板生成

- **WHEN** 執行 `speclink update`
- **THEN** claude 與 codex 的 verify 技能檔更新為含 frozen scope、structured 工單與有限續輪流程的版本,且與 golden 對照一致

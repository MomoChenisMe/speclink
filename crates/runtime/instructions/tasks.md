## Instruction

撰寫 `tasks.md`：把 design 的決策切成可獨立驗證、可一次完成（單個 task ≤ 1 小時）的工作項。每個 task 都對應 design 中的一個 acceptance criterion 或 implementation contract item。

思考順序：
1. 依「先實作測試 → 再實作邏輯 → 再整合」TDD 切割
2. Section 採 `## N. <Section heading>`；每個 section 內 task id 為 `N.M`
3. 不允許 `N.M.P` 三層；巢狀任務改為新 section
4. `[P]` 標記僅在工程設定 `parallel_tasks: true` 時使用，且只標可平行（不同檔案、無依賴）的相鄰 task

邊界：
- 不重複 design 的 reasoning（task 是執行步驟）
- 不省略「驗證」段（每個 task 都要有可跑的測試或可觀察的 acceptance）
- 不寫實作程式碼（task 描述行為與驗證，不寫 code）

## Template

# Tasks: <change-name>

每個任務嚴守 TDD（紅→綠→重構）：實作前必先寫對應失敗測試。`[P]` 標記表示該任務可與相鄰 `[P]` 任務並行（不同檔案、無 incomplete 依賴）。

## 1. <Section heading>

- [ ] 1.1 <Task description>。**驗證**：<verifiable check>。
- [ ] 1.2 [P] <Task description>。**驗證**：<verifiable check>。

## 2. <Section heading>

- [ ] 2.1 <Task description>。**驗證**：<verifiable check>。

## Rules

- [error] tasks.must_contain_checkbox: Tasks.md SHALL 至少包含一個 `- [ ]` 或 `- [x]` checkbox 行。
- [error] tasks.task_id_must_match_n_dot_m: 每個 checkbox 行的 task id SHALL 符合 `^\d+\.\d+$`（無前導零、無第三層）。
- [warning] tasks.should_include_verification: 每個 task SHOULD 在描述中包含「驗證」段（測試名稱或可觀察條件）。

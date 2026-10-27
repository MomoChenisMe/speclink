## ADDED Requirements

### Requirement: 設定頁政策下拉的未知值顯性呈現

專案設定頁的 locale 與 spec_locale 下拉（local 專案的 config.yaml 簽與遠端 workspace 的 Workflow 簽皆適用）在儲存值非空且不在合法選項集（locale：tw／ja／en；spec_locale：tw／ja／en／auto）時，SHALL 於下拉顯示該原始值並帶無效標註與警示樣式，且該欄位下方 SHALL 顯示引導改選合法代碼的提示文字；SHALL NOT 呈現為空白，SHALL NOT 於讀取時自動清空或改寫儲存值（寫入嚴格、讀取寬容）。使用者改選合法選項並儲存後，SHALL 以所選代碼覆蓋原值，下拉 SHALL 恢復正常呈現且提示文字 SHALL 消失。儲存值為空（未設定）或在合法選項集內時，本需求 SHALL NOT 改變既有呈現。

#### Scenario: 未知儲存值顯性呈現且不被改寫

- **WHEN** 專案的 locale 儲存值為「繁體中文」（合法選項集外的字串），使用者開啟專案設定頁
- **THEN** locale 下拉顯示「繁體中文」並帶無效標註與警示樣式，欄位下方出現改選合法代碼的提示文字；未執行任何儲存動作前，儲存端的值仍為「繁體中文」

#### Scenario: 改選合法代碼即修復

- **WHEN** 於上述狀態，使用者將 locale 下拉改選 tw 並儲存
- **THEN** 儲存端的 locale 值成為 tw，下拉正常顯示 tw 選項，無效標註與提示文字消失

#### Scenario: 合法值與未設定不受影響

- **WHEN** 專案的 locale 儲存值為 tw、spec_locale 未設定，使用者開啟專案設定頁
- **THEN** locale 下拉正常顯示 tw、spec_locale 顯示未設定預設狀態，無任何無效標註或提示文字

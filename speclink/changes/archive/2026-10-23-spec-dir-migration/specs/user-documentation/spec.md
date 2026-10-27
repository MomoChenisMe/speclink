## MODIFIED Requirements

### Requirement: 使用者文件載明本地產物的純文字可攜性

<!-- BEFORE: 不得宣稱可從 OpenSpec 匯入；資料夾未指明、資料檔為 .openspec.yaml -->
<!-- REMOVED-SCENARIO: 本地產物說明不承諾 OpenSpec 相容或匯入 -->

使用者文件 SHALL 說明 **Local 模式**的產物位置——規格資料夾（預設 `speclink/`）下的 `specs/<capability>/spec.md`、`changes/<名稱>/`、`changes/archive/` 與 `config.yaml`——並說明內容為純 Markdown 與 YAML、可不經 Speclink 讀寫、每次變動皆呈現於 Git diff；說明 SHALL 明示此可攜性不適用遠端模式（遠端的正式規格在 Store，本機只有唯讀投影）。說明 SHALL 一併載明 Speclink 在這些產物上的兩項擴充：`discussions/` 與各變更目錄的 `.meta.yaml`。此說明 SHALL 出現於 README 中英兩版與入門教學中英兩版。這些說明 SHALL NOT 宣稱與 OpenSpec 相容或沿用 OpenSpec 的目錄結構；README 特色段與入門教學 SHALL 寫明可從 OpenSpec 專案匯入、且相容為單向：在 OpenSpec 專案執行 `speclink init` 會把 `openspec/` 搬到 `speclink/`，之後 OpenSpec CLI 讀不到這個專案。README 致謝段對 OpenSpec 目錄結構的致謝不受此限。

#### Scenario: 讀者判斷得出資料可攜

- **WHEN** 讀者在 README 或入門教學（中英任一版）讀到本地產物的說明
- **THEN** 可見產物位於 `speclink/` 下的目錄位置、純文字格式、不裝 Speclink 也能讀寫、Git diff 可見的事實，以及 Speclink 額外新增的兩樣東西（`discussions/` 與 `.meta.yaml`），並可見此說明只適用 Local 模式

#### Scenario: 本地產物說明寫明單向匯入

- **WHEN** 讀者在 README 特色段或入門教學中，尋找 speclink 與 OpenSpec 的關係
- **THEN** 說明寫出在 OpenSpec 專案執行 `speclink init` 即可匯入、資料夾會搬到 `speclink/`、之後 OpenSpec CLI 讀不到這個專案；沒有「相容 OpenSpec」或「沿用 OpenSpec 的目錄結構」的宣稱（英文版同樣沒有 OpenSpec-compatible 或 OpenSpec folder layout 的宣稱）

## ADDED Requirements

### Requirement: 入門教學與設定說明載明規格資料夾的遷移

入門教學（中英）SHALL 說明：新專案的規格資料夾是 `speclink/`；舊版建立的 `openspec/` 專案在升級後執行 `speclink update` 會自動搬到 `speclink/`，有其他 worktree 或未提交修改時會暫緩並說明原因，搬完要自己 commit，團隊成員要先升級再 pull。設定說明（中英）的 `spec_dir` 欄位 SHALL 寫明預設值為 `speclink`、遷移與匯入都不會寫入此欄位，以及明寫 `spec_dir` 時不會自動遷移。

#### Scenario: 舊專案使用者查得到遷移方式

- **WHEN** 讀者在 docs/getting-started.zh-TW.md 或 docs/getting-started.md 查舊專案升級後要做什麼
- **THEN** 可見執行 `speclink update` 自動遷移、暫緩的兩個條件、要自己 commit 與團隊先升級再 pull 的說明，且中英兩版一一對應

#### Scenario: 設定說明的 spec_dir 預設值

- **WHEN** 讀者在 docs/configuration.zh-TW.md 或 docs/configuration.md 查 `spec_dir`
- **THEN** 預設值為 `speclink`，並說明遷移與匯入都不寫入此欄位、明寫時不自動遷移

# audit-skill Specification

## Purpose

/speclink-audit 技能檔的內容規定，目前涵蓋獨立模式報告的撰寫語言與維持英文的固定 token。本 capability 只管 audit 技能檔的文字，不管 apply 何時以 Discipline Mode 引用它（屬 apply 的流程）。

## Requirements

### Requirement: 獨立模式報告的撰寫語言依 locale

渲染產出的 speclink-audit 技能檔 SHALL 在獨立模式的彙整與報告階段規定報告語言：以 `speclink workflow-config languages --json` 讀取 `locale` 欄位（解析後的語言名稱，如 Traditional Chinese (繁體中文)、Japanese (日本語)、English），報告（含結尾摘要）的散文、標題與表格標籤 SHALL 以該語言撰寫——`tw` 為繁體中文、`ja` 為日文、`en` 為英文；未設定時 SHALL 經查詢採執行 Speclink 主機的系統語言（Remote 採 Server 語言，不採 Client 語言）；查詢失敗時 SHALL 回報錯誤並停止，不猜測語言。嚴重度標籤（Critical／High／Medium／Low）、檔案路徑、指令行與程式碼引用 SHALL 維持英文。獨立模式的自我檢查清單 SHALL 含一項報告語言的檢查。Discipline Mode（apply 經 `speclink instructions --skill audit` 引用的精簡檢查清單）不產出報告，其段落 SHALL NOT 因本條而改變。

此為刻意變更：變更前技能未規定報告語言；變更後依 `locale`，與 analyze、drift 的報告語言規定一致。

#### Scenario: 渲染產物含報告語言規則

- **WHEN** 執行 speclink init 或 speclink update 渲染 claude 與 codex 工具的技能檔
- **THEN** speclink-audit 技能檔的彙整與報告階段含讀取 `speclink workflow-config languages --json` 的 `locale` 欄位、依該語言撰寫報告（含結尾摘要）、嚴重度標籤與路徑維持英文、查詢失敗時回報並停止的規定，且獨立模式的自我檢查清單含報告語言一項

#### Scenario: locale tw 時報告為繁體中文

- **WHEN** 專案設 `locale: tw`，使用者以獨立模式執行 audit 且找到一個問題
- **THEN** 報告的說明與修法為繁體中文，嚴重度標籤（例如 High）與檔案路徑為英文

#### Scenario: Discipline Mode 段落不變

- **WHEN** 執行 speclink instructions --skill audit
- **THEN** stdout 的 Discipline Mode 段落與變更前逐字相同，報告語言規則只出現在獨立模式的彙整與報告階段；exit code 為 0

##### Example: 報告的撰寫語言

| locale | 報告的散文、標題與表格標籤 | 嚴重度標籤、路徑與指令 |
| ------ | -------------------------- | ---------------------- |
| tw | 繁體中文 | 英文 |
| ja | 日文 | 英文 |
| en | 英文 | 英文 |
| 未設定 | 主機／Server 系統語言 | 英文 |
| 任一（指令失敗） | 回報錯誤並停止 | 英文 |

<!-- @trace
source: skill-conversation-language
updated: 2026-10-16T23:14:02+08:00
-->
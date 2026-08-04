## REMOVED Requirements

### Requirement: touched 記錄的刪除排在封存與提交之後

**Reason**: evidence 記錄改寫入 change 目錄、隨目錄提交與封存移動後，技能流程不再存在任何刪除步驟——原需求規範的時序對象消失。
**Migration**: 技能檔移除刪除指示；記錄的耐久保存與生命週期由 verify-evidence 的「task done 寫入逐任務 evidence」（change 目錄新 home）承接。

### Requirement: @trace 來源敘述與引擎行為一致

**Reason**: 敘述對象——@trace 檔案清單、其 evidence 優先與髒檔退路的來源優先序、bulk 封存的整潔工作樹要求——已全數自引擎移除。
**Migration**: 由本 delta 新增的「trace 與 evidence 的技能敘述」承接技能檔對新引擎行為的敘述義務。

## ADDED Requirements

### Requirement: trace 與 evidence 的技能敘述

內嵌 speclink-archive 技能（事實來源 crates/speclink-core/assets/skills/archive.md，經 init 與 update 渲染至工具技能目錄）SHALL 敘明：@trace 僅含 source 與 updated、於 ADDED／MODIFIED 物化時一律注入、不含檔案清單。技能檔 SHALL NOT 要求 bulk 封存前工作樹整潔，SHALL NOT 指示刪除 evidence 記錄，SHALL NOT 敘述任何 evidence 守門、拒絕情形或放行旗標。技能檔 SHALL 敘明零證據提示行的意義：封存無任何任務證據記錄的 change 時 stderr 會出現一行提示，見到提示應確認該 change 是否漏走 apply 流程（純規格或文件變更的零證據屬正常）。渲染產物內容由 speclink-core 的 render_golden 測試保護，golden 快照更新屬刻意變更。

#### Scenario: 技能檔無刪除步驟、整潔要求與守門敘述殘留

- **WHEN** 檢視渲染產出的 speclink-archive 技能檔全文
- **THEN** 不含刪除 evidence／touched 記錄的指示，不含 bulk 封存前工作樹必須整潔的要求，亦不含任何 evidence 守門拒絕或 waive-evidence 放行旗標的敘述

#### Scenario: trace 與提示敘述到位

- **WHEN** 檢視渲染產出的 speclink-archive 技能檔對 @trace 與零證據提示的敘述
- **THEN** @trace 敘述為 source 與 updated 兩欄一律注入、無檔案清單；零證據提示段敘明提示出現的條件與應對（確認是否漏走 apply；純規格變更屬正常）

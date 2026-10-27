## ADDED Requirements

### Requirement: task done 消費 touchedFiles 且 evidence 有唯讀端點

task done 端點 SHALL 消費請求 payload 的 touchedFiles 並作為 Host 解析的候選交給 Engine，SHALL NOT 丟棄；payload 未攜帶 touchedFiles 時 SHALL 視為無候選（沿無新髒檔語意），SHALL NOT 視為錯誤。server SHALL 提供該 change 的 evidence 唯讀端點：viewer 以上角色可讀，回應欄位為 camelCase 的 evidence 記錄集合；記錄缺席 SHALL 回空集合而非 not_found——缺席是正常狀態，SHALL NOT 讓讀取端以錯誤碼區分「change 存在但無 evidence」。

#### Scenario: task done 攜檔案後 evidence 端點可讀回

- **WHEN** 以 editor 角色對某任務執行 task done 且 payload 攜帶 touchedFiles，隨後以 viewer 角色讀取該 change 的 evidence 端點
- **THEN** 回應含該任務 entry，touchedFiles 與 payload 一致，欄位為 camelCase

#### Scenario: 無 evidence 回空集合

- **WHEN** 對存在但從未落 evidence 的 change 讀取 evidence 端點
- **THEN** 回應為成功的空集合，非 not_found

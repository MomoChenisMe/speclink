## ADDED Requirements

### Requirement: 新開 capability 的近似名 warning

`speclink validate <change>` 對 delta 中「正典無同名」（以正典 capability 清單逐字比對）的每個 capability，SHALL 以與建立點主閘相同的建議池（正典 capabilities 加未封存 change 的 delta capabilities，含同 change 的其他 delta、僅排除受檢 capability 自身）與相同排序規則求取近似名；建議池非空時 SHALL 報 warning 級發現，訊息 SHALL 含近似名清單與指引——同一 capability 就把 delta 目錄改用既有名、確為新 capability 可忽略本警告。此 warning SHALL NOT 改變驗證結果的通過與否，SHALL NOT 影響 exit code。正典已有同名規格的 delta capability SHALL NOT 觸發此檢查；建議池為空時 SHALL NOT 報 warning。

#### Scenario: 近似新名報 warning 且驗證仍通過

- **WHEN** 正典有 `auth`，change 的 delta 含目錄 `authentication` 且其餘內容全部合法，執行 `speclink validate <change>`
- **THEN** 輸出含一筆指名 `authentication` 與近似名 `auth` 的 warning，驗證結果為通過，exit code 為 0

#### Scenario: 既有 capability 的 delta 不觸發

- **WHEN** change 的 delta 目錄名稱與正典規格同名
- **THEN** 驗證輸出不含近似名 warning

#### Scenario: 無近似名的新 capability 不報

- **WHEN** change 的 delta 含正典未收錄、且與所有既有名毫無交集的目錄名稱
- **THEN** 驗證輸出不含近似名 warning（既有的新開 capability Purpose 檢查不受影響、照常執行）

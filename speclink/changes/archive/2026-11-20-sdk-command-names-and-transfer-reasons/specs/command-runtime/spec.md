## ADDED Requirements

### Requirement: 引擎提供全部命令名稱的清單

引擎 SHALL 提供一份命令名稱的清單：每一種引擎命令的名稱在清單裡正好出現一次，清單 SHALL NOT 含任何不是引擎命令名稱的字串。清單裡的名稱 SHALL 與「引擎命令有穩定的名稱與可序列化的參數」規定的名稱逐字相同。引擎的測試 SHALL 釘住這個對應：新增一種命令而清單沒有它的名稱、或清單多出一個沒有對應命令的名稱時，測試 SHALL 失敗。這份清單 SHALL NOT 帶命令的其他性質（例如會不會寫入）。

#### Scenario: 清單與命令的種類一一對應

- **WHEN** 把清單裡的名稱與每一種引擎命令的名稱各自排序後比較
- **THEN** 兩者逐項相同；目前是 54 個名稱，含 `new-change`、`claim-release`、`discuss-add-round` 與 `workflow-config-edit`

#### Scenario: 清單沒有重複的名稱

- **WHEN** 檢視清單
- **THEN** 沒有任何名稱出現兩次

#### Scenario: 清單漏掉一個命令時測試失敗

- **WHEN** 引擎多了一種命令，而清單沒有加上它的名稱
- **THEN** 引擎的測試失敗，失敗訊息點名缺少的名稱

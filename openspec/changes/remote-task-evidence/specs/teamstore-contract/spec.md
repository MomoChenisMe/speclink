## MODIFIED Requirements

### Requirement: 文件定址採 Project 與 Repo scope 的邏輯 locator

TeamStore 的文件身分 SHALL 為 ProjectId、RepoId 與 DocumentId 三元組；DocumentId SHALL 為封閉的邏輯種類集合（change metadata、change artifact、change evidence、canonical spec、discussion、workflow config、archived 文件、language 與 board order），SHALL NOT 以實體路徑作跨媒介身分。board order 為 scope 層級單文件種類（同 workflow config 形狀），change evidence 為 change 層級單文件種類（一個 change 至多一份，內容為 evidence 記錄的序列化文字，store 不解讀），三個官方 driver 的編碼／解碼與 conformance suite SHALL 涵蓋兩者，export bundle 的泛用文件列舉 SHALL 自動包含兩者。跨 project 或跨 repo 的讀寫 SHALL 被隔離：對不屬於該 scope 的文件操作回 not_found 或 permission_denied，SHALL NOT 回傳其他 tenant 的資料。

#### Scenario: tenant scope 隔離

- **WHEN** 以 repo A 的 scope 讀取僅存在於 repo B 的同名 canonical spec
- **THEN** 回傳成功的空值或 permission_denied（依實作的權限模型），絕不回傳 repo B 的內容

#### Scenario: board order 種類 round-trip

- **WHEN** 對任一官方 driver 以 UoW 寫入 board order 文件後重開 store 讀取
- **THEN** 內容逐位元組一致，且該文件出現於同 scope 的 export bundle

#### Scenario: change evidence 種類 round-trip

- **WHEN** 對任一官方 driver 以 UoW 寫入某 change 的 evidence 文件後重開 store 讀取
- **THEN** 內容逐位元組一致，且該文件出現於同 scope 的 export bundle

#### Scenario: 封閉集合外的種類不存在

- **WHEN** 檢視 DocumentId 的種類定義
- **THEN** 集合恰為 change metadata、change artifact、change evidence、canonical spec、discussion、workflow config、archived 文件、language 與 board order 九種，無其他變體

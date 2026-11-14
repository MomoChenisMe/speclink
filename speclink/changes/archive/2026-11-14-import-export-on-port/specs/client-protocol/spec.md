## ADDED Requirements

### Requirement: 以鍵標示文件的匯入包與匯出回應

Client Protocol SHALL 定義第二版的匯入包：`formatVersion` 為 2，`documents` 為陣列，每項含 `key`（字串，儲存契約的文件鍵）、`content`（字串）與 `digest`（字串，`sha256:` 加十六進位，內容的 SHA-256）。匯入端點 SHALL 依 `formatVersion` 接受第一版（以文件種類標示）與第二版；其他值 SHALL 以 refused 拒絕。第二版的匯入成功回應為 `scopeVersion`（整數）與 `documents`（整數，寫入的份數）；第一版的請求 SHALL 維持既有的回應形狀。Client Protocol SHALL 定義匯出端點：對專案底下 export 的 GET，回應為第二版的包加上 `scopeVersion`，documents 依鍵排序。typed client SHALL 提供匯出的方法與送出第二版匯入包的方法；Server 以「格式版本不認得」拒絕第二版時，用戶端 SHALL 能以第一版重送同一批內容。第一版的型別與其回應 SHALL 不變。

#### Scenario: 第二版的包匯入成功

- **WHEN** 對空的範圍送出 formatVersion 2、含三份文件的匯入請求
- **THEN** 回應是 200，payload 的 documents 是 3，並帶 scopeVersion

#### Scenario: 第一版的包照常可用

- **WHEN** 對空的範圍送出既有第一版形狀的匯入請求
- **THEN** 回應的形狀與既有行為相同，逐份列出文件與結果

#### Scenario: 不認得的格式版本

- **WHEN** 送出 formatVersion 3 的匯入請求
- **THEN** 回應的 reason 為 refused；範圍不變

#### Scenario: 匯出的回應

- **WHEN** 對含三份文件的範圍請求 GET export
- **THEN** 回應的 formatVersion 是 2，documents 有三項且依鍵排序，每項的 digest 等於其 content 的摘要，並帶 scopeVersion

#### Scenario: 匯出的回應可以原樣匯入

- **WHEN** 把匯出回應的 formatVersion 與 documents 當成匯入請求送到另一個空的範圍
- **THEN** 匯入成功，份數相同

#### Scenario: 舊 Server 拒絕第二版時改送第一版

- **WHEN** 用戶端對只認得第一版的 Server 送出第二版的包而被以 refused 拒絕
- **THEN** 用戶端以第一版重送相同的內容並成功

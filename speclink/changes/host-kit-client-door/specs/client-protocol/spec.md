## ADDED Requirements

### Requirement: API 根的身分查詢

Client Protocol SHALL 定義 API 根底下的身分查詢：對 `/api/speclink/v1/whoami` 的 GET，請求需要有效的身分但 SHALL NOT 需要專案、repo 或版本標頭。回應的 payload 與既有根層身分查詢相同：`user` 物件含 `name`（string，顯示名稱）與 `handle`（string，識別）。沒有有效身分時 SHALL 回應 401。用戶端在登入後查詢身分時 SHALL 先使用這個路徑；server 回應 404 或 405 時 SHALL 改用既有的根層路徑，兩者的 payload SHALL 以同一個型別解析。

#### Scenario: 查詢成功

- **WHEN** 帶有效身分請求 API 根的身分查詢
- **THEN** 回應是 200，payload 含 user.name 與 user.handle，皆為字串

#### Scenario: 沒有身分

- **WHEN** 不帶身分請求 API 根的身分查詢
- **THEN** 回應是 401

#### Scenario: 不需要專案

- **WHEN** 一個不屬於任何專案的使用者帶有效身分請求 API 根的身分查詢
- **THEN** 回應是 200

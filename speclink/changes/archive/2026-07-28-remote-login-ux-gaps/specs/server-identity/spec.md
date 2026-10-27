## MODIFIED Requirements

### Requirement: 帳號 browser API 保持憑證祕密邊界

登入使用者 SHALL 能經 `/api/speclink/v1/web/account` 讀取 user、自己的專案隸屬清單、PAT metadata、Web sessions 與 device families，並建立／撤銷自己的 PAT、經 `POST /logout` 結束目前 Web session、撤銷 device family。專案隸屬清單 SHALL 每項含專案 key、專案顯示名與角色（camelCase 欄位），無任何隸屬時 SHALL 為空陣列；admin 與一般成員 SHALL 得到同一形狀。Web session 清單 SHALL 為唯讀呈現，SHALL NOT 提供逐一撤銷其他 session 的操作。讀取 payload SHALL 僅含呈現與 eligibility 所需 metadata，SHALL NOT 包含 PAT hash、password hash、refresh credential 或可重播的 session secret。PAT 建立回應 SHALL 只在該次 `{data}` 內回傳 plaintext；後續讀取 SHALL 僅回 prefix、名稱、到期、撤銷時戳與 last-used。所有 mutation SHALL 驗證同源與 active session。

#### Scenario: PAT 明文只在建立回應出現

- **WHEN** 使用者經 browser API 建立 PAT，接著重新讀取 account summary
- **THEN** 建立回應包含 plaintext；summary 只含 prefix 與 metadata，沒有途徑再次取得 plaintext

#### Scenario: 撤銷 device family 即時生效

- **WHEN** 使用者從帳號頁撤銷一個仍有 active refresh credential 的 device family
- **THEN** family 內 refresh credential 立即失效，後續 account summary 顯示該 family 已撤銷且不回傳 credential

#### Scenario: Account summary 不外洩其他使用者資料

- **WHEN** 一般成員呼叫 account summary
- **THEN** 回應只含該 session user 自己的 user、專案隸屬、PAT、Web session 與 device family metadata

#### Scenario: summary 回傳自己的專案隸屬

- **WHEN** 隸屬兩個專案（一為 editor、一為 viewer）的成員呼叫 account summary
- **THEN** 回應的隸屬清單恰含兩項，各含專案 key、專案顯示名與角色；無隸屬的使用者得到空陣列

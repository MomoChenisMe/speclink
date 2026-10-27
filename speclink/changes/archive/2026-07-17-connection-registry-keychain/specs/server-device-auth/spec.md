## ADDED Requirements

### Requirement: root 層 bearer 身分查詢

Server SHALL 於 root 層提供 GET /auth/whoami：以 Authorization bearer 解析身分，成功回傳該使用者的顯示名與識別。bearer 解析 SHALL 與 project-scoped Binding 的第一步一致——`spk_at_` 前綴走 device access token 驗證、其餘走 PAT 驗證；任何解析失敗 SHALL 回同一 401 permission_denied、SHALL NOT 區分原因。PAT 命中 SHALL 前進其 last-used。此端點 SHALL NOT 要求 project scope、API version header 或 repo header——它是登入完成當下、尚未選定 project 的 client 取得身分顯示名的來源。

#### Scenario: access token 查得身分

- **WHEN** 以 device flow 核准取得的 access token 呼叫 GET /auth/whoami
- **THEN** 回 200 與核准者的顯示名與識別

#### Scenario: PAT 查得身分且前進 last-used

- **WHEN** 以有效 PAT 呼叫 GET /auth/whoami
- **THEN** 回 200 與擁有者的顯示名；該 PAT 的 last-used 時間前進

#### Scenario: 無效 bearer 是同一 401

- **WHEN** 以缺席、格式錯誤、已撤銷或已過期的 bearer 呼叫 GET /auth/whoami
- **THEN** 回 401 permission_denied，回應不區分失敗原因

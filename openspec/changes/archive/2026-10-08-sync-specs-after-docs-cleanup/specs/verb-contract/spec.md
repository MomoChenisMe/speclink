## MODIFIED Requirements

### Requirement: 樂觀並行控制與 409 語意
artifact 寫入 SHALL 攜帶讀取時取得的版本（If-Match）；版本過期時 server 回 409 且 body SHALL 含機器可判的 reason 欄位。reason 的值 SHALL 取自 Protocol 的錯誤 reason 註冊表（409 用到 `revision_conflict` 與 `refused`），SHALL NOT 使用註冊表以外的值。CLI 對 409 SHALL 輸出可讀的建議動作訊息，SHALL NOT 顯示裸狀態碼：reason 為 `revision_conflict` 時 SHALL 說明內容在讀取後已被更新、建議重新讀取後再套用修改；reason 為 `refused` 時 SHALL 轉述 server 的訊息（認領衝突時訊息含目前持有人與建議動作）。

#### Scenario: 版本衝突的可讀訊息
- **WHEN** 於 remote 模式寫入 artifact，而該 artifact 已被他人更新（server 回 409、reason 為 revision_conflict）
- **THEN** exit code 非 0，stderr 單行訊息含 `content changed since you read it — re-read it and re-apply your edit`，不顯示裸狀態碼

#### Scenario: 認領被搶佔
- **WHEN** 執行 speclink claim 某 change，而 server 回 409、reason 為 refused（已被他人認領）
- **THEN** exit code 非 0，stderr 訊息含目前持有人資訊與建議動作，例如 `change 'add-auth' is already claimed by Alice <alice@example.com> — coordinate with them, or ask them to release it`

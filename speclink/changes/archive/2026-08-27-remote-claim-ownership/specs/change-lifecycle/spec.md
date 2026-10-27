## ADDED Requirements

### Requirement: 認領標記欄位

change meta SHALL 支援選填的 claimed_by 與 claimed_at 欄位作為認領標記：引擎 Claim 命令於支援團隊模式的 store 上，對未認領的 change SHALL 以 ExecutionContext 的 actor 寫入 claimed_by、以當下時間寫入 claimed_at，隨 Unit of Work 原子提交、發布 change-claimed 事件且 scope revision 前進，meta 既有欄位 SHALL 逐字元保留；同一身分重複認領 SHALL 冪等成功——零寫入、零事件、revision 不前進；已被其他身分認領時 SHALL 以 ownership 衝突拒絕且 message 含目前持有人。本地 fs store 上 Claim SHALL 維持既有的明確拒絕（RemoteOnly 語意零改動）。欄位缺席即未認領（「meta 新欄位向後相容」規則適用）；meta 解析失敗時 SHALL 沿既有 fail-closed 守門拒絕且零寫入。

#### Scenario: 首次認領寫章與事件

- **WHEN** 於團隊模式 store 對未認領的 change 執行 Claim（actor 為 "Alice <a@example.com>"）
- **THEN** meta 新增 claimed_by 與 claimed_at、既有欄位逐字元保留，change-claimed 事件發布且 revision 前進

#### Scenario: 同人冪等與他人衝突

- **WHEN** 同一 actor 再次 Claim 同 change，接著另一 actor（"Bob <b@example.com>"）Claim 同 change
- **THEN** 前者冪等成功且零寫入、零事件、首章逐字元保留；後者被 ownership 衝突拒絕、message 含 "Alice <a@example.com>"、meta 零改動

#### Scenario: fs store 拒絕語意不變

- **WHEN** 於本地 fs store 專案執行 Claim
- **THEN** 以既有的明確拒絕文案失敗，零寫入

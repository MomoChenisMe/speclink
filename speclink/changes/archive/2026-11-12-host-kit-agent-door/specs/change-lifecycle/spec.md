## MODIFIED Requirements

### Requirement: 認領標記欄位

change meta SHALL 支援選填的 claimed_by、claimed_at 與 claimed_by_id 欄位作為認領標記：引擎 Claim 命令於支援團隊模式的 store 上，對未認領的 change SHALL 以 ExecutionContext 的 actor 寫入 claimed_by、以當下時間寫入 claimed_at，ExecutionContext 帶識別（actor_id）時 SHALL 同時寫入 claimed_by_id，隨 Unit of Work 原子提交、發布 change-claimed 事件且 scope revision 前進，meta 既有欄位 SHALL 逐字元保留；同一身分重複認領 SHALL 冪等成功——零寫入、零事件、revision 不前進；已被其他身分認領時 SHALL 以 ownership 衝突拒絕且 message 含目前持有人（claimed_by）。判斷是否同一身分時，ExecutionContext 帶識別且 meta 有 claimed_by_id 者 SHALL 只比對識別——識別相同而 actor 不同仍為同一身分，識別不同而 actor 相同仍為不同身分；任一方沒有識別時 SHALL 比對 actor。本地 fs store 上 Claim SHALL 維持既有的明確拒絕（RemoteOnly 語意零改動）。欄位缺席即未認領（「meta 新欄位向後相容」規則適用）；meta 解析失敗時 SHALL 沿既有 fail-closed 守門拒絕且零寫入。

#### Scenario: 首次認領寫章與事件

- **WHEN** 於團隊模式 store 對未認領的 change 執行 Claim（actor 為 "Alice <a@example.com>"）
- **THEN** meta 新增 claimed_by 與 claimed_at、既有欄位逐字元保留，change-claimed 事件發布且 revision 前進

#### Scenario: 同人冪等與他人衝突

- **WHEN** 同一 actor 再次 Claim 同 change，接著另一 actor（"Bob <b@example.com>"）Claim 同 change
- **THEN** 前者冪等成功且零寫入、零事件、首章逐字元保留；後者被 ownership 衝突拒絕、message 含 "Alice <a@example.com>"、meta 零改動

#### Scenario: fs store 拒絕語意不變

- **WHEN** 於本地 fs store 專案執行 Claim
- **THEN** 以既有的明確拒絕文案失敗，零寫入

#### Scenario: 帶識別的認領以識別判斷身分

- **WHEN** 於團隊模式 store，以 actor "Alice"、識別 "u_1" 認領 change，接著以 actor "Alice"、識別 "u_2" 認領同一個 change，再以 actor "Alice Chen"、識別 "u_1" 認領同一個 change
- **THEN** 第一次寫入 claimed_by "Alice"、claimed_by_id "u_1" 與 claimed_at；第二次被 ownership 衝突拒絕、message 含 "Alice"、meta 零改動；第三次冪等成功、零寫入，claimed_by 仍是 "Alice"

#### Scenario: 沒有識別時照舊比對 actor

- **WHEN** ExecutionContext 不帶識別，以 actor "Alice <a@example.com>" 認領一個 meta 已有 claimed_by "Alice <a@example.com>" 與 claimed_by_id "u_1" 的 change，接著以 actor "Bob <b@example.com>" 認領同一個 change
- **THEN** 前者冪等成功且零寫入；後者被 ownership 衝突拒絕、meta 零改動

## ADDED Requirements

### Requirement: 釋放認領

引擎 SHALL 提供釋放認領的命令（穩定名稱 `claim-release`），CLI 動詞是 `speclink claim <change> --release`。於支援團隊模式的 store 上，對已被認領的 change，命令 SHALL 自 change meta 移除 claimed_by、claimed_at 與 claimed_by_id 三個欄位（存在哪幾個就移除哪幾個），隨 Unit of Work 原子提交、發布 change-claim-released 事件且 scope revision 前進；meta 的其他欄位 SHALL 逐字元保留。釋放之後，任何身分 SHALL 能重新認領該 change。

引擎 SHALL NOT 限定誰可以釋放：執行者不必是認領的人，也不必帶身分。限定由宿主的執行前掛鉤負責；掛鉤收到的命令名稱是 `claim-release`，參數含變更名稱。

change 沒有任何認領欄位時，命令 SHALL 冪等成功——零寫入、零事件、revision 不前進。meta 只有 claimed_at 而沒有 claimed_by 時（不一致的半套標記），命令 SHALL 把它當成已認領處理：移除殘留的欄位並發布事件，使該 change 回到可認領的狀態。本地 fs store 上，命令 SHALL 以與 Claim 相同的明確拒絕文案失敗且零寫入。change 不存在時 SHALL 以 not-found 拒絕。meta 解析失敗時 SHALL 沿既有 fail-closed 守門拒絕且零寫入。Claim 命令的語意與拒絕訊息 SHALL 不變。

CLI 輸出：釋放成功時 stdout SHALL 是 `✓ Released claim on change: <name> (was claimed by <holder>)`（釋放前沒有 claimed_by 可讀時省略括號那一段）；change 沒有被認領時 stdout SHALL 是 `Change '<name>' is not claimed — nothing to release`。兩種情況 exit code SHALL 都是 0。`--no-color` 下文字相同、不含色碼。失敗時訊息走 stderr、exit code 非零。人眼輸出在本機以外的兩種執行方式（連到遠端的 CLI、宿主套件）SHALL 逐位元一致。

`speclink claim <change> --release --json` SHALL 於 stdout 輸出一個物件：`change`（字串，變更名稱）、`released`（布林，這次呼叫是否移除了認領）、`previousHolder`（字串，釋放前的 claimed_by；沒有可讀的值時 SHALL 不輸出這個鍵）。

遠端路徑：釋放撞上 revision 衝突時 SHALL 沿既有的寫入動詞行為失敗並請使用者重跑；離線或認證失效時 SHALL 直接失敗、不排隊，與其他寫入動詞相同。

#### Scenario: 別人釋放後可以重新認領

- **WHEN** 於團隊模式 store，Alice 已認領 change demo；Bob 執行 `speclink claim demo --release`，接著執行 `speclink claim demo`
- **THEN** 第一個指令 exit code 0、stdout 是 `✓ Released claim on change: demo (was claimed by Alice)`，meta 不再有 claimed_by、claimed_at、claimed_by_id，其他欄位逐字元保留，change-claim-released 事件發布且 revision 前進；第二個指令成功，claimed_by 是 Bob

#### Scenario: 沒有被認領時冪等成功

- **WHEN** 對一個沒有任何認領欄位的 change demo 執行 `speclink claim demo --release --json`
- **THEN** exit code 0，stdout 是 `{"change":"demo","released":false}`（鍵的順序不拘），零寫入、零事件、revision 不前進

##### Example: 釋放的三種起始狀態

| 釋放前的 meta | `released` | `previousHolder` | 寫入與事件 |
| --- | --- | --- | --- |
| claimed_by: Alice、claimed_by_id: u_1、claimed_at 有值 | true | "Alice" | 三個欄位移除，發布事件 |
| 沒有任何認領欄位 | false | 不輸出 | 零寫入、零事件 |
| 只有 claimed_at | true | 不輸出 | claimed_at 移除，發布事件 |

#### Scenario: 半套標記經釋放恢復

- **WHEN** change demo 的 meta 只有 claimed_at 而沒有 claimed_by（此時 Claim 會拒絕），執行 `speclink claim demo --release`，接著執行 `speclink claim demo`
- **THEN** 第一個指令成功，stdout 是 `✓ Released claim on change: demo`，claimed_at 被移除；第二個指令成功寫入認領

#### Scenario: 本機專案拒絕

- **WHEN** 於本地 fs store 專案執行 `speclink claim demo --release`
- **THEN** 指令以非零 exit code 結束，stderr 是與 `speclink claim demo` 相同的拒絕文案，零寫入

#### Scenario: 變更不存在

- **WHEN** 於團隊模式 store 執行 `speclink claim ghost --release`，而沒有名為 ghost 的 change
- **THEN** 指令以非零 exit code 結束，stderr 指出找不到 ghost，零寫入

#### Scenario: 宿主的掛鉤擋下釋放

- **WHEN** 宿主的執行前掛鉤對命令 `claim-release` 回答拒絕、原因「只有認領的人可以釋放」，使用者經宿主套件釋放 change demo 的認領
- **THEN** 呼叫以 `refused` 失敗，訊息是「只有認領的人可以釋放」，meta 的認領欄位不變

### Requirement: 認領記錄的讀取

`speclink show <change> --json` SHALL 增列選填的 `claimed` 物件：`by`（字串，meta 的 claimed_by）、`byId`（字串，meta 的 claimed_by_id，有記錄才出現）、`at`（字串，meta 的 claimed_at 原樣，有記錄才出現）。change 沒有 claimed_by 時 SHALL 不輸出 `claimed` 這個鍵，此時 `show --json` 的輸出 SHALL 與引入這個鍵之前逐位元相同。show 的人眼輸出 SHALL 維持既有的位元級輸出。`speclink list --json` SHALL NOT 因認領記錄改變形狀。

`speclink claim <change> --json` SHALL 於認領成功（含同一身分重複認領）時在 stdout 輸出一個物件：`change`（字串，變更名稱）、`claimedBy`（字串，認領後的持有人）。不帶 `--json` 的 `speclink claim <change>` 的 stdout 與 exit code SHALL 維持既有的位元級輸出。認領被拒絕時（已被別人認領、變更不存在、本機專案），帶不帶 `--json` 都 SHALL 以非零 exit code 結束、訊息走 stderr、stdout 沒有 JSON。

`claimed` 與 `claim --json` 的輸出 SHALL 不隨 `locale`（`tw`、`ja`、`en` 或未設定）改變。連到遠端的 CLI 與宿主套件對同一個 change SHALL 輸出相同的 `claimed` 物件；連到不提供識別與時間的舊版 Server 時，`claimed` SHALL 只含 `by`。

#### Scenario: 被認領的變更在 show --json 帶出認領的人

- **WHEN** change demo 的 meta 有 claimed_by "Alice"、claimed_by_id "u_1"、claimed_at "2026-11-18T10:00:00+00:00"，執行 `speclink show demo --json`
- **THEN** 輸出含 `"claimed": { "by": "Alice", "byId": "u_1", "at": "2026-11-18T10:00:00+00:00" }`，其餘欄位與沒有認領時相同

#### Scenario: 沒有識別的認領不輸出 byId

- **WHEN** change demo 的 meta 有 claimed_by "Alice <a@example.com>" 與 claimed_at，沒有 claimed_by_id，執行 `speclink show demo --json`
- **THEN** `claimed` 含 `by` 與 `at`，沒有 `byId` 這個鍵

#### Scenario: 沒有被認領時輸出不變

- **WHEN** 對一個沒有認領欄位的 change 執行 `speclink show demo --json` 與 `speclink show demo`
- **THEN** 兩個輸出都與引入 `claimed` 之前逐位元相同，JSON 沒有 `claimed` 這個鍵

#### Scenario: claim --json 輸出持有人

- **WHEN** 於團隊模式 store，Alice 對未認領的 change demo 執行 `speclink claim demo --json`，接著再執行一次
- **THEN** 兩次的 stdout 都是 `{"change":"demo","claimedBy":"Alice"}`（鍵的順序不拘），exit code 都是 0；第二次零寫入

#### Scenario: 認領被拒絕時 stdout 沒有 JSON

- **WHEN** change demo 已被 Alice 認領，Bob 執行 `speclink claim demo --json`
- **THEN** exit code 非零，stderr 含目前的持有人 Alice，stdout 是空的

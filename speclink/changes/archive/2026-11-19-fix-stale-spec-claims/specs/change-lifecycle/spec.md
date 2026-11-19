## MODIFIED Requirements

### Requirement: 變更以 discard 動詞廢棄

<!-- BEFORE: remote store 模式下 discard 以非零 exit code 回報不支援 -->
<!-- REMOVED-SCENARIO: remote store 模式不支援 -->

speclink discard SHALL 接受一個位置參數（變更名）與 --force、--json 旗標，廢棄一個尚未動工的變更：刪除 speclink/changes/<change>/ 目錄整棵，並刪除該變更的 touched 紀錄檔（若存在）。變更不存在時 SHALL 以非零 exit code 結束並於 stderr 說明。變更有動工痕跡——meta 含 started_at，或 tasks.md 有任何已勾任務——且未帶 --force 時 SHALL 拒絕：非零 exit code、stderr 提示動工痕跡與 --force，且 SHALL NOT 改動任何檔案；帶 --force 時 SHALL 照常執行。成功時 exit code 0，stdout 報告已刪除的變更名與每份解鏈討論的 slug 及回退後狀態（--no-color 下無 ANSI 色彩）；帶 --json 時 SHALL 輸出 camelCase payload：變更名與解鏈討論清單（各含 slug 與回退後狀態）。remote 模式下 discard SHALL 經 Server 執行，SHALL NOT 回報不支援：動工痕跡的守門、--force 的放行、成功時的人眼輸出與 --json payload SHALL 與本機模式相同；被守門拒絕時 SHALL 以非零 exit code 結束，stderr 的訊息 SHALL 指出動工痕跡與 --force，SHALL NOT 只印 HTTP 狀態碼；Server 上的變更 SHALL 不變。連不上 Server、登入憑證失效與版本衝突時的可觀察行為依 verb-contract 與 remote-connection 對 remote 寫入動詞的通則，本需求不另訂。變更目錄刪除失敗時 SHALL 以非零 exit code 回報，已完成的討論解鏈不回滾且輸出 SHALL 明示已解鏈清單。本指令為 Speclink 自有延伸；既有指令的人眼與 --json 輸出 SHALL 逐位元不變。

#### Scenario: 未動工變更成功廢棄

- **WHEN** 對 meta 無 started_at 且 tasks.md 無已勾任務的變更執行 speclink discard
- **THEN** exit code 0；speclink/changes/<change>/ 目錄消失；stdout 報告已刪除的變更名與解鏈結果

#### Scenario: 動工痕跡守衛拒絕

- **WHEN** 對有動工痕跡的變更執行 speclink discard 且未帶 --force
- **THEN** 非零 exit code；stderr 提示動工痕跡與 --force；speclink/ 下任何檔案逐位元不變

##### Example: 動工痕跡判定

| meta 有 started_at | tasks.md 有已勾任務 | 未帶 --force 的結果 |
| ------------------ | ------------------- | ------------------- |
| 否                 | 否                  | 放行                |
| 是                 | 否                  | 拒絕                |
| 否                 | 是                  | 拒絕                |
| 是                 | 是                  | 拒絕                |

#### Scenario: --force 放行動過工的變更

- **WHEN** 對有動工痕跡的變更執行 speclink discard --force
- **THEN** exit code 0；變更目錄與其 touched 紀錄檔皆刪除；stdout 報告同成功路徑

#### Scenario: 變更不存在報錯

- **WHEN** 執行 speclink discard 給定不存在的變更名
- **THEN** 非零 exit code；stderr 說明變更不存在；無任何檔案變動

#### Scenario: remote 模式沒帶 --force 被守門拒絕

- **WHEN** 於 remote 模式對 Server 上已有動工痕跡的變更 demo 執行 speclink discard demo
- **THEN** 非零 exit code；stderr 的訊息指出動工痕跡與 --force，不含 `not available in remote mode`，也不是只有 HTTP 狀態碼；Server 上的 demo 不變

#### Scenario: remote 模式帶 --force 成功

- **WHEN** 於 remote 模式執行 speclink discard demo --force --json，demo 連結了討論 auth
- **THEN** exit code 0；stdout 為與本機模式同形的 JSON：`change` 為 `demo`，`unlinkedDiscussions` 含一項 slug 為 `auth` 與它回退後的狀態；Server 上的 demo 已刪除

#### Scenario: --json 輸出 payload

- **WHEN** 執行 speclink discard <change> --json 成功廢棄
- **THEN** stdout 為 JSON：含變更名欄位與解鏈討論陣列（每項含 slug 與回退後狀態），欄位名一律 camelCase

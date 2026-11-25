# terminal-activity-report Specification

## Purpose

speclink CLI 跑在 Herdr（herdr-core）的終端機裡時，替自己所在的終端機回報「正在做哪個變更或討論」：哪些指令算數、貼哪些標籤、怎麼送、時間上限與關閉方式。本 capability 也保證不在 Herdr 裡、或回報失敗時，指令的輸出、結束碼與速度都和沒有這個功能時相同；Herdr 面板怎麼使用這些標籤不在本 capability 的範圍內。

## Requirements

### Requirement: 做變更或討論的指令成功後回報給所在的 Herdr 終端機

speclink CLI 執行下表的指令並以 exit code 0 結束時，如果程序的環境變數 `HC_TERMINAL_ID` 與 `HC_TERMINAL_SOCKET` 都有值（不是空字串）、`SPECLINK_TERMINAL_REPORT` 沒有關掉回報、而且平台提供 Unix domain socket，CLI SHALL 在程序結束前連線到 `HC_TERMINAL_SOCKET` 指的 Unix domain socket，送出恰好一則 herdr-core 的 JSON-RPC 2.0 請求；連不上或逾時時，依「回報不改變指令的結果也不啟動任何程序」安靜放棄。這則請求 SHALL 是一行 JSON、以換行結尾，`method` 為 `terminal.label`，`params.terminal` 為 `HC_TERMINAL_ID` 的原值，`params.labels` 為只含 `speclink.change` 與 `speclink.discussion` 兩個鍵的物件，值依下表。名稱與 slug SHALL 取自指令列上寫的值並原樣送出。值是空字串時，herdr-core 會移除那個標籤。

| 指令 | `speclink.change` | `speclink.discussion` |
| --- | --- | --- |
| `new change <名稱>` | `<名稱>` | 帶 `--from-discussion <slug>` 時為 `<slug>`，否則為空字串 |
| `instructions apply --change <名稱>` | `<名稱>` | 空字串 |
| 帶 `--change <名稱>` 的 `task done` | `<名稱>` | 空字串 |
| `in-progress add <名稱>` | `<名稱>` | 空字串 |
| `review prepare <名稱>` | `<名稱>` | 空字串 |
| 只帶一個名稱的 `archive <名稱>` | `<名稱>` | 空字串 |
| 帶 `--slug <slug>` 的 `discuss new` | 空字串 | `<slug>` |
| `discuss context <slug>`、`discuss add-round <slug>`、`discuss conclude <slug>` | 空字串 | `<slug>` |

`params.labels` SHALL NOT 含 `speclink.skill` 或其他鍵。表外的指令 SHALL NOT 送出請求，包括 `show`、`status`、`list`、`artifact`、`plan`、其他產物的 `instructions`、沒帶 `--change` 的 `task done`、沒帶 `--slug` 的 `discuss new`、帶多個名稱、沒帶名稱或帶 `--all` 的 `archive`、`--help`、`--version` 與參數解析失敗。表內的指令以非 0 的 exit code 結束時 SHALL NOT 送出請求。帶不帶 `--json`、`--no-color` 都適用同一規則，而且這些指令的人眼輸出、`--json` 輸出、stderr 與 exit code SHALL 和沒有這兩個環境變數時位元級相同。本機 store 與 remote store 兩種模式 SHALL 用同一規則：remote 模式下，請求 SHALL 只在 Server 接受指令、指令以 exit code 0 結束之後送出，回報 SHALL NOT 對 Server 多送任何請求。

#### Scenario: 完成任務後貼上變更標籤

- **WHEN** `HC_TERMINAL_ID` 為 `t2p`、`HC_TERMINAL_SOCKET` 指向一個在聽的 socket，執行 `speclink task done 1 --change add-auth` 並成功
- **THEN** socket 收到一行請求，`method` 為 `terminal.label`，`params` 為 `{"terminal":"t2p","labels":{"speclink.change":"add-auth","speclink.discussion":""}}`；stdout、stderr 與 exit code 和不設這兩個環境變數時相同

#### Scenario: --json 與 --no-color 的輸出不變

- **WHEN** 在同一個終端機執行 `speclink task done 2 --change add-auth --json`，再執行 `speclink task done 3 --change add-auth --no-color`
- **THEN** 兩次的 stdout 都和不設這兩個環境變數時位元級相同，`--json` 的欄位與值不變；socket 各收到一則與上一個情境相同形狀的請求

#### Scenario: 從討論建立變更時留著討論標籤

- **WHEN** 在 Herdr 終端機裡執行 `speclink new change add-auth --from-discussion auth-flow` 並成功
- **THEN** 請求的 `labels` 為 `{"speclink.change":"add-auth","speclink.discussion":"auth-flow"}`

#### Scenario: 討論的指令貼討論標籤並清掉變更標籤

- **WHEN** 在 Herdr 終端機裡執行 `speclink discuss add-round auth-flow --stdin` 並成功
- **THEN** 請求的 `labels` 為 `{"speclink.change":"","speclink.discussion":"auth-flow"}`

#### Scenario: 只讀或沒帶名稱的指令不回報

- **WHEN** 在 Herdr 終端機裡依序執行下例的指令，每一個都成功
- **THEN** socket 沒有收到任何連線

##### Example: 不回報的指令

| 指令 | 不回報的原因 |
| --- | --- |
| `speclink show add-auth` | 只讀 |
| `speclink status --change add-auth --json` | 只讀 |
| `speclink list --json` | 只讀 |
| `speclink artifact cat proposal --change add-auth` | 只讀 |
| `speclink instructions proposal --change add-auth --json` | 不是 apply 的指示 |
| `speclink task done 1` | 沒帶 `--change` |
| `speclink discuss new "登入流程"` | 沒帶 `--slug` |
| `speclink archive add-auth add-login` | 帶多個名稱，終端機一次只標一個變更 |

#### Scenario: 失敗的指令不回報

- **WHEN** 在 Herdr 終端機裡執行 `speclink task done 99 --change add-auth`，任務 99 不存在
- **THEN** exit code 不是 0，stderr 的錯誤訊息和不設這兩個環境變數時相同，socket 沒有收到任何連線

#### Scenario: remote 模式在 Server 接受後才回報

- **WHEN** 在連到遠端的 checkout、Herdr 終端機裡執行 `speclink task done 1 --change add-auth`，Server 接受了這個指令
- **THEN** socket 收到與本機模式相同的請求；Server 收到的請求和不設這兩個環境變數時相同

#### Scenario: remote 模式的指令被拒時不回報

- **WHEN** 在連到遠端的 checkout、Herdr 終端機裡執行 `speclink task done 1 --change add-auth`，Server 因 revision 衝突（409）拒絕、Server 連不上、或認證失效
- **THEN** exit code 不是 0，stderr 和不設這兩個環境變數時相同，socket 沒有收到任何連線


<!-- @trace
source: cli-terminal-activity-report
updated: 2026-11-25T17:16:14+08:00
-->

---
### Requirement: 不在 Herdr 終端機裡時不做任何事

`HC_TERMINAL_ID` 或 `HC_TERMINAL_SOCKET` 沒有設定或是空字串時，speclink CLI SHALL NOT 連線到任何 socket，SHALL NOT 啟動任何程序，SHALL NOT 讀寫任何為了回報而存在的檔案；所有指令的 stdout、stderr 與 exit code SHALL 和沒有本 capability 時位元級相同。CLI SHALL NOT 自行推測 herdr-core 的預設 socket 位置。在 Rust 標準函式庫不提供 Unix domain socket 的平台（Windows）上，即使兩個環境變數都有值，CLI 也 SHALL NOT 回報。

#### Scenario: 一般終端機裡完全照舊

- **WHEN** 在沒有 `HC_TERMINAL_ID` 與 `HC_TERMINAL_SOCKET` 的終端機執行上一個需求表內的任一個指令
- **THEN** 沒有任何 socket 連線、沒有子程序，stdout、stderr 與 exit code 和沒有本 capability 時相同

#### Scenario: 只有終端機代號時不猜 socket

- **WHEN** `HC_TERMINAL_ID` 為 `t2p`，沒有 `HC_TERMINAL_SOCKET`，執行 `speclink task done 1 --change add-auth`
- **THEN** CLI 不連線到任何 socket，也不執行 `hc`

#### Scenario: Windows 上不回報

- **WHEN** 在 Windows 上設好 `HC_TERMINAL_ID` 與 `HC_TERMINAL_SOCKET`，執行 `speclink task done 1 --change add-auth`
- **THEN** CLI 不回報，stdout、stderr 與 exit code 和不設這兩個環境變數時相同


<!-- @trace
source: cli-terminal-activity-report
updated: 2026-11-25T17:16:14+08:00
-->

---
### Requirement: 回報不改變指令的結果也不啟動任何程序

speclink CLI SHALL NOT 為了回報而執行 `hc` 或其他任何程式，SHALL 只連線到 `HC_TERMINAL_SOCKET` 寫的那個路徑，因此 SHALL NOT 啟動 herdr-core。整段回報（連線、送出請求、等待一行回覆）SHALL 在 100 毫秒內結束；到了上限，CLI SHALL 不再等待並照常結束，CLI 結束後 SHALL NOT 留下任何程序。socket 不存在、拒絕連線、路徑不是 socket、送不出去、沒有回覆、回覆是錯誤（例如 `not_found`、`invalid_params`）或不是 JSON 時，CLI SHALL NOT 印出任何內容，SHALL NOT 改變 stdout、stderr 與 exit code。回報 SHALL NOT 讀取 stdin。

#### Scenario: socket 不存在時不會啟動新的 core

- **WHEN** `HC_TERMINAL_ID` 為 `t2p`、`HC_TERMINAL_SOCKET` 指向一個不存在的路徑，`PATH` 上有一個被執行時會留下記號檔的 `hc`，執行 `speclink task done 1 --change add-auth`
- **THEN** 記號檔不存在，那個 socket 路徑上沒有建立任何檔案；stdout、stderr 與 exit code 和不設這兩個環境變數時相同

#### Scenario: core 不回覆時最多多等 100 毫秒

- **WHEN** `HC_TERMINAL_SOCKET` 指向一個接受連線但永遠不回覆的 socket，執行 `speclink task done 1 --change add-auth`
- **THEN** 指令在完成任務後最多再過 100 毫秒就結束，exit code 為 0，stdout 與 stderr 和不設這兩個環境變數時相同

#### Scenario: core 回覆錯誤時照常結束

- **WHEN** socket 對請求回覆 `{"jsonrpc":"2.0","id":1,"error":{"code":-32000,"message":"not_found","data":{"kind":"not_found"}}}`
- **THEN** exit code 為 0，stdout 與 stderr 和不設這兩個環境變數時相同

#### Scenario: 路徑不是 socket 時照常結束

- **WHEN** `HC_TERMINAL_SOCKET` 指向一個一般檔案，執行 `speclink task done 1 --change add-auth`
- **THEN** 那個檔案的內容不變，exit code 為 0，stdout 與 stderr 和不設這兩個環境變數時相同


<!-- @trace
source: cli-terminal-activity-report
updated: 2026-11-25T17:16:14+08:00
-->

---
### Requirement: 用 SPECLINK_TERMINAL_REPORT 關掉回報

環境變數 `SPECLINK_TERMINAL_REPORT` 去掉前後空白後、不分大小寫等於 `false` 時，speclink CLI SHALL NOT 回報，也 SHALL NOT 連線到任何 socket。沒有設定、或是其他任何值（包括 `true`、`0`、`no` 與空字串）時，回報 SHALL 照第一個需求進行：和 Speclink 其他布林環境變數相同，只認 `true` 與 `false`，其他值當成沒設。這個開關 SHALL 只是環境變數，`speclink/config.yaml`、`.speclink.yaml` 與使用者層級設定檔 SHALL NOT 多出對應的鍵。

#### Scenario: 設成 false 時不回報

- **WHEN** 在 Herdr 終端機裡設 `SPECLINK_TERMINAL_REPORT=false`，執行 `speclink task done 1 --change add-auth` 並成功
- **THEN** socket 沒有收到任何連線，stdout、stderr 與 exit code 和不設 `HC_TERMINAL_ID` 時相同

##### Example: 開關的值

| `SPECLINK_TERMINAL_REPORT` | 回報 |
| --- | --- |
| 沒有設定 | 有 |
| `false` | 沒有 |
| `FALSE` | 沒有 |
| ` false `（前後有空白） | 沒有 |
| `true` | 有 |
| `0` | 有 |
| `no` | 有 |
| 空字串 | 有 |

<!-- @trace
source: cli-terminal-activity-report
updated: 2026-11-25T17:16:14+08:00
-->
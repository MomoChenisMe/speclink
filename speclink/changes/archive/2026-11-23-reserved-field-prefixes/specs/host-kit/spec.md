## ADDED Requirements

### Requirement: 宿主保留的自由欄位鍵

宿主套件 SHALL 在建立時接受一組選填的保留前綴（字串清單，預設是空的）。鍵以其中任何一個前綴開頭的自由欄位是宿主保留的鍵；比對 SHALL 是單純的字串開頭比對（前綴 `host.` 保留 `host.project`；前綴 `host` 同時保留 `host`、`hostname` 與 `host.project`）。每個前綴 SHALL 符合自由欄位的鍵的規則（`^[a-z][a-z0-9_.-]{0,63}$`）；有不符合的前綴時 SHALL 建立不出宿主套件，並回報是哪一個前綴。重複的前綴 SHALL 視為一個。

經任何一個門來的呼叫，命令或請求自己要寫入或刪除宿主保留的鍵時，宿主套件 SHALL 拒絕這次呼叫：錯誤碼 SHALL 是 `refused`，訊息 SHALL 是 `field '<key>' is reserved by the host`（`<key>` 是那個鍵；有多個時是依位元組順序的第一個）；經 HTTP 入口時回應 SHALL 是 409、reason 為 refused。這個檢查 SHALL 涵蓋每一條從請求寫欄位的路：建立變更、建立討論、從討論轉出變更時請求帶的欄位，以及變更與討論的自由欄位的設定與移除。它 SHALL 先於執行前掛鉤：被拒絕時執行前掛鉤 SHALL NOT 被問，儲存 SHALL 連讀取都沒有收到，提交後掛鉤 SHALL NOT 被呼叫，變更與討論 SHALL 不變。一次呼叫同時有保留的鍵與不合規則的欄位時，SHALL 回報保留鍵的拒絕。

宿主在這次呼叫附上的欄位 SHALL NOT 受這個檢查限制：宿主保留的鍵 SHALL 只能經宿主附上的欄位寫入，行為照「呼叫入口時附上自由欄位」。命令或請求自己帶了宿主保留的鍵時，即使宿主這次也附上同名的鍵，呼叫 SHALL 被拒絕，SHALL NOT 以宿主的值蓋過後繼續。不是宿主保留的鍵 SHALL 不受影響。讀取 SHALL 照常輸出宿主保留的鍵與它的值；封存 SHALL 照常讓它們隨文件進入封存。範圍的匯入與匯出寫入與讀出的是整份文件，SHALL NOT 做這個檢查。

沒有提供保留前綴、或清單是空的時候，宿主套件的行為 SHALL 與沒有這條需求時相同。引擎 SHALL 仍然只存放自由欄位而不解讀任何鍵：哪些鍵是保留的 SHALL 只由宿主的設定決定，檢查 SHALL 只在宿主套件。沒有宿主套件的本機專案 SHALL 不受影響。

#### Scenario: 建立時請求寫保留的鍵被拒絕

- **WHEN** 宿主套件的保留前綴是 `host.`，宿主這次沒有附上任何欄位，使用者經 Agent 工具的門執行建立變更 spoof 並帶欄位 `host.project=p2`
- **THEN** 呼叫以 `refused` 失敗，訊息是 `field 'host.project' is reserved by the host`；變更 spoof 沒有被建立；儲存沒有收到任何呼叫；執行前掛鉤沒有被問

#### Scenario: 宿主附上的保留鍵照常寫入

- **WHEN** 保留前綴是 `host.`，宿主附上 `host.project=p1` 執行建立變更 ok，請求自己沒有帶欄位
- **THEN** 變更 ok 建立成功並有 `host.project=p1`；執行前掛鉤收到的參數的 fields 含 `host.project=p1`

#### Scenario: 請求與宿主帶同一個保留的鍵

- **WHEN** 保留前綴是 `host.`，宿主附上 `host.project=p1`，請求自己帶 `host.project=p2`，執行建立變更 both
- **THEN** 呼叫以 `refused` 失敗；變更 both 沒有被建立

#### Scenario: 設定與移除保留的鍵被拒絕

- **WHEN** 保留前綴是 `host.`，變更 ok 已有 `host.project=p1`，使用者分別執行把 `host.project` 設成 p2 的欄位設定，以及移除 `host.project` 的欄位移除
- **THEN** 兩次呼叫都以 `refused` 失敗；ok 的 `host.project` 仍是 p1

##### Example: 各條寫入路徑

| 動詞文字或請求 | 結果 |
| --- | --- |
| `new change spoof --field host.project=p2` | `refused` |
| `discuss new 主題 --field host.project=p2` | `refused` |
| `change field set ok host.project=p2` | `refused` |
| `change field unset ok host.project` | `refused` |
| `discuss field set talk host.project=p2` | `refused` |
| `discuss field unset talk host.project` | `refused` |
| `POST /discussions/talk/promote`，本文的 `fields` 含 `host.project` | 409、reason 為 refused |
| `PUT /changes/ok/fields`，本文設定或移除 `host.project` | 409、reason 為 refused |
| `PUT /discussions/talk/fields`，本文設定或移除 `host.project` | 409、reason 為 refused |
| `new change web --field team=web` | 成功 |
| `change field set ok team=web host.project=p2` | `refused`，`team` 也沒有被寫入 |

#### Scenario: 不是保留的鍵不受影響

- **WHEN** 保留前綴是 `host.`，請求帶 `team=cli`、宿主附上 `team=web` 執行建立變更 demo，之後使用者把 demo 的 `team` 設成 api
- **THEN** 建立時 `team` 是 web；之後的設定成功，`team` 是 api

#### Scenario: 讀取照常輸出保留的鍵

- **WHEN** 變更 ok 有 `host.project=p1`，使用者列出變更並讀取 ok
- **THEN** 兩個輸出的 fields 都含 `host.project` 與值 p1

#### Scenario: 沒有設定保留前綴時行為不變

- **WHEN** 宿主套件沒有提供保留前綴，使用者執行建立變更 demo 並帶欄位 `host.project=p2`
- **THEN** 變更建立成功並有 `host.project=p2`，與沒有這條需求時相同

#### Scenario: 前綴不合規則

- **WHEN** 分別以保留前綴 `Host.`、空字串與 `host project` 建立宿主套件
- **THEN** 三次都建立不出宿主套件，錯誤指出是哪一個前綴

#### Scenario: 前綴是單純的字串開頭比對

- **WHEN** 保留前綴是 `host`，使用者執行建立變更 demo 並帶欄位 `hostname=web1`
- **THEN** 呼叫以 `refused` 失敗，訊息指出 `hostname`

## MODIFIED Requirements

### Requirement: 呼叫入口時附上自由欄位

<!-- BEFORE: 同名時宿主的值一律蓋過命令或請求自己帶的值，沒有例外 -->

宿主套件的兩個門 SHALL 各接受宿主在這次呼叫附上的一組自由欄位。這組欄位 SHALL 只在這次呼叫會建立變更或討論時生效（建立變更、建立討論、從討論轉出變更）：它 SHALL 併入該命令要寫入的欄位，同名時宿主的值 SHALL 蓋過命令或請求自己帶的值，並 SHALL 與建立在同一次提交；例外是宿主保留的鍵：命令或請求自己帶了那種鍵時，這次呼叫 SHALL 被拒絕而不是被蓋過（見「宿主保留的自由欄位鍵」）。其他命令 SHALL 忽略它，SHALL NOT 因此失敗，也 SHALL NOT 產生任何寫入。合併 SHALL 先於執行前掛鉤，掛鉤收到的參數 SHALL 是最後要寫入的欄位。合併後不合自由欄位的規則時，這次呼叫 SHALL 以 invalid_argv 失敗且儲存不變。動詞文字、stdin 與 HTTP 請求的內容 SHALL NOT 能移除或改寫宿主附上的欄位。宿主 SHALL NOT 能藉這個輸入改變命令的其他部分。

#### Scenario: 建立變更時附上來源

- **WHEN** 宿主以附上的欄位 source=portal 經 Agent 工具的門執行建立變更 demo
- **THEN** demo 建立時就有 source=portal；儲存收到正好一次提交

#### Scenario: 宿主的值蓋過請求的值

- **WHEN** HTTP 請求的建立變更 payload 帶 fields source=cli，宿主在這次呼叫附上 source=portal
- **THEN** 建立的變更的 source 是 portal

#### Scenario: 其他命令忽略

- **WHEN** 宿主以同一組附上的欄位執行列出變更與勾選任務
- **THEN** 兩者照常成功；沒有任何變更的自由欄位被改動

#### Scenario: 掛鉤看到最後的欄位

- **WHEN** 宿主附上 source=portal 執行建立變更，並設有執行前掛鉤
- **THEN** 掛鉤收到的參數的 fields 含 source=portal

#### Scenario: 附上的欄位不合規則

- **WHEN** 宿主附上鍵為 Bad 的欄位執行建立變更
- **THEN** 呼叫以 invalid_argv 失敗；變更沒有被建立

#### Scenario: 握手宣告支援

- **WHEN** 對宿主套件的 HTTP 入口送出握手
- **THEN** capabilities 的 fields 為 true

## Purpose

變更與討論上的自由欄位：宿主或使用者自訂的鍵值，存在既有的 metadata 與討論記錄裡。本 capability 規定鍵與值的規則、寫入與移除的動詞、建立時附帶與讀取的輸出；引擎只存不解讀，依欄位做判斷屬於宿主。

## ADDED Requirements

### Requirement: 自由欄位的規則與存放

每個進行中的變更與每個進行中的討論 SHALL 能帶一組自由欄位：鍵對字串值。寫入時 SHALL 檢查：鍵符合 `^[a-z][a-z0-9_.-]{0,63}$`；值是 0 到 1024 個位元組的 UTF-8 字串，不含換行與其他控制字元；一份文件的鍵數不超過 32。任一項不符 SHALL 拒絕整次寫入且不留任何變動。變更的自由欄位 SHALL 存在其 metadata 的 `fields` 區塊，討論的 SHALL 存在討論記錄開頭區塊的 `fields` 區塊；SHALL NOT 為它新增文件種類。寫入的值 SHALL 讀回逐字相同——包含看起來像數字、布林、null 的值，以及含冒號、井號、引號、反斜線與非 ASCII 字元的值——寫入後該文件 SHALL 仍可被引擎解析。最後一個鍵被移除時 SHALL 一併移除 `fields` 區塊。引擎 SHALL 只存放自由欄位：SHALL NOT 解讀任何鍵或值，SHALL NOT 因為它們改變任何動詞的行為、順序或守門。引擎既有的每一個寫入動詞（含蓋章、變更 schema、宣告依賴、排序、封存）SHALL 原樣保留既有的自由欄位；封存 SHALL 讓自由欄位隨文件進入封存。讀取 SHALL 寬鬆：區塊內既有的純量值讀成字串，非純量的項目略過，SHALL NOT 因此讓整份文件無法讀取。

#### Scenario: 值讀回逐字相同

- **WHEN** 對變更 demo 分別寫入值 `123`、`true`、`null`、`a: b # c`、含雙引號與反斜線的字串、以及一段中文，之後讀出
- **THEN** 每個值與寫入時逐字相同，都是字串；demo 的 metadata 仍可解析，其餘欄位不變

#### Scenario: 鍵不合規則

- **WHEN** 以鍵 `Ticket`、`1st`、含空白的鍵、65 個字元的鍵分別寫入
- **THEN** 每一次都被拒絕，文件逐位元不變

#### Scenario: 值含換行

- **WHEN** 以含換行的值寫入
- **THEN** 被拒絕，文件逐位元不變

#### Scenario: 超過 32 個鍵

- **WHEN** 變更已有 32 個自由欄位，再寫入第 33 個不同的鍵
- **THEN** 被拒絕；既有的 32 個欄位不變

#### Scenario: 既有的寫入動詞保留欄位

- **WHEN** 對帶有自由欄位的變更依序執行勾選任務、宣告依賴、審查蓋章與封存
- **THEN** 每一步之後自由欄位都與原本相同；封存後的 metadata 含同一組欄位

#### Scenario: 引擎的行為不因欄位改變

- **WHEN** 兩個內容相同的變更，其中一個帶有任意的自由欄位（含名為 schema、depends_on 的鍵），分別執行 plan、validate 與 archive
- **THEN** 兩者的結果除了自由欄位本身以外相同

#### Scenario: 移除最後一個鍵

- **WHEN** 變更只有一個自由欄位，將它移除
- **THEN** metadata 不再有 `fields` 區塊

#### Scenario: 手改過的區塊仍可讀

- **WHEN** metadata 的 `fields` 區塊被人手改成含一個未加引號的數字值與一個巢狀的項目
- **THEN** 變更仍可列出；數字值讀成字串，巢狀的項目被略過

### Requirement: 自由欄位的寫入動詞

CLI SHALL 提供四個動詞，皆有 `--json` 與 `--no-color`、無 stdin：`speclink change field set <name> <key>=<value>...`、`speclink change field unset <name> <key>...`、`speclink discuss field set <slug> <key>=<value>...`、`speclink discuss field unset <slug> <key>...`。`<key>=<value>` SHALL 在第一個 `=` 分開，值可含 `=`；沒有 `=` 的參數、同一次呼叫重複的鍵 SHALL 為用法錯誤。一次呼叫的全部鍵 SHALL 全部寫入或全部不寫，並 SHALL 是對該文件的一次寫入。unset 不存在的鍵 SHALL 成功且不產生變動。變更或討論不存在、討論已封存 SHALL 以非零 exit code 回報找不到。成功時 stdout SHALL 為一行：`✓ <名稱>: set <鍵…>`、`✓ <名稱>: unset <鍵…>`，或沒有任何變動時的 `✓ <名稱>: no change`；`--json` 時 SHALL 為 `{ "change": string, "fields": object }`（討論為 `"discussion"`），`fields` 是寫入後的全部欄位。有變動時命令層 SHALL 回報一筆領域事件（change-fields-changed 或 discussion-fields-changed），payload SHALL 含名稱與變動的鍵，SHALL NOT 含值；沒有變動時 SHALL NOT 產生事件。`speclink new change` 與 `speclink discuss new` SHALL 接受可重複的 `--field <key>=<value>`：欄位 SHALL 與建立在同一次寫入，規則不符時建立 SHALL 整個失敗。四個動詞與 `--field` SHALL 經命令執行層，本機、remote 模式、Node SDK 的 dispatch 與宿主套件的兩個門 SHALL 得到相同的結果。

#### Scenario: 設定兩個欄位

- **WHEN** 執行 speclink change field set demo ticket=PROJ-1 source=portal
- **THEN** stdout 為 `✓ demo: set ticket, source`（鍵依位元組順序），exit code 為 0；demo 的自由欄位是這兩組

#### Scenario: --json 回傳全部欄位

- **WHEN** demo 已有 source，執行 speclink change field set demo ticket=PROJ-1 --json
- **THEN** stdout 為 `{"change":"demo","fields":{"source":"portal","ticket":"PROJ-1"}}` 的等價 JSON

#### Scenario: 值含等號

- **WHEN** 執行 speclink change field set demo query=a=b
- **THEN** 欄位 query 的值是 `a=b`

#### Scenario: 全部寫入或全部不寫

- **WHEN** 執行 speclink change field set demo ok=1 Bad=2
- **THEN** exit code 非零；欄位 ok 沒有被寫入

#### Scenario: 移除欄位

- **WHEN** 執行 speclink change field unset demo ticket ghost
- **THEN** stdout 為 `✓ demo: unset ticket`，exit code 為 0

#### Scenario: 沒有變動

- **WHEN** 對沒有欄位 ghost 的 demo 執行 speclink change field unset demo ghost
- **THEN** stdout 為 `✓ demo: no change`，文件逐位元不變，沒有事件

#### Scenario: 討論的欄位

- **WHEN** 執行 speclink discuss field set board-search owner=alice --json
- **THEN** stdout 含 `"discussion":"board-search"` 與 fields；討論記錄的開頭區塊含 `fields`，各輪的內容不變

#### Scenario: 已封存的討論

- **WHEN** 對已封存的討論執行 speclink discuss field set
- **THEN** exit code 非零，stderr 說明找不到

#### Scenario: 建立時附帶

- **WHEN** 執行 speclink new change demo --field ticket=PROJ-1 --field source=portal
- **THEN** demo 建立，其 metadata 在建立當下就含兩個欄位；經儲存接口執行時是正好一次提交

#### Scenario: 建立時欄位不合規則

- **WHEN** 執行 speclink new change demo --field Bad=1
- **THEN** exit code 非零；demo 沒有被建立

#### Scenario: 事件只帶鍵

- **WHEN** 經儲存接口執行 change field set demo secret=abc
- **THEN** 提交帶一筆 change-fields-changed 事件，payload 含 demo 與鍵 secret，不含 abc

#### Scenario: remote 模式與本機同形

- **WHEN** 分別於 fs 模式與 remote 模式執行 speclink change field set demo ticket=PROJ-1 --json
- **THEN** 兩模式的 stdout 相同

### Requirement: 自由欄位的讀取輸出

下列 `--json` 輸出 SHALL 增列選填的 `fields` 物件（字串對字串）：`speclink list --json` 的每個變更項目、`speclink show <change> --json`、`speclink discuss list --json`、`speclink discuss show --json` 與 `speclink discuss search --json` 的討論資訊。文件沒有任何自由欄位時 SHALL 不輸出 `fields` 鍵，使沒有用到自由欄位的專案的輸出維持既有的位元級輸出。人眼輸出 SHALL NOT 因自由欄位改變。remote 模式的輸出 SHALL 與 fs 模式同形。

#### Scenario: 清單帶出欄位

- **WHEN** demo 有兩個自由欄位，other 沒有，執行 speclink list --json
- **THEN** demo 的項目含 fields 物件，other 的項目沒有 fields 鍵

#### Scenario: 沒有欄位的專案輸出不變

- **WHEN** 專案裡沒有任何自由欄位，比對新增這個能力前後 speclink list --json、speclink show demo --json 與 speclink discuss list --json 的輸出
- **THEN** 三者都逐位元相同

#### Scenario: 人眼輸出不變

- **WHEN** 對有自由欄位的 demo 執行 speclink show demo 與 speclink list
- **THEN** 輸出與 demo 沒有自由欄位時相同

#### Scenario: 討論資訊帶出欄位

- **WHEN** 討論 board-search 有欄位 owner，執行 speclink discuss show board-search --json
- **THEN** info 物件含 `"fields":{"owner":"alice"}`

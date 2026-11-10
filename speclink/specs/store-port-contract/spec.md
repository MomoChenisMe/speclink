# store-port-contract Specification

## Purpose

由宿主實作的儲存契約：列出、讀取、提交三個操作的輸入與輸出，提交的三項保證，文件鍵的格式與命名規則，版本號的型別，以及檢查任何實作是否符合契約的合格測試與記憶體版參考實作。本 capability 的邊界止於契約本身——引擎怎麼經它讀寫、Node 的綁定、匯入匯出都不在這裡。

## Requirements

### Requirement: 儲存契約只有三個操作

儲存契約 SHALL 只有三個操作：列出、讀取、提交。每個操作 MUST 以一個範圍為對象。同一個儲存物件 MUST 能被多個執行緒同時呼叫。契約 SHALL NOT 包含建立範圍、遷移資料表、健康檢查、讀回事件、歷史查詢或匯入匯出的操作。

#### Scenario: 從沒寫過的範圍

- **WHEN** 呼叫端對一個從沒寫過的範圍執行列出與讀取
- **THEN** 列出回傳範圍版本號 0 與空的清單，讀取對每一個要求的鍵回傳「沒有」，兩者都不是錯誤

#### Scenario: 第一次提交不需要先建立範圍

- **WHEN** 呼叫端對一個從沒寫過的範圍提交一份新文件
- **THEN** 提交成功並回傳大於 0 的範圍版本號


<!-- @trace
source: store-port-contract
updated: 2026-11-10T23:52:03+08:00
-->

---
### Requirement: 文件鍵的格式

文件鍵 MUST 是 1 到 512 個位元組的 UTF-8 字串，以 `/` 分段。文件鍵 SHALL NOT 含控制字元（U+0000 到 U+001F、U+007F）或反斜線，SHALL NOT 以 `/` 開頭或結尾，SHALL NOT 有空的段，SHALL NOT 有內容正好是 `.` 或 `..` 的段。範圍的識別 MUST 是 1 到 256 個位元組、不含控制字元的字串。不合格式的值 MUST 在建立時被拒絕，並回報是哪個值違反哪一條規則；儲存的實作 SHALL NOT 收到不合格式的鍵。

#### Scenario: 合法與不合法的鍵

- **WHEN** 呼叫端用一個字串建立文件鍵
- **THEN** 合乎格式的字串建立成功，不合格式的字串被拒絕並附上原因

##### Example: 鍵的邊界

| 輸入 | 結果 | 說明 |
| ---- | ---- | ---- |
| `changes/add-x/proposal.md` | 成功 | 一般的鍵 |
| `changes/add-x/.meta.yaml` | 成功 | 段以 `.` 開頭是允許的 |
| （空字串） | 拒絕 | 至少 1 個位元組 |
| 513 個位元組的字串 | 拒絕 | 超過 512 個位元組 |
| `/config.yaml` | 拒絕 | 以 `/` 開頭 |
| `changes/` | 拒絕 | 以 `/` 結尾 |
| `changes//proposal.md` | 拒絕 | 空的段 |
| `changes/../config.yaml` | 拒絕 | 段是 `..` |
| `changes\add-x` | 拒絕 | 含反斜線 |


<!-- @trace
source: store-port-contract
updated: 2026-11-10T23:52:03+08:00
-->

---
### Requirement: 版本號的型別與遞增

範圍版本號與文件版本 MUST 是 0 到 9007199254740991 的整數。最後修改時間與提交時間 MUST 是 Unix epoch 起算的毫秒數，範圍相同。從沒寫過的範圍，版本號 MUST 是 0。每次成功的提交之後，範圍版本號 MUST 嚴格大於提交前的值。失敗的提交 SHALL NOT 改變範圍版本號。文件版本 MUST 等於最後一次寫入該文件的那次提交所回傳的範圍版本號。

#### Scenario: 改寫讓版本前進

- **WHEN** 呼叫端以正確的預期版本改寫一份文件
- **THEN** 提交回傳的範圍版本號大於改寫前的值，該文件的版本等於這個新值，同一個範圍裡沒被這次提交碰到的文件版本與最後修改時間都不變

#### Scenario: 刪除後重建的文件拿到更大的版本

- **WHEN** 呼叫端刪除一份文件，再以「必須不存在」建立同一個鍵
- **THEN** 重建後的文件版本大於刪除前的版本

#### Scenario: 超出範圍的版本號被拒絕

- **WHEN** 呼叫端用 9007199254740992 建立版本號
- **THEN** 建立被拒絕，並回報超出上限


<!-- @trace
source: store-port-contract
updated: 2026-11-10T23:52:03+08:00
-->

---
### Requirement: 列出回傳排序後的鍵與版本

列出 MUST 收一個範圍與一個前綴，回傳該範圍目前的版本號，以及每份鍵以該前綴開頭的文件的鍵、文件版本、最後修改時間。前綴比對 MUST 是位元組層級的開頭相符，不認得分段。空的前綴 MUST 符合全部的鍵。結果 MUST 依鍵的位元組順序遞增排列。已刪除的文件 SHALL NOT 出現在結果裡。

#### Scenario: 依前綴列出

- **WHEN** 呼叫端以一個前綴列出
- **THEN** 結果只含鍵以該前綴開頭的文件，依鍵的位元組順序遞增，每一筆帶文件版本與最後修改時間

##### Example: 前綴不認得分段

- **GIVEN** 範圍裡有 `changes/add-x/proposal.md`、`changes/addendum/tasks.md`、`specs/auth/spec.md`
- **WHEN** 以前綴 `changes/add` 列出
- **THEN** 結果依序是 `changes/add-x/proposal.md`、`changes/addendum/tasks.md`

#### Scenario: 空的前綴列出全部

- **WHEN** 呼叫端以空的前綴列出
- **THEN** 結果含該範圍全部現存的文件


<!-- @trace
source: store-port-contract
updated: 2026-11-10T23:52:03+08:00
-->

---
### Requirement: 讀取一次收多個鍵

讀取 MUST 收一個範圍與一串鍵，回傳與這串鍵等長、順序相同的結果。每一項 MUST 是文件的內容、文件版本與最後修改時間，或「沒有」。不存在或已刪除的鍵 MUST 回傳「沒有」，SHALL NOT 是錯誤。同一個鍵出現兩次時，兩個位置 MUST 回傳相同的結果。空的要求 MUST 回傳空的結果。文件的內容 MUST 與寫入時逐位元組相同。一次讀取回傳的全部文件 MUST 來自同一個已完成提交之後的狀態，SHALL NOT 含某次提交只完成一部分的內容。

#### Scenario: 結果與要求的鍵對齊

- **WHEN** 呼叫端一次讀取多個鍵，其中有不存在的鍵與重複的鍵
- **THEN** 結果的長度與順序和要求相同，不存在的位置是「沒有」，重複的位置結果相同

##### Example: 對齊

- **GIVEN** 範圍裡只有 `config.yaml`（內容 `a`）
- **WHEN** 依序讀取 `config.yaml`、`LANGUAGE.md`、`config.yaml`
- **THEN** 結果依序是：內容 `a` 的文件、「沒有」、內容 `a` 的文件

#### Scenario: 內容原樣讀回

- **WHEN** 呼叫端寫入空字串、多位元組文字、含 CRLF 換行的文字、1 MiB 的文字，再讀回
- **THEN** 每一份讀回的內容與寫入時逐位元組相同

#### Scenario: 批次讀取看不到提交到一半的內容

- **WHEN** 一個執行緒連續 200 次在同一次提交裡改寫兩份文件，另一個執行緒反覆一次讀取這兩份文件直到寫入結束，結束後再讀一次
- **THEN** 每次讀到的兩份文件都屬於同一次提交


<!-- @trace
source: store-port-contract
updated: 2026-11-10T23:52:03+08:00
-->

---
### Requirement: 提交全部成功或全部不做

提交 MUST 一次帶：操作者的識別與顯示名稱、命令名稱、提交時間、一串寫入與刪除、一串事件。每一筆寫入 MUST 帶預期——某個版本，或「必須不存在」；每一筆刪除 MUST 帶預期的版本。一次提交的全部寫入與刪除 MUST 全部生效或全部不生效。被寫入的文件，最後修改時間 MUST 等於這次的提交時間。同一個鍵出現在同一次提交的兩筆變動裡，或寫入、刪除與事件全部都是空的，MUST 在建立提交時被拒絕，儲存的實作 SHALL NOT 收到這樣的提交。只有事件、沒有寫入與刪除的提交 MUST 成功並讓範圍版本號前進。

#### Scenario: 成功的提交

- **WHEN** 呼叫端提交兩份新文件，預期都是「必須不存在」
- **THEN** 提交回傳新的範圍版本號；之後讀取與列出都看得到這兩份文件，文件版本等於這個版本號，最後修改時間等於提交時間

#### Scenario: 刪除

- **WHEN** 呼叫端以正確的預期版本刪除一份文件
- **THEN** 提交成功，範圍版本號變大，之後讀取該鍵回傳「沒有」，列出的結果不含該鍵

#### Scenario: 只有事件的提交

- **WHEN** 呼叫端提交一筆事件，沒有寫入也沒有刪除
- **THEN** 提交成功，範圍版本號變大，全部文件的內容、版本與最後修改時間都不變

#### Scenario: 不合法的提交在建立時被拒絕

- **WHEN** 呼叫端建立一個同一個鍵出現兩次的提交，或一個沒有任何寫入、刪除與事件的提交
- **THEN** 建立被拒絕並附上原因，沒有任何東西送到儲存


<!-- @trace
source: store-port-contract
updated: 2026-11-10T23:52:03+08:00
-->

---
### Requirement: 版本對不上時整筆拒絕

儲存的實作 MUST 在套用任何變動之前確認每一筆寫入與刪除的預期都相符。任何一筆不符時，提交 MUST 失敗並回報衝突，衝突 MUST 指出其中一個不符的鍵、呼叫端的預期、文件目前的版本（文件不存在時是「沒有」）。衝突的提交 SHALL NOT 改變任何文件，SHALL NOT 改變範圍版本號，SHALL NOT 留下任何事件。多個呼叫端同時用同一個預期版本改寫同一份文件時，MUST 正好一筆成功，其餘都是衝突。

#### Scenario: 預期不符的三種情況

- **WHEN** 呼叫端的預期與文件目前的狀態不符
- **THEN** 提交失敗並回報衝突，指出鍵、預期與實際

##### Example: 衝突的對照

| 文件目前的狀態 | 變動 | 預期 | 回報的實際 |
| -------------- | ---- | ---- | ---------- |
| 版本 7 | 寫入 | 版本 5 | 版本 7 |
| 版本 7 | 刪除 | 版本 5 | 版本 7 |
| 版本 7 | 寫入 | 必須不存在 | 版本 7 |
| 不存在 | 寫入 | 版本 5 | 沒有 |
| 不存在 | 刪除 | 版本 5 | 沒有 |

#### Scenario: 一筆衝突讓整次提交不寫

- **WHEN** 呼叫端提交三筆變動與一筆事件，前兩筆的預期相符，最後一筆不符
- **THEN** 提交回報衝突；前兩筆沒有寫入，範圍版本號與提交前相同，事件沒有留下

#### Scenario: 同時改寫只有一筆成功

- **WHEN** 八個執行緒同時用同一個預期版本改寫同一份文件
- **THEN** 正好一筆提交成功，其餘七筆回報衝突，文件的內容是成功那一筆寫入的內容


<!-- @trace
source: store-port-contract
updated: 2026-11-10T23:52:03+08:00
-->

---
### Requirement: 事件與寫入同一次提交

事件 MUST 在提交裡和寫入一起交給儲存。儲存的實作要不要保存事件由實作決定。保存事件的實作 MUST 讓事件與文件的變動一起生效：成功的提交 MUST 保存它的全部事件，順序與提交裡相同，並能對應到這次提交的範圍版本號；失敗的提交 SHALL NOT 保存任何事件。契約 SHALL NOT 提供讀回事件的操作。

#### Scenario: 成功的提交保存事件

- **WHEN** 呼叫端對一個保存事件的實作提交兩筆事件與一份文件
- **THEN** 該實作保存這兩筆事件，順序與提交裡相同，並標上這次提交回傳的範圍版本號

#### Scenario: 失敗的提交沒有事件

- **WHEN** 呼叫端對一個保存事件的實作提交一筆會衝突的變動與一筆事件
- **THEN** 提交回報衝突，該實作沒有保存這筆事件


<!-- @trace
source: store-port-contract
updated: 2026-11-10T23:52:03+08:00
-->

---
### Requirement: 範圍彼此隔離

一個範圍裡的文件、版本號與事件 SHALL NOT 影響另一個範圍。兩個範圍 MUST 能使用相同的鍵而互不干擾。

#### Scenario: 兩個範圍用同一個鍵

- **WHEN** 呼叫端在範圍甲提交 `config.yaml`，再對範圍乙列出並讀取 `config.yaml`
- **THEN** 範圍乙的列出回傳版本號 0 與空的清單，讀取回傳「沒有」；範圍甲的版本號不因對範圍乙的操作而改變


<!-- @trace
source: store-port-contract
updated: 2026-11-10T23:52:03+08:00
-->

---
### Requirement: 儲存失敗的分類

儲存的實作回報的失敗 MUST 是三類之一：衝突（預期不符）、暫時無法使用（例如連不上資料庫或逾時，呼叫端能重試）、其他後端錯誤。後兩類 MUST 帶一段說明文字。值的格式錯誤 MUST 是另一種失敗，在建立值時回報，SHALL NOT 由儲存的實作回報。讀取到不存在的文件 SHALL NOT 是失敗。

#### Scenario: 暫時無法使用與後端錯誤分開

- **WHEN** 儲存的實作連不上它的後端
- **THEN** 操作回報「暫時無法使用」並附說明，不回報成衝突，也不回報成「沒有」


<!-- @trace
source: store-port-contract
updated: 2026-11-10T23:52:03+08:00
-->

---
### Requirement: 鍵的命名規則

契約 MUST 提供建立各種文件的鍵的函式，這些函式 SHALL NOT 做任何 I/O。每種文件的鍵 MUST 符合下面的對照；已封存的文件 MUST 放在最上層的 `archive/` 底下，讓以 `changes/` 或 `discussions/` 為前綴的列出不含已封存的文件。名稱的段是空的或含 `/` 時，建立 MUST 被拒絕（artifact 與已封存文件的路徑能有多段）。

#### Scenario: 各種文件的鍵

- **WHEN** 呼叫端為一種文件建立鍵
- **THEN** 得到的鍵符合對照表

##### Example: 鍵的對照

| 文件 | 輸入 | 鍵 |
| ---- | ---- | -- |
| 進行中變更的 metadata | 變更 `add-x` | `changes/add-x/.meta.yaml` |
| 進行中變更的 artifact | 變更 `add-x`、artifact `specs/auth/spec.md` | `changes/add-x/specs/auth/spec.md` |
| 進行中變更的 evidence | 變更 `add-x` | `changes/add-x/.evidence.json` |
| 已封存變更的文件 | `2026-11-10-add-x`、文件 `proposal.md` | `archive/changes/2026-11-10-add-x/proposal.md` |
| 正式規格 | capability `auth` | `specs/auth/spec.md` |
| 討論 | slug `board-search` | `discussions/board-search.md` |
| 已封存的討論 | 建立日 `2026-11-10`、slug `board-search` | `archive/discussions/2026-11-10-board-search.md` |
| 流程設定 | （無） | `config.yaml` |
| 詞彙表 | （無） | `LANGUAGE.md` |
| 看板順序 | （無） | `board/order` |

#### Scenario: 列出進行中的變更不含已封存的

- **WHEN** 範圍裡同時有進行中變更與已封存變更的文件，呼叫端以前綴 `changes/` 列出
- **THEN** 結果只含進行中變更的文件

#### Scenario: 不合法的名稱被拒絕

- **WHEN** 呼叫端用空字串或含 `/` 的字串當變更名稱、capability 名稱或討論的 slug
- **THEN** 建立被拒絕並附上原因


<!-- @trace
source: store-port-contract
updated: 2026-11-10T23:52:03+08:00
-->

---
### Requirement: 記憶體版參考實作

契約 MUST 附一份把資料放在記憶體裡的參考實作。它 MUST 支援多個範圍，MUST 保存每次成功提交的事件，並 MUST 通過合格測試的全部檢查，沒有任何一項略過。它 MUST 提供讀出某個範圍已保存事件的方法，供測試使用；這個方法 SHALL NOT 屬於契約的三個操作。

#### Scenario: 參考實作通過合格測試

- **WHEN** 對記憶體版參考實作執行合格測試
- **THEN** 報告是通過，每一項檢查的結果都是通過，沒有略過


<!-- @trace
source: store-port-contract
updated: 2026-11-10T23:52:03+08:00
-->

---
### Requirement: 合格測試對任何實作跑同一組檢查

契約 MUST 附一套合格測試，以一個入口對任何實作執行同一組檢查，並回傳一份報告。報告 MUST 列出每一項檢查的穩定名稱與結果：通過、失敗並附說明、或略過並附理由。任何一項失敗時，整份報告 MUST 是不通過。受測的實作回傳非預期的結果時，該項 MUST 記為失敗，其餘檢查 MUST 照常執行，合格測試 SHALL NOT 中途中止。合格測試 MUST 經一個測試用的轉接取得受測的儲存、取得空的範圍、觀察已保存的事件；轉接回報實作不保存事件時，事件的檢查 MUST 標為略過，SHALL NOT 算失敗。合格測試 SHALL NOT 要求受測的實作提供故障注入點。

檢查的名稱 MUST 是：`empty-scope`、`put-then-read`、`list-prefix-order`、`read-alignment`、`update-advances-version`、`delete-then-recreate`、`conflict-stale-version`、`conflict-absent-but-exists`、`conflict-missing-document`、`conflict-rejects-whole-commit`、`events-with-commit`、`events-only-commit`、`scope-isolation`、`concurrent-single-winner`、`batch-read-atomic`、`content-roundtrip`。

#### Scenario: 邊檢查邊寫入的實作被抓到

- **WHEN** 對一個「逐筆檢查預期、相符就立刻寫入」的實作執行合格測試
- **THEN** 報告是不通過，`conflict-rejects-whole-commit` 的結果是失敗並附說明

#### Scenario: 不檢查預期版本的實作被抓到

- **WHEN** 對一個忽略預期、一律寫入的實作執行合格測試
- **THEN** 報告是不通過，`conflict-stale-version`、`conflict-absent-but-exists`、`conflict-missing-document` 的結果都是失敗

#### Scenario: 提交失敗仍保存事件的實作被抓到

- **WHEN** 對一個在檢查預期之前就保存事件的實作執行合格測試
- **THEN** 報告是不通過，`conflict-rejects-whole-commit` 的結果是失敗

#### Scenario: 批次讀取回傳不同提交內容的實作被抓到

- **WHEN** 對一個批次讀取時第二份文件回傳上一次提交內容的實作執行合格測試
- **THEN** 報告是不通過，`batch-read-atomic` 的結果是失敗

#### Scenario: 不保存事件的實作

- **WHEN** 對一個其他行為都正確、但不保存事件的實作執行合格測試，轉接回報「不保存事件」
- **THEN** 報告是通過，`events-with-commit` 的結果是略過並附理由，其餘檢查都是通過

#### Scenario: 一項失敗不影響其他檢查

- **WHEN** 受測的實作讓其中一項檢查失敗
- **THEN** 報告仍列出全部十六項檢查的結果，其餘檢查各自依實際行為記為通過、失敗或略過

<!-- @trace
source: store-port-contract
updated: 2026-11-10T23:52:03+08:00
-->
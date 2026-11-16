# store-port-examples Specification

## Purpose

三個有持久化的儲存接口範例（SQLite、PostgreSQL、檔案系統）共同的行為：通過儲存契約的合格測試、把事件記在自己的事件記錄並依序號讀回、辨認自己的配置並拒絕不認得的資料、重新開啟與中斷後的一致性、兩個儲存物件開同一份資料時的行為。本 capability 的邊界止於範例本身——契約的內容屬於 store-port-contract，示範用 server 怎麼用這些範例不在這裡。

## Requirements

### Requirement: 三個範例通過合格測試

SQLite、PostgreSQL 與檔案系統三個範例 SHALL 各自只以列出、讀取、提交三個操作提供儲存契約，並 SHALL 通過儲存契約的合格測試：全部檢查的結果是通過，SHALL NOT 有任何一項失敗或略過（含兩項事件的檢查）。每份文件回報的最後修改時間 SHALL 是最後寫入它的那次提交帶的時間。每個範圍 SHALL 有自己的版本號：對一個範圍提交 SHALL NOT 改變另一個範圍的版本號與文件。

#### Scenario: 合格測試全部通過

- **WHEN** 對一個剛初始化的 SQLite 檔案、一個空的 PostgreSQL schema、一個空的資料夾各自開啟範例並執行合格測試
- **THEN** 三份報告都是 16 項通過、0 項失敗、0 項略過

#### Scenario: 最後修改時間來自提交

- **WHEN** 以時間 1700000000000（毫秒）提交一筆對鍵 changes/a/proposal.md 的寫入，再讀取這個鍵
- **THEN** 讀到的文件的最後修改時間是 1700000000000

#### Scenario: 範圍各有版本號

- **WHEN** 對範圍 A 成功提交兩次、對範圍 B 成功提交一次，再分別列出兩個範圍
- **THEN** 範圍 A 的版本號是 2，範圍 B 的版本號是 1；範圍 B 沒有範圍 A 的文件


<!-- @trace
source: store-port-examples
updated: 2026-11-16T21:59:35+08:00
-->

---
### Requirement: 事件與寫入同一次交易並可依序號讀回

提交帶的事件 SHALL 與該次提交的寫入在同一次交易裡寫進範例的事件記錄。每個範圍的事件 SHALL 有序號：從 1 開始，每筆加 1；同一次提交的事件依它們在提交裡的順序連續編號。範例 SHALL 提供「讀某個序號之後的事件」：輸入是範圍、一個序號與筆數上限，回傳序號大於它的事件，依序號由小到大，每筆帶序號、事件名稱、內容與該次提交之後的範圍版本號，筆數不超過上限。被拒絕的提交 SHALL NOT 留下任何事件，SHALL NOT 用掉序號。只帶事件、不帶寫入的提交 SHALL 同樣記下它的事件。

#### Scenario: 一次提交的兩筆事件連續編號

- **WHEN** 對一個沒有事件的範圍提交一筆寫入並帶 artifact-written 與 change-ranked 兩筆事件，再讀序號 0 之後的事件
- **THEN** 回傳兩筆：序號 1 是 artifact-written、序號 2 是 change-ranked，兩筆的範圍版本號都是 1，內容與提交時給的相同

#### Scenario: 被拒絕的提交不留事件

- **WHEN** 範圍已有序號 1 的事件，一次帶事件的提交因為預期版本不符被拒絕，之後另一次帶一筆事件的提交成功
- **THEN** 讀序號 0 之後的事件得到兩筆，序號是 1 與 2；被拒絕的那次提交的事件不在其中

#### Scenario: 從中間讀起並受筆數上限限制

- **WHEN** 範圍有序號 1 到 5 的事件，以序號 2、上限 2 讀取
- **THEN** 回傳序號 3 與 4 兩筆

#### Scenario: 沒有事件的範圍

- **WHEN** 對一個從未提交過的範圍讀序號 0 之後的事件
- **THEN** 回傳空的清單，不是錯誤


<!-- @trace
source: store-port-examples
updated: 2026-11-16T21:59:35+08:00
-->

---
### Requirement: 清除舊事件與下限

範例 SHALL 提供「清掉某個序號以前（含該序號）的事件」與「回報某個範圍的下限」。下限是已經被清掉的最大序號；沒有清過時是 0。清除之後，讀取 SHALL 只回傳序號大於下限的事件；之後提交的事件 SHALL 接著原本的序號繼續編，SHALL NOT 重用被清掉的序號。清除 SHALL NOT 改變任何文件與範圍的版本號。給的序號大於目前最大的序號時，SHALL 只清到目前最大的序號，下限等於目前最大的序號。給的序號小於或等於下限時 SHALL 不做任何事。關閉並重新開啟儲存之後，下限 SHALL 不變。

#### Scenario: 清除後只讀得到較新的事件

- **WHEN** 範圍有序號 1 到 5 的事件，清掉序號 3 以前的事件，再讀序號 0 之後的事件
- **THEN** 回傳序號 4 與 5 兩筆；下限是 3；範圍的版本號與文件不變

#### Scenario: 清除後序號繼續往上編

- **WHEN** 範圍有序號 1 到 5 的事件，清掉序號 5 以前的事件，再提交一筆帶一筆事件的寫入
- **THEN** 新事件的序號是 6

#### Scenario: 清除的序號超過目前最大的序號

- **WHEN** 範圍有序號 1 到 5 的事件，清掉序號 100 以前的事件，再提交一筆帶一筆事件的寫入
- **THEN** 下限是 5；新事件的序號是 6


<!-- @trace
source: store-port-examples
updated: 2026-11-16T21:59:35+08:00
-->

---
### Requirement: 開啟時只認自己的配置

每個範例 SHALL 在資料的固定位置寫入識別標記 `speclink-store-port` 與配置版本：SQLite 與 PostgreSQL 是 meta 資料表的 `format` 與 `layout_version` 兩個鍵；檔案系統是資料夾根目錄的 meta.json 的 `format` 與 `layout_version` 兩個欄位。開啟時 SHALL 依下列規則：

- 位置是空的 SHALL 初始化：SQLite 的檔案不存在或沒有任何資料表；PostgreSQL 的 schema 沒有 meta 資料表；資料夾不存在或是空的。
- 標記是 `speclink-store-port` 且配置版本是範例支援的 SHALL 開啟，SHALL NOT 改動既有的資料。
- 其他情況 SHALL 拒絕開啟：標記是別的值；配置版本比範例支援的新；meta 存在但沒有標記；SQLite 的檔案有資料表但沒有 meta；資料夾不是空的但沒有 meta.json。

拒絕時 SHALL NOT 建立或改動任何資料表與檔案，錯誤 SHALL 帶出讀到的標記（沒有標記時是空值）與讀到的配置版本（沒有時是空值）。初始化 SHALL 是全有或全無：初始化在中途被中斷之後再開啟，SHALL 得到一個可用的空儲存，SHALL NOT 把只寫了一半的配置當成合格的儲存。

#### Scenario: 空的位置初始化

- **WHEN** 對一個不存在的 SQLite 檔案路徑開啟範例
- **THEN** 開啟成功；檔案被建立；列出任何範圍都得到版本號 0 與沒有文件；meta 的 format 是 `speclink-store-port`

#### Scenario: 重新開啟自己的資料

- **WHEN** 一個範例寫入過文件後關閉，再對同一個位置開啟
- **THEN** 開啟成功；先前的文件、版本號與事件都在

#### Scenario: 別的標記被拒絕

- **WHEN** 對一個 meta 的 format 是 `speclink-team-store` 的 SQLite 檔案開啟範例
- **THEN** 開啟失敗，錯誤帶出讀到的標記 `speclink-team-store`；檔案的內容逐位元不變

#### Scenario: 較新的配置版本被拒絕

- **WHEN** 對一個標記正確、layout_version 是 2 的資料夾開啟只支援版本 1 的檔案系統範例
- **THEN** 開啟失敗，錯誤帶出配置版本 2；資料夾的內容不變

#### Scenario: 有內容但沒有標記

- **WHEN** 對一個放著其他檔案、沒有 meta.json 的資料夾開啟檔案系統範例
- **THEN** 開啟失敗，錯誤的標記是空值；資料夾裡沒有新增任何檔案

#### Scenario: PostgreSQL 的 schema 有其他資料表但沒有 meta

- **WHEN** 對一個只有不相干資料表、沒有 meta 資料表的 schema 開啟 PostgreSQL 範例
- **THEN** 開啟成功並初始化；不相干的資料表沒有被改動


<!-- @trace
source: store-port-examples
updated: 2026-11-16T21:59:35+08:00
-->

---
### Requirement: 重新開啟與中斷後的一致性

提交成功回傳之後，關閉並重新開啟儲存，列出、讀取、範圍的版本號、事件與下限 SHALL 與關閉之前相同。提交在中途被中斷（程序結束或寫入失敗）時，重新開啟之後 SHALL 只看到兩種狀態之一：該次提交完全不存在，或完全存在（文件、版本號與事件一起）。檔案系統的範例 SHALL 以一次檔案改名作為一次提交唯一的發布點：改名之前寫出的檔案在列出、讀取與讀事件時 SHALL 不可見，並 SHALL 在之後的提交或開啟時被清掉或覆寫，SHALL NOT 影響之後的提交。

#### Scenario: 重新開啟後內容相同

- **WHEN** 對範圍提交三次（共寫入四份文件、刪除一份、帶三筆事件），關閉後重新開啟
- **THEN** 列出回傳相同的三個鍵與版本號 3；讀取回傳相同的內容、文件版本與最後修改時間；讀序號 0 之後的事件回傳相同的三筆

#### Scenario: 檔案系統的提交在發布前中斷

- **WHEN** 檔案系統範例的一次提交在寫出內容檔之後、改名發布之前被中斷，之後重新開啟
- **THEN** 列出與讀取顯示中斷之前的狀態，範圍的版本號沒有前進，沒有該次提交的事件；之後的提交成功，版本號接著中斷之前的值加 1

#### Scenario: 資料庫的提交在交易中途失敗

- **WHEN** SQLite 或 PostgreSQL 範例的一次提交在寫入部分文件之後、交易提交之前失敗
- **THEN** 之後的讀取看不到該次提交的任何文件與事件；範圍的版本號沒有前進


<!-- @trace
source: store-port-examples
updated: 2026-11-16T21:59:35+08:00
-->

---
### Requirement: 兩個儲存物件開同一份資料

SQLite 與 PostgreSQL 的範例：兩個各自開啟同一份資料的儲存物件 SHALL 一起遵守版本比對——一方提交之後，另一方以舊的預期版本提交 SHALL 得到衝突且整筆不寫入；兩方同時對同一份文件以相同的預期版本提交 SHALL 正好一方成功、另一方得到衝突。等不到寫入的鎖而逾時 SHALL 回報為暫時無法使用，SHALL NOT 回報為衝突。檔案系統的範例：資料夾已經被一個儲存物件持有時，第二個開啟 SHALL 失敗並回報資料夾使用中，SHALL NOT 改動資料夾；持有的程序結束（含不正常結束）之後 SHALL 可以再開啟。

#### Scenario: 另一個物件的提交讓舊版本衝突

- **WHEN** 兩個 SQLite 範例的儲存物件開同一個檔案，物件甲把鍵 k 從版本 1 改到版本 2，物件乙接著以預期版本 1 提交對 k 的寫入
- **THEN** 物件乙得到衝突，回報 k 目前的版本是 2；k 的內容是物件甲寫的

#### Scenario: 兩個物件同時提交只有一個成功

- **WHEN** 兩個 PostgreSQL 範例的儲存物件連到同一個 schema，同時以「必須不存在」提交同一個鍵
- **THEN** 正好一個成功、一個得到衝突；範圍的版本號只前進 1

#### Scenario: 資料夾已被持有

- **WHEN** 一個檔案系統範例的儲存物件還開著，另一個程序對同一個資料夾開啟範例
- **THEN** 第二個開啟失敗，錯誤說明資料夾使用中；第一個物件照常提交

#### Scenario: 持有者不正常結束之後可以再開啟

- **WHEN** 持有資料夾的程序被強制結束，之後另一個程序對同一個資料夾開啟範例
- **THEN** 開啟成功；先前成功的提交都在


<!-- @trace
source: store-port-examples
updated: 2026-11-16T21:59:35+08:00
-->

---
### Requirement: 可用性檢查與失敗的分類

範例 SHALL 提供「檢查儲存是否可用」：可用時成功；資料庫連不上、檔案或資料夾不見或不能讀寫時回報失敗。連線中斷與逾時這類暫時的失敗 SHALL 回報為「暫時無法使用」；其他失敗 SHALL 回報為儲存的失敗；兩者都 SHALL NOT 被回報成版本衝突。PostgreSQL 的範例在連線中斷後的下一次呼叫 SHALL 重新連線一次，重連成功就照常執行該次呼叫。PostgreSQL 的密碼 SHALL 可由環境變數 `SPECLINK_POSTGRES_PASSWORD` 提供，並優先於連線網址裡的密碼；範例 SHALL 讓呼叫端查得到「連線網址是否帶密碼」。任何錯誤訊息 SHALL NOT 含密碼。

#### Scenario: 可用的儲存

- **WHEN** 對一個開啟中的範例執行可用性檢查
- **THEN** 檢查成功

#### Scenario: 資料庫連不上

- **WHEN** PostgreSQL 範例連著的資料庫停止服務，接著執行讀取
- **THEN** 讀取回報暫時無法使用，不是衝突；可用性檢查回報失敗

#### Scenario: 連線中斷後自動重連

- **WHEN** PostgreSQL 範例的連線被伺服器端中斷，資料庫仍在服務，接著執行列出
- **THEN** 列出成功，回傳中斷之前提交的內容

#### Scenario: 錯誤訊息不含密碼

- **WHEN** 以帶密碼 s3cret 的連線網址連到一個不存在的主機
- **THEN** 連線失敗的錯誤訊息不含 s3cret


<!-- @trace
source: store-port-examples
updated: 2026-11-16T21:59:35+08:00
-->

---
### Requirement: PostgreSQL 範例的測試不靜默略過

沒有設定測試資料庫的網址（環境變數 `SPECLINK_TEST_POSTGRES_URL`）時，PostgreSQL 範例的每個需要資料庫的測試 SHALL 回報為略過，輸出 SHALL 含 `skipped` 字樣，SHALL NOT 回報為通過。CI 的 PostgreSQL job SHALL 執行這些測試，並 SHALL 在輸出含 `skipped` 時失敗。

#### Scenario: 沒有資料庫時回報略過

- **WHEN** 沒有設定 SPECLINK_TEST_POSTGRES_URL，執行 PostgreSQL 範例的測試
- **THEN** 輸出逐一列出需要資料庫的測試並標為 skipped；exit code 為 0

#### Scenario: CI 上真的執行

- **WHEN** CI 的 PostgreSQL job 以可用的資料庫執行 PostgreSQL 範例的測試
- **THEN** 輸出不含 skipped；合格測試的 16 項都通過

<!-- @trace
source: store-port-examples
updated: 2026-11-16T21:59:35+08:00
-->
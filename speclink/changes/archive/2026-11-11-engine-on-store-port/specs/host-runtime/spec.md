## MODIFIED Requirements

### Requirement: Host 承擔 TeamStore 的 UoW 與 event commit
<!-- BEFORE: Host 直接以 ExecutionContext 開 unit of work、把領域事件轉成 event records 後 commit；這條路徑不接線任何 CLI 流程 -->

Host SHALL 經 TeamStore 的轉接提供 commit 組合路徑：儲存接口的一次提交轉成一個 unit of work，提交裡的全部寫入與刪除放進同一個 unit of work，領域事件單向映射為 event records，經 commit 原子落入文件與 outbox；TeamStore 的 revision_conflict SHALL 轉成儲存接口的衝突並保留文件、expected 與 actual 的詳情。event record 的時間 SHALL 是該次提交的時間。這條 commit 路徑 SHALL NOT 接線任何本機模式的 CLI 流程。

#### Scenario: commit 後 outbox 含對應事件

- **WHEN** Host 經 TeamStore 的轉接對 in-memory reference store 提交一份文件的寫入並帶一筆領域事件
- **THEN** 提交成功後自 cursor 0 重讀 outbox 得到恰一筆對應 event record（含 actor 與事件名）；文件與事件同 commit 可見

#### Scenario: CAS 衝突原樣傳遞

- **WHEN** 兩次提交以相同的預期版本競寫同一文件
- **THEN** 敗方收到的 Host 錯誤是衝突，保留 revision_conflict 分類與 expected/actual 詳情，並指出是哪一份文件

### Requirement: engine 動詞經橋接於 TeamStore 上執行
<!-- BEFORE: 橋接以 TeamStore snapshot 把整個範圍載入成讀取視圖，寫入捕捉為 UnitOfWork staged ops 後原子提交 -->

Host SHALL 提供 engine-over-TeamStore 執行橋接：橋接 SHALL 把 TeamStore 的範圍經轉接包成儲存接口，再走「engine 動詞經儲存接口執行」的同一條執行路徑；Host SHALL NOT 另外保留一套把整個範圍載入記憶體的讀取視圖。同一動詞對語意相同的內容分別經本地 fs seam 與經橋接執行，typed outcome、錯誤分類與領域事件 SHALL 一致；TeamStore 的 revision_conflict SHALL 映射為命令層錯誤且保留 expected/actual 詳情。橋接 SHALL NOT 分叉 engine 命令層的動詞語意，發現的檔案系統暗依賴 SHALL 修在引擎的儲存視圖。

#### Scenario: 雙路徑 outcome 一致

- **WHEN** 對含相同 change 內容的 fs workspace 與 TeamStore scope 分別執行同一查詢動詞與同一變更型動詞
- **THEN** 兩路徑的 typed outcome 結構相等、變更型動詞回報相同種類的領域事件；失敗情境（如 not_found）的錯誤碼相同

#### Scenario: 橋接寫入原子落店

- **WHEN** 經橋接執行 task done 成功
- **THEN** 任務勾選後的文件內容、revision 遞增與 task-completed 事件記錄在同一 commit 內可見；commit 前 store 無任何中間狀態

#### Scenario: 版本衝突經 Server 回報

- **WHEN** 經橋接執行的變更型動詞在提交時遇到別人先寫入的同一份文件
- **THEN** Server 回應 HTTP 409，reason 是 revision_conflict，message 含該文件的鍵、expected 與 actual

## ADDED Requirements

### Requirement: engine 動詞經儲存接口執行

Host SHALL 提供在任何儲存接口實作與一個範圍上執行一個 engine 命令的入口，回傳 typed outcome、領域事件，以及有提交時的新範圍版本號。engine 儲存介面的每一個操作——列出、是否存在、讀取、寫入、刪除、搬移——SHALL 只用儲存接口的列出、讀取、提交三個操作完成。一個動詞的全部寫入與刪除 SHALL 連同它的領域事件收成正好一次提交；同一次執行裡，動詞 SHALL 讀得到自己剛寫入的內容。每筆寫入與刪除的預期版本 SHALL 取自這次執行第一次看到該文件時的版本，文件當時不存在時預期是「必須不存在」。查詢，以及沒有任何寫入也沒有領域事件的動詞，SHALL NOT 提交。engine 命令層回報錯誤時，或讀取途中儲存回報失敗時，Host SHALL NOT 提交，儲存 SHALL 沒有任何變動；這兩種失敗 SHALL 以各自的分類回報，不合併。同一動詞對語意相同的內容分別經本地 fs seam 與經記憶體版的儲存接口執行，typed outcome、錯誤分類與領域事件種類 SHALL 一致。

#### Scenario: 寫入的動詞一次提交

- **WHEN** 經儲存接口執行 task done 成功
- **THEN** 儲存收到正好一次提交，內含任務勾選後的文件與 task-completed 事件；回傳的範圍版本號大於執行前的值

#### Scenario: 查詢不提交

- **WHEN** 經儲存接口執行列出變更
- **THEN** 儲存沒有收到任何提交，範圍版本號不變，回傳的範圍版本號是「沒有」

#### Scenario: 封存是一次提交裡的搬移

- **WHEN** 經儲存接口封存一個變更
- **THEN** 儲存收到正好一次提交：該變更的每份文件寫到已封存的鍵、進行中的鍵全部刪除、受影響的正式規格寫入；提交之後以 `changes/` 為前綴列出時不含該變更

#### Scenario: 別人先寫入時回報衝突且不留半套

- **WHEN** 動詞讀過一份文件之後、提交之前，另一方改寫了同一份文件
- **THEN** 執行回報儲存的衝突，指出鍵、預期與實際；這個動詞的其他寫入與領域事件都沒有進入儲存

#### Scenario: 命令層拒絕時儲存不變

- **WHEN** 經儲存接口對不存在的變更執行 task done
- **THEN** 執行回報命令層的 not_found，儲存沒有收到提交

#### Scenario: 讀取途中儲存失敗時不提交

- **WHEN** 儲存在動詞執行途中的一次讀取回報「暫時無法使用」
- **THEN** 執行回報儲存的失敗，不回報成命令層的錯誤，儲存沒有收到提交

#### Scenario: 兩條路徑結果一致

- **WHEN** 對含相同內容的 fs workspace 與記憶體版的儲存接口分別執行同一查詢動詞、同一寫入動詞、刪除變更、封存變更與封存討論
- **THEN** 兩路徑的 typed outcome 結構相等、領域事件種類相同；not_found 這類失敗的錯誤碼相同

### Requirement: 經儲存接口執行時只讀取用到的文件

經儲存接口執行一個動詞時，Host SHALL 只向儲存要動詞實際用到的文件。同一次執行裡，同一個前綴 SHALL 最多列出一次，同一個鍵 SHALL 最多讀取一次。engine 儲存介面的單一操作需要多份文件時 SHALL 以一次讀取取得，SHALL NOT 逐份讀取。列出變更、查詢單一變更的狀態、列出進行中的討論（不帶封存旗標）這三類動詞 SHALL NOT 讀取任何鍵在 `archive/` 底下的文件內容，也 SHALL NOT 列出 `archive/changes/` 前綴。動詞覆寫一份這次執行沒看過的文件時，Host SHALL 先讀取該鍵以取得預期版本。

#### Scenario: 列出變更不碰封存資料

- **WHEN** 範圍裡有 3 個進行中的變更與 165 個已封存的變更，經儲存接口執行列出變更
- **THEN** 儲存收到的列出不含 `archive/changes/` 前綴，讀取的鍵沒有任何一個在 `archive/` 底下

#### Scenario: 多份 metadata 一次讀取

- **WHEN** 經儲存接口執行列出變更，範圍裡有 3 個進行中的變更
- **THEN** 3 份 metadata 由同一次讀取取得

#### Scenario: 同一個鍵不重複讀取

- **WHEN** 一個動詞在執行途中兩次需要同一份文件
- **THEN** 儲存只收到一次對該鍵的讀取

### Requirement: TeamStore 轉接成儲存接口

Host SHALL 提供把一個 TeamStore 範圍包成儲存接口的轉接，讓示範用的 Server 繼續使用既有的 driver。轉接 SHALL 讓鍵與 TeamStore 的九種文件雙向對應，對應 SHALL 依儲存接口的鍵命名規則。轉接在一次動詞執行裡 SHALL 最多匯出該範圍一次。文件版本 SHALL 是 TeamStore 的文件 revision，範圍版本號 SHALL 是專案 revision。轉接回報的最後修改時間 SHALL 是 0。對不到任何一種文件的鍵：讀取 SHALL 回傳「沒有」，寫入 SHALL 回報後端錯誤並指出該鍵。轉接收到不是它綁定的範圍時 SHALL 回報後端錯誤。

#### Scenario: 鍵與文件種類雙向對應

- **WHEN** 對九種文件各取一份，由文件種類換成鍵再換回
- **THEN** 得到原本的文件種類與名稱

##### Example: 對應

| TeamStore 的文件 | 鍵 |
| ---------------- | -- |
| 變更 `add-x` 的 metadata | `changes/add-x/.meta.yaml` |
| 變更 `add-x` 的 artifact `specs/auth/spec.md` | `changes/add-x/specs/auth/spec.md` |
| 變更 `add-x` 的 evidence | `changes/add-x/.evidence.json` |
| 已封存變更 `2026-11-10-add-x` 的 `proposal.md` | `archive/changes/2026-11-10-add-x/proposal.md` |
| 正式規格 `auth` | `specs/auth/spec.md` |
| 流程設定 | `config.yaml` |
| 詞彙表 | `LANGUAGE.md` |
| 看板順序 | `board/order` |

#### Scenario: 一次執行只匯出一次

- **WHEN** 經轉接執行一個會列出兩個前綴並讀取多份文件的動詞
- **THEN** TeamStore 的匯出只被呼叫一次

#### Scenario: 對不到文件種類的鍵

- **WHEN** 提交裡有一筆寫入的鍵不屬於九種文件的任何一種
- **THEN** 提交回報後端錯誤並指出該鍵，TeamStore 沒有任何變動

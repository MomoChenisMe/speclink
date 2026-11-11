## ADDED Requirements

### Requirement: 不經引擎命令的文件讀寫只用儲存接口

Host SHALL 為遠端用戶端需要、但不是引擎命令的文件讀寫提供函式，每個函式 SHALL 只用儲存接口的列出、讀取、提交三個操作，SHALL NOT 依賴任何 HTTP 框架，並 SHALL 能在任何儲存接口實作上執行。涵蓋：範圍版本號；流程設定的原文與 schema 名稱；看板順序的讀取與整份寫入；某個變更的 evidence；單份正式規格；已封存的清單、單份已封存文件與已封存變更的 capability 清單；搜尋；Context 快照；變更清單與單筆變更回應補充的欄位；spec drift。需要回報範圍版本號的函式 SHALL 先取版本再讀內容，使回報的版本 SHALL NOT 比內容新。讀取一批文件 SHALL 以一次帶多個鍵的讀取完成。這些函式 SHALL NOT 匯出整個範圍來取得其中一部分：只需要某個前綴底下的鍵時 SHALL 只列出該前綴。示範用 Server 的對應路由 SHALL 呼叫這些函式；它的專案路由中，直接呼叫 TeamStore 的 SHALL 只剩匯入與推送事件的串流。改用這些函式後，各路由的狀態碼、回應本文與 ETag 標頭 SHALL 維持既有的位元級輸出，例外只在儲存失敗時：兩個請求幾乎同時、比對之後提交之前別人改了同一份文件時，409 的訊息文字與儲存回報後端錯誤時 500 的訊息文字改用儲存接口的格式；TeamStore 回報找不到或無權限時改回 500，因為儲存接口只有衝突、暫時無法使用與後端錯誤三種失敗。

看板順序的整份寫入 SHALL：內容超過 1 MiB 時拒絕；請求帶的預期範圍版本與當下不同時以版本衝突拒絕；以一次提交寫入，文件存在時預期它的文件版本、不存在時預期「必須不存在」，並在同一次提交帶一筆 board-order-updated 事件。任何一種拒絕發生時儲存 SHALL 沒有變動。

#### Scenario: 流程設定的讀取只讀一份文件

- **WHEN** 在一個有 165 個已封存變更的範圍上讀取流程設定
- **THEN** 回傳原文、schema 名稱與範圍版本號；儲存收到的讀取只含 `config.yaml` 一個鍵，沒有列出 `archive/` 底下的任何前綴

#### Scenario: 單筆變更的 delta capability 只列出該變更的前綴

- **WHEN** 取得變更 add-x 的 delta capability 清單
- **THEN** 儲存收到一次前綴為 `changes/add-x/specs/` 的列出，沒有讀取任何文件內容

#### Scenario: 已封存清單一次讀取

- **WHEN** 在有三個已封存變更的範圍上取得已封存清單
- **THEN** 儲存收到一次前綴為 `archive/changes/` 的列出與一次讀取（含各變更的任務清單與 metadata）；清單依名稱由新到舊，各項含任務總數、已完成數與規格數

#### Scenario: 不存在的正式規格

- **WHEN** 讀取不存在的 capability 的正式規格
- **THEN** 回報找不到並指出該 capability

#### Scenario: 壞的流程設定

- **WHEN** 範圍的流程設定無法解析，讀取流程設定
- **THEN** 回報無效的設定，不回傳部分內容

#### Scenario: 壞的 evidence

- **WHEN** 某個變更的 evidence 文件不是合法的內容，讀取它
- **THEN** 回報內容損壞；不回傳空的清單

#### Scenario: 看板順序寫入成功

- **WHEN** 範圍版本號是 7、看板順序文件不存在，以預期版本 7 寫入一份 200 位元組的內容
- **THEN** 儲存收到正好一次提交：一筆對 `board/order` 的寫入（預期是「必須不存在」）與一筆 board-order-updated 事件；回傳的範圍版本號大於 7

#### Scenario: 看板順序的預期版本過期

- **WHEN** 範圍版本號是 8，以預期版本 7 寫入看板順序
- **THEN** 回報版本衝突；儲存沒有收到提交

#### Scenario: 看板順序內容過大

- **WHEN** 寫入超過 1 MiB 的看板順序內容
- **THEN** 回報內容過大；儲存沒有收到任何呼叫

#### Scenario: Context 快照沒有變時不讀內容

- **WHEN** 取得 Context 快照時帶上等於當下範圍版本號的識別
- **THEN** 回報「沒有變」；儲存只收到一次列出，沒有收到讀取

#### Scenario: Context 快照指定不存在的變更

- **WHEN** 取得 Context 快照並指定不存在的變更
- **THEN** 回報找不到

#### Scenario: 回報的版本不比內容新

- **WHEN** 讀取流程設定的過程中，在取得版本之後、讀取內容之前，另一方提交了新的流程設定
- **THEN** 回傳的內容是新的，回傳的範圍版本號是舊的；用戶端以該版本號輪詢時得知範圍已變動

#### Scenario: 改寫前後線上回應相同

- **WHEN** 對同一批資料，分別在改寫前與改寫後的示範用 Server 請求流程設定、看板順序、evidence、單份正式規格、已封存清單、單份已封存文件、搜尋、Context 快照與 drift
- **THEN** 每一對回應的狀態碼、本文與 ETag 標頭逐位元相同

#### Scenario: 儲存暫時無法使用

- **WHEN** 儲存在讀取流程設定時回報暫時無法使用
- **THEN** 函式回報儲存的「暫時無法使用」；示範用 Server 回應既有的 unavailable

### Requirement: 沒有 workspace 時封存不寫入本機檔案

引擎的封存在執行情境沒有 workspace 時 SHALL NOT 讀寫程序的檔案系統：SHALL NOT 寫入正式規格的本機備份與其清單檔，SHALL NOT 移除本機的開工標記檔；封存結果的「已建立備份」SHALL 為 false。封存的其餘效果——驗證、合併 delta、寫入正式規格、搬移變更的文件、一併封存來源討論——SHALL 與有 workspace 時相同，並 SHALL 全部經儲存介面完成。有 workspace 的封存（本機專案與 Node SDK 的 fs 形式）SHALL 維持既有行為：備份照寫，既有的位元級輸出不變。

#### Scenario: 經 Host 封存不在程序目錄寫檔

- **WHEN** 程序的工作目錄是一個空的暫存資料夾，經儲存接口封存一個會改寫既有正式規格的變更
- **THEN** 封存成功，結果的「已建立備份」是 false；該暫存資料夾仍然是空的；儲存收到正好一次提交

#### Scenario: 遠端封存的回應

- **WHEN** 對示範用 Server 送出封存請求
- **THEN** 回應的 snapshotCreated 是 false，其餘欄位（datedName、各 capability 的計數、archivedDiscussions、evidenceRecorded）與既有行為相同

#### Scenario: 本機封存照舊寫備份

- **WHEN** 在本機專案執行 speclink archive，該變更改寫一份既有的正式規格
- **THEN** `.speclink/snapshots/<封存名>/specs/<capability>/spec.md` 含改寫前的內容；stdout 與既有行為逐位元相同

## MODIFIED Requirements

### Requirement: TeamStore 轉接成儲存接口
<!-- BEFORE: 轉接綁定一個範圍、每個命令建一個；一次動詞執行最多匯出一次；收到別的範圍時回報後端錯誤 -->

Host SHALL 提供把 TeamStore 包成儲存接口的轉接，一個轉接 SHALL 能長期持有並服務多個範圍，讓示範用的 Server 繼續使用既有的 driver。轉接 SHALL 讓鍵與 TeamStore 的九種文件雙向對應，對應 SHALL 依儲存接口的鍵命名規則。範圍識別 SHALL 是 `<project>/<repo>` 的形式；project 裡的 `%` 與 `/` SHALL 寫成 `%25` 與 `%2F`，使一個範圍識別只對應一個範圍。轉接 SHALL 為每個用過的範圍保留最近一次載入的內容：下一次列出或讀取時，該範圍所屬專案的 revision 與載入當時相同 SHALL 重用它而 SHALL NOT 再次匯出；revision 不同 SHALL 重新載入。因此轉接在一次動詞執行裡 SHALL 最多匯出該範圍一次，而連續只查詢範圍版本號 SHALL NOT 造成任何匯出。轉接自己的提交成功後 SHALL 丟棄該範圍已載入的內容。同一個範圍的載入 SHALL 一次一個：等待中的呼叫在 revision 相同時 SHALL 重用剛完成的載入，SHALL NOT 另行匯出。直接經轉接（不經視圖）的每次呼叫，回傳的內容 SHALL NOT 比該次呼叫開始時 TeamStore 的 revision 舊。轉接 SHALL 另外提供一個請求用的視圖：視圖的第一次列出或讀取取得該範圍當下的載入，同一個請求之後的列出與讀取 SHALL 由這份載入回答；經視圖提交之後，視圖 SHALL 重新載入，因此讀得到自己寫入的內容。示範用 Server 的每個請求 SHALL 經這樣的視圖讀寫，使一個請求只載入一次範圍，命令內的讀取也來自同一個 revision。文件版本 SHALL 是 TeamStore 的文件 revision，範圍版本號 SHALL 是專案 revision。轉接回報的最後修改時間 SHALL 是 0。對不到任何一種文件的鍵：讀取 SHALL 回傳「沒有」，寫入 SHALL 回報後端錯誤並指出該鍵。範圍識別的形式不合時 SHALL 回報後端錯誤。

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

#### Scenario: 一個轉接服務兩個範圍

- **WHEN** 同一個轉接先後對範圍 `p1/main` 與 `p2/main` 各列出一次變更
- **THEN** 兩次各得到自己範圍的內容，互不混用

#### Scenario: revision 沒變時不再匯出

- **WHEN** 對同一個範圍連續兩次經轉接取得範圍版本號，中間沒有任何提交
- **THEN** 兩次得到相同的版本號；TeamStore 的匯出在第二次沒有被呼叫

#### Scenario: 別人提交後讀到新內容

- **WHEN** 轉接載入某個範圍之後，另一方直接對 TeamStore 提交了一份文件，之後再經轉接讀取該文件
- **THEN** 讀到的是新內容與新的文件版本

#### Scenario: 自己提交後讀到新內容

- **WHEN** 經轉接提交一筆寫入後，再經同一個轉接讀取該文件
- **THEN** 讀到剛寫入的內容，文件版本等於提交回傳的範圍版本號

#### Scenario: 一個請求只載入一次

- **WHEN** 一個請求經視圖列出兩個前綴並讀取一份文件，之後另一方直接對 TeamStore 提交了該文件，同一個請求再讀取它
- **THEN** TeamStore 的快照只被取一次；第二次讀取得到的仍是原本的內容與文件版本

#### Scenario: 經視圖提交後讀到自己寫的內容

- **WHEN** 經視圖提交一筆寫入後，再經同一個視圖讀取該文件
- **THEN** 讀到剛寫入的內容，文件版本等於提交回傳的範圍版本號

#### Scenario: 同時錯過同一份載入的兩個請求

- **WHEN** 轉接沒有該範圍的載入，兩個請求同時列出同一個範圍
- **THEN** TeamStore 的匯出只被呼叫一次，兩個請求都得到同樣的內容

#### Scenario: 含 `/` 的專案名稱

- **WHEN** 專案 `team/alpha` 的 repo `main` 與專案 `team` 的 repo `alpha/main` 各有一份不同的文件，經同一個轉接各列出一次
- **THEN** 範圍識別分別是 `team%2Falpha/main` 與 `team/alpha/main`，兩次各得到自己範圍的內容

#### Scenario: 範圍識別的形式不合

- **WHEN** 以不含 `/` 的範圍識別經轉接列出
- **THEN** 回報後端錯誤並指出範圍識別；TeamStore 沒有被呼叫

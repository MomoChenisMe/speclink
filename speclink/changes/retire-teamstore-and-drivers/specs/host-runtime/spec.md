## REMOVED Requirements

### Requirement: Host 承擔 TeamStore 的 UoW 與 event commit

**Reason**: TeamStore 與它的轉接移除。提交不再轉成 TeamStore 的 unit of work

**Migration**: 「一個動詞的全部寫入與事件收成正好一次提交」由 host-runtime「engine 動詞經儲存接口執行」承接；「事件與寫入同一次提交」「版本對不上時整筆拒絕並回報文件、預期與實際」由 store-port-contract 承接；示範用 Server 的事件進事件記錄由 store-port-examples「事件與寫入同一次交易並可依序號讀回」承接

### Requirement: engine 動詞經橋接於 TeamStore 上執行

**Reason**: engine-over-TeamStore 的執行橋接移除。它只是把 TeamStore 包成儲存接口再走同一條路徑

**Migration**: 兩條路徑結果一致（本地 fs 與儲存接口）由 host-runtime「engine 動詞經儲存接口執行」的情境「兩條路徑結果一致」承接；寫入原子性由同一條需求的「寫入的動詞一次提交」承接；版本衝突經 Server 回報 409 由 reference-server「寫入原子提交且 CAS 衝突可辨」承接

### Requirement: TeamStore 轉接成儲存接口

**Reason**: 示範用 Server 改用直接實作儲存接口的範例，沒有任何程式再把 TeamStore 包成儲存接口。轉接特有的行為（依專案 revision 重用載入的內容、請求用的視圖、最後修改時間回報 0、文件版本取 TeamStore 的 revision）隨它一起消失

**Migration**: 範圍識別字的形式由 host-runtime「專案與儲存庫對映成範圍識別字」承接；九種文件與鍵的對應由 host-runtime「文件種類與鍵的對應」承接；其餘條款不需要承接——範例的每份文件有真實的最後修改時間，每個範圍有自己的版本號（store-port-examples「三個範例通過合格測試」）

## ADDED Requirements

### Requirement: 專案與儲存庫對映成範圍識別字

Host SHALL 提供把一組專案與儲存庫對映成儲存接口範圍識別字的函式，以及它的反向。範圍識別字 SHALL 是 `<project>/<repo>` 的形式；project 裡的 `%` 與 `/` SHALL 寫成 `%25` 與 `%2F`，使一個範圍識別字只對應一組專案與儲存庫。專案與儲存庫的名稱組不成合格的範圍識別字（例如超過範圍識別字的長度上限）時，函式 SHALL 回報錯誤。反向的函式遇到形式不合的範圍識別字（沒有 `/`、任一段是空的、project 含 `%25` 與 `%2F` 以外的跳脫）SHALL 回報錯誤並指出該範圍識別字，SHALL NOT 猜測。示範用 Server SHALL 用這一對函式決定每個請求讀寫的範圍。

#### Scenario: 含斜線的專案名稱

- **WHEN** 把專案 `team/alpha` 的儲存庫 `main` 與專案 `team` 的儲存庫 `alpha/main` 各自對映成範圍識別字
- **THEN** 得到 `team%2Falpha/main` 與 `team/alpha/main`，兩者不同；各自經反向的函式得回原本的專案與儲存庫

#### Scenario: 含百分號的專案名稱

- **WHEN** 把專案 `50%` 的儲存庫 `main` 對映成範圍識別字再反向
- **THEN** 範圍識別字是 `50%25/main`；反向得回專案 `50%` 與儲存庫 `main`

#### Scenario: 範圍識別字的形式不合

- **WHEN** 以不含 `/` 的範圍識別字呼叫反向的函式
- **THEN** 回報錯誤並指出該範圍識別字

#### Scenario: 兩個儲存庫各自的範圍

- **WHEN** 經示範用 Server 對同一個專案的兩個儲存庫各寫入一份文件
- **THEN** 兩份文件落在兩個不同的範圍；讀其中一個儲存庫看不到另一個的文件

### Requirement: 文件種類與鍵的對應

Host SHALL 提供把第一版的包用的九種文件種類換成儲存接口的鍵的對應，以及它的反向，對應 SHALL 依儲存接口的鍵命名規則。這份對應 SHALL 是第一版匯入包與第一版備份檔唯一的轉換來源。換不成鍵的文件種類 SHALL 回報「沒有對應」，呼叫端據此拒絕整個包，SHALL NOT 略過該文件。

#### Scenario: 文件種類換成鍵再換回

- **WHEN** 對九種文件各取一份，由文件種類換成鍵再換回
- **THEN** 得到原本的文件種類與名稱

##### Example: 對應

| 文件種類 | 鍵 |
| -------- | -- |
| 變更 `add-x` 的 metadata | `changes/add-x/.meta.yaml` |
| 變更 `add-x` 的 artifact `specs/auth/spec.md` | `changes/add-x/specs/auth/spec.md` |
| 變更 `add-x` 的 evidence | `changes/add-x/.evidence.json` |
| 已封存變更 `2026-11-10-add-x` 的 `proposal.md` | `archive/changes/2026-11-10-add-x/proposal.md` |
| 正式規格 `auth` | `specs/auth/spec.md` |
| 流程設定 | `config.yaml` |
| 詞彙表 | `LANGUAGE.md` |
| 看板順序 | `board/order` |

#### Scenario: 不認得的文件種類

- **WHEN** 第一版的包裡有一份文件的種類不在九種之內，把它換成鍵
- **THEN** 回報沒有對應；匯入這個包的呼叫以「不合格的包」結束並指出該文件，範圍沒有被寫入

## MODIFIED Requirements

### Requirement: 不經引擎命令的文件讀寫只用儲存接口
<!-- BEFORE: 示範用 Server 的專案路由中直接呼叫 TeamStore 的只剩推送事件的串流；例外清單含「TeamStore 回報找不到或無權限時改回 500」；有情境「示範用 Server 的匯入不直接呼叫 TeamStore」 -->
<!-- REMOVED-SCENARIO: 示範用 Server 的匯入不直接呼叫 TeamStore -->

Host SHALL 為遠端用戶端需要、但不是引擎命令的文件讀寫提供函式，每個函式 SHALL 只用儲存接口的列出、讀取、提交三個操作，SHALL NOT 依賴任何 HTTP 框架，並 SHALL 能在任何儲存接口實作上執行。涵蓋：範圍版本號；流程設定的原文與 schema 名稱；看板順序的讀取與整份寫入；某個變更的 evidence；單份正式規格；已封存的清單、單份已封存文件與已封存變更的 capability 清單；搜尋；Context 快照；變更清單與單筆變更回應補充的欄位；spec drift。需要回報範圍版本號的函式 SHALL 先取版本再讀內容，使回報的版本 SHALL NOT 比內容新。讀取一批文件 SHALL 以一次帶多個鍵的讀取完成。這些函式 SHALL NOT 匯出整個範圍來取得其中一部分：只需要某個前綴底下的鍵時 SHALL 只列出該前綴。示範用 Server 的對應路由 SHALL 呼叫這些函式；它的專案路由 SHALL NOT 經儲存接口以外的途徑讀寫文件；匯入 SHALL 由宿主套件以儲存接口完成。改用這些函式後，各路由的狀態碼、回應本文與 ETag 標頭 SHALL 維持既有的位元級輸出，例外只在儲存失敗時：兩個請求幾乎同時、比對之後提交之前別人改了同一份文件時，409 的訊息文字與儲存回報後端錯誤時 500 的訊息文字改用儲存接口的格式。儲存接口只有衝突、暫時無法使用與後端錯誤三種失敗；儲存回報後端錯誤時示範用 Server SHALL 回 500。

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

#### Scenario: 示範用 Server 的匯入經儲存接口的一次提交

- **WHEN** 對示範用 Server 送出匯入請求並成功
- **THEN** 全部文件經儲存接口的一次提交寫入；儲存沒有收到其他寫入的呼叫

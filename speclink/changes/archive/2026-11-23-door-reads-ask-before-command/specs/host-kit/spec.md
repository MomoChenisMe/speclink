## ADDED Requirements

### Requirement: 入口的文件讀取先問執行前掛鉤

入口裡不是引擎命令的九條文件讀取 SHALL 在讀取儲存之前問執行前掛鉤正好一次，交給它身分、範圍、命令名稱與參數（參數的鍵一律 camelCase）：

| 路由 | 命令名稱 | 參數 |
| --- | --- | --- |
| `GET /config` | `workflow-config-show` | 空物件 |
| `GET /specs/{capability}/document` | `show` | `item` 是 capability 名稱，`itemType` 是 `spec` |
| `GET /archived/{datedName}/artifacts/{path}` | `artifact-cat` | `artifact` 是網址裡的文件路徑（例如 `proposal.md`、`specs/auth/spec.md`），`change` 是帶日期的名稱 |
| `GET /archived` | `list` | `archived` 是 true |
| `GET /archived/{datedName}/capabilities` | `list` | `archived` 是 true |
| `GET /search` | `workspace-search` | `query` 是搜尋字串 |
| `POST /context` | `context-snapshot` | 請求指定變更時 `change` 是它的名稱，否則是空物件 |
| `GET /board-order` | `get-board-order` | 空物件 |
| `GET /changes/{name}/drift` | `spec-drift` | `change` 是網址裡的變更名稱 |

前五條用的 SHALL 是引擎既有的命令名稱，名稱與參數的鍵 SHALL 與經另一個門執行對應的動詞時掛鉤收到的相同，使宿主對那些命令的規則直接適用。`workspace-search`、`context-snapshot`、`get-board-order`、`spec-drift` SHALL 是宿主套件自己的命令名稱，SHALL 列在宿主套件的命令名稱清單裡；它們不是引擎命令，執行前掛鉤的讀取器 SHALL NOT 接受它們。

掛鉤放行時，回應的狀態碼、本文與 ETag SHALL 與問掛鉤之前相同。掛鉤拒絕時，回應 SHALL 是 409、reason 為 refused、訊息為宿主給的原因（原因是空字串時用固定的文字），儲存 SHALL 連讀取都沒有收到。掛鉤失敗或回答的形狀不對 SHALL 視為不放行，與引擎命令相同。這些路由只讀取：提交後掛鉤 SHALL NOT 被呼叫。沒有提供執行前掛鉤時，這九條路由的回應與儲存收到的呼叫 SHALL 與這條需求之前相同。

掛鉤 SHALL 在下列檢查都通過之後才被問，檢查沒有通過時 SHALL NOT 被問：沒有身分（401）、範圍清單對不到專案或 repo（404）、請求的本文或查詢字串不合法（400）。`POST /context` 帶著與範圍現況相符的 `If-None-Match` 時，掛鉤 SHALL 照樣先被問，放行後回應才是 304。

入口其他讀寫範圍內容的路由也 SHALL 在讀取儲存之前問掛鉤，只有兩個例外：流程設定的寫入要先讀設定才知道變動的鍵；一般的 `GET /plan` 先讀看板順序才組出命令。這兩個例外被拒絕時，回應 SHALL 仍是 409、reason 為 refused，不帶範圍的內容。

文件的寫入（`PUT /changes/{name}/artifacts/{path}`）SHALL 在請求的格式檢查通過之後先問掛鉤，再讀文件的版本來比對 `If-Match`，掛鉤 SHALL 只被問一次，命令名稱與參數 SHALL 與之前相同（`new-artifact`）。掛鉤拒絕時，不論 `If-Match` 是否與文件的版本相符，回應 SHALL 都是 409、reason 為 refused，SHALL NOT 帶文件的版本，儲存 SHALL 連讀取都沒有收到。掛鉤放行時，`If-Match` 不相符的回應 SHALL 仍是 409、reason 為 revision_conflict，與之前相同。

#### Scenario: 掛鉤一律拒絕時九條讀取都被擋下

- **WHEN** 執行前掛鉤對某個身分的每一次詢問都回答拒絕、原因「帳號已停用」，這個身分在範圍清單裡，把九條讀取的請求逐一交給入口
- **THEN** 九個回應都是 409、reason 為 refused、訊息是「帳號已停用」；儲存沒有收到任何列出或讀取

##### Example: 改動前後的對照

| 請求 | 這條需求之前 | 之後 |
| --- | --- | --- |
| `GET /changes` | 409 refused「帳號已停用」 | 409 refused「帳號已停用」 |
| `GET /archived` | 200 與已封存清單 | 409 refused「帳號已停用」 |
| `POST /context` | 200 與整個範圍的文件 | 409 refused「帳號已停用」 |
| `GET /search?q=auth` | 200 與搜尋結果 | 409 refused「帳號已停用」 |
| `GET /changes/demo/drift` | 200 與 demo 的 design、tasks 原文 | 409 refused「帳號已停用」 |

#### Scenario: 掛鉤收到的命令名稱與參數

- **WHEN** 執行前掛鉤記下每次收到的命令名稱與參數並一律放行，依序把 `GET /config`、`GET /specs/auth/document`、`GET /archived/2026-11-22-add-two/artifacts/proposal.md`、`GET /archived`、`GET /archived/2026-11-22-add-two/capabilities`、`GET /search?q=login`、指定變更 demo 的 `POST /context`、`GET /board-order`、`GET /changes/demo/drift` 交給入口
- **THEN** 掛鉤被問 9 次，名稱依序是 `workflow-config-show`、`show`、`artifact-cat`、`list`、`list`、`workspace-search`、`context-snapshot`、`get-board-order`、`spec-drift`；`show` 的參數有 `item` `auth` 與 `itemType` `spec`；`artifact-cat` 的參數有 `artifact` `proposal.md` 與 `change` `2026-11-22-add-two`；兩次 `list` 的參數裡 `archived` 都是 true；`workspace-search` 的參數有 `query` `login`；`context-snapshot` 的參數有 `change` `demo`；`spec-drift` 的參數有 `change` `demo`

#### Scenario: 既有命令的規則直接適用

- **WHEN** 執行前掛鉤只對命令 `show` 回答拒絕，把 `GET /specs/auth/document` 交給入口，並經另一個門執行 `show auth --item-type spec`
- **THEN** 兩者都被拒絕：入口的回應是 409、reason 為 refused；另一個門的呼叫以 `refused` 失敗

#### Scenario: 放行時回應不變

- **WHEN** 執行前掛鉤對全部命令回答放行，把九條讀取的請求交給入口；另以沒有掛鉤的宿主套件對相同的資料送相同的請求
- **THEN** 每一對回應的狀態碼、本文與 ETag 相同；提交後掛鉤沒有被呼叫

#### Scenario: 沒有掛鉤時行為不變

- **WHEN** 建立宿主套件時沒有提供執行前掛鉤，把九條讀取的請求交給入口
- **THEN** 九個回應都是 200，儲存收到的列出與讀取次數與這條需求之前相同

#### Scenario: 先前的檢查沒過時不問掛鉤

- **WHEN** 執行前掛鉤記下被問的次數，分別把下列請求交給入口：身分是「沒有」的 `GET /archived`；專案不在範圍清單裡的 `GET /board-order`；本文不是合法 JSON 的 `POST /context`
- **THEN** 回應依序是 401、404、400；掛鉤被問 0 次

#### Scenario: 沒有變動的快照也先問掛鉤

- **WHEN** 用戶端帶著與範圍現況相符的 `If-None-Match` 送 `POST /context`，執行前掛鉤先放行、之後改成拒絕再送一次
- **THEN** 第一次的回應是 304；第二次的回應是 409、reason 為 refused；兩次掛鉤各被問一次，命令名稱是 `context-snapshot`

#### Scenario: 握手與輪詢不問掛鉤

- **WHEN** 執行前掛鉤對每一次詢問都回答拒絕，把握手與 `GET /sync-state` 的請求交給入口
- **THEN** 兩個回應都是 200；掛鉤被問 0 次

#### Scenario: 每一條讀寫範圍內容的路由都問掛鉤

- **WHEN** 執行前掛鉤對每一次詢問都回答拒絕，對入口的每一條專案路由各送一個合法的請求
- **THEN** 除了握手、輪詢、專案底下的身分查詢與技能的取得之外，每一個回應都是 409、reason 為 refused，儲存沒有收到提交；除了流程設定的寫入（它要先讀設定才知道變動的鍵）與一般的 `GET /plan`（它先讀看板順序才組出命令）之外，儲存也沒有收到讀取

#### Scenario: 被拒絕的文件寫入不透露文件的版本

- **WHEN** 執行前掛鉤對每一次詢問都回答拒絕、原因「帳號已停用」，變更 demo 的 design 已經存在，把兩個 `PUT /changes/demo/artifacts/design` 交給入口：一個的 `If-Match` 是 0，另一個的 `If-Match` 是 999
- **THEN** 兩個回應都是 409、reason 為 refused、訊息是「帳號已停用」；儲存沒有收到任何列出、讀取或提交

#### Scenario: 放行的文件寫入照舊比對版本

- **WHEN** 執行前掛鉤記下每次收到的命令名稱並一律放行，變更 demo 的 design 已經存在，先把 `If-Match` 是 999 的 `PUT /changes/demo/artifacts/design` 交給入口，再把 `If-Match` 是文件目前版本的同一個請求交給入口；另以沒有掛鉤的宿主套件對相同的資料送相同的兩個請求
- **THEN** 第一個回應是 409、reason 為 revision_conflict；第二個回應是 200；每一對回應的狀態碼與本文相同；兩次掛鉤各被問一次，命令名稱都是 `new-artifact`

#### Scenario: 文件寫入的格式檢查沒過時不問掛鉤

- **WHEN** 執行前掛鉤記下被問的次數，把沒有 `If-Match` 標頭的 `PUT /changes/demo/artifacts/design` 交給入口
- **THEN** 回應是 400、reason 為 invalid_argument；掛鉤被問 0 次

## MODIFIED Requirements

### Requirement: HTTP 入口處理原版用戶端的請求

<!-- BEFORE: 只有經入口執行的引擎命令與看板順序的寫入經過掛鉤；不是引擎命令的文件讀取不問掛鉤 -->

宿主套件 SHALL 提供一個和 HTTP 框架無關的入口：收一個請求（方法、路徑、查詢字串、標頭、本文）與宿主已驗證的身分（驗不出來時是「沒有」），回傳一個回應（狀態碼、標頭、本文），或回報「這不是入口的路徑」。入口 SHALL NOT 讀取或驗證任何憑證；身分 SHALL 只來自宿主。入口負責的路徑 SHALL 都在 `/api/speclink/v1/` 底下：專案底下的動詞與讀取路由、握手、輪詢、專案底下的身分查詢、範圍清單、API 根的身分查詢與登入方式的宣告。對這些路徑，回應的網址、請求格式、回應格式、狀態碼與 ETag SHALL 與 Client Protocol 及各路由既有的規格相同，使原版的 CLI 與 Speclink Desktop 不經修改即可使用。入口 SHALL NOT 提供登入端點、帳號與成員管理、管理介面、備份；對它不認得的路徑 SHALL 回報「這不是入口的路徑」而不是錯誤，讓宿主在同一個前綴底下放自己的路由。需要身分的路徑沒有身分時 SHALL 回應 401；登入方式的宣告 SHALL NOT 需要身分。入口讀取或寫入範圍內容的每一條路由 SHALL 經過與另一個門相同的執行前掛鉤：經入口執行的引擎命令、看板順序的寫入，以及不是引擎命令的文件讀取（見「入口的文件讀取先問執行前掛鉤」）。有提交的路由 SHALL 另外經過提交後掛鉤。掛鉤拒絕時回應 SHALL 是 409、reason 為 refused、訊息為宿主的原因。握手、輪詢、兩個身分查詢、範圍清單、登入方式的宣告與技能的取得 SHALL NOT 問執行前掛鉤：它們不回範圍裡的文件內容。請求轉成引擎命令、結果轉成線上格式、錯誤對到狀態碼的實作 SHALL 只有一份，示範用 Server 與任何宿主 SHALL 共用它。

#### Scenario: 不用 HTTP 框架完成一次握手與寫入

- **WHEN** 以記憶體版儲存與一個回傳單一專案單一 repo 的範圍函式建立宿主套件，依序把握手、建立變更 demo、列出變更三個請求連同身分交給入口
- **THEN** 三個回應的狀態碼都是 200；握手的回應含操作者的識別與顯示名稱；清單的回應含 demo；建立變更的回應帶 ETag，其值大於握手當時的範圍版本號

#### Scenario: 沒有身分

- **WHEN** 把列出變更的請求交給入口，身分是「沒有」
- **THEN** 回應是 401、reason 為 permission_denied；儲存沒有被呼叫

#### Scenario: 不認得的路徑交還宿主

- **WHEN** 把專案底下一條入口沒有對照的路徑交給入口
- **THEN** 入口回報「這不是入口的路徑」，沒有產生回應

#### Scenario: 掛鉤拒絕經入口來的命令

- **WHEN** 執行前掛鉤對命令 archive 回答拒絕、原因「程式碼還沒合併」，把封存的請求交給入口
- **THEN** 回應是 409、reason 為 refused、訊息是「程式碼還沒合併」；變更仍在進行中

#### Scenario: 看板順序的寫入也經過掛鉤

- **WHEN** 執行前掛鉤對命令名稱 put-board-order 回答拒絕，把看板順序的寫入請求交給入口
- **THEN** 回應是 409、reason 為 refused；看板順序文件不變

#### Scenario: 提交後掛鉤收到經入口的提交

- **WHEN** 把勾選任務的請求交給入口並成功
- **THEN** 提交後掛鉤被呼叫一次，收到命令 task-done 與 task-completed 事件

#### Scenario: 本文不是合法的請求

- **WHEN** 把建立變更的請求交給入口，本文不是合法的 JSON
- **THEN** 回應是 400、reason 為 invalid_argument；儲存沒有收到提交

#### Scenario: 示範用 Server 的回應不變

- **WHEN** 示範用 Server 改用入口前後，對同一批資料與同一個使用者送出既有整合測試涵蓋的每一種專案路由請求
- **THEN** 每一對回應的狀態碼、本文與 ETag 相同

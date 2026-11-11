## Purpose

宿主套件的契約：內嵌引擎的宿主建立一個由全部使用者共用的宿主套件，之後每次呼叫給身分與範圍，並能經兩個掛鉤在寫入前拒絕、在提交後旁觀。本 capability 規定宿主能介入什麼與不能介入什麼；引擎怎麼經儲存接口執行屬於 host-runtime，三個儲存函式的保證屬於 store-port-contract。

## ADDED Requirements

### Requirement: HTTP 入口處理原版用戶端的請求

宿主套件 SHALL 提供一個和 HTTP 框架無關的入口：收一個請求（方法、路徑、查詢字串、標頭、本文）與宿主已驗證的身分（驗不出來時是「沒有」），回傳一個回應（狀態碼、標頭、本文），或回報「這不是入口的路徑」。入口 SHALL NOT 讀取或驗證任何憑證；身分 SHALL 只來自宿主。入口負責的路徑 SHALL 都在 `/api/speclink/v1/` 底下：專案底下的動詞與讀取路由、握手、輪詢、專案底下的身分查詢、範圍清單、API 根的身分查詢與登入方式的宣告。對這些路徑，回應的網址、請求格式、回應格式、狀態碼與 ETag SHALL 與 Client Protocol 及各路由既有的規格相同，使原版的 CLI 與 Speclink Desktop 不經修改即可使用。入口 SHALL NOT 提供登入端點、帳號與成員管理、管理介面、備份；對它不認得的路徑 SHALL 回報「這不是入口的路徑」而不是錯誤，讓宿主在同一個前綴底下放自己的路由。需要身分的路徑沒有身分時 SHALL 回應 401；登入方式的宣告 SHALL NOT 需要身分。經入口執行的引擎命令與看板順序的寫入 SHALL 經過與另一個門相同的執行前掛鉤與提交後掛鉤；掛鉤拒絕時回應 SHALL 是 409、reason 為 refused、訊息為宿主的原因。請求轉成引擎命令、結果轉成線上格式、錯誤對到狀態碼的實作 SHALL 只有一份，示範用 Server 與任何宿主 SHALL 共用它。

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

### Requirement: 範圍與能力由宿主的函式決定

宿主套件 SHALL 在建立時接受一個選填的函式：給一個身分，回傳這個人可以使用的專案清單，每個專案含識別字與名稱及其 repo 清單，每個 repo 含識別字、名稱、儲存用的範圍識別與兩個能力開關（能否寫入、能否改流程政策）。入口 SHALL 用它把網址裡的專案與請求指定的 repo 對到範圍識別：專案不在清單裡 SHALL 回應 404；請求指定的 repo 不在該專案的清單裡 SHALL 回應 404；沒有指定 repo 而專案只有一個 repo SHALL 使用它；沒有指定而有多個 SHALL 以 409 拒絕並列出候選，SHALL NOT 自動選擇。入口 SHALL NOT 自行快取這個函式的結果。沒有寫入能力的人送出寫入請求 SHALL 得到 403 且儲存不變；唯讀的 Context 快照請求不受此限。沒有改流程政策能力的人送出會變動政策鍵（schema、locale、spec_locale、tdd、audit、worktree）的流程設定寫入 SHALL 得到 403 且流程設定不變；只變動其他鍵的寫入 SHALL 照常。這個函式失敗時回應 SHALL 是 503。沒有提供這個函式時，範圍清單與全部專案底下的路徑 SHALL 回應 404。API 版本標頭不符的專案請求 SHALL 以 409 拒絕。

#### Scenario: 專案不在這個人的清單裡

- **WHEN** 範圍函式對某個身分只回傳專案 alpha，該身分請求專案 beta 的變更清單
- **THEN** 回應是 404、reason 為 not_found；儲存沒有被呼叫

#### Scenario: 多個 repo 而沒有指定

- **WHEN** 專案 alpha 有 web 與 api 兩個 repo，請求沒有指定 repo
- **THEN** 回應是 409，訊息列出 web 與 api；沒有任何範圍被讀取

#### Scenario: 指定 repo 決定範圍

- **WHEN** 專案 alpha 的 web 與 api 各對到不同的範圍識別，帶指定 api 的標頭請求建立變更 demo
- **THEN** demo 只出現在 api 對到的範圍

#### Scenario: 沒有寫入能力

- **WHEN** 範圍函式回傳的 repo 寫入能力為否，該身分送出建立變更的請求
- **THEN** 回應是 403、reason 為 permission_denied；儲存沒有收到提交；同一個人請求變更清單得到 200

#### Scenario: 沒有改政策的能力

- **WHEN** repo 的寫入能力為是、改流程政策的能力為否，該身分送出把 tdd 從 false 改成 true 的流程設定寫入，再送出只改 context 的寫入
- **THEN** 第一個回應是 403 且流程設定不變；第二個回應成功

#### Scenario: 握手依能力開關宣告

- **WHEN** repo 的寫入能力為否，該身分送出握手
- **THEN** 回應的 capabilities 中，代表可改流程政策、可刪除變更、可搬移任務的欄位都是 false

#### Scenario: 沒有提供範圍函式

- **WHEN** 建立宿主套件時沒有提供範圍函式，把範圍清單與任一專案路徑的請求交給入口
- **THEN** 兩個回應都是 404

#### Scenario: 範圍函式失敗

- **WHEN** 範圍函式回報失敗，把變更清單的請求交給入口
- **THEN** 回應是 503、reason 為 unavailable

#### Scenario: API 版本不符

- **WHEN** 專案請求的 API 版本標頭不是目前的版本
- **THEN** 回應是 409、reason 為 refused；儲存沒有被呼叫

### Requirement: 握手、輪詢與範圍清單由入口回答

入口 SHALL 回答握手：回應含操作者（身分的識別與顯示名稱）、專案與 repo 的識別、API 版本、引擎版本與 capabilities；capabilities 的更新方式宣告 SHALL 含輪詢，並 SHALL 只在宿主於建立時宣告推送事件的串流時才含推送的 transport。入口 SHALL NOT 自己提供事件串流。入口 SHALL 回答「範圍有沒有變」的輪詢：請求帶的識別等於當下的範圍版本號時回應 304，否則回應 200 並帶新的識別；輪詢 SHALL NOT 讀取任何文件內容。入口 SHALL 回答範圍清單，內容是範圍函式對該身分回傳的專案與 repo。入口 SHALL 回答專案底下與 API 根的身分查詢，使用者的名稱取自身分的顯示名稱、識別取自身分的識別。

#### Scenario: 預設只宣告輪詢

- **WHEN** 建立宿主套件時沒有宣告推送，送出握手
- **THEN** capabilities 的更新方式宣告沒有任何推送的 transport，含輪詢的宣告

#### Scenario: 宿主宣告推送

- **WHEN** 建立時宣告一個 SSE 的 transport 與它的網址，送出握手
- **THEN** capabilities 的更新方式宣告含該 transport 與網址

#### Scenario: 輪詢沒有變

- **WHEN** 以握手後第一次讀取回應的 ETag 當識別送出輪詢，期間沒有提交
- **THEN** 回應是 304；儲存沒有收到讀取

#### Scenario: 提交後輪詢

- **WHEN** 另一個請求建立了一個變更之後，以舊的識別送出輪詢
- **THEN** 回應是 200 並帶新的 ETag

#### Scenario: 範圍清單

- **WHEN** 範圍函式對某個身分回傳專案 alpha（repo web、api），該身分請求範圍清單
- **THEN** 回應含一個專案 alpha 與兩個 repo，各帶識別字與名稱，形狀是 Client Protocol 既有的範圍清單；不含儲存用的範圍識別與能力開關

#### Scenario: API 根的身分查詢

- **WHEN** 以識別 u_42、顯示名稱 Alice 的身分請求 API 根的身分查詢
- **THEN** 回應的使用者名稱是 Alice、識別是 u_42

### Requirement: 登入方式由宿主宣告

宿主套件 SHALL 在建立時接受選填的登入方式宣告：支援的方式清單，以及支援瀏覽器登入時登入服務的發行者網址、用戶端識別與選填的 scope。有提供時，入口 SHALL 在 API 根的 auth-methods 路徑以 Client Protocol 規定的 payload 回答它，這個路徑 SHALL NOT 需要身分或版本標頭。沒有提供時該路徑 SHALL 回應 404，使用戶端維持既有的登入行為。入口 SHALL 只轉述宿主的設定，SHALL NOT 實作任何一種登入方式的端點。示範用 Server SHALL 宣告它支援的方式：裝置登入與存取金鑰。

#### Scenario: 宿主宣告瀏覽器登入

- **WHEN** 建立時宣告 methods 為 browser 與 token、發行者網址與用戶端識別，不帶身分請求 auth-methods
- **THEN** 回應是 200，payload 的 methods 是 browser 與 token，browser 物件含 issuer 與 clientId

#### Scenario: 沒有宣告

- **WHEN** 建立時沒有提供登入方式的宣告，請求 auth-methods
- **THEN** 回應是 404

#### Scenario: 示範用 Server 的宣告

- **WHEN** 不帶憑證對示範用 Server 請求 auth-methods
- **THEN** 回應是 200，methods 是 device 與 token，沒有 browser 物件

### Requirement: 每個引擎命令都有線上的歸屬

宿主套件 SHALL 維護一張涵蓋每一種引擎命令的對照：每種命令 SHALL 正好屬於兩類之一——對到一條線上的路由（方法與路徑樣式），或明寫不上線並附理由。引擎新增一種命令而這張對照沒有處理它時，建置 SHALL 失敗。對照 SHALL 與入口實際的路由一致：每一個對到路由的命令，該路由 SHALL 存在且執行的就是那一種命令；入口每一條執行引擎命令的路由 SHALL 被對照裡的某個命令指到。

#### Scenario: 對照涵蓋全部命令

- **WHEN** 對每一種引擎命令查詢它的線上歸屬
- **THEN** 每一種都得到「路由」或「不上線與理由」之一，理由不是空字串

#### Scenario: 對照與路由一致

- **WHEN** 比對對照與入口的路由表
- **THEN** 對照指到的每一條路由都在路由表裡；路由表裡每一條執行引擎命令的路由都被對照指到

#### Scenario: 建立變更對到它的路由

- **WHEN** 查詢「建立變更」命令的線上歸屬
- **THEN** 得到專案底下 changes 的 POST

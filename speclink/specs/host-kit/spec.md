# host-kit Specification

## Purpose

宿主套件的契約：內嵌引擎的宿主建立一個由全部使用者共用的宿主套件，之後每次呼叫給身分與範圍，並能經兩個掛鉤在寫入前拒絕、在提交後旁觀。本 capability 規定宿主能介入什麼與不能介入什麼；引擎怎麼經儲存接口執行屬於 host-runtime，三個儲存函式的保證屬於 store-port-contract。

## Requirements

### Requirement: 宿主套件以儲存接口與選填的掛鉤建立

宿主套件 SHALL 在建立時收下一個儲存接口的實作與一組具名的掛鉤；每個掛鉤 SHALL 是選填。建立之後，同一個宿主套件 SHALL 能同時服務多個使用者與多個範圍的呼叫，呼叫之間 SHALL NOT 共用身分或範圍。沒有提供任何掛鉤時，宿主套件執行一個命令的結果——outcome、領域事件、範圍版本號——SHALL 和直接經儲存接口執行同一個命令相同。

#### Scenario: 沒有掛鉤時結果與直接執行相同

- **WHEN** 以記憶體版儲存建立沒有任何掛鉤的宿主套件，對同樣內容的兩個範圍，一個經宿主套件、一個直接經儲存接口，各執行建立變更與列出變更
- **THEN** 兩邊的 outcome 結構相等、領域事件種類相同、範圍版本號相同

#### Scenario: 兩個使用者交錯呼叫互不影響

- **WHEN** 同一個宿主套件先後以使用者甲在範圍一、使用者乙在範圍二各建立一個變更
- **THEN** 範圍一只有甲建立的變更且建立者是甲的顯示名稱；範圍二只有乙建立的變更且建立者是乙的顯示名稱


<!-- @trace
source: host-kit-agent-door
updated: 2026-11-12T09:48:03+08:00
-->

---
### Requirement: 每次呼叫由宿主給身分與範圍

每次呼叫 SHALL 帶身分與範圍。身分 SHALL 有兩個必填欄位：識別與顯示名稱，兩者都 SHALL NOT 是空字串或只有空白，否則宿主套件 SHALL 在執行前拒絕。身分 MAY 另帶一個選填的顯示用名稱，有帶時同樣 SHALL NOT 是空字串或只有空白；它 SHALL 只用在 HTTP 入口回給用戶端的名稱（握手的操作者名稱、身分查詢的使用者名稱），SHALL NOT 進入提交記錄或任何蓋章值；沒有帶時這些地方 SHALL 用顯示名稱。識別 SHALL 進入提交記錄的操作者識別；顯示名稱 SHALL 進入提交記錄的操作者顯示名稱，並 SHALL 成為這次命令寫進文件的蓋章值（建立者、審查者、驗證者）。提交記錄的命令欄位 SHALL 等於該命令的穩定名稱。識別 SHALL 也是宿主套件路徑上認領（claim）判斷擁有者的依據：識別不同的兩個呼叫即使顯示名稱相同，也 SHALL 是不同的擁有者；識別相同的呼叫即使顯示名稱改變，也 SHALL 是同一個擁有者。宿主套件執行命令時 SHALL NOT 讀取程序的環境變數、git 身分或任何 checkout 來決定操作者。

#### Scenario: 識別與顯示名稱各有去處

- **WHEN** 以識別 `u_42`、顯示名稱 `Alice <alice@example.com>` 的身分經宿主套件建立變更 demo
- **THEN** 儲存收到的提交記錄操作者識別是 `u_42`、顯示名稱是 `Alice <alice@example.com>`、命令是 `new-change`；demo 的 metadata 的 created_by 是 `Alice <alice@example.com>`

#### Scenario: 空的識別在執行前被拒絕

- **WHEN** 以識別是空字串的身分呼叫宿主套件
- **THEN** 呼叫失敗並指出識別不合格；儲存沒有收到任何呼叫

#### Scenario: 同名的兩個人不是同一個擁有者

- **WHEN** 同一個宿主套件以識別 `u_1`、顯示名稱 `Alice` 認領變更 demo，接著以識別 `u_2`、顯示名稱 `Alice` 認領 demo，再以識別 `u_1`、顯示名稱 `Alice Chen` 認領 demo
- **THEN** 第一次寫入 metadata 的 claimed_by 是 `Alice`、claimed_by_id 是 `u_1`；第二次以 `refused` 失敗、訊息含 `Alice`、儲存沒有收到提交；第三次成功、儲存沒有收到提交、claimed_by 仍是 `Alice`

#### Scenario: 程序的 git 身分不影響蓋章

- **WHEN** 程序所在的資料夾設有 git user.name，以顯示名稱 `Bob` 的身分經宿主套件建立變更
- **THEN** 該變更的 created_by 是 `Bob`

#### Scenario: 顯示用名稱只用在入口回給用戶端的名稱

- **WHEN** 以識別 `u_7`、顯示名稱 `Bob <bob@example.com>`、顯示用名稱 `Bob` 的身分，經 HTTP 入口送出握手、身分查詢與建立變更 demo
- **THEN** 握手的操作者名稱與身分查詢的使用者名稱都是 `Bob`；demo 的 created_by 與提交記錄的操作者顯示名稱都是 `Bob <bob@example.com>`

#### Scenario: 空白的顯示用名稱被拒絕

- **WHEN** 以只有空白的顯示用名稱建立身分
- **THEN** 建立失敗並指出顯示用名稱不合格


<!-- @trace
source: host-kit-client-door
updated: 2026-11-12T13:00:47+08:00
-->

---
### Requirement: 執行前掛鉤可以拒絕命令

有提供執行前掛鉤時，宿主套件 SHALL 對每次呼叫正好問它一次，並交給它身分、範圍、命令的穩定名稱與命令的參數。掛鉤 SHALL 只能回答放行或拒絕；它 SHALL NOT 能改變命令、參數或結果。拒絕時，這次呼叫 SHALL 以錯誤碼 `refused` 結束，訊息 SHALL 是宿主給的原因（原因是空字串時用固定的文字），儲存 SHALL 沒有收到提交，提交後掛鉤 SHALL NOT 被呼叫。對流程設定寫入以外的命令，掛鉤 SHALL 在引擎執行之前被問，拒絕時儲存 SHALL 連讀取都沒有收到。沒有提供執行前掛鉤時 SHALL 一律放行。

#### Scenario: 拒絕封存並帶原因

- **WHEN** 執行前掛鉤對命令 `archive` 回答拒絕、原因「程式碼還沒合併」，使用者經宿主套件封存一個任務全部完成的變更
- **THEN** 呼叫以 `refused` 失敗，訊息是「程式碼還沒合併」；該變更仍在進行中的清單裡；儲存沒有收到提交，也沒有收到列出與讀取

#### Scenario: 掛鉤看得到參數裡的旗標

- **WHEN** 使用者帶「未通過審查也放行」的旗標封存變更 demo
- **THEN** 執行前掛鉤收到命令名稱 `archive`，參數裡變更名稱是 demo、這個旗標的欄位是 true

#### Scenario: 放行時照常執行

- **WHEN** 執行前掛鉤對全部命令回答放行，使用者建立一個變更
- **THEN** 變更建立成功，結果和沒有掛鉤時相同；掛鉤正好被問一次

#### Scenario: 查詢也會被問

- **WHEN** 執行前掛鉤對命令 `list` 回答拒絕，使用者列出變更
- **THEN** 呼叫以 `refused` 失敗，沒有回傳任何變更的資料


<!-- @trace
source: host-kit-agent-door
updated: 2026-11-12T09:48:03+08:00
-->

---
### Requirement: 流程設定的寫入在提交前帶著變動的鍵詢問掛鉤

對流程設定的寫入命令，執行前掛鉤收到的資料 SHALL 多一個欄位：這次寫入會變動的鍵，內容 SHALL 等於該命令結果列出的變動的鍵。掛鉤 SHALL 在任何寫入到達儲存之前被問；拒絕時流程設定 SHALL 維持原內容。試算（dry-run）與內容沒有變動的寫入 SHALL 照樣問一次。兩個人同時改流程設定時，掛鉤核准的那一次寫入若根據的內容已被另一人改過，提交 SHALL 以版本衝突失敗，SHALL NOT 寫入掛鉤沒有看過的變動。

#### Scenario: 只有特定的人能改政策鍵

- **WHEN** 執行前掛鉤的規則是「變動的鍵含 tdd 時只放行識別 `u_admin`」，識別 `u_42` 的使用者把 tdd 設成 true
- **THEN** 掛鉤收到命令 `workflow-config-edit` 與變動的鍵 `["tdd"]`；呼叫以 `refused` 失敗；流程設定的內容不變

#### Scenario: 改專案說明不受政策鍵的規則影響

- **WHEN** 同一條規則下，識別 `u_42` 的使用者取代流程設定的 context
- **THEN** 掛鉤收到變動的鍵 `["context"]`；寫入成功

#### Scenario: 整份取代也列出變動的鍵

- **WHEN** 使用者以整份取代送出一份只把 audit 從 false 改成 true 的流程設定
- **THEN** 掛鉤收到命令 `workflow-config-replace` 與變動的鍵 `["audit"]`

#### Scenario: 核准之後內容被別人改過時不寫入

- **WHEN** 掛鉤放行一次流程設定的寫入，而在這次呼叫讀取設定之後、提交之前，另一個呼叫已經改寫了流程設定
- **THEN** 這次呼叫以儲存的版本衝突失敗；流程設定是另一個呼叫寫入的內容


<!-- @trace
source: host-kit-agent-door
updated: 2026-11-12T09:48:03+08:00
-->

---
### Requirement: 提交後掛鉤只能旁觀

有提供提交後掛鉤時，宿主套件 SHALL 在儲存的提交成功回傳之後呼叫它一次，並交給它身分、範圍、命令的穩定名稱、新的範圍版本號與這次提交的領域事件（每筆含名稱與 payload，和交給儲存的那一份相同）。查詢、沒有寫入也沒有事件的命令、被拒絕的命令與失敗的命令 SHALL NOT 觸發它。它 SHALL NOT 能改變這次呼叫的結果：它失敗時，呼叫的結果與儲存的內容 SHALL 和它成功時相同。宿主套件 SHALL NOT 保證這個掛鉤一定被呼叫到；需要保證的宿主 SHALL 在儲存的提交函式裡處理事件，該函式和寫入在同一次提交收到同一份事件。

#### Scenario: 提交後收到事件

- **WHEN** 使用者經宿主套件勾選一個任務成功
- **THEN** 提交後掛鉤被呼叫一次，收到命令 `task-done`、大於執行前的範圍版本號，以及一筆 task-completed 事件

#### Scenario: 查詢不觸發

- **WHEN** 使用者經宿主套件列出變更
- **THEN** 提交後掛鉤沒有被呼叫

#### Scenario: 提交失敗時不觸發

- **WHEN** 儲存的提交回報版本衝突
- **THEN** 呼叫以版本衝突失敗；提交後掛鉤沒有被呼叫

#### Scenario: 掛鉤失敗不影響結果

- **WHEN** 提交後掛鉤在被呼叫時失敗，使用者建立一個變更
- **THEN** 呼叫回傳成功，變更存在於儲存；結果和掛鉤沒有失敗時相同


<!-- @trace
source: host-kit-agent-door
updated: 2026-11-12T09:48:03+08:00
-->

---
### Requirement: 宿主套件的失敗分成命令層與儲存層

宿主套件 SHALL 把失敗分成兩類回報，不合併：命令層的失敗帶引擎的錯誤碼（含掛鉤拒絕的 `refused`）；儲存層的失敗帶儲存契約的分類（版本衝突、暫時無法使用、其他）。任何一類失敗發生時，這次命令的寫入與領域事件 SHALL 全部沒有進入儲存。

#### Scenario: 找不到變更是命令層的失敗

- **WHEN** 使用者經宿主套件對不存在的變更勾選任務
- **THEN** 呼叫以命令層的 `not_found` 失敗；儲存沒有收到提交

#### Scenario: 儲存暫時無法使用是儲存層的失敗

- **WHEN** 儲存在一次讀取回報暫時無法使用
- **THEN** 呼叫以儲存層的「暫時無法使用」失敗，不回報成命令層的錯誤；儲存沒有收到提交

<!-- @trace
source: host-kit-agent-door
updated: 2026-11-12T09:48:03+08:00
-->

---
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

<!-- @trace
source: host-kit-client-door
updated: 2026-11-12T13:00:47+08:00
-->

---
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

<!-- @trace
source: host-kit-client-door
updated: 2026-11-12T13:00:47+08:00
-->

---
### Requirement: 握手、輪詢與範圍清單由入口回答

入口 SHALL 回答握手：回應含操作者（身分的識別與顯示名稱）、專案與 repo 的識別、API 版本、引擎版本與 capabilities；capabilities 的更新方式宣告 SHALL 含輪詢，並 SHALL 只在宿主於建立時宣告推送事件的串流時才含推送的 transport。入口 SHALL NOT 自己提供事件串流。入口 SHALL 回答「範圍有沒有變」的輪詢：請求帶的識別等於當下的範圍版本號時回應 304，否則回應 200 並帶新的識別；輪詢 SHALL NOT 讀取任何文件內容。入口 SHALL 回答範圍清單，內容是範圍函式對該身分回傳的專案與 repo。入口 SHALL 回答專案底下與 API 根的身分查詢，使用者的名稱取自身分的顯示用名稱（宿主沒有給時為顯示名稱）、識別取自身分的識別；握手的操作者名稱同樣取自顯示用名稱。

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

<!-- @trace
source: host-kit-client-door
updated: 2026-11-12T13:00:47+08:00
-->

---
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

<!-- @trace
source: host-kit-client-door
updated: 2026-11-12T13:00:47+08:00
-->

---
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

<!-- @trace
source: host-kit-client-door
updated: 2026-11-12T13:00:47+08:00
-->

---
### Requirement: 入口提供這個範圍的技能

HTTP 入口 SHALL 回答技能清單的請求：用戶端給它要的工具（內建工具的名稱，或帶呼叫形式的自訂工具），入口回傳每個工具要安裝的技能。回傳的集合 SHALL 由三件事決定，依序過濾：只給特定工具的技能不給其他工具；這個範圍的流程設定沒有開啟 worktree 時不含兩支 worktree 技能（設定不存在讀作預設值；設定無法解析時保留這兩支；環境變數不參與）；宿主在建立時給了技能清單時只含清單裡的技能。每支技能的內容 SHALL 由引擎內建的範本以「規格資料夾不在 checkout」渲染；宿主套件 SHALL NOT 接受任何自訂或改寫的技能文字。同一個範圍、同一份流程設定、同一組工具，每個使用者得到的內容 SHALL 相同。這個請求是唯讀的：能讀這個範圍的人都能用，SHALL NOT 受寫入能力限制，SHALL NOT 產生提交。入口能回答這個請求時，握手 SHALL 宣告技能範本的版號。宿主在建立時給的技能清單含不認得的名稱時，建立 SHALL 失敗並列出這些名稱。技能清單是空的時建立 SHALL 同樣失敗：不提供任何技能的入口會讓自它更新的 checkout 失去既有的技能，要提供全部技能就不給清單。一次請求的工具數超過 32 時 SHALL 以 invalid_argument 拒絕。

#### Scenario: worktree 政策開啟時含兩支 worktree 技能

- **WHEN** 範圍的流程設定為 worktree: true，請求 claude 的技能清單
- **THEN** 回應含 apply-with-worktree 與 worktree-merge

#### Scenario: worktree 政策關閉時不含

- **WHEN** 範圍沒有流程設定文件，請求 claude 的技能清單
- **THEN** 回應不含 apply-with-worktree 與 worktree-merge，含其餘的技能

#### Scenario: 宿主只提供部分技能

- **WHEN** 宿主以技能清單 discuss、propose 建立宿主套件，請求 claude 的技能清單
- **THEN** 回應只含 discuss 與 propose

#### Scenario: 只給 Claude 的技能不給其他工具

- **WHEN** 同時請求 claude 與 codex 的技能清單
- **THEN** claude 的清單含 analyze，codex 的清單不含

#### Scenario: 內容以規格不在 checkout 渲染

- **WHEN** 請求 claude 的技能全文
- **THEN** 每支技能的內容不含規格資料夾的路徑，frontmatter 的版本等於握手宣告的技能版號

#### Scenario: 自訂工具得到中性渲染

- **WHEN** 請求一個呼叫形式為 tool-call 的自訂工具的技能全文
- **THEN** 內容的開頭是 tool-call 的呼叫說明，對其他技能的引用是 `speclink-<名稱>`

#### Scenario: 沒有寫入能力的人也能取得

- **WHEN** 寫入能力為否的人請求技能清單
- **THEN** 回應是 200；儲存沒有收到提交

#### Scenario: 技能清單含不認得的名稱

- **WHEN** 以技能清單 discuss、no-such-skill 建立宿主套件
- **THEN** 建立失敗，訊息列出 no-such-skill

#### Scenario: 空的技能清單

- **WHEN** 以空的技能清單建立宿主套件
- **THEN** 建立失敗，訊息說明清單是空的

#### Scenario: 握手宣告技能版號

- **WHEN** 對有範圍函式的宿主套件送出握手
- **THEN** 回應的 capabilities 含 skills，其 version 是引擎的技能範本版號

<!-- @trace
source: remote-skill-delivery
updated: 2026-11-13T14:35:20+08:00
-->
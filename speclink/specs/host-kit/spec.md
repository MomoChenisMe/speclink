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


<!-- @trace
source: door-reads-ask-before-command
updated: 2026-11-23T15:04:25+08:00
-->

---
### Requirement: 範圍與能力由宿主的函式決定

宿主套件 SHALL 在建立時接受一個選填的函式：給一個身分，回傳這個人可以使用的專案清單，每個專案含識別字與名稱及其 repo 清單，每個 repo 含識別字、名稱、儲存用的範圍識別與兩個能力開關（能否寫入、能否改流程政策）。入口 SHALL 用它把網址裡的專案與請求指定的 repo 對到範圍識別：專案不在清單裡 SHALL 回應 404；請求指定的 repo 不在該專案的清單裡 SHALL 回應 404；沒有指定 repo 而專案只有一個 repo SHALL 使用它；沒有指定而有多個 SHALL 以 409 拒絕並列出候選，SHALL NOT 自動選擇；拒絕的訊息 SHALL 同時指出兩種指定方式：`X-Speclink-Repo` 標頭，以及 `.speclink.yaml` 的 `remote.repo`。專案底下的身分查詢是例外：它 SHALL NOT 經過 repo 的選擇——不論專案有幾個 repo、請求有沒有指定 repo、指定的 repo 是否在清單裡，回應 SHALL 是 200 並列出該專案的全部 repo，且 SHALL NOT 帶 ETag；沒有身分時的 401、專案不在清單裡的 404 與 API 版本標頭不符的 409 SHALL 照常。入口 SHALL NOT 自行快取這個函式的結果。沒有寫入能力的人送出寫入請求 SHALL 得到 403 且儲存不變；唯讀的 Context 快照請求不受此限。沒有改流程政策能力的人送出會變動政策鍵（schema、locale、spec_locale、tdd、audit、worktree）的流程設定寫入 SHALL 得到 403 且流程設定不變；只變動其他鍵的寫入 SHALL 照常。這個函式失敗時回應 SHALL 是 503。沒有提供這個函式時，範圍清單與全部專案底下的路徑 SHALL 回應 404。API 版本標頭不符的專案請求 SHALL 以 409 拒絕。

#### Scenario: 專案不在這個人的清單裡

- **WHEN** 範圍函式對某個身分只回傳專案 alpha，該身分請求專案 beta 的變更清單
- **THEN** 回應是 404、reason 為 not_found；儲存沒有被呼叫

#### Scenario: 多個 repo 而沒有指定

- **WHEN** 專案 alpha 有 web 與 api 兩個 repo，請求沒有指定 repo
- **THEN** 回應是 409，訊息列出 web 與 api，並含 `X-Speclink-Repo` 與 `remote.repo` 兩個字樣；沒有任何範圍被讀取

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

#### Scenario: 多個 repo 的專案查詢身分

- **WHEN** 專案 alpha 有 web 與 api 兩個 repo，該專案的成員送出專案底下的身分查詢，請求沒有指定 repo
- **THEN** 回應是 200，本文含該使用者的名稱與識別，repos 列出 web 與 api；回應沒有 ETag 標頭；沒有任何範圍被讀取

#### Scenario: 身分查詢不看指定的 repo

- **WHEN** 專案 alpha 有 web 與 api 兩個 repo，請求指定一個不在清單裡的 repo nope，送出專案底下的身分查詢
- **THEN** 回應是 200，repos 列出 web 與 api；同一個請求改送變更清單時回應是 404

#### Scenario: 身分查詢仍檢查專案與版本

- **WHEN** 範圍函式對某個身分只回傳專案 alpha，該身分送出專案 beta 底下的身分查詢；另以不符的 API 版本標頭送出專案 alpha 底下的身分查詢
- **THEN** 第一個回應是 404、reason 為 not_found；第二個回應是 409、reason 為 refused


<!-- @trace
source: project-whoami-and-denied-state
updated: 2026-11-21T17:43:14+08:00
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

---
### Requirement: 呼叫入口時附上自由欄位

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


<!-- @trace
source: reserved-field-prefixes
updated: 2026-11-23T16:56:02+08:00
-->

---
### Requirement: 範圍的匯出與匯入

宿主套件 SHALL 提供一個範圍的匯出與匯入，兩者 SHALL 只用儲存接口的列出、讀取、提交完成，使任何實作儲存接口的儲存不需額外實作即可使用。匯出 SHALL 回傳範圍內全部文件的鍵、內容與內容摘要，以及匯出當時的範圍版本號；匯出期間範圍被改動時 SHALL 重試，仍無法取得一致的內容時 SHALL 回報暫時無法使用，SHALL NOT 回傳新舊混合的內容。匯入 SHALL 有兩種模式：建立模式（預設）與取代模式（見「匯入的取代模式與試跑」）。兩種模式 SHALL 在呼叫儲存之前檢查：每份文件的摘要與內容相符、鍵符合儲存契約的格式、鍵屬於引擎的命名規則、沒有重複的鍵、看板順序文件不超過看板順序寫入的大小上限。建立模式 SHALL 只接受列出時沒有任何文件的範圍；檢查通過後 SHALL 以正好一次提交寫入全部文件——每一筆預期「必須不存在」——並在同一次提交帶一筆 scope-imported 事件。建立模式下，任何檢查不過、列出時目標範圍不是空的、提交時發現包裡的某個鍵已被另一方寫入、或提交失敗時，匯入 SHALL 被拒絕，包裡的文件 SHALL 一份都沒有被寫入。儲存接口的提交只能對個別的鍵下預期，所以在列出與提交之間另一方寫入一份鍵不在包裡的文件時，匯入不會察覺：這時匯入 SHALL 成功，那份文件 SHALL 原樣保留，範圍裡 SHALL NOT 出現只寫入一部分的包。匯出與匯入 SHALL 經過執行前掛鉤（命令名稱 scope-export 與 scope-import；scope-import 的參數 SHALL 是 `documents`（份數）、`bytes`（總位元組數）、`mode`（`create` 或 `replace`）與 `dryRun`（布林）四個鍵，SHALL NOT 含文件內容），匯入成功後 SHALL 通知提交後掛鉤。兩者 SHALL 同時可由宿主的程式直接呼叫，以及經 HTTP 入口使用（匯入為 POST /import，匯出為 GET /export）；經 HTTP 入口時兩者 SHALL 都要求寫入的能力；經 HTTP 入口的匯入 SHALL 一律是建立模式、不試跑；請求本文帶 `mode` 或 `dryRun` 欄位時，它不是合格的包，SHALL 與帶其他多餘欄位時一樣以 422 被拒絕，儲存與執行前掛鉤 SHALL NOT 被呼叫。匯入的回報 SHALL 含範圍版本號、寫入的份數，以及新增、更新、刪除三個鍵的清單（各依鍵的位元組順序排列）；建立模式的新增清單 SHALL 是包裡全部的鍵，另外兩個清單 SHALL 是空的。匯出一個範圍再匯入另一個空的範圍後，兩個範圍的鍵與每份文件的內容 SHALL 相同。

#### Scenario: 匯出再匯入後內容相同

- **WHEN** 匯出一個含進行中變更、正式規格、討論、已封存變更、流程設定、evidence 與看板順序的範圍，把結果匯入另一個空的範圍
- **THEN** 兩個範圍以空的前綴列出的鍵相同，每個鍵讀到的內容相同

#### Scenario: 匯入是一次提交

- **WHEN** 把含 300 份文件的包匯入空的範圍
- **THEN** 儲存收到正好一次提交，內含 300 筆預期「必須不存在」的寫入與一筆 scope-imported 事件

#### Scenario: 非空的範圍

- **WHEN** 目標範圍已有一份文件，匯入一個合格的包
- **THEN** 匯入被拒絕；儲存沒有收到提交，既有的文件不變

#### Scenario: 匯入途中另一方寫入

- **WHEN** 匯入在確認範圍是空的之後、提交之前，另一方對同一個範圍寫入了一份鍵與包裡某份文件相同的文件
- **THEN** 匯入以「範圍不是空的」被拒絕；包裡的文件一份都沒有被寫入

#### Scenario: 匯入途中另一方寫入包裡沒有的鍵

- **WHEN** 匯入在確認範圍是空的之後、提交之前，另一方對同一個範圍寫入了一份鍵不在包裡的文件
- **THEN** 匯入成功；範圍裡有包的全部文件，以及另一方寫入的那一份文件

#### Scenario: 摘要不符

- **WHEN** 包裡一份文件的摘要與內容不符
- **THEN** 匯入被拒絕並指出該鍵；儲存沒有被呼叫

#### Scenario: 不屬於引擎命名規則的鍵

- **WHEN** 包裡有一份鍵為 `secrets/token` 的文件
- **THEN** 匯入被拒絕並指出該鍵；儲存沒有被呼叫

#### Scenario: 重複的鍵

- **WHEN** 包裡有兩份相同鍵的文件
- **THEN** 匯入被拒絕；儲存沒有被呼叫

#### Scenario: 看板順序文件超過上限

- **WHEN** 包裡的看板順序文件比看板順序寫入的大小上限多 1 個位元組
- **THEN** 匯入被拒絕並指出該鍵；儲存沒有被呼叫

#### Scenario: 空的範圍匯出

- **WHEN** 匯出一個沒有任何文件的範圍
- **THEN** 得到沒有文件的包，不是錯誤

#### Scenario: 匯出期間範圍持續變動

- **WHEN** 匯出的每一次嘗試期間都有另一方提交
- **THEN** 匯出在有限次重試後回報暫時無法使用，沒有回傳任何包

#### Scenario: 掛鉤拒絕匯入

- **WHEN** 執行前掛鉤對 scope-import 回答拒絕
- **THEN** 匯入以 refused 失敗並帶宿主的原因；目標範圍不變

#### Scenario: 掛鉤看不到內容

- **WHEN** 匯入一個含 3 份文件、共 1200 位元組的包
- **THEN** 執行前掛鉤收到命令名稱 scope-import 與參數 documents 3、bytes 1200、mode `create`、dryRun `false`，參數沒有文件內容

#### Scenario: 經 HTTP 入口匯出需要寫入能力

- **WHEN** 寫入能力為否的人請求 GET /export
- **THEN** 回應是 403

#### Scenario: 建立模式的回報列出全部的鍵

- **WHEN** 把含 `specs/auth/spec.md` 與 `LANGUAGE.md` 兩份文件的包以建立模式匯入空的範圍
- **THEN** 回報的寫入份數是 2；新增清單是 `LANGUAGE.md`、`specs/auth/spec.md`（依鍵的位元組順序）；更新清單與刪除清單都是空的

#### Scenario: 經 HTTP 入口的匯入一律是建立模式

- **WHEN** 目標範圍已有一份文件，原版的用戶端送出 POST /import，請求本文是合格的包
- **THEN** 匯入以「範圍不是空的」被拒絕；執行前掛鉤收到的 mode 是 `create`、dryRun 是 `false`；既有的文件不變

#### Scenario: 經 HTTP 入口帶 mode 或 dryRun 的請求照舊被拒絕

- **WHEN** 目標範圍已有一份文件，送出 POST /import，請求本文除了合格的包的欄位之外還帶 `"mode": "replace"` 或 `"dryRun": true`
- **THEN** 回應是 422，和帶其他多餘欄位時相同；儲存與執行前掛鉤都沒有被呼叫；既有的文件不變


<!-- @trace
source: import-replace-mode
updated: 2026-11-22T10:01:56+08:00
-->

---
### Requirement: 匯入的大小上限由宿主設定

宿主套件 SHALL 在建立時接受選填的匯入大小上限（位元組）；沒有提供時 SHALL 為 32 MiB。經 HTTP 入口的匯入，本文超過上限時 SHALL 在解析之前以 413 拒絕；直接呼叫的匯入，文件內容的總位元組數超過上限時 SHALL 拒絕。兩種情況目標範圍 SHALL 沒有變動。匯出 SHALL NOT 受這個上限限制。宿主套件 SHALL NOT 壓縮或解壓縮匯入匯出的內容。

#### Scenario: 預設的上限

- **WHEN** 沒有設定上限的宿主套件收到本文為 33 MiB 的匯入請求
- **THEN** 回應是 413；儲存沒有被呼叫

#### Scenario: 宿主調高上限

- **WHEN** 以 128 MiB 的上限建立宿主套件，收到本文為 40 MiB 的合格匯入請求
- **THEN** 匯入成功

#### Scenario: 宿主調低上限

- **WHEN** 以 1 MiB 的上限建立宿主套件，收到本文為 2 MiB 的匯入請求
- **THEN** 回應是 413

#### Scenario: 匯出不受限

- **WHEN** 以 1 MiB 的上限建立宿主套件，匯出一個內容共 5 MiB 的範圍
- **THEN** 匯出成功

<!-- @trace
source: import-export-on-port
updated: 2026-11-14T22:04:44+08:00
-->

---
### Requirement: 執行前掛鉤可以用讀取器讀取

宿主套件問執行前掛鉤時，SHALL 一併交給它一個讀取器。掛鉤 SHALL 能用讀取器執行讀取類的命令，拿到的結果 SHALL 與同一個身分在同一個範圍經宿主套件執行該命令的結果相同。讀取器 SHALL 遵守下列規則：

- 只執行讀取類的命令。讀取類的命令是不寫入任何文件的命令：列出變更與正式規格、顯示變更或正式規格、變更的狀態、產出指示、驗證、分析、排程、溯源、讀 evidence、讀審查工單與驗證工單、列出／顯示／搜尋討論、讀 artifact、讀詞彙表、讀流程設定。其他命令 SHALL 以錯誤碼 `invalid_argv` 拒絕，訊息 SHALL 指出讀取器只接受讀取類的命令並點名被拒絕的命令；拒絕時儲存 SHALL 連讀取都沒有收到。
- 經讀取器執行的命令 SHALL NOT 再問執行前掛鉤，SHALL NOT 呼叫提交後掛鉤，儲存 SHALL NOT 收到提交。
- 讀取器 SHALL 沿用這次呼叫的身分與範圍；掛鉤 SHALL NOT 能經它以別的身分或別的範圍讀取。
- 經讀取器執行的命令失敗時（例如變更不存在），讀取器 SHALL 回傳該命令的錯誤（錯誤碼與訊息與直接執行時相同）；這個失敗本身 SHALL NOT 結束外層的呼叫，由掛鉤決定放行或拒絕。

讀取器讀到的內容與外層命令自己讀到的內容 SHALL NOT 被保證是同一份：兩者是各自獨立的讀取，中間有別人提交時內容可能不同。引擎既有的守門（例如封存時的任務完成度、未結工單與章失效判定）SHALL 照常以外層命令自己讀到的內容判斷。

兩個入口 SHALL 行為相同：經 Agent 的入口與經 HTTP 入口進來的呼叫，掛鉤都拿到讀取器。流程設定的寫入在提交前詢問掛鉤時，掛鉤 SHALL 同樣拿到讀取器。既有的規則 SHALL 不變：每次呼叫掛鉤正好被問一次；掛鉤沒有使用讀取器時，對流程設定寫入以外的命令，拒絕時儲存 SHALL 連讀取都沒有收到。

#### Scenario: 掛鉤讀出變更的狀態後拒絕封存

- **WHEN** 變更 demo 的任務全部完成、沒有審查章；執行前掛鉤在命令是 `archive` 時用讀取器執行「顯示變更 demo」，發現結果沒有已蓋章的審查狀態，回答拒絕、原因「還沒通過審查」；使用者經宿主套件封存 demo
- **THEN** 呼叫以 `refused` 失敗，訊息是「還沒通過審查」；demo 仍在進行中的清單裡；儲存沒有收到提交；執行前掛鉤正好被問一次（命令是 `archive`），沒有因為讀取而被再問一次

#### Scenario: 掛鉤讀出狀態後放行

- **WHEN** 變更 demo 的任務全部完成、審查與驗證都已蓋章、有落地記錄；執行前掛鉤在命令是 `archive` 時用讀取器讀出這三項都成立，回答放行
- **THEN** 封存成功，結果與沒有掛鉤時相同

#### Scenario: 讀取器拒絕會寫入的命令

- **WHEN** 執行前掛鉤用讀取器執行「建立變更 other」
- **THEN** 讀取器回傳錯誤碼 `invalid_argv` 的錯誤，訊息點名該命令；儲存沒有因為這次讀取器的呼叫收到列出、讀取或提交；範圍裡沒有變更 other

##### Example: 哪些命令可以經讀取器執行

| 命令 | 經讀取器執行的結果 |
| --- | --- |
| 顯示變更 demo（`show`） | 回傳結果 |
| 列出變更（`list`） | 回傳結果 |
| 讀審查工單（`review show`） | 回傳結果 |
| 讀流程設定（`workflow-config show`） | 回傳結果 |
| 建立變更（`new change`） | `invalid_argv` |
| 勾選任務（`task done`），即使該任務已經勾過 | `invalid_argv` |
| 封存（`archive`） | `invalid_argv` |
| 認領（`claim`） | `invalid_argv` |

#### Scenario: 讀取的對象不存在時由掛鉤決定

- **WHEN** 執行前掛鉤用讀取器執行「顯示變更 ghost」，而範圍裡沒有 ghost
- **THEN** 讀取器回傳錯誤碼 `not_found` 的錯誤；外層的呼叫沒有因此結束，掛鉤接著回答放行時外層命令照常執行

#### Scenario: 讀取器沿用呼叫的身分與範圍

- **WHEN** Alice 在範圍 s1 發出一個呼叫，執行前掛鉤用讀取器列出變更；另一個範圍 s2 有變更 other
- **THEN** 讀取器回傳的清單只含 s1 的變更，不含 other

#### Scenario: HTTP 入口進來的呼叫也拿得到讀取器

- **WHEN** 原版用戶端經 HTTP 入口送出封存變更 demo 的請求，執行前掛鉤用讀取器讀出 demo 沒有落地記錄並回答拒絕、原因「程式碼還沒落地」
- **THEN** 請求得到拒絕的回應，訊息是「程式碼還沒落地」；儲存沒有收到提交

#### Scenario: 沒有使用讀取器時儲存沒有收到讀取

- **WHEN** 執行前掛鉤沒有使用讀取器，對命令 `archive` 直接回答拒絕
- **THEN** 呼叫以 `refused` 失敗，儲存沒有收到列出、讀取與提交，與引入讀取器之前相同

<!-- @trace
source: hook-read-function
updated: 2026-11-18T15:33:01+08:00
-->

---
### Requirement: 匯入的取代模式與試跑

宿主的程式直接呼叫匯入時 SHALL 能指定取代模式。取代模式 SHALL 讓目標範圍的內容等於包：包有而範圍沒有的鍵 SHALL 被新增；兩邊都有而內容不同的鍵 SHALL 被更新；兩邊都有而內容相同的鍵 SHALL NOT 被寫入，它的版本與修改時間 SHALL 不變；範圍有而包沒有的鍵 SHALL 被刪除，不論它是哪一種文件——流程設定、匯入之後才建立的變更與討論、不屬於引擎命名規則的鍵都算。取代模式 SHALL 只用儲存接口的列出、讀取、提交完成；全部的寫入與刪除 SHALL 在正好一次提交裡，新增的寫入預期「必須不存在」，更新的寫入與刪除預期「必須是列出時的版本」，同一次提交 SHALL 帶一筆 scope-imported 事件，payload 含寫入的份數 `documents` 與刪除的份數 `removed`。沒有任何鍵需要寫入或刪除時 SHALL NOT 提交：回報的三個清單 SHALL 是空的，範圍版本號 SHALL 不變，SHALL NOT 有事件，提交後掛鉤 SHALL NOT 被通知。

取代模式 SHALL 在呼叫儲存之前做與建立模式相同的檢查，匯入的大小上限 SHALL 同樣適用。包沒有任何文件時，取代模式 SHALL 拒絕，儲存 SHALL NOT 被呼叫：一次取代 SHALL NOT 能清空整個範圍。列出之後、提交之前，另一方寫入或刪除了這次會寫入或刪除的鍵時，取代 SHALL 整筆被拒絕，範圍 SHALL 與另一方寫入之後相同，包裡的文件 SHALL 一份都沒有被寫入；取代 SHALL NOT 自動重試。儲存接口的提交只能對個別的鍵下預期，所以另一方在這段時間改了一個內容相同而不需要寫入的鍵，或新增了一個列出時不存在、包裡也沒有的鍵時，取代不會察覺：這時取代 SHALL 成功，那份文件 SHALL 保留另一方的內容。

兩種模式 SHALL 都能試跑。試跑 SHALL 做全部的檢查、列出與比對，並回報與真的執行相同的新增、更新、刪除三個清單與寫入的份數；試跑 SHALL NOT 呼叫儲存的提交，回報的範圍版本號 SHALL 是列出時的版本，提交後掛鉤 SHALL NOT 被通知。試跑與真的執行 SHALL 同樣先經過執行前掛鉤，掛鉤拒絕時儲存 SHALL 連讀取都沒有收到。建立模式對非空的範圍試跑時 SHALL 與真的執行一樣被拒絕。

取代模式與試跑 SHALL 只在宿主的程式直接呼叫時提供，SHALL NOT 經 HTTP 入口提供（見「範圍的匯出與匯入」）。

#### Scenario: 取代讓範圍等於包

- **WHEN** 範圍有下面例子的五份文件，以取代模式匯入例子的包
- **THEN** 儲存收到正好一次提交，內含 3 筆寫入、2 筆刪除與一筆 scope-imported 事件（payload 的 `documents` 是 3、`removed` 是 2）；之後範圍以空的前綴列出的鍵正好是包的四個鍵，每個鍵的內容等於包的內容；`LANGUAGE.md` 的版本與修改時間沒有改變

##### Example: 取代前的範圍、包與回報

| 鍵 | 取代前的範圍 | 包 | 回報 |
| --- | --- | --- | --- |
| `LANGUAGE.md` | 內容 A | 內容 A | 不在任何清單 |
| `changes/add-x/proposal.md` | 有 | 沒有 | 刪除 |
| `config.yaml` | 含 `tdd: true` | 沒有 `tdd` | 更新 |
| `discussions/d1.md` | 有 | 沒有 | 刪除 |
| `specs/auth/spec.md` | 舊內容 | 新內容 | 更新 |
| `specs/billing/spec.md` | 沒有 | 有 | 新增 |

回報的寫入份數是 3；新增清單是 `specs/billing/spec.md`；更新清單是 `config.yaml`、`specs/auth/spec.md`；刪除清單是 `changes/add-x/proposal.md`、`discussions/d1.md`。

#### Scenario: 試跑不寫入

- **WHEN** 對同一個範圍以取代模式試跑同一個包
- **THEN** 回報的三個清單與寫入的份數和真的執行相同；回報的範圍版本號是試跑前的版本；儲存的提交沒有被呼叫，範圍的每份文件不變；提交後掛鉤沒有被通知

#### Scenario: 再取代一次時沒有提交

- **WHEN** 取代成功之後，緊接著以取代模式再匯入同一個包
- **THEN** 匯入成功；回報的三個清單都是空的，寫入的份數是 0，範圍版本號與上一次的回報相同；儲存的提交沒有被呼叫

#### Scenario: 空的包不能取代

- **WHEN** 範圍有三份文件，以取代模式匯入一個沒有任何文件的包
- **THEN** 匯入被拒絕；儲存沒有被呼叫，範圍的三份文件不變

#### Scenario: 取代途中另一方寫入會動到的鍵

- **WHEN** 取代在列出範圍之後、提交之前，另一方更新了 `specs/auth/spec.md`（這次取代要更新的鍵）
- **THEN** 取代以「範圍在匯入途中改變」被拒絕；包裡的文件一份都沒有被寫入，範圍裡沒有文件被刪除，`specs/auth/spec.md` 是另一方寫入的內容

#### Scenario: 取代途中另一方新增包裡沒有的鍵

- **WHEN** 取代在列出範圍之後、提交之前，另一方新增了 `discussions/d2.md`（列出時不存在，包裡也沒有）
- **THEN** 取代成功；範圍裡有包的全部文件，以及另一方寫入的 `discussions/d2.md`

#### Scenario: 掛鉤看得到取代與試跑

- **WHEN** 以取代模式試跑一個含 3 份文件、共 1200 位元組的包
- **THEN** 執行前掛鉤正好被問一次，命令名稱是 scope-import，參數是 documents 3、bytes 1200、mode `replace`、dryRun `true`，參數沒有文件內容

#### Scenario: 掛鉤拒絕取代

- **WHEN** 執行前掛鉤對 mode 是 `replace` 的 scope-import 回答拒絕
- **THEN** 匯入以 refused 失敗並帶宿主的原因；儲存連列出都沒有收到，範圍不變

#### Scenario: 取代也受大小上限限制

- **WHEN** 宿主套件的匯入上限是 1,024 位元組，以取代模式匯入內容共 2,000 位元組的包
- **THEN** 匯入被拒絕；儲存沒有被呼叫，範圍不變

<!-- @trace
source: import-replace-mode
updated: 2026-11-22T10:01:56+08:00
-->

---
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

<!-- @trace
source: door-reads-ask-before-command
updated: 2026-11-23T15:04:25+08:00
-->

---
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

<!-- @trace
source: reserved-field-prefixes
updated: 2026-11-23T16:56:02+08:00
-->
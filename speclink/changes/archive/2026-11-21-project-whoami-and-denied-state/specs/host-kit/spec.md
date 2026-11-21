## MODIFIED Requirements

### Requirement: 範圍與能力由宿主的函式決定
<!-- BEFORE: 專案底下的身分查詢和其他路由一樣先選 repo（多個 repo 而沒有指定時 409）且回應帶 ETag；409 的訊息只提 X-Speclink-Repo 標頭 -->

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

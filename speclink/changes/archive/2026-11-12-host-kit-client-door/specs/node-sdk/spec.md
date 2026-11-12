## ADDED Requirements

### Requirement: host.handleRequest 與 createHost 的入口設定

createHost SHALL 另外接受三個選填的欄位：`scopes`（函式，收身分並回傳專案清單，回傳值或 Promise 皆可）、`authMethods`（登入方式的宣告）、`pushTransports`（推送事件的宣告）。`scopes` 回傳的每個專案是 `{ key, name, repos }`，每個 repo 是 `{ key, name, scope, canWrite, canWritePolicy }`，欄位一律 camelCase。Node SDK SHALL 提供 host.handleRequest：第一個參數是 `{ method, path, query, headers, body }`（headers 是字串對字串的物件，body 是位元組陣列，query 與 body 可省略），第二個參數是身分或 null；回傳 Promise，解析為 `{ status, headers, body }`，或在路徑不屬於入口時解析為 null。它的行為 SHALL 等同宿主套件的 HTTP 入口，並與 host.dispatch 共用同一個儲存與同一組掛鉤。handleRequest SHALL 於背景工作執行，SHALL NOT 阻塞 JS 事件迴圈。`scopes` 拋出例外或 Promise 被拒絕時回應 SHALL 是 503。方法不是 GET、HEAD、POST、PUT、DELETE 時 SHALL 解析為狀態碼 405 的回應。handleRequest SHALL NOT 讀取 Authorization 標頭來決定身分。

#### Scenario: 用 JS 儲存服務握手與寫入

- **WHEN** 以 JS 的記憶體儲存與回傳單一專案單一 repo 的 scopes 建立 host，依序以同一個身分對 handleRequest 送出握手、建立變更 demo、列出變更
- **THEN** 三個回應的 status 都是 200，body 解析後是 Client Protocol 的對應 payload；儲存的 commit 被呼叫正好一次

#### Scenario: 身分為 null

- **WHEN** 以 null 當身分送出變更清單的請求
- **THEN** 回應的 status 是 401

#### Scenario: 不屬於入口的路徑

- **WHEN** 送出路徑為 `/healthz` 的請求
- **THEN** Promise 解析為 null

#### Scenario: 兩個門共用掛鉤

- **WHEN** beforeCommand 對 archive 回傳拒絕，分別經 host.dispatch 與 host.handleRequest 封存同一個變更
- **THEN** 前者以 code 為 refused 的 Error 拒絕，後者的回應 status 是 409 且 body 的 reason 是 refused；兩次的訊息相同

#### Scenario: scopes 是非同步函式

- **WHEN** scopes 是稍後才解析的 async 函式
- **THEN** handleRequest 等它解析後照常回應

#### Scenario: scopes 拋出例外

- **WHEN** scopes 拋出例外，送出變更清單的請求
- **THEN** 回應的 status 是 503

#### Scenario: 登入方式的宣告

- **WHEN** createHost 帶 `authMethods: { methods: ['browser'], browser: { issuer, clientId } }`，以 null 當身分請求 `/api/speclink/v1/auth-methods`
- **THEN** 回應的 status 是 200，body 含 methods 與 browser 的 issuer、clientId

#### Scenario: 不支援的方法

- **WHEN** 以 method 為 PATCH 送出請求
- **THEN** 回應的 status 是 405；儲存沒有被呼叫

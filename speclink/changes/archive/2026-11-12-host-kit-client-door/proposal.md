## Why

原版的 CLI 與 Speclink Desktop 連到 Server 時會送出約五十種請求，每一種有固定的網址、請求格式、回應格式與錯誤碼。這些格式的完整實作今天只存在於官方的示範 server 裡，而且和它的 HTTP 框架、帳號系統寫在一起。

把 speclink 內嵌進自己後端的宿主因此沒有路可走：它要嘛旁掛一台示範 server（討論已否決），要嘛自己重做這五十種端點並跟著 speclink 改版（討論也已否決）。

前面的變更已經備好零件：宿主套件與兩個掛鉤（host-kit-agent-door）、不經引擎的文件讀寫函式（server-routes-on-store-port）、登入方式宣告的線上型別（client-browser-login）。這個變更把請求處理搬進宿主套件，成為它的第二個門：宿主只寫一條路由，驗完 token 後把「請求加身分」交給入口。示範 server 改成用同一個入口，所以它從此是宿主套件的第一個使用者，也是對照。

- **目標使用者**：把 speclink 內嵌進自己後端的整合者。間接受益的是連到這種宿主的開發者：他們用原版的 CLI 與 Speclink Desktop，不用任何特製的用戶端。
- **使用情境**：工程師在自己的電腦用 CLI 與 Speclink Desktop 連到宿主，做 apply、review、verify；宿主這一端只有一條路由。
- **既有規格的對照**：server-verb-api、client-protocol、verb-contract、remote-board-order、server-policy-write 規定各路由的線上行為，這個變更不改它們。host-kit（由 host-kit-agent-door 建立）規定宿主套件的建立、身分與掛鉤，沒有 HTTP 的門。node-sdk 沒有 HTTP 的入口。沒有正式規格規定「範圍與能力由宿主的函式決定」或「登入方式由宿主宣告」。

Source discussion: embedded-host-needs（cut 7 的第二塊；第一塊是 server-routes-on-store-port）。

## What Changes

- `speclink-host` 的宿主套件新增 **HTTP 的入口**：收一個和框架無關的請求（方法、路徑、查詢字串、標頭、本文）與身分（驗不出來時是「沒有」），回傳一個回應（狀態碼、標頭、本文），或回報「這不是入口的路徑」。
- 入口負責的路徑都在 `/api/speclink/v1/` 底下：
  - 專案底下的全部動詞與讀取路由（從示範 server 原樣搬來，網址、格式、狀態碼不變）。
  - 握手、輪詢範圍有沒有變、專案底下的 whoami。
  - 範圍清單。
  - 新增兩條在 API 根的路徑：登入方式的宣告（不需要身分）與身分查詢。
- 入口自己做的檢查，依序：需要身分的路徑沒有身分 → 401；API 版本標頭不符 → 既有的拒絕；用宿主給的函式找出這個人能用的範圍，找不到 → 404，repo 有多個而沒有指定 → 既有的多義拒絕；寫入的請求而這個人沒有寫入的能力 → 403。
- **建立宿主套件時新增三個選填的輸入**：
  - 「列出這個人可以用哪些範圍」的函式：回傳專案與 repo 的清單，每個 repo 帶儲存用的範圍識別與兩個能力開關（能不能寫入、能不能改流程政策）。沒有提供時，入口對專案與範圍清單的路徑回 404。
  - 登入方式的宣告：支援哪些方式、瀏覽器登入的發行者網址與用戶端識別。沒有提供時，宣告的路徑回 404，用戶端就走既有的行為。
  - 推送事件的宣告：宿主自己提供串流時填它的網址。預設是空的，握手只宣告輪詢。
- 能力開關的作用：握手回應用它們填 `capabilities`；入口也照它們把關，所以握手說的和實際做的一致。沒有改流程政策的能力時，會變動政策鍵的流程設定寫入被拒絕。
- 掛鉤照樣生效：經入口執行的引擎命令與看板順序的寫入，都會先問執行前掛鉤、提交後通知提交後掛鉤。
- **每個引擎命令的線上歸屬**：宿主套件放一張涵蓋全部引擎命令的對照，每個命令必須選一個——對到哪一條路由，或明寫不上線與理由。引擎新增命令而沒有處理時編譯不過。
- `speclink-node`：`createHost` 接受上面三個輸入；新增 `host.handleRequest(request, identity)`。宿主在自己的 Web 框架裡寫一條路由，把請求轉交給它。
- **示範 server 改用宿主套件**：專案路由與範圍清單改成一個轉交的處理函式。它留下自己的部分：登入與帳號、管理介面、靜態資源、匯入（等變更 import-export-on-port）、推送事件的串流。請求轉命令、結果轉線上格式、錯誤對應的程式從 `speclink-server` 移到 `speclink-host`。
- **相容性影響**：
  - CLI 與 Speclink Desktop 連到示範 server：全部既有路由的回應不變。
  - 示範 server 新增兩條路徑：登入方式的宣告（回 `device` 與 `token`）與 API 根的身分查詢。新版的用戶端因此會走「有宣告」的分支，選到的仍是裝置登入，流程不變。
  - 示範 server 的錯誤：專案不存在回 404、存在但不是成員回 403，維持不變（示範 server 在轉交前自己先查這一項）。
  - 提交記錄的操作者識別從顯示名稱變成使用者的識別；顯示名稱不變。這只出現在儲存的記錄裡。
- **CLI 指令、設定欄位、技能**：都沒有新增或變更。

## Non-Goals

- **登入的端點、帳號與成員、管理介面、備份**：入口不做。它們是宿主（或示範 server）自己的事。
- **驗證 token**：宿主驗完再把身分交給入口；入口不看 `Authorization` 標頭。
- **匯入匯出**：變更 import-export-on-port。在那之前示範 server 的匯入路由留在 server。
- **推送事件的串流本身**：入口只在握手宣告它；串流由宿主提供。示範 server 的串流照舊讀它自己的 outbox。Node 的宿主這次沒有現成的串流可用，寫成已知缺口。
- **用掛鉤個別把關每一種讀取**：不經引擎的讀取（流程設定原文、已封存內容、搜尋等）由「這個人看不看得到這個範圍」決定，不逐一問掛鉤。
- **掛在別的網址前綴底下**：用戶端假設 API 在 `/api/speclink/v1/`，這次不改。
- **遠端提供技能、關卡狀態上線、自由欄位**：各自有後面的變更。
- **使用者文件**：這個變更不動 README 與 docs。

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `host-kit`: 新增五條——HTTP 入口的範圍、範圍與能力由宿主的函式決定、握手與輪詢與範圍清單、登入方式由宿主宣告、每個引擎命令的線上歸屬；修改一條——「每次呼叫由宿主給身分與範圍」加上選填的顯示用名稱（只用在入口回給用戶端的名稱，不進蓋章）。這個 capability 由變更 host-kit-agent-door 建立，這個變更排在它之後。
- `node-sdk`: 新增一條——`host.handleRequest` 與 `createHost` 新增的三個輸入。
- `client-protocol`: 新增一條——API 根的身分查詢。

## Impact

- Affected specs: host-kit（新增五條、修改一條）、node-sdk（新增一條）、client-protocol（新增一條）
- Affected crates: `speclink-host`、`speclink-server`、`speclink-node`，以及 `speclink-core`（只有 `discuss`：四個討論寫入的拒絕加上型別，訊息與錯誤碼不變，CLI 的行為不變）。`speclink-protocol` 不需要新的路徑常數（用戶端已有 API 根身分查詢的路徑）。`speclink-cli`、`speclink-remote`、桌面 app 不改。
- Affected code:
  - New:
    - crates/host/speclink-host/src/http/mod.rs
    - crates/host/speclink-host/src/http/router.rs
    - crates/host/speclink-host/src/http/routes.rs
    - crates/host/speclink-host/src/http/wire.rs
    - crates/host/speclink-host/src/http/error.rs
    - crates/host/speclink-host/src/http/coverage.rs
    - crates/host/speclink-host/tests/http_door.rs
    - crates/host/speclink-server/src/api/forward.rs
    - crates/adapters/speclink-node/__test__/handle-request.spec.ts
  - Modified:
    - crates/host/speclink-host/Cargo.toml
    - crates/host/speclink-host/src/kit.rs
    - crates/host/speclink-host/src/lib.rs
    - crates/host/speclink-host/src/port_exec.rs
    - crates/host/speclink-host/src/documents.rs
    - crates/host/speclink-host/src/teamstore_port.rs
    - crates/host/speclink-server/src/app.rs
    - crates/host/speclink-server/src/api.rs
    - crates/host/speclink-server/src/api/error.rs
    - crates/host/speclink-server/src/api/routes.rs
    - crates/host/speclink-server/src/state.rs
    - crates/host/speclink-server/src/lib.rs
    - crates/host/speclink-server/src/admin.rs
    - crates/host/speclink-server/src/identity/device.rs
    - crates/host/speclink-server/Cargo.toml
    - crates/host/speclink-server/src/identity/auth.rs
    - crates/host/speclink-server/tests/it/api/read_api.rs
    - crates/host/speclink-server/tests/it/api/health.rs
    - crates/host/speclink-server/tests/it/api/write_guard.rs
    - crates/host/speclink-server/tests/it/identity/auth_pat.rs
    - crates/host/speclink-host/tests/bridge_dual_path.rs
    - crates/engine/speclink-core/src/lifecycle/discuss.rs
    - Cargo.lock
    - crates/adapters/speclink-node/Cargo.toml
    - crates/adapters/speclink-node/src/host.rs
    - crates/adapters/speclink-node/index.js
    - crates/adapters/speclink-node/index.d.ts
    - （`binding.js`、`binding.d.ts` 由 `napi build` 重新產生，被 gitignore，不進 repo）
  - Removed:
    - crates/host/speclink-server/src/api/read_api.rs
    - crates/host/speclink-server/src/api/context.rs
    - crates/host/speclink-server/src/api/verb.rs
- 相依：排在 host-kit-agent-door（宿主套件與掛鉤）、server-routes-on-store-port（不經引擎的讀寫函式）、client-browser-login（登入方式宣告的線上型別）、skills-verbs-first-wave（它在示範 server 新增排序的路由，這裡一起搬）之後。
- 三種形態下的結果：
  - 本機 repo：不適用。本機沒有 Server。
  - 連到遠端的 checkout：可用。連到示範 server 時行為不變；連到內嵌了引擎的宿主時，原版的 CLI 與 Speclink Desktop 第一次能用。已知缺口：Node 的宿主沒有推送事件，用戶端只靠輪詢；匯入在 import-export-on-port 之前不可用；遠端的技能安裝在 remote-skill-delivery 之前仍來自本機的 CLI。
  - 沒有本機規格檔的內嵌宿主：可用。宿主以 `createHost` 建立宿主套件並掛一條路由。它自己的 Agent 工具仍走另一個門（`host.dispatch`），兩個門共用同一個儲存與同一組掛鉤。
- 文件影響（留給最後一個變更 docs-embedded-host，這個變更不動使用者文件）：
  - 「把 Speclink 內嵌進你的系統」：宿主要做的四件事裡的第二、三件（掛一條路由、驗不出身分時傳空值），以及範圍函式、能力開關、登入方式宣告的說明。
  - Node SDK 文件：`host.handleRequest` 與 `createHost` 的新輸入。
  - 專案能力狀態：Node 的宿主沒有推送事件；匯入的狀態。
  - Server 部署文件：示範 server 的定位句。

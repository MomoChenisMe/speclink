# 動詞與旗標契約

**繁體中文** · [English](verb-contract.md)

## <a id="audience"></a>這份文件給誰

要自己寫客戶端、自己架 server，或想用 HTTP 直接接 Speclink 的開發者看這份。只用桌面 app 或 CLI 的話可以跳過，日常操作看[完整 SDD 工作流](workflow.zh-TW.md)就夠。

這份文件回答三件事：

- CLI 的每個動詞在本機模式與遠端模式能不能用、輸出哪裡不同。本機模式指文件存在 repo 裡的 `openspec/`；遠端模式指 `.speclink.yaml` 有 `remote:` 區段、文件存在 server 上。
- 遠端 HTTP API 的共同規則、錯誤碼與全部端點。
- 需要送 body 的端點，請求與回應長什麼樣子。

官方 `speclink-server` 是參考實作，你可以照這份文件自建 server。這份文件與正式規格有出入時，以正式規格為準：

- [動詞契約正式規格](../openspec/specs/verb-contract/spec.md)
- [客戶端協定正式規格](../openspec/specs/client-protocol/spec.md)

架 server、建立存取金鑰（PAT）的步驟見 [Remote 入門](remote-getting-started.zh-TW.md)。

## <a id="mode-assignment"></a>動詞的模式歸屬

除了下表「只限本機」與「只限遠端」兩列，其他動詞兩種模式都能用。

| 歸屬 | 動詞 | 在不支援的模式下執行 |
| --- | --- | --- |
| 只限本機 | `demo`、`trace`、`change rank`、`plan --strict-overlap` | 遠端模式直接拒絕、非零結束，連一個請求都不送（離線也一樣拒絕） |
| 只限遠端 | `claim` | 本機模式直接拒絕、非零結束，stderr 說明需要遠端儲存後端 |
| 兩種模式都可用 | `list`、`show`、`validate`、`analyze`、`drift`、`archive`、`discard`、`artifact`、`language`、`status`、`instructions`、`new`、`workflow-config`、`task`、`in-progress`、`discuss`、`review`、`verify`、`plan`、`change`（`change rank` 除外） | 不適用。遠端模式只讀寫 server，不會偷偷改用本機檔案 |
| 不分模式 | `init`、`update`、`link`、`unlink`、`auth`、`schemas`、`templates`、`feedback`、`schema`、`config`、`completion` | 不適用。這些動詞不碰儲存後端 |

為什麼只限本機：

- `demo` 把示範變更寫進本機的 `openspec/`。
- `trace` 從本機 `openspec/` 的封存變更與討論檔組出溯源鏈。
- `change rank` 寫本機卡片的排序；遠端的看板順序存在 `/board-order`。
- `plan --strict-overlap` 依目錄判斷重疊；server 只依 requirement 判斷。

`claim` 只限遠端，因為「認領」記錄的是團隊裡誰在做這個變更，本機沒有這個狀態。

## <a id="output-differences"></a>兩種模式的輸出差異

兩種模式都可用的動詞，給人看的輸出（含 `--no-color`）應該一字不差。已知的差異只有下表。

| # | 動詞 | 遠端模式的不同 |
| --- | --- | --- |
| 1 | `new change` | 不印 `Path:` 行（server 上的路徑對你沒有意義） |
| 2 | `list` | 不出現 worktree 標示（worktree 屬於本機 checkout） |
| 3 | `status --schema` | 拒絕；schema 由 server 的工作流設定決定 |
| 4 | `workflow-config` | 文件標籤固定是 `config.yaml` |
| 5 | `discuss promote` | 不印 `Path:` 行，也不印它下一行的提示 |
| 6 | `plan --strict-overlap` | 拒絕，原因見上一節 |
| 7 | `list` | 本機預設依最近修改排序；遠端依名稱排序。`--sort created` 與 `--sort modified` 在遠端也是名稱順序 |
| 8 | `validate`（一次驗多個變更） | 本機依最近修改排列結果；遠端依名稱排列。遠端驗變更時一律不嚴格，`--strict` 只對 `--specs` 有效 |
| 9 | `instructions --schema` | 拒絕，原因同第 3 項 |
| 10 | `archive` | 一次只封存一個，而且要寫名字；`--all` 或多個名字會被拒絕。`--skip-specs`、`--no-validate`、`--mark-tasks-complete` 在遠端不生效 |

第 1–6 項是正式規格明列的差異。第 7–10 項是目前程式的實際行為，正式規格還沒列入。

`--json` 的欄位集合與 camelCase 命名是凍結的契約：不改名、不刪除既有欄位，而且工單原文不會出現在任何 `--json` 輸出裡。

## <a id="request-rules"></a>請求的共同規則

### 網址

每個專案的 API 都在這個網址底下：

```text
https://<server>/api/speclink/v1/projects/<專案代號>
```

下文端點表的路徑都接在它後面。CLI 的 `.speclink.yaml` 裡 `remote.url` 存的就是這個網址（由 `speclink link <url>` 寫入）。

### 每個請求都要帶的標頭

| 標頭 | 值 | 缺少或錯誤時 |
| --- | --- | --- |
| `Authorization` | `Bearer <憑證>` | 401 `permission_denied` |
| `X-Speclink-Api-Version` | `1` | 409 `refused`（版本不相容） |
| `X-Speclink-Repo` | 儲存庫代號 | 專案只有一個 repo 時可省略；有多個 repo 卻省略回 409 `refused`；寫了沒註冊的 repo 回 404 `not_found` |
| `Content-Type` | `application/json`（有 body 時） | 415，回應是純文字 |

憑證有兩種：存取金鑰（PAT，`spk_pat_` 開頭，在 server 的帳號頁建立），或 `speclink auth login` 取得的裝置登入憑證（`spk_at_` 開頭）。憑證有效但帳號不是這個專案的成員，回 403。

### 角色

專案成員有 `reader` 與 `editor` 兩種角色。reader 只能讀：所有寫入端點（方法不是 GET 的端點）都要 editor，reader 呼叫會收到 403 `permission_denied`，`message` 為 `your role in project '<代號>' is reader; this action needs the editor role`，server 什麼都不寫。唯一的例外是 `POST /context`：它用 POST 傳送查詢條件，但只讀資料，reader 照常可用。

### 握手

連上之後先呼叫 `GET /binding`。它告訴你目前是誰、綁到哪個專案與 repo、server 版本，以及依角色開放的能力：

```json
{
  "actor": { "id": "usr_…", "name": "Demo" },
  "project": { "id": "prj_acme", "key": "acme", "name": "acme" },
  "repo": { "id": "repo_backend", "key": "backend", "name": "backend" },
  "apiVersion": "1",
  "engineVersion": "0.8.0",
  "capabilities": {
    "contextSnapshots": true,
    "policyWrite": true,
    "validate": true,
    "analyze": true,
    "deleteChange": true,
    "moveTask": true,
    "authentication": [],
    "events": {
      "transports": [{ "type": "sse", "url": "/events", "resume": true }],
      "polling": { "url": "/sync-state", "etag": true }
    }
  }
}
```

- `apiVersion` 與你送出的版本不同時，客戶端應該停下來，不要繼續呼叫其他端點。
- `policyWrite`、`deleteChange`、`moveTask` 只有 editor 是 `true`；`validate`、`analyze` 對所有角色都是 `true`。
- `capabilities` 只是讓畫面停用按鈕的提示。真正的權限檢查發生在每個請求上。

### ETag 與寫入前的版本檢查

ETag 是 HTTP 回應標頭，用來判斷資料有沒有變。

- 成功回應大多帶 `ETag`，值是專案修訂號，例如 `"16"`。專案裡任何一次寫入都會讓它變大。`GET /binding` 與 `GET /events` 不帶 `ETag`。
- 想知道資料有沒有變：呼叫 `GET /sync-state` 並帶 `If-None-Match: <ETag>`，沒變回 304。`POST /context` 也接受 `If-None-Match`。
- 寫入前要附版本的端點只有三個，而且帶法不同：

| 端點 | 版本放在哪裡 | 版本從哪裡來 |
| --- | --- | --- |
| `PUT /changes/{name}/artifacts/{artifact}` | `If-Match: <數字>`，不加引號 | 同一份 artifact 的 `GET` 回應裡的 `version`；`0` 代表「只能新建」 |
| `PUT /board-order` | `If-Match: <ETag>`，引號可有可無 | 任一回應的 `ETag`，例如 `GET /board-order` |
| `PUT /config` | body 的 `expectedRevision` | `GET /config` 回應裡的 `revision` |

版本對不上時回 409 `revision_conflict`，什麼都不寫。前兩個端點缺 `If-Match` 時回 400 `invalid_argument`。其他寫入端點不檢查版本。

## <a id="repo-ownership"></a>變更的 repo 歸屬

每個變更恰好屬於一個 repo；同時牽涉兩個 repo 的需求，要拆成多個變更。

- 遠端模式建立變更（`POST /changes`）時，歸屬取自請求的 `X-Speclink-Repo`。專案只有一個 repo 時自動用那一個。回應的 `repo` 欄位會回報歸屬。
- 列舉變更（`GET /changes`）只回請求那個 repo 的變更，其他 repo 的變更不會出現。
- 一個需求要同時改兩個 repo（例如 backend 與 frontend）時，拆成兩個變更，各自在自己的 repo 建立。契約沒有「跨 repo 的變更」。
- 不只變更：討論、正式規格、封存、工作流設定與看板順序也都分 repo 存放。
- CLI 送出的 repo 名稱來自 `.speclink.yaml` 的 `remote.repo`，用 `speclink link <url> --repo <名稱>` 設定。

## <a id="errors"></a>錯誤封套與錯誤碼

非 2xx 的回應都是同一個 JSON 形狀，叫作錯誤封套：

```json
{ "status": 409, "reason": "refused", "message": "change 'add-login' has started work (started_at set or tasks checked) — discard refuses to delete it; pass --force to discard anyway" }
```

- `reason` 給程式判斷，`message` 給人看（與本機 CLI 印的是同一句）。
- `reason` 只有下表八個值。收到不認得的值，當成一般錯誤、顯示 `message` 即可。
- `DELETE /changes/{name}/in-progress` 被拒時，封套會多帶 `checkedTasks` 與 `touchedFiles` 兩個欄位。
- 例外：請求 body 本身格式錯誤時，回應是純文字而不是封套。例如缺 `Content-Type: application/json` 回 415，缺必填欄位回 422。

| `reason` | HTTP | 什麼時候發生 |
| --- | --- | --- |
| `permission_denied` | 401 | 沒帶憑證，或憑證無效、過期、已撤銷，或帳號已停用 |
| `permission_denied` | 403 | 帳號不是這個專案的成員；reader 呼叫寫入端點 |
| `not_found` | 404 | 專案或 repo 沒有註冊；變更、討論、artifact 或規格不存在 |
| `invalid_argument` | 400 | 參數值不對：slug 格式錯、缺 `If-Match`、`If-Match` 不是數字、未知的 artifact、搜尋關鍵字是空的；或寫入的內容格式不對：`tasks.md` 沒有 `- [ ]` 核取方塊、proposal 缺 `## Why`／`## Problem`／`## Summary`、design 缺 `## Context`、delta spec 沒有操作區段、審查站或驗證站的輪缺 `**Scope**:` 行，或輪的 `**Phase**:` 與工單狀態不符 |
| `invalid_config` | 422 | 變更的 metadata 壞掉；工作流設定文件解析不了 |
| `refused` | 409 | 前置條件不成立：API 版本不符、多 repo 卻沒指定 repo、需要 `force`、任務序號超出範圍、變更已被別人認領、依賴成環、匯入的目標不是空的 |
| `refused` | 413 | 請求 body 超過 32 MiB，或看板順序內容超過 1 MiB |
| `revision_conflict` | 409 | 寫入時附的版本已經不是最新（見[請求的共同規則](#request-rules)） |
| `unavailable` | 503 | 儲存後端暫時無法服務 |
| `internal` | 500 | 其他失敗 |

## <a id="endpoints"></a>端點總表

路徑都接在[專案網址](#request-rules)後面。`{name}` 是變更名稱，`{slug}` 是討論代號，`{artifact}` 是 `proposal`、`design`、`tasks` 或 `specs/<capability>`。「最低角色」寫 `reader` 代表所有成員都能呼叫。

### 連線與同步

| 方法 | 路徑 | 最低角色 | 用途 |
| --- | --- | --- | --- |
| GET | `/binding` | reader | 握手：身分、專案、repo、版本與能力 |
| GET | `/whoami` | reader | 目前使用者，以及這個專案的所有 repo |
| GET | `/sync-state` | reader | 只回 `ETag`；帶 `If-None-Match` 且沒變時回 304 |
| GET | `/events` | reader | SSE 事件流，見下方[事件流](#events) |
| POST | `/context` | reader | 一次取回一致的文件快照；body 可帶 `change` 縮小範圍 |

### 變更

| 方法 | 路徑 | 最低角色 | 用途 |
| --- | --- | --- | --- |
| GET | `/changes` | reader | 列出這個 repo 的變更（`speclink list`） |
| POST | `/changes` | editor | 建立變更（`speclink new change`） |
| GET | `/changes/{name}` | reader | 單一變更的狀態與 metadata（`speclink status`、`show`） |
| DELETE | `/changes/{name}?force=<bool>` | editor | 捨棄變更（`speclink discard`） |
| GET | `/changes/{name}/drift` | reader | 規格那一側的偏移資料（`speclink drift`） |
| GET | `/changes/{name}/validate` | reader | 驗證單一變更（`speclink validate`） |
| GET | `/changes/{name}/analyze` | reader | 交叉分析報告（`speclink analyze`） |
| GET | `/changes/{name}/instructions/{kind}` | reader | 產出指示；`{kind}` 是 `proposal`、`design`、`specs` 或 `tasks`，寫 `apply` 時回實作進度（`speclink instructions`） |
| GET | `/changes/{name}/artifacts/{artifact}` | reader | 讀 artifact 內容與 `version`（`speclink artifact cat`） |
| PUT | `/changes/{name}/artifacts/{artifact}` | editor | 寫 artifact，需帶 `If-Match` |
| GET | `/changes/{name}/evidence` | reader | 完成證據：勾選任務時記下的檔案 |
| POST | `/changes/{name}/in-progress` | editor | 標記開工（`speclink in-progress add`） |
| DELETE | `/changes/{name}/in-progress` | editor | 移除開工標記（`speclink in-progress remove`） |
| POST | `/changes/{name}/claim` | editor | 認領（`speclink claim`） |
| POST | `/changes/{name}/depends` | editor | 宣告或移除前置變更（`speclink change depends`） |
| POST | `/changes/{name}/archive?carryReview=<bool>&carryVerify=<bool>` | editor | 封存（`speclink archive`） |

### 任務

| 方法 | 路徑 | 最低角色 | 用途 |
| --- | --- | --- | --- |
| POST | `/changes/{name}/tasks/{taskId}/done` | editor | 勾選任務（`speclink task done`）；`{taskId}` 是序號或 `tsk_` 開頭的穩定 ID |
| POST | `/changes/{name}/tasks/{taskId}/undone` | editor | 取消勾選（`speclink task undone`） |
| POST | `/changes/{name}/tasks/move` | editor | 搬移任務並重排編號（桌面 app 的拖曳） |

### 品質關卡

`{station}` 是 `review`（審查）或 `verify`（驗證），兩站的端點與 payload 相同。

| 方法 | 路徑 | 最低角色 | 用途 |
| --- | --- | --- | --- |
| GET | `/changes/{name}/{station}` | reader | 讀工單（`speclink review show`） |
| POST | `/changes/{name}/{station}/rounds` | editor | 新增一輪（`speclink review add-round`） |
| POST | `/changes/{name}/{station}/stamp` | editor | 蓋章（`speclink review stamp`） |
| DELETE | `/changes/{name}/{station}` | editor | 丟棄工單（`speclink review discard`） |

### 討論

| 方法 | 路徑 | 最低角色 | 用途 |
| --- | --- | --- | --- |
| GET | `/discussions?archived=<bool>` | reader | 列出討論；`archived=true` 列出封存的討論 |
| POST | `/discussions` | editor | 建立討論（`speclink discuss new`） |
| GET | `/discussions/search?q=<關鍵字>` | reader | 搜尋討論，多個關鍵字以空白分隔（`speclink discuss search`） |
| GET | `/discussions/{slug}` | reader | 讀單一討論與全文（`speclink discuss show`） |
| DELETE | `/discussions/{slug}?force=<bool>` | editor | 刪除討論（`speclink discuss discard`） |
| PUT | `/discussions/{slug}/context` | editor | 寫背景段（`speclink discuss context`） |
| POST | `/discussions/{slug}/rounds` | editor | 新增一輪（`speclink discuss add-round`） |
| POST | `/discussions/{slug}/conclude` | editor | 寫結論（`speclink discuss conclude`） |
| POST | `/discussions/{slug}/archive` | editor | 封存討論（`speclink discuss archive`） |
| POST | `/discussions/{slug}/promote` | editor | 轉為變更（`speclink discuss promote`） |
| POST | `/discussions/{slug}/link` | editor | 把既有變更連到這個討論（`speclink discuss link`） |
| POST | `/discussions/{slug}/seal` | editor | 內容寫好後，把討論標為已轉出變更（`speclink discuss seal`） |

### 專案層資料

| 方法 | 路徑 | 最低角色 | 用途 |
| --- | --- | --- | --- |
| GET | `/plan` | reader | 變更的執行順序（`speclink plan`） |
| GET | `/specs` | reader | 正式規格清單 |
| GET | `/specs/{capability}/document` | reader | 一份正式規格的全文 |
| GET | `/archived` | reader | 已封存變更清單 |
| GET | `/archived/{datedName}/artifacts/{path}` | reader | 已封存變更裡的一份文件，`{path}` 例如 `proposal.md` |
| GET | `/archived/{datedName}/capabilities` | reader | 該次封存動到的 capability 名稱 |
| GET | `/search?q=<文字>` | reader | 桌面 app 的全文搜尋（只搜在途的變更與討論） |
| GET | `/language` | reader | 共用詞彙文件（`speclink language show`） |
| GET | `/config` | reader | 工作流設定原文與修訂號 |
| PUT | `/config` | editor | 寫工作流設定（`speclink workflow-config set`） |
| GET | `/board-order` | reader | 看板順序 |
| PUT | `/board-order` | editor | 整份取代看板順序，需帶 `If-Match` |
| POST | `/import` | editor | 把本機專案整包搬上 server（桌面 app 使用）；目標 repo 必須是空的 |

### 專案網址以外的端點

這幾個端點直接接在 `https://<server>` 後面：

| 方法 | 路徑 | 需要什麼 | 用途 |
| --- | --- | --- | --- |
| GET | `/healthz` | 不需要 | 程序在執行就回 200 |
| GET | `/readyz` | 不需要 | 儲存後端可用才回 200，否則 503 |
| GET | `/auth/whoami` | 只需憑證 | 憑證屬於誰（還沒選專案時用） |
| GET | `/api/speclink/v1/scopes` | 只需憑證 | 這個帳號看得到的專案與 repo |

`/auth/device`、`/auth/device/token`、`/auth/refresh` 與 `/auth/revoke` 是 `speclink auth login` 用的裝置登入流程。

### <a id="events"></a>事件流

`GET /events` 是 SSE（Server-Sent Events：server 透過一條長連線持續推送事件）。

- `invalidate` 事件：某個資源變了。`data` 例如 `{"eventId":"42","scope":"change","resourceId":"add-login","revision":42}`，`scope` 是 `change`、`discussion` 或 `spec`。收到後重新讀取對應資源。
- `reset` 事件：你的續傳位置已被清掉，請重新讀取全部資料。
- 斷線重連時帶 `Last-Event-ID`，可以補收中間漏掉的事件。
- 閒置時 server 會送 heartbeat 註解行，維持連線。

## <a id="payload-examples"></a>請求與回應範例

以下範例的欄位形狀取自官方 server 的實際回應。請求 body 欄寫「無」的端點，送 `{}` 或不送都可以；其他端點一定要送 JSON body，並帶 `Content-Type: application/json`。

### 變更與任務

| 端點 | 請求 body | 成功回應 |
| --- | --- | --- |
| `POST /changes` | `{"name":"add-login","agent":"claude"}` | `{"name":"add-login","schema":"spec-driven","repo":"backend"}` |
| `DELETE /changes/{name}?force=true` | 無 | `{"change":"add-search-bar","unlinkedDiscussions":[{"slug":"board-search-bar","status":"concluded"}]}` |
| `POST /changes/{name}/claim` | 無 | `{"claimedBy":"Demo <demo@example.com>"}` |
| `POST /changes/{name}/depends` | `{"on":["add-login"],"remove":false}` | `{"change":"add-logout","dependsOn":["add-login"]}` |
| `POST /changes/{name}/in-progress` | 無 | `{}` |
| `DELETE /changes/{name}/in-progress` | 無 | `{"removed":true}`；本來就沒開工時是 `{"removed":false}` |
| `POST /changes/{name}/archive` | 無 | `{"specs":[],"datedName":"2026-10-07-fix-typo","snapshotCreated":false,"archivedDiscussions":[],"evidenceRecorded":false}` |
| `POST /changes/{name}/tasks/{taskId}/done` | `{"touchedFiles":["src/login.rs"],"headCommit":"0123…"}` | `{"taskDesc":"1.1 Add the login form","alreadyDone":false}` |
| `POST /changes/{name}/tasks/{taskId}/undone` | 無 | `{"taskDesc":"1.1 Add the login form","alreadyUndone":false}` |
| `POST /changes/{name}/tasks/move` | `{"from":2,"to":1}` | `{"change":"add-login","description":"1.1 Add the session check"}` |

補充：

- `POST /changes` 的選填欄位：`schema`、`description`、`agent`、`fromDiscussion`（來源討論代號）、`last`（搭配 `fromDiscussion`，表示這是結論規劃的最後一個變更）。
- `DELETE /changes/{name}` 的 `force` 預設 `false`。變更已開工（有開工標記或任一任務已勾）時，不帶 `force=true` 會回 409 `refused`，什麼都不刪。metadata 壞掉時，即使 `force=true` 也回 422 `invalid_config`。回應的 `unlinkedDiscussions` 列出被解除連結的來源討論，以及解除後的狀態。
- `POST /changes/{name}/claim`：同一個人重複認領，照樣成功、不重寫；別人已認領時回 409 `refused`，`message` 寫出目前的持有人。
- `POST /changes/{name}/depends`：`on` 必填，`remove` 預設 `false`。自己依賴自己、目標不存在或已封存、依賴成環，都回 409 `refused`。
- `POST /changes/{name}/in-progress`：重複呼叫，或變更名稱不存在，照樣回 200 `{}`，什麼都不寫。
- `DELETE /changes/{name}/in-progress`：還有勾選的任務或記錄過的檔案時回 409 `refused`，封套多帶 `"checkedTasks":1,"touchedFiles":["src/login.rs"]`。
- `POST /changes/{name}/archive`：`specs` 的每一項有 `capability`，以及 `added`、`modified`、`removed`、`renamed` 四個 requirement 數量。`carryReview=true`、`carryVerify=true` 表示即使該站還有未結的工單也封存，工單跟著變更一起搬進封存區。
- `tasks/{taskId}/done`：`touchedFiles` 是這個任務動到的檔案，`headCommit` 是你看到那些檔案時的 commit；兩者都可省略，但 body 至少要送 `{}`。任務已勾時照樣回 200，`alreadyDone` 為 `true`。
- `tasks/move`：`from`、`to` 是從 1 起算的任務序號；選填的 `before` 為 `true` 時插在目標任務之前，`false` 插在之後，省略時依搬移方向判斷。序號超出範圍回 409 `refused`。

### 寫入 artifact

先讀，拿到 `version`：

```text
GET /changes/add-login/artifacts/tasks
→ {"artifact":"tasks","content":"## 1. Login\n\n- [ ] 1.1 Add the login form <!-- speclink-task:tsk_01M4… -->\n…","version":2}
```

再帶著 `version` 寫回：

```text
PUT /changes/add-login/artifacts/tasks
If-Match: 2
{"content":"## 1. Login\n\n- [ ] 1.1 Add the login form\n…"}
→ {"artifact":"tasks","version":<新的版本號>}
```

- 新建一份還不存在的 artifact 時用 `If-Match: 0`。
- 寫入 `tasks.md` 時，server 會在每個任務行尾補上穩定 ID 註解（`<!-- speclink-task:tsk_… -->`）。

### 品質關卡

| 端點 | 請求 body | 成功回應 |
| --- | --- | --- |
| `POST /changes/{name}/review/rounds` | `{"content":"<一輪工單的 Markdown>"}` | `{"round":1}` |
| `POST /changes/{name}/review/stamp` | `{"accept":false,"agent":"claude-code","scope":[{"path":"src/login.rs","hash":"<64 字元>"}],"missing":[]}` | `{"change":"add-login"}` |
| `DELETE /changes/{name}/review` | 無 | `{"change":"add-login"}` |

- 一輪的內容格式與 `speclink review add-round` 從 stdin 收的內容相同，必須有 `**Scope**:` 行。
- `GET /changes/{name}/review` 回 `change`、`rounds`、`lastRound` 與 `content`（工單原文）。每一輪有 `index`、`phase`、`patchHash`、`scope` 與 `findings`（每項 `severity`、`path`、`text`）。
- 蓋章的 `scope` 是工單各輪 Scope 列過的檔案，在你工作目錄裡的內容指紋。`hash` 是檔案內容的 SHA-256（小寫十六進位）；UTF-8 文字檔先把 CRLF 換成 LF 再算。
- 工作目錄裡已經不存在的檔案放進 `missing`。`scope` 加 `missing` 必須剛好等於工單列過的檔案；server 不重算指紋，只檢查檔案集合。
- 蓋章前，變更的寫碼任務必須全部完成。`accept: true` 表示最後一輪還有必須修的發現（CRITICAL／WARNING）也照樣蓋章。

### 討論

| 端點 | 請求 body | 成功回應 |
| --- | --- | --- |
| `POST /discussions` | `{"topic":"看板搜尋列","slug":"board-search-bar"}` | `{"slug":"board-search-bar","topic":"看板搜尋列","path":"discussions/board-search-bar.md"}` |
| `PUT /discussions/{slug}/context` | `{"content":"…"}` | `{}` |
| `POST /discussions/{slug}/rounds` | `{"mode":"interview","content":"…"}` | `{"round":1}` |
| `POST /discussions/{slug}/conclude` | `{"content":"…","hold":true}` | `{"restaleFlagged":[],"held":true}` |
| `POST /discussions/{slug}/promote` | `{"name":"add-search-bar"}` | `{"change":"add-search-bar"}` |
| `POST /discussions/{slug}/link` | `{"change":"add-auth"}` | `{"slug":"auth-scope","change":"add-auth"}` |
| `POST /discussions/{slug}/seal` | `{"change":"add-auth"}` | `{"slug":"auth-scope","change":"add-auth"}` |
| `POST /discussions/{slug}/archive` | 無 | `{"archivedTo":"discussions/archive/<檔名>"}` |
| `DELETE /discussions/{slug}?force=true` | 無 | `{"slug":"board-search-bar"}` |

- `POST /discussions`：`slug` 選填，省略時由 `topic` 推導；必須是小寫英數字以單一連字號分隔，不合格回 400 `invalid_argument`。選填 `kind` 目前只接受 `improve`。
- `conclude`：`hold: true` 表示結論之後記錄保留在途，還欠尚未建立的變更。回應的 `restaleFlagged` 列出因重寫結論而被打回的變更；`autoArchived: true` 只在結論順便封存了記錄時出現。
- `promote`：`name` 選填。`promote` 與 `seal` 都接受選填的 `last: true`，表示這是結論規劃的最後一個變更，會解除記錄的保留旗標。
- `seal`：變更必須先用 `link` 連到這個討論，否則回 409 `refused`。
- `DELETE /discussions/{slug}`：`force` 預設 `false`。已有輪的討論不帶 `force=true` 會回 409 `refused`；封存的討論不能刪，也回 409 `refused`。

### 專案層資料

| 端點 | 請求 body | 成功回應 |
| --- | --- | --- |
| `PUT /config` | `{"content":"schema: spec-driven\n","expectedRevision":16}` | `{"revision":17}` |
| `PUT /board-order`（加 `If-Match: "16"`） | `{"content":"{\"changes\":{\"add-logout\":\"a\"}}"}` | `{"revision":17}` |

- `PUT /config` 收整份設定文件；server 先解析，解析不了回 422 `invalid_config`。
- `POST /context` 的 body 可帶 `change`，只取這個變更的文件；送 `{}` 就取全部。回應有 `snapshotId`、`digest` 與 `documents`，每份文件有 `path`、`content`、`revision` 與 `digest`。CLI 用它把 server 上的文件鏡像到本機的唯讀目錄 `.speclink/context/`。
- 看板順序的 `content` 是一段 JSON 文字，形如 `{"changes":{"<變更>":"<排序鍵>"},"discussions":{…}}`。server 只存不解析；`GET /plan` 讀取時看不懂的內容一律當作沒有排序。

`GET /changes/{name}/validate` 的回應：

```json
{ "change": "add-login", "valid": true, "errors": [], "warnings": ["No delta specs found"] }
```

端點一次只驗一個變更。想一次驗多個，先 `GET /changes` 再逐一呼叫，CLI 的 `validate --all` 就是這樣做。

`GET /plan` 的回應：

```json
{
  "waves": [{ "index": 1, "changes": ["add-login"] }, { "index": 2, "changes": ["add-logout"] }],
  "changes": [
    { "name": "add-login", "wave": 1, "stage": "in-progress", "dependsOn": [], "overlaps": [], "blockedBy": [], "ready": true, "requirementOverlap": [], "archiveAfter": [] },
    { "name": "add-logout", "wave": 2, "stage": "proposed", "dependsOn": ["add-login"], "overlaps": [], "blockedBy": ["add-login"], "ready": false, "requirementOverlap": [], "archiveAfter": [] }
  ],
  "next": null,
  "skipped": []
}
```

依賴成環時回 409 `refused`，`message` 例如 `dependency cycle: add-login -> add-logout -> add-login`。

## <a id="response-fields"></a>清單與詳情的選填欄位

下列選填欄位沒有值時，整個鍵都不出現。客戶端遇到缺席的欄位，不要自己補預設值。

### `GET /changes` 的每一項

必定出現：`name`、`summary`、`status`、`completedTasks`、`totalTasks`。

選填：

| 欄位 | 內容 |
| --- | --- |
| `startedAt` | 開工日期，格式 `YYYY-MM-DD`；沒開工就不出現 |
| `createdBy`、`created` | 建立者與建立日期 |
| `fromDiscussions` | 來源討論代號清單 |
| `claimedBy` | 認領人；沒人認領就不出現 |
| `deltaCapabilities` | 這個變更的 delta 規格動到的 capability |
| `restaleFrom` | 重寫了結論、需要這個變更重新吸收內容的來源討論代號 |
| `metaError` | metadata 壞掉時的原因 |

```json
{ "name": "add-login", "summary": "", "status": "in-progress", "completedTasks": 0, "totalTasks": 2, "startedAt": "2026-10-07", "createdBy": "Demo <demo@example.com>", "created": "2026-10-07" }
```

### `GET /changes/{name}`

必定出現：`changeName`、`schemaName`、`isComplete`、`applyRequires`、`artifacts`（每項 `id`、`outputPath`、`status`，被擋住時多 `missingDeps`）。

八個選填欄位：`created`（只有 metadata 同時記了 schema 與建立日期時才出現）、`fromDiscussions`、`deltaCapabilities`、`createdBy`、`createdWith`、`startedAt`、`startedBy`、`claimedBy`。CLI 遠端模式的 `show` 與桌面 app 的詳情面板都靠這些欄位。

```json
{ "changeName": "add-login", "schemaName": "spec-driven", "isComplete": false, "applyRequires": ["tasks"], "artifacts": [{ "id": "tasks", "outputPath": "tasks.md", "status": "done" }], "created": "2026-10-07", "createdBy": "Demo <demo@example.com>", "createdWith": "claude", "startedAt": "2026-10-07", "startedBy": "Demo <demo@example.com>" }
```

### `GET /discussions` 的每一項

必定出現：`slug`、`topic`、`status`、`rounds`、`created`、`path`、`archived`。官方 server 也一定會填 `concluded`（結論段是否已寫）與 `hold`（是否保留在途）。

選填：`createdBy`、`kind`（改進討論為 `improve`），以及 `promotedTo`——這則討論已轉出的變更名稱，依轉出先後排列。`promotedTo` 直接取自討論記錄，沒有轉出過就不出現。

```json
{ "slug": "board-search-bar", "topic": "看板搜尋列", "status": "promoted", "rounds": 1, "created": "2026-10-07", "createdBy": "Demo <demo@example.com>", "promotedTo": ["add-search-bar"], "concluded": true, "hold": true, "path": "discussions/board-search-bar.md", "archived": false }
```

### `speclink list --json` 的本機 `worktree` 欄位

本機模式的 `list --json` 可能在某個變更上多一個 `worktree` 物件，表示這個變更正在某個 git worktree 裡實作：

```json
{ "completedTasks": 3, "name": "add-dark-mode", "status": "in-progress", "totalTasks": 5, "worktree": { "path": "/path/to/speclink.worktrees/add-dark-mode", "branch": "speclink/add-dark-mode" } }
```

- `path` 是 worktree 目錄的絕對路徑，`branch` 是完整分支名（`speclink/<變更>`）。
- 只在這些條件都成立時出現：本機模式、從主 checkout 執行、工作流設定開啟 `worktree`，而且找得到這個變更對應的 worktree。其他情況整個鍵都不出現。
- 遠端模式的 `list` 永遠不帶這個欄位，因為 server 不知道你本機的 checkout。所以沒有 worktree 時，兩種模式的輸出逐欄一致。
- 欄位出現時，這一項的 `completedTasks`、`totalTasks`、`status` 與 `metaError` 取自 worktree 裡的副本，不是主 checkout 的。

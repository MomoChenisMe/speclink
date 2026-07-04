# Speclink 動詞契約（v1）

本文件是 Speclink 薄 client（remote 模式的 `speclink` CLI，以及任何作為 client 的 host）與內嵌 Speclink 引擎的 server（經 Rust crate 或 Node SDK）之間 REST 契約的**正典參考**。團隊系統實作契約的 server 端；CLI 的 remote 模式是第一個消費者。

> 英文版（`docs/verb-contract.md`）為正典；本文為對照翻譯，歧義時以英文版為準。

> **正典 vs. server 自由度。** 本文定義的狀態轉移語意與錯誤 `reason` 目錄是**契約正典**：所有 server 實作對同一事實必須做出相同裁決——host 不得自行發明變體語意。裁決邏輯的參考實作未來由選用的 `speclink-team` 模組承載（見設計記錄，討論第 17 輪）。server 的實作自由度僅限兩處：
>
> 1. **Gate 政策設定**——*哪些*轉移需要人工核准（零、一或多個 gate；由誰核准）。gated 轉移的*語意*是正典；*政策資料*歸 server。
> 2. **Repos 註冊表管理**——repo 如何註冊到專案，及其管理介面。
>
> 其餘一切——payload 形狀、狀態碼、`reason` 值、守衛條件、擁有權規則——皆由本文釘死。

---

## 1. 共通約定

### 1.1 Base URL 與專案範疇

`.speclink.remote.yaml` 中的連接 URL 是**專案範疇**的：

```
https://team.example.com/api/speclink/v1/projects/{project}
```

本文所有端點路徑皆相對於該 base。專案範疇因此是連接的一部分，永遠不是 query 參數。路徑中的 `v1` 供路由衛生；權威的版本檢查是下方 header。

### 1.2 必要 headers

每個請求皆攜帶：

| Header | 值 | 規則 |
|---|---|---|
| `Authorization` | `Bearer <PAT>` | 由 host 系統簽發的 personal access token。token 永不出現在 URL、query string 或 request body。 |
| `X-Speclink-Api-Version` | `1` | 契約主版本。server 不支援請求的版本——或未收到版本 header——**必須拒絕**（`400 api_version_unsupported`，body 列 `supportedVersions`）。server 永不猜測。 |
| `X-Speclink-Repo` | 註冊的 repo 名 | 呼叫端的 repo 身分（來自連接檔 `repo` 欄位）。專案有多個註冊 repo 時必填；單 repo 專案缺省時 server 解析為唯一註冊 repo。多 repo 專案缺 header → `400 repo_required`。 |

### 1.3 Payload 風格

- 所有 request/response body 皆為 JSON，欄位名 **camelCase**，與 CLI 既有 `--json` 輸出的欄位名對齊（fs 模式 payload 是命名的權威來源）。
- server 可在回應中包含**額外**欄位；client 必須忽略未知欄位（向前相容）。client 只從文件化欄位渲染輸出，因此 CLI stdout 在 fs 與 remote 模式間逐 byte 一致。
- 無快取 fallback：連不上 server 即大聲失敗。沒有離線模式、沒有 outbox、沒有過期的本地答案。

### 1.4 錯誤信封

每個非 2xx 回應（除 server 可能無法產生 body 的傳輸層 5xx 外）攜帶：

```json
{ "reason": "<machine-readable-token>", "message": "<human text>", "...context": "…" }
```

- `reason` 是 client **唯一**分流依據。`message` 僅供參考；client 絕不解析它。
- 409 回應**一律**攜帶 `reason`（硬規則）。
- CLI 把每個非 2xx 翻譯為單行語義化訊息＋建議動作。**裸 HTTP 狀態碼絕不作為錯誤輸出的主體交給使用者或 agent。**

### 1.5 樂觀並行（`If-Match`）

- 讀取 artifact 回其 `version`（單調遞增整數，建立時為 `1`）。
- 寫入 artifact（`PUT`）**必須**攜帶 `If-Match: <version>`——即讀取內容時取得的版本。
  - `If-Match: 0` 表示「建立；artifact 必須尚不存在」。已存在 → `409 version_conflict`（附 `currentVersion`）。
  - 版本過期 → `409 version_conflict`。
  - **不帶** `If-Match` 的 `PUT` 以 `428 if_match_required` 拒絕。沒有強制寫入、沒有「後寫者勝」模式、也沒有任何關閉此檢查的方法。
- 生命週期轉移（claim、archive）不要求 client 送出狀態版本；其原子性是 server 端守衛（conditional update）。`GET /changes/{name}` 對需要的 host（看板渲染、host 端鎖定）暴露 `statusVersion`；CLI 不送它。

---

## 2. Repo 身分與 change 歸屬

**v1 規則：一個 change 恰歸屬一個 repo。**

- **建立**——`POST /changes` 從請求的 `X-Speclink-Repo` 指派 change 的 repo（單 repo 專案：唯一註冊 repo）。request 中沒有選擇其他 repo 的欄位；契約不提供跨 repo 的 change 形狀。
- **列舉**——`GET /changes` 只回傳歸屬請求 repo 的 changes。同專案其他 repo 的 change 不出現。
- **每個 change 範疇的動詞**皆驗證此鏈：PAT 有效 → repo ∈ 專案註冊表 → `change.repo == 請求 repo`。不符即以 `403 repo_mismatch` 大聲失敗，body **同時**指名兩個 repo（`changeRepo`、`requestRepo`）。
- **跨 repo 需求**以拆分為多個 change 處理，每 repo 一個（一場討論可衍生多個 change；血緣由 discussion slug 承載）。

repos 註冊表由 server 管理（server 自由度 #2）。`GET /whoami` 暴露註冊的 repos——含每個 repo 選填的 `gitUrl` 參考值，client 僅用於 fork/鏡像的輔助警告（絕不作為身分依據；URL 推斷已被明確否決）。

---

## 3. Change 生命週期（契約正典）

### 3.1 狀態

change 在線上可見的生命週期為六值之一：

```
drafting → (review) → ready → applying → archived
                                  ↕
                                busy（暫態）
```

| `lifecycle` | 含義 |
|---|---|
| `drafting` | artifacts 撰寫中；尚不可認領。 |
| `review` | 全部必要 artifacts 完成且**proposal gate** 已啟用並待核准。僅存在於 gate 政策啟用它的專案。 |
| `ready` | 完成、gate（若有）已滿足、未被認領。可認領。 |
| `applying` | 已由工程師認領（`claimedBy` 已設）。task 勾選與實作期寫入在此發生。 |
| `busy` | 暫態的 server 端操作持有 change（如 host 驅動的 artifact 合併）。擁有權保留；操作完成前寫入以 `409 change_busy` 拒絕。 |
| `archived` | 終態。delta 已合併入正典規格。 |

`lifecycle` 由**引擎自事實推導**（artifact 完成度、gate 核准、認領狀態）——host 渲染它，不得自行儲存一份獨立發明的狀態。

### 3.2 轉移與守衛

| 轉移 | 觸發 | 守衛（正典） |
|---|---|---|
| *(建立)* → `drafting` | `POST /changes` 或 promote 討論 | 名稱於專案內唯一 → 否則 `409 already_exists` |
| `drafting` → `review` | 最後一個必要 artifact 完成，proposal gate **啟用** | artifact 完成度由引擎推導 |
| `drafting` → `ready` | 最後一個必要 artifact 完成，proposal gate **停用** | — |
| `review` → `ready` | gate 核准（host UI；核准端點屬 host 介面，語意屬正典） | 核准對應當前內容 |
| `review` → `drafting` | gate 退回，**或 `review` 期間任何 artifact 寫入** | 內容變更使待核/已核核准失效——核准僅及於被審閱的內容，絕不靜默延伸到更新後的內容 |
| `ready` → `applying` | `POST /changes/{name}/claim` | **原子** compare-and-set：僅在未認領且 `ready` 時成功。已被他人認領 → `409 ownership_lost`（附 `claimedBy`）；gate 待核 → `409 gate_pending`；未完成／暫態 → `409 change_busy` |
| `applying` → `applying` | `POST .../tasks/{taskId}/done`、artifact `PUT` | 寫入者必須是持有人：`claimedBy == caller`，否則 `409 ownership_lost` |
| `applying` → `ready` | 擁有權釋放／回收（host 治理；v1 無 CLI 動詞） | 前持有人的後續寫入 → `409 ownership_lost` |
| `applying` ↔ `busy` | server 端暫態操作開始／結束 | 全程保留擁有權；`busy` 期間寫入 → `409 change_busy` |
| `applying` → `archived` | `POST /changes/{name}/archive` | **check-all-then-apply**，原子：全部 task 完成（否則 `409 tasks_incomplete`）→ 每個 delta 的 base spec 版本仍為當前（否則 `409 version_conflict` 附 `conflicts[]`）→ archive gate（若啟用）已核准（否則 `409 gate_pending`）→ 單一交易內合併並歸檔。失敗不留部分狀態。 |

備註：

- **Verify** 在 v1 是本地紀律（`/speclink-verify` skill 於 client 端對動詞取回的 artifact 執行）；不是 server 狀態。未來若契約加入 verify gate，將隨 `speclink-team` 參考實作以契約版本升級交付。
- **Ingest** 不是端點。實作中途的需求變更透過 `PUT` 帶 `If-Match` 重寫 artifact——與任何寫入相同的樂觀並行路徑。`busy`/`change_busy` 保留給 UI 執行 server 端合併的 host。
- `archived` 為終態且唯讀。正典：**對已歸檔 change 的任何寫入動詞回 `409 change_busy`，body 帶 `lifecycle: "archived"`**；CLI 回報「此 change 已歸檔」。讀取（`GET` change/artifacts）在 host 保留其可定址性的期間仍有效；host 若將已歸檔 change 移出使用中的命名空間則回 `404 not_found`。

### 3.3 裁決即正典

上表中的每個 409——*哪種*事實情境回*哪個* reason——由本文釘死。給定相同事實，兩個 server 必須回相同的 `reason`。這是 CLI 指引（以及據此行動的 agent）能跨 host 移植的原因。

---

## 4. 錯誤 reason 目錄

完整目錄。client 可能碰到的每條錯誤路徑恰對應一列；不存在未定義的錯誤路徑。

| HTTP | `reason` | 時機 | context 欄位 | CLI 建議動作 |
|---|---|---|---|---|
| 400 | `api_version_unsupported` | 版本 header 缺失或主版本不支援 | `supportedVersions` | 「server 不支援此 CLI 的 API 版本——升級 CLI 或 server。」 |
| 400 | `repo_required` | 多 repo 專案、無 `X-Speclink-Repo` | `availableRepos` | 「請在 `.speclink.remote.yaml` 設定 `repo:`（見 `speclink link`）。」 |
| 400 | `bad_request` | payload 格式錯誤（client bug） | — | 「CLI 內部錯誤——請更新 speclink 或回報。」（大聲失敗，不交 agent 判讀） |
| 401 | `token_missing` | 無 `Authorization` header | — | 「請執行 `speclink auth login`。」（CLI 通常在送出前就本地擋下） |
| 401 | `token_invalid` | token 無效 | — | 「憑證被拒——請執行 `speclink auth login`。」 |
| 401 | `token_expired` | token 過期 | — | 「憑證過期——請執行 `speclink auth login`。」 |
| 401 | `token_revoked` | token 已撤銷 | — | 「憑證已撤銷——請執行 `speclink auth login`。」 |
| 403 | `access_denied` | token 有效但無此專案存取權 | — | 「你的帳號無此專案存取權——請洽專案管理員。」 |
| 403 | `repo_unknown` | `X-Speclink-Repo` 不在專案註冊表 | `availableRepos` | 「repo 未註冊於此專案。可用：<list>。請修正 `repo:` 或重跑 `speclink link`。」 |
| 403 | `repo_mismatch` | change 歸屬另一個 repo | `changeRepo`、`requestRepo` | 「change '<name>' 歸屬 repo '<changeRepo>'；你是 '<requestRepo>'。請於歸屬 repo 執行此動詞。」 |
| 404 | `not_found` | change／artifact／task／discussion／capability／詞彙文件不存在 | `resource`、`name` | 「找不到 '<name>'——請以 `speclink list`（或對應的列舉動詞）確認名稱。」 |
| 409 | `already_exists` | `POST /changes` 或 `POST /discussions` 名稱／slug 已被使用 | `name` | 「名稱已被使用——請換一個。」 |
| 409 | `version_conflict` | artifact 寫入的 `If-Match` 過期；或 archive 時 delta base 落後正典規格 | `currentVersion`——或 archive 時 `conflicts[]`（`{capability, baseVersion, currentVersion}`） | 「內容在你讀取後已被更新——重新讀取（`speclink artifact cat`）後再套用修改。」archive：「規格 <capabilities> 自 propose 後已變動——請於團隊系統調和後重試。」 |
| 409 | `ownership_lost` | claim 已被認領的 change；或非持有人寫入 | `claimedBy` | 「change 由 <claimedBy> 持有——請協調，或於釋放後重新認領。」 |
| 409 | `change_busy` | 對暫態或不合資格狀態的 change 執行動詞（合併中、對 `drafting` 認領、已歸檔） | `lifecycle` | 「change 目前為 <lifecycle>——請等進行中的操作完成後重試。」 |
| 409 | `gate_pending` | gate 核准未決時 claim 或 archive | `gate`（`"proposal"` \| `"archive"`） | 「等待團隊系統中的 <gate> 核准——請洽核准者。」 |
| 409 | `tasks_incomplete` | 尚有未勾 task 時 archive | `remaining` | 「還有 <remaining> 個 task 未完成——先完成（`speclink task done`）再歸檔。」 |
| 409 | `discussion_archived` | 對已歸檔 discussion 執行 `add-round`／`conclude`／`promote` | `slug` | 「discussion '<slug>' 已歸檔——請先於團隊系統還原再繼續。」 |
| 409 | `project_not_empty` | **保留**給 `store push`（未來遷移動詞） | — | 「目標專案已含 changes——push 需要空專案。」 |
| 422 | `validation_failed` | 使用者提供的內容被引擎拒絕（change 名稱不合法、artifact id 不在 schema、artifact 內容驗證失敗） | `errors[]` | 逐條原樣列印驗證錯誤——這是使用者輸入問題，不是 CLI bug。 |
| 428 | `if_match_required` | artifact `PUT` 未帶 `If-Match` | — | 「CLI 內部錯誤——請更新 speclink 或回報。」 |
| 5xx／網路 | *（不保證信封）* | server 崩潰、逾時、拒絕連線 | — | 「server 不可用——請檢查 `.speclink.remote.yaml` 的連接 url（或 `SPECLINK_STORE_URL`）。」**不重試迴圈、不快取 fallback、stdout 保持無資料。** |

### 4.1 範例 body（每個 409 reason 一則）

```json
{ "reason": "already_exists", "message": "change 'add-auth' already exists", "name": "add-auth" }
```

```json
{ "reason": "version_conflict", "message": "design was updated since you read it", "currentVersion": 7 }
```

```json
{ "reason": "version_conflict", "message": "canonical spec moved since propose",
  "conflicts": [ { "capability": "user-auth", "baseVersion": 3, "currentVersion": 5 } ] }
```

```json
{ "reason": "ownership_lost", "message": "change is claimed by chiang", "claimedBy": "chiang" }
```

```json
{ "reason": "change_busy", "message": "change is being updated by the team system", "lifecycle": "busy" }
```

```json
{ "reason": "gate_pending", "message": "archive requires approval", "gate": "archive" }
```

```json
{ "reason": "tasks_incomplete", "message": "3 tasks still open", "remaining": 3 }
```

```json
{ "reason": "discussion_archived", "message": "discussion 'auth-scope' is archived", "slug": "auth-scope" }
```

```json
{ "reason": "project_not_empty", "message": "target project already has changes" }
```

---

## 5. 端點參考

「CLI 動詞」欄為 1:1 對映該端點的 remote 模式指令。

### 5.1 身分與政策 side-car

| 端點 | CLI 動詞 |
|---|---|
| `GET /whoami` | `speclink auth status`（`link`／`init` 驗證亦用） |
| `GET /config` | *（內部——instructions、tdd/audit、locale 的政策 side-car）* |
| `GET /language` | `speclink language show` |

**`GET /whoami` → 200**

```json
{
  "user": { "id": "u_42", "name": "王小明", "handle": "xiaoming" },
  "project": "erp",
  "repos": [
    { "name": "backend",  "gitUrl": "git@github.com:acme/erp-backend.git" },
    { "name": "frontend" }
  ]
}
```

`repos[].gitUrl` 選填——僅供 fork/鏡像輔助警告的參考值。缺省時 client 靜默跳過該檢查。

**`GET /config` → 200**——有效 `WorkflowConfig`（server 端已 resolve；remote 模式下 client 絕不自行合併政策層）：

```json
{ "schema": "spec-driven", "locale": "tw", "specLocale": "tw",
  "tdd": true, "audit": true,
  "context": "…project context…", "rules": { "proposal": ["…"] } }
```

**`GET /language` → 200** `{ "content": "…LANGUAGE 文件…" }`——專案無共用詞彙文件時 404 `not_found`（`resource: "language"`）；CLI 以非 0 exit code 與語義化訊息結束（技能視之為「跳過詞彙載入」）。

### 5.2 正典規格

| 端點 | CLI 動詞 |
|---|---|
| `GET /specs` | `speclink list --specs` |
| `GET /specs/{capability}` | *（技能內的規格讀取）* |

**`GET /specs` → 200** `{ "specs": [ { "id": "user-auth", "path": "specs/user-auth" } ] }`——`path` 為 store 邏輯位置（fs 模式 CLI 印實際目錄；欄位名與形狀一致）。

**`GET /specs/{capability}` → 200** `{ "capability": "user-auth", "content": "…spec.md…", "version": 5 }`

### 5.3 Changes

| 端點 | CLI 動詞 |
|---|---|
| `GET /changes` | `speclink list` |
| `POST /changes` | `speclink new change` |
| `GET /changes/{name}` | `speclink status` |
| `POST /changes/{name}/claim` | `speclink claim` |
| `POST /changes/{name}/archive` | `speclink archive` |

**`GET /changes` → 200**——依請求 repo 過濾。選用 query `?lifecycle=<value>`。

```json
{ "changes": [
  { "name": "add-rate-limit", "summary": "Protect the public API…",
    "status": "in-progress", "completedTasks": 3, "totalTasks": 9,
    "repo": "backend", "lifecycle": "applying", "claimedBy": "chiang" } ] }
```

`name`/`summary`/`status`/`completedTasks`/`totalTasks` 與 fs 模式 `speclink list --json` 欄位完全一致（`status` 維持 task 推導：全勾為 `done`、否則 `in-progress`）；`repo`/`lifecycle`/`claimedBy` 是 remote 模式附加欄位，CLI 的 parity 視圖不印。

**`POST /changes`**——request `{ "name": "add-rate-limit", "schema": "spec-driven", "description": "…", "fromDiscussion": "some-slug" }`（除 `name` 外皆選填；schema 預設取專案 config）→ 201：

```json
{ "name": "add-rate-limit", "schema": "spec-driven", "repo": "backend", "lifecycle": "drafting" }
```

失敗路徑：`409 already_exists`（名稱已用）、`422 validation_failed`（名稱違反引擎命名規則）。

**`GET /changes/{name}` → 200**——fs 模式 `speclink status --json` 報告的超集：

```json
{
  "changeName": "add-rate-limit", "schemaName": "spec-driven",
  "isComplete": false, "applyRequires": ["tasks"],
  "artifacts": [
    { "id": "proposal", "outputPath": "proposal.md", "status": "done",    "version": 3 },
    { "id": "design",   "outputPath": "design.md",   "status": "ready" },
    { "id": "specs",    "outputPath": "specs/**/*.md", "status": "blocked", "missingDeps": ["design"] },
    { "id": "tasks",    "outputPath": "tasks.md",    "status": "blocked", "missingDeps": ["specs"] }
  ],
  "repo": "backend", "lifecycle": "drafting", "statusVersion": 4, "claimedBy": null
}
```

**`POST /changes/{name}/claim`**——空 body → 200 `{ "claimed": true, "claimedBy": "you", "statusVersion": 5 }`。原子：兩個並發 claim——恰一個成功；輸家收 `409 ownership_lost`。

**`POST /changes/{name}/archive`**——空 body → 200：

```json
{ "archived": true, "change": "add-rate-limit",
  "specs": [ { "capability": "api-quota", "version": 6 } ] }
```

失敗路徑：`tasks_incomplete`／`version_conflict`（附 `conflicts[]`）／`gate_pending`／`repo_mismatch`——見 §4。

### 5.4 Artifacts（樂觀並行的讀寫）

| 端點 | CLI 動詞 |
|---|---|
| `GET /changes/{name}/artifacts/{artifact}` | `speclink artifact cat` |
| `PUT /changes/{name}/artifacts/{artifact}` | `speclink new artifact`／技能驅動的 artifact 寫入 |

`{artifact}` ∈ `proposal` \| `design` \| `tasks` \| `specs/{capability}`（delta 規格用巢狀路徑）。

**GET → 200** `{ "artifact": "design", "content": "…", "version": 7 }`——artifact 尚未建立時 404 `not_found`。

**PUT**——request `{ "content": "…完整文件…" }`，header `If-Match: <version>`（`0`＝僅建立）：

- 200 `{ "artifact": "design", "version": 8 }`
- `409 version_conflict`（過期）、`428 if_match_required`（缺 header）、`409 ownership_lost`（change 為 `applying` 且呼叫者非持有人）、`409 change_busy`（暫態／已歸檔）、`422 validation_failed`（artifact id 不在該 change 的 schema，或內容未通過引擎驗證——`errors[]` 逐條列出）。
- 寫入為整份文件替換。沒有 PATCH，server 不為 client 寫入做合併。

### 5.5 Tasks

**`POST /changes/{name}/tasks/{taskId}/done`**——CLI 動詞 `speclink task done`。`{taskId}` 即 instructions 端點 `tasks` payload 所列的序號 id（spec-driven schema 為 1 起算數字的字串）。

Request（選用的歸因）：`{ "touchedFiles": ["src/api/quota.rs"] }`——server 可持久化歸因；client 能算出時就送。

- 200 `{ "change": "add-rate-limit", "taskId": "3", "taskDesc": "…", "status": "done", "alreadyDone": false, "tasksVersion": 12 }`
  - task 已勾選時 `alreadyDone: true` 且 server 端不變；CLI 重現 fs 模式行為（錯誤訊息「Task 3 is already done」、非 0 exit）。
- 守衛：僅持有人（`ownership_lost`）、僅 applying 狀態（`change_busy`）、task 存在（`not_found` 帶 `resource: "task"`）。
- 註：CLI 的 fs-parity `--json` stdout 保留既有鍵名（`change`、`status`、`task_desc`、`task_id`）；從契約 camelCase 回應到該形狀的對映是 CLI 的事。

### 5.6 Instructions（server 計算）

**`GET /changes/{name}/instructions/{artifact}`**——CLI 動詞 `speclink instructions`。`{artifact}` 為 schema artifact id 或字面值 `apply`。

server 以引擎的 instruction builders 對其 store 與專案有效政策計算，回傳與 fs 模式 CLI **相同形狀**的 payload：

- Artifact 形：`changeName`、`artifactId`、`schemaName`、`changeDir`、`outputPath`、`description`、`instruction?`、`context?`、`rules?`、`locale`、`template`、`dependencies[]`、`unlocks[]`。
- Apply 形：`changeName`、`changeDir`、`schemaName`、`contextFiles{}`、`progress{total,complete,remaining}`、`tasks[{id,description,done,parallel}]`、`state`（`blocked`\|`ready`\|`all_done`）、`missingArtifacts?`、`locale`、`instruction?`。

remote 模式下路徑形欄位的值語意：`changeDir` 與 `contextFiles` 的值攜帶 **store 邏輯路徑**（`changes/<name>`、`proposal.md`…）。它們識別文件、不是本地檔案——remote 模式的技能一律以動詞讀文件（`speclink artifact cat`），絕不開這些路徑。fs 專屬的 `preflight` 區塊（本地檔案存在性檢查）在 remote 模式省略。

### 5.7 Discussions

| 端點 | CLI 動詞 |
|---|---|
| `GET /discussions?archived=` | `speclink discuss list [--archived]` |
| `POST /discussions` | `speclink discuss new` |
| `GET /discussions/{slug}` | `speclink discuss show` |
| `PUT /discussions/{slug}/context` | `speclink discuss context` |
| `POST /discussions/{slug}/rounds` | `speclink discuss add-round` |
| `POST /discussions/{slug}/conclude` | `speclink discuss conclude` |
| `POST /discussions/{slug}/archive` | `speclink discuss archive` |
| `POST /discussions/{slug}/promote` | `speclink discuss promote` |

Speclink 的討論是**結構化、append-only 的文件**，寫入介面因此是動詞形（round 只能追加、結論只能 conclude）而非泛用文件 PATCH。文件規則由 server 強制。

- **list → 200** `{ "discussions": [ { "slug": "…", "topic": "…", "status": "open", "rounds": 4, "created": "2026-07-03", "archived": false, "path": "discussions/….md" } ] }`（`path` 為 store 邏輯路徑）。
- **new**——`{ "topic": "…" }` → 201 `{ "slug": "…", "topic": "…", "path": "…" }`；slug 重複 → `409 already_exists`。
- **show → 200** `{ "info": { …list 項目… }, "content": "…完整文件…" }`。
- **context**——`{ "content": "…" }` → 200 `{ "slug": "…", "context": "set" }`（冪等替換 Context 章節）。
- **add-round**——`{ "mode": "assumptions", "content": "…" }` → 200 `{ "slug": "…", "round": 5, "mode": "assumptions" }`（append-only；先前輪次不可變）。
- **conclude**——`{ "content": "…" }` → 200 `{ "slug": "…", "status": "concluded" }`。
- **archive** → 200 `{ "slug": "…", "archivedTo": "discussions/archive/<file>" }`。
- **promote**——`{ "name": "change-name"? }` → 201 `{ "change": "…", "slug": "…", "status": "promoted" }`。建立以結論為種子的 change（依 §2 歸屬請求 repo）；一場討論可 promote 為多個 change。
- 對已歸檔 discussion 執行 `add-round`／`conclude`／`promote` → `409 discussion_archived`。

`speclink discuss discard` **不在 v1**——remote 模式下破壞性刪除討論仍屬 host 治理動作（fs 模式保留本地動詞）。

---

## 6. 涵蓋對照（CLI remote 動詞 ↔ 端點）

所有在 remote 模式運作的 CLI 動詞及其落點：

| CLI 動詞 | 端點 |
|---|---|
| `speclink list`／`list --specs` | `GET /changes`、`GET /specs` |
| `speclink status` | `GET /changes/{name}` |
| `speclink instructions [artifact\|apply]` | `GET /changes/{name}/instructions/{artifact}` |
| `speclink new change` | `POST /changes` |
| `speclink new artifact` | `PUT /changes/{name}/artifacts/{artifact}`（`If-Match: 0`） |
| `speclink task done` | `POST /changes/{name}/tasks/{taskId}/done` |
| `speclink claim` | `POST /changes/{name}/claim` |
| `speclink archive` | `POST /changes/{name}/archive` |
| `speclink artifact cat` | `GET /changes/{name}/artifacts/{artifact}` |
| `speclink language show` | `GET /language` |
| `speclink discuss list/new/show/context/add-round/conclude/archive/promote` | §5.7 |
| `speclink auth status`／`link`／`init --store remote` | `GET /whoami` |
| *（政策 side-car，內部）* | `GET /config` |

remote 模式下刻意**留在 client 端**（無端點；CLI 內嵌引擎，對動詞取回的文件執行）：`speclink analyze`、`speclink drift`、`speclink validate`、`speclink show`。刻意**僅本地**：`init`/`update`/`config`/`completion`/`feedback`/`schemas`/`templates`/`schema`、`speclink discuss discard`、`speclink in-progress`（fs 模式簿記；remote lifecycle 取而代之）。

---

## 7. 與 wadpilot `04-speclink-final-design.md` §5.3 的逐項對照

wadpilot 設計（04）是本契約的證據基礎。本契約與其相異處在此記錄差異與理由。（04 端點以其原始大小寫引用。）

| 04 §5.3 | 本契約 | 差異與理由 |
|---|---|---|
| `POST/GET/DELETE /tokens` | *（契約外）* | PAT 簽發／撤銷是 host 治理 UI。契約只*消費* Bearer token。 |
| `GET /whoami`（401 `token_invalid\|token_expired\|token_revoked`） | `GET /whoami` | **採納**，含三個 401 reason（另加 `token_missing`）。擴充 `repos[]`（＋選填 `gitUrl` 參考值）——repos 註冊表驅動 link 驗證與 fork 警告。 |
| `POST /changes`（JWT） | `POST /changes`（PAT） | 採納。speclink 的 client 一律 PAT；JWT vs PAT 之分屬 host 內部。 |
| `GET /changes?project=&status=` | `GET /changes`（＋`?lifecycle=`） | 專案範疇移入 base path（連接 URL 即專案範疇）。新增依 **repo**（header）過濾——04 無 per-repo 過濾；speclink v1 的一 change 一 repo 規則需要它。`sourceDiscussKey` 移除：可視單號屬 host 呈現層，非契約。 |
| `GET /changes/:id`（＋`artifactVersions`） | `GET /changes/{name}` | change 身分是**名稱**（引擎語彙）而非 DB id——可視單號屬 host 呈現。`artifactVersions{}` map 收攏為 `artifacts[].version`，對齊既有 `speclink status --json` 陣列。 |
| `GET /changes/:id/bundle` | *（無）* | speclink v1 無本地物化、無 outbox、無快取——動詞按需讀文件（`artifact cat`、`instructions`）。04 的動機（超大 payload 撞 Bash 30k 截斷）在逐文件讀取下不發生。聚合端點日後可作為純最佳化補上。 |
| `GET /changes/:id/instructions/:artifact`（apply 形移除，LC-4） | 保留，`{artifact}` ∈ schema ∪ `apply` | speclink 保留 **apply** instruction 形——沒有 bundle sidecar 可攜帶 `locale`／state，且 apply skill 的進入點必須在 fs 與 remote 模式形狀一致。 |
| `POST /changes/:id/analyze` | *（client 端）* | CLI 內嵌引擎，analyze 於本地對動詞取回的 artifact 執行。server 端 analyze 屬 host 選項（04 需要它是因其 client 無引擎），非契約端點。 |
| `POST /changes/:id/approve`／`reject` | *（host 介面）* | gate 核准在 host UI 進行。**語意**（review 狀態、`gate_pending`、內容寫入使核准失效）是正典（§3.2）；端點不入 client 契約——CLI 從不核准。 |
| `POST /changes/:id/claim`（409 `already_claimed\|wrong_status`） | `POST /changes/{name}/claim`（409 `ownership_lost\|change_busy\|gate_pending`） | 原子性照採。reason 更名／拆分使**每個 reason 恰對應一個 CLI 動作**：`already_claimed` → `ownership_lost`（與事後失去擁有權同一建議：協調／重新認領）；`wrong_status` 拆為 `change_busy`（等待）與 `gate_pending`（尋求核准）——正確的使用者動作不同故拆開。 |
| `POST /changes/:id/release` | *（host 介面／未來動詞）* | v1 無釋放認領的 CLI 動詞；host 治理可為之。client 可見的後果由 `ownership_lost` 完整涵蓋。 |
| `PATCH /changes/:id/tasks/:stableId/done`（`{touchedFiles, appliedAgainstVersion}`；409 `version_mismatch` 附最新 bundle／`ownership_lost`／`change_busy`） | `POST /changes/{name}/tasks/{taskId}/done` | `ownership_lost`/`change_busy` 裁決**逐字採納**（正典）。差異：POST（動作動詞端點一律 POST；冪等性由語意承載、非方法）；task id 為 tasks payload 的序號 id（speclink 引擎無 stable-id 註解慣例）；無 `appliedAgainstVersion`／最新 bundle 夾帶——沒有本地快取就沒有要調和的過期勾選檔，task 的版本仲裁收斂為 server 自己的 `tasksVersion`。 |
| `POST .../request-verify`、`POST .../verify-result` | *（無）* | verify 在 speclink v1 是本地紀律（`/speclink-verify` 於 client 端執行）；無 `Verifying` 線上狀態。未來若 verify gate 升入契約，將隨 `speclink-team` 參考實作以版本升級交付。 |
| `POST /changes/:id/ingest` | *（無——`PUT` artifacts）* | 04 的 client 無引擎，合併必須是 server 呼叫。speclink 的 ingest skill 在 client 端跑合併、以 `If-Match` PUT 寫回。host 驅動的合併以 `busy`/`change_busy` 呈現。 |
| `POST /changes/:id/archive`（→ `ArchiveConflict` 狀態、`resolve-conflict` 端點） | `POST /changes/{name}/archive`（409 `version_conflict` ＋ `conflicts[]`） | check-all-then-apply 照採。**無 `ArchiveConflict` 狀態、無 resolve 端點**：archive 失敗時 change 留在 `applying` 並回報衝突的 capabilities；調和（對已變動規格重釘 base）屬 host 端。線上狀態更少、資訊相同。 |
| `POST .../tasks/:stableId/claim`（04 亦後置） | *（無）* | 同樣後置。 |
| `GET /discussions`／`POST /discussions` | 採納 | — |
| `PATCH /discussions/:id`（summary/status） | `PUT …/context`、`POST …/rounds`、`POST …/conclude` | speclink 的討論是引擎強制規則的結構化 append-only 文件（輪次不可變、僅一份現行結論）——泛用 PATCH 表達不了「append-only」，故寫入介面為動詞形。 |
| `DELETE /discussions/:id` | *（v1 無）* | 破壞性刪除仍歸 host 治理；CLI 的 fs 專屬 `discard` 在 v1 不過線。 |
| `POST /discussions/:id/propose`（＋`planRef`） | `POST /discussions/{slug}/promote` | 更名以對齊 CLI 動詞（`promote`）。`planRef` 自 v1 移除：多 change 衍生血緣由每個 promoted change 上的 discussion slug 承載；host 需要時可增量加入 plan-reference 欄位。 |
| `GET/PUT /projects/:projectId/config` | `GET /config`（唯讀） | 專案範疇已在 base path。`PUT` config 屬 host 管理介面——恰為「gate 政策設定」自由度；單號格式 DSL 欄位屬 host 擴充，永不過此契約。 |
| `GET /specs?project=`、`GET /specs/search` | `GET /specs`、`GET /specs/{capability}` | 搜尋屬 host UI 關注點（無 CLI 動詞需要）；新增依 capability 直讀——技能經動詞讀正典規格。 |
| `GET /projects?repo=`（git remote 反查） | *（無）* | 原則性否決：git remote URL 推斷不可靠（fork／鏡像——討論第 15 輪）。綁定由連接檔宣告、經 `whoami.repos[]` 驗證；git URL 僅作輔助警告素材。 |
| `GET /changes/:id/version`（快取過期檢查） | *（無）* | 無本地快取 → 無物可檢。`statusVersion` 仍在 `GET /changes/{name}` 供 host 使用。 |
| `GET /version`（啟動時比對 major） | `X-Speclink-Api-Version` header | 以逐請求協商取代啟動時檢查：無狀態、無競態（不會「啟動時查過、session 中 server 升級」）、對每種 host 都成立且省一次往返。server 拒絕而非警告。 |
| 錯誤信封 `{reason, message, ...context}`；「409 一律帶 `reason`」；「CLI 是狀態碼唯一判讀者」 | 逐字採納 | 這三條 04 規則在此為契約正典（§1.4、§4）。 |
| 409 `version_mismatch` | `version_conflict` | 更名：單一 reason 現涵蓋所有版本過期情境（artifact 寫入**與** archive 規格合併），且 `_conflict` 字尾與 HTTP 409 狀態名一致。 |
| outbox ＋ 離線佇列（§7.6） | *（無）* | speclink v1 明確否決（討論第 3 輪）：連線失敗即大聲失敗、絕不佇列寫入——無快取、無 outbox、無分岔的本地真相。 |

---

## 8. 契約本身的版本管理

- 本文定義**主版本 1**（`X-Speclink-Api-Version: 1`）。
- 增量變更（新端點、新選填欄位、*新*錯誤路徑上的新 `reason` 值）不升主版本。client 忽略未知欄位；已知路徑上遇未知 `reason` 走通用 fallback（「非預期的 server 回應——請更新 speclink」）。
- 對既有守衛語意、狀態推導、`reason` 裁決或欄位含義的任何變更皆為**主版本**升級——server 可同時支援多個主版本，依請求 header 選擇。

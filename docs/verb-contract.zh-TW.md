# 動詞與旗標契約

**繁體中文** · [English](verb-contract.md)

## <a id="audience"></a>這份文件給誰

要自己寫用戶端、把 Speclink 內嵌進自己的系統，或想用 HTTP 直接接 Speclink 的開發者看這份。只用桌面 app 或 CLI 的話可以跳過，日常操作看[完整 SDD 工作流](workflow.zh-TW.md)就夠。

這份文件回答四件事：

- CLI 的每個動詞在本機專案與遠端專案能不能用、輸出哪裡不同。本機專案的文件存在 repo 裡的 `speclink/`（或舊版建立的 `openspec/`）。遠端專案的文件由一台 Server 持有，`.speclink.yaml` 有 `remote:` 區段。下文的「本機模式」「遠端模式」指 CLI 在這兩種專案裡的行為。
- 落地回報、自由欄位與 evidence 這幾個動詞怎麼用，`--json` 多了哪些欄位。
- 遠端 HTTP API 的共同規則、錯誤碼與全部端點。
- 需要送 body 的端點，請求與回應長什麼樣子。

這些端點由宿主套件的 HTTP 入口提供。內嵌了引擎的宿主把請求交給入口，就有同一組端點。官方的示範用 server（`speclink-server`）用的是同一套宿主套件，所以 CLI 與 Speclink Desktop 對兩者的用法相同。怎麼把入口接進自己的後端，見 [Node SDK 的「服務 CLI 與 Speclink Desktop」](sdk-node.zh-TW.md#http-entry)與 README 的[把 Speclink 內嵌進你的系統](../README.zh-TW.md#embedding)。

這份文件與正式規格有出入時，以正式規格為準：

- [動詞契約正式規格](../speclink/specs/verb-contract/spec.md)
- [用戶端協定正式規格](../speclink/specs/client-protocol/spec.md)

啟動示範用 server、建立存取金鑰（PAT）的步驟見 [Remote 入門](remote-getting-started.zh-TW.md)。

## <a id="mode-assignment"></a>動詞的模式歸屬

除了下表「只限本機」與「只限遠端」兩列，其他動詞在本機專案與遠端專案都能用。

| 歸屬 | 動詞 | 在不支援的模式下執行 |
| --- | --- | --- |
| 只限本機 | `demo` | 遠端模式直接拒絕、非零結束，連一個請求都不送（離線也一樣拒絕） |
| 只限遠端 | `claim`（含 `--release`） | 本機模式直接拒絕、非零結束，stderr 說明這個動詞需要遠端專案 |
| 兩種模式都可用 | `list`、`show`、`validate`、`analyze`、`drift`、`trace`、`plan`（含 `--strict-overlap`）、`change`（`depends`、`schema`、`rank`、`landed`、`field`）、`archive`、`discard`、`artifact`、`evidence`、`language`、`status`、`instructions`、`new`、`workflow-config`、`task`、`in-progress`、`discuss`、`review`、`verify` | 不適用。遠端模式只讀寫 Server，不會偷偷改用本機檔案 |
| 不分模式 | `init`、`update`、`link`、`unlink`、`auth`、`schemas`、`templates`、`feedback`、`schema`、`config`、`completion` | 不適用。這些動詞不讀寫規格文件 |

`demo` 只限本機，因為它把示範變更寫進本機的規格資料夾。`claim` 只限遠端，因為「認領」記錄的是團隊裡誰在做這個變更，本機專案沒有這個狀態。

`trace`、`change rank` 與 `plan --strict-overlap` 在遠端專案由 Server 執行，輸出和本機相同：

- `trace`：Server 用同一份引擎組出溯源鏈（`GET /trace/{capability}`）。
- `change rank`：順序寫進 Server 的看板順序（`POST /changes/{name}/rank`）。
- `plan --strict-overlap`：Server 算出嚴格重疊的計畫（`GET /plan?strictOverlap=true`）。

連到還沒有這些路由的舊版 Server 時，三者都以非零結束。`trace` 與 `change rank` 得到 404。`plan --strict-overlap` 的回應少了 `strictOverlap` 標記，CLI 會說明這台 Server 不支援，不會把一般的計畫當成嚴格重疊的計畫印出來。

有四個子指令要讀你電腦上的 git 與工作目錄：`drift`、`review prepare`、`review scope`、`verify scope`。用 CLI 在遠端專案的 checkout 裡執行，它們照常可用。沒有 checkout 的宿主經呼叫入口（`host.dispatch`）執行它們會被拒絕。帶 `--skill` 的 `instructions` 印的是 CLI 內建的技能文字，經呼叫入口同樣被拒絕，所以被拒絕的一共五個。見專案能力狀態的[已知缺口](product-status.zh-TW.md#known-gaps)。

## <a id="output-differences"></a>兩種模式的輸出差異

兩種模式都可用的動詞，給人看的輸出（含 `--no-color`）應該一字不差。已知的差異只有下表。

| # | 動詞 | 遠端模式的不同 |
| --- | --- | --- |
| 1 | `new change` | 不印 `Path:` 行（Server 上的路徑對你沒有意義） |
| 2 | `list` | 不出現 worktree 標示（worktree 屬於本機 checkout） |
| 3 | `status --schema` | 拒絕；schema 由 Server 的工作流設定決定 |
| 4 | `workflow-config` | 文件標籤固定是 `config.yaml` |
| 5 | `discuss promote` | 不印 `Path:` 行，也不印它下一行的提示 |
| 6 | `list` | 本機預設依最近修改排序；遠端依名稱排序。`--sort created` 與 `--sort modified` 在遠端也是名稱順序 |
| 7 | `validate`（一次驗多個變更） | 本機依最近修改排列結果；遠端依名稱排列。遠端驗變更時一律不嚴格，`--strict` 只對 `--specs` 有效 |
| 8 | `instructions --schema` | 拒絕，原因同第 3 項 |
| 9 | `archive` | 一次只封存一個，而且要寫名字；`--all` 或多個名字會被拒絕。`--skip-specs`、`--no-validate`、`--mark-tasks-complete` 在遠端不生效 |

第 1–5 項是正式規格明列的差異。第 6–9 項是目前程式的實際行為，正式規格還沒列入。

`--json` 的欄位集合與欄位名稱是凍結的契約：不改名、不刪除既有欄位。欄位名稱大多是 camelCase；少數在這條規則之前就有的欄位是 snake_case，[下表](#snake-case-fields)列出全部。工單原文不會出現在不帶 `--detail` 的任何 `--json` 輸出裡；只有 `review show --json --detail` 與 `verify show --json --detail` 以 `content` 帶出它（見[工單全文](#ticket-detail)）。新欄位只會以選填的方式加入，名稱是 camelCase，見[下一節](#verb-reference)。

### <a id="snake-case-fields"></a>snake_case 的欄位

下列 `--json` 欄位的名稱是 snake_case。它們和 camelCase 的欄位一樣屬於凍結的契約：名稱維持不變，不會改成 camelCase，也不會消失。其他動詞沒有 snake_case 的欄位。

| 動詞 | snake_case 的欄位 |
| --- | --- |
| `task done`、`task undone` | `task_id`、`task_desc` |
| `discuss archive` | `archived_to` |
| `analyze` | `change_id`、`artifacts_analyzed`、`artifacts_missing`、`finding_count`，以及每一條發現的 `summary_msg`、`recommendation_msg` |
| `drift` | `change_id`、`last_commit`、`commits_since_created`、`broken_anchors`、`tasks_maybe_resolved`、`tasks_blocked_external`、`spec_assumptions`、`total_score`、`primary_recommendation`、`contributes_to_total` |

## <a id="verb-reference"></a>動詞、旗標與 `--json` 欄位

這一節列出認領、共用詞彙文件的寫入、討論搬回、落地回報、自由欄位與 evidence 的動詞、`auth login` 的登入方式旗標，以及 `list --json` 與 `show --json` 的選填欄位。除了 `auth login` 與 `claim`（它們只用在遠端專案），這些動詞與旗標在本機專案與遠端專案都能用，輸出相同。每個動詞都接受 `--no-color`。

### `speclink evidence show <變更>`

印出這個變更每個已完成任務動到的檔案。這份記錄由 `speclink task done` 寫入，`evidence show` 只讀不寫。

- `<變更>` 先找在途的變更；沒有時找已封存的變更。已封存的變更可以寫帶日期的封存名稱，也可以只寫變更名稱（取最新封存的那一筆）。
- 旗標：`--json`。
- 變更存在但沒有任何記錄：印一行說明，exit code 是 0。名稱找不到：非零結束。
- `--json` 的欄位：

| 欄位 | 內容 |
| --- | --- |
| `change` | 變更名稱 |
| `archivedAs` | 帶日期的封存名稱；只有記錄來自已封存的變更時才出現 |
| `entries` | 每筆一個任務：`taskId`、`taskDesc`、`touchedFiles`、`recordedAt`，以及有值才出現的 `actor`、`repo`、`headCommit` |
| `files` | 全部任務動到的檔案，去重複並排序 |

### `speclink claim <變更>`

記下「這個變更我在做」，別人就不會重複開工。只有遠端專案有認領；本機專案會拒絕這個動詞，帶不帶 `--release` 都一樣。

- 旗標：`--release`、`--json`。
- 不帶 `--release` 是認領。同一個人重複認領照樣成功、不重寫；別人已經認領時以非零結束，stderr 寫出持有人。
- `--release` 是釋放認領。釋放之後，任何人都可以再認領。
- **有寫入權限的人都可以釋放，不必是認領的人。** 引擎不管誰可以釋放。「只有認領的人可以釋放」這類規則由宿主自己加，寫在執行前掛鉤（`beforeCommand`）裡，見 [Node SDK 的範例](sdk-node.zh-TW.md#release-hook)。
- 釋放成功時輸出 `✓ Released claim on change: <變更> (was claimed by <持有人>)`。
- 對沒有被認領的變更執行 `--release` 不是錯誤：輸出 `Change '<變更>' is not claimed — nothing to release`，什麼都不寫，exit code 是 0。
- `claim --json` 輸出 `{"change":"add-login","claimedBy":"Demo <demo@example.com>"}`。
- `claim --release --json` 輸出 `{"change":"add-login","released":true,"previousHolder":"Demo <demo@example.com>"}`。變更本來就沒有被認領時，`released` 是 `false`。`previousHolder` 只在釋放前的認領有寫持有人時出現。
- 認領被拒絕、變更不存在、在本機專案執行，都以非零結束。訊息寫到 stderr，stdout 沒有 JSON。
- 在 Node SDK，`dispatch(['claim', name])` 現在回傳 `--json` 的物件，和其他有 JSON 形式的動詞一樣。`claim` 有 JSON 形式之前，它回傳 `{ output }`；讀 `output` 的宿主要改讀 `claimedBy`。

掛鉤收到的釋放，命令名稱是 `claim-release`，變更名稱在 `params.name`。這個名稱不是 `claim`：只擋 `claim` 的掛鉤擋不到釋放。只讓認領的人與管理者釋放的掛鉤，見 [Node SDK](sdk-node.zh-TW.md#release-hook)。

要知道誰認領了一個變更，讀 `speclink show <變更> --json` 的 `claimed`，見[下面的欄位說明](#verb-reference)。

### `speclink language replace --stdin`

把 stdin 的內容整份寫成專案的共用詞彙文件（`LANGUAGE.md`）。文件不存在時建立，存在時整份取代，不做合併。

- 旗標：`--stdin`（必填）、`--json`。沒有 `--stdin` 是用法錯誤。
- 要改一個詞條：先用 `speclink language show` 讀出全文，修改後整份寫回。讀出、修改、寫回之間不要隔著其他步驟。
- 取代成功時輸出 `✓ Replaced the LANGUAGE document`。這次呼叫建立了原本不存在的文件時，輸出 `✓ Created the LANGUAGE document`。
- `--json` 輸出 `{"created":true}`。`created` 表示這次呼叫是否建立了原本不存在的文件。
- 內容是空的或只有空白時拒絕，以非零結束，文件不變。訊息寫到 stderr，stdout 沒有輸出。
- 有寫入權限就能用，不需要改工作流政策的權限。
- 掛鉤收到的命令名稱是 `language-replace`。`params` 只有內文的位元組數（`bytes`），沒有內文。成功後的事件是 `language-replaced`。
- 連到還沒有這個端點的舊版 Server 時以非零結束，訊息說明這台 Server 不支援這個動詞。

### `speclink discuss restore <slug>`

把已封存的討論記錄搬回進行中。用在討論封存錯了的時候，例如最後一刀的旗標下錯，記錄跟著變更一起被封存。

- 旗標：`--json`。
- 同一個 slug 有多份封存時，搬回最新的一份，也就是 `speclink discuss show <slug>` 讀到的那一份。其他的留在封存。記錄的內容不變。
- 成功時輸出兩行：`✓ Restored discussion: <slug>` 與 `  From: <封存時的檔名>`。
- `--json` 輸出 `{"slug":"board-search-bar","restoredFrom":"2026-11-10-board-search-bar.md"}`。
- 同一個 slug 已經有進行中的記錄時拒絕；沒有封存的記錄時回報找不到。兩種情況都以非零結束，什麼都不寫。
- 搬回不改記錄的內容，保留旗標也不會補回來。搬回之後接著做原本被擋住的那一步：再轉出一個變更，或繼續討論。
- 繼續討論、轉出變更、`discuss link` 與 `discuss seal` 遇到已封存的討論時，錯誤訊息會指向這個動詞。
- 掛鉤收到的命令名稱是 `discuss-restore`，slug 在 `params.slug`。成功後的事件是 `discussion-restored`。
- 連到還沒有這個端點的舊版 Server 時以非零結束，訊息說明這台 Server 不支援這個動詞。

### `speclink change landed <變更>`

回報這個變更的程式碼落在哪個 commit。

- 旗標：`--commit <sha>`、`--branch <名稱>`、`--json`。
- 不帶 `--commit` 時，CLI 用目前 checkout 的 HEAD。不在 git checkout 裡又沒帶 `--commit`，就以非零結束。
- `--branch` 會取代變更上記錄的分支；不帶就不動它。
- **它只記錄，不檢查。** 引擎存下 commit、分支（有給時）、回報的時間與回報的人，不讀 git，也不查這個 commit 存不存在、在不在主分支。「沒有落地就不能封存」這類規則由宿主自己加，寫在執行前掛鉤（`beforeCommand`）裡。
- `--commit` 只看形狀：7 到 64 個十六進位字元。分支名稱是 1 到 255 個字元，不含空白與控制字元。形狀不對就拒絕，什麼都不寫。
- 重複回報會覆寫上一次的記錄。變更還在提案中也可以回報，階段不會因此改變。
- `--json` 輸出 `{"change":"add-login","commit":"<sha>","branch":"speclink/add-login"}`；`branch` 只在這次有帶 `--branch` 時出現。

`commit` 與 `worktree-merge` 兩支技能會在提交或合併之後替你執行它，見[工作流](workflow.zh-TW.md#commit)。

### 自由欄位：`change field` 與 `discuss field`

自由欄位是你自己的鍵值資料，例如對應的工單編號。Speclink 只存不讀：沒有任何動詞會依欄位改變行為。不要把密碼或金鑰放進去。

| 動詞 | 做什麼 |
| --- | --- |
| `speclink change field set <變更> <鍵>=<值>...` | 設定變更的一個或多個欄位 |
| `speclink change field unset <變更> <鍵>...` | 移除變更的欄位 |
| `speclink discuss field set <討論代號> <鍵>=<值>...` | 設定討論的一個或多個欄位 |
| `speclink discuss field unset <討論代號> <鍵>...` | 移除討論的欄位 |

- 四個動詞都接受 `--json`。
- `set` 的值是第一個 `=` 之後的全部文字。一次給多組時，全部寫入或全部不寫。
- `unset` 一個不存在的鍵不算錯誤。
- 鍵：小寫英文字母開頭，之後是小寫字母、數字、`_`、`.`、`-`，最長 64 個字元。
- 值：單行文字，最長 1024 個位元組，不含控制字元。值一律是字串。
- 每個變更或討論最多 32 個鍵。
- 變更或討論不存在、鍵或值不合規則、超過 32 個鍵：非零結束，什麼都不寫。已封存的變更與討論不能改欄位。
- `--json` 輸出寫入後的全部欄位：`{"change":"add-login","fields":{"ticket":"PROJ-1"}}`；討論的鍵名是 `discussion`。

建立時就帶上欄位，用 `--field <鍵>=<值>`（可以重複）：

```bash
speclink new change add-login --field ticket=PROJ-1 --field source=portal
speclink discuss new "看板搜尋列" --field owner=alice
```

欄位和建立在同一次寫入。連到還不認得自由欄位的舊版 Server 時，建立照樣成功、欄位被忽略，CLI 會印一行警告。

### `auth login` 的登入方式旗標

`speclink auth login` 不帶旗標時，先問 Server 支援哪些登入方式（`GET /api/speclink/v1/auth-methods`），再依固定順序選第一個：瀏覽器登入、裝置登入、存取金鑰。Server 沒有宣告時，照舊先試裝置登入。四個旗標用來指定方式：

| 旗標 | 做什麼 |
| --- | --- |
| `--browser` | 用 Server 宣告的登入服務，在瀏覽器登入。Server 沒有宣告瀏覽器登入時非零結束 |
| `--device` | 用一組代碼授權這台裝置 |
| `--pat` | 貼上一把存取金鑰 |
| `--token-stdin` | 從 stdin 讀存取金鑰，給 CI 與腳本用 |

登入的完整步驟見 [Remote 入門](remote-getting-started.zh-TW.md#cli)。

### `list --json` 與 `show --json` 新增的欄位

下列欄位都是選填的：沒有值時整個鍵不出現，所以沒有用到這些功能的專案，輸出和以前逐位元相同。

`speclink list --json` 的每個變更一定有 `completedTasks`、`name`、`status` 與 `totalTasks`；摘要不是空字串時才有 `summary`。每個變更新增的欄位：

| 欄位 | 內容 |
| --- | --- |
| `deltaCapabilities` | 這個變更的 delta spec 的 capability 名稱，依名稱遞增。變更有 delta spec 時才出現。舊版 Server 不送這個欄位時，鍵不出現，指令不失敗 |
| `created` | 建立日期，格式 `YYYY-MM-DD` |
| `modifiedAt` | 這個變更底下的文件最後一次被修改的時間，RFC 3339 的 UTC 時間。儲存答不出來時不出現。本機專案的值取自檔案時間，checkout 或複製也會改到它，只適合拿來估「閒置多久」 |
| `fields` | 自由欄位 |

不帶 `--detail` 的 `list --json` 沒有品質關卡的欄位，遠端專案也一樣。

#### <a id="list-detail"></a>`speclink list --json --detail`

`--detail` 讓每個變更多帶出它記下的狀態：兩個品質關卡、認領、分支、落地、建立的人與來源討論。這樣一次呼叫就能回答「哪些變更已經通過審查」「我認領了哪些」。

- `--detail` 只能和 `--json` 一起用，也不能和 `--specs` 一起用。`speclink list --detail` 與 `speclink list --specs --json --detail` 會以用法錯誤結束：exit code 不是 0、訊息在 stderr、stdout 沒有輸出。經呼叫入口（`host.dispatch`）執行時，錯誤代碼是 `invalid_argv`。
- 不帶 `--detail` 時，`list --json` 的輸出和以前逐位元相同。
- `--detail` 不改變原有的欄位、項目的順序與 `specs` 區段。

下面每個鍵都只在變更有那一筆記錄時出現：

| 欄位 | 什麼時候出現 | 內容 |
| --- | --- | --- |
| `review`、`verify` | 這個關卡有未結的工單，或已經蓋章 | 和下面 `show --json` 的同名物件相同。進行中的關卡（`inProgress`）另有 `rounds`：工單目前的輪數 |
| `claimed` | 有人認領了這個變更 | 和下面 `show --json` 的同名物件相同：`by`，以及認領時有記下才出現的 `byId` 與 `at` |
| `branch` | 開工或落地回報時記下了分支 | 分支名稱 |
| `landed` | 有落地回報 | 只有 `commit`。要看回報時間與回報的人，請用 `show --json` |
| `createdBy` | metadata 記下了建立的人 | 建立的人，照 metadata 記的原樣 |
| `fromDiscussions` | 這個變更由討論轉出 | 來源討論的 slug，順序和 `show --json` 的 `fromDiscussions` 相同 |

```json
{
  "changes": [
    {
      "completedTasks": 5,
      "name": "add-login",
      "status": "done",
      "totalTasks": 5,
      "created": "2026-11-01",
      "review": { "state": "stamped", "stampedAt": "2026-11-02T14:03:44+08:00", "rounds": 2, "tasksTotal": 5, "commit": "0123456789ab" },
      "verify": { "state": "inProgress", "rounds": 1 },
      "claimed": { "by": "Demo <demo@example.com>", "byId": "u_42", "at": "2026-11-18T10:00:00+08:00" },
      "branch": "speclink/add-login",
      "landed": { "commit": "0123456789ab" },
      "createdBy": "Demo <demo@example.com>",
      "fromDiscussions": ["login-flow"]
    },
    { "completedTasks": 0, "name": "add-search", "status": "in-progress", "totalTasks": 3, "created": "2026-11-10" }
  ]
}
```

- 同樣的資料在本機專案、遠端專案與 `host.dispatch` 會得到相同的鍵與值。本機專案沒有認領，所以不會出現 `claimed`。
- 在遠端專案，CLI 用一次 `GET /changes` 的回應組出這些鍵，不會對每個變更另外發請求。
- 鍵不出現，代表這個變更沒有那一筆記錄。舊版 Server 不提供其中一些欄位，這時對應的鍵同樣不出現，指令不會失敗。請不要把缺少 `review` 或 `verify` 讀成「這個關卡沒跑過」。
- metadata 損壞的變更帶 `metaError`，這些鍵一個都沒有。
- 清單不判斷蓋章之後檔案有沒有再改過。`review` 與 `verify` 只反映記下的狀態，和 `show --json` 相同。
- `claimed.byId` 是宿主給這個人的識別。讀得到這個專案的人都看得到它。
- 工單解析不了的關卡沒有 `rounds`，清單照常成功。不送 `rounds` 的舊版 Server 給的是 `{ "state": "inProgress" }`。
- 經 `host.dispatch` 執行時，`--detail` 會讀未結的工單來數輪數：全部合成一次儲存讀取，沒有未結工單時不多讀。

#### <a id="list-archived"></a>`speclink list --archived --json`

`--archived` 改成列出已封存的變更，最新的在前。顯示封存區的宿主畫面因此能用動詞讀到它。

- `--archived` 只能和 `--json` 一起用，也不能和 `--specs`、`--changes`、`--detail` 一起用。`speclink list --archived` 與例如 `speclink list --archived --json --detail` 會以用法錯誤結束：exit code 不是 0、訊息在 stderr、stdout 沒有輸出。經 `host.dispatch` 執行時，錯誤代碼是 `invalid_argv`。
- 輸出是 `{ "archived": [ … ] }`，沒有 `changes` 也沒有 `specs`。沒有任何已封存的變更時是 `{ "archived": [] }`。
- 項目依 `datedName` 由大到小排列，`--sort` 不影響這個順序。
- 不帶 `--archived` 時，`list` 與 `list --json` 的輸出和以前逐位元相同。

| 欄位 | 什麼時候出現 | 內容 |
| --- | --- | --- |
| `datedName` | 一定出現 | 封存後帶日期的名稱，例如 `2026-11-22-add-two` |
| `date` | 一定出現 | `datedName` 開頭的日期，格式 `YYYY-MM-DD`；名稱沒有日期時是空字串 |
| `name` | 一定出現 | 去掉日期的名稱；名稱沒有日期時等於整個 `datedName` |
| `reviewOutcome`、`verifyOutcome` | 一定出現 | `none`、`stamped` 或 `notPassed`，和 [`GET /archived`](#archived-items) 相同 |
| `tasksTotal`、`tasksDone` | 封存的文件有任務清單時，兩個都出現 | 任務總數與已勾選的任務數 |
| `createdBy` | metadata 記下了建立的人 | 建立的人 |
| `fromDiscussions` | 這個變更由討論轉出 | 來源討論的 slug |
| `deltaCapabilities` | 封存的文件裡有 delta spec | capability 名稱，依名稱遞增 |
| `fields` | 這個變更有自由欄位 | 自由欄位 |

```json
{
  "archived": [
    {
      "createdBy": "Ann <ann@example.com>",
      "date": "2026-11-22",
      "datedName": "2026-11-22-add-two",
      "deltaCapabilities": ["auth"],
      "fromDiscussions": ["two"],
      "name": "add-two",
      "reviewOutcome": "stamped",
      "tasksDone": 3,
      "tasksTotal": 3,
      "verifyOutcome": "stamped"
    },
    { "date": "2026-11-10", "datedName": "2026-11-10-old-one", "name": "old-one", "reviewOutcome": "none", "verifyOutcome": "none" }
  ]
}
```

- metadata 解析不了的已封存變更照常列出：沒有 `createdBy`、`fromDiscussions`、`fields`，兩個結局以「沒有章」判定。
- 要讀已封存變更的文件，用 `speclink artifact cat <文件> --change <datedName>`，例如 `speclink artifact cat specs/auth --change 2026-11-22-add-two`。
- 同樣的資料在本機專案、遠端專案與 `host.dispatch` 得到相同的項目。遠端專案的 CLI 用一次 `GET /archived` 的回應組出輸出。不送 `deltaCapabilities` 的舊版 Server，項目沒有這個鍵，指令不失敗。連不到 Server 或登入失效時，和 `list --json` 相同地失敗。
- 經 `host.dispatch` 執行時，`beforeCommand` 收到的命令名稱是 `list`，參數裡 `archived` 是 true。這份清單對儲存的成本是一次列出封存區加一次讀取。

`speclink show <變更> --json`：

| 欄位 | 內容 |
| --- | --- |
| `review`、`verify` | 審查站與驗證站的狀態。`state` 是 `inProgress`（有未結的工單）或 `stamped`（已蓋章）。已蓋章時另有 `stampedAt`、`rounds`、`tasksTotal`，以及有值才出現的 `stampedBy`、`stampedWith`、`commit`（蓋章時 checkout 的 HEAD）。沒有工單也沒有章的站，整個鍵不出現 |
| `branch` | 開工時記下的分支，或落地回報給的分支 |
| `landed` | 落地記錄：`commit`、`at`（回報時間，RFC 3339），以及有回報人時才出現的 `by` |
| `claimed` | 認領記錄：`by`（認領的人），以及有記錄才出現的 `byId`（認領人的識別）與 `at`（認領的時間，照變更資料記的原樣）。沒有被認領的變更，整個鍵不出現。連到不提供識別與時間的舊版 Server 時，只有 `by` |
| `fields` | 自由欄位 |

```json
{
  "name": "add-login",
  "review": { "state": "stamped", "stampedAt": "2026-11-02T14:03:44+08:00", "rounds": 2, "tasksTotal": 5, "commit": "0123456789ab" },
  "verify": { "state": "inProgress" },
  "branch": "speclink/add-login",
  "landed": { "commit": "0123456789ab", "at": "2026-11-03T10:10:00+08:00", "by": "Demo <demo@example.com>" },
  "claimed": { "by": "Demo <demo@example.com>", "byId": "u_42", "at": "2026-11-18T10:00:00+08:00" },
  "fields": { "ticket": "PROJ-1" }
}
```

上面的範例省略了 `schema`、`proposal`、`tasks` 等既有欄位。自由欄位也出現在 `discuss list --json`、`discuss show --json` 與 `discuss search --json` 的討論資訊裡。

#### <a id="show-detail"></a>`speclink show <變更> --json --detail`

`--detail` 帶出每個章的範圍：蓋章時涵蓋的每個檔的路徑與指紋；也帶出未結工單的輪數。宿主因此能比對正式分支上的檔案和通過關卡的檔案，squash 或 rebase 換掉 commit 編號之後也一樣。

- `--detail` 只能和 `--json` 一起用。`speclink show <變更> --detail` 以用法錯誤結束：非零 exit code、stderr 的訊息指出 `--json`、stdout 沒有輸出。經 `host.dispatch`、掛鉤的 `read` 與 `engine.dispatch` 執行時，錯誤代碼是 `invalid_argv`。
- 已蓋章的關卡（`state` 是 `stamped` 的 `review` 或 `verify`）另有 `scope`：陣列，每個檔一項 `{ "path", "hash" }`，順序照記錄的順序。`path` 是相對 repo 根目錄、以正斜線分隔的路徑；`hash` 是蓋章時的指紋（小寫十六進位）。沒有記下任何指紋的章是 `"scope": []`。
- `scope` 只在已蓋章的關卡出現。有未結工單的關卡改帶 `rounds`：工單目前的輪數，例如 `{ "state": "inProgress", "rounds": 2 }`；工單解析不了、或舊版 Server 沒有送時不出現這個鍵。沒有記錄的關卡不出現，和不帶 `--detail` 時相同。
- 不帶 `--detail` 時，`show --json` 的輸出和以前逐位元相同。`list --json --detail` 不帶 `scope`。對象是正式規格時（`--item-type spec`，或名稱是正式規格），帶不帶 `--detail` 輸出都相同。
- 相同的資料在本機專案、遠端專案與 `host.dispatch` 得到相同的 `review` 與 `verify`。遠端專案的 CLI 從 `GET /changes/{name}` 的那一次回應取得 `scope` 與 `rounds`，不另外發請求。不回報關卡的舊版 Server 不給 `review` 與 `verify`，指令不失敗。
- `scope` 是蓋章時記下的內容。這個輸出不判斷檔案在蓋章後有沒有改過；蓋章前已刪除的檔也不在裡面。算指紋的函式與比對檔案的掛鉤，見 [Node SDK](sdk-node.zh-TW.md#stamp-scope)。

```json
{
  "name": "add-login",
  "review": {
    "state": "stamped",
    "stampedAt": "2026-11-02T14:03:44+08:00",
    "rounds": 2,
    "tasksTotal": 5,
    "commit": "0123456789ab",
    "scope": [
      { "path": "src/auth.ts", "hash": "42c1e65b2c948bb754efb6ac171319d6e97ecb3d9afd4f20bd91b3ded25183c0" },
      { "path": "src/login.ts", "hash": "e9024f1a07d29d52ad3aa5e1a18e94db1f3a9fd32b89e39d47c472cd99071e13" }
    ]
  },
  "verify": { "state": "inProgress", "rounds": 2 }
}
```

#### <a id="ticket-detail"></a>`speclink review show <變更> --json --detail`

`--detail` 帶出工單的全文。顯示審查或驗證進行中的宿主畫面因此能用動詞讀到整份工單。`speclink verify show <變更> --json --detail` 對驗證工單做同樣的事。

- `--detail` 只能和 `--json` 一起用。`speclink review show <變更> --detail` 以用法錯誤結束：非零 exit code、stderr 的訊息指出 `--json`、stdout 沒有輸出。經 `host.dispatch` 執行時，錯誤代碼是 `invalid_argv`。
- 輸出在 `change`、`rounds`、`lastRound` 之外多一個 `content`：工單的全文，和 `speclink review show <變更>` 印出的原文相同。鍵照字母順序排列，所以 `content` 在 `change` 之後。其餘三個鍵和不帶 `--detail` 時相同。
- 不帶 `--detail` 時，`review show --json` 與 `verify show --json` 的輸出和以前逐位元相同，沒有 `content`。審查與驗證技能讀的是這個形式，所以工單全文不會進到它們的上下文。
- 變更沒有工單、或變更不存在時，和不帶 `--detail` 時相同地失敗。
- 同一張工單在本機專案、遠端專案與 `host.dispatch` 得到相同的輸出。遠端專案的 CLI 從 `GET /changes/{name}/{station}` 的那一次回應取得 `content`。不送 `content` 的舊版 Server，輸出沒有這個鍵，指令不失敗。

```json
{
  "change": "add-login",
  "content": "# Review — add-login\n\n## Round 1\n\n**Scope**: src/login.rs\n",
  "lastRound": { "index": 1, "phase": null, "patchHash": null, "rules": null, "scope": ["src/login.rs"], "findings": [] },
  "rounds": [{ "index": 1, "phase": null, "patchHash": null, "rules": null, "scope": ["src/login.rs"], "findings": [] }]
}
```

#### <a id="discussion-promoted-to"></a>`discuss list --json`、`discuss show --json` 與 `discuss search --json` 的 `promotedTo`

討論轉出或併入了變更時，`discuss list --json` 的每一項（帶 `--archived` 也一樣）、`discuss show <slug> --json` 的 `info` 與 `discuss search --json` 的每一筆命中有 `promotedTo`：變更名稱，順序和討論記錄 `promoted_to` 的順序相同。搜尋的命中項是清單的項目加上 `matches`，所以它的 `promotedTo` 和清單相同。

- 這是刻意的輸出變更。沒有轉出的討論，輸出和以前逐位元相同。鍵照字母順序排列，所以 `promotedTo` 在 `path` 與 `rounds` 之間；其他鍵的名稱、順序與值不變。
- 三個動詞的人眼輸出不變。
- 同樣的資料在本機專案、遠端專案與 `host.dispatch` 得到相同的 `promotedTo`。遠端專案的 CLI 從 Server 本來就送的討論資訊與搜尋命中取得它。不送這個欄位的舊版 Server，輸出沒有 `promotedTo`，指令不失敗。

```json
{ "archived": false, "created": "2026-10-07", "path": "discussions/board-search-bar.md", "promotedTo": ["add-search-bar"], "rounds": 1, "slug": "board-search-bar", "status": "promoted", "topic": "看板搜尋列" }
```

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
| `X-Speclink-Repo` | 儲存庫代號 | 專案只有一個 repo 時可省略；有多個 repo 卻省略回 409 `refused`，訊息寫出兩種指定方式（這個標頭，或 `.speclink.yaml` 的 `remote.repo`）；寫了沒註冊的 repo 回 404 `not_found`。專案底下的 `/whoami` 是例外：不看這個標頭，一律列出專案的全部 repo |
| `Content-Type` | `application/json`（有 body 時） | 415，回應是純文字 |

憑證由 Server 決定。示範用 server 接受兩種：存取金鑰（PAT，`spk_pat_` 開頭，在 server 的帳號頁建立），或 `speclink auth login` 取得的裝置登入憑證（`spk_at_` 開頭）；憑證有效但帳號不是這個專案的成員，回 403。內嵌的宿主自己驗用戶端送來的 token，再把身分交給入口；驗不出身分時傳空值，入口就回 401。宿主宣告了瀏覽器登入時，憑證是它的登入服務發的 access token。

### 角色

示範用 server 的專案成員有 `reader` 與 `editor` 兩種角色。reader 只能讀：所有寫入端點（方法不是 GET 的端點）都要 editor，reader 呼叫會收到 403 `permission_denied`，`message` 為 `your role in project '<代號>' is reader; this action needs the editor role`，Server 什麼都不寫。有三個例外：

- `POST /context` 與 `POST /skills` 用 POST 傳送查詢條件，但只讀資料，reader 照常可用。
- `GET /export` 一次交出整個範圍的全部文件，所以要 editor。

內嵌的宿主沒有這兩個角色名稱：宿主的範圍函式對每個 repo 回答「這個人能不能寫入」，入口照它把關。下文端點表的「最低角色」寫 `editor`，就是指需要寫入的能力。

### 握手

連上之後先呼叫 `GET /binding`。它告訴你目前是誰、綁到哪個專案與 repo、Server 版本，以及依角色開放的能力：

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
    },
    "skills": { "version": "v1.58.0" },
    "fields": true
  }
}
```

- `apiVersion` 與你送出的版本不同時，用戶端應該停下來，不要繼續呼叫其他端點。
- `policyWrite`、`deleteChange`、`moveTask` 只有 editor 是 `true`；`validate`、`analyze` 對所有角色都是 `true`。
- `events.transports` 列出 Server 提供的推送串流。上面是示範用 server 的回應。沒有自己提供串流的宿主回空陣列，用戶端就只用 `polling` 的 `GET /sync-state` 輪詢。
- `skills.version` 是 Server 提供的技能版本，技能本身用 `POST /skills` 取得。`fields` 為 `true` 表示 Server 會存自由欄位。舊版 Server 沒有這兩個鍵。
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
| `not_found` | 404 | 專案或 repo 沒有註冊；變更、討論、artifact、規格或 capability 不存在 |
| `invalid_argument` | 400 | 參數值不對：slug 格式錯、缺 `If-Match`、`If-Match` 不是數字、未知的 artifact、搜尋關鍵字是空的；或寫入的內容格式不對：`tasks.md` 沒有 `- [ ]` 核取方塊、proposal 缺 `## Why`／`## Problem`／`## Summary`、design 缺 `## Context`、delta spec 沒有操作區段、審查站或驗證站的輪缺 `**Scope**:` 行、輪的 `**Phase**:` 與工單狀態不符，或套用規則記錄有誤（`**Rules**:`／`**Rule**:` 行格式錯誤，或出現在首輪以外的輪） |
| `invalid_config` | 422 | 變更的 metadata 壞掉；工作流設定文件解析不了 |
| `refused` | 409 | 前置條件不成立：API 版本不符、多 repo 卻沒指定 repo、需要 `force`、任務序號超出範圍、變更已被別人認領、依賴成環、匯入的目標不是空的、匯入的包是不認得的格式版本 |
| `refused` | 413 | 請求 body 超過大小上限（匯入的上限由宿主設定，預設 32 MiB），或看板順序內容超過 1 MiB |
| `revision_conflict` | 409 | 寫入時附的版本已經不是最新（見[請求的共同規則](#request-rules)） |
| `unavailable` | 503 | 儲存暫時無法服務，或匯出期間 repo 一直被改動 |
| `internal` | 500 | 其他失敗 |

## <a id="endpoints"></a>端點總表

路徑都接在[專案網址](#request-rules)後面。`{name}` 是變更名稱，`{slug}` 是討論代號，`{artifact}` 是 `proposal`、`design`、`tasks` 或 `specs/<capability>`。「最低角色」寫 `reader` 代表所有成員都能呼叫。

除了特別註明的幾條，下面的路由都在宿主套件的 HTTP 入口裡，內嵌的宿主與示範用 server 都有。

內嵌引擎的宿主的入口，對讀寫範圍文件的每一條路由先問宿主的執行前掛鉤，不執行引擎命令的讀取也一樣：`GET /config`、`GET /board-order`、`GET /specs/{capability}/document`、`GET /search`、`GET /archived` 與它底下的兩條、`POST /context`、`GET /changes/{name}/drift`。掛鉤拒絕時回 409 `refused` 與宿主的原因。示範用 server 沒有掛鉤。掛鉤收到的命令名稱見 Node SDK 的[「服務 CLI 與 Speclink Desktop」](sdk-node.zh-TW.md#http-entry)。

### 連線與同步

| 方法 | 路徑 | 最低角色 | 用途 |
| --- | --- | --- | --- |
| GET | `/binding` | reader | 握手：身分、專案、repo、版本與能力 |
| GET | `/whoami` | reader | 目前使用者，以及這個專案的所有 repo。不需要 `X-Speclink-Repo`，回應沒有 ETag |
| GET | `/sync-state` | reader | 只回 `ETag`；帶 `If-None-Match` 且沒變時回 304 |
| GET | `/events` | reader | SSE 事件流，見下方[事件流](#events)。這一條是示範用 server 自己的，不在宿主套件的入口裡 |
| POST | `/context` | reader | 一次取回一致的文件快照；body 可帶 `change` 縮小範圍 |
| POST | `/skills` | reader | 取得這個專案的技能檔（`speclink update` 在遠端專案使用） |

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
| GET | `/changes/{name}/evidence` | reader | 完成證據：勾選任務時記下的檔案（`speclink evidence show`）。`{name}` 也可以是已封存變更的名稱 |
| POST | `/changes/{name}/in-progress` | editor | 標記開工（`speclink in-progress add`） |
| DELETE | `/changes/{name}/in-progress` | editor | 移除開工標記（`speclink in-progress remove`） |
| POST | `/changes/{name}/claim` | editor | 認領（`speclink claim`） |
| DELETE | `/changes/{name}/claim` | editor | 釋放認領（`speclink claim --release`） |
| POST | `/changes/{name}/depends` | editor | 宣告或移除前置變更（`speclink change depends`） |
| POST | `/changes/{name}/schema` | editor | 切換變更的 schema（`speclink change schema`） |
| POST | `/changes/{name}/rank` | editor | 把變更排到同一欄另一個變更的前面或後面（`speclink change rank`） |
| POST | `/changes/{name}/landed` | editor | 回報程式碼落在哪個 commit（`speclink change landed`） |
| PUT | `/changes/{name}/fields` | editor | 設定與移除變更的自由欄位（`speclink change field set`、`unset`） |
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
| PUT | `/discussions/{slug}/fields` | editor | 設定與移除討論的自由欄位（`speclink discuss field set`、`unset`） |
| POST | `/discussions/{slug}/rounds` | editor | 新增一輪（`speclink discuss add-round`） |
| POST | `/discussions/{slug}/conclude` | editor | 寫結論（`speclink discuss conclude`） |
| POST | `/discussions/{slug}/archive` | editor | 封存討論（`speclink discuss archive`） |
| POST | `/discussions/{slug}/restore` | editor | 把已封存的討論搬回進行中（`speclink discuss restore`） |
| POST | `/discussions/{slug}/promote` | editor | 轉為變更（`speclink discuss promote`） |
| POST | `/discussions/{slug}/link` | editor | 把既有變更連到這個討論（`speclink discuss link`） |
| POST | `/discussions/{slug}/seal` | editor | 內容寫好後，把討論標為已轉出變更（`speclink discuss seal`） |

### 專案層資料

| 方法 | 路徑 | 最低角色 | 用途 |
| --- | --- | --- | --- |
| GET | `/plan` | reader | 變更的執行順序（`speclink plan`） |
| GET | `/plan?strictOverlap=true` | reader | 嚴格重疊的執行順序（`speclink plan --strict-overlap`）；回應多帶 `strictOverlap: true` |
| GET | `/trace/{capability}` | reader | 一個 capability 的溯源鏈（`speclink trace`） |
| GET | `/specs` | reader | 正式規格清單 |
| GET | `/specs/{capability}/document` | reader | 一份正式規格的全文 |
| GET | `/archived` | reader | 已封存變更清單 |
| GET | `/archived/{datedName}/artifacts/{path}` | reader | 已封存變更裡的一份文件，`{path}` 例如 `proposal.md` |
| GET | `/archived/{datedName}/capabilities` | reader | 該次封存動到的 capability 名稱 |
| GET | `/search?q=<文字>` | reader | 桌面 app 的全文搜尋（只搜在途的變更與討論） |
| GET | `/language` | reader | 共用詞彙文件（`speclink language show`） |
| PUT | `/language` | editor | 整份寫入共用詞彙文件（`speclink language replace`） |
| GET | `/config` | reader | 工作流設定原文與修訂號 |
| PUT | `/config` | editor | 寫工作流設定（`speclink workflow-config set`） |
| GET | `/board-order` | reader | 看板順序 |
| PUT | `/board-order` | editor | 整份取代看板順序，需帶 `If-Match` |
| GET | `/export` | editor | 匯出這個 repo 的全部文件，回一個第 2 版的包（見[匯出與匯入的包](#bundles)） |
| POST | `/import` | editor | 把一個包整批寫進這個 repo（桌面 app 搬本機專案時使用）；目標 repo 必須是空的 |

CLI 目前沒有匯出與匯入的動詞，這兩條路由由桌面 app 與宿主自己的程式使用。

### 專案網址以外的端點

這幾個端點直接接在 `https://<server>` 後面。前三條在宿主套件的入口裡：

| 方法 | 路徑 | 需要什麼 | 用途 |
| --- | --- | --- | --- |
| GET | `/api/speclink/v1/auth-methods` | 不需要 | Server 支援的登入方式。用戶端在登入前呼叫它，不用帶憑證，也不用帶版本標頭 |
| GET | `/api/speclink/v1/whoami` | 只需憑證 | 憑證屬於誰（還沒選專案時用） |
| GET | `/api/speclink/v1/scopes` | 只需憑證 | 這個帳號看得到的專案與 repo |

`GET /api/speclink/v1/auth-methods` 的回應：

```json
{ "methods": ["browser", "device", "token"], "browser": { "issuer": "https://id.example.com", "clientId": "speclink", "scopes": ["openid"] } }
```

- `methods` 的值是 `browser`（瀏覽器登入）、`device`（裝置登入）與 `token`（存取金鑰）。用戶端依這個固定順序選 Server 有列的第一個。看不懂的值不算錯誤。
- `browser` 物件只在支援瀏覽器登入時出現：`issuer` 是登入服務的網址，`clientId` 是用戶端識別，`scopes` 選填。
- 宿主沒有宣告登入方式時回 404，用戶端照舊先試裝置登入、不行再問存取金鑰。示範用 server 回 `device` 與 `token`。

下面幾條是示範用 server 自己的路由，不在宿主套件的入口裡：

| 方法 | 路徑 | 需要什麼 | 用途 |
| --- | --- | --- | --- |
| GET | `/healthz` | 不需要 | 程序在執行就回 200 |
| GET | `/readyz` | 不需要 | 儲存後端可用才回 200，否則 503 |
| GET | `/auth/whoami` | 只需憑證 | 憑證屬於誰；較早的路徑，新的用戶端先問 `/api/speclink/v1/whoami` |

`/auth/device`、`/auth/device/token`、`/auth/refresh` 與 `/auth/revoke` 是示範用 server 的裝置登入流程，`speclink auth login` 的裝置登入使用它們。

### <a id="events"></a>事件流

每一次寫入的提交都帶著事件。Server 把事件推給用戶端的方式由它自己決定，並在握手的 `events.transports` 宣告。示範用 server 提供 `GET /events`，它是 SSE（Server-Sent Events：Server 透過一條長連線持續推送事件）。用 Node 內嵌的宿主目前沒有現成的串流，用戶端靠 `GET /sync-state` 輪詢。

事件只指出哪個資源變了，不帶新的內容，這是刻意的設計：收到事件後重新讀取那個資源。這樣用戶端顯示的永遠是 Server 上的內容；漏掉一筆事件也不會遺失什麼，因為下一次讀取就拿到目前的狀態。

`sse` transport 的 `url` 有兩種寫法：

- 以 `/` 開頭的路徑，例如 `/events`。它接在專案網址之後，和 `polling` 的 url 相同：`/events` 就是 `{origin}/api/speclink/v1/projects/{project}/events`。
- 以 `http://` 或 `https://` 開頭的完整網址。它必須和 Server 是同一個 origin：協定、主機、埠都相同。

Speclink Desktop 使用第一個 `sse` transport。網址是別的 origin 時，Desktop 不連線，也不把你的憑證送過去，改用 `GET /sync-state` 輪詢，而且不顯示錯誤。握手沒有宣告 `sse` transport 時（例如只宣告 `websocket`）也一樣輪詢。

- `invalidate` 事件：某個資源變了。`data` 例如 `{"eventId":"42","scope":"change","resourceId":"add-login","revision":42}`。收到後重新讀取對應資源。
- `reset` 事件：你的續傳位置已被清掉，請重新讀取全部資料。
- 斷線重連時帶 `Last-Event-ID`，可以補收中間漏掉的事件。
- 串流閒置時，Server 送一行註解當作心跳（heartbeat）。示範用 server 預設每 15 秒送一次（設定檔的 `events.heartbeat_secs`，必須小於 45）。
- 用戶端 45 秒沒有收到任何資料（心跳也算）時，把連線視為中斷：先用 `GET /sync-state` 確認有沒有漏掉的變更，再重新訂閱。自己提供串流的宿主，心跳的間隔必須小於 45 秒。

`scope` 與 `resourceId` 由事件的種類決定。內嵌的宿主自己推送事件時，用同一張對照：

| 事件種類 | `scope` | `resourceId` |
| --- | --- | --- |
| `change-created`、`artifact-created`、`task-completed`、`task-uncompleted`、`task-moved`、`change-claimed`、`change-claim-released`、`change-marked-in-progress`、`change-in-progress-removed`、`change-depends-changed`、`change-schema-changed`、`change-ranked`、`change-fields-changed`、`change-landed`、`change-discarded`、`change-provenance-changed` | `change` | 變更名稱 |
| `review-round-added`、`review-stamped`、`review-discarded`、`verify-round-added`、`verify-stamped`、`verify-discarded` | `change` | 變更名稱 |
| `change-archived` | `spec` | 變更名稱 |
| `discussion-created`、`discussion-context-set`、`discussion-round-added`、`discussion-concluded`、`discussion-promoted`、`discussion-linked`、`discussion-sealed`、`discussion-unlinked`、`discussion-archived`、`discussion-restored`、`discussion-fields-changed`、`discussion-discarded` | `discussion` | 討論代號 |
| 其他（例如匯入的 `scope-imported`、寫入看板排序的 `board-order-updated`、寫入共用詞彙文件的 `language-replaced`、寫入工作流設定的 `workflow-config-changed`） | `unknown` | 空字串 |

- 審查站與驗證站的六種事件都歸到變更：新增一輪、蓋章、丟棄工單，`resourceId` 是變更名稱。用戶端收到後重新讀取那個變更，就拿得到新的關卡狀態。
- `review-stamped` 與 `verify-stamped` 的 payload 有 `change`；這次蓋章有記下 commit 時（蓋章請求帶了 `headCommit`），還有 `commit`，值和變更 metadata 記下的相同。
- `change-landed` 的 payload 有 `change`、`commit`，以及這次有回報分支時才出現的 `branch`。
- 兩種自由欄位的事件，payload 只有名稱與動到的鍵（`keys`），沒有值。
- `workflow-config-changed` 來自真的寫入了文件的工作流設定寫入；dry-run、內容相同的寫入與被拒絕的寫入都沒有事件。payload 是 `{"keys": [...]}`：值有變動的鍵，順序與這次寫入相同；只改了排版或註解時是空陣列。payload 沒有值。這筆事件歸在 `unknown`，用戶端收到後重新讀取全部資料。
- 推送給用戶端的訊息不帶 payload，沒有 `commit` 也沒有 `keys`；只有宿主的儲存與 `afterCommit` 掛鉤拿得到它們。
- 收到 `unknown` 時重新讀取全部資料。匯入一次改動整個範圍，所以刻意歸在這一類。

#### <a id="multi-event-commits"></a>一次提交的多筆事件

一筆事件只指一個資源。一次提交寫到不只一個進行中的變更或討論時，每個資源各有一筆事件：命令原本的事件在前，為其他資源補的事件在後。事件只為真的發生的寫入送：另一個資源這次沒有改變時，它沒有事件。

| 動詞與情況 | 事件（依序） |
| --- | --- |
| `new change --from-discussion` | `change-created`，接著 `discussion-promoted`（`slug`、`change`） |
| `archive`，來源討論隨變更一起封存 | `change-archived`，接著每份討論一筆 `discussion-archived`（`slug`） |
| `discuss conclude`，同一次執行把討論封存 | `discussion-concluded`，接著 `discussion-archived`（`slug`） |
| `discard`，來源討論被解開 | `change-discarded`，接著每份討論一筆 `discussion-unlinked`（`slug`、`change`） |
| 再次 `discuss conclude`，已反映這份討論的變更被記上「待重新反映」 | `discussion-concluded`，接著這次新記上標記的每個變更一筆 `change-provenance-changed`（`change`） |
| `discuss link`，第一次連結這個變更 | `discussion-linked`，接著 `change-provenance-changed`（`change`） |
| `discuss seal`，清掉變更的「待重新反映」 | `discussion-sealed`，接著 `change-provenance-changed`（`change`） |
| `discuss promote` | `discussion-promoted`，接著 `change-created`（和以前相同） |

- `discussion-unlinked` 與 `change-provenance-changed` 是為這些情況新增的兩種事件，payload 只有名稱：`{"slug", "change"}` 與 `{"change"}`。
- 同一組再 `discuss link` 一次、沒有標記可清的 `discuss seal`、變更已經帶著標記的重新結論，都只送命令原本的那一筆事件。
- 示範用 server 為每一筆事件各推一筆 `invalidate`，順序和提交裡相同。每一筆都有自己的事件識別，所以用 `Last-Event-ID` 續傳是以事件為單位，不是以提交為單位。

## <a id="payload-examples"></a>請求與回應範例

以下範例的欄位形狀取自示範用 server 的實際回應。請求 body 欄寫「無」的端點，送 `{}` 或不送都可以；其他端點一定要送 JSON body，並帶 `Content-Type: application/json`。

### 變更與任務

| 端點 | 請求 body | 成功回應 |
| --- | --- | --- |
| `POST /changes` | `{"name":"add-login","agent":"claude"}` | `{"name":"add-login","schema":"spec-driven","repo":"backend"}` |
| `DELETE /changes/{name}?force=true` | 無 | `{"change":"add-search-bar","unlinkedDiscussions":[{"slug":"board-search-bar","status":"concluded"}]}` |
| `POST /changes/{name}/claim` | 無 | `{"claimedBy":"Demo <demo@example.com>"}` |
| `DELETE /changes/{name}/claim` | 無 | `{"released":true,"previousHolder":"Demo <demo@example.com>"}`；變更沒有被認領時是 `{"released":false}` |
| `POST /changes/{name}/depends` | `{"on":["add-login"],"remove":false}` | `{"change":"add-logout","dependsOn":["add-login"]}` |
| `POST /changes/{name}/schema` | `{"schema":"no-spec-change"}` | `{"change":"tidy","schema":"no-spec-change","previous":"spec-driven","changed":true,"verifyStampCleared":false}` |
| `POST /changes/{name}/rank` | `{"anchor":"add-login","position":"before"}` | `{"change":"add-logout","anchor":"add-login","position":"before"}` |
| `POST /changes/{name}/landed` | `{"commit":"0123456789ab","branch":"speclink/add-login"}` | `{"change":"add-login","commit":"0123456789ab","branch":"speclink/add-login"}` |
| `PUT /changes/{name}/fields` | `{"set":{"ticket":"PROJ-1"},"unset":["source"]}` | `{"fields":{"ticket":"PROJ-1"},"changed":["source","ticket"]}` |
| `POST /changes/{name}/in-progress` | 無，或 `{"branch":"speclink/add-login"}` | `{}` |
| `DELETE /changes/{name}/in-progress` | 無 | `{"removed":true}`；本來就沒開工時是 `{"removed":false}` |
| `POST /changes/{name}/archive` | 無 | `{"specs":[],"datedName":"2026-10-07-fix-typo","snapshotCreated":false,"archivedDiscussions":[],"evidenceRecorded":false}` |
| `POST /changes/{name}/tasks/{taskId}/done` | `{"touchedFiles":["src/login.rs"],"headCommit":"0123…"}` | `{"taskDesc":"1.1 Add the login form","alreadyDone":false}` |
| `POST /changes/{name}/tasks/{taskId}/undone` | 無 | `{"taskDesc":"1.1 Add the login form","alreadyUndone":false}` |
| `POST /changes/{name}/tasks/move` | `{"from":2,"to":1}` | `{"change":"add-login","description":"1.1 Add the session check"}` |

補充：

- `POST /changes` 的選填欄位：`schema`、`description`、`agent`、`fromDiscussion`（來源討論代號）、`last`（搭配 `fromDiscussion`，表示這是結論規劃的最後一個變更）、`fields`（建立時就帶上的自由欄位，和建立在同一次寫入）。
- `DELETE /changes/{name}` 的 `force` 預設 `false`。變更已開工（有開工標記或任一任務已勾）時，不帶 `force=true` 會回 409 `refused`，什麼都不刪。metadata 壞掉時，即使 `force=true` 也回 422 `invalid_config`。回應的 `unlinkedDiscussions` 列出被解除連結的來源討論，以及解除後的狀態。
- `POST /changes/{name}/claim`：同一個人重複認領，照樣成功、不重寫；別人已認領時回 409 `refused`，`message` 寫出目前的持有人。
- `DELETE /changes/{name}/claim`：editor 都可以釋放，不必是認領的人。要更嚴格的宿主在掛鉤裡拒絕命令 `claim-release`，這時 Server 回 409 `refused`，`message` 是宿主給的原因。`previousHolder` 是釋放前的持有人名稱，只在那次認領有寫持有人時出現。變更沒有被認領時回 200、`released` 是 `false`，什麼都不寫。變更不存在回 404；還沒有這個端點的舊版 Server 也回 404，這時 CLI 會說這個 Server 不支援 `claim --release`、要更新 Server。
- `POST /changes/{name}/depends`：`on` 必填，`remove` 預設 `false`。自己依賴自己、目標不存在或已封存、依賴成環，都回 409 `refused`。
- `POST /changes/{name}/schema`：Server 只解析內建的 schema。schema 找不到、要切到沒有 specs 的 schema 而變更帶著 delta spec、有未結的驗證工單，都回 409 `refused`。切到同一個 schema 時回 200、`changed` 為 `false`，Server 不寫入。這次切換移除了驗證章時，`verifyStampCleared` 為 `true`。
- `POST /changes/{name}/rank`：`anchor` 是同一欄的另一個變更，`position` 是 `before` 或 `after`，其他值回 400 `invalid_argument`。選填的 `force` 為 `true` 時，已經手動排過順序的變更也照樣移動。
- `POST /changes/{name}/landed`：`commit` 必填，`branch` 選填。Server 只檢查兩者的形狀，不查 commit 存不存在。回應的 `branch` 只在請求有帶時出現。
- `PUT /changes/{name}/fields`：`set` 是要設定的鍵值，`unset` 是要移除的鍵，兩者至少要有一個不是空的，否則回 400。回應的 `fields` 是寫入後的全部欄位，`changed` 是這次真的改到的鍵；什麼都沒改到時不寫入，`ETag` 不變。`PUT /discussions/{slug}/fields` 的請求與回應相同。
- `POST /changes/{name}/in-progress`：重複呼叫，或變更名稱不存在，照樣回 200 `{}`，什麼都不寫。body 可以帶 `branch`，記下這個 checkout 目前的分支。
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
- 寫入 `tasks.md` 時，Server 會在每個任務行尾補上穩定 ID 註解（`<!-- speclink-task:tsk_… -->`）。

### 品質關卡

| 端點 | 請求 body | 成功回應 |
| --- | --- | --- |
| `POST /changes/{name}/review/rounds` | `{"content":"<一輪工單的 Markdown>"}` | `{"round":1}` |
| `POST /changes/{name}/review/stamp` | `{"accept":false,"agent":"claude-code","scope":[{"path":"src/login.rs","hash":"<64 字元>"}],"missing":[]}` | `{"change":"add-login"}` |
| `DELETE /changes/{name}/review` | 無 | `{"change":"add-login"}` |

- 一輪的內容格式與 `speclink review add-round` 從 stdin 收的內容相同，必須有 `**Scope**:` 行。
- `GET /changes/{name}/review` 回 `change`、`rounds`、`lastRound` 與 `content`（工單原文）。每一輪有 `index`、`phase`、`patchHash`、`rules`、`scope` 與 `findings`（每項 `severity`、`path`、`text`）。`rules` 為 null，或是帶 `baseline`（`extend` 或 `replace`）與 `critical`、`warning`、`suggestion` 三個清單的物件；只有記錄了套用規則的首輪才有物件。
- 蓋章的 `scope` 是工單各輪 Scope 列過的檔案，在你工作目錄裡的內容指紋。`hash` 是檔案內容的 SHA-256（小寫十六進位）；UTF-8 文字檔先把 CRLF 換成 LF 再算。
- 工作目錄裡已經不存在的檔案放進 `missing`。`scope` 加 `missing` 必須剛好等於工單列過的檔案；Server 不重算指紋，只檢查檔案集合。
- 蓋章前，變更的寫碼任務必須全部完成。`accept: true` 表示最後一輪還有必須修的發現（CRITICAL／WARNING）也照樣蓋章。
- 蓋章的 body 可以帶選填的 `headCommit`：你的 checkout 目前的 commit。Server 只檢查形狀，把它記在章上；之後出現在變更回應的 `review.commit`（或 `verify.commit`）。

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
| `POST /discussions/{slug}/restore` | 無 | `{"slug":"board-search-bar","restoredFrom":"2026-11-10-board-search-bar.md"}` |
| `DELETE /discussions/{slug}?force=true` | 無 | `{"slug":"board-search-bar"}` |

- `POST /discussions`：`slug` 選填，省略時由 `topic` 推導；必須是小寫英數字以單一連字號分隔，不合格回 400 `invalid_argument`。選填 `kind` 目前只接受 `improve`。選填 `fields` 是建立時就帶上的自由欄位。
- `conclude`：`hold: true` 表示結論之後記錄保留在途，還欠尚未建立的變更。回應的 `restaleFlagged` 列出因重寫結論而被打回的變更；`autoArchived: true` 只在結論順便封存了記錄時出現。
- `promote`：`name` 選填；選填的 `fields` 是新變更一開始就帶的自由欄位。`promote` 與 `seal` 都接受選填的 `last: true`，表示這是結論規劃的最後一個變更，會解除記錄的保留旗標。
- `seal`：變更必須先用 `link` 連到這個討論，否則回 409 `refused`。
- `POST /discussions/{slug}/restore`：搬回同一個 slug 最新的一份封存記錄，內容不變；`restoredFrom` 是它封存時的檔名。同一個 slug 已有進行中的記錄時回 409 `refused`；沒有封存的記錄時回 404 `not_found`。
- `DELETE /discussions/{slug}`：`force` 預設 `false`。已有輪的討論不帶 `force=true` 會回 409 `refused`；封存的討論不能刪，也回 409 `refused`。

### 專案層資料

| 端點 | 請求 body | 成功回應 |
| --- | --- | --- |
| `PUT /language` | `{"content":"# Language\n"}` | `{"created":true}` |
| `PUT /config` | `{"content":"schema: spec-driven\n","expectedRevision":16}` | `{"revision":17}` |
| `PUT /board-order`（加 `If-Match: "16"`） | `{"content":"{\"changes\":{\"add-logout\":\"a\"}}"}` | `{"revision":17}` |

- `PUT /language` 收整份共用詞彙文件，整份取代；`created` 表示這次是否建立了原本不存在的文件。`content` 是空的或只有空白時回 400 `invalid_argument`，文件不變。有寫入權限就能寫，不需要改工作流政策的權限。
- `PUT /config` 收整份設定文件；Server 先解析，解析不了回 422 `invalid_config`。
- `POST /context` 的 body 可帶 `change`，只取這個變更的文件；送 `{}` 就取全部。回應有 `snapshotId`、`digest` 與 `documents`，每份文件有 `path`、`content`、`revision` 與 `digest`。CLI 用它把 Server 上的文件鏡像到本機的唯讀目錄 `.speclink/context/`。
- 看板順序的 `content` 是一段 JSON 文字，形如 `{"changes":{"<變更>":"<排序鍵>"},"discussions":{…}}`。`PUT /board-order` 原樣存下內容。`GET /plan` 與 `POST /changes/{name}/rank` 會讀它，看不懂的那一段當作沒有排序；`rank` 寫回時只保留在途變更的排序鍵，討論那一段原樣帶回。

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

`GET /plan?strictOverlap=true` 回同一個形狀，另外多一個 `"strictOverlap": true`。這個標記只在嚴格重疊的計畫出現。舊版 Server 不認得這個查詢參數，會回一般的計畫而且沒有標記；用戶端看到沒有標記，就要當成「這台 Server 不支援」，不要把它當成嚴格重疊的計畫。

`GET /trace/{capability}` 的回應和 `speclink trace --json` 相同：

```json
{
  "capability": "csv-export",
  "requirements": [{ "name": "Export report as CSV", "source": "add-csv-export" }],
  "changes": [
    { "name": "add-csv-export", "archivedDir": "2026-10-07-add-csv-export", "fromDiscussion": "csv-export-scope", "evidence": [{ "taskId": "tsk_01M4…", "files": ["src/export.rs"] }] }
  ],
  "discussions": [{ "slug": "csv-export-scope", "archived": true, "promotedTo": [{ "change": "add-csv-export", "capabilities": ["csv-export"] }] }]
}
```

- `changes` 是動過這個 capability 的已封存變更，由舊到新。`fromDiscussion` 沒有來源討論時是 `null`；`evidence` 沒有記錄時是 `null`。
- capability 不存在時回 404 `not_found`，`message` 會列出名稱相近的 capability。

`GET /changes/{name}/evidence` 的回應：

```json
{ "change": "add-login", "entries": [{ "taskId": "tsk_01M4…", "taskDesc": "1.1 Add the login form", "touchedFiles": ["src/login.rs"], "recordedAt": "2026-10-07T03:12:45Z" }] }
```

- `{name}` 先找在途的變更，沒有時找已封存的變更（封存名稱，或變更名稱取最新一筆）。讀到已封存的變更時多一個 `archivedAs`。
- 變更存在但沒有記錄：`entries` 是空陣列。在途與已封存都沒有這個名稱：404 `not_found`。
- 每筆的 `actor`、`repo`、`headCommit` 有值才出現。
- 回應一定帶 `change`。較早的 Server 只讀在途的變更而且不帶這個鍵；CLI 的 `evidence show` 沒看到它就以非零結束，不會把空集合印成「沒有記錄」。

### 技能

`POST /skills` 回這個專案的 checkout 該安裝的技能。它只讀資料，reader 也能呼叫。

```text
POST /skills
{"targets":[{"tool":"claude"},{"tool":"my-agent","custom":{"invocation":"tool-call"}}],"contents":true}
→ {"version":"v1.58.0","targets":[{"tool":"claude","skills":[{"name":"apply","digest":"<摘要>","content":"…"}]},{"tool":"my-agent","skills":[…]}]}
```

- `targets` 列出要哪些工具的技能，至少一個、最多 32 個。內建工具只寫 `tool`；自訂工具另外帶 `custom.invocation`。不認得的工具回 400 `invalid_argument`。
- `contents` 為 `true` 才回每支技能的全文（`content`）；省略時只回 `name` 與 `digest`，用來比對本機的技能檔是不是最新的。
- `version` 是技能的版本，和握手的 `skills.version` 相同。
- 技能的文字是給「checkout 裡沒有規格檔」的專案用的版本：讀寫規格都經動詞，不寫規格資料夾的路徑。兩支 worktree 技能只在工作流設定開啟 `worktree` 時出現。宿主設定了技能清單時，只回清單裡的技能。

### <a id="bundles"></a>匯出與匯入的包

`GET /export` 回一個包，裡面是這個 repo 的全部文件。`POST /import` 把一個包整批寫進空的 repo。包有兩個版本，用 `formatVersion` 區分：

| | 第 1 版 | 第 2 版 |
| --- | --- | --- |
| 每份文件怎麼標示 | 文件種類（`document` 物件） | 字串鍵（`key`） |
| 誰產生 | 較早的桌面 app | `GET /export`、現在的桌面 app |
| `POST /import` 收不收 | 收 | 收 |
| 匯入的回應 | 每份文件各一筆結果 | 範圍版本號與文件數 |

第 2 版：

```json
{
  "formatVersion": 2,
  "scopeVersion": 16,
  "documents": [
    { "key": "changes/add-login/proposal.md", "content": "## Why\n…", "digest": "sha256:<64 個十六進位字元>" },
    { "key": "config.yaml", "content": "schema: spec-driven\n", "digest": "sha256:<…>" }
  ]
}
```

- `key` 是文件在儲存裡的鍵，例如 `changes/<變更>/.meta.yaml`、`changes/<變更>/<artifact 路徑>`、`changes/<變更>/.evidence.json`、`specs/<capability>/spec.md`、`discussions/<代號>.md`、`config.yaml`、`LANGUAGE.md`、`board/order`，以及 `archive/changes/<封存名稱>/<文件>` 與 `archive/discussions/<建立日期>-<代號>.md`。
- `digest` 是 `sha256:` 加上 `content` 的 SHA-256（小寫十六進位）。
- `scopeVersion` 是匯出當時的範圍版本號，也是回應的 `ETag`。匯入時會忽略它，所以匯出的回應可以原封不動拿去匯入。
- 匯出的文件依鍵排序。匯出期間 repo 一直被改動時回 503 `unavailable`，稍後再試。
- 匯入成功回 `{"scopeVersion":17,"documents":2}`。

第 1 版：

```json
{
  "formatVersion": 1,
  "scope": { "project": "acme", "repo": "backend" },
  "projectRevision": 0,
  "documents": [
    { "document": { "type": "changeArtifact", "change": "add-login", "artifact": "proposal.md" }, "content": "## Why\n…", "digest": "sha256:<…>" }
  ]
}
```

- `document.type` 是 `changeMeta`、`changeArtifact`、`changeEvidence`、`canonicalSpec`、`discussion`、`workflowConfig`、`archivedChange`、`language` 或 `boardOrder`，各自帶該種類需要的名稱欄位。入口把它換成鍵再匯入。
- `scope` 必須等於請求綁定的專案與 repo，否則回 409 `refused`。
- 匯入成功回 `{"projectRevision":17,"documents":[{"document":{…},"outcome":"created"}]}`。

兩個版本共同的規則：

- 目標 repo 必須是空的，否則回 409 `refused`，什麼都不寫。
- `POST /import` 只收空的 repo。取代模式（讓已經有文件的範圍等於包）與試跑只在宿主的程式呼叫 `host.importScope` 時提供，見 [Node SDK](sdk-node.zh-TW.md#export-import)。body 帶 `mode` 或 `dryRun` 欄位時不是包的形狀，回 422。
- 全部文件在一次提交寫入：全部成功，或全部不寫。提交帶一筆 `scope-imported` 事件，payload 是 `documents`（寫入的份數）。宿主的程式以取代模式匯入時，這筆事件多一個 `removed`（刪除的份數）。
- 每份文件都會檢查：鍵要符合引擎的命名規則、`digest` 要等於內容的摘要、同一個鍵不能出現兩次。任何一項不過回 400 `invalid_argument`，`message` 指出第一份有問題的文件。
- body 不是 JSON 回 400；是 JSON 但不是包的形狀回 422；`formatVersion` 不是 1 也不是 2 回 409 `refused`，`message` 以 `unsupported bundle format version` 開頭。
- body 超過匯入的大小上限回 413。上限由宿主設定，預設 32 MiB。宿主的 Web 框架自己的本文上限不能比它小。

## <a id="response-fields"></a>清單與詳情的選填欄位

下列選填欄位沒有值時，整個鍵都不出現。用戶端遇到缺席的欄位，不要自己補預設值。

### `GET /changes` 的每一項

必定出現：`name`、`summary`、`status`、`completedTasks`、`totalTasks`。

選填：

| 欄位 | 內容 |
| --- | --- |
| `startedAt` | 開工日期，格式 `YYYY-MM-DD`；沒開工就不出現 |
| `createdBy`、`created` | 建立者與建立日期 |
| `fromDiscussions` | 來源討論代號清單 |
| `claimedBy` | 認領人；沒人認領就不出現 |
| `claimedById`、`claimedAt` | 認領人的識別，以及認領的時間（照變更資料記的原樣）。各自有記錄才出現：沒有識別的認領就沒有 `claimedById`。識別是宿主給這個人的那一個，讀得到專案的人都看得到，所以宿主不要拿機密的值當識別 |
| `deltaCapabilities` | 這個變更的 delta 規格動到的 capability |
| `restaleFrom` | 重寫了結論、需要這個變更重新吸收內容的來源討論代號 |
| `metaError` | metadata 壞掉時的原因 |
| `modifiedAt` | 這個變更底下的文件最後一次被修改的時間，RFC 3339 的 UTC 時間；儲存答不出來時不出現 |
| `review`、`verify` | 審查站與驗證站的狀態，見下方[品質關卡的狀態物件](#station-state)。清單項不帶 `scope` |
| `branch` | 開工時記下的分支，或落地回報給的分支 |
| `landedCommit` | 落地回報的 commit |
| `fields` | 自由欄位；沒有欄位時不出現 |

```json
{ "name": "add-login", "summary": "", "status": "in-progress", "completedTasks": 0, "totalTasks": 2, "startedAt": "2026-10-07", "createdBy": "Demo <demo@example.com>", "created": "2026-10-07", "modifiedAt": "2026-10-07T03:12:45Z", "review": { "state": "inProgress" }, "verify": { "state": "none" }, "branch": "speclink/add-login" }
```

`speclink list --json` 只輸出其中的 `restaleFrom`、`metaError`、`deltaCapabilities`、`created`、`modifiedAt` 與 `fields`，不輸出 `review`、`verify`、`branch` 與 `landedCommit`。帶上 `--detail` 時，它另外輸出 `review`、`verify`、`branch`、`createdBy` 與 `fromDiscussions`，用 `landedCommit` 組出 `landed`，用 `claimedBy`、`claimedById`、`claimedAt` 組出 `claimed`。見 [`speclink list --json --detail`](#list-detail)。`GET /changes` 一定帶 `summary`，空字串也照送；`list --json` 遇到空字串就不輸出 `summary`。

### `GET /changes/{name}`

必定出現：`changeName`、`schemaName`、`isComplete`、`applyRequires`、`artifacts`（每項 `id`、`outputPath`、`status`，被擋住時多 `missingDeps`）。

選填欄位：

- `created`（只有 metadata 同時記了 schema 與建立日期時才出現）、`fromDiscussions`、`deltaCapabilities`、`createdBy`、`createdWith`、`startedAt`、`startedBy`、`claimedBy`。
- `claimedById`、`claimedAt`：認領人的識別與認領的時間，規則和上面變更清單的相同。
- `review`、`verify`：品質關卡的狀態物件。已蓋章的站在這裡多帶 `scope`。
- `branch`：開工時記下的分支，或落地回報給的分支。
- `landedCommit`、`landedAt`、`landedBy`：落地回報的 commit、回報時間（RFC 3339）與回報的人。
- `fields`：自由欄位。

CLI 遠端模式的 `show` 與桌面 app 的詳情頁都靠這些欄位。

```json
{ "changeName": "add-login", "schemaName": "spec-driven", "isComplete": false, "applyRequires": ["tasks"], "artifacts": [{ "id": "tasks", "outputPath": "tasks.md", "status": "done" }], "created": "2026-10-07", "createdBy": "Demo <demo@example.com>", "createdWith": "claude", "startedAt": "2026-10-07", "startedBy": "Demo <demo@example.com>", "branch": "speclink/add-login", "landedCommit": "0123456789ab", "landedAt": "2026-10-08T10:10:00+08:00", "landedBy": "Demo <demo@example.com>", "fields": { "ticket": "PROJ-1" } }
```

### <a id="station-state"></a>品質關卡的狀態物件

變更清單項與單筆變更的 `review`、`verify` 是同一個形狀：

| 欄位 | 內容 |
| --- | --- |
| `state` | `none`（沒有工單也沒有章）、`inProgress`（有未結的工單）或 `stamped`（已蓋章）。遇到不認得的值，當成沒有資料 |
| `stampedAt`、`stampedBy`、`stampedWith` | 蓋章的時間、蓋章的人與使用的 agent；只在已蓋章時出現 |
| `rounds` | 已蓋章：蓋章前工單走了幾輪。有未結的工單：工單目前的輪數；工單解析不了時不出現 |
| `tasksTotal` | 蓋章時的任務總數 |
| `commit` | 蓋章的用戶端回報的 commit。Server 不檢查它 |
| `scope` | 蓋章涵蓋的每個檔案的 `path` 與 `hash`。只在單筆變更的回應出現 |

```json
{ "state": "stamped", "stampedAt": "2026-11-02T14:03:44+08:00", "stampedBy": "Demo <demo@example.com>", "stampedWith": "claude-code", "rounds": 2, "tasksTotal": 5, "commit": "0123456789ab", "scope": [{ "path": "src/login.rs", "hash": "<64 字元>" }] }
```

- Server 只回報事實。「蓋章後檔案有沒有變」要由有 checkout 的用戶端自己判斷：拿 `scope` 的指紋比對工作目錄，再比對 `tasksTotal` 與目前的任務總數。
- 變更的 metadata 壞掉時，或 Server 還不回報關卡狀態時，`review` 與 `verify` 兩個鍵都不出現。用戶端把這兩種情況都當成沒有資料。

### <a id="archived-items"></a>`GET /archived` 的每一項

必定出現：`datedName`、`date`、`name`、`specCount`、`fromDiscussions`。

選填：`tasksTotal`、`tasksDone`、`createdBy`、`fields`、`deltaCapabilities`（delta spec 的 capability，依名稱遞增，個數等於 `specCount`），以及兩個關卡的結局 `reviewOutcome` 與 `verifyOutcome`。結局的值是 `none`（沒跑這一站）、`stamped`（已蓋章）或 `notPassed`（帶著未結的工單封存）。封存的文件裡有工單就是 `notPassed`，章還留在 metadata 也一樣。

### `GET /discussions` 的每一項

必定出現：`slug`、`topic`、`status`、`rounds`、`created`、`path`、`archived`。示範用 server 也一定會填 `concluded`（結論段是否已寫）與 `hold`（是否保留在途）。

選填：`createdBy`、`kind`（改進討論為 `improve`）、`fields`（自由欄位），以及 `promotedTo`——這則討論已轉出的變更名稱，依轉出先後排列。`promotedTo` 直接取自討論記錄，沒有轉出過就不出現。

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
- 遠端模式的 `list` 永遠不帶這個欄位，因為 Server 不知道你本機的 checkout。所以沒有 worktree 時，兩種模式的輸出逐欄一致。
- 欄位出現時，這一項的值全部取自 worktree 裡的副本，不是主 checkout 的，例如 `completedTasks`、`totalTasks`、`status`、`metaError` 與 `deltaCapabilities`。`show --json` 仍然讀主 checkout，所以它的 `deltaSpecs` 可能和清單的 `deltaCapabilities` 不同。

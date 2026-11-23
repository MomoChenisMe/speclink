## ADDED Requirements

### Requirement: 已封存變更的清單

`speclink list` SHALL 接受旗標 `--archived`，不讀 stdin。帶上它時，stdout SHALL 是一個 JSON 物件 `{ "archived": [ … ] }`，SHALL NOT 含 `changes` 與 `specs` 鍵；成功時 exit code SHALL 是 0。`archived` 的每一項代表一個已封存的變更，SHALL 含下列鍵（鍵名一律 camelCase）：

- `datedName`：字串，封存後帶日期的名稱（例如 `2026-11-22-add-two`）。SHALL 一定出現。
- `date`：字串，`datedName` 開頭的日期 `YYYY-MM-DD`；名稱沒有日期前綴時 SHALL 是空字串。SHALL 一定出現。
- `name`：字串，去掉日期前綴的名稱；名稱沒有日期前綴時 SHALL 等於 `datedName`。SHALL 一定出現。
- `reviewOutcome`、`verifyOutcome`：字串，`none`、`stamped` 或 `notPassed`，判定 SHALL 與線上格式已封存清單的同名欄位相同（封存的文件含該關卡的工單是 `notPassed`；否則 metadata 有章是 `stamped`；否則 `none`）。SHALL 一定出現。
- `tasksTotal`、`tasksDone`：整數。封存的文件含任務清單時兩者 SHALL 都出現，否則 SHALL 都不出現。
- `createdBy`：字串。metadata 沒有建立者時 SHALL 不出現。
- `fromDiscussions`：字串陣列。沒有來源討論時 SHALL 不出現。
- `deltaCapabilities`：字串陣列，封存的文件裡有 delta spec 的 capability 名稱，依名稱遞增。沒有時 SHALL 不出現。
- `fields`：物件，自由欄位。沒有時 SHALL 不出現。

項目 SHALL 依 `datedName` 由大到小排列；`--sort` SHALL NOT 影響這個順序。沒有已封存的變更時輸出 SHALL 是 `{ "archived": [] }`。metadata 無法解析的已封存變更 SHALL 照常列出：`createdBy`、`fromDiscussions`、`fields` 不出現，兩個結局以「沒有章」判定，指令 SHALL NOT 失敗。

`--archived` 不帶 `--json`，以及 `--archived` 與 `--specs`、`--changes` 或 `--detail` 並用，SHALL 以用法錯誤拒絕：CLI 以非零 exit code 結束、訊息走 stderr、stdout 沒有輸出；經宿主套件執行時以 `invalid_argv` 拒絕。`--archived` 沒有人眼輸出，因為它不能脫離 `--json` 使用；`--no-color` 對 JSON 輸出沒有作用。輸出 SHALL 不隨 `locale`（`tw`、`ja`、`en` 或未設定）改變。這個旗標只讀取：SHALL NOT 寫入任何檔案，`.meta.yaml` 與 `.speclink/` 底下的檔案 SHALL 不變。不帶 `--archived` 的 `speclink list` 的人眼輸出與 `--json` 輸出 SHALL 維持既有的位元級輸出。

三種執行方式對相同的資料 SHALL 輸出相同的鍵與值：本機專案、連到遠端的 CLI、宿主套件。連到遠端的 CLI SHALL 以既有的已封存清單端點組出這份輸出，SHALL NOT 對每個已封存的變更另外發一次請求，SHALL NOT 讀取本機的規格資料夾。連到已封存清單的項目沒有 `deltaCapabilities` 的舊版 Server 時，該鍵 SHALL 不出現，指令 SHALL NOT 失敗。離線或認證失效時 SHALL 與不帶 `--archived` 的 `list` 相同地失敗。這個動詞只讀取，沒有 revision 衝突的情況。經宿主套件執行時，執行前掛鉤 SHALL 收到命令名稱 `list`，參數裡 `archived` 是 true。

已封存變更的文件 SHALL 以既有的 `speclink artifact cat <artifact> --change <datedName>` 讀取；這條需求 SHALL NOT 新增讀文件的動詞。

#### Scenario: 列出已封存的變更

- **WHEN** 專案有兩個已封存的變更：`2026-11-22-add-two`（任務 3 個全部完成、建立者 `Ann <ann@example.com>`、由討論 two 轉出、對 auth 有 delta spec、兩個關卡都蓋了章）與 `2026-11-10-old-one`（沒有任務清單、沒有建立者、沒有章也沒有工單），執行 `speclink list --archived --json`
- **THEN** exit code 是 0；`archived` 依序是 `2026-11-22-add-two` 與 `2026-11-10-old-one`；輸出沒有 `changes` 與 `specs` 鍵

##### Example: 兩個項目的內容

| 鍵 | `2026-11-22-add-two` | `2026-11-10-old-one` |
| --- | --- | --- |
| `datedName` | `2026-11-22-add-two` | `2026-11-10-old-one` |
| `date` | `2026-11-22` | `2026-11-10` |
| `name` | `add-two` | `old-one` |
| `tasksTotal`、`tasksDone` | 3、3 | 兩個鍵都不出現 |
| `createdBy` | `Ann <ann@example.com>` | 不出現 |
| `fromDiscussions` | `["two"]` | 不出現 |
| `deltaCapabilities` | `["auth"]` | 不出現 |
| `reviewOutcome` | `stamped` | `none` |
| `verifyOutcome` | `stamped` | `none` |
| `fields` | 不出現 | 不出現 |

#### Scenario: 帶著未結的工單封存

- **WHEN** 一個變更帶著未結的驗證工單被封存，執行 `speclink list --archived --json`
- **THEN** 它的 `verifyOutcome` 是 `notPassed`

#### Scenario: 沒有已封存的變更

- **WHEN** 專案沒有任何已封存的變更，執行 `speclink list --archived --json`
- **THEN** exit code 是 0，stdout 是 `{ "archived": [] }`

#### Scenario: 用清單的名稱讀文件

- **WHEN** 清單有一項 `datedName` 是 `2026-11-22-add-two`、`deltaCapabilities` 是 `["auth"]`，執行 `speclink artifact cat specs/auth --change 2026-11-22-add-two`
- **THEN** stdout 是該封存變更對 auth 的 delta spec 全文

#### Scenario: 不帶 --json 被拒絕

- **WHEN** 執行 `speclink list --archived`
- **THEN** 指令以非零 exit code 結束，stderr 指出 `--archived` 要和 `--json` 一起用，stdout 沒有輸出

#### Scenario: 與其他旗標並用被拒絕

- **WHEN** 分別執行 `speclink list --archived --json --specs`、`speclink list --archived --json --changes` 與 `speclink list --archived --json --detail`
- **THEN** 三次都以非零 exit code 結束，stderr 指出兩個旗標不能並用，stdout 沒有輸出

#### Scenario: 宿主套件以 invalid_argv 拒絕

- **WHEN** 經 `host.dispatch` 執行 `['list', '--archived']`
- **THEN** 呼叫以 Error 拒絕，`code` 是 `invalid_argv`，儲存沒有收到寫入

#### Scenario: 三種執行方式輸出相同

- **WHEN** 本機專案、一台 Server 與一個宿主套件的儲存含相同的已封存變更，分別執行 `speclink list --archived --json`、連到該 Server 執行同一個指令、以及經 `host.dispatch` 執行同一個 argv
- **THEN** 三個結果的 `archived` 深度相等

#### Scenario: 連到舊版 Server 時不失敗

- **WHEN** CLI 連到一台已封存清單的項目沒有 `deltaCapabilities` 的 Server，執行 `speclink list --archived --json`
- **THEN** 指令成功，每一項都沒有 `deltaCapabilities`，其餘的鍵照常輸出

#### Scenario: metadata 無法解析的已封存變更

- **WHEN** 已封存變更 `2026-11-01-broken` 的 metadata 無法解析，它有任務清單（2 個任務、1 個完成），執行 `speclink list --archived --json`
- **THEN** 指令成功；這一項有 `datedName`、`date`、`name`、`tasksTotal` 2、`tasksDone` 1，`reviewOutcome` 與 `verifyOutcome` 都是 `none`，沒有 `createdBy`、`fromDiscussions`、`fields`

#### Scenario: 不帶 --archived 時輸出不變

- **WHEN** 對同一個有已封存變更的專案執行 `speclink list` 與 `speclink list --json`
- **THEN** 兩個輸出與引入 `--archived` 之前逐位元相同

#### Scenario: 離線時與不帶 --archived 相同地失敗

- **WHEN** CLI 連到的 Server 無法連線，執行 `speclink list --archived --json`
- **THEN** exit code 與 stderr 訊息與執行 `speclink list --json` 時相同，stdout 沒有輸出

### Requirement: 關卡工單的詳細輸出

`speclink review show <change> --json` 與 `speclink verify show <change> --json` SHALL 接受旗標 `--detail`，不讀 stdin。帶上它時，輸出 SHALL 在既有的 `change`、`rounds`、`lastRound` 之外增列 `content`：字串，工單文件的全文，SHALL 與同一個動詞不帶 `--json` 時印到 stdout 的原文逐字相同。既有三個鍵的值 SHALL 與不帶 `--detail` 時相同。成功時 exit code SHALL 是 0。

不帶 `--detail` 的 `review show --json` 與 `verify show --json` SHALL 維持既有的位元級輸出，SHALL NOT 含 `content`。不帶 `--json` 的 `--detail` SHALL 以用法錯誤拒絕：CLI 以非零 exit code 結束、訊息走 stderr 並指出 `--json`、stdout 沒有輸出；經宿主套件執行時以 `invalid_argv` 拒絕。`--detail` SHALL NOT 改變人眼輸出，因為它不能脫離 `--json` 使用；`--no-color` 對 JSON 輸出沒有作用。輸出 SHALL 不隨 `locale`（`tw`、`ja`、`en` 或未設定）改變。這個旗標只讀取：SHALL NOT 寫入任何檔案。該變更沒有工單、或變更不存在時，指令 SHALL 與不帶 `--detail` 時相同地失敗（非零 exit code、既有的 stderr 訊息）。

三種執行方式對相同的工單 SHALL 輸出相同的鍵與值：本機專案、連到遠端的 CLI、宿主套件。連到遠端的 CLI SHALL 以既有的工單回應的 `content` 組出這個鍵，SHALL NOT 另外發請求。連到工單回應沒有 `content` 的舊版 Server 時，輸出 SHALL 沒有 `content` 鍵，指令 SHALL NOT 失敗。離線或認證失效時 SHALL 與不帶 `--detail` 的同一個動詞相同地失敗。這個動詞只讀取，沒有 revision 衝突的情況。

#### Scenario: 帶出審查工單的全文

- **WHEN** 變更 demo 的審查工單有兩輪，執行 `speclink review show demo --json --detail`
- **THEN** exit code 是 0；輸出有 `change`、`rounds`（長度 2）、`lastRound` 與 `content`；`content` 與 `speclink review show demo --no-color` 印出的原文逐字相同

#### Scenario: 驗證工單同樣帶出全文

- **WHEN** 變更 demo 的驗證工單有一輪，執行 `speclink verify show demo --json --detail`
- **THEN** 輸出的 `content` 是驗證工單的全文，`rounds` 長度是 1

#### Scenario: 不帶 --detail 時輸出不變

- **WHEN** 對同一張工單執行 `speclink review show demo --json`
- **THEN** 輸出與引入 `--detail` 之前逐位元相同，沒有 `content` 鍵

#### Scenario: --detail 不帶 --json 被拒絕

- **WHEN** 執行 `speclink review show demo --detail`
- **THEN** 指令以非零 exit code 結束，stderr 指出 `--detail` 要和 `--json` 一起用，stdout 沒有輸出

#### Scenario: 宿主套件讀得到工單全文

- **WHEN** 經 `host.dispatch` 執行 `['review', 'show', 'demo', '--json', '--detail']`
- **THEN** 回傳的物件有 `content`，值是 demo 的審查工單全文

#### Scenario: 沒有工單

- **WHEN** 變更 demo 沒有審查工單，執行 `speclink review show demo --json --detail`
- **THEN** exit code 非零，stderr 與執行 `speclink review show demo --json` 時相同，stdout 沒有輸出

#### Scenario: 三種執行方式輸出相同

- **WHEN** 本機專案、一台 Server 與一個宿主套件的儲存含相同的變更 demo 與相同的審查工單，分別執行 `speclink review show demo --json --detail`、連到該 Server 執行同一個指令、以及經 `host.dispatch` 執行同一個 argv
- **THEN** 三個結果深度相等

#### Scenario: 連到舊版 Server 時不失敗

- **WHEN** CLI 連到一台工單回應沒有 `content` 的 Server，執行 `speclink review show demo --json --detail`
- **THEN** 指令成功，輸出有 `change`、`rounds`、`lastRound`，沒有 `content`

## MODIFIED Requirements

### Requirement: 動詞 --json 輸出形狀凍結

<!-- BEFORE: 工單原文不得出現在任何 --json 輸出；討論的 --json 輸出沒有 promotedTo -->

動詞的 --json 輸出欄位集合與 camelCase 命名 SHALL 維持既有契約不變;工單原文 SHALL NOT 出現在不帶 `--detail` 的任何 --json 輸出；只有 `review show --json --detail` 與 `verify show --json --detail` SHALL 以 `content` 鍵帶出工單原文。manual-task-marker 系列的兩次刻意契約更新:instructions apply 的任務項增列 `manual` 欄位、progress 增列 `codeTotal`/`codeComplete`/`codeRemaining`;隨 `[P]` 語意移除,任務項的 `parallel` 欄位 SHALL 自基線移除——任務項欄位集合為 `id`/`description`/`done`/`manual`。另一次刻意契約更新:list 的 change 條目增列 `created` 與 `modifiedAt` 兩個可缺席的欄位。再一次刻意契約更新：`discuss list --json` 的每一項、`discuss show --json` 的 `info` 與 `discuss search --json` 的每一筆命中增列可缺席的 `promotedTo`，只在該討論有轉出或併入的變更時出現，依字母順序排在 `path` 與 `rounds` 之間。除上述刻意更新外 SHALL NOT 改名或移除既有欄位;更新後的欄位集合即凍結契約的新基線。

#### Scenario: 工單 --json 兩模式同形且無原文欄位

- **WHEN** 分別於本機與 remote 模式執行 review show --json
- **THEN** 兩模式 payload 欄位集合一致——change、rounds、lastRound,rounds 各項含 index、phase、patchHash、scope、findings——且不存在攜帶工單原文的欄位

#### Scenario: instructions apply 兩模式同形且無 parallel 欄位

- **WHEN** 分別於本機與 remote 模式對同一 change 執行 instructions apply --json
- **THEN** 兩模式任務項欄位集合皆為 id/description/done/manual(無 parallel),progress 皆含 total/complete/remaining 與 codeTotal/codeComplete/codeRemaining,欄位集合一致

#### Scenario: list 新增欄位不影響既有欄位

- **WHEN** 比對新增欄位前後 speclink list --json 對同一專案的輸出
- **THEN** 既有欄位的名稱、型別與值不變，差異只有每個 change 條目多出的 created 與 modifiedAt

#### Scenario: 帶 --detail 的工單 --json 才有原文

- **WHEN** 對同一張工單分別執行 review show --json 與 review show --json --detail
- **THEN** 前者沒有 `content` 鍵，輸出與引入 `--detail` 之前逐位元相同；後者多一個 `content` 鍵，其餘鍵與值和前者相同

#### Scenario: 討論的 promotedTo 不影響既有欄位

- **WHEN** 比對新增 `promotedTo` 前後 speclink discuss list --json 對同一個專案的輸出，專案裡有一份已轉出的討論與一份沒有轉出的討論
- **THEN** 沒有轉出的那一項逐位元相同；已轉出的那一項既有鍵的名稱、相對順序與值不變，差異只有多出的 `promotedTo`；對 speclink discuss search --json 的命中項作同樣的比對，結果相同

### Requirement: 清單的詳細輸出

<!-- BEFORE: --detail 只有 review、verify、claimed、branch、landed 五個鍵；進行中的關卡只有 state -->

`speclink list --json` SHALL 接受旗標 `--detail`。帶上它時，`changes` 的每一項 SHALL 在既有欄位之外增列下列選填的鍵，每個鍵 SHALL 只在該變更有對應的記錄時出現：

- `review`、`verify`：物件。形狀 SHALL 與 `speclink show <change> --json --detail` 的同名鍵相同，但 SHALL NOT 含 `scope`（`state` 是 `inProgress` 或 `stamped`，蓋章後另有蓋章的時間、輪數、任務數等既有欄位）。`state` 是 `inProgress` 時 SHALL 另有 `rounds`（整數）：工單目前的輪數；工單無法解析時 SHALL 不輸出 `rounds`，清單 SHALL NOT 因此失敗。該關卡沒有工單也沒有章時 SHALL 不輸出這個鍵。
- `claimed`：物件，`by`（字串）、有記錄才出現的 `byId`（字串）與 `at`（字串）。形狀 SHALL 與 `show --json` 的 `claimed` 相同。沒有被認領時 SHALL 不輸出。
- `branch`：字串，開工時記下的分支。沒有記錄時 SHALL 不輸出。
- `landed`：物件，只有 `commit`（字串）。沒有落地記錄時 SHALL 不輸出。
- `createdBy`：字串，metadata 記下的建立者。沒有記錄時 SHALL 不輸出。
- `fromDiscussions`：字串陣列，這個變更的來源討論的 slug，順序 SHALL 與 `show --json` 的 `fromDiscussions` 相同。沒有來源討論時 SHALL 不輸出。

鍵名一律 camelCase。清單的既有欄位、項目的順序與 `specs` 區段 SHALL 不受 `--detail` 影響；metadata 損壞的變更 SHALL 照既有行為列出（帶 `metaError`），且 SHALL NOT 帶上述任何一個鍵。

不帶 `--detail` 的 `speclink list --json` SHALL 維持既有的位元級輸出。不帶 `--json` 的 `--detail`、以及 `--detail` 與 `--specs` 並用，SHALL 以用法錯誤拒絕：CLI 以非零 exit code 結束、訊息走 stderr、stdout 沒有輸出；經宿主套件執行時以 `invalid_argv` 拒絕。`--detail` SHALL NOT 改變人眼輸出，因為它不能脫離 `--json` 使用。輸出 SHALL 不隨 `locale`（`tw`、`ja`、`en` 或未設定）改變。

三種執行方式對相同的資料 SHALL 輸出相同的鍵與值：本機專案、連到遠端的 CLI、宿主套件。連到遠端的 CLI SHALL 以既有的變更清單回應組出這些鍵，SHALL NOT 對每個變更另外發一次請求。連到不提供這些欄位的舊版 Server 時，對應的鍵 SHALL 不出現，指令 SHALL NOT 失敗。離線或認證失效時 SHALL 與不帶 `--detail` 的 `list` 相同地失敗。本機專案沒有認領，所以 `claimed` 在本機 SHALL 不出現（除非 metadata 本來就帶著認領欄位）。

蓋章之後檔案是否又被改過，SHALL NOT 由清單判斷：`review` 與 `verify` 只反映記錄下來的狀態，與 `show --json` 相同。

`--detail` 與 `--archived` 並用 SHALL 以用法錯誤拒絕（見「已封存變更的清單」）。

#### Scenario: 有記錄的變更帶出四類狀態

- **WHEN** 變更 demo 的審查已蓋章、驗證有未結的工單、由 Alice（識別 u_1）認領、開工時記下分支 `speclink/demo`、落地的 commit 是 `0123456789abcdef0123456789abcdef01234567`，執行 `speclink list --json --detail`
- **THEN** demo 這一項含 `review`（`state` 是 `stamped`）、`verify`（`state` 是 `inProgress`）、`claimed`（`by` 是 `Alice`、`byId` 是 `u_1`、有 `at`）、`branch`（`speclink/demo`）、`landed`（`{ "commit": "0123456789abcdef0123456789abcdef01234567" }`），既有的欄位與不帶 `--detail` 時相同

##### Example: 各種記錄對應的鍵

| 變更的記錄 | 多出的鍵 |
| --- | --- |
| 什麼記錄都沒有 | 沒有任何新鍵；這一項與不帶 `--detail` 時相同 |
| 只有審查章 | `review` |
| 審查工單未結（兩輪）、沒有章 | `review`（`state` 是 `inProgress`、`rounds` 是 2） |
| metadata 記下建立者 `Ann <ann@example.com>` | `createdBy` |
| 由討論 `two` 轉出 | `fromDiscussions`（`["two"]`） |
| 被認領、沒有識別 | `claimed`（只有 `by` 與 `at`） |
| 記下分支、還沒落地 | `branch` |
| 已落地 | `branch`（有記錄時）、`landed` |

#### Scenario: 不帶 --detail 時輸出不變

- **WHEN** 對同一個專案執行 `speclink list --json`，專案裡有已蓋章、被認領、已落地的變更
- **THEN** 輸出與引入 `--detail` 之前逐位元相同，任何一項都沒有 `review`、`verify`、`claimed`、`branch`、`landed`、`createdBy`、`fromDiscussions`

#### Scenario: --detail 不帶 --json 被拒絕

- **WHEN** 執行 `speclink list --detail`
- **THEN** 指令以非零 exit code 結束，stderr 指出 `--detail` 要和 `--json` 一起用，stdout 沒有清單

#### Scenario: --detail 與 --specs 並用被拒絕

- **WHEN** 執行 `speclink list --specs --json --detail`
- **THEN** 指令以非零 exit code 結束，stderr 指出兩個旗標不能並用

#### Scenario: 三種執行方式輸出相同

- **WHEN** 本機專案、一台 Server 與一個宿主套件的儲存含相同的變更與相同的記錄（不含認領），分別執行 `speclink list --json --detail`、連到該 Server 執行同一個指令、以及經宿主套件執行同一個 argv
- **THEN** 三個結果的 `changes` 深度相等（本機專案才有的 `worktree` 鍵除外）

#### Scenario: 連到舊版 Server 時不失敗

- **WHEN** CLI 連到一台變更清單回應沒有關卡、分支與落地欄位的 Server，執行 `speclink list --json --detail`
- **THEN** 指令成功，每一項都沒有新鍵

#### Scenario: metadata 損壞的變更不帶新鍵

- **WHEN** 專案含一筆 metadata 損壞的變更，執行 `speclink list --json --detail`
- **THEN** 該項照既有行為帶 `metaError`，沒有 `review`、`verify`、`claimed`、`branch`、`landed`、`createdBy`、`fromDiscussions`；指令成功

#### Scenario: 帶出建立的人與來源討論

- **WHEN** 變更 add-two 的 metadata 記下建立者 `Ann <ann@example.com>`，它由討論 two 轉出；變更 solo 沒有建立者記錄也沒有來源討論；執行 `speclink list --json --detail`
- **THEN** add-two 這一項含 `"createdBy": "Ann <ann@example.com>"` 與 `"fromDiscussions": ["two"]`；solo 這一項沒有這兩個鍵

#### Scenario: 進行中的關卡帶輪數

- **WHEN** 變更 demo 的審查工單有兩輪、沒有章，執行 `speclink list --json --detail`
- **THEN** demo 這一項的 `review` 是 `{ "state": "inProgress", "rounds": 2 }`

#### Scenario: 舊版 Server 沒有回報進行中的輪數

- **WHEN** CLI 連到一台關卡物件在 `inProgress` 時沒有 `rounds` 的 Server，demo 的審查工單未結，執行 `speclink list --json --detail`
- **THEN** 指令成功，demo 這一項的 `review` 是 `{ "state": "inProgress" }`

### Requirement: 單筆變更的詳細輸出

<!-- BEFORE: --detail 對進行中的關卡沒有作用，輸出只有 state -->

`speclink show <change> --json` SHALL 接受旗標 `--detail`，不讀 stdin。帶上它時，輸出的 `review` 與 `verify` 物件在 `state` 為 `stamped` 時 SHALL 在既有欄位之外增列 `scope`：陣列，每一項是 `{ "path": 字串, "hash": 字串 }`，`path` 是蓋章時記下的 repo 相對路徑（以 `/` 分隔），`hash` 是蓋章時記下的內容指紋（小寫十六進位），順序 SHALL 與蓋章時記錄的順序相同。已蓋章但沒有記下任何指紋的關卡，`scope` SHALL 是空陣列。`scope` 的鍵名與每一項的鍵名 SHALL 與線上格式單筆變更回應的 `scope` 相同。`state` 為 `inProgress` 的關卡 SHALL 在 `state` 之外增列 `rounds`（整數）：工單目前的輪數；工單無法解析時 SHALL 不輸出 `rounds`，指令 SHALL NOT 因此失敗。沒有工單也沒有章而不輸出的關卡 SHALL 與不帶 `--detail` 時相同。`show` 的其他欄位 SHALL 不受 `--detail` 影響；`show` 的對象是正式規格時（`--item-type spec`，或名稱解析成正式規格），輸出 SHALL 與不帶 `--detail` 時相同。

不帶 `--detail` 的 `speclink show <change> --json` SHALL 維持既有的位元級輸出，SHALL NOT 含 `scope`，進行中的關卡 SHALL NOT 含 `rounds`。`speclink list --json --detail` SHALL NOT 含 `scope`。不帶 `--json` 的 `--detail` SHALL 以用法錯誤拒絕：CLI 以非零 exit code 結束、訊息走 stderr 並指出 `--json`、stdout 沒有輸出；經宿主套件執行（`host.dispatch`、掛鉤的 `read`、Node 的 `engine.dispatch`）時以 `invalid_argv` 拒絕。成功時 exit code SHALL 是 0。`--detail` SHALL NOT 改變人眼輸出，因為它不能脫離 `--json` 使用；`--no-color` 對 JSON 輸出沒有作用。輸出 SHALL 不隨 `locale`（`tw`、`ja`、`en` 或未設定）改變。這個旗標 SHALL NOT 寫入任何檔案，`.meta.yaml` 與 `.speclink/` 底下的檔案 SHALL 不變。

三種執行方式對相同的資料 SHALL 輸出相同的 `review` 與 `verify`：本機專案、連到遠端的 CLI、宿主套件。連到遠端的 CLI SHALL 以既有的單筆變更回應組出 `scope` 與進行中的 `rounds`，SHALL NOT 另外發請求。連到不回報關卡欄位的舊版 Server 時，`review` 與 `verify` SHALL 不出現，指令 SHALL NOT 失敗；Server 的進行中關卡沒有 `rounds` 時，輸出的進行中關卡 SHALL 沒有 `rounds`。離線或認證失效時 SHALL 與不帶 `--detail` 的 `show` 相同地失敗。這個動詞只讀取，沒有 revision 衝突的情況。

蓋章之後檔案是否又被改過，SHALL NOT 由這個輸出判斷：`scope` 只是蓋章時記下的指紋，比對由呼叫端自己做。

#### Scenario: 已蓋章的關卡帶出指紋

- **WHEN** 變更 demo 的審查已蓋章，章記下 `src/a.rs` 與 `src/b.rs` 兩個檔的指紋；驗證有未結的工單，工單有兩輪；執行 `speclink show demo --json --detail`
- **THEN** exit code 是 0；`review` 含 `state` `stamped`、`stampedAt`、`rounds`、`tasksTotal` 與 `scope`，`scope` 依序是 `src/a.rs` 與 `src/b.rs` 兩項，各有 `path` 與 `hash`；`verify` 是 `{ "state": "inProgress", "rounds": 2 }`，沒有 `scope`

##### Example: 各種關卡記錄對應的輸出

| 關卡的記錄 | 不帶 `--detail` | 帶 `--detail` |
| --- | --- | --- |
| 沒有工單也沒有章 | 不輸出這個關卡 | 不輸出這個關卡 |
| 工單未結（兩輪） | `{ "state": "inProgress" }` | `{ "state": "inProgress", "rounds": 2 }` |
| 已蓋章，記下兩個檔 | 既有欄位，沒有 `scope` | 既有欄位加 `scope`（兩項） |
| 已蓋章，沒有記下任何指紋 | 既有欄位，沒有 `scope` | 既有欄位加 `"scope": []` |

#### Scenario: 不帶 --detail 時輸出不變

- **WHEN** 對同一個已蓋章的 demo 執行 `speclink show demo --json`
- **THEN** 輸出與引入 `--detail` 之前逐位元相同，`review` 與 `verify` 都沒有 `scope`

#### Scenario: 清單的詳細輸出不帶指紋

- **WHEN** 對含已蓋章的 demo 的專案執行 `speclink list --json --detail`
- **THEN** demo 這一項的 `review` 沒有 `scope`

#### Scenario: --detail 不帶 --json 被拒絕

- **WHEN** 執行 `speclink show demo --detail`
- **THEN** 指令以非零 exit code 結束，stderr 指出 `--detail` 要和 `--json` 一起用，stdout 沒有輸出

#### Scenario: 宿主套件以 invalid_argv 拒絕

- **WHEN** 經 `host.dispatch` 執行 `['show', 'demo', '--detail']`，以及在 `beforeCommand` 裡執行 `read(['show', 'demo', '--detail'])`
- **THEN** 兩者都以 Error 拒絕，`code` 是 `invalid_argv`，儲存沒有收到寫入

#### Scenario: 正式規格不受影響

- **WHEN** 對正式規格 auth 執行 `speclink show auth --item-type spec --json --detail`
- **THEN** 輸出與 `speclink show auth --item-type spec --json` 相同

#### Scenario: 三種執行方式輸出相同

- **WHEN** 本機專案、一台 Server 與一個宿主套件的儲存含相同的變更 demo 與相同的審查章，分別執行 `speclink show demo --json --detail`、連到該 Server 執行同一個指令、以及經 `host.dispatch` 執行同一個 argv
- **THEN** 三個結果的 `review` 深度相等，都含相同的 `scope`

#### Scenario: 連到舊版 Server 時不失敗

- **WHEN** CLI 連到一台單筆變更回應沒有關卡欄位的 Server，執行 `speclink show demo --json --detail`
- **THEN** 指令成功，輸出沒有 `review` 與 `verify`

#### Scenario: 離線時與不帶 --detail 相同地失敗

- **WHEN** CLI 連到的 Server 無法連線，執行 `speclink show demo --json --detail`
- **THEN** exit code 與 stderr 訊息與執行 `speclink show demo --json` 時相同，stdout 沒有輸出

#### Scenario: 不帶 --detail 時進行中的關卡沒有輪數

- **WHEN** 變更 demo 的驗證工單有兩輪、沒有章，執行 `speclink show demo --json`
- **THEN** `verify` 是 `{ "state": "inProgress" }`，輸出與引入進行中的 `rounds` 之前逐位元相同

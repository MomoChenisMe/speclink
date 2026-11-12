## Context

- 技能檔的產生全部經引擎的同一個計畫：依工具清單（`.speclink.yaml` 的 `tools`，內建工具與自訂描述子）與技能登錄表，算出每個工具要寫的檔案（`<技能資料夾>/speclink-<名稱>/SKILL.md`），寫入並清掉多出來的 `speclink-*` 資料夾。登錄表的兩個過濾條件：只給 Claude 的技能、要 worktree 政策開啟才有的兩支。worktree 的值是直接讀本機規格資料夾的 `config.yaml`。
- 每個技能檔的 frontmatter 有版本欄位，值是引擎的技能範本版號（和 CLI `--version` 顯示的「engine」相同）。沒有清單檔。
- 降級守門：要寫的檔案裡有任何一個版號比引擎的新，就拒絕；`speclink update --allow-downgrade` 是唯一的越過方式。
- 探測（唯讀）：比對 checkout 裡的技能檔與「現在會產生的內容」，回報缺少、過期、較新、現版、無法判定，並列出內容不同的檔案。Speclink Desktop 只對本機專案呼叫它。
- 握手回應的 `engineVersion` 是套件版本，不是技能範本的版號；沒有用戶端讀它。
- 上一個變更讓渲染多了「規格資料夾在不在 checkout」的軸；遠端專案用「不在」。
- 宿主套件的 HTTP 入口有一張靜態的路由表；唯讀的 POST 要列進一張清單才不受寫入能力限制。

限制：

- 討論的擴充規則：宿主只能選擇提供哪幾支技能，不能改文字。
- 技能檔會進版本控制，不能因人而異。
- 安裝後是 checkout 裡的檔案；agent 不在執行期向 Server 取技能。

## Goals / Non-Goals

**Goals:**

- 遠端專案的技能檔，內容與集合由 Server 上的引擎決定。
- 用戶端知道 Server 的技能版本，並在自己比較舊時提醒使用者。
- Speclink Desktop 對遠端專案的技能檔有和本機專案相同的提示。
- 舊版的 Server 與舊版的用戶端互相連線時都照常運作。

**Non-Goals:**

- 改文字、加技能、簽章、自我更新、本機專案、因人而異。

## Decisions

### D1: 比對哪一個版本

用戶端與 Server 之間比的是**技能範本的版號**（`ASSET_VERSION`），不是套件版本。理由：技能檔的 frontmatter 記的就是它；它只在技能文字或技能用到的動詞改變時前進，正好是「兩邊對不對得上」的訊號。

握手的 `capabilities` 新增選填的 `skills: { version }`。`engineVersion` 的意義不變。

### D2: 線上的路由

`POST /projects/{key}/skills`，唯讀（列進唯讀 POST 的清單；讀得到這個範圍的人都能用）。

請求（camelCase）：

```json
{ "targets": [ { "tool": "claude" }, { "tool": "my-agent", "custom": { "invocation": "tool-call" } } ], "contents": true }
```

- `tool`：內建工具的名稱，或自訂工具的名稱。帶 `custom` 時當成自訂工具（中性渲染，`invocation` 是 `cli` 或 `tool-call`）；不帶時必須是內建工具的名稱。
- `contents`：省略或 `false` 時只回摘要；`true` 時多回全文。

回應：

```json
{ "version": "v1.58.0",
  "targets": [ { "tool": "claude", "skills": [ { "name": "apply", "digest": "sha256:…", "content": "…" } ] } ] }
```

- `skills` 依名稱排序；`digest` 是內容把換行正規化成 LF 之後的 SHA-256；`content` 只在要求全文時出現。
- 技能要寫到哪個資料夾由用戶端決定（它知道每個工具的技能資料夾），所以回應不帶路徑。

錯誤：`targets` 是空的、內建工具的名稱不認得、`invocation` 不是兩個值之一 → 400 `invalid_argument`。

`speclink-protocol` 新增 `skills.rs` 放這些型別。線上歸屬表不受影響：這條路由不執行引擎命令。

- **否決：用 GET 與查詢參數** — 自訂工具的描述子放不進查詢字串；Context 快照已經有唯讀 POST 的先例。
- **否決：回應帶檔案路徑** — 自訂工具的技能資料夾是用戶端的設定，Server 不該知道也不該決定寫到哪裡。

### D3: Server 產生哪些技能

入口對每個目標，從技能登錄表依序過濾：

1. 目標不是 Claude 時，去掉只給 Claude 的技能。
2. 讀這個範圍的流程設定（鍵 `config.yaml`，一次讀取）：`worktree` 不是真時，去掉兩支 worktree 的技能。設定不存在時當成預設值；設定無法解析時保留這兩支（和本機既有的規則相同）。環境變數不參與。
3. 宿主有給技能清單時，只留清單裡的。

留下的每一支用引擎的渲染函式產生，「規格資料夾在不在 checkout」是「不在」，規格資料夾名稱用預設值（「不在」的渲染不會出現它）。

過濾的前兩步是引擎既有的函式（今天只在 crate 內可見）；把它改成公開並多收一個「worktree 是否開啟」的值，本機與 Server 共用，不寫第二份。

建立宿主套件時的新輸入：`HostKitOptions.skills: Option<Vec<String>>`。有不在登錄表裡的名稱時，建立回報錯誤並列出不認得的名稱。握手只在入口能回答這條路由時（有範圍函式）宣告 `skills`。

### D4: 用戶端取得並安裝

`speclink-remote` 新增 `skills.rs`：

- `fetch(client, targets, contents) -> RemoteSkills`：呼叫 D2 的路由。
- `targets_of(app_config) -> Vec<Target>`：把 `.speclink.yaml` 的工具清單轉成請求的目標，規則和本機的工具解析相同。

`speclink-core` 的同步計畫新增一個建構方式：`SyncPlan::from_rendered(root, tools, rendered)`，檔案集合來自外部給的內容而不是本機的渲染。寫入、清掉多出來的資料夾、移除被取消選取的工具的足跡，都沿用同一個 `apply`。寫入順序與今天相同：逐檔寫入，最後清理；中途失敗時已寫的檔案留著，重跑會收斂（既有行為，技能檔沒有跨檔的一致性要求）。

`speclink update` 在遠端專案的步驟：

1. 讀 `.speclink.yaml`，解析工具。
2. 握手。沒有憑證、連不上、或 Server 沒有宣告 `skills` → 走本機的範本（以「不在」渲染，沿用降級守門），stderr 印一行說明原因。
3. 有宣告 → 取得全文，建計畫，套用。這條路徑**不經降級守門**：遠端專案的技能版本跟著 Server。Server 的版本比 checkout 裡既有的舊時，stdout 的摘要多一行說明「技能已換成 Server 提供的版本」。
4. 摘要印出來源（`server` 或 `local`）與版本。

第 3 步取得失敗（握手成功之後路由回錯誤）→ 非 0，不寫任何檔案，不退回本機的範本：這時 Server 明確說它提供技能，退回本機會默默裝上可能對不上的內容。

`speclink init --store remote` 不變：這時常常還沒登入。它的結尾多一行提示。Speclink Desktop 綁定 checkout 時的工具同步也不變；之後由 D6 的提示帶使用者更新。

- **否決：遠端專案的 `update` 永遠需要連線** — 舊版 Server 與離線時仍要能產生技能。
- **否決：保留降級守門並要求 `--allow-downgrade`** — 在遠端專案，「checkout 比 Server 新」代表技能可能用到 Server 沒有的動詞，換成 Server 的版本才是正確的方向。

### D5: CLI 的提醒

每次遠端動詞握手之後：Server 有宣告 `skills.version`，而且本機的技能範本版號在數值上比它舊（比較規則用引擎既有的版號比較；任一邊無法解析就不提醒）→ 在 stderr 印一行：

```
speclink: this CLI's skill templates (v1.57.0) are older than this server's (v1.58.0) — update Speclink: <更新方式>
```

- 頻率：同一個 Server 位址一天最多一次。狀態記在使用者設定目錄的 `skill-notice.json`（位址對日期）。檔案讀寫失敗時不提醒也不報錯。
- 「更新方式」沿用 CLI 既有的更新指引文字（和技能檔過期警告用的同一段），不新增第二份。
- 不影響 stdout、`--json` 與 exit code。`speclink update` 自己的執行不印這一行（它的摘要已經有版本）。
- 提醒的邏輯放在 CLI 的 `skill_notice.rs`，由建立遠端連線語境的單一入口呼叫，所以每個遠端動詞都經過它，不用各自呼叫。

### D6: Speclink Desktop

- `speclink-desktop-core` 新增 `probe_remote_assets_at(checkout_root, remote_skills)`：把 Server 回的摘要和 checkout 裡各工具的技能檔比對（讀檔、換行正規化、算摘要），回報和本機探測相同形狀的結果：
  - 任一工具的技能資料夾沒有任何 `speclink-` 技能 → 缺少。
  - 有任何一支的摘要不同、少了、或多了 → 過期，並列出這些檔案。
  - 全部相同 → 現版。
  - 另外帶兩個值：Server 的版本，以及「Speclink Desktop 內建的技能版號比 Server 舊」這個布林。
- Tauri 殼：`probe_remote_assets(connection, scope, checkout)` 取得摘要後委派給上面的函式；`update_remote_assets(...)` 取得全文後用 D4 的同一個計畫套用。兩個 command 各只做委派。
- 前端：作用中的分頁是「有 checkout 的遠端專案」時，改呼叫遠端的探測；提示卡、略過的記憶、其他分頁的琥珀點都沿用既有的元件與邏輯（`assetPrompt.ts` 與 `AssetNoticeDialog`），不新增元件。略過的鍵改成「專案加 Server 的版本」。
- 確認框的列：這個專案的技能檔版本、**Server 提供的技能版本**（遠端專案才有）、你的 Speclink 版本、會改寫的檔案數。
- 按鈕：缺少 →「安裝技能檔」；過期 →「更新技能檔」。Speclink Desktop 比 Server 舊時，確認框多一句說明並多一個「更新 Speclink」的按鈕，原本的安裝或更新按鈕仍然可用（Server 的技能是對的，只是本機的 CLI 可能認不得新動詞）。
- Server 沒有宣告技能、或沒有 checkout → 不探測，沒有提示（今天的行為）。
- 探測的時機和本機專案相同（分頁成為作用中、更新完成後、收到變動事件）。摘要的請求約幾 KB。

### D7: Node 與示範用 server

- `createHost({ skills?: string[] })`：傳給 `HostKitOptions.skills`。有不認得的名稱時 `createHost` 同步拋出錯誤並列出名稱。
- 示範用 server：不給清單（提供全部）。

## Implementation Contract

**行為**

- 新版 CLI 連到宣告技能的 Server，在遠端專案執行 `speclink update`：checkout 裡每個工具的技能檔內容等於 Server 路由回的內容；Server 的流程設定開啟 worktree 時有那兩支，關閉時沒有（既有的會被清掉）。
- 連到沒有宣告的 Server：結果和這個變更之前相同，stderr 多一行說明。
- 本機 CLI 的技能版號比 Server 舊：遠端動詞照常成功，stderr 有一行提醒；同一天第二次執行沒有提醒。
- Speclink Desktop 開啟有 checkout 的遠端專案：技能和 Server 不同時出現提示卡；按「更新技能檔」後提示消失，checkout 的技能檔等於 Server 的內容。
- 宿主以 `skills: ['discuss', 'propose']` 建立宿主套件：路由只回這兩支。

**介面與資料形狀**

- 線上：`capabilities.skills.version`；`POST /projects/{key}/skills` 的請求與回應（D2）。
- Rust：`HostKitOptions.skills`；`speclink_remote::skills::{fetch, targets_of}`；`SyncPlan::from_rendered`；引擎公開的「某個目標要產生哪些技能」函式。
- Node：`CreateHostOptions.skills`。
- Tauri：`probe_remote_assets`、`update_remote_assets`。
- CLI 的狀態檔：使用者設定目錄的 `skill-notice.json`。

**失敗的樣子**

| 情況 | 結果 |
|---|---|
| 遠端專案 `update`，沒有憑證或連不上 | 用本機的範本，stderr 說明，exit code 0 |
| 握手宣告了技能，但路由回錯誤 | 非 0，不寫任何檔案 |
| 請求的內建工具名稱不認得 | 400 `invalid_argument` |
| 宿主的技能清單有不認得的名稱 | 建立宿主套件失敗，列出名稱 |
| 提醒的狀態檔不能讀寫 | 不提醒，不報錯 |
| Desktop 探測時連不上 | 沒有提示（無法判定），不記入略過 |
| `.speclink.yaml` 的自訂描述子無效 | 和本機相同：`update` 報錯；探測略過它 |

**驗收**

- `cargo test -p speclink-host --test http_door`：握手的宣告、路由的過濾（工具、worktree、宿主的清單）、摘要與全文、錯誤。
- `cargo test -p speclink-core`：`from_rendered` 的寫入與清理；公開的過濾函式與本機產生的集合一致；既有的 golden 不變。
- `cargo test -p speclink-remote`：`fetch` 與 `targets_of`。
- `cargo test -p speclink-cli --test it remote_skills::`：上面「行為」的前三項與失敗的前兩項。
- `cargo test -p speclink-desktop-core`、`cargo test -p speclink-desktop`、`npm test -w apps/desktop`：探測、更新、提示與確認框。
- Node：`skills` 的過濾與不認得的名稱。

**範圍**

- 在範圍內：上面列的介面。
- 不在範圍內：技能的原稿、本機專案的產生、使用者文件。

## Risks / Trade-offs

- [回歸對照] 本機專案的產生不能變。→ 過濾函式只是改成公開並多一個輸入；既有的 golden 與 CLI 測試不改斷言。
- [回歸對照] 握手多一個欄位。→ 選填且有預設值；`speclink-remote` 與桌面的既有測試不改就要通過，另加一個「舊型別解析新回應」的測試。
- [信任] Server 能決定寫進 checkout 的技能文字，而技能會指示 agent 執行指令。→ 內容只能來自引擎內建的範本（宿主套件不接受自訂文字）；但自己改過程式的 Server 仍然能回任何內容。這是連到一台 Server 的固有信任，寫進文件，不在這裡做簽章。
- [跨平台] 摘要要在三個平台對同一份內容得到同一個值。→ 一律先把換行正規化成 LF；測試放 CRLF 的檔案。
- [技能檔寫到一半] 逐檔寫入中途失敗。→ 既有行為；重跑會收斂。
- [提醒太吵或太安靜] 一天一次是討論定的頻率。→ 狀態檔按 Server 位址記；測試用可注入的日期。
- [儲存解耦] 產生技能不再需要本機的規格資料夾；流程設定從持有它的地方讀。

## Migration Plan

沒有資料搬移。升級後，遠端專案的使用者執行一次 `speclink update`（或在 Speclink Desktop 按提示）。要退回時還原這個變更的 commit；checkout 裡的技能檔仍然是合法的技能檔。

## Open Questions

- `speclink init --store remote` 在已經有憑證時要不要直接向 Server 取技能。目前定為不要（一條路徑比較好懂）；有人反映第一次要多跑一次 `update` 太麻煩時再改。
- 自訂工具的 `toolName`（tool-call 形式裡工具的名稱）要不要放進請求。實作時對照中性渲染實際用到的輸入決定；用到就加進 `custom`。

## Why

技能要在「規格不在 checkout 裡」的形態下執行，每一個讀取都要有動詞可用。第一波（discuss、propose、ingest）用到的動詞本來就存在。盤點其餘技能後，發現有五種讀取沒有動詞，或有動詞但在遠端被擋下：

- **讀 evidence。** evidence 是每個任務完成時記下的「改了哪些檔」。commit、worktree 開工前與 improve 三支技能直接讀 `.evidence.json` 這個檔。沒有任何動詞或引擎命令能讀它；線上有一條路由，但 CLI 沒有用它，而且它讀不到已封存的變更。
- **`speclink trace` 在遠端被拒絕。** 拒絕的訊息說溯源鏈要從本機的規格資料夾組裝。查證後發現引擎的溯源只讀儲存（正式規格、已封存的變更、討論、已封存的 evidence），完全不碰 git；用到 git 的是技能，不是動詞。所以它在遠端被擋只是當初的範圍限制。
- **`speclink plan --strict-overlap` 在遠端被拒絕。** 它的計算也不需要任何本機事實。
- **已封存變更的文件讀不到。** trace 技能要讀已封存變更的提案，今天是從溯源結果的封存資料夾名稱去開檔。`speclink artifact cat` 只認進行中的變更。
- **變更閒置多久查不到。** apply 與 drift 兩支技能判斷「這個變更是不是放太久」時，用 `git log` 看規格資料夾的 commit 時間。變更的清單沒有帶出建立時間與最後修改時間。

這個變更把這五種讀取補齊。它只動引擎、線上與各入口，不動任何技能的文字；技能在下一個變更 skills-verbs-second-wave 改用它們。

- **目標使用者**：連到遠端專案的開發者；在宿主的 AI Agent 裡工作的 PO、PM。間接：下一個變更的技能。
- **使用情境**：apply、commit、archive、improve、trace 這幾支技能執行時的讀取步驟；開發者直接執行 `speclink trace` 與 `speclink plan --strict-overlap`。
- **既有規格的對照**：verb-contract 把 trace 列為本質本機動詞、把 `plan --strict-overlap` 列為只限本機的子情形，並把它的遠端拒絕列在兩模式輸出的分歧清單。verify-evidence 規定 evidence 怎麼寫入，沒有規定讀取的動詞。server-verb-api 規定 evidence 的唯讀端點，只涵蓋進行中的變更。trace-verb 規定溯源鏈的內容與 `--json` 的形狀，沒有談遠端。client-protocol 的變更清單沒有最後修改時間。

Source discussion: embedded-host-needs（cut 11；提案時從原本的 skills-verbs-second-wave 拆出）。

## What Changes

**新動詞：`speclink evidence show <change>`**

- 旗標：`--json`、`--no-color`。沒有 stdin。
- `<change>` 先找進行中的變更；沒有時找已封存的變更（可以給帶日期的封存名稱，也可以只給變更名稱，後者取最新的一筆）。
- 人眼輸出（stdout）：第一行是變更名稱與記錄的任務數，之後每個任務一段——任務編號與說明、記錄時間、它改到的檔案。沒有任何記錄時印一行說明，exit code 0。
- `--json`（stdout）：`change`（字串）、`archivedAs`（字串，只有讀到的是已封存的變更時才有）、`entries`（陣列，每筆 `taskId`、`taskDesc`、`touchedFiles`、`recordedAt`，以及有值才出現的 `actor`、`repo`、`headCommit`）、`files`（字串陣列：全部任務改到的檔案，去重複並排序）。
- 找不到這個名稱（進行中與已封存都沒有）：exit code 非 0，stderr 是 `not_found` 的訊息。
- 引擎新增命令 `EvidenceShow`。它是查詢，不產生事件。

**`speclink trace` 與 `speclink plan --strict-overlap` 在遠端可用**

- 線上新增兩條唯讀的路由：某個 capability 的溯源鏈；帶嚴格重疊旗標的執行計畫。
- CLI 的遠端分支改成送出請求，再用和本機相同的輸出函式印出。stdout、`--json` 的形狀與 exit code 和本機相同。
- 兩個動詞在本機的行為不變。

**`speclink artifact cat` 讀得到已封存的變更**

- `--change` 給的名稱沒有對應的進行中變更時，改找已封存的變更（規則和 `evidence show` 相同：封存名稱或原本的名稱，取最新的一筆）。旗標與輸出不變。
- 對進行中的變更，行為與輸出逐位元不變。

**變更的清單帶出時間**

- `speclink list --json` 的每個變更多兩個欄位：`created`（字串，建立日期）與 `modifiedAt`（字串，這個變更底下的文件最後一次被修改的時間，UTC）。儲存答不出最後修改時間時不輸出 `modifiedAt`。
- 線上的變更清單項多一個選填欄位 `modifiedAt`（`created` 已經有）。
- 人眼輸出不變。

**各入口**

- `speclink-core`：命令 `EvidenceShow` 與它的結果；清單項的兩個欄位。
- `speclink-verbs`：新的動詞家族 `evidence`，以及它的輸出。
- `speclink-host`：宿主套件的 HTTP 入口新增兩條路由，evidence 的路由改成執行引擎命令（因此也讀得到已封存的變更）；每個引擎命令的線上歸屬表跟著更新（溯源、嚴格重疊、讀 evidence 三個都對到路由）。
- `speclink-protocol`、`speclink-remote`：溯源回應的型別與呼叫；evidence 回應新增 `archivedAs` 與 `change`；清單項新增 `modifiedAt`；plan 回應新增只在嚴格重疊時出現的 `strictOverlap`。
- `speclink-cli`：三個動詞的遠端分支。
- `speclink-node`：`dispatch` 與 `host.dispatch` 經共用的解析與輸出自動得到 `evidence show`。

**相容性影響**

- `speclink trace` 在遠端：從「非 0 並說明只限本機」變成「執行並輸出」。
- `speclink plan --strict-overlap` 在遠端：同上。兩句拒絕的訊息不再出現。
- `speclink artifact cat --change <已封存的名稱>`：從「找不到變更」變成輸出該文件。
- `speclink list --json`：每個變更多了 `created` 與 `modifiedAt`。既有欄位不變。這是刻意的契約更新，同批更新 golden。
- evidence 的線上路由：對「進行中與已封存都沒有」的名稱，從回空集合變成 404 `not_found`。變更存在但沒有記錄時仍然回空集合。
- 連到舊版 Server 的新版 CLI：溯源的路由不存在，回 404，CLI 顯示既有的找不到訊息並以非 0 結束。`plan --strict-overlap` 的情況不同——舊版 Server 會忽略不認得的查詢參數、回一般的計畫，所以嚴格重疊的回應多帶一個標記 `strictOverlap: true`，CLI 沒收到標記時以非 0 結束並說明這個 Server 不支援，不會把一般的計畫當成嚴格重疊印出來。`evidence show` 同理：新路由的回應一律帶 `change`，CLI 沒看到它時以非 0 結束並說明這個 Server 不支援，不會把舊路由的空集合印成「沒有記錄」。
- 變更名稱必須是單一路徑段：含 `/`、`\` 的名稱，以及 `.`、`..`、空字串與 `archive`，對每個以名稱找變更的動詞都一律是找不到。在這之前，本機的 `artifact cat --change archive/<封存名稱>` 會意外讀到已封存的文件；現在改用變更名稱或封存名稱本身。
- 示範用 server：在舊的儲存層移除之前，它的儲存答不出最後修改時間，所以遠端的清單不帶 `modifiedAt`。
- **設定欄位**：沒有新增或變更。
- **影響的技能**：沒有。技能的文字在下一個變更才改。

## Non-Goals

- **改任何技能的文字**：變更 skills-verbs-second-wave。
- **溯源鏈納入進行中的變更，或由引擎讀 git 歷史**：溯源的內容不變。
- **寫入或修改 evidence 的動詞**：evidence 只由 `task done` 寫入。
- **宿主套件替宿主收集「改了哪些檔」**：維持由呼叫端提供。
- **`drift`、`review prepare`、`review scope`、`verify scope` 在沒有 checkout 的一端執行**：它們需要 git 與工作樹，維持只在有 checkout 的一端。
- **`demo` 在遠端可用**：它是本機的示範指令。
- **使用者文件**：這個變更不動 README 與 docs。

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `verify-evidence`: 新增「evidence show 動詞」。
- `verb-contract`: 新增「artifact cat 讀取已封存變更的文件」；修改「動詞契約的涵蓋面與 payload 形狀」（trace 不再是本質本機動詞；清單的時間欄位）、「動詞人眼輸出的兩模式同形」（拿掉嚴格重疊的分歧）、「模式分岔的單點宣告」（只限本機的只剩 demo）、「動詞 --json 輸出形狀凍結」（清單項新增兩個欄位）。
- `server-verb-api`: 修改「task done 消費 touchedFiles 且 evidence 有唯讀端點」（讀得到已封存的變更、不存在的名稱回 not_found）；新增「溯源與嚴格重疊的唯讀端點」。
- `client-protocol`: 新增「溯源回應 payload」（含 plan 回應的 `strictOverlap` 標記）與「變更清單的最後修改時間欄位」。
- `command-runtime`: 修改「動詞覆蓋與跨入口一致性」（覆蓋表加入 evidence show、trace、plan）。

## Impact

- Affected specs: verify-evidence（新增一條）、verb-contract（新增一條、修改四條）、server-verb-api（修改一條、新增一條）、client-protocol（新增兩條）、command-runtime（修改一條）
- Affected crates: `speclink-core`、`speclink-verbs`、`speclink-host`、`speclink-protocol`、`speclink-remote`、`speclink-cli`、`speclink-node`（只加測試）。`speclink-server` 與桌面 app 只改測試（釘住舊行為的斷言、因協定型別多了欄位而要補的測試資料）；store crates 不改。
- Affected code:
  - New:
    - crates/engine/speclink-core/src/command/evidence.rs
    - crates/adapters/speclink-verbs/src/families/evidence.rs
    - crates/adapters/speclink-verbs/src/render/evidence.rs
    - crates/adapters/speclink-cli/src/verbs/evidence.rs
    - crates/adapters/speclink-cli/tests/it/remote_read_verbs.rs
  - Modified:
    - crates/engine/speclink-core/src/command/mod.rs
    - crates/engine/speclink-core/src/command/describe.rs
    - crates/engine/speclink-core/src/lifecycle/listing.rs
    - crates/engine/speclink-core/tests/golden/command-describe.json
    - crates/adapters/speclink-verbs/src/families/mod.rs
    - crates/adapters/speclink-verbs/src/render/mod.rs
    - crates/adapters/speclink-verbs/src/render/query.rs
    - crates/host/speclink-host/src/http/router.rs
    - crates/host/speclink-host/src/http/routes.rs
    - crates/host/speclink-host/src/http/wire.rs
    - crates/host/speclink-host/src/http/coverage.rs
    - crates/host/speclink-host/src/documents.rs
    - crates/host/speclink-host/tests/http_door.rs
    - crates/host/speclink-host/tests/documents.rs
    - crates/protocol/speclink-protocol/src/query.rs
    - crates/protocol/speclink-remote/src/client.rs
    - crates/protocol/speclink-remote/src/convert.rs
    - crates/adapters/speclink-cli/src/main.rs
    - crates/adapters/speclink-cli/src/verbs/trace.rs
    - crates/adapters/speclink-cli/src/verbs/plan.rs
    - crates/adapters/speclink-cli/src/verbs/query.rs
    - crates/adapters/speclink-cli/tests/it/main.rs
    - crates/adapters/speclink-cli/Cargo.toml（測試用的相依：speclink-store-port，讓整合測試能對真的 HTTP 入口執行）
    - Cargo.lock
    - crates/adapters/speclink-cli/src/verbs/mod.rs
    - crates/adapters/speclink-verbs/src/tool.rs（只加測試）
    - crates/engine/speclink-core/src/command/typed.rs
    - crates/engine/speclink-core/src/lifecycle/model.rs（所有動詞共用的名稱查找與守門）
    - crates/engine/speclink-core/src/lifecycle/plan.rs（只改註解）
    - crates/host/speclink-host/src/http/error.rs（移除沒有產生點的錯誤分支）
    - crates/engine/speclink-core/src/command/tests.rs
    - crates/engine/speclink-core/src/teststore.rs
    - crates/engine/speclink-core/tests/it/command_describe.rs
    - 因清單多了兩個欄位、兩個拒絕訊息移除而同批改寫的既有測試：
      - crates/adapters/speclink-cli/tests/it/list_delta_capabilities.rs
      - crates/adapters/speclink-cli/tests/it/meta_fail_closed.rs
      - crates/adapters/speclink-cli/tests/it/meta_time_stamps.rs
      - crates/adapters/speclink-cli/tests/it/remote_read_path.rs
      - crates/adapters/speclink-cli/tests/it/mode_dispatch.rs
      - crates/adapters/speclink-cli/tests/it/plan_verbs.rs
      - crates/adapters/speclink-cli/tests/it/remote_plan.rs
      - crates/adapters/speclink-cli/tests/it/worktree_overlay.rs（比對清單輸出時略過 `modifiedAt`）
      - crates/host/speclink-host/tests/bridge_dual_path.rs
      - crates/host/speclink-host/tests/port_dual_path.rs
      - crates/host/speclink-server/tests/it/api/query_routes.rs
      - crates/protocol/speclink-remote/tests/it/typed_client.rs
      - crates/adapters/speclink-node/__test__/agent-tool.spec.ts
      - crates/adapters/speclink-node/__test__/dispatch-contract.spec.ts
      - crates/adapters/speclink-node/__test__/host.spec.ts
      - crates/adapters/speclink-node/__test__/store-bridge.spec.ts
    - 因協定型別多了欄位而補一行的桌面測試資料：
      - apps/desktop/src-tauri/src/remote.rs
      - apps/desktop/src-tauri/tests/it/remote_data.rs
  - Removed: (none)
- 相依：沒有未完成的前置變更。下一個變更 skills-verbs-second-wave 排在它之後。
- 三種形態下的結果：
  - 本機 repo：`evidence show` 可用；`artifact cat` 讀得到已封存的變更；`trace` 與 `plan --strict-overlap` 不變；清單多兩個欄位。
  - 連到遠端的 checkout：五項都可用。已知缺口：連到示範用 server 時清單不帶 `modifiedAt`（它的儲存答不出最後修改時間，等變更 retire-teamstore-and-drivers）。
  - 沒有本機規格檔的內嵌宿主：五項都可用（經 `host.dispatch` 與 HTTP 的入口）。`modifiedAt` 取自宿主的儲存回報的最後修改時間。
- 文件影響（留給最後一個變更 docs-embedded-host，這個變更不動使用者文件）：
  - 動詞與旗標的參考文件：新增 `evidence show`；trace 與 `plan --strict-overlap` 從只限本機的清單移除；`list --json` 的新欄位；兩條新路由。
  - 正式規格 user-documentation 有一句「本質本機動詞列含 demo 與 trace 兩者」，要在文件的變更一起改。

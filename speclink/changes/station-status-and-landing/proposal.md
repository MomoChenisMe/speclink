## Why

每個變更封存前要過兩個品質關卡：審查與驗證。各自有幾輪工單，最後蓋章。封存由宿主這一邊發起時，宿主要回答三個問題：兩個關卡過了沒有、誰蓋的章、程式碼是不是已經進了正式分支。今天在遠端這三個問題都答不出來：

- **線上的變更清單沒有關卡的欄位。** 清單項與單筆變更的回應裡，沒有任何審查或驗證的資料。Speclink Desktop 對本機專案是自己讀 metadata 算出來的；對遠端專案，看板卡片上就沒有關卡的標示，封存時的「還有未結工單」對話框也永遠不會出現。
- **審查與驗證的事件在線上沒有型別。** 引擎有六種事件（各關卡的新增一輪、蓋章、放棄），但推送給用戶端時歸不到任何資源，類別是 unknown、資源識別是空的。
- **蓋章不記錄程式碼的版本。** 蓋章記下的是每個檔案的內容指紋，沒有 commit。
- **變更不記錄分支，也沒有地方回報「程式碼落在哪個 commit」。** 引擎裡沒有這樣的欄位，也沒有這樣的動詞。

討論 embedded-host-needs 的裁定：把關卡的狀態加成線上的選填欄位；事件歸到該變更；蓋章多帶 commit、變更記下分支、新增回報落地 commit 的動詞。引擎只存這些資料，不附帶任何規則——「commit 必須在正式分支才能封存」這類規則屬於宿主，由宿主在執行前掛鉤裡自己判斷。

- **目標使用者**：把 speclink 內嵌進自己後端的整合者（追蹤關卡、在封存前加條件）；用 Speclink Desktop 看遠端專案的開發者與 PO、PM。
- **使用情境**：review、verify、quality、commit、worktree-merge、archive 這幾個階段；宿主的看板或工單系統顯示每個變更的進度。
- **既有規格的對照**：client-protocol 的「變更清單的審查狀態欄位」與「變更清單的驗證狀態欄位」規定的是 Speclink Desktop 內部的清單項，並規定「蓋章後檔案有沒有變」要在有工作樹的一端算。review-station 與 verify-station 規定蓋章寫入的欄位，沒有 commit；兩者都釘住 `speclink list --json` 不含關卡欄位。server-event-stream 允許對映不了的事件以 unknown 發出。沒有任何正式規格涵蓋分支與落地。

Source discussion: embedded-host-needs（cut 14）。

## What Changes

**線上的關卡狀態**

- 變更清單的每一項新增選填的 `review` 與 `verify`：`state`（`none`、`inProgress`、`stamped`）；蓋章後另有 `stampedAt`、`stampedBy`、`stampedWith`、`rounds`、`tasksTotal`、`commit`。
- 單筆變更的回應有相同的兩個物件，另外多 `scope`（蓋章時每個檔案的路徑與內容指紋），讓有 checkout 的用戶端自己算「蓋章後檔案有沒有變」。
- 已封存清單的每一項新增選填的 `reviewOutcome` 與 `verifyOutcome`：`none`、`stamped`、`notPassed`（帶著未結的工單封存）。封存的文件集裡有工單就是 `notPassed`，章還留在 metadata 也一樣。
- 本機的已封存清單（Speclink Desktop 的 `reviewStatus`、`verifyStatus`）改成同一個順序：有工單優先。這是品質關卡找到的問題——蓋章後重開工單、再帶著工單封存的變更，原本會顯示成通過。
- Server 不判斷「蓋章後檔案有沒有變」。它沒有程式碼。

**事件**

- 六種審查與驗證的事件（各關卡的新增一輪、蓋章、放棄）在推送時歸到「變更」這一類，資源識別是變更名稱。

**蓋章帶 commit**

- 蓋章的命令與線上請求新增選填的 commit。CLI 在蓋章時把 checkout 目前的 HEAD 帶上（本機與遠端都帶）。引擎把它存進 metadata，不檢查它。

**分支與落地**

- 開工標記（`speclink in-progress add`）新增選填的分支：CLI 把 checkout 目前的分支名稱帶上，引擎存進 metadata。
- 新動詞 `speclink change landed <name> [--commit <sha>] [--branch <name>] [--json]`：回報這個變更的程式碼落在哪個 commit。
  - 不帶 `--commit` 時，CLI 用 checkout 目前的 HEAD。
  - 引擎記下 commit、（有給時）分支、回報的時間與回報的人。重複回報會覆寫。
  - stdout：一行確認；`--json` 是 `{ "change", "commit", "branch"? }`。沒有 stdin。
  - exit code：成功 0；變更不存在、或沒有 commit 可用（不帶 `--commit` 而且不在 git checkout 裡）非 0。
  - 產生事件 `change-landed`。
- 線上新增對應的路由；單筆變更與清單項新增選填的 `branch`、`landedCommit`、`landedAt`、`landedBy`。
- `speclink show <change> --json` 新增 `review`、`verify`、`branch`、`landed` 四個選填欄位，讓內嵌的宿主經 `host.dispatch` 也讀得到。沒有記錄的欄位不輸出：關卡沒有工單也沒有章時不輸出 `review`（或 `verify`），所以沒有任何記錄的變更輸出不變。
- commit 與 worktree-merge 兩支技能各多一步：提交或合併完成後執行 `speclink change landed`，並以 `--commit` 明確帶上 HEAD（技能的動詞可能在沒有 checkout 的宿主那一端執行）。commit 技能在封存子流程已經封存變更時略過這一步。

**Speclink Desktop**

- 遠端專案的看板卡片、詳情頁與已封存頁顯示審查與驗證的標示，規則和本機專案相同。有 checkout 時用單筆回應的 `scope` 在本機算「蓋章後有沒有變」；沒有 checkout 時只比任務數。
- 遠端專案封存時，「還有未結工單」的三選項對話框會出現，選「照樣帶走」時把旗標帶給 Server；對話框的「放棄審查／放棄驗證」在遠端也接上（刪掉該站的工單）。

**相容性影響**

- `speclink list --json`：不變（兩個關卡的規格釘住它不含關卡欄位）。CLI 在遠端收到新欄位時不輸出它們。
- `speclink show <change> --json`：多四個選填欄位，只在變更有對應的記錄時出現；沒有記錄的變更輸出與之前逐位元相同。人眼輸出不變。
- 本機 repo 的 commit 與 worktree-merge 流程：回報落地會寫進變更的資料檔，所以回報之後那個檔有一筆未提交的改動，留給這個變更的下一次提交（技能會說明，不代為提交）。主 checkout 在下一次 worktree 合併前要先把它提交。
- Speclink Desktop 的已封存清單：工單與章並存的封存項，`reviewStatus` 由 `reviewed` 變成 `reviewedNotPassed`（驗證同理），詳情頁因此多出那一站的工單分頁。其他封存項不變。本機快取的版本加一，升級後第一次開啟會重算一次，使用者不用做任何事。
- 蓋章、開工標記：metadata 多了 `reviewed_commit`、`verified_commit`、`branch`。沒有帶值時不寫，既有的 metadata 照常可讀。
- 舊版用戶端連到新版 Server：回應多了它不認得的選填欄位，照常運作（這是討論留下的未驗證項，這個變更用測試確認）。
- 新版用戶端連到舊版 Server：沒有新欄位，遠端專案的標示維持今天的樣子；`change landed` 得到 404。
- 技能檔：commit 與 worktree-merge 的內容改變，要執行 `speclink update`。資產版號前進一版。
- **設定欄位**：沒有新增或變更。

## Non-Goals

- **任何依這些資料做判斷的規則**：例如「沒有落地就不能封存」「commit 必須在某個分支」。引擎只存；宿主用執行前掛鉤自己擋。
- **Server 檢查 commit 是否存在、分支是否正確**：Server 沒有程式碼。
- **`speclink list --json` 帶出關卡欄位**：維持兩個關卡規格的釘住。
- **Server 計算「蓋章後檔案有沒有變」**：維持由有 checkout 的一端判斷。
- **自動偵測落地**（監看 git、接 webhook）：由技能或宿主明確回報。
- **把分支與落地放進宿主的自由欄位**：討論已否決，避免宿主的鍵和 speclink 的鍵混在一起。
- **使用者文件**：這個變更不動 README 與 docs。

## Capabilities

### New Capabilities

- `change-landing`: 變更的分支與程式碼落地記錄——開工時記下分支、`change landed` 動詞回報落地的 commit、這些資料怎麼存與怎麼讀。既有的 change-lifecycle 管的是變更的階段與封存，verify-evidence 管的是逐任務改了哪些檔；兩者都不涵蓋「這個變更的程式碼最後落在哪裡」。

### Modified Capabilities

- `client-protocol`: 新增「線上變更回應的關卡欄位」與「落地回報的請求與回應」；修改「已封存清單的審查結局欄位」與「已封存清單的驗證結局欄位」（工單與章並存時讀作未通過）。
- `server-verb-api`: 新增「變更回應攜帶關卡狀態與落地記錄」。
- `server-event-stream`: 新增「審查與驗證事件歸到變更」。
- `review-station`: 新增「蓋章記錄程式碼的 commit」。
- `verify-station`: 新增「驗證蓋章記錄程式碼的 commit」。
- `desktop-app`: 新增「remote 專案的審查與驗證標示」。
- `commit-skill`: 新增「提交後回報落地」。
- `worktree-merge-skill`: 新增「合併後回報落地」。

## Impact

- Affected specs: change-landing（新增）、client-protocol（新增兩條、修改兩條）、server-verb-api、server-event-stream、review-station、verify-station、desktop-app、commit-skill、worktree-merge-skill（各新增一條）
- Affected crates 與 app: `speclink-core`、`speclink-verbs`、`speclink-host`、`speclink-protocol`、`speclink-remote`、`speclink-server`（事件的分類）、`speclink-cli`、`speclink-node`（只加測試）、apps/desktop（core、Tauri 殼與前端）。
- Affected code:
  - New:
    - crates/engine/speclink-core/src/command/landing.rs
    - crates/adapters/speclink-cli/tests/it/change_landing.rs
    - crates/host/speclink-host/tests/station_wire.rs
  - Modified:
    - crates/engine/speclink-core/src/command/mod.rs
    - crates/engine/speclink-core/src/command/describe.rs
    - crates/engine/speclink-core/src/lifecycle/model.rs
    - crates/engine/speclink-core/src/lifecycle/inprogress.rs
    - crates/engine/speclink-core/src/quality/station.rs
    - crates/engine/speclink-core/tests/golden/command-describe.json
    - crates/engine/speclink-core/assets/skills/commit.md
    - crates/engine/speclink-core/assets/skills/worktree-merge.md
    - crates/engine/speclink-core/src/workspace/init.rs
    - crates/engine/speclink-core/tests/golden/assets.lock
    - crates/adapters/speclink-verbs/src/inputs.rs
    - crates/adapters/speclink-verbs/src/families/plan.rs
    - crates/adapters/speclink-verbs/src/families/station.rs
    - crates/adapters/speclink-verbs/src/families/progress.rs
    - crates/adapters/speclink-verbs/src/render/plan.rs
    - crates/adapters/speclink-verbs/src/render/query.rs
    - crates/host/speclink-host/src/commit.rs
    - crates/host/speclink-host/src/documents.rs
    - crates/host/speclink-host/src/http/router.rs
    - crates/host/speclink-host/src/http/routes.rs
    - crates/host/speclink-host/src/http/wire.rs
    - crates/host/speclink-host/src/http/coverage.rs
    - crates/protocol/speclink-protocol/src/query.rs
    - crates/protocol/speclink-protocol/src/command.rs
    - crates/protocol/speclink-remote/src/client.rs
    - crates/host/speclink-server/src/api/events.rs
    - crates/adapters/speclink-cli/src/verbs/plan.rs
    - crates/adapters/speclink-cli/src/verbs/station.rs
    - crates/adapters/speclink-cli/src/verbs/progress.rs
    - crates/adapters/speclink-cli/src/verbs/query.rs
    - crates/adapters/speclink-cli/tests/it/main.rs
    - apps/desktop/core/src/query.rs
    - apps/desktop/src-tauri/src/remote.rs
    - crates/engine/speclink-core/src/command/typed.rs
    - crates/engine/speclink-core/src/command/tests.rs
    - crates/engine/speclink-core/src/lifecycle/change_schema.rs
    - crates/engine/speclink-core/src/lifecycle/tasks.rs
    - crates/engine/speclink-core/src/quality/station/tests.rs
    - crates/engine/speclink-core/src/util.rs
    - crates/engine/speclink-core/tests/it/command_describe.rs
    - crates/engine/speclink-core/tests/it/skill_verbization.rs
    - crates/adapters/speclink-verbs/src/render/mod.rs
    - crates/adapters/speclink-verbs/src/tool.rs
    - crates/adapters/speclink-cli/src/common.rs
    - crates/host/speclink-host/src/port_exec.rs
    - crates/host/speclink-host/tests/documents.rs
    - crates/host/speclink-host/tests/http_door.rs
    - crates/host/speclink-server/tests/it/api/review_api.rs
    - crates/host/speclink-server/tests/it/api/verify_api.rs
    - crates/host/speclink-server/tests/it/api/write_guard.rs
    - crates/protocol/speclink-remote/src/convert.rs
    - crates/protocol/speclink-remote/tests/it/typed_client.rs
    - crates/adapters/speclink-node/__test__/host.spec.ts
    - crates/adapters/speclink-node/__test__/agent-tool.spec.ts
    - apps/desktop/core/src/manage.rs
    - apps/desktop/src-tauri/src/lib.rs
    - apps/desktop/src-tauri/tests/it/remote_data.rs
    - apps/desktop/src-tauri/tests/it/remote_runtime.rs
    - apps/desktop/src-tauri/tests/it/phase3_chain.rs
    - apps/desktop/src/adapter/remoteDataSource.ts
    - apps/desktop/src/session.ts
    - apps/desktop/src/__tests__/remoteDataSource.test.ts
    - apps/desktop/src/__tests__/remoteCapabilities.test.tsx
    - apps/desktop/core/src/cache.rs
    - apps/desktop/src/App.tsx
    - packages/ui/src/adapter.ts
    - crates/host/speclink-server/tests/it/api/sse_events.rs
  - Removed: (none)
  - 另外會重新產生：技能的 golden 快照（10 份）與本 repo 的技能檔（`.claude/skills` 與 `.agents/skills` 共 37 份，只有版號行與兩支技能的內文有變）。
  - `speclink-node` 只加測試：crates/adapters/speclink-node/src/lib.rs 不用改（蓋章的 stdin 解析在共用的動詞定義裡）。
- 相依：排在變更 skills-verbs-second-wave 之後（它改寫 commit 的原稿；這個變更在它之上加一步）。和 remote-skill-delivery 沒有先後。
- 三種形態下的結果：
  - 本機 repo：蓋章記下 commit、開工記下分支、`change landed` 可用，資料存在變更的 metadata。Speclink Desktop 的標示不變。
  - 連到遠端的 checkout：同上，資料存在 Server；Speclink Desktop 的遠端專案有關卡的標示。
  - 沒有本機規格檔的內嵌宿主：宿主經線上的清單、`show --json` 與提交時的事件讀到關卡狀態、分支與落地；宿主的程式也能自己執行 `change landed`（要給 `--commit`）。蓋章與開工標記在這個形態不帶 commit 與分支，除非呼叫端提供：宿主套件不替宿主讀 git。蓋章的 commit 由 `--stdin` 的 JSON（`headCommit`）提供；開工標記經 `host.dispatch` 沒有帶分支的輸入，分支要由線上的開工請求帶上，或事後以 `change landed --branch` 補記。
- 文件影響（留給最後一個變更 docs-embedded-host，這個變更不動使用者文件）：
  - 動詞與旗標的參考文件：`change landed`、新的線上欄位與路由、事件的分類。
  - 「把 Speclink 內嵌進你的系統」：怎麼用執行前掛鉤加「沒有落地就不能封存」這類條件。
  - 工作流程文件：commit 與合併之後會回報落地。

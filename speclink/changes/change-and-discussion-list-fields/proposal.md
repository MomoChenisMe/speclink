## Why

內嵌的宿主經呼叫入口只讀得到動詞的輸出。引擎與儲存已經知道的幾件事，動詞卻沒有說出來，其他出口（HTTP 回應、Speclink Desktop）反而有：變更的階段、開工日、提案 Why 的第一行、建立者的識別、討論的最後修改時間。宿主因此要自己推導階段、用 30 個字元就截斷的 `summary` 當說明、用自己的保留欄位記建立者，而且在兩種邊界情況下和 Desktop 的看板不一致。討論 embedded-host-needs-10 逐項對照程式碼後，決定把這些事實補進動詞的輸出，並新記一筆事後補不回來的資料：任務全部完成的時間。

目標使用者有兩種：把引擎內嵌進自己後端、經動詞組出畫面的宿主開發者；以及在那些畫面與 Desktop 上追進度的開發者、PM 與 PO。對應的階段是 apply 之後的追蹤，不是任何單一技能。

## What Changes

- `speclink list --json --detail` 的每個變更與 `speclink show <變更> --json` 增加選填的鍵：
  - `whyExcerpt`：提案 Why 段落第一個非空行的原文，不截斷。規則從 Desktop 搬進 speclink-core，Desktop 改成呼叫它。既有的 `summary` 與它的 30 個字元上限不動。
  - `stage`：`proposed`、`in-progress` 或 `ready`，直接用引擎既有的階段規則，不經 plan，所以依賴成環時照樣有值。
  - `startedAt`：開工日，只有日期。
  - `createdById`：建立者的識別。
  - `readyAt`：任務全部完成的日期，只有日期，而且只在目前全部任務都勾完時出現。
  - `show --json` 另外補上它一直沒有的 `createdBy`。
- `speclink discuss list --json`（含 `--archived`）的每一筆與 `speclink discuss show <slug> --json` 的 `info` 增加 `modifiedAt`（儲存回報的最後修改時間，RFC 3339 的 UTC 時間，意思和變更清單的 `modifiedAt` 相同）與 `createdById`。
- `speclink list --archived --json` 的每一筆增加 `createdById`。
- 建立時多記一筆資料：執行環境有呼叫者的識別時，`new change`、`discuss new` 與 `discuss promote` 多寫 `created_by_id`。宿主套件每次呼叫都已經把識別交給引擎，所以宿主不用多傳任何東西；本機 CLI 沒有識別，不會寫。
- 任務全部完成時多記一筆資料：一次任務完成讓全部任務都勾完時，在變更的 metadata 寫入 `ready_at`（再次發生時覆寫）。`task undone` 不動它。Desktop 的「全部標完成」同樣寫入。
- HTTP 回應帶上同樣的新欄位：變更清單、單一變更、討論清單、單一討論與已封存清單。經遠端連線的 CLI 輸出與本機相同；Desktop 的遠端專案因此也拿得到 `whyExcerpt`，卡片多出描述列。
- 動詞參考文件寫明 `list --json` 的 `status` 只有 `done` 與 `in-progress` 兩個值，`done` 等於 `stage` 的 `ready`。`status` 本身不改。
- 不新增任何 CLI 指令與旗標；不改 stdin 與 exit code；不改設定欄位；不改任何技能檔。

### 相容性影響

- 新的鍵都是選填的：沒有值時整個鍵不出現，不用 null。
- 不帶 `--detail` 的 `list --json`、`status --json`、`plan --json` 與全部的人眼輸出逐位元不變。
- `show --json` 會變：每個變更多出 `stage`，有記錄時多出其他鍵。`discuss list --json` 與 `discuss show --json` 會變：儲存答得出修改時間時（本機專案與經儲存接口都答得出）每一筆多出 `modifiedAt`。這兩組輸出的回歸對照（render 的單元測試與 CLI 測試）同批更新。
- 舊資料不遷移、不回填：沒有 `created_by_id` 與 `ready_at` 的變更與討論照常運作，對應的鍵不出現。
- 新的 CLI 連到舊的 Server：回應沒有新欄位，對應的鍵不出現，指令不失敗。舊的 CLI 與舊的 Desktop 連到新的 Server：多出的欄位被忽略。
- metadata 多了兩個鍵（`created_by_id`、`ready_at`）。舊版引擎讀到不認得的鍵會略過。
- 讀動詞 JSON 的程式若用嚴格的 schema 驗證輸出，要放行新的鍵。

### 三種形態

- 本機 repo：輸出的鍵可用。`createdById` 不會出現，因為本機 CLI 沒有呼叫者的識別（除非 metadata 本來就帶著）；`modifiedAt` 取檔案時間，checkout 或複製也會改到它，只是提示。
- 連到遠端的 checkout：可用，輸出與本機同形；要等 Server 升級到含這個變更的版本，之前對應的鍵不出現。
- 沒有本機規格檔的內嵌宿主：可用。這是這個變更的主要對象：經呼叫入口執行動詞就拿得到全部新鍵，`created_by_id` 自動記下呼叫者的識別，`modifiedAt` 取儲存接口回報的鍵的修改時間（從別處匯入的文件，這個時間是匯入的時間）。

## Non-Goals (optional)

見 design 的 Goals / Non-Goals。

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `verb-contract`: 新增需求，規定清單的詳細輸出與 `show --json` 帶說明、階段、開工日與建立者，以及五個輸出帶建立者的識別；三種執行方式輸出相同。
- `change-lifecycle`: 新增需求，規定建立變更時記下建立者的識別，以及任務全部完成時記下就緒時間與它的輸出規則。
- `discussion-docs`: 新增需求，規定討論記錄記下建立者的識別，以及討論的清單與單項輸出最後修改時間。
- `client-protocol`: 新增需求，規定變更、討論與已封存清單的回應型別帶新的選填欄位，舊 Server 不送時以缺席容錯。

查過的相關正式規格：verb-contract（清單的詳細輸出、已封存變更的清單、任務取消勾選）、change-lifecycle（change metadata 時間欄位寫入時戳、對外輸出日期；任務完成蘊含開工）、change-plan（plan 的 stage）、discussion-docs（討論記錄蓋建立者章）、client-protocol（變更清單的建立者與來源討論欄位）、server-verb-api、store-abstraction、desktop-app（看板全尺寸卡的描述列）、verify-evidence、free-fields、host-kit。既有的需求都不修改；新的鍵以新增的需求規定。

## Impact

- Affected specs: verb-contract、change-lifecycle、discussion-docs、client-protocol
- Affected crates and apps: speclink-core、speclink-fs、speclink-host、speclink-protocol、speclink-remote、speclink-verbs、speclink-cli、speclink-node（只有測試）、apps/desktop（desktop-core 與 Tauri 殼的遠端資料源）
- Affected code:
  - Modified:
    - crates/engine/speclink-core/src/lifecycle/listing.rs
    - crates/engine/speclink-core/src/lifecycle/model.rs
    - crates/engine/speclink-core/src/lifecycle/newcmd.rs
    - crates/engine/speclink-core/src/lifecycle/discuss.rs
    - crates/engine/speclink-core/src/lifecycle/tasks.rs
    - crates/engine/speclink-core/src/command/mod.rs
    - crates/engine/speclink-core/src/store.rs
    - crates/engine/speclink-core/src/teststore.rs
    - crates/engine/speclink-fs/src/lib.rs
    - crates/host/speclink-host/src/port_view.rs
    - crates/host/speclink-host/src/documents.rs
    - crates/host/speclink-host/src/http/wire.rs
    - crates/host/speclink-host/src/http/routes.rs
    - crates/protocol/speclink-protocol/src/query.rs
    - crates/adapters/speclink-verbs/src/render/query.rs
    - crates/adapters/speclink-cli/src/verbs/query.rs
    - crates/adapters/speclink-cli/src/verbs/discuss.rs
    - apps/desktop/core/src/query.rs
    - apps/desktop/core/src/cache.rs
    - apps/desktop/core/src/manage.rs
    - apps/desktop/src-tauri/tests/it/remote_data.rs
    - crates/engine/speclink-fs/tests/store_fs.rs
    - crates/host/speclink-host/tests/http_door.rs
    - crates/adapters/speclink-cli/tests/it/remote_verb_parity.rs
    - crates/adapters/speclink-cli/tests/it/show_detail.rs
    - docs/verb-contract.md
    - docs/verb-contract.zh-TW.md
    - docs/product-status.md
    - docs/product-status.zh-TW.md
  - New: (none)
  - Removed: (none)

## Why

把 Speclink 引擎內嵌進自己後端的宿主（用 Node SDK 的 `createHost`，使用者經宿主自己的 Agent 工具與管理畫面操作），有兩處要自己維護脆弱的對照：

- 宿主的執行前掛鉤把每個命令名稱對應到自己的權限動作，對照表沒有的命令一律拒絕。命令名稱是對外契約，但 SDK 沒有匯出這份清單，宿主只能手抄，升級後多出來的命令會默默變成「不能用」。
- 匯出與匯入失敗時，Error 上只有粗分類的 `code` 與英文訊息。宿主要對自己的管理者顯示自己的說明，只能比對訊息的開頭；Speclink 改措辭時這個比對會默默失效。

引擎早就有這兩份資訊，只差一個穩定的出口。這個變更開這兩個出口，並修正同一輪查到的三個落差：SDK 文件說 `--json` 的欄位都是 camelCase（實際有凍結的 snake_case 例外）、內嵌時 `new artifact` 回報的 `path` 不是儲存的鍵（違反 node-sdk 規格的承諾）、`workflow-config set` 的說明沒有寫空字串的作用。

這個變更來自討論 embedded-host-needs-4 的 cut 2，是這份討論的最後一個變更。cut 1（型別定義檔的修正）已經以 0.13.1 發版。

規格掃描：相關的正式規格是 node-sdk（SDK 的匯出、`exportScope`／`importScope`、「Agent 工具的說明指出 path 是儲存的鍵」）、command-runtime（「引擎命令有穩定的名稱與可序列化的參數」）、host-runtime（「文件種類與鍵的對應」）、verb-contract（「動詞 --json 輸出形狀凍結」）、workflow-config（「workflow-config set 政策欄位寫入」）。host-kit 的「範圍的匯出與匯入」不改：失敗的分類（`code`）與訊息都維持原樣。

## What Changes

- **SDK 匯出 `commandNames`**（`@speclink/engine`）：唯讀的字串陣列，57 個名稱，是引擎的 54 個命令名稱加上宿主套件自己的三個名稱：`scope-export`、`scope-import`，以及 `host.handleRequest` 寫看板排序（`PUT /board-order`）時問掛鉤與寫 commit 用的 `put-board-order`。清單只給名稱，不帶「這個命令會不會寫入」。型別定義檔同步宣告。
  - 引擎的 54 個名稱來自 speclink-core 的一份常數；既有的逐命令對照測試多一條斷言，釘住「常數等於全部命令的名稱」，新增命令而沒有加進常數時測試失敗。
- **匯出與匯入的失敗多帶選填的 `reason`**：`engine.exportScope`、`host.exportScope`、`host.importScope` 三個呼叫 reject 的 Error，在既有的 `code` 與 `message` 之外，原因屬於下列清單時多一個字串屬性 `reason`。`code` 與 `message` 完全不變。
  - 匯出本機資料夾（`engine.exportScope`）：`no_spec_folder`、`spec_folder_outside_project`、`symlink_in_spec_folder`、`spec_folder_unreadable`、`invalid_change_metadata`。
  - 匯入（`host.importScope`）：`scope_not_empty`、`bundle_too_large`、`invalid_bundle_document`。
  - 不在清單裡的失敗不帶 `reason`：儲存錯誤、掛鉤拒絕、匯出時範圍一直在變（所以 `host.exportScope` 目前沒有任何失敗帶 `reason`）、匯入包的格式版本不支援或不是匯入包、變更指定的產出流程找不到。一般動詞（`dispatch`）的失敗不加這個屬性。
  - 清單寫進 SDK 文件與型別定義檔的說明；SDK 文件裡要宿主比對訊息開頭的句子改成指向 `reason`。
- **內嵌時 `new artifact` 的 `path` 改成儲存的鍵**：經 `host.dispatch` 執行 `new artifact … --json`，結果的 `path` 由 `speclink/changes/<name>/<artifact>` 改成 `changes/<name>/<artifact>`，和同一個宿主裡 `discuss new` 回報的 `path` 一致。`discuss promote … --json` 的 `path` 同樣由 `speclink/changes/<name>` 改成 `changes/<name>`（apply 期排查找到）。
- **文件更正 `--json` 的欄位命名**：欄位不改名。`docs/sdk-node` 的「欄位是 camelCase」改成符合現況；`docs/verb-contract` 列出全部 snake_case 的既有欄位，並寫明它們和 camelCase 的欄位一樣是凍結的契約。
- **`workflow-config set` 的說明補一句**：`--help` 與 SDK 文件寫明空字串把 `locale`、`spec_locale` 設回未設定。行為不變。

### 相容性影響

- `new artifact --json` 與 `discuss promote --json` 的 `path`：只有內嵌宿主（`host.dispatch`）的值會變，少了開頭的 `speclink/`。人眼輸出不受影響（內嵌時本來就不印路徑行）。本機 CLI、連到遠端的 CLI、`createEngine` 的輸出都逐位元不變。依賴舊前綴的宿主要拿掉對 `speclink/` 的假設；SDK 文件與 `agentTool.description` 一直寫的是「`path` 是儲存的鍵」，所以這是把行為修到和文件一致。
- `commandNames` 與 `reason` 都是新增，沒有既有行為改變。沒有 `reason` 的 Error 和現在完全相同。
- `workflow-config set --help` 多一行說明文字；exit code、stdin、旗標都不變。
- 不新增設定欄位，不改技能與 Agent 指令的內容，不 bump ASSET_VERSION。

### 三種形態

| 項目 | 本機 repo | 連到遠端的 checkout | 沒有本機規格檔的內嵌宿主 |
| --- | --- | --- | --- |
| `commandNames` | 不適用：它是 SDK 的匯出，CLI 沒有對應的輸出 | 不適用，同左 | 可用 |
| 失敗的 `reason` | `engine.exportScope` 可用（它讀本機資料夾） | 不適用：遠端 CLI 的匯入匯出不經這三個呼叫，HTTP 回應不變 | `host.importScope` 可用；`host.exportScope` 目前沒有帶 `reason` 的失敗 |
| `new artifact`、`discuss promote` 的 `path` | 不變（檔案路徑） | 不變（CLI 自己組出的路徑） | 改成儲存的鍵 |
| `workflow-config set --help` | 可用 | 可用 | 不適用：內嵌宿主沒有 `--help` 的輸出；SDK 文件補同一句 |

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `command-runtime`: 新增需求——引擎提供一份全部命令名稱的清單，並由測試釘住它和命令的種類一一對應。
- `node-sdk`: 新增兩條需求——SDK 匯出 `commandNames`；匯出與匯入的失敗帶選填的 `reason`。
- `host-runtime`: 新增需求——經儲存接口寫入 artifact 時，回報的路徑是儲存的鍵。
- `verb-contract`: 新增需求——`--json` 輸出裡既有的 snake_case 欄位是凍結契約的一部分，文件要列出它們。
- `workflow-config`: 新增需求——`workflow-config set` 的說明寫出空字串對 `locale`、`spec_locale` 的作用。

## Impact

- Affected specs: command-runtime、node-sdk、host-runtime、verb-contract、workflow-config
- Affected crates／apps: speclink-core（命令名稱常數）、speclink-host（宿主套件的名稱清單、匯出匯入失敗的原因、儲存接口回報的路徑）、speclink-node（SDK 的匯出與 Error 的屬性）、speclink-verbs（說明文字）。speclink-cli、speclink-server、apps/desktop 的行為不變；speclink-desktop（apps/desktop/src-tauri）呼叫本機匯出清單的那一處要跟著改型別，行為不變。
- Affected code:
  - Modified:
    - `crates/engine/speclink-core/src/command/describe.rs`
    - `crates/engine/speclink-core/src/command/mod.rs`
    - `crates/engine/speclink-core/tests/it/command_describe.rs`
    - `crates/host/speclink-host/src/transfer.rs`
    - `crates/host/speclink-host/src/kit.rs`
    - `crates/host/speclink-host/src/port_view.rs`
    - `crates/host/speclink-host/src/documents.rs`
    - `crates/host/speclink-host/src/http/routes.rs`
    - `crates/host/speclink-host/tests/transfer.rs`
    - `crates/adapters/speclink-node/src/lib.rs`
    - `crates/adapters/speclink-node/src/host.rs`
    - `crates/adapters/speclink-node/index.js`
    - `crates/adapters/speclink-node/index.d.ts`
    - `crates/adapters/speclink-node/__test__/engine.spec.ts`
    - `crates/adapters/speclink-node/__test__/export-local.spec.ts`
    - `crates/adapters/speclink-node/__test__/transfer.spec.ts`
    - `crates/adapters/speclink-node/__test__/host.spec.ts`
    - `crates/adapters/speclink-node/__test__/handle-request.spec.ts`
    - `crates/adapters/speclink-verbs/src/families/workflow_config.rs`
    - `apps/desktop/src-tauri/src/remote.rs`
    - `docs/sdk-node.md`
    - `docs/sdk-node.zh-TW.md`
    - `docs/verb-contract.md`
    - `docs/verb-contract.zh-TW.md`
  - New: (none)
  - Removed: (none)

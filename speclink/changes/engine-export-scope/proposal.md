## Why

把一個本機專案的規格搬進宿主（內嵌了引擎的系統），需要先把本機的規格資料夾做成一個匯入包，再交給宿主的 `host.importScope`。今天能做出這個匯入包的程式只有一份，而且寫在 Speclink Desktop 的殼層裡（「遷移到 Server」）。Node SDK 的 `createEngine` 沒有匯出，CLI 也沒有匯入匯出的動詞。

結果是宿主沒辦法在自己的管理腳本裡做搬遷。組織有二十套規格要搬，就要有一個人用 Speclink Desktop 操作二十次。想批次搬、想在持續整合裡搬、或想把搬遷做進宿主的管理畫面，都沒有可以呼叫的函式。

受影響的人是寫宿主程式的整合者，情境是導入時把既有的本機專案搬進宿主，以及之後定期把本機專案匯出備份。來源討論 embedded-host-needs-2 裁定：把做匯入包的程式搬到共用的地方，Node 的 `createEngine` 加上匯出；CLI 的匯入匯出動詞這一輪不做。

掃過的相關正式規格：workspace-migration（「Bundle 組裝涵蓋本地 workspace 全集」規定匯入包要含哪些文件）、node-sdk（「宿主的匯出與匯入方法」「createEngine 只接受本機資料夾」）、host-kit（「範圍的匯出與匯入」）、client-protocol（「以鍵標示文件的匯入包與匯出回應」）。

## What Changes

- Node SDK 的本機引擎新增 `engine.exportScope()`：讀取 `createEngine` 指向的本機專案，回傳第 2 版的匯入包（每份文件以儲存的鍵命名），形狀與 `host.exportScope` 的回傳值相同，可以直接交給 `host.importScope`。
  - 內容與 Speclink Desktop「遷移到 Server」送出的相同：進行中變更的 metadata、文件與 evidence，正式規格，進行中與已封存的討論，已封存變更的全部文件，流程設定，詞彙表，看板順序。
  - 本機專案沒有範圍版本號，所以回傳的包沒有 `scopeVersion`；匯入本來就不讀它。
  - 任何一個變更的 metadata 壞掉，或變更裡有引擎不認得的文件形式時，整個匯出失敗並指出是哪個檔案，不回傳不完整的包。
- 做匯入包的程式從 Speclink Desktop 的殼層搬到宿主套件所在的 crate（`speclink-host`），Speclink Desktop 改用共用的版本。Desktop 的遷移流程、畫面與送出的內容不變。
- 使用者文件：Node SDK 文件的「匯出與匯入」一節加上本機專案的匯出與一段批次搬遷的範例；專案能力狀態把「CLI 沒有匯入與匯出的動詞」那一列的替代做法更新成可以用 `engine.exportScope()`；中英兩版同步。

相容性影響：

- 不新增或變更任何 CLI 子指令、旗標、stdin、exit code、輸出。
- Node SDK 只新增一個方法；`engine.dispatch`、`createHost` 與既有型別不變（`ScopeBundle` 的 `scopeVersion` 本來就是選填）。
- Speclink Desktop 的遷移行為不變；它的既有測試（匯入包的內容、壞 metadata 的中止）繼續當作回歸對照。
- 線上格式不變。

三種形態：

- 本機 repo：可用。`engine.exportScope()` 就是為本機專案做的。
- 連到遠端的 checkout：不適用。規格不在本機，`createEngine` 指向的資料夾沒有規格檔；要匯出遠端的內容，用宿主的 `host.exportScope` 或 HTTP 入口的匯出端點。
- 沒有本機規格檔的內嵌宿主：這個方法在宿主的程式裡被呼叫，對象是放在宿主機器上的本機專案資料夾（例如從 git 取出的 checkout）；匯出的包再交給 `host.importScope` 寫進宿主的儲存。

不涉及設定欄位，不涉及生成的技能與 Agent 指令。

## Non-Goals

- CLI 的匯入與匯出動詞不做，維持列在已知缺口。
- 不提供「本機專案 → 第 1 版匯入包」的公開函式；第 1 版只留給 Speclink Desktop 連舊版 Server 時的退回路徑。
- 不做壓縮、不做部分匯出（只匯出某幾個變更）。
- 不改匯入的規則（只能進空的範圍、大小上限）。

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `node-sdk`: 新增一條需求——本機引擎的 `exportScope` 方法。
- `workspace-migration`: 新增一條需求——本機專案的匯出與 Speclink Desktop 的遷移產生相同的文件集合。

## Impact

- Affected specs: node-sdk、workspace-migration
- Affected crates: speclink-host（共用的匯入包組裝）、speclink-node（新方法與型別）、apps/desktop（改用共用的版本）
- Affected code:
  - New:
    - crates/adapters/speclink-node/__test__/export-local.spec.ts
  - Modified:
    - crates/host/speclink-host/src/transfer.rs
    - apps/desktop/src-tauri/src/remote.rs
    - crates/adapters/speclink-node/src/lib.rs
    - crates/adapters/speclink-node/index.js
    - crates/adapters/speclink-node/index.d.ts
    - docs/sdk-node.md
    - docs/sdk-node.zh-TW.md
    - docs/product-status.md
    - docs/product-status.zh-TW.md
  - Removed: (none)

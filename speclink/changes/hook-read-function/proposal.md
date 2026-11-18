## Why

宿主（內嵌了引擎的系統）用執行前掛鉤 `beforeCommand` 加上自己的規則，例如「審查通過、驗證通過、程式碼已落地，才能封存」。引擎刻意不內建這類規則：沒有蓋章的變更可以封存，這是正式規格 change-lifecycle 的明文。所以判斷只能由掛鉤做。

問題是掛鉤拿不到判斷需要的資料。它只收到身分、範圍、命令名稱與參數；`archive` 的參數是變更名稱與幾個旗標，沒有這個變更的審查、驗證與落地狀態。今天唯一的做法是在掛鉤裡再呼叫同一個宿主套件的 `dispatch` 去讀，但 SDK 文件禁止這樣做，而且那個內層呼叫會再問一次掛鉤，寫得不小心就無限遞迴。

受影響的人是用 Node SDK（或 Rust 的宿主套件）把引擎內嵌進自己後端的整合者，情境是封存前的把關，以及其他要看變更狀態才能決定放不放行的規則。來源討論 embedded-host-needs-2 把這一項列為「不解決就無法導入」，並裁定做法：把一個唯讀的讀取函式傳進掛鉤。

掃過的相關正式規格：host-kit（「執行前掛鉤可以拒絕命令」規定掛鉤收到什麼、只能放行或拒絕）、node-sdk（「createHost 與 host.dispatch」規定掛鉤的 JS 形狀）、change-lifecycle（封存的守門）、verb-contract（動詞的 `--json` 輸出）。前兩份要新增需求，後兩份不變。

## What Changes

- 執行前掛鉤多收到一個讀取函式。掛鉤用它執行讀取類的動詞，拿到和 `dispatch` 相同的結果（例如 `show <變更> --json` 的物件），再決定放行或拒絕。
- 讀取函式的規則：
  - 只接受不寫入任何文件的動詞（`list`、`show`、`status`、`plan`、`trace`、`evidence show`、`review show`、`verify show`、`discuss list`、`discuss show`、`discuss search`、`artifact cat`、`language show`、`workflow-config show`、`validate`、`analyze`、`instructions`）。其他動詞以 `invalid_argv` 拒絕，儲存不會收到任何呼叫。
  - 不再問一次執行前掛鉤，所以不會遞迴。
  - 沿用這次呼叫的身分與範圍，掛鉤不能換成別人或別的範圍。
  - 只在這次掛鉤執行期間有效；掛鉤結束後再呼叫會被拒絕。
- 兩個入口共用：`host.dispatch` 與 `host.handleRequest` 進來的呼叫，掛鉤都拿得到讀取函式。流程設定的寫入也一樣。
- Rust 的宿主套件同步提供：執行前掛鉤多一個參數，是一個可以執行讀取類命令的讀取器。
- SDK 的型別說明與文件改正一句話：「在掛鉤裡等同一個宿主套件的 dispatch 永遠不會完成」不符合實際行為。實際上它會完成，但會再問一次掛鉤，而且宿主的儲存自己持有鎖時可能卡住。新的說法是：要在掛鉤裡讀資料，用讀取函式。
- Node SDK 文件「一個動詞的四個步驟」的封存範例改成用讀取函式讀出審查、驗證與落地的狀態，中英兩版同步。

相容性影響：

- 不新增或變更任何 CLI 子指令、旗標、stdin 或 exit code；動詞的人眼輸出與 `--json` 輸出都不變。
- Node：`beforeCommand` 的第一個參數不變，新增第二個參數。只宣告一個參數的既有掛鉤照常運作。
- Rust：執行前掛鉤的函式型別多一個參數，這是原始碼層級的不相容。repo 內唯一的正式使用者是示範用 server，它沒有設定執行前掛鉤，所以只需要更新測試裡的掛鉤。
- 掛鉤的其他規則不變：每次呼叫正好問一次、只能放行或拒絕、不能改內容、拒絕發生在儲存收到任何呼叫之前（流程設定的寫入除外）。掛鉤自己用讀取函式讀資料時，儲存會收到那些讀取。

三種形態：

- 本機 repo：不適用。`createEngine` 沒有掛鉤；本機專案的規則由使用者自己在流程外處理。
- 連到遠端的 checkout：可用，在宿主那一端生效。CLI 與 Speclink Desktop 的請求經 `host.handleRequest` 進來時，宿主的掛鉤拿得到讀取函式。用戶端不用改，被拒絕時照舊看到宿主給的原因。連到示範用 server 時不適用：它沒有設定執行前掛鉤。
- 沒有本機規格檔的內嵌宿主：可用，這是本變更服務的對象。

不涉及設定欄位，不涉及生成的技能與 Agent 指令。

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `host-kit`: 新增一條需求，規定執行前掛鉤可以用讀取器執行讀取類的命令，以及讀取器的四條規則（只准讀、不再問掛鉤、沿用身分與範圍、和命令本身的讀取不是同一份）。
- `node-sdk`: 新增一條需求，規定 `beforeCommand` 第二個參數的 `read` 函式的 JS 形狀、回傳值與錯誤。

## Impact

- Affected specs: host-kit、node-sdk
- Affected crates: speclink-host（讀取器與讀取類命令的判定）、speclink-node（JS 的轉接與型別）；speclink-core、speclink-cli、speclink-server、apps/desktop 的程式碼不改
- Affected code:
  - New:
    - crates/host/speclink-host/src/read_only.rs
  - Modified:
    - crates/host/speclink-host/src/kit.rs
    - crates/host/speclink-host/src/lib.rs
    - crates/host/speclink-host/tests/common/mod.rs
    - crates/host/speclink-host/tests/kit.rs
    - crates/host/speclink-host/tests/http_door.rs
    - crates/host/speclink-host/tests/station_wire.rs
    - crates/host/speclink-host/tests/free_fields.rs
    - crates/host/speclink-host/tests/transfer.rs
    - crates/adapters/speclink-node/src/host.rs
    - crates/adapters/speclink-node/index.js
    - crates/adapters/speclink-node/index.d.ts
    - crates/adapters/speclink-node/__test__/host.spec.ts
    - crates/adapters/speclink-node/__test__/handle-request.spec.ts
    - docs/sdk-node.md
    - docs/sdk-node.zh-TW.md
  - Removed: (none)

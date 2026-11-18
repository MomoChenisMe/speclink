## Why

遠端專案的變更可以被認領（`speclink claim`）：引擎把認領的人與時間寫進變更的 metadata，別人再認領會被拒絕。這個功能今天有兩個缺口：

1. 認領不能釋放。認領錯了，或接手的人離開，沒有任何動詞能清掉認領。可是拒絕訊息卻寫著「ask them to release it」。
2. 只走動詞的使用者看不到認領的人。認領的人只出現在線上格式（給 Speclink Desktop 用）；`show --json` 沒有它，`claim` 也沒有 `--json`。認領人的識別（`claimed_by_id`）寫進了 metadata，但沒有任何輸出帶出它。宿主（內嵌了引擎的系統）裡的 AI Agent 只看得到動詞輸出，所以回答不出「誰接手了這個變更」；宿主的程式也拿不到識別來對照自己的帳號。

受影響的人：在宿主裡經 Agent 操作的非工程角色（問「誰在做這個」）、寫宿主程式的整合者（用識別對照帳號、在掛鉤裡依認領人判斷）、以及用 CLI 連到遠端專案的工程師（要放手一個認領錯的變更）。來源討論 embedded-host-needs-2 裁定：新增釋放認領，引擎不限定誰可以釋放；動詞輸出補上認領的人。

掃過的相關正式規格：change-lifecycle（「認領標記欄位」規定認領怎麼寫）、server-verb-api（「claim 端點持久化與 ownership 衝突語意」）、client-protocol（變更清單與單筆回應的欄位）、remote-workspace-data（Speclink Desktop 的認領面，不改）、change-landing（「分支與落地記錄的讀取」是 `show --json` 加選填欄位的前例）。

## What Changes

- 新增釋放認領：`speclink claim <change> --release`。
  - 清掉變更 metadata 的認領標記（認領的人、識別、時間），發出事件 `change-claim-released`。
  - 引擎不限定誰可以釋放：有寫入權限的人都可以，不必是認領的人。要限定的宿主用執行前掛鉤擋（命令名稱是 `claim-release`）。
  - 變更沒有被認領時成功結束、什麼都不寫、不發事件。
  - 本機專案照 `claim` 既有的方式拒絕（本機沒有認領）。
  - 人眼輸出：`✓ Released claim on change: <name> (was claimed by <holder>)`；沒有被認領時是 `Change '<name>' is not claimed — nothing to release`。兩者 exit code 都是 0。
- `claim` 加 `--json`：
  - `speclink claim <change> --json` 輸出 `{ "change", "claimedBy" }`。
  - `speclink claim <change> --release --json` 輸出 `{ "change", "released", "previousHolder" }`；`previousHolder` 只在釋放前有認領的人時出現。
- `speclink show <change> --json` 增列選填的 `claimed` 物件：`by`（認領的人）、`byId`（識別，有記錄才出現）、`at`（認領的時間，有記錄才出現）。沒有被認領時不輸出這個鍵。
- 線上格式：
  - 新增 `DELETE /changes/{name}/claim`，回應 `{ "released", "previousHolder" }`。
  - 變更清單與單筆回應在既有的 `claimedBy` 之外，新增選填的 `claimedById` 與 `claimedAt`。
- 事件 `change-claim-released` 歸到該變更，用戶端收到後重新讀取這個變更。
- 使用者文件：動詞契約文件補上新端點、新事件與新欄位；專案能力狀態補上釋放認領；中英兩版同步。

相容性影響：

- `speclink claim <change>`（不帶新旗標）的人眼輸出與 exit code 不變。
- `show` 的人眼輸出不變。`show --json` 對沒有被認領的變更逐位元不變；被認領的變更多一個 `claimed` 鍵，這是刻意新增的輸出。
- `list --json` 不變（review-station 與 verify-station 固定它的欄位；清單的詳細輸出由另一個變更處理）。
- 線上格式只新增選填欄位與一個端點。舊版用戶端忽略新欄位；新版用戶端連到舊版 Server 時，`claim --release` 得到 404，`show --json` 的 `claimed` 只有 `by`。
- 引擎新增一個命令，命令名稱與參數的 golden 同批更新。

三種形態：

- 本機 repo：不適用。本機專案沒有認領；`claim` 與 `claim --release` 都以既有的訊息拒絕。`show --json` 可用，本機的變更沒有認領記錄所以不會出現 `claimed`。
- 連到遠端的 checkout：可用。CLI 的 `claim --release`、`claim --json`、`show --json` 經線上格式取得相同的結果。
- 沒有本機規格檔的內嵌宿主：可用。`host.dispatch` 執行同樣的動詞，輸出相同；釋放認領會先問執行前掛鉤。

不涉及設定欄位。生成的技能不改：沒有技能使用 `claim`。Agent 工具的動詞清單由引擎的動詞定義產生，會自動帶出新旗標。

## Non-Goals

- Speclink Desktop 不加釋放認領的按鈕；這一輪由 CLI 與宿主的 Agent 提供。列進專案能力狀態的已知缺口。
- 引擎不加「只有認領的人或管理者才能釋放」的規則；這類規則屬於宿主。
- 不改認領本身的規則（同一身分重複認領冪等、別人認領被拒）與它的拒絕訊息。
- 不改 `list --json`。

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `change-lifecycle`: 新增兩條需求——釋放認領的引擎語意，以及認領記錄經 `show --json` 與 `claim --json` 的讀取。
- `server-verb-api`: 新增一條需求——釋放認領的端點與它的回應、權限與錯誤。
- `client-protocol`: 新增一條需求——變更清單與單筆回應的 `claimedById`、`claimedAt`，以及釋放認領回應的 payload。

## Impact

- Affected specs: change-lifecycle、server-verb-api、client-protocol
- Affected crates: speclink-core（命令、事件、show 的結果）、speclink-verbs（解析與輸出）、speclink-protocol（線上型別）、speclink-host（路由、事件歸類、讀取類命令以外的對照表）、speclink-remote（用戶端方法）、speclink-cli（遠端分支）；speclink-server 與 apps/desktop 只更新手寫線上型別字面值的地方（測試與一處組裝），行為不改；speclink-node 只加一個測試，程式碼不改
- Affected code:
  - New: (none)
  - Modified:
    - crates/engine/speclink-core/src/command/mod.rs
    - crates/engine/speclink-core/src/command/describe.rs
    - crates/engine/speclink-core/src/command/tests.rs
    - crates/engine/speclink-core/src/lifecycle/model.rs
    - crates/engine/speclink-core/tests/golden/command-describe.json
    - crates/adapters/speclink-verbs/src/parse.rs
    - crates/adapters/speclink-verbs/src/inputs.rs
    - crates/adapters/speclink-verbs/src/tool.rs
    - crates/adapters/speclink-verbs/src/render/mod.rs
    - crates/adapters/speclink-verbs/src/render/lifecycle.rs
    - crates/adapters/speclink-verbs/src/render/query.rs
    - crates/protocol/speclink-protocol/src/command.rs
    - crates/protocol/speclink-protocol/src/query.rs
    - crates/protocol/speclink-remote/src/client.rs
    - crates/host/speclink-host/src/commit.rs
    - crates/host/speclink-host/src/http/coverage.rs
    - crates/host/speclink-host/src/http/routes.rs
    - crates/host/speclink-host/src/http/wire.rs
    - crates/host/speclink-host/tests/http_door.rs
    - crates/host/speclink-host/tests/kit.rs
    - crates/adapters/speclink-cli/src/main.rs
    - crates/adapters/speclink-cli/src/verbs/lifecycle.rs
    - crates/adapters/speclink-cli/src/verbs/query.rs
    - crates/adapters/speclink-cli/tests/it/remote_verb_parity.rs
    - crates/host/speclink-server/tests/it/api/command_routes.rs
    - apps/desktop/src-tauri/src/remote.rs
    - apps/desktop/src-tauri/tests/it/remote_data.rs
    - crates/adapters/speclink-node/__test__/host.spec.ts
    - docs/verb-contract.md
    - docs/verb-contract.zh-TW.md
    - docs/product-status.md
    - docs/product-status.zh-TW.md
  - Removed: (none)

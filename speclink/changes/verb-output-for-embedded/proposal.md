## Why

宿主（內嵌了引擎的系統）裡的 AI Agent 只能經 `host.dispatch` 執行動詞，並且只看得到動詞的輸出。這條路今天有兩個問題：

1. 清單沒有狀態。`speclink list --json` 的每一項沒有審查與驗證的狀態、認領的人、分支與落地記錄。這是刻意的：正式規格 review-station 與 verify-station 各有一條需求固定它的欄位，因為很多技能會呼叫它。可是線上格式 `GET /changes` 有這些資料（給 Speclink Desktop 的看板用）。結果是 Agent 要回答「哪些變更已經通過審查」「我認領了哪些」時，只能對每個變更各呼叫一次 `show`；宿主要做總覽畫面時，只能在自己的程序裡呼叫 HTTP 入口。
2. 輸出印出不存在的路徑。`new change`、`discuss promote` 與 `discuss new` 的人眼輸出有一行 `Path: speclink/changes/demo` 這樣的路徑。本機專案裡那是真的資料夾；內嵌時規格存在宿主的儲存裡，沒有這個資料夾。Agent 會把它當成可以打開的檔案。CLI 連到遠端專案時，正式規格 verb-contract 早已規定前兩個動詞不印這一行，理由相同；內嵌是那條規定沒有涵蓋到的執行方式。

受影響的人：在宿主裡經 Agent 做 discuss 與 propose 的非工程角色，以及寫宿主程式（總覽畫面、Agent 工具）的整合者。來源討論 embedded-host-needs-2 裁定：清單的預設輸出不動，加一個旗標；內嵌時不印路徑行，JSON 的 `path` 保留並在 Agent 工具的說明寫明它是儲存的鍵。

掃過的相關正式規格：verb-contract（清單的 payload、「動詞人眼輸出的兩模式同形」的分歧清單）、review-station 與 verify-station（各一條固定 `list --json` 的需求）、client-protocol（`GET /changes` 的欄位）、node-sdk（「Agent 工具定義的匯出」）、change-landing 與 change-lifecycle（`show --json` 的 `landed`、`claimed` 是這次沿用的鍵名）。

## What Changes

- `speclink list --json` 新增旗標 `--detail`。帶上它時，變更清單的每一項多出這些選填的鍵，有記錄才出現：
  - `review`、`verify`：關卡的狀態，物件的形狀與 `show --json` 的同名鍵相同。
  - `claimed`：認領的人（`by`）、識別（`byId`）、時間（`at`），形狀與 `show --json` 相同。
  - `branch`：開工時記下的分支。
  - `landed`：落地記錄，清單裡只有 `commit`。
- 不帶 `--detail` 的 `list --json` 逐位元不變。`--detail` 只能和 `--json` 一起用，不能和 `--specs` 一起用；違反時以用法錯誤拒絕。
- 資訊量對齊線上格式 `GET /changes`：CLI 連到遠端專案時用那個回應組出相同的輸出，不新增端點。
- `host.dispatch` 的人眼輸出不再印三個動詞的路徑行：`new change` 的 `Path:` 行、`discuss promote` 的 `Path:` 行與緊接的 propose 提示行、`discuss new` 的 `Path:` 行。JSON 輸出的 `path` 鍵保留。
- Agent 工具的說明（`agentTool.description`）加一句：結果裡的 `path` 是文件在儲存裡的鍵，不是可以打開的檔案；讀文件要用 `artifact cat`、`show`、`discuss show`。
- 使用者文件：動詞契約文件補上 `--detail` 與它的欄位；Node SDK 文件的 Agent 工具一節說明路徑行與 `path` 鍵；專案能力狀態同步；中英兩版。

相容性影響：

- CLI 在本機專案與遠端專案的人眼輸出全部不變。`createEngine`（本機資料夾）的 `dispatch` 輸出也不變。
- `host.dispatch` 對三個動詞回傳的 `{ output }` 文字少了路徑行，這是刻意的變更。它們的 `--json` 形式不變。
- `list --json` 的預設輸出不變，review-station 與 verify-station 固定欄位的兩條需求改寫成「不帶 `--detail` 時」，原本的保證完整保留。
- 引擎的 `list` 命令多一個參數，命令名稱與參數的 golden 同批更新。

三種形態：

- 本機 repo：`list --json --detail` 可用；本機專案沒有認領，所以不會出現 `claimed`，其他鍵依記錄出現。路徑行照常印（那是真的資料夾）。
- 連到遠端的 checkout：`list --json --detail` 可用，內容來自 `GET /changes`。路徑行的行為不變（`new change` 與 `discuss promote` 本來就不印）。
- 沒有本機規格檔的內嵌宿主：`list --json --detail` 可用；三個動詞不印路徑行。這是本變更服務的對象。

不涉及設定欄位。生成的技能不改：技能繼續用不帶 `--detail` 的 `list --json`。Agent 工具的動詞清單由引擎的動詞定義產生，會自動帶出新旗標；說明文字多一句，影響 `agentTool` 的全部使用者。

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `verb-contract`: 新增兩條需求——清單的詳細輸出（`--detail`），以及內嵌宿主的人眼輸出不印路徑行。
- `review-station`: 「CLI 清單輸出的相容性釘住」改寫成只約束不帶 `--detail` 的輸出。
- `verify-station`: 「CLI 清單輸出的驗證欄位釘住」同樣改寫。
- `node-sdk`: 新增一條需求——Agent 工具的說明指出 `path` 是儲存的鍵。

## Impact

- Affected specs: verb-contract、review-station、verify-station、node-sdk
- Affected crates: speclink-core（清單命令與每一項的資料）、speclink-verbs（解析、輸出、Agent 工具說明）、speclink-node（宿主的輸出入口）、speclink-cli（遠端分支）；speclink-host、speclink-protocol、apps/desktop 的程式碼不改
- Affected code:
  - New: (none)
  - Modified:
    - crates/engine/speclink-core/src/command/mod.rs
    - crates/engine/speclink-core/src/command/describe.rs
    - crates/engine/speclink-core/src/lifecycle/listing.rs
    - crates/engine/speclink-core/tests/golden/command-describe.json
    - crates/adapters/speclink-verbs/src/parse.rs
    - crates/adapters/speclink-verbs/src/inputs.rs
    - crates/adapters/speclink-verbs/src/tool.rs
    - crates/adapters/speclink-verbs/src/render/mod.rs
    - crates/adapters/speclink-verbs/src/render/new.rs
    - crates/adapters/speclink-verbs/src/render/discuss.rs
    - crates/adapters/speclink-verbs/src/render/query.rs
    - crates/adapters/speclink-node/src/lib.rs
    - crates/adapters/speclink-node/src/host.rs
    - crates/adapters/speclink-node/index.d.ts
    - crates/adapters/speclink-node/__test__/host.spec.ts
    - crates/adapters/speclink-node/__test__/agent-tool.spec.ts
    - crates/adapters/speclink-cli/src/verbs/query.rs
    - crates/adapters/speclink-cli/tests/it/remote_verb_parity.rs
    - crates/host/speclink-host/tests/port_cost.rs
    - docs/verb-contract.md
    - docs/verb-contract.zh-TW.md
    - docs/sdk-node.md
    - docs/sdk-node.zh-TW.md
    - docs/product-status.md
    - docs/product-status.zh-TW.md
  - Removed: (none)

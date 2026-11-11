## Why

流程設定（`config.yaml`）的讀寫今天完全不經引擎的命令層。CLI 在本機模式直接讀寫檔案，在遠端模式直接送出整份文件；示範用 server 的寫入路由自己解析、自己比對版本、自己開交易。引擎的 43 種命令裡沒有任何一種碰流程設定，正式規格還明寫 `workflow-config show`「不進入命令執行層」。

結果是內嵌的宿主拿不到這條路徑：Node SDK 的 dispatch 不認得 `workflow-config`；之後宿主的執行前檢查也看不到「誰要改哪個政策鍵」，因為寫入根本不經過它能攔的地方。需求文件的用途之一就是「政策鍵只有特定的人能改」。

討論 embedded-host-needs 定案：流程設定的讀寫新增為引擎命令。這是「動詞共用一份」拆出的第三個變更，排在 shared-verb-output 之後。

- **目標使用者**：把 speclink 內嵌進自己後端的整合者（之後能在執行前檢查裡依變動的鍵決定放行或拒絕）；在宿主的 AI Agent 裡調整流程設定的 PO、PM。使用 CLI 的開發者看到的行為不變。
- **使用情境**：`/speclink-config` 技能與任何呼叫 `speclink workflow-config` 的步驟。
- **既有規格的對照**：workflow-config 規定 show 與各寫入動詞的行為，其中 show 明寫不進入命令執行層；command-runtime 的覆蓋表沒有列 workflow-config；node-sdk 的 dispatch 沒有這組動詞。server-policy-write 規定線上的寫入要帶預期版本並經引擎驗證，這個變更不改它的可觀察行為。

## What Changes

- `speclink-core` 新增三種命令：
  - **讀取**：回傳解析後的流程設定，供 show 的人眼與 `--json` 輸出使用。
  - **編輯**：收一筆編輯（設定一個政策欄位、取代或移除 context、取代或移除某個 artifact 的規則、改審查規則的一個清單或 baseline、改子代理模型的設定），由引擎讀出現有內容、套用、驗證、寫回。帶 dry-run 時只計算、不寫入。
  - **整份取代**：收整份新的內容，驗證後寫入。給線上的整份寫入用。
- 兩種寫入命令的結果都帶：寫入前的內容、寫入後的內容、變動的鍵（例如 `tdd`、`rules.proposal`、`review.critical`）、是否真的寫入。內容沒有變動時不寫入。
- 驗證失敗（YAML 無法解析、locale 不合法）在寫入前以 `invalid_config` 拒絕。
- 引擎內層的儲存介面新增「寫入流程設定」的方法。檔案實作、Host 的儲存視圖、測試用的儲存各自實作；沒有實作的儲存預設拒絕。
- `speclink-verbs` 收進 workflow-config 這個家族：clap 定義、轉成命令、輸出（含 dry-run 的差異）都搬進去，成為第十二個共用的家族。
- `speclink-cli` 的本機分支改成執行引擎命令。需要 checkout 的前後處理留在 CLI：關閉 worktree 前檢查有沒有進行中的 worktree、worktree 欄位變動後同步技能檔、審查規則變動後的技能檔過期提示。遠端分支送出的請求不變。
- 示範用 server 的整份寫入路由改成執行「整份取代」命令。角色檢查、解析失敗先於版本衝突的順序、回應的形狀都不變。
- `speclink-node`：dispatch 能用 `workflow-config` 的全部動詞。宿主的 JS 儲存新增一個選填方法 `writeWorkflowConfig(content)`；沒有提供時，寫入動詞在動手前拒絕。
- **相容性影響**：
  - CLI 的指令、旗標、`--help`、人眼輸出、`--json`、`--dry-run` 的差異、exit code 都不變，只有一種例外：值沒有變的寫入不再有後續動作。變更前，`set worktree <值>` 在值沒有變時仍會重新同步技能檔，`review …` 與 `subagent-model …` 仍會印技能檔過期提示，專案沒有 config.yaml 時還會建一個空檔；變更後這三件事都不發生，確認訊息照常輸出。
  - shell 補全腳本裡 workflow-config 在子指令清單的位置改變，內容不變。
  - 示範用 server：內容與現有流程設定相同的整份寫入不再讓版本前進。
  - CLI 的 remote 模式：過期提示和本機模式用同一條規則，值沒有變時不提示（變更前會提示）。寫入照常送出。
  - 線上協定不變。
  - Node SDK：`workflow-config` 開頭的 argv 原本回傳 `invalid_argv`，現在會執行。argv 帶 `--stdin` 而沒有傳 stdin 內容時以 `invalid_argv` 拒絕（空字串照常執行）。
  - 沒有設定欄位的新增或變更；`config.yaml` 的格式不變，既有檔案照常讀取。
- 影響的技能：`/speclink-config` 的文字不變（它呼叫的動詞沒有改）。claude 與 codex 兩種工具都不用重新產生技能檔。

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `workflow-config`: 「workflow-config show 動詞」拿掉「不進入命令執行層」；新增一條——流程設定的寫入經命令執行層，結果列出變動的鍵。
- `command-runtime`: 「動詞覆蓋與跨入口一致性」的覆蓋範圍加入 workflow-config 的讀取與寫入動詞。
- `node-sdk`: 新增一條——dispatch 的流程設定動詞，以及宿主儲存的選填寫入方法。

## Impact

- Affected specs: workflow-config（修改一條、新增一條）、command-runtime（修改一條）、node-sdk（新增一條）
- Affected crates: `speclink-core`、`speclink-fs`、`speclink-host`、`speclink-verbs`、`speclink-cli`、`speclink-server`、`speclink-node`。store crates 與桌面 app 不改。
- Affected code:
  - New:
    - crates/engine/speclink-core/src/command/workflow_config.rs
    - crates/adapters/speclink-verbs/src/families/workflow_config.rs
    - crates/adapters/speclink-verbs/src/render/workflow_config.rs
    - crates/adapters/speclink-node/__test__/workflow-config-verbs.spec.ts
  - Modified:
    - crates/engine/speclink-core/src/command/mod.rs
    - crates/engine/speclink-core/src/command/typed.rs
    - crates/engine/speclink-core/src/command/tests.rs
    - crates/engine/speclink-core/src/store.rs
    - crates/engine/speclink-core/src/teststore.rs
    - crates/engine/speclink-core/src/workspace/config.rs
    - crates/engine/speclink-core/src/workspace/config/tests.rs
    - crates/engine/speclink-fs/src/lib.rs
    - crates/engine/speclink-fs/tests/store_fs.rs
    - crates/host/speclink-host/src/port_view.rs
    - crates/host/speclink-host/src/port_exec.rs
    - crates/host/speclink-host/src/bridge.rs
    - crates/host/speclink-host/tests/bridge_dual_path.rs
    - crates/host/speclink-server/src/api/routes.rs
    - crates/host/speclink-server/tests/it/api/policy_write.rs
    - crates/host/speclink-server/tests/it/api/command_routes.rs
    - crates/host/speclink-server/tests/it/common/mod.rs
    - crates/adapters/speclink-verbs/src/families/mod.rs
    - crates/adapters/speclink-verbs/src/inputs.rs
    - crates/adapters/speclink-verbs/src/render/mod.rs
    - crates/adapters/speclink-cli/src/main.rs
    - crates/adapters/speclink-cli/src/color.rs
    - crates/adapters/speclink-cli/src/verbs/config.rs
    - crates/adapters/speclink-cli/tests/it/workflow_config.rs
    - crates/adapters/speclink-cli/tests/it/init_tools.rs
    - crates/adapters/speclink-node/src/lib.rs
    - crates/adapters/speclink-node/src/store_bridge.rs
    - crates/adapters/speclink-node/index.d.ts
    - crates/adapters/speclink-node/__test__/helpers.ts
    - crates/adapters/speclink-node/__test__/dispatch-contract.spec.ts
  - Removed: (none)
- 三種形態下的結果：
  - 本機 repo：可用。CLI 的 `workflow-config` 改經引擎命令，行為與輸出不變。
  - 連到遠端的 checkout：可用。CLI 送出的請求不變；示範用 server 收到整份寫入後改經引擎命令。
  - 沒有本機規格檔的內嵌宿主：可用，有一項已知缺口。SDK 的 dispatch 能讀寫流程設定；變動的鍵在引擎命令的結果裡，dispatch 回傳給 JS 的只有確認訊息；worktree 欄位變動後的技能同步與技能檔過期提示不適用（宿主沒有 checkout，技能之後由遠端提供）。執行前檢查本身還沒有，由變更 host-kit-agent-door 補上。
- 桌面 app 的本機寫入（apps/desktop/core/src/settings.rs）仍直接寫檔，這次不改：它只發生在本機專案，沒有宿主的檢查要經過；編輯的邏輯本來就和 CLI 共用引擎的同一個函式。
- 文件影響（留給最後一個變更 docs-embedded-host，這個變更不動使用者文件）：docs/sdk-node.zh-TW.md 與 docs/sdk-node.md 要補上流程設定的動詞與選填的儲存方法。

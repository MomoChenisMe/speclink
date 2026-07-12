## Why

平台架構藍圖（docs/platform-architecture.zh-TW.md §4.1、§14 Phase 1）的不變式是「同一套流程語意只有一份 Rust 實作」，但現況有三個入口各自組裝引擎呼叫：CLI 的 handler 直呼 core 函式並自行渲染（`crates/speclink-cli/src/commands.rs`）、Node SDK 以手刻 argv router 重組 list/status/new/claim 四個動詞（`crates/speclink-node/src/lib.rs`）、桌面 app 的 Tauri 命令再直呼一次 core。沒有共同的命令層，動詞行為一改就要多處同步；Phase 2 的 server 一旦成為第四個入口，語意漂移風險倍增。同時設定解析是 fail-open：`.speclink.yaml` 或 `openspec/config.yaml` 存在但損壞時靜默退回預設——壞 `.speclink.yaml` 被當成「無 remote」、壞 `config.yaml` 使工作流政策整份消失——是藍圖 §15 P0 明列的正確性漏洞。

目標使用者：透過 AI 代理跑 SDD 的開發者、PO 與 PM——discuss/propose/apply/verify/archive 全部 workflow 階段的動詞都經過這一層；直接受益者是 Node SDK 整合者與後續 server、desktop 遠端刀（它們將以本刀的 typed 介面為地基）。

## What Changes

- `speclink-core` 新增 typed Command Runtime：每個觸及 Store 的動詞有 typed command 輸入、typed outcome 輸出與帶穩定錯誤碼的 typed error，取代「各入口自行組裝 core 函式」。
- 變更型動詞（new change、new artifact、task done、task undone、claim、in-progress、archive、discard 與 discuss 系列等）成功時發出 typed domain events；事件隨 outcome 回傳。本刀只固定事件契約與發出點，不做持久化與訂閱。
- `speclink-cli` 的 handler 改為經 runtime 取得 outcome 後渲染；人眼輸出與 `--json` 形狀不變。
- `speclink-node` 的 dispatch 保留現有 argv 介面與回傳 envelope 作相容層，內部改路由到 runtime；錯誤碼改由統一 typed error 映射產生。
- **BREAKING（刻意的行為變更）**：`.speclink.yaml` 與 `openspec/config.yaml`「存在但解析失敗」時，所有入口回 typed error 並停止（CLI 以非零 exit code 結束並印出指向壞檔的錯誤），不再靜默退回預設；只有「檔案不存在」才允許預設。壞 `.speclink.yaml` 從此不再被解讀為 fs 模式。

## Non-Goals

- 不動 Store trait、revision、CAS、Unit of Work（teamstore-contract 刀）。
- 不做事件持久化、outbox 與任何訂閱/推播（teamstore-contract 與 server 刀）。
- 不動 Project/Repo binding 與隱式 workspace/git identity（binding-and-policy 刀）。
- 不遷移桌面 app 的直呼路徑（Phase 3 WorkspaceSession 重構時一併收編）。
- 不擴增 dispatch 的動詞覆蓋（維持 list/status/new/claim），不改其 envelope 形狀。
- 不改任何既有動詞的人眼與 `--json` 輸出形狀；唯一新增輸出是壞設定檔的錯誤訊息。
- init、update、config（管理使用者層設定檔，非 store 領域）、schema（schema 檔工具）、completion、templates、feedback、demo 等 workspace bootstrap 與周邊工具動詞不進 runtime；remote 模式的 HTTP 攔截路徑（`crates/speclink-cli/src/remote_commands.rs`）維持現狀。
- 不引入生命週期狀態機閘門（drafting→review→⋯）——該閘門依賴 revision 語意，屬後續刀。

## Capabilities

### New Capabilities

- `command-runtime`: 引擎動詞的唯一 typed 執行層——command/outcome/error 契約、domain events 的種類與發出點、CLI 與 Node dispatch 一律經此層執行。

### Modified Capabilities

- `workflow-config`: `openspec/config.yaml` 存在但解析失敗時改為 typed error 並停止（原行為：靜默退回預設，政策全滅）。
- `remote-connection`: `.speclink.yaml` 存在但解析失敗時模式解析 fail-closed（原行為：靜默退回預設，被當成 fs 模式）。
- `node-sdk`: dispatch 的輸入輸出契約改述為 runtime 相容層——argv 介面與 envelope 不變，錯誤碼出自與 CLI 共用的統一映射。

## Impact

- 相容性影響：人眼與 `--json` 輸出形狀不變，parity/color/twin 回歸對照必須全綠；唯一刻意變更是壞設定檔由「靜默預設」改為「報錯停止」，既有使用者若無意間依賴壞檔靜默行為，修正該設定檔即可。Node dispatch 的 envelope 與錯誤碼字串維持既有值域。
- Affected specs: `command-runtime`（新增）、`workflow-config`、`remote-connection`、`node-sdk`（修改）。
- Affected code:
  - New: crates/speclink-core/src/command/mod.rs（runtime 與 typed command/outcome/error/event 模組）
  - Modified: crates/speclink-core/src/config.rs、crates/speclink-core/src/workspace.rs、crates/speclink-core/src/instructions.rs、crates/speclink-core/src/init.rs、crates/speclink-core/src/discuss.rs、crates/speclink-cli/src/commands.rs、crates/speclink-node/src/lib.rs、apps/desktop/core/src/settings.rs（typed 載入的機械式跟進）、crates/speclink-node/index.d.ts 與 crates/speclink-remote/src/lib.rs（清除指向已移除文件的註解）
  - Removed: 無

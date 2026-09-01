## Why

「哪個 Command 產哪個 outcome」的對應規則由引擎的 execute 實作，卻在三個消費端各自重新斷言：CLI 動詞層 28 處 let-else 加 unreachable! 拆封、server 約 20 個 wrong_outcome 兜底臂、node SDK 8 處。CLI 側每個 fs 臂都要重複「開專案、&dyn 轉型、執行、拆封」四行儀式（轉型 21 處、拆封 28 處），interface 的學習成本逼近 implementation——這是 improve-cli-verb-layer 討論（2026-09-01 結論）裁定要收的第一刀。

目標使用者是本專案的維護者與經由 AI 代理跑 SDD 流程的開發者；使用情境是日後每次新增或修改 CLI 動詞（propose → apply 階段觸碰 fs 臂）時，不再手寫拆封儀式、也不可能拆錯臂。

## What Changes

- speclink-core：在 command 模組新增型別化轉換層——為 CLI 現行消費的 24 個 outcome payload 型別各補一支 TryFrom<CommandOutcome>（additive，不動 Command 與 CommandOutcome 既有形狀）。同一 payload 型別由多個 variant 共用時（TaskFlipOutcome 之於 TaskDone/TaskUndone、DiscussBindOutcome 之於 DiscussLink/DiscussSeal、DiscussSubjectOutcome 之於 DiscussContext/DiscussDiscard、String 之於 ArtifactCat/Language），該支轉換接受所有載同型別的 variant。轉換失敗回傳新增的 WrongOutcome 錯誤型別（載期望型別名與實際 variant 名）。
- speclink-cli：common.rs 的 run_command 改為薄泛型入口 run（同參數、回傳泛型 T），吸收 &dyn 轉型、ExecutionContext 組裝、execute 與 outcome 轉換；9 個動詞族檔共 29 處呼叫點遷移——28 處拆封改為一行型別化呼叫、1 處丟棄式呼叫（in-progress add）明標型別，21 處 let store: &dyn Store 轉型行刪除；verbs/ 內 unreachable! 歸零。
- open_project 維持獨立不動（discuss 族一次開專案供 11 次呼叫、多個臂在引擎呼叫後仍使用 workspace，吸收不合身）。
- CLI 指令面零變更：無新子指令、無旗標變更、stdin 與 exit code 照舊。
- 相容性影響：無——人眼輸出與 --json 逐位元不變，command-runtime 規格的「覆蓋動詞輸出凍結」scenario 即驗證網；既有回歸對照全數照跑。

## Non-Goals

- 不遷移 server（routes.rs 的 wrong_outcome 臂）與 node SDK 的拆封——core 層就位後各自留作後續小刀（討論 Deferred 記錄在案）。
- 不吸收 open_project、不動 dispatch 模式表（cli-mode-dispatch-convergence 定案）、不動渲染單份化（cli-render-unification 定案）、不動族檔硬規則。
- 不為 CLI 未消費的 outcome 型別（Claim、TaskMove、Review 與 Verify 家族）補轉換——出現消費端時再補，不預先鋪彈性。
- 不涉及設定欄位（openspec/config.yaml、.speclink.yaml）與技能注入區塊。

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

(none)——規格掃描比對：最近的既有規格為 command-runtime（動詞執行期跨入口一致性與輸出凍結）。本刀是純內部重構，不改任何 SHALL；command-runtime 的「覆蓋動詞輸出凍結」scenario 反而是本刀的驗收條件，故不出 delta。

## Impact

- Affected specs: none（command-runtime 為驗證網，不修訂）
- Affected code:
  - New: crates/speclink-core/src/command/typed.rs（WrongOutcome 與 TryFrom 轉換層）
  - Modified: crates/speclink-core/src/command/mod.rs、crates/speclink-cli/src/common.rs、crates/speclink-cli/src/verbs/query.rs、crates/speclink-cli/src/verbs/discuss.rs、crates/speclink-cli/src/verbs/progress.rs、crates/speclink-cli/src/verbs/lifecycle.rs、crates/speclink-cli/src/verbs/checks.rs、crates/speclink-cli/src/verbs/new.rs、crates/speclink-cli/src/verbs/documents.rs、crates/speclink-cli/src/verbs/instructions.rs、crates/speclink-cli/src/verbs/trace.rs、crates/speclink-cli/src/verbs/station.rs（不走 run，僅刪 2 行 &dyn 轉型——21 處刪除帳含此 2 處）
  - Removed: none

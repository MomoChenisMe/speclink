## Context

引擎的 execute 以 CommandOutcome sum type 回傳結果，「哪個 Command 產哪個 outcome」的對應由 execute 實作，但消費端各自重新斷言：CLI 動詞層 28 處 let-else 加 unreachable! 拆封加 1 處丟棄式呼叫（in-progress add）、server 約 20 個 wrong_outcome 兜底臂、node SDK 8 處。CLI 的 fs 臂另有 21 處 let store: &dyn Store 轉型行。來源討論 improve-cli-verb-layer（2026-09-01）已裁定：型別轉換下沉 core（additive）、CLI 收薄泛型入口、server 與 node 留作後續小刀。

## Goals / Non-Goals

**Goals:**

- CommandOutcome 到各 payload 型別的轉換只存在一份，落在擁有此不變式的 speclink-core
- CLI 的 fs 臂引擎呼叫縮成一行型別化呼叫；verbs/ 內 unreachable! 歸零、&dyn 轉型行刪除
- 人眼輸出與 --json 逐位元不變（command-runtime 規格「覆蓋動詞輸出凍結」scenario 為驗收網）

**Non-Goals:**

- 不遷移 server 與 node SDK 的拆封（core 層就位後各自的後續小刀）
- 不吸收 open_project、不動 dispatch 模式表、不動渲染單份化與族檔硬規則
- 不為 CLI 未消費的 outcome 型別（Claim、TaskMove、Review 與 Verify 家族）補轉換
- 不改任何 CLI 指令語法、旗標、stdin 行為與 exit code

## Decisions

**D1：轉換以 payload 型別為單位，不以 variant 為單位。** 為 CLI 消費的 24 個 payload 型別各補一支 TryFrom<CommandOutcome>；同一型別由多個 variant 共用時，該支轉換接受全部載此型別的 variant——TaskFlipOutcome 收 TaskDone 與 TaskUndone、DiscussBindOutcome 收 DiscussLink 與 DiscussSeal、DiscussSubjectOutcome 收 DiscussContext 與 DiscussDiscard、String 收 ArtifactCat 與 Language。理由：呼叫端要的是 payload；variant 級區分不增加安全性（Command 對 variant 的對應是引擎自身的不變式），硬做要為共用型別造 marker newtype，儀式繞一圈回來。捨棄案：per-variant newtype 包裝（拆封儀式換個形狀重生）；自訂 FromOutcome trait（std TryFrom 即可，CommandOutcome 是本地型別、orphan rule 無礙）。

**D2：mismatch 是錯誤值，不是 panic。** core 新增 WrongOutcome 錯誤型別（載期望型別名與實際 variant 名，實作 Display 與 std::error::Error，訊息為英文、無 ANSI）。CLI 入口以 ? 上拋為 anyhow 錯誤。實務上不會觸發（引擎不變式保證），選錯誤值而非 unreachable! 的理由：server 後續遷移時 wrong_outcome 臂可直接以同一型別對映，且 core 作為函式庫不該替呼叫端決定 panic。

**D3：24 支 impl 以檔內 macro_rules! 生成，落在新檔 crates/speclink-core/src/command/typed.rs。** 表驅動一行一型別（型別 => 接受的 variant 清單），macro 不匯出；typed.rs 同時收 WrongOutcome 與取 variant 名的私有輔助。command 模組的 mod.rs 只加 mod typed 與 pub use。理由：24 支手寫 impl 約兩百行同構樣板，表驅動把「哪型別收哪些 variant」壓成一眼可讀的一張表。捨棄案：手寫 24 支（樣板噪音）；proc-macro 或 derive（為一張本地表引整套編譯依賴，違反專案「不加沒被要求的彈性」）。

**D4：CLI 入口 run 取代 run_command，不是包住它。** common.rs 的 run_command 改名並泛型化為 run，簽名維持三參數（store: &dyn Store、ws: Option<&Workspace>、cmd: Command），回傳 Result<T>，約束 T: TryFrom<CommandOutcome, Error = WrongOutcome>；函式體為既有 ExecutionContext 組裝加 execute 加 T::try_from。全部 29 處呼叫點遷移：28 處拆封點改為 let 具型別綁定的一行呼叫；in-progress add 的丟棄式呼叫改為 let 底線綁定並標註 InProgressOutcome 型別。呼叫點的 &dyn 轉型行刪除——FsStore 參照在引數位置自動 coerce，info_if_no_changes 等其他 &dyn 消費者逐呼叫傳 &store 即可。理由：留兩個入口（run_command 與 run）等於留下回退到舊儀式的門。

## Implementation Contract

- **Behavior**：使用者可觀察行為零變更。全部 CLI 動詞的人眼輸出、--json、stderr 與 exit code 逐位元一致；WrongOutcome 錯誤路徑在現行引擎下不可觸發。
- **Interface / data shape**：core 新增 pub struct WrongOutcome 與 24 支 TryFrom<CommandOutcome> impl（詳 D1 表）；CLI 的 common.rs 以 pub(crate) fn run 取代 run_command，簽名見 D4。Command 與 CommandOutcome 本體零變更，已發佈的 node API 不受影響。
- **Verification**：
  - 新增 core 單元測試（typed.rs 檔內）：成功轉換、錯 variant 回 WrongOutcome 且訊息含兩個型別名、共用 payload 型別接受其全部 variant（以 TaskFlipOutcome 對 TaskDone 與 TaskUndone 為代表）
  - cargo test -p speclink-core 與 cargo test -p speclink-cli --test it 全綠（凍結輸出整合測試即位元級驗收）
  - 遷移完成斷言：grep 檢查 crates/speclink-cli/src/verbs/ 內 unreachable! 為零、let store: &dyn Store 為零、run_command 識別符為零；cargo build 零 dead-code warning

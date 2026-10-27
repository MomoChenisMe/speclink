## Why

Windows CI 的 build-and-smoke job 掛在 `new_artifact::a_canonical_capability_keeps_the_exact_success_output`(run 33133958088,418 過 1 敗)。這批測試隨 df41cb7(capability 命名主閘)上線,Windows 的 Rust 測試步驟這次才第一次跑到它——先前同 job 都死在更早的 scripts 步驟。目標使用者:仰賴三平台 CI 綠燈守門的維護者;情境:跨平台測試正典(期望值與 CLI 輸出同底比對)。

## Problem 與 Root Cause

測試 helper `TempProject::new`(crates/speclink-cli/tests/it/new_artifact.rs 第 28 行)無條件對 temp 目錄 `canonicalize()`。Windows 的 canonicalize 會給路徑加 `\\?\` 前綴,期望字串變成 `\\?\C:\...`;CLI 實際輸出是普通的 `C:\...`,逐位元比對必炸。macOS/Linux 上 canonicalize 只是解 symlink(/var → /private/var),兩邊同底,所以只有 Windows 紅。

repo 對這個坑已有正典寫法:crates/speclink-cli/tests/it/trace.rs 第 42-43 行與 crates/speclink-cli/tests/it/discuss_promote_snapshot.rs 第 26-28 行都是「非 Windows 才 canonicalize」的同一句模式,new_artifact.rs 是漏套的那一處。

## What Changes

- crates/speclink-cli/tests/it/new_artifact.rs 的 `TempProject::new` 改用既有模式:`if cfg!(windows) { dir } else { dir.canonicalize().unwrap() }`,並沿用 trace.rs 的註解措辭說明原因。
- 只動測試 helper,不動任何產品碼與 CLI 輸出。

## Non-Goals

- 不改 CLI 的路徑輸出行為(產品碼輸出本來就對)。
- 不把三處重複的正規化模式抽成共用 helper——三行級重複、各檔自帶脈絡註解,抽象成本高於效益。
- 不處理其他測試檔——canonicalize 全量盤點顯示其餘各處已各自帶 cfg!(windows) 分支或僅在非比對用途使用。

## Success Criteria

- macOS 本機 `cargo test -p speclink-cli --test it new_artifact` 全綠(行為不變)。
- 推送後 Windows CI 的 build-and-smoke job 之 Test (Rust workspace) 步驟綠,該測試通過。

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

(none)——規格掃描:最近的是 capability-naming-guard(該測試所屬功能),其需求正確且不變;這是測試 helper 的跨平台修正,需求層級零變更,無 delta。

## Impact

- Affected specs: 無。
- Affected code:
  - Modified: crates/speclink-cli/tests/it/new_artifact.rs
  - New: 無
  - Removed: 無
- 影響的 crate:speclink-cli(僅整合測試檔)。
- 相容性影響:無 CLI 指令、人眼輸出或 `--json` 變更;golden 不受影響。

## Why

Windows CI 的 build-and-smoke job 掛在 `settings::tests::schemas_snapshot_carries_the_disk_path_for_file_backed_layers`(run 33136409656,speclink-desktop-core 226 過 1 敗)。這是 8/13 全綠基準後批次測試在 Windows 的第三顆同族雷:前兩顆(scripts 路徑推導、speclink-cli 的 canonicalize)已由前兩個 change 修復,本測試所在的 binary 這次才第一次在 Windows 跑到。掃描 agent 已對基準後全部 Rust 測試變更掃過一輪:除本處外無新雷,修完此處 Windows 管線應可走完。目標使用者:仰賴三平台 CI 綠燈守門的維護者;情境:跨平台測試正典(期望值與產品輸出同底比對)。

## Problem 與 Root Cause

產品碼 `schemas_snapshot`(apps/desktop/core/src/settings.rs 第 230、237 行)以原生 `join` 組出磁碟路徑再 `to_string_lossy`,Windows 上是全反斜線的正確路徑。測試期望值(第 1112、1118 行)卻把嵌著正斜線的整段字串丟給 `join`——`fx.root().join("openspec/schemas/my-flow")` 與 `user_dir.join("schemas/their-flow")`——Windows 的 `join` 保留正斜線,期望值變成 `...\Temp\...\openspec/schemas/my-flow` 的混合分隔符,`to_str` 逐位元比對必炸。macOS/Linux 上兩種組法同字串,所以只有 Windows 紅。

## What Changes

- apps/desktop/core/src/settings.rs 該測試的兩處期望值改逐段 join:`.join("openspec").join("schemas").join("my-flow")` 與 `.join("schemas").join("their-flow")`,期望值與產品輸出同底,兩平台皆為原生分隔符。
- 只動測試期望值的組法,不動產品碼與斷言語意。

## Non-Goals

- 不改 `schemas_snapshot` 產品碼的路徑輸出(原生路徑本來就對)。
- 不動同檔其餘測試——掃描已逐行確認其餘 join 僅供 fs 存取或存在性判斷,無字串比對。
- 不抽跨平台路徑比對 helper——兩處逐段 join 即收斂,抽象成本高於效益。

## Success Criteria

- macOS 本機 `cargo test -p speclink-desktop-core settings::` 全綠(行為不變)。
- 推送後 Windows CI 的 build-and-smoke job 全綠(Rust workspace 步驟走完後段全部 binary)。

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

(none)——規格掃描:最近的是 desktop-config/workflow-schemas(該測試守的功能面),需求正確且不變;純測試期望值組法修正,無 delta。

## Impact

- Affected specs: 無。
- Affected code:
  - Modified: apps/desktop/core/src/settings.rs
  - New: 無
  - Removed: 無
- 影響的 crate:speclink-desktop-core(僅 `#[cfg(test)]` 模組)。
- 相容性影響:無任何輸出或 `--json` 變更;golden 不受影響。

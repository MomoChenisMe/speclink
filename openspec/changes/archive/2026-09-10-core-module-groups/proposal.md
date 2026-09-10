## Summary

`crates/engine/speclink-core/src` 的 30 個平鋪模組依功能分成 lifecycle／quality／workspace 三組資料夾（command 維持、基礎件留根），`lib.rs` 以一組 re-export 讓 `speclink_core::<模組>` 全部既有路徑逐字不變，並把 inline 測試達 500 行的十個模組搬成同名子檔；零行為變化。

## Motivation

目標使用者是第一次打開引擎 crate 的人（新加入的開發者、透過 AI 代理讀碼的 SDD 使用者），使用情境是「找封存在哪、品質關卡在哪、工作區交付在哪」。來源討論 improve-repo-layout（Round 1、3–5）的查證：

- 30 個模組按字母序平鋪，同一概念交錯：變更生命週期是 model／newcmd／inprogress／discard／archive／tasks／status／listing 八檔，中間夾著 init／instructions／drift／demo；品質關卡是 station／validate／analyzer／drift，散在 a 到 v；工作區與指令交付是 init／skills／instructions／config／schema／workspace 六檔。相依圖證實三群是真的群：archive 引用的 13 個模組有 9 個在生命週期與品質兩群；init 只引用 config／skills／util；skills 只引用 config／init。
- 十個模組的 inline 測試超過 500 行（discuss.rs 2152、command/mod.rs 1902、station.rs 1739、init.rs 1655、archive.rs 1556、config.rs 1367、tasks.rs 1052、analyzer.rs 589、validate.rs 575、drift.rs 509）——打開檔案要先滾過一千多行才確認程式碼到哪裡為止。
- 這是 improve-repo-layout 三刀中的第二刀。前提已成立：刀一（repo-layout-groups）已封存，crate 位於 `crates/engine/speclink-core`；improve-lifecycle-layer 三刀已封存，檔案集合定型（review.rs／verify.rs 已刪、keylines.rs 已在）。

相關規格掃描：`delivery-baseline`（本案修改對象——刀一立的「倉庫路徑守門」要延伸到 core 模組路徑）；`dev-harness`、`workflow-config` 名字相近但不涵蓋 crate 內部模組佈局。

## Proposed Solution

影響的 crate：speclink-core（模組搬移、`lib.rs`、三個 `include_str!` 相對路徑）；crate 外只有引用 core 內部檔案路徑的七處（scripts/desktop 兩檔、packages/ui 三檔、core 自己的 render_golden 訊息、docs/product-status 兩份）。不新增或變更 CLI 指令、旗標、stdin、exit code；不涉及設定欄位。

1. **三組資料夾**：`lifecycle/`（model、newcmd、inprogress、discard、archive、tasks、status、listing、capname、preflight、discuss、trace）、`quality/`（station、validate、analyzer、drift）、`workspace/`（init、skills、instructions、config、schema、workspace）；`command/` 維持；根留 lib、store、util、keylines、testkit、teststore、demo。
2. **永久 re-export**：`lib.rs` 內三個資料夾模組私有（`mod lifecycle;` 等），各資料夾的 `mod.rs` 以 `pub mod` 列子模組，`lib.rs` 以 `pub use lifecycle::{archive, discuss, …};` 一組 re-export 把 22 個名字掛回根——crate 外約 500 處 `speclink_core::<模組>` 引用與 crate 內 544 處 `crate::<模組>` 引用一字不改。
3. **測試搬子檔**：十個模組的 `mod tests` 內容逐字搬到同名子目錄的 `tests.rs`（如 `lifecycle/archive.rs`＋`lifecycle/archive/tests.rs`、`command/mod.rs`＋`command/tests.rs`），模組檔尾端一行 `#[cfg(test)] mod tests;`；`use super::*` 的私有存取不變，`cargo test -p speclink-core <名字>` 過濾字串不變。
4. **相對路徑**：`workspace/skills.rs` 與 `workspace/schema.rs` 的 `include_str!("../assets/…")` 改為 `../../assets/…`；根的 `demo.rs` 不動。
5. **crate 外引用同批改**：scripts/desktop/desktop-install.mjs 與其測試讀 `crates/engine/speclink-core/src/workspace/init.rs`；packages/ui 的 discussionDrawer.test.tsx 讀 `lifecycle/discuss.rs`，tasks.ts 與 taskList.test.tsx 的註解指 `lifecycle/tasks.rs`；render_golden.rs 的 bump 提示訊息；docs/product-status 兩份的模組路徑。
6. **守門延伸**：delivery-baseline「倉庫路徑守門」增 (c) 條——已搬入子目錄的 core 模組不得再以 `crates/engine/speclink-core/src/<模組>.rs` 舊形引用（根層模組 lib／store／util／keylines／testkit／teststore／demo 與 command/ 除外）。

**相容性影響**：人眼輸出與 `--json` 不變；render golden 不變（asset 內容與版號未動，`ASSET_VERSION` 不 bump）；`speclink_core` 的公開 API 路徑不變，四個消費 crate 零改動；rustdoc 會同時顯示根層名字與資料夾路徑，屬預期。

## Non-Goals

- 不改任何模組的行為、簽名或錯誤字串；不合併或拆分模組本體。
- 不讓消費端改走 `speclink_core::lifecycle::archive` 這類新路徑；資料夾模組不設為 pub。
- 不搬測試未達 500 行的模組（keylines 381、model 360、schema 208、newcmd 218、inprogress 317、trace 311、util 258、discard 201、skills 173、listing 168、capname 132、instructions 93、workspace 60、typed 56、teststore 28）。
- 不動 `tests/golden`、`assets/`。`tests/it` 只改兩條架構守門的掃描面（`no_direct_fs` 改遞迴、兩條都跳過 `tests.rs` 子檔）與 render_golden 的提示字串，測試意圖零變化。
- 不動 server 模組（刀三）。

## Alternatives Considered

- 消費端遷移到新路徑——500 處零收益改動，且內部分組升格為六個 crate 的耦合面。
- 資料夾模組設為 pub、新舊路徑並存——讀者要問哪條是正典。
- `#[path]` 屬性保留平鋪名只搬檔案——社群公認的可讀性反樣式。
- 全部模組一律搬測試——十幾個一屏內看得完的小檔多出子檔，跳檔成本大於收益。
- 測試搬到 `tests/` 整合目錄——失去私有存取、要改寫上千行、增加 test binary。
- 五組以上（討論單獨成組）或依動詞一資料夾——一檔一組反樣式；動詞切法為 cli-verb-family-modules 已否決。

## Impact

- Affected specs：delivery-baseline（MODIFIED「倉庫路徑守門」：增 (c) core 模組舊形路徑條款與一條 scenario，既有五條 scenario 逐字保留）。
- Affected code:
  - New: crates/engine/speclink-core/src/lifecycle/mod.rs、crates/engine/speclink-core/src/quality/mod.rs、crates/engine/speclink-core/src/workspace/mod.rs、crates/engine/speclink-core/src/lifecycle/discuss/tests.rs、crates/engine/speclink-core/src/lifecycle/archive/tests.rs、crates/engine/speclink-core/src/lifecycle/tasks/tests.rs、crates/engine/speclink-core/src/quality/station/tests.rs、crates/engine/speclink-core/src/quality/analyzer/tests.rs、crates/engine/speclink-core/src/quality/validate/tests.rs、crates/engine/speclink-core/src/quality/drift/tests.rs、crates/engine/speclink-core/src/workspace/init/tests.rs、crates/engine/speclink-core/src/workspace/config/tests.rs、crates/engine/speclink-core/src/command/tests.rs
  - Modified: crates/engine/speclink-core/src/lib.rs、22 個搬移的模組檔（git mv；其中十個尾端加 `#[cfg(test)] mod tests;`）、crates/engine/speclink-core/src/workspace/skills.rs 與 crates/engine/speclink-core/src/workspace/schema.rs 的 include_str! 路徑、crates/engine/speclink-core/tests/it/render_golden.rs、crates/engine/speclink-core/tests/it/no_direct_fs.rs、crates/engine/speclink-core/tests/it/no_process_env.rs、scripts/desktop/desktop-install.mjs、scripts/desktop/desktop-install.test.mjs、scripts/release/delivery-gate.test.mjs、packages/ui/src/tasks.ts、packages/ui/src/__tests__/discussionDrawer.test.tsx、packages/ui/src/__tests__/taskList.test.tsx、docs/product-status.md、docs/product-status.zh-TW.md
  - Removed: （無；搬移不刪檔）

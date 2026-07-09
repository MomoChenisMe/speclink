## Problem

discuss 的 context／add-round／conclude 在漏帶 --stdin 時把內容當空字串靜默寫入、卻仍回報成功，導致空的 Round／Conclusion。本 session 已實際發生兩份記錄的 Round 與 Conclusion 全空，直到人工檢視才發現。

## Root Cause

- CLI（本地 commands 與遠端 remote_commands）以「帶 --stdin 才讀取 stdin、否則給空字串」的分支處理內容，故漏帶旗標即得空字串。
- core::discuss 的 add_round／conclude／set_context 對傳入 content 零檢查，直接寫入區段。
- 二者疊加成典型 silent failure：壞輸入不報錯、留下損毀記錄。

## Proposed Solution

- 於引擎 core::discuss 的 add_round／conclude／set_context 加空內容 guard：內容去除前後空白為空即以錯誤中止（bail），訊息提示內容為空並提醒可能漏帶 --stdin。guard 置於 core，一次覆蓋所有前門（本地 CLI、遠端 CLI、桌面 Tauri）。
- CLI 於標準輸入為管線（非互動終端）時讀取其內容作為動詞內容，不論是否帶 --stdin；--stdin 維持被接受以相容既有腳本。互動終端且無管線時內容為空，交由 core guard 明確報錯。

## Success Criteria

- 對 add-round／conclude／context 提供空內容（含漏帶 --stdin 且無管線輸入）以錯誤中止、不寫入空區段、且 conclude 不將 status 翻為 concluded。
- 以管線提供非空內容但未帶 --stdin 時，內容仍被正確寫入（不因漏旗標而靜默成空）。
- 三前門（本地 CLI、遠端 CLI、桌面）皆受同一 core guard 保護。

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `discussion-docs`：新增「討論內容寫入動詞拒絕空內容」需求（空內容 guard 置於 core；CLI 管線輸入不論旗標皆讀取；add-round 維持純附加）。

## Impact

- Affected specs: discussion-docs（modified）
- Affected code:
  - Modified:
    - crates/speclink-core/src/discuss.rs
    - crates/speclink-cli/src/commands.rs
    - crates/speclink-cli/src/remote_commands.rs
  - New: (none)
  - Removed: (none)

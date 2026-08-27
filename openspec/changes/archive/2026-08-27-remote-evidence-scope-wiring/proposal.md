## Why

remote 模式的 task done 已把 touched files、head commit、task 與 actor 落進 server store（remote-task-evidence，2026-08-25 封存），server 也有 change evidence 讀回端點；但 review 與 verify 解析 scope 時，remote 分支仍以本地 FsStore 讀 evidence 記錄——remote 模式下本地不存在 evidence 檔，touched 恆為空，站台 fail-closed 成 needsInput，只能以 --candidate-hash 加 --include-hunk 手動續行。apply 到品質站之間的證據鏈在 remote 斷了最後一哩，且無任何既有立案覆蓋此縫。（源討論：remote-fix-plan-gaps，刀 2）

## What Changes

- review scope 與 verify scope 的 remote 分支改自 typed remote client 的 change evidence 端點讀取 touched 認領——端點既存、現無生產呼叫者；修法照 remote drift 把 store 端 evidence 送進 checkout 端計算的既有前例。
- 併行認領守門（other claims）於 remote 同樣生效：其他 active change 的 evidence 逐一自 server 讀取，與 fs 模式同語意。
- 多 actor evidence 的 touched 取聯集，與 fs 模式 TouchedRecord 的 all_files 語意一致；evidence 內的 head commit 不參與 scope 解析，維持僅存證。
- evidence 缺席或空時維持 EmptyTouched fail-closed 與 needsInput 手動路徑，跳脫閥不變。
- 正典 change-diff-scope 的「remote workspace 使用同一 host resolver」requirement 修訂：touched 來源自「local checkout 的 evidence 記錄」改為「Store 的 evidence 記錄」（fs＝本地檔、remote＝server 端點）；baseline 與 snapshots 維持 local checkout，server 仍不新增 Git diff 端點。
- 測試：remote scope 測試不再手塞本地 touched 檔，改由 mock server 供應 evidence；補「task done 後 scope 自動解析」鏈測試與「evidence 缺席 needsInput」釘住測試。

## Non-Goals

- 不動 crates/speclink-core/assets/skills 的 review 與 verify 技能敘述——手動逃生路徑（--base、--candidate-hash、--include-hunk）仍是正當跳脫閥，敘述未失真；避免 MARKER_VERSION 與 golden 與 assets.lock 三連動波及。
- 不動 server 與 protocol——evidence 端點與 wire 形狀已存在，本刀只補消費端。
- 不做 desktop 遠端看板勾任務的本機 git 探測（touched 收集）——remote-task-evidence 既有 Non-Goal，維持。
- 不做 evidence 回填工具——已丟失的歷史補不回，既有紅線。

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `change-diff-scope`: 「remote workspace 使用同一 host resolver」requirement 的 touched 來源改為 Store evidence 記錄；新增多 actor 聯集、併行認領守門於 remote 生效、evidence 缺席 fail-closed 的場景。

## Impact

- Affected specs: change-diff-scope
- Affected code:
  - Modified: crates/speclink-cli/src/verbs/station.rs, crates/speclink-cli/tests/it/remote_verb_parity.rs
  - New: (none)
  - Removed: (none)

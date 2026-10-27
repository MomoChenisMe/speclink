## Why

remote_ctx()（crates/speclink-cli/src/remote_base.rs）必須先探索到 Workspace 才能解析 store 模式，卻只回傳 RemoteCtx { client } 把 workspace 丟掉。於是 6 個 remote 臂各自重取，養出 4 種「找不到 workspace 怎麼辦」的缺席寫法——而 remote 模式成立的前提就是 workspace 存在（探索不到即回 fs 模式），多數缺席分支守的是不可能發生的狀態。這是 improve-cli-verb-layer 討論（2026-09-01 結論）裁定的第二刀，排在 cli-typed-engine-entry 之後。

目標使用者是本專案的維護者與經由 AI 代理跑 SDD 流程的開發者；使用情境是日後每次新增 remote 動詞時，workspace 直接從 RemoteCtx 取用，不再重探索、不再發明第五種缺席策略。

## What Changes

- speclink-cli 的 remote_base.rs：RemoteCtx 加 ws 欄位（owned Workspace，remote_ctx() 探索到手的那份帶出）；「remote 模式 ⇒ workspace 存在」的不變式收進 remote_base 一份。
- 6 處 remote 臂的 workspace 重取刪除，改讀 ctx.ws——station 的 review prepare 與 station scope（原 require_workspace）、station stamp（原 discover 加自訂 bail）、drift 的 remote 臂（原 discover 加 git_available 過濾，過濾保留、只刪探索）、instructions 的 context 投影（原靜默略過分支，分支刪除）、task done 的 touched 蒐集（原 best-effort 探索，改直接取用）。
- 動詞級語意全數保留：git_available 過濾（有 workspace 不代表有 git）與 task done 的空集合語意不變；刪除的只有「workspace 缺席」這個不可能狀態的處理。
- 順帶同刀：new artifact、task done、task undone、artifact cat 四處 remote 臂以外層 match 重寫了 remote_resolve_change 已實作的 Some 直通，改為 validate 臂既有的 let-else 一行形。
- station.rs 因遷移而孤兒化的 require_workspace import 清除。
- CLI 指令面零變更：無新子指令、無旗標變更、stdin 與 exit code 照舊。
- 相容性影響：無——人眼輸出與 --json 逐位元不變；被刪的缺席分支在現行結構下不可觸發，凍結輸出與 remote parity 整合測試全數照跑。

## Non-Goals

- 不動 wire→core 轉接的落點——improve-wire-convert-seam（2026-08-10）落點規則照舊：出現第二個要同一 core 型別的消費端才進 speclink-remote::convert。
- 不動 dispatch 模式表與 remote_ctx() 的惰性執行時機（cli-mode-dispatch-convergence 定案）。
- 不動 remote 臂的明文拒絕與 C 類明文分歧（remote-verb-parity 定案）。
- 不涉及 server、protocol、desktop 與設定欄位。

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

(none)——規格掃描比對：最近的既有規格為 remote-connection（連線解析與握手語意，不動）與 change-diff-scope（station scope 於本地 checkout 解析，不動）。本刀是 CLI 內部管線重構，不改任何 SHALL，不出 delta。

## Impact

- Affected specs: none
- Affected code:
  - New: none
  - Modified: crates/speclink-cli/src/remote_base.rs、crates/speclink-cli/src/verbs/station.rs、crates/speclink-cli/src/verbs/checks.rs、crates/speclink-cli/src/verbs/instructions.rs、crates/speclink-cli/src/verbs/progress.rs、crates/speclink-cli/src/verbs/new.rs、crates/speclink-cli/src/verbs/documents.rs
  - Removed: none

## Context

remote_ctx()（crates/speclink-cli/src/remote_base.rs）的流程是：探索 Workspace → 解析 store 模式 → 握手 → 回傳 RemoteCtx { client }。workspace 在第一步到手、第四步丟棄。6 個 remote 臂因此各自重取：station 的 review prepare 與 scope 走 require_workspace、station 的 stamp 走 discover 加自訂 bail、drift 走 discover 加 git_available 過濾、instructions 的 context 投影走靜默略過、task done 的 touched 蒐集走 best-effort。其中除 git_available 外，全部缺席處理守的是不可能狀態——remote 模式成立即代表 workspace 存在。來源討論 improve-cli-verb-layer（2026-09-01）已裁定形狀；本刀排在 cli-typed-engine-entry 之後。

## Goals / Non-Goals

**Goals:**

- 「remote 模式 ⇒ workspace 存在」的不變式只活在 remote_base.rs 一份
- 6 處重取歸零；不可能狀態的缺席分支刪除；動詞級語意（git 可用性、空集合）原樣保留
- 4 處 remote_resolve_change 呼叫儀式收斂為 let-else 一行形
- 人眼輸出與 --json 逐位元不變

**Non-Goals:**

- 不動 wire→core 轉接落點（improve-wire-convert-seam 落點規則照舊）
- 不動 dispatch 模式表與 remote_ctx() 的惰性執行時機
- 不動 remote 臂的明文拒絕與明文分歧
- 不涉及 server、protocol、desktop

## Decisions

**D1：ws 以 owned Workspace 欄位落在 RemoteCtx，remote_ctx() 把探索到的那份移入。** 零額外探索、零 clone（remote_ctx() 的區域變數直接移動）。捨棄案：lazy getter 或另開輔助函式（多一層 wrapper，臂還是要會一套取用姿勢）；借用形（RemoteCtx 持 &Workspace 需生命週期參數，dispatch 組合子全面沾染，成本遠大於一個 owned struct）。

**D2：缺席分支逐點處置，語意差異只刪「不可能」的那部分。** station 的 review prepare 與 scope：require_workspace 呼叫改為讀 ctx.ws；station 的 stamp：discover 加自訂 bail 整段改為讀 ctx.ws，bail 訊息隨之刪除（該狀態不可觸發，無輸出相容性問題）；drift：探索改為 Some(&ctx.ws)，git_available 過濾保留——有 workspace 不代表有 git，這是真語意；instructions 的 context 投影：靜默略過分支刪除，投影一律嘗試；task done：touched 與 head 直接從 ctx.ws 計算，git 取不到東西時的空集合語意不變。

**D3：remote_resolve_change 的呼叫儀式統一為 let-else 一行形。** new artifact、task done、task undone、artifact cat 四處 remote 臂目前以外層 match 先攔 Some 再於 None 臂呼叫 remote_resolve_change——重寫了被呼叫端已實作的 Some 直通。統一改為 validate 臂既有的形狀：let Some(name) = remote_resolve_change(ctx, 引數的 change 選項, 提示字串)? else 提前回傳。提示字串逐字保留各臂現值，輸出不變。

## Implementation Contract

- **Behavior**：使用者可觀察行為零變更。全部 remote 動詞的人眼輸出、--json、stderr 與 exit code 逐位元一致；被刪的缺席分支在現行 dispatch 結構下不可觸發。
- **Interface / data shape**：RemoteCtx 新增 pub(crate) ws: Workspace 欄位；remote_ctx() 簽名不變；remote_resolve_change 簽名不變。protocol 與 server 面零變更。
- **Verification**：
  - cargo test -p speclink-cli --test it 全綠（含 remote_verb_parity 凍結對照——兩模式輸出位元級一致即驗收）
  - 遷移完成斷言：grep 檢查 crates/speclink-cli/src/verbs/ 內 discover_cwd 只剩 toolchain.rs 與 connection.rs（ModeFree 動詞的合法使用）；station.rs 的 require_workspace import 若孤兒化則清除；cargo build -p speclink-cli 零 dead-code 與零 unused-import warning

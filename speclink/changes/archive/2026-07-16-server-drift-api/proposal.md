## Why

drift 是 remote 動詞覆蓋面的最後一個洞：drift-client-server-split 刀完成了引擎端分解——compute_spec_drift（純規格面）、collect_workspace_facts＋compute_workspace_drift（本機 code/git 面）、merge_drift_reports（單一合併器）——但 wire 從未接線：protocol 無 drift DTO、typed client 無方法、CLI remote 攔截層無此動詞，remote 模式跑 drift 至今走不通。架構 §6.5 的遠端 drift 序列（server 算規格面、client 算工作區面、單點合併）就是為此刻設計的；§14 Phase 2 第 5 項的全鏈 e2e 明列 drift 環節，phase2-e2e-chain 刀以本刀為前置。

目標使用者：remote 模式帶本機 checkout 的 RD/Agent（完整 drift 報告）與無 checkout 的 PM（規格面 drift）；phase2-e2e-chain 刀（drift 環節有真端點可走）。

## What Changes

- protocol 新增 drift 模組 DTO：spec drift 報告（規格面維度、規格假設）、drift basis（spec/tasks/policy digests——server 對當下 snapshot 算出的固定基準），以及該 change 的 store 面輸入（created 與 design/tasks 內容，缺席與空可區別）wire 形狀，camelCase、JSON Schema 匯出、序列化往返測試。broken anchors 屬工作區面（引擎的 `WorkspaceDriftReport`，由 git 事實算出），不進 wire。
- host 新增 drift 的專用查詢入口與 wire↔引擎型別的單點雙向映射：入口內部 materialize 私有橋接視圖、跑 compute_spec_drift、由同一 snapshot 算 basis digests 與讀 store 面輸入一起回傳——橋接的 `Store` 視圖不外洩給 adapter；另加唯讀最小 `Store` adapter，讓 client 以 server 供給的 store 面輸入餵引擎的工作區面計算。
- server 新增 change-scoped 的 spec drift 端點：呼叫 host 入口取規格面報告與 basis；沿用 bearer 前置與 binding 裁決；未知 change 回 404。
- typed client 新增 drift 方法；CLI remote 攔截層接上 drift 動詞：向 server 取規格面報告、在本機（有 workspace 時）collect_workspace_facts 並 compute_workspace_drift、經 merge_drift_reports 合併——人眼與 --json 輸出形狀以 fs 模式為權威逐位元對齊；無本機 checkout 時工作區面依三值語意如實標示不可得（unavailable 不等於 clean），不偽造乾淨。
- drift 維持診斷性質：本刀不引入任何以 drift 結果擋動詞的 gate（與既有 drift-computation 能力語意一致）。

## Capabilities

### New Capabilities

- `server-drift-api`: server 的規格面 drift 端點與 basis、typed client 方法、remote drift 動詞的合併與輸出凍結、無 checkout 時的三值誠實標示。

### Modified Capabilities

(none)

## Impact

- 相容性影響：wire 純新增；fs 模式 drift 輸出零變更（凍結對照基準）；remote 模式從「動詞不可用」變「可用」，無既有行為被改變。parity 31 項、color 16 項、twin 8 情境凍結不動（twin 如需覆蓋 drift 屬 e2e 刀的劇本，不動既有 stub 情境）。
- Affected specs: `server-drift-api`（新增）
- Affected code:
  - New: crates/speclink-protocol/src/drift.rs、crates/speclink-server/tests/drift_api.rs
  - Modified: crates/speclink-protocol/src/lib.rs、crates/speclink-host/src/drift.rs（wire↔引擎映射與 spec_drift 查詢入口）、crates/speclink-host/src/bridge.rs（供 host 內部取橋接唯讀視圖，不對外公開）、crates/speclink-server/src/app.rs、crates/speclink-server/src/routes.rs、crates/speclink-remote/src/client.rs、crates/speclink-cli/src/remote_commands.rs
  - Removed: 無

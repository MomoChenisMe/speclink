## Why

Phase 2 各刀都有自己的 e2e，但架構 §14 Phase 2 第 5 項要的是另一種東西：「以 CLI/Client SDK 完成端到端 propose、task、policy、context、evidence、drift、archive 與 event recovery 測試」——單一連續劇本，驗的是環節**之間**的縫：setup 建的帳號能不能走完 propose、propose 的 change 能不能被 task done 的 evidence 指到、policy 改了 instructions 會不會跟著變、archive 之後投影與事件是否如實反映。分刀 e2e 各自播種各自斷言，縫隙從未被連續走過。這是 Phase 2 收官的最終驗收面，也是日後任何 server 改動的整鏈回歸保護。

目標使用者：Phase 3 起的每一把刀（desktop/N-API 動 server 消費面時有整鏈紅綠燈）與發版前的驗收者。

## What Changes

- 新增全鏈劇本測試（真實 CLI binary 對真 server、SQLite driver、tempdir 隔離）：**單一測試劇本依序**——(1) 全新資料庫啟動、stdout 取 setup token、HTTP 走完 /setup（Admin＋Project/Repo）；(2) invite 子命令→接受頁設密碼→登入→建 PAT；(3) CLI 以 PAT 連線：new change→寫 proposal/design/specs/tasks artifacts；(4) 設定 workflow config（policy）後斷言 instructions 輸出反映政策變化；(5) task done 帶 touched files，斷言 outbox 有 task-completed 事件、evidence 記錄可查；(6) apply 階段動詞後斷言投影完整（正典 specs、delta、manifest verify 通過）；(7) 執行 remote drift（有 checkout）得完整報告；(8) archive 後斷言正典 specs 更新、change 入 archive、清單如實。
- event recovery 演練併入同劇本：步驟 (3)–(8) 期間以 SSE 訂閱者伴隨——中途強制斷線並漏掉數筆事件，以 Polling＋ETag 收斂後重新訂閱，最終斷言「訂閱者視角的最終狀態」與「直接查詢」一致（§9.2 收斂規則的整鏈實證）。
- 劇本以步驟編號組織、任一步失敗即報出步驟名與現場（server logs、workspace 狀態），失敗可讀性是驗收條件之一。
- 劇本進 CI 必跑路徑。

## Capabilities

### New Capabilities

- `phase2-acceptance`: 全鏈劇本的行為保證——八環節連續可走、event recovery 收斂、失敗現場可讀、CI 必跑。

### Modified Capabilities

(none)

## Impact

- 相容性影響：純新增測試與其 helpers；不動任何產品程式碼；劇本若揭露環節縫隙的 bug，修復屬獨立的 bug-fix change（本刀不順手修，維持外科紀律）。前置依賴：server-drift-api 刀（步驟 7）；與 server-release-packaging 無交集可平行。
- Affected specs: `phase2-acceptance`（新增）
- Affected code:
  - New: crates/speclink-server/tests/phase2_chain.rs
  - Modified: crates/speclink-server/tests/common（劇本共用的播種/訂閱 helpers 擴充）、.github/workflows/ci.yml（若劇本測試需獨立 job 標註）
  - Removed: 無

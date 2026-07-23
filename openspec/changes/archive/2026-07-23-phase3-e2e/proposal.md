## Why

Phase 3 各刀都有自己的測試，但架構 §14 Phase 3 第 5 項要的是縫隙驗收：「以 PM 無 checkout、RD 有 checkout、多 server、多 tab 與失聯恢復情境做端到端測試」——單一連續劇本驗環節之間的縫：登入建的 session 能不能走完 PM 讀寫、RD 的 checkout 與 CLI 互通後 desktop 面能不能秒級反映、兩個 server 的憑證與事件是否真隔離、同 locator 的 sessions 是否共用單條 SSE 且不同 scope 的流是否隔離、殺 server 撤 family 之後是否真的原地復活。這是 Phase 3 關門的最終驗收面，也是 Phase 4（agent adapters 動 server/desktop 消費面）的整鏈回歸保護。

## What Changes

- 新增 Phase 3 全鏈劇本測試（desktop 資料面＝src-tauri 的 remote runtime／event manager／session 層對真 server；RD 情境以真實 CLI binary 於 checkout 資料夾實走；GUI 面由手動鐵律清單補完）：**單一劇本依序**——(1) 起兩個 server（sqlite、tempdir 隔離）、各自 setup 出帳號與 scope；(2) PM 無 checkout：登入（in-memory credential store）→ handshake 開 session → 清單/文件讀取、任務勾選、policy 讀取、role 對應的 capability 停用斷言；(3) RD 有 checkout：tempdir git repo 寫 marker → CLI 於該資料夾以 remote 模式 new change＋artifacts＋task done → desktop 面經 SSE invalidate 數秒內反映；(4) 多 server：第二連線並開 session，憑證逐 origin 隔離、事件互不串流；(5) 多 tab：同 server 的 PM／RD 兩 scope 各維持一條 scope SSE，同 locator 的多個 sessions 以參照計數共用該流，失效提示只按對應 locator 分發；(6) 失聯恢復：殺 server → offline 廣播與寫入即拒 → 期間 CLI（另一 server 路徑不受影響）→ 重啟自動收斂含期間變更 → 撤 device family → needs-reauth → 重登入原地恢復。
- 任一步失敗報出情境名與現場（server 輸出、connection 狀態），失敗可讀性是驗收條件。
- 劇本進 CI 必跑路徑。
- GUI 鐵律手動清單：五情境的 UI 面對照走查（真實視窗、remote-dev-harness）。

## Capabilities

### New Capabilities

- `phase3-acceptance`: Phase 3 收官驗收的行為保證——五情境連續可走、憑證與事件隔離、同 locator 單流共用與跨 scope 隔離、失聯復活、失敗現場可讀、CI 必跑。

### Modified Capabilities

(none)

## Impact

- 相容性影響：純新增測試與 helpers；不動任何產品程式碼；劇本若揭露縫隙 bug，修復屬獨立 bug-fix change（本刀不順手修）。與 local-remote-migration 可平行——僅 apps/desktop/src-tauri/tests/common 有共用 helpers 檔的合流注意。
- Affected specs: `phase3-acceptance`（新增）
- Affected code:
  - New: apps/desktop/src-tauri/tests/phase3_chain.rs
  - Modified: apps/desktop/src-tauri/tests/common/mod.rs（雙 server、CLI 驅動、家族撤銷等 helpers 擴充）、.github/workflows/ci.yml（劇本測試 job 標註）
  - Removed: 無

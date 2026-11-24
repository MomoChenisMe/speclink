## Problem

使用者是用 Speclink Desktop 同時開著多個專案的開發者，情境是 workspace-chooser 的開啟流程與圖示列切換專案。

在 Desktop 開啟一個同時有本機規格資料夾（`speclink/`，或舊佈局 `openspec/`）與 `.speclink.yaml` remote 區段的資料夾時，Desktop 會停下來顯示三出口對話框。選「繼續本機」後，本機分頁正常開啟。但之後有三條路徑會把這個分頁判成失效，方塊轉成錯誤態（角落紅點），提示「已不是 speclink 專案」，點下去只剩「自專案列移除」：

- 切到別的專案，再切回這個分頁。
- 關掉 app 再開，而上次最後停在這個分頁。
- 關掉 app 再開，而這個分頁在背景。

（範圍調整，品質關卡第一輪後）原本這裡還列了一條：「在這個分頁的專案設定按『遷移到 Server…』會拒絕遷移」。查證後 Desktop 的專案設定沒有這個入口：唯一的「遷移到 Server…」按鈕在新增專案選擇器，而且只在探測結果是純本機專案時出現，並存資料夾走不到它。這一條因此移出範圍，`requestMigration` 維持原狀；並存資料夾要遷移時，走三出口對話框的「遷移本機內容」。

## Root Cause

- `apps/desktop/core/src/project.rs` 的 `open_project_at` 只要 `.speclink.yaml` 有 remote 區段就回 `RemoteBinding`；本機規格資料夾也在時，只多帶 `has_local_openspec: true`。這個判定與 CLI 的模式解析一致（remote-connection：並存時 remote 生效並警告），本身沒有錯。
- 「繼續本機」（`apps/desktop/src/store.ts` 的 `continueLocalFromConflict`）第一次開啟時直接呼叫 `enterProject`，不經探測，所以第一次正常。
- 切回時，`activateTab` 對本機分頁重新探測，只接受 `project` 一種結果，其他結果一律寫入錯誤訊息 `store.tabInvalid`。app 啟動時，上次作用中的分頁走同一個 `activateTab`。
- app 啟動時，背景本機分頁呼叫 `project_stats_at`。它同樣只接受「`Project` 且專案根等於路徑」，其他結果一律回錯誤，前端把錯誤寫進分頁錯誤。
- 探測結果不帶專案根：分頁路徑是並存專案的子目錄，或巢狀專案的規格資料夾已刪而上層是並存資料夾時，只看探測結果會開出上層專案的內容（品質關卡第一輪找到）。

## Proposed Solution

把「繼續本機」的選擇延續到這個本機分頁之後的每一次開啟：本機分頁的探測結果是「本機規格資料夾與 remote 區段並存」時，照樣當成本機專案。

- `activateTab`：本機分頁的探測結果為 `remoteBinding` 且 `hasLocalOpenspec` 為 true 時，以分頁本身的路徑與名稱呼叫 `enterProject`，不轉錯誤態，也不再跳三出口對話框。進入前先以 `projectStats` 確認分頁路徑就是專案根（與背景啟動檢查同一道比對），不是根時照舊轉錯誤態。
- `project_stats_at`：探索命中的專案根等於路徑，且本機規格資料夾與 remote 區段並存時，照常回報待收尾數。
- `requestMigration` 不改：維持只接受純本機專案（理由見 Problem 的範圍調整）。
- 其他探測結果的處理不變：只有 remote 區段、沒有本機規格資料夾（`hasLocalOpenspec` 為 false）、`uninitialized`、`unadopted`、路徑不存在，照舊轉錯誤態。
- 經「新增專案」或最近開啟清單再次開啟同一個資料夾時，照舊顯示三出口對話框：明確的開啟動作仍要使用者選。

## Success Criteria

- 資料夾 `/work/coexists` 同時有 `speclink/` 與 remote 區段。選「繼續本機」後切到別的專案再切回，作用中專案是 `local:/work/coexists`，分頁沒有錯誤，也沒有出現三出口對話框。
- 同一個分頁是上次作用中的分頁時，`restoreTabs` 以本機開啟它，分頁沒有錯誤。
- 同一個分頁在背景時，`restoreTabs` 之後它沒有錯誤；`project_stats_at` 對這個資料夾回報待收尾數。
- 分頁路徑是並存專案的子目錄（Rust 端的根比對不通過）時，切回轉錯誤態，提示「已不是 speclink 專案」，不開出上層專案的內容。
- 對並存資料夾呼叫 `requestMigration`，照舊拒絕（行為不變）。
- 資料夾只有 remote 區段、沒有本機規格資料夾時，本機分頁切回仍轉錯誤態（行為不變）。
- 對同一個資料夾執行 `openProjectAt`，仍停在三出口對話框（行為不變）。

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `workspace-chooser`：「remote marker 資料夾的探測分流」的「繼續本地」從「本次以本地開啟」改為「這個本機分頁之後的切回、app 重開與背景啟動檢查，都以本地為準（限分頁路徑就是並存專案根的分頁）」。相關的 desktop-config「專案分頁列存於 app 本機」只說「路徑失效即轉錯誤態」，沒有定義失效，與這次的行為不衝突，不改。

## Impact

- Affected specs: `workspace-chooser`（MODIFIED「remote marker 資料夾的探測分流」）
- Affected code（只影響 apps/desktop，引擎、CLI、Server、Node SDK 不動）:
  - Modified: `apps/desktop/src/store.ts`（`activateTab` 的本機分支）
  - Modified: `apps/desktop/core/src/project.rs`（`project_stats_at`）
  - Modified: `apps/desktop/src/__tests__/remoteOpen.test.ts`（補切回、根比對、再次開啟與 `requestMigration` 照舊拒絕的測試）
  - Modified: `apps/desktop/src/__tests__/workspace.test.ts`（補 app 重開的測試）
- 相容性影響：沒有 CLI 指令、`--json` 輸出、設定欄位或技能的變動。Tauri command `project_stats` 的回應形狀不變，只是並存資料夾從回錯誤改為回待收尾數。
- 三種形態：只有 Desktop 的本機分頁會碰到；連到遠端的 checkout 與內嵌宿主沒有本機分頁，不適用。
- 來源討論：`rail-remote-status-and-continue-local` 的第一刀；第二刀（圖示列不用點就標出連不上的 Server）另立變更。

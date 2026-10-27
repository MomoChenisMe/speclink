## Why

桌面 app 只在啟動時背景檢查一次更新（App.tsx 掛載時呼叫 checkForUpdates(false)），之後沒有任何重檢；使用者把 app 開著幾天不關（開發者的常態），期間發布的新版只能靠設定頁手動「檢查更新」才會發現，等於自動更新只對「每天重開 app」的人生效。討論 update-check-on-focus-and-cli 裁定：以「視窗回到前景」為重檢時機，重用既有自動模式的狀態機，成本是一個監聽加一個時間戳；同一討論也裁定「只裝 CLI 的使用者不加更新檢查機制」——npm／brew 通路自帶升級流程、CLI 主要由 agent 以 --json 呼叫、舊 CLI 的真正傷害已有守門——該項不立變更，此處只記錄。

目標使用者：以 AI 代理跑 SDD 且長時間開著桌面 app 的開發者。使用情境：任何 workflow 階段之間，使用者從別的視窗切回 Speclink app 時。

## What Changes

- 桌面 app（apps/desktop）新增「視窗回到前景時重檢更新」：主視窗（label `main`）取得焦點，且距上次檢查（自動或手動皆算）超過 1 小時，才呼叫既有的自動檢查 checkForUpdates(false)。自動模式的既有規則全數沿用：檢查失敗靜默、已是最新靜默、下載中與待重啟不重檢。
- 節流的時間戳只放 store 記憶體（closure 變數，與 pendingUpdate 同層），不寫檔；app 重啟本就檢查一次。
- 更新 adapter 介面多一個「訂閱主視窗焦點變化」的方法，Tauri 實作委派 `getCurrentWindow().onFocusChanged`；測試以假 adapter 注入。非 Tauri 環境（未注入 updater）一律 no-op，與現有 checkForUpdates 一致。
- 正典 desktop-app「桌面自動更新」需求補述「回到前景重檢」與節流規則，新增兩個 scenario。
- 不涉及 CLI 指令、`--json` 輸出、設定欄位、生成的技能：無相容性影響。

## Capabilities

### New Capabilities

（無）

### Modified Capabilities

- `desktop-app`: 「桌面自動更新」需求由「啟動後在背景檢查」擴充為「啟動後與主視窗回到前景時（距上次檢查逾 1 小時）在背景檢查」，其餘規則（徵求同意、簽章驗證、失敗靜默、手動入口）不變。

相關規格掃描：desktop-app（本次修改對象）；desktop-release（updater 簽章與 latest.json 端點，行為不變、不動）；cli-distribution（只寫安裝，未寫更新檢查；討論裁定 CLI 不加檢查，不動）。

## Impact

- Affected specs: `desktop-app`（MODIFIED：桌面自動更新）
- Affected code:
  - Modified: `apps/desktop/src/core/updater.ts`（節流判定純函式與 1 小時常數）、`apps/desktop/src/adapter/updater.ts`（UpdaterAdapter 介面新增焦點訂閱、Tauri 實作）、`apps/desktop/src/store.ts`（上次檢查時間戳、回到前景重檢動作）、`apps/desktop/src/App.tsx`（掛載時訂閱焦點事件、卸載時取消）、`apps/desktop/src/__tests__/updater.test.ts`（節流判定與 store 接線測試）
  - New: （無）
  - Removed: （無）
- 影響的 app：apps/desktop（前端 React 層與 adapter 層）。不動 src-tauri 殼、不動 speclink-core／speclink-cli。
- macOS 系統匣面板是獨立的 `tray-panel` 視窗（nonactivating NSPanel、不奪焦點），開關面板不觸發主視窗焦點事件；面板動作「詳情／設定／登入」會聚焦主視窗，因此會經同一節流走重檢。

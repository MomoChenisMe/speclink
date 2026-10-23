## Why

- 透過 AI 代理跑 SDD 的開發者、PO 與 PM 在封存後仍需回看審查與驗證的問題、修正過程及接受理由；目前資料有保存，Desktop 已封存頁的品質章卻不能展開歷史。
- 目前共用歷史索引會驗證全部文件，手動刪除或修改任一份紀錄可能讓同一提案其他完整紀錄也無法查閱；查閱應逐筆明示異常，寫入與驗收仍維持嚴格檢查。

## What Changes

- 已封存詳情的「已審查／已驗證」分別展開對應歷史，依開始時間由新到舊排列，選取後在閱讀區唯讀顯示全文；沿用活動提案的操作與共用元件，支援鍵盤與切換項目時的資料隔離。
- 預設完整保留歷次審查、驗證與各次完整輪次，沿用三類封存保留設定與既有索引，不增加只保留最後一次的模式，也不重寫舊封存資料。
- 查閱時保留全部索引項目並逐筆標示文件缺失、內容完整性錯誤或讀取錯誤；其他完整紀錄繼續可讀，同一提案其他類別的異常不連帶封鎖審查／驗證。
- 區分未保留完整紀錄、無紀錄、單筆異常與整體來源失敗；來源失敗提供重試，不編造空清單，不自動修復索引或接受內容異動。
- 維持文件區只列進行中工單，歷史僅從品質關卡展開；補正正式規格中排除已封存詳情歷史的文字。查閱不重跑、不蓋章、不改原章。
- 相容性影響：review history／verify history 子指令、旗標、stdin、合法紀錄全文及健康清單輸出不變；部分缺檔時清單刻意改為成功列出含單筆錯誤標記的紀錄，JSON 只在異常項目新增選填 readError，人眼同列明示錯誤。壞索引、整體來源失敗及查詢缺損單筆仍失敗；本機與遠端共用判定。Node SDK 沿用 Host 查詢而同步取得新增選填欄位，不新增 SDK 動詞。
- 設定相容性：check_records.archive.review／verify／security 仍預設 true，false 仍依既有政策標示 omitted。安全政策、技能生成與各 AI 工具目錄均不變，寫入、驗收與封存的完整性守門不放寬。

## Capabilities

### New Capabilities

無；本次擴充既有能力，不另立同義能力。

### Modified Capabilities

- `desktop-app`: 已封存品質關卡的歷史入口、逐筆異常與重試、唯讀內容及跨項目隔離。
- `review-station`: 審查歷史清單與單筆查閱隔離缺損，同時維持嚴格寫入守門。
- `verify-station`: 驗證歷史清單與單筆查閱隔離缺損，同時維持安全驗收守門。
- `client-protocol`: 共用歷史清單增加選填 readError，保留 outcome 的原結果語意與健康輸出。
- `store-abstraction`: 檢查文件讀取區分不存在與儲存讀取失敗，避免權限／連結／IO 錯誤被吞成無紀錄。

## Impact

- Affected specs: 掃描 desktop-app、review-station、verify-station、client-protocol、store-abstraction；以上皆修改現有能力。change-lifecycle 的預設完整保留及按類別保留契約沿用，不另改設定。
- Affected code:
  - Modified: `crates/engine/speclink-core/src/quality/history.rs`, `crates/engine/speclink-core/src/store.rs`, `crates/engine/speclink-fs/src/checks.rs`, `crates/engine/speclink-fs/src/lib.rs`, `crates/host/speclink-host/src/history.rs`, `crates/host/speclink-host/src/bridge.rs`, `crates/protocol/speclink-protocol/src/query.rs`, `crates/adapters/speclink-cli/src/verbs/station.rs`, `packages/ui/src/checkRecords.ts`, `packages/ui/src/components/CheckRecords.tsx`, `packages/ui/src/components/ChangeDetailPage.tsx`, `packages/ui/src/components/ArchivedDetailPage.tsx`, `packages/ui/src/i18n.tsx`, `apps/desktop/src/adapter/checkRecords.ts`, `apps/desktop/src/App.tsx`。
  - Modified tests: `crates/engine/speclink-core/src/quality/station/tests.rs`, `crates/host/speclink-host/tests/security_remote.rs`, `crates/host/speclink-server/tests/it/api/review_api.rs`, `crates/adapters/speclink-cli/tests/it/remote_verb_parity.rs`, `packages/ui/src/__tests__/archivedDetailPage.test.tsx`, `packages/ui/src/__tests__/changeDetailPage.test.tsx`, `apps/desktop/src/__tests__/checkRecords.test.ts`, `apps/desktop/src/__tests__/App.test.tsx`；其餘回歸測試寫於對應既有模組。
  - New: 無預計新增的應用程式檔案。
  - Removed: 無。
- speclink-core 負責讀取與完整性判斷；speclink-fs 與 BridgeStore 提供檢查文件的可信讀取；speclink-host 為 Local／Remote／CLI／Node 共用查詢落點，Protocol 為 wire 型別唯一來源；Desktop adapter 與共用 UI 負責顯示。
- 不新增依賴套件、資料遷移、刪除工具或安全標準資源；規劃階段不修改正式規格與應用程式。

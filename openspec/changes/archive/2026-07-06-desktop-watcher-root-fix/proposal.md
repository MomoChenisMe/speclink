## Why

桌面 app 的檔案監看器掛載於「啟動當下工作目錄」拼接固定 openspec 名稱的路徑，而非查詢指令所用、經 Workspace::discover 向上探索出的專案根。以 Explorer 雙擊等 cwd 不等於專案根的方式啟動時，查詢因向上探索照常顯示資料，監看卻建立失敗且僅寫入 stderr（GUI 啟動下不可見），於是外部寫者（CLI、agent、編輯器）的變更完全不再自動反映——實際案例：apply 開工蓋章 started_at 後，看板卡片停留在「提案中」，直到重啟才更新。既有規格說監看「目前專案」的 openspec/ 樹，但從未定義「目前專案」相對啟動 cwd 如何解析——缺陷正是鑽了這個縫，因此規格層也要把解析語意釘死。

目標使用者與情境：透過 AI 代理跑 SDD 的開發者，在桌面看板旁執行 apply／task done 等 CLI 動詞（workflow 的 apply 階段），期待看板秒級反映開工狀態與任務進度。

## What Changes

- 桌面 Tauri 殼啟動時先以既有專案探索（speclink-desktop-core 的 init_core_context，內部即 Workspace::discover）解析實際專案根，監看器與 AppState 一律採用探索後的根——與查詢指令的根語意一致。
- 監看目標由「cwd 下的固定 openspec 目錄」改為探索出的 workspace spec 目錄，尊重非預設 spec 目錄名。
- cwd 不在任何 speclink 專案內時維持既有降級行為：app 照常運作、無自動刷新、錯誤記錄於日誌。
- desktop-app 規格新增一條需求：監看根解析 SHALL 與查詢的專案探索一致，明定非專案根 cwd 啟動時監看仍生效。

## Non-Goals

- 不做監看失敗的 UI 可見性（狀態提示、通知）——屬 desktop-config-multiproject 對根語意與專案切換的重做範圍。
- 不做專案切換時的監看重掛——同上。
- 不動 speclink-core 與 speclink-cli 兩個 crate：無 CLI 指令、旗標、輸出變動。
- 不改動 desktop-board-parity 待歸檔 delta 中既有的監看需求本文——本變更以獨立新增需求補縫，不與該 delta 交叉編輯。

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `desktop-app`: 新增（ADDED）需求「監看根解析與專案探索一致」——監看所依附的專案根 SHALL 由啟動 cwd 向上探索取得，與查詢指令同源；非專案根 cwd 啟動時自動刷新 SHALL 照常生效；探索不到專案時維持既有降級行為。

## Impact

- Affected specs: desktop-app（新增一條需求的 delta；不觸碰既有需求本文）
- Affected code:
  - Modified: apps/desktop/src-tauri/src/lib.rs（啟動時以專案探索解析根，監看與 AppState 共用該根）
  - Modified: apps/desktop/src-tauri/src/watch.rs（監看目標改收探索後的 spec 目錄路徑；既有測試同步調整）
  - New: 無
  - Removed: 無
- 影響的 crate：僅桌面 app（speclink-desktop 殼與其對 speclink-desktop-core 的呼叫）；speclink-core / speclink-cli 零改動。
- 相容性影響：無人眼或 --json 輸出變動，parity／color／twin 回歸對照不受影響。桌面 app 的行為變化僅限「非專案根 cwd 啟動時，外部變更自動刷新由失效變為生效」；cwd 即專案根的啟動方式行為不變。

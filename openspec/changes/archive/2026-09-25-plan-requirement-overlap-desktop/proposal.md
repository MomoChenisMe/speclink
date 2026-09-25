## Why

第一刀 plan-requirement-overlap 讓引擎與 CLI 的 plan 改用 requirement 級重疊、只以 depends_on 推波次，並新增 `requirementOverlap` 與 `archiveAfter` 兩個欄位。第一刀在品質站階段也把兩個欄位帶上了 wire（protocol 型別、server 的 GET /plan、remote convert），但桌面清單 payload 還停在舊的四欄，桌面詳情抽屜的排程分頁也仍以目錄級「重疊」段與獨立「阻擋」段呈現，而且它的版面（裸小標題加純文字）與審查／驗證分頁的卡片式版面不一致。目標使用者是在桌面看板上排優先序、決定封存順序的開發者：他們需要在排程分頁看到「為什麼被擋」「封存時要排在誰後面」「哪個 requirement 撞名」，remote 專案也要拿到同一份資訊。

本 change 為討論 plan-overlap-granularity 的第二刀（最後一刀）：桌面（清單 payload 與排程分頁）。原訂在本刀的 wire 群組（protocol、server、remote convert）已提前到第一刀。

## What Changes

- **桌面清單 payload 加欄位**：local 與 remote 的變更清單項在既有 `wave`、`blockedBy`、`dependsOn`、`overlaps` 四欄之外增列 `requirementOverlap` 與 `archiveAfter`，六欄同進同出，值取自引擎 plan 的同一入口；`speclink list --json` 仍不含任何排程欄位。
- **排程分頁改內容並卡片化**：四段改為卡片式版面（與審查／驗證分頁同一種邊框卡片，標題列與內容分格）。「波次」卡不變；「前置」卡的標題列右側加狀態徽章（blockedBy 空為「可以開工」、非空為「等 N 項」附前置名），原本獨立的「阻擋」段併入此徽章；「重疊」卡改列 requirement 級重疊（變更名、`capability › requirement`、雙方操作的小標籤），conflict 為 true 者標「同名衝突」，目錄級 overlaps 不再顯示；新增「封存順序」卡列 archiveAfter 的變更名並附一句「先封存它們，再對照正式規格重寫同名 requirement 後封存」，遇同名衝突或雙方互相等待時改顯示警示，其餘空時顯示「可直接封存」；舊 server 把重疊夥伴算進阻擋時，前置卡另列這些名稱，徽章數字與內容一致。
- **卡片波次章與系統匣不動**：blockedBy 語意已在第一刀窄化為宣告前置，「等待：a、b」tooltip 與變淡只反映前置，程式碼不變、只補測試。
- **相容性影響**：桌面清單 payload 為追加欄位，舊前端忽略即可；桌面讀舊 server 的 plan 回應時，缺席的兩個欄位沿 wire 型別的 default 讀作空陣列。排程分頁的 DOM 結構改變（`data-plan-section` 從 wave／depends／overlaps／blocked 改為 wave／depends／overlaps／archive），既有 planTab 測試同批改寫。i18n 新增與改寫的詞條列於 design。
- **不涉及 CLI 指令變更、不涉及設定欄位、不涉及技能資產**。

## Non-Goals

- 不改引擎演算法與 CLI 輸出（第一刀）。
- 不改 wire 型別、server GET /plan 與 remote convert（第一刀已帶上兩個欄位）。
- 不提供 `change rank` 的 remote 臂或桌面上的「插隊」按鈕——桌面拖排已是同一把手。
- 不重做卡片波次章與系統匣的視覺。
- 不在桌面或 server 加 `--strict-overlap` 對應的開關。

## Capabilities

### New Capabilities

（無。步驟 3 掃描命中 client-protocol、server-verb-api、desktop-app、tray-status-menu、change-plan、remote-board-order；全部落在既有 capability 的 requirement 修改，tray-status-menu 與 change-plan 的 remote 臂字面不變。）

### Modified Capabilities

- `client-protocol`：「變更清單的排程欄位」四欄改六欄；「remote 變更清單的排程欄位」四欄改六欄。（「plan 回應 payload」已在第一刀修改。）
- `desktop-app`：「詳情抽屜的排程分頁」改為卡片式四段與新內容；「看板卡片的波次與阻擋標示」字面不變。

## Impact

- Affected specs: client-protocol、desktop-app
- Affected crates／apps: apps/desktop/core、apps/desktop/src-tauri、packages/ui、apps/desktop
- Affected code:
  - New: （無）
  - Modified:
    - apps/desktop/core/src/query.rs（清單項注入六欄；匯出本機與 remote 共用的 PLAN_FIELDS 鍵清單）
    - apps/desktop/src-tauri/src/remote.rs（remote 清單合併 plan 的六欄，照 PLAN_FIELDS 逐鍵搬運）
    - apps/desktop/src-tauri/tests/it/remote_data.rs
    - packages/ui/src/adapter.ts（ChangeItem 加兩欄與 ChangeRequirementOverlap 型別）
    - packages/ui/src/stage.ts（planArchiveAfter／planRequirementOverlap 讀取入口）
    - packages/ui/src/index.ts（匯出 ChangeRequirementOverlap 型別，與 ChangeOverlap 並列）
    - packages/ui/src/components/RichDetailDrawer.tsx（PlanTab 卡片化與新內容）
    - packages/ui/src/i18n.tsx（排程分頁詞條）
    - packages/ui/src/components/DeltaBadges.tsx（匯出 DELTA_LABEL_KEYS，操作標籤與規格分頁共用用詞）
    - packages/ui/src/__tests__/planTab.test.tsx
    - packages/ui/src/__tests__/planBadge.test.tsx
    - packages/ui/src/__tests__/stage.test.ts
    - apps/desktop/src/__tests__/App.test.tsx
    - apps/desktop/src/__tests__/remoteCapabilities.test.tsx
    - apps/desktop/src/__tests__/helpers/changeList.ts（共用 requirementOverlap 測試資料）
  - Removed: （無）

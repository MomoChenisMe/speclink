## Why

規格頁與已封存頁以行內收合／展開呈現內容，與變更頁的抽屜模式形成兩套閱讀心智模型；收合卡片資訊貧乏（規格卡僅名稱＋相對時間，封存卡僅日期＋名稱＋任務數），無法在不展開的情況下判斷內容份量與狀態。專案分頁徽章顯示「進行中變更數」，在活躍分頁與看板「進行中」欄標頭純屬重複，對背景分頁也不是行動訊號（進行中代表 agent 在做事、不需要人）。本變更出自已結論討論 spec-archive-drawer-ux，目標使用者為透過桌面 app 檢視 SDD 工作狀態的開發者／PO／PM，對應 workflow 全階段的檢視面。

## What Changes

- 規格頁：移除行內展開與 chevron，整列點擊開啟唯讀規格抽屜（正典全文＋溯源 footer），寬度與變更詳情抽屜一致並帶全螢幕切換。
- 已封存頁：封存變更列與封存討論列同樣改為點擊開抽屜——封存變更抽屜呈現提案／設計／任務／規格四分頁唯讀檢視，封存討論抽屜呈現背景／討論過程／結論唯讀檢視；行內展開移除。
- 懶載入語意搬進抽屜：開啟才載入、refreshGen 世代重載（外部變更反映），與變更詳情抽屜同款資料流。
- 收合卡片資訊強化（清單 payload 擴欄位、不開新動詞）：
  - 規格卡：需求數、Purpose 首句摘要（偵測「TBD - created by archiving」佔位符時改顯「Purpose 待補」警示）、溯源變更數。
  - 封存變更卡：任務徽章配色分級（全完成／未全完成可辨）、觸及規格數、建立者標記、來源討論標記。
  - 封存討論卡：補 slug 複製鈕、衍生變更數。
- 新做的卡片一律採「標題文字後緊跟複製鈕」版面（標題＋複製鈕成群組、meta 靠右）；計數 meta 統一「裸 icon＋數字」樣式，任務數徽章維持 pill＋配色分級（2026-07-11 真實視窗驗證回饋）。
- 封存變更抽屜 header 顯示來源討論可點 chips，點擊於同一抽屜切換至該討論唯讀檢視（2026-07-11 真實視窗驗證回饋：原設計無法自封存變更連至討論）。
- 專案分頁徽章語意改為「待收尾數」＝已就緒變更數＋已結論未轉出討論數（等使用者執行動詞的卡片）；活躍分頁隨看板刷新派生、背景分頁 stats 快照擴欄位；tooltip 文案同步更新。
- openspec/LANGUAGE.md 收錄新詞「待收尾」。

相容性影響：桌面 app 內部的清單 payload 屬 app 自有介面，欄位為向後相容之新增；封存清單的衍生快取因欄位擴充需要版本遞升與重建（快取遺失重建語意既有）。speclink-core／speclink-cli 的 CLI 人眼與 --json 輸出一律不動，回歸對照不受影響。

## Non-Goals

- 看板卡片解剖學統一（ChangeCard／DiscussionColumn 的等寬標題、描述列、作者圓點化）——同討論扇出的後續變更，排在 desktop-ux-polish 落地後另行提案。
- desktop-ux-polish 在途任務的調整（其新增複製鈕的出生位置）——另經 ingest 併入該變更，不屬本變更。
- RichDetailDrawer 加唯讀模式——已於討論中否決（change 專屬互動太多，分支地獄）。
- 前端預讀全文計算卡片資訊——已否決（封存清單量大，啟動成本不可接受）。
- 不新增任何引擎動詞、CLI 子指令或設定欄位；不動 crates/speclink-core 與 crates/speclink-cli。

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `desktop-app`: 規格內容呈現由行內展開改為抽屜；已封存變更與封存討論檢視由行內展開改為抽屜；規格卡／封存變更卡／封存討論卡的收合資訊欄位擴充與複製鈕位置。
- `desktop-config`: 專案分頁徽章由「進行中變更數」改為「待收尾數」（已就緒變更＋已結論未轉出討論），活躍與背景分頁的更新語意不變。

## Impact

- Affected specs: `desktop-app`、`desktop-config`
- Affected crate: speclink-desktop-core（apps/desktop/core，清單 payload 與 stats 擴欄位）；speclink-core 與 speclink-cli 不動
- Affected code:
  - New:
    - packages/ui/src/components/SpecDrawer.tsx
    - packages/ui/src/components/ArchivedDrawer.tsx
    - packages/ui/src/__tests__/specDrawer.test.tsx
    - packages/ui/src/__tests__/archivedDrawer.test.tsx
  - Modified:
    - packages/ui/src/components/SpecList.tsx
    - packages/ui/src/components/ArchivedList.tsx
    - packages/ui/src/adapter.ts
    - packages/ui/src/index.ts
    - packages/ui/src/i18n.tsx
    - packages/ui/src/__tests__/specList.test.tsx（若既有測試檔名不同，以現行測試檔為準改寫）
    - apps/desktop/src/App.tsx
    - apps/desktop/src/store.ts
    - apps/desktop/src/tabs.ts
    - apps/desktop/src/adapter/workspace.ts
    - apps/desktop/core/src/query.rs
    - apps/desktop/core/src/cache.rs
    - apps/desktop/core/src/project.rs
    - openspec/LANGUAGE.md
  - Removed: (none)

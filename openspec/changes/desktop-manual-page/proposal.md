## Why

manual 技能（變更 manual-skill）會把正式規格轉成 `openspec/manual/*.md` 的 wiki 式手冊，但目前 desktop 沒有任何頁面能讀它——手冊的頭號讀者（PM、PO、第一天加入的新人）正是不開終端機、只用 desktop 的人。討論 `manual-generation-skill` 定案：呈現一致性由 desktop 的「手冊」頁承擔，不由生成端保證。本變更在 desktop 新增這個頁面：側欄樹、搜尋、上一頁／下一頁全部從各頁 frontmatter 機械推導，並把「可能過期」的頁與手冊生成後新增的規格標示出來。

## What Changes

- 側欄新增「手冊」導覽項（位於「規格」之後、「已封存」之前），進入手冊頁；零分頁時同其他頁呈空狀態引導頁。
- 手冊頁（唯讀）：左側依 frontmatter 的 `section` 分區、`order` 排序的頁面樹，附以標題與 `keywords` 過濾的搜尋列；右側渲染選定頁的 Markdown（沿用共用閱讀欄與行寬上限），頁尾提供上一頁／下一頁；頁尾出處行的 capability 名可點，切至規格頁並展開該規格卡。
- 過期標示：依 manual-pages 契約（來源規格最新 `@trace updated` 晚於頁 `generated`）在側欄該頁加「可能過期」標記；側欄底部另提示「手冊生成後新增且未入冊」的規格數（規格首次 `@trace updated` 晚於手冊最後生成日且不在任何頁的 `sources`）。
- Markdown 渲染新增 GitHub Alert 語法（`> [!NOTE]`、`[!TIP]`、`[!WARNING]`、`[!CAUTION]`）的提示框呈現，以介面語意色分層；此能力對所有 Markdown 檢視生效，既有內容無此語法者呈現不變。
- 外部變更即時反映：`openspec/manual/` 已在既有監看樹內；手冊頁的索引與已開啟頁面隨外部寫入（技能重生、手動編輯）於秒級重載。
- 資料層：desktop core 新增手冊查詢（讀目錄、解析 frontmatter、推導順序、計算過期與未入冊）；Tauri 殼新增兩個單行委派 command；資料源介面新增兩個方法，remote 資料源回報「尚不支援」。
- 文案：新增手冊頁的 zh-TW 與 en i18n 鍵（鍵集合維持相等），用詞遵循 LANGUAGE.md 的「手冊」「可能過期」詞條。

不新增 CLI 子指令、旗標或設定欄位；CLI 的人眼與 `--json` 輸出零變動。

**相容性影響**：
- 側欄由五項變六項，既有五項的順序、行為與無障礙標籤不變；desktop 的側欄快照測試隨本變更刻意更新。
- Markdown 元件對含 GitHub Alert 語法的內容改以提示框呈現，屬刻意變更；不含該語法的既有文件渲染逐位元不變。
- remote 模式（PM 無 checkout）：手冊頁呈現「remote 模式尚不支援手冊」的空狀態；手冊投影列於討論 Deferred，不在本變更。

## Capabilities

### New Capabilities

- `desktop-manual-page`: desktop 手冊頁的行為——頁面樹與搜尋的推導、內頁渲染與上下頁、出處跳規格、過期與未入冊標示、空狀態（無手冊／remote）、外部變更重載，以及 Markdown 的 GitHub Alert 提示框。掃描後無既有 capability 承載：`desktop-app` 管看板、規格頁、已封存頁與抽屜，不含任何讀取 `openspec/manual/` 的行為；`manual-pages`（in-flight）只定義頁格式契約，不含讀取端呈現。比照 `workspace-chooser`、`desktop-config` 自 `desktop-app` 拆出獨立 capability 的先例。

### Modified Capabilities

- `desktop-app`: 「側欄導覽結構」由五個導覽項改為六個，加入「手冊」。

## Impact

- Affected specs: 新增 `desktop-manual-page`；修改 `desktop-app`（側欄導覽結構）；引用 `manual-pages`（in-flight，變更 manual-skill）的格式與過期判定契約
- Affected code:
  - New:
    - apps/desktop/core/src/manual.rs（手冊目錄讀取、frontmatter 解析、順序推導、過期與未入冊計算）
    - packages/ui/src/components/ManualPage.tsx（手冊頁：側欄樹、搜尋、內頁、上下頁、出處）
    - packages/ui/src/__tests__/manualPage.test.tsx
    - apps/desktop/core/src/manual.rs 內的 #[cfg(test)] 單元測試
  - Modified:
    - apps/desktop/core/src/lib.rs（匯出 manual 模組）
    - apps/desktop/src-tauri/src/lib.rs（兩個單行委派 command）
    - apps/desktop/src/adapter/tauriDataSource.ts、apps/desktop/src/adapter/remoteDataSource.ts（實作新方法；remote 回報尚不支援）
    - packages/ui/src/adapter.ts（SpeclinkDataSource 新增 listManualPages 與 getManualPage 及其型別）
    - packages/ui/src/components/Markdown.tsx（GitHub Alert 提示框）
    - packages/ui/src/i18n.tsx、apps/desktop/src/i18n/messages.ts（手冊頁文案，zh-TW 與 en）
    - apps/desktop/src/App.tsx、apps/desktop/src/store.ts（側欄項、手冊視圖、外部變更重載）
    - apps/desktop/src/__tests__/App.test.tsx（側欄六項與切頁）
  - Removed: 無
- 影響的 crate／app：apps/desktop/core（純邏輯，`cargo test -p speclink-desktop-core`）、apps/desktop/src-tauri（委派）、apps/desktop（React）、packages/ui（元件與資料源介面）。speclink-core、speclink-cli 不動。
- 影響的技能與工具：無（技能側在變更 manual-skill）。

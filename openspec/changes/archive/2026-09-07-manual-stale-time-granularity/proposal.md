## Why

手冊頁的「可能過期」判定只比到日：來源規格的 `@trace updated` 與頁的 `generated` 同一天就算過期。使用者的常態工作流是「封存完立刻跑 /speclink-manual」，於是每次生成後，桌面 app 手冊頁都會對剛重生的頁標「可能過期」，而且同日再跑一次也消不掉（重生結果逐位元相同、零寫入）。2026-09-05 的實例：23:17 封存 discuss-search-recall，23:31 重生手冊，「認識資料」「討論」兩頁仍被標記。

目標使用者是透過 AI agent 跑 SDD 的開發者與 PO／PM：他們在封存（archive 技能）之後執行 manual 技能生成手冊，再於 desktop 手冊頁閱讀。本變更源自討論 manual-stale-time-granularity 的結論。

## What Changes

- **`@trace updated` 改寫成帶時區偏移量的 RFC 3339 時戳**（例 `2026-09-05T23:17:28+08:00`）。由 speclink-core 的 archive 在物化正典需求時注入；封存目錄名仍用純日期前綴，兩者取自同一個「現在」。只影響未來封存，現有正典的純日期行一律不回改。
- **手冊頁 `generated` 改寫成同格式的 RFC 3339 時戳**。由 manual 技能寫入；技能取當下時間寫入 frontmatter。
- **過期判定改成三段式，寫進 manual-pages 契約**，生成端（manual 技能）與讀取端（desktop 手冊頁）採同一基準：
  1. 兩邊都帶時間：規格時戳晚於頁時戳才算過期，同一秒不算。
  2. 任一邊只有日期：維持現行「規格日期不早於頁日期（同日也算）」。
  3. `sources` 為空、`generated` 缺席或格式不對、規格不存在：不標記。
- **「未入冊」判定同樣三段式**：規格最早的 `@trace updated` 相對全手冊最新的 `generated`，兩邊都帶時間時要嚴格晚於，否則退回同日也算。
- **desktop-core 的手冊索引改用新基準**：`apps/desktop/core/src/manual.rs` 解析 `updated:` 與 `generated` 時同時接受純日期與 RFC 3339，依三段式比較。
- **UI 文案不變**：側欄標記仍為「可能過期」（`manual.stale`），LANGUAGE.md 詞條只更新 definition 為三段式。
- **manual 技能檔（asset）更新**：stale page 的定義與 frontmatter 表的 `generated` 格式改為 RFC 3339；連動 ASSET_VERSION、render_golden 快照與 assets.lock。

相容性影響：

- `speclink archive` 之後正典 spec.md 的 trace 區塊 `updated:` 行人眼可見變長；沒有 `--json` 介面輸出此欄。archive 單元測試中以 `util::today()` 組出的 trace 期望值要改成新格式。
- desktop `list_manual_pages` 回傳的 `generated` 字串維持「frontmatter 原字串」，之後可能是 RFC 3339；前端不顯示此欄，無畫面變化。
- `speclink trace`、`speclink show`、規格頁 footer 只讀 `source`，不受影響。
- 既有純日期的規格與手冊頁不需遷移：下一個日曆日重生一次，頁的 `generated` 帶時間、舊規格純日期退回日比較，標記自然消失。

## Capabilities

### New Capabilities

（無。步驟 3 掃描到的相關規格 manual-pages、desktop-manual-page、manual-skill、verify-evidence 已涵蓋此行為，只需修改。archive-skill 只規定 trace 兩欄不規定格式，不動。）

### Modified Capabilities

- `manual-pages`：「frontmatter 六欄」的 `generated` 格式放寬為 RFC 3339 時戳（純日期仍可讀）；「過期判定基準」改為三段式，判定表補時間列。
- `desktop-manual-page`：「可能過期與未入冊的標示」改依契約三段式計算，判定表補時間列。
- `manual-skill`：「生成模式的讀取策略」讀取的是 `@trace updated` 時戳（不再限定日期）；「生成模式的輸出與報告」寫入的 `generated` 為 RFC 3339 時戳。
- `verify-evidence`：「archive trace 注入與零證據提示」的 `updated` 欄由「封存日期」改為「封存時戳（RFC 3339 帶時區偏移量）」。

## Impact

- Affected specs: manual-pages、desktop-manual-page、manual-skill、verify-evidence
- Affected crates／apps: speclink-core（archive 注入、util 時戳、manual 技能 asset）、speclink-desktop-core（apps/desktop/core，手冊索引判定）
- Affected code:
  - Modified:
    - crates/speclink-core/src/archive.rs（trace_block 改帶時戳；同一個「現在」同時產生目錄日期前綴與 updated；單元測試期望值）
    - crates/speclink-core/src/util.rs（新增取 RFC 3339 本地時戳的函式，與 today 共用同一瞬間）
    - crates/speclink-core/assets/skills/manual.md（stale page 定義、frontmatter 表 generated 格式、範例）
    - crates/speclink-core/src/init.rs（ASSET_VERSION 版號）
    - crates/speclink-core/tests/golden/assets.lock 與 crates/speclink-core/tests/golden/claude.snapshot.md、codex.snapshot.md、neutral-cli.snapshot.md、neutral-tool-call.snapshot.md（render_golden 快照）
    - apps/desktop/core/src/manual.rs（updated 與 generated 的雙格式解析、三段式 stale 與 uncovered 判定、單元測試）
    - openspec/LANGUAGE.md（「可能過期」詞條 definition）
    - .claude/skills/speclink-manual/SKILL.md 與 .agents 下對應渲染產物（由 speclink update 再生，不手改）
  - New: 無
  - Removed: 無

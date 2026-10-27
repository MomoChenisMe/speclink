## Why

前刀 desktop-reading-experience 修好了字內排版（字體、16px、列表符號、單換行、HTML 註解過濾），但抽屜文件仍是一條連續的字流：討論抽屜的討論過程分頁輪與輪之間只有標題邊距、欄位標籤與長文擠成一坨；變更抽屜四分頁內容全寬貼邊，全螢幕時行長遠超可讀範圍；規格分頁把 delta 機器標記直出成標題。另一半成因在內容側——discuss 技能的 add-round 範本慣性把 Position 寫成單行數百字。目標使用者是以桌面 app 檢視 SDD 文件的開發者／PO／PM，情境是討論抽屜與變更抽屜（提案／設計／任務／規格分頁）的日常閱讀。源自討論 drawer-document-readability。

## What Changes

- 討論過程分頁結構化：每輪一張卡片——以 CLI scaffold 固定格式（Round N — mode (date) 的 h3 標題）切分，卡頭呈現輪次徽章、mode chip 與日期，卡身把 Focus／Position／Ruled out／Open 粗體前綴拆成標籤欄位區塊；無法照格式切分的記錄整篇單一 markdown 檢視退回（與現行 sections fallback 同型）。已封存討論檢視渲染同一 Rounds 形狀，重用同一輪卡片元件。
- 抽屜 markdown 文件容器：內容行寬設上限並保留容器留白，取代現行全寬貼邊；變更抽屜提案／設計／規格分頁、討論抽屜各分頁與已封存檢視經共用 Markdown 元件一體生效。
- 規格分頁去機器標記：變更抽屜規格分頁將 delta 區段標題（ADDED／MODIFIED／REMOVED／RENAMED Requirements）轉為色標區段標題，色彩對齊 DeltaBadges 既有配色（新增綠、修改琥珀、移除紅、更名藍）；requirement 原文照排，不逐條卡片化。已封存變更檢視的規格分頁渲染同一 delta 形狀，重用同一區段元件。
- 結論分頁欄位標籤化（實作驗收後追加）：conclude scaffold 的六個粗體前綴欄位（Decision／Rationale／Rejected alternatives／Deferred／Capture to／Next）比照輪卡片拆成標籤區塊（決定／理由／否決替代案／擱置／記錄去向／下一步），前綴原文不直出；來源缺席的欄位不渲染；無任何白名單欄位的結論整篇單一 markdown 檢視退回。已封存討論檢視的結論區重用同一元件。
- discuss 技能 add-round 範本小改：Position 改為鼓勵列點多行（單行長文是文字牆的內容側成因）；影響 discuss 一個技能、claude 與 codex 兩工具實例（crates/speclink-core/assets 為單一來源，repo 技能實例與 render golden 同步再生）；既有討論記錄不回改。
- 相容性影響：CLI 人眼輸出與 --json 逐位元不變（無指令行為變更，speclink-core／speclink-cli 程式邏輯不動）；render golden 快照因技能內容更新而刻意再生，屬預期 diff。

## Non-Goals

- 規格分頁 requirement 逐條卡片與 scenario 展開（Spectra 全套）——範圍膨脹，色標區段已除噪音主因，需要時另開刀。
- 任務分頁——已是結構化元件，互動與工具列在 desktop-task-interactions 刀。
- 左側導覽規格頁（desktop-specs-view 刀）的功能——其 prose 渲染經共用 Markdown 元件自然吃到文件容器，但不在此刀驗收。
- 既有討論記錄不回寫；CLI 的 discuss new scaffold（Document rules 註解）不動。
- 背景分頁不做結構化——context 是自由散文、無固定欄位可解析；已由文件容器（行寬與 prose）涵蓋，維持現狀。
- 純 CSS 間距微調方案——做不出區塊邊界，討論已否決。

## Capabilities

### New Capabilities

（無）

### Modified Capabilities

- `desktop-app`: 新增文件版面結構需求——討論過程分頁每輪以卡片呈現（卡頭輪次／mode／日期＋欄位標籤區塊、非標準格式整篇退回）；討論結論以欄位標籤呈現（六欄位白名單、同型退回）；markdown 文件內容行寬有上限且留白一致；變更抽屜規格分頁的 delta 區段標題以色標呈現、機器標記不直出。

## Impact

- Affected specs: desktop-app
- Affected crates: 無 Rust 程式邏輯變更（speclink-core／speclink-cli 不動）；speclink-core 僅 assets 內容與 render golden 快照更新
- Affected code:
  - Modified: packages/ui/src/components/DiscussionDrawer.tsx、packages/ui/src/components/Markdown.tsx、packages/ui/src/components/RichDetailDrawer.tsx、packages/ui/src/components/ArchivedList.tsx、packages/ui/src/components/DeltaBadges.tsx、packages/ui/src/delta.ts、packages/ui/src/i18n.tsx、apps/desktop/src/index.css、packages/ui/src/__tests__/delta.test.ts、packages/ui/src/__tests__/discussionDrawer.test.tsx、packages/ui/src/__tests__/richDrawer.test.tsx、packages/ui/src/__tests__/archivedList.test.tsx、packages/ui/src/__tests__/components.test.tsx、crates/speclink-core/assets/skills/discuss.md、.claude/skills/speclink-discuss/SKILL.md、.agents/skills/speclink-discuss/SKILL.md、crates/speclink-core/tests/golden/claude.snapshot.md、crates/speclink-core/tests/golden/codex.snapshot.md、crates/speclink-core/tests/golden/neutral-cli.snapshot.md、crates/speclink-core/tests/golden/neutral-tool-call.snapshot.md
  - New: （無）
  - Removed: （無）
- 新增依賴：（無）

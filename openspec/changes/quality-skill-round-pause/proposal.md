## Why

/speclink-quality 目前的時序是「兩站檢查完 → 自動統一修正 → 自動複驗 → 自動接連蓋章」，代理從頭到尾不徵詢使用者——實際收尾輪（quality-skill-canonicalization）中使用者明確要求：每一輪兩站檢查完成後應停下來，由使用者決定修哪些、何時蓋章，技能不得自顧自把所有發現全修掉。討論 quality-skill-pause-and-ui-polish（2026-08-07 結論）裁定：技能全程不自行跨階段，乾淨輪也停。

目標使用者：透過 AI 代理跑 SDD 的開發者，於 workflow 品質站階段（apply 完成後、archive 前）呼叫 /speclink-quality 技能。

## What Changes

- 正典技能 asset `crates/speclink-core/assets/skills/quality.md` 的時序改為「每輪暫停制」：
  - 每一輪兩站（審查 → 驗證）檢查完成後，彙整兩站 findings 停下來詢問使用者下一步（全修／挑著修／不修就停），未經裁示不開修
  - 修正落地後的每一輪兩站複驗完成後，同樣停下詢問
  - 乾淨輪（兩站零 findings）也停：報告兩站皆綠，由使用者決定是否進入收尾補蓋與封存；蓋章與封存建議不再自動發生
  - 兩站的收尾補蓋機制（明示收尾補蓋呼叫、兩章接連落、中間零編輯）原樣保留，僅改為在使用者裁示後觸發
- `crates/speclink-core/src/skills.rs` 的 quality 技能條目 description 同步改寫為每輪暫停語意
- golden 斷言釘住暫停語意（`crates/speclink-core/tests/it/render_golden.rs`）
- MARKER_VERSION v1.17.4 → v1.18.0，乾淨樹再生 golden snapshots 與 assets.lock，並以 speclink update 落地本 repo 生成物
- README.md 與 README.en.md 分工表的 quality 時序一句同步改寫

## Non-Goals

- 不動 review／verify 兩站自身的檢查、工單、蓋章語意與其既有的「quality 時序例外」段——兩站「先不蓋章」離場機制原樣沿用
- 不新增引擎狀態、CLI 子指令或設定欄位——暫停純屬技能編排文字層
- 單站直接呼叫的行為不變（修完即蓋章預設維持）
- 桌面 UI 修整（tooltip 延遲、詳情章列、tray hover、截斷統一、sticky 橫幅）屬同一討論轉出的另一變更，不在本變更範圍

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `quality-skill`: 「兩站時序的編排行為」requirement 改為每輪暫停制——每輪兩站檢查／複驗完成後 SHALL 停下詢問使用者下一步，乾淨輪亦停；補蓋與封存建議 SHALL 於使用者裁示後才發生

## Impact

- Affected specs: `quality-skill`（modified）
- Affected code:
  - Modified: crates/speclink-core/assets/skills/quality.md、crates/speclink-core/src/skills.rs、crates/speclink-core/src/init.rs、crates/speclink-core/tests/it/render_golden.rs、crates/speclink-core/tests/golden/assets.lock、crates/speclink-core/tests/golden/claude.snapshot.md、crates/speclink-core/tests/golden/codex.snapshot.md、crates/speclink-core/tests/golden/neutral-cli.snapshot.md、crates/speclink-core/tests/golden/neutral-tool-call.snapshot.md、README.md、README.en.md
  - Modified（speclink update 落地生成物）: .claude/skills/speclink-quality/SKILL.md、.agents/skills/speclink-quality/SKILL.md；版號波及——.claude/skills/ 與 .agents/skills/ 全部 SKILL.md 的 frontmatter 版號、CLAUDE.md 與 AGENTS.md 的 SPECLINK 標記版號
  - New: (none)
  - Removed: (none)
- 相容性影響：CLI 人眼輸出與 --json 皆無變動；技能檔生成內容與版號屬刻意變更，golden 對照（render_golden）同批更新；既有專案下次執行 speclink update 取得新版技能檔，無需遷移動作
- 影響工具：claude 與 codex 兩個 render target 的 speclink-quality 技能檔，以及 CLAUDE.md／AGENTS.md 的技能清單條目文字與版號標記

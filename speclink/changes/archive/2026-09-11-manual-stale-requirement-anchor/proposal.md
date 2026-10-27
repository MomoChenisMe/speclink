## Why

manual-stale-time-granularity 修好了「同日平手」的誤標，但桌面 app 手冊側欄仍有三頁（認識桌面 app、規格／討論／已封存與搜尋、桌面上的品質關卡）長期標「可能過期」。根因有兩層：過期判定以整個 capability 為單位，而 `desktop-app` 一份規格（3324 行、71 個 @trace 區塊）餵了四頁手冊，任何一次動到桌面 app 的封存都讓四頁全亮；同時 manual 技能規定「重生後內文逐位元相同即不寫」，內文沒受影響的頁 `generated` 永遠停在舊值，燈永遠消不掉。這對透過桌面 app 讀手冊的 PM／PO 與開發者是持續的假警報，對跑 /speclink-manual 的 agent 則是每次都要重讀整份大規格。

## What Changes

- **來源錨定到 Requirement（讀取端粒度）**：manual-pages 契約的 `sources` 每一項可帶 `#<Requirement 名>` 錨定（例 `desktop-app#看板與任務`）。帶錨定的項，過期判定只取該 Requirement 段落內的 `@trace updated`；不帶錨定的項維持整份規格語意。既有 41 頁不回填，各頁下次重生時自然換成錨定寫法。錨定的 Requirement 在正典規格中找不到（改名或刪除）時，該項視為過期——寧多亮一次燈，不靜默失聯。未入冊判定與頁尾出處行都只看井號前的 capability 名。
- **內文不變只推進時戳（生成端）**：manual 技能重生一頁後內文逐位元相同時，改為只推進該頁 `generated` 為本次時戳，其餘位元不動；摘要另計「只換時戳」的頁數。manual-pages 契約「未被重生的頁逐位元不變」對應放寬：被判過期且重生後內文不變的頁，得只更新 `generated`。
- **桌面端讀取端跟進**：`apps/desktop/core` 的手冊索引解析錨定、按 Requirement 段落取時戳（切段沿用引擎 `speclink_core::archive::parse_canonical`，該函式由 crate 內部改為公開）；`list_manual_pages` 回傳的 `sources` 仍為 frontmatter 原字串陣列（含錨定），欄位形狀不變。
- **桌面前端出處列跟進**：`packages/ui` 的手冊頁出處列讀索引 `sources` 畫可點的 capability chip；改為取每項井號前的名稱再去重，錨定頁的出處仍可點、不重複。
- **詞彙表同步**：`openspec/LANGUAGE.md`「可能過期」詞條的定義補上錨定範圍與「內文不變只推進 generated」。
- **技能 asset 同批更新**：manual 技能本文改寫「stale page」定義、`sources` 欄說明與「byte-identical」規則；ASSET_VERSION 進版，golden 與 assets.lock 同批再生。影響 claude 與 codex 兩個目標的 manual 技能。

## Capabilities

### New Capabilities

（無）

### Modified Capabilities

- `manual-pages`：`sources` 項格式放寬為可帶 Requirement 錨定；過期判定基準改為錨定段落級；重生保序規則放寬「只換時戳」。相關規格掃描：manual-pages 即本契約，desktop-manual-page 與 manual-skill 為其兩個引用端，三者皆列入本變更。
- `manual-skill`：生成模式輸出改為「內文相同只推進 generated」並在摘要計數；生成新頁與重生頁時 `sources` 寫 Requirement 錨定。
- `desktop-manual-page`：側欄「可能過期」判定改依契約的錨定段落級規則，錨定找不到即標記。

## Impact

- Affected specs: `manual-pages`、`manual-skill`、`desktop-manual-page`
- Affected code:
  - Modified: `apps/desktop/core/src/manual.rs`（錨定解析、按 Requirement 段落取 @trace 時戳、錨定失效判定與測試）
  - Modified: `crates/engine/speclink-core/src/lifecycle/archive.rs`（`parse_canonical` 改為 pub，供桌面讀取端共用同一條切段規則）
  - Modified: `packages/ui/src/components/ManualPage.tsx`、`packages/ui/src/__tests__/manualPage.test.tsx`（出處列取井號前名稱去重）
  - Modified: `openspec/LANGUAGE.md`（「可能過期」詞條）
  - Modified: `crates/engine/speclink-core/assets/skills/manual.md`（stale page 定義、sources 欄說明、byte-identical 改只推進時戳、摘要新增只換時戳計數）
  - Modified: `crates/engine/speclink-core/src/workspace/init.rs`（ASSET_VERSION 進版）
  - Modified: `crates/engine/speclink-core/tests/golden/claude.snapshot.md`、`crates/engine/speclink-core/tests/golden/claude-worktree.snapshot.md`、`crates/engine/speclink-core/tests/golden/codex.snapshot.md`、`crates/engine/speclink-core/tests/golden/neutral-cli.snapshot.md`、`crates/engine/speclink-core/tests/golden/neutral-tool-call.snapshot.md`、`crates/engine/speclink-core/tests/golden/assets.lock`（隨 asset 內文再生）
  - Modified: `.claude/skills/speclink-manual/SKILL.md` 與 `.agents/` 下對應的 manual 技能檔（由 speclink update 再生，不手改）
  - New: 無
  - Removed: 無
- 相容性影響：
  - `list_manual_pages` 的 `--json` 形狀不變（`sources` 仍為字串陣列，內容多了可選的 `#錨定`；`stale` 布林不變）。桌面前端出處列讀這個欄位，已同批改為只取井號前名稱。
  - 不帶錨定的舊頁判定結果與現在完全相同；只有帶錨定的頁判定範圍變窄。
  - 既有 41 頁手冊不在本變更內回改；本變更也不重生手冊，重生由使用者事後跑 /speclink-manual。
  - 引擎封存寫 `@trace` 的格式不變；trace 動詞不變。
- 影響的技能與工具：manual 技能（claude、codex 皆渲染）；其他技能不變，但 ASSET_VERSION 進版會讓 speclink update 再生全部技能檔的版號行。

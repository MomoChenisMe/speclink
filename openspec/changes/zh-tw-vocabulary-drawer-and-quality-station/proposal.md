## Why

使用者裁定現行兩個繁中用詞不合台灣用語習慣，要收斂為：「抽屜」→「詳情面板」、「品質站」→「品質關卡」。

`user-docs-overhaul` 已把 `docs/` 與兩份 README 的這兩個詞換掉，但那個變更的 Non-Goals 明列不動行為、CLI 動詞與 GUI，因此**文件以外**的使用者可見面完全沒動。現況是同一個產品裡文件講「詳情面板／品質關卡」、GUI 與技能輸出仍講「抽屜／品質站」，使用者在兩個介面之間會看到兩套詞。

更關鍵的是**沒有任何機制擋住詞彙漂回去**。這兩個詞目前不是 `openspec/LANGUAGE.md` 的獨立詞條，只散落在其他詞條的定義文裡；沒有詞條就沒有 `avoid` 清單，下一個寫 GUI 文案的代理沒有依據可循，改完就會再漂回來。這是本次要一併關掉的洞。

目標使用者是透過 AI 代理跑 SDD 的開發者；使用情境橫跨 desktop GUI 的封存與審查提示、server-web 後台的首次導覽，以及 worktree 兩支技能對使用者說的交棒話術。

## What Changes

- **改掉全部使用者可見的兩個舊詞**（共 9 處）：desktop 的品質關卡工單提示、server-web 的兩則導覽提示、以及 worktree 兩支技能資產對使用者說的交棒文字。
- **把兩個詞立為 `openspec/LANGUAGE.md` 正典詞條**，各帶 definition／avoid／why 與裁定日期，舊詞進 `avoid`——這是防漂回的正典錨點。
- **更新 LANGUAGE.md 兩條既有明文例外的字面用詞**（討論 slug 例外、worktree 例外）：只換用詞，裁定內容、適用範圍與四筆範圍擴充紀錄一字不動。
- **新增詞彙守門測試** `scripts/vocabulary-guard.test.mjs`，掛進既有的 `node --test "scripts/**/*.test.mjs"` 套件，掃描使用者可見文案面確認舊詞歸零。
- **BREAKING（產物層）**：技能資產內文改動觸發既有的三連動——`MARKER_VERSION` 自 `v1.19.12` 進版、golden 快照重生、`assets.lock` 重生。既有工作區跑 `speclink update` 會整套再生受管檔。

### 相容性影響

- 人眼輸出：desktop 一則 toast 文字與兩支 worktree 技能的交棒文字改字面；`--json` 欄位名與 shape **完全不變**（改的是繁中訊息內容，不是鍵名）。
- 回歸對照：`crates/speclink-core/tests/golden/claude-worktree.snapshot.md` 與 `assets.lock` 屬**刻意變更**，同批重生。
- 遷移：既有工作區執行 `speclink update` 即取得新文案；未執行者維持舊文案，功能不受影響（純文案）。

### 影響的 crate 與 app

`speclink-core`（技能資產與 `MARKER_VERSION`）、`apps/desktop`、`apps/server-web`。不動 `speclink-cli`、`speclink-host`、`speclink-server` 的行為。

### 技能與工具影響

影響 `speclink-apply-with-worktree` 與 `speclink-worktree-merge` 兩支技能的內文，claude 與 codex 兩個工具的產出（`.claude/skills/`、`.agents/skills/`）皆隨資產再生。

## Non-Goals

- **不做全 repo 大改名**：程式碼註解、測試名稱與 `openspec/specs/` 散文的舊詞不在本次回改範圍（理由與替代控制見 design D1）。
- **不動 `openspec/changes/archive/`**：已封存變更是稽核資料，不回改（design D3）。
- **不改英文文案**：`LANGUAGE.md` 範圍已排除英文 CLI 輸出，`quality-station` 等英文字面維持不動。
- **不改任何識別符**：`RichDetailDrawer`、`SpecDrawer`、`archivedDrawerBase` 等 Rust／TS 識別符與 CSS 類名一律不動。
- **不改任何行為**：純文案與正典詞彙，無邏輯、無 CLI 旗標、無 `--json` 形狀變更。

## Capabilities

### New Capabilities

- `ui-copy-vocabulary`: 使用者可見繁中文案的正典詞彙守門——界定「使用者可見文案面」的範圍，要求該面不出現 `LANGUAGE.md` 的 avoid 詞，並以自動化守門測試釘死。

### Modified Capabilities

- `worktree-apply-skill`: 技能收尾指示的正典敘述由「品質站」改為「品質關卡」。
- `worktree-merge-skill`: 技能收尾流程指示的正典敘述由「品質站」改為「品質關卡」。

## Impact

- Affected specs: `ui-copy-vocabulary`（新增）、`worktree-apply-skill`、`worktree-merge-skill`
- Affected code:
  - New:
    - scripts/vocabulary-guard.test.mjs
  - Modified:
    - openspec/LANGUAGE.md
    - apps/desktop/src/i18n/messages.ts
    - apps/server-web/src/i18n/messages.ts
    - crates/speclink-core/assets/skills/apply-worktree-post.md
    - crates/speclink-core/assets/skills/worktree-merge.md
    - crates/speclink-core/src/init.rs
    - crates/speclink-core/tests/golden/claude-worktree.snapshot.md
    - crates/speclink-core/tests/golden/assets.lock
  - Removed: （無）

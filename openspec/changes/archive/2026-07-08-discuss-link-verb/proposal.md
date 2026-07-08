## Why

討論結論走 `/speclink-ingest` 併入既有變更時，生命週期斷鏈：自動封存由變更側 from_discussion 鏈驅動，而該鏈只有 promote（建新變更）會鑄——ingest 僅為技能、非引擎動詞，兩側皆無連結，討論以 concluded 永卡看板。conclude 模板的 Next 欄位明文支持 ingest 路徑，屬設計缺口而非誤用；事故現場：討論「專案選擇對齊-spectra」在 desktop-config-multiproject 完成封存後仍滯留看板，需人工發現並手動封存。

目標使用者是透過 AI agent 跑 SDD 的開發者／PO／PM；使用情境是 discuss 技能的 conclude 步驟與 ingest 技能——結論指向既有變更時，agent 執行本動詞鑄鏈，後續看板群組、抽屜互跳與自動封存全部接上既有機制，不再靠人記得。

## What Changes

- `speclink-core`：discuss 模組新增 link 動詞的流程函式——對既有變更的 meta 寫入 from_discussion、討論側標記 promoted（promoted_to 累加，與 promote 共用同一標記機制）。
- `speclink-cli`：新增子指令 `speclink discuss link <slug> <change>`（兩個位置參數；旗標僅 `--json`，與 promote 對齊；不吃 stdin）。成功時 exit code 0 並輸出一行成功訊息；守衛失敗時以非零 exit code 結束、stderr 說明原因，且 SHALL NOT 寫入任何一側。
- 守衛：討論不存在或已封存、目標變更不存在、目標變更已有其他 from_discussion（欄位單值）時拒絕。
- discuss 技能：conclude 步驟新增指示——結論的 Capture to 指向既有變更時，先執行 link 再導向 `/speclink-ingest`。
- ingest 技能：新增提示——自討論結論而來時確認來源討論已 link。
- 內嵌技能資產三處同步：crates/speclink-core/assets、.claude/skills、.agents/skills（claude 與 codex 兩工具皆受影響），render golden 基準隨之再生。
- openspec/LANGUAGE.md：「已轉出變更」定義自「至少轉出過一個變更」放寬為「至少連結一個變更」。

相容性影響：純新增子指令，既有指令的人眼與 `--json` 輸出逐位元不變（discuss list 的 status: promoted 為既有值域，僅多一條產生路徑）；parity／color 回歸對照不受影響。技能內容變動屬 render golden 既有更新慣例（乾淨樹再生）。

## Non-Goals

- 桌面 GUI 與 node bridge 曝露 link——本動詞由 agent 技能流程驅動，無 GUI 需求；需要時另刀。
- from_discussion 多值累加（一個變更連結多份討論）——欄位維持單值，衝突即拒絕；真實需求出現再議。
- 自動封存規則本身不動——archive 側的既有機制照舊，本刀只補鑄鏈。
- conclude 結論文字解析自動連結——脆弱的自由文字解析，討論階段已否決。

## Capabilities

### New Capabilities

（無）

### Modified Capabilities

- `discussion-docs`: 新增「討論以 link 動詞併入既有變更」需求——連結語意、守衛條件與 CLI 契約。

## Impact

- Affected specs: discussion-docs
- Affected code:
  - Modified:
    - crates/speclink-core/src/discuss.rs
    - crates/speclink-cli/src/main.rs
    - crates/speclink-cli/src/commands.rs
    - crates/speclink-core/assets/skills/discuss.md
    - crates/speclink-core/assets/skills/ingest.md
    - .claude/skills/speclink-discuss/SKILL.md
    - .agents/skills/speclink-discuss/SKILL.md
    - .claude/skills/speclink-ingest/SKILL.md
    - .agents/skills/speclink-ingest/SKILL.md
    - crates/speclink-core/tests/golden/claude.snapshot.md
    - crates/speclink-core/tests/golden/codex.snapshot.md
    - crates/speclink-core/tests/golden/neutral-cli.snapshot.md
    - crates/speclink-core/tests/golden/neutral-tool-call.snapshot.md
    - openspec/LANGUAGE.md
  - New: （無）
  - Removed: （無）

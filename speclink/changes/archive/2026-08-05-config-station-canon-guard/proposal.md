## Summary

把「品質站技能已承載的正典標準不得重述」納入 speclink-config 技能判準一的反證集合，防止 workflow config 長出第二份會 drift 的正典。

## Motivation

判準一目前只反證引擎注入內容（政策開關與 schema 內建 instruction，以 `speclink instructions <artifact> --json` 的 payload 逐條比對）。審查站落地後，smell baseline 等正典標準改由品質站技能檔承載，不出現在任何 instructions payload——當使用者或未來的 config 執行想把 12 條 Fowler smells 抄進 rules 時，判準一抓不到這種重複，會產生第二份正典並隨時間 drift。code-review-stage 的裁量討論中已實際出現過此提議，防護值得釘進技能而非留在對話紀錄。

目標使用者：透過 AI 代理執行 `/speclink-config` 維護 workflow config 的開發者。使用情境：config 技能的 Step 2 判準審查階段。

## Proposed Solution

判準一的反證集合擴充為兩類：(a) 引擎注入內容——維持既有 payload 逐條反證；(b) 品質站技能承載的正典標準（如審查站的 smell baseline）——對照生成的品質站技能檔內容，同樣不得憑印象。改動落點：config-skill 正典規格的判準需求（MODIFIED delta）、speclink-core 內嵌的 config 技能模板 Criterion 1 段落、golden 再生（模板內容變更須遵循 MARKER_VERSION 版本鎖紀律同批 bump）。

## Non-Goals

- 不改四判準的其他三條與固定輸入來源清單
- 不改品質站技能本身——review 的 smell baseline 正典位置不動
- 不為 verify 站另寫條文——verify 落地後其維度自然屬「品質站技能承載的正典」，判準文字無需再改
- 不做 config 文件回溯掃描——本 change 只動技能文字；現行 openspec/config.yaml 已於重整時逐條確認無此類重複

## Alternatives Considered

- 把 smell baseline 也放進 instructions payload 讓既有反證涵蓋——否決：baseline 是審查站 sub-agent 的指示本文，不是 artifact 撰寫指引，塞進 payload 是為了反證而扭曲注入面
- 只靠對話共識不落地——否決：結論會隨 session 消失，防護必須在技能文字裡

## Impact

- Affected specs: `config-skill`（修改）
- Affected code:
  - Modified: crates/speclink-core/assets/skills/config.md、crates/speclink-core/src/init.rs、crates/speclink-core/tests/it/render_golden.rs、crates/speclink-core/tests/golden/assets.lock、crates/speclink-core/tests/golden/claude.snapshot.md、crates/speclink-core/tests/golden/codex.snapshot.md、crates/speclink-core/tests/golden/neutral-cli.snapshot.md、crates/speclink-core/tests/golden/neutral-tool-call.snapshot.md、.claude/skills/speclink-config/SKILL.md、.agents/skills/speclink-config/SKILL.md
  - Modified（MARKER_VERSION 版本戳的機械後果，非本 change 的文字改動）: 其餘帶版號戳記的生成檔與 golden——兩工具的所有 SKILL.md、AGENTS.md、CLAUDE.md、crates/speclink-core/tests/golden/claude-worktree.snapshot.md、crates/speclink-core/tests/golden/remote-claude.marker.md
  - New: 無
  - Removed: 無

相容性影響：CLI 行為與 `--json` 輸出零變化；僅技能檔文字變更與 MARKER_VERSION 版本戳遞增（生成檔於 speclink update 後更新，claude 與 codex 兩工具皆受影響）。

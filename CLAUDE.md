<!-- SPECLINK:START v1.2.0 -->

# Speclink Instructions

This project uses Speclink for Spec-Driven Development(SDD). Specs live in `openspec/specs/`, change proposals in `openspec/changes/`, discussion records in `openspec/discussions/`.

## Use `/speclink-*` skills when:

- Requirements are fuzzy or worth debating → `/speclink-discuss` (recorded as a document; promote turns it into a change)
- User wants to plan, propose, or design a change → `/speclink-propose` (`--from-discussion <slug>` seeds it from a concluded discussion)
- Adopting Speclink on an existing codebase → `/speclink-onboard`
- Tasks are ready to implement → `/speclink-apply`
- Resuming a change that sat idle → run `/speclink-drift` first
- Requirements change mid-work → `/speclink-ingest`
- Implementation is done → `/speclink-verify`, then `/speclink-archive`
- Commit only files related to a specific change → `/speclink-commit`

## Workflow

discuss? → propose → apply ⇄ ingest → verify? → archive

- `discuss` is optional — skip if requirements are clear; conclude and archive it even when the outcome is "don't do it"
- A promoted discussion is archived automatically with its last remaining change (one discussion can fan out into several changes)
- Resuming after a pause? Run `drift` first — stale delta assumptions route to `ingest`
- Requirements change mid-work? Plan mode → `ingest` → resume `apply`

<!-- SPECLINK:END -->

# 開發備忘（跨機器）

## 桌面 app（apps/desktop、packages/ui）

- **GUI 改動必須真實視窗驗證**：jsdom（vitest）測不出 pointer/拖曳互動失效。用 PowerShell 啟動 release exe ＋ Win32 SetCursorPos/mouse_event 實點 ＋ CopyFromScreen 截圖檢視。**操作前先確認使用者沒在使用螢幕**。
- dnd-kit 可拖曳元素必須設 `PointerSensor activationConstraint: { distance: 8 }`，否則單純點擊被拖曳監聽吃掉；拖曳視覺用 DragOverlay（否則被欄位 overflow 裁切）。
- 常用指令：`npm test -w packages/ui`、`npm test -w apps/desktop`、`npm run build -w apps/desktop`（vite → dist）、`cargo build --release -p speclink-desktop`（重建前先關閉執行中的 exe，否則 linker 存取被拒）。
- 前端體系：Tailwind v4（`@source` 納入 packages/ui）＋ shadcn 原語（原始碼在 packages/ui/src/components/ui/）＋ Zustand（apps/desktop/src/store.ts）；主色 teal（index.css 的 oklch hue 192）。

## 引擎/設定既知風險

- `openspec/config.yaml` 任何 YAML 解析錯誤會使**整份政策靜默退回預設**；rules 條目勿以反引號開頭；桌面設定頁寫入前後須驗證可解析。
- CLI 輸出是回歸保護對象：重構前先保存 baseline exe 做自我基線雙沙盒對照（scratchpad 基建會消失，勿依賴）。

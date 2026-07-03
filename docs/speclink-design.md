# Speclink 設計決策（對照 Spectra 2.3.1）

本文件記錄 speclink 相對於 spectra 的對映與差異，作為實作與技能轉換的單一真實來源（source of truth）。Ground truth 由已安裝的 `spectra.exe` 2.3.1 在乾淨沙箱擷取，存於 scratchpad `gt/`。

## 1. 品牌 / 命名對映

| 面向 | Spectra | Speclink |
|---|---|---|
| CLI 執行檔 | `spectra` / `spectra.exe` | `speclink` / `speclink.exe` |
| 應用設定檔 | `.spectra.yaml` | `.speclink.yaml` |
| 工作流設定檔 | `openspec/config.yaml` | `openspec/config.yaml`（不變，維持一致） |
| 規格目錄 | `openspec/`（`spec_dir`） | `openspec/`（`spec_dir`，不變） |
| 變更中繼資料 | `openspec/changes/<c>/.openspec.yaml` | 同（不變） |
| 工作資料目錄 | `.spectra/` | `.speclink/` |
| 技能目錄/前綴 | `.claude/skills/spectra-*`，`/spectra-*` | `.claude/skills/speclink-*`，`/speclink-*` |
| 其他 agent 前綴 | `$spectra-*` | `$speclink-*` |
| CLAUDE.md 標記 | `<!-- SPECTRA:START vX -->` | `<!-- SPECLINK:START vX -->` |
| 指令標題 | `Spectra Instructions` | `Speclink Instructions` |
| SKILL frontmatter | name `spectra-*`、author `spectra`、generatedBy `Spectra`、compatibility `Requires spectra CLI.` | `speclink-*`、`speclink`、`Speclink`、`Requires speclink CLI.` |
| demo 變更前綴 | `spx-<字>-<pokemon>` | `slx-<字>-<pokemon>` |
| drift 建議 | `spectra archive X --skip-specs` | `speclink archive X --skip-specs` |
| feedback 目標 | kaochenlong/Spectra | speclink（本專案 issues） |

> instruction 與 template 的**文字內容不含 CLI 品牌名稱**（只引用 `openspec/` 路徑），因此逐字沿用。

## 2. CLI 指令對映

保留（行為與輸出一致）：`init, update, list, show, validate, analyze, drift, archive, status, instructions, new(change/artifact), schemas, templates, schema(which/validate/fork/init), config, completion, feedback, task done, in-progress add, demo`。

**移除**（依需求）：
- `search`（向量語意搜尋）— 整個命令移除。
- `park` / `unpark` — 整個命令移除；`list` 移除 `--parked`；propose 不再 park、apply 不再 unpark。
- 技能 `debug`、`ask` — 不生成、不內嵌。
- worktree — `instructions apply` 不輸出 `worktreePath`；設定無 worktree/worktrees_dir。
- `parallel_tasks` — 設定移除；tasks 不加 `[P]`；`instructions apply` 的 task 仍有 `parallel:false` 欄位以維持 JSON 結構一致（恆 false）。
- `claude_effort` — 設定移除。

**新增 / 強化**：
- `discuss` 指令群（見 §4）：`discuss new|list|show|context|add-round|conclude|archive|discard|promote`。

## 3. 設定檔 schema

`.speclink.yaml`（應用設定）保留鍵：`spec_dir`（預設 openspec）、`locale`（tw/ja/en）、`tdd`、`audit`、`tools`。移除：parallel_tasks、worktree、worktrees_dir、claude_effort、claude_slash_commands。

`openspec/config.yaml`（工作流設定）：`schema`、`context`、`rules`（per-artifact）。不變。

locale 對映：`tw`→`Traditional Chinese (繁體中文)`，`ja`→`Japanese (日本語)`，其他→`English`。

## 4. Discuss 強化（唯一與 spectra 不同的流程）

Spectra 的 discuss 是唯讀 fork、不留文件。Speclink 讓 discuss 具**延續性**：每次討論記錄成文件，迭代累積，並可作為 propose 來源。

**文件位置**：`openspec/discussions/<topic-slug>/discussion.md`（納入版控，屬專案歷史）。

**文件格式**：
```markdown
---
topic: <人類可讀主題>
slug: <kebab-case>
status: open | concluded
created: YYYY-MM-DD
---

# Discussion: <主題>

## Round 1 — <mode: assumptions|interview> (YYYY-MM-DD)
<該輪的關鍵點、假設/提問、證據檔案、開放問題>

## Round 2 — ...
...

## Conclusion
- **Decision**: ...
- **Rationale**: ...
- **Capture to**: proposal | design | spec | tasks | LANGUAGE.md
- **Next**: /speclink-propose --from-discussion <slug>
```

**CLI 支援**（引擎提供確定性格式，內容由技能透過 stdin 傳入）：
- `speclink discuss new <topic> [--json]` — 建立 discussion 骨架，回傳 slug 與路徑。
- `speclink discuss list [--json]` — 列出討論（slug、status、round 數、主題）。
- `speclink discuss show <slug> [--json]` — 顯示內容。
- `speclink discuss add-round <slug> [--mode M] [--stdin] [--json]` — 追加一輪（內容自 stdin）。
- `speclink discuss conclude <slug> [--stdin] [--json]` — 追加 `## Conclusion` 並標記 status=concluded。
- `speclink discuss discard <slug> [--force] [--json]` — 刪除 live 討論（放棄的出口）；已有 rounds 時拒絕，須 `--force`。

**discuss 技能**：與 spectra discuss 相同的步驟邏輯（Step 0 讀 LANGUAGE.md、關鍵字掃原始碼、選 Assumptions/Interview 模式、介面深度檢查、收斂、捕捉結論），**但每一輪都以 `discuss add-round` 持久化到文件**，開場時 `discuss new` 或沿用既有討論，收斂時 `discuss conclude`。

**propose 技能**：需求來源新增第三個選項 —「從 discuss 文件」。優先序：argument > plan 檔 > **discuss 文件** > 對話脈絡。當使用者指定 `--from-discussion <slug>` 或存在 concluded 討論時，讀取 discussion.md 的 Conclusion + rounds 作為提案種子。

## 5. 內部儲存（取代 SQLite）

Spectra 用 `.spectra/spectra.db`（SQLite）。Speclink 因移除 park/unpark，僅需：
- `.speclink/in_progress.json` — in-progress 標記（`{"changes":[...]}`）。
- `.speclink/touched/<change>.json` — task→動過檔案（格式同 spectra）。
- `.speclink/snapshots/<archived>/created_specs.json` — 歸檔快照。

CLI 輸出與 spectra 對齊；內部以 JSON 檔取代 SQLite。

## 6. Schema（spec-driven，內嵌）

artifacts 順序：`proposal, specs, design, tasks`；apply requires `[tasks]`。

| id | outputPath | template | requires | description |
|---|---|---|---|---|
| proposal | `proposal.md` | proposal.md | [] | Initial proposal document outlining the change |
| specs | `specs/**/*.md` | spec.md | [proposal] | Detailed specifications for the change |
| design | `design.md` | design.md | [proposal] | Technical design document with implementation details |
| tasks | `tasks.md` | tasks.md | [specs] | Implementation checklist with trackable tasks |

## 7. 技能清單（speclink）

Claude（`.claude/skills/speclink-*`）：`analyze, apply, archive, audit, commit, discuss, drift, ingest, propose, verify`（10；移除 spectra 的 debug、ask）。
Codex（`.agents/skills/speclink-*`）：`apply, archive, audit, commit, discuss, drift, ingest, propose`（8；再移除 analyze、verify，如 spectra 規則）。
內部技能（`instructions --skill`）：`sync, clarify, tdd, audit`（供其他技能取用；移除 debug/ask 內部技能）。

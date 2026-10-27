## Why

用 GitHub Copilot 跑規格驅動開發的開發者、PO、PM，現在沒有可選的 Copilot 技能。Copilot 的所有產品（雲端 agent、code review、Copilot CLI、Copilot app、VS Code、JetBrains、Visual Studio）都會讀專案的 `.github/skills/`、`.agents/skills/`、`.claude/skills/`，所以它今天已經讀得到 speclink 的技能，但讀到的版本是錯的：

- codex 版的技能互相引用時寫 `$speclink-apply`，這是 Codex 的叫用語法；Copilot 用 `/speclink-apply`。
- claude 版帶 Claude 專用的 frontmatter（`agent: Explore`、`disallowedTools`）與「Claude fork context」前言，而且建立變更、蓋章時把 agent 記成 `claude`。
- Copilot CLI 的官方優先序是 `.github/skills` → `.agents/skills` → `.claude/skills`，所以同時勾 claude＋codex 的專案，Copilot CLI 會先吃到 codex 版。

2026-07-02 把工具矩陣收回 claude＋codex（commit 9d67e7d1），當時每多一個工具就多一種指令檔格式。現在 Copilot 讀同一份 SKILL.md（Agent Skills 開放標準），多一個工具只多一個目錄與一個叫用前綴，代價低到值得收回該決定。

這個變更作用在 init（建立工作區）與 update（技能同步）兩個階段，影響所有在 init 或 desktop 選工具的使用者。

## What Changes

- 新增第三個內建 Agent 工具 `copilot`：
  - 技能寫到 `.github/skills/speclink-*/SKILL.md`。這個目錄只有 Copilot 讀，而且在 Copilot CLI 的優先序最高，同時勾 claude 或 codex 時 Copilot 仍拿到自己的版本。
  - 內容照 codex 版產生：同一個技能子集（registry 中 claude_only 為假者，即除 analyze 以外的技能；worktree 政策關閉時不含兩顆 worktree 技能）、同一份中性 frontmatter（name、description、license、compatibility、metadata），只把叫用前綴換成 `/speclink-`，沒有 plan 目錄。
  - 不帶 Claude 專用的 `context`、`agent`、`disallowedTools` 與 fork 前言；也不加 Copilot 的 `context: fork`、`allowed-tools`、`argument-hint`。
  - 技能內文的 `--agent {{TOOL}}` 自動變成 `--agent copilot`。`--agent` 本來就是自由字串，引擎的變更記錄不需改。
  - 不產生任何指令檔（`.github/copilot-instructions.md`、`AGENTS.md` 都不寫），也沒有遺留 marker 要剝除。
  - 工作區 footprint 偵測不推斷 copilot：大多數 GitHub repo 本來就有 `.github/`，看到它不代表使用者要 Copilot。
- CLI `speclink init`（speclink-cli）：
  - `--tools` 接受 `copilot`，可與 claude、codex 用逗號任意組合；未知名稱的錯誤訊息列出三個工具。
  - 互動終端依序問 Claude、Codex、Copilot 三題，三題都答 no 時重問。
  - 非互動終端缺 `--tools` 的單行錯誤改列三個工具名。stdin 與 exit code 規則不變。
- `.speclink.yaml` 的 `tools` 清單接受 `copilot` 字串，沒有新欄位、沒有預設值改變。
- desktop（apps/desktop）：初始化確認、啟用確認、checkout 綁定、設定頁「AI 工具」卡四處的勾選都加上 copilot；checkout 綁定的空選集錯誤訊息列出三個工具。
- Node SDK（speclink-node）：`skills.render` 的 target 接受 `'copilot'`，TypeScript 型別同步。
- 產物層版號從 v1.42.0 遞增為 v1.43.0：assets.lock 的指紋涵蓋全部 render 輸出，加入 copilot 渲染後指紋改變。
- 規格順帶修正：這次要改寫的 workspace-chooser 與 desktop-config 條文裡，仍寫「生成 `AGENTS.md`／`CLAUDE.md` 區塊」的過期子句，一併改成現況（只生成技能檔、只剝除遺留區塊）。程式行為與測試早已如此，不需改。
- 文件：README 與 getting-started、configuration、workflow、sdk-node、product-status 的中英文版補上 Copilot 的目錄與 `/speclink-*` 叫用方式。

相容性影響：

- **BREAKING**：`.speclink.yaml` 裡名為 `copilot` 的自訂描述子，或 skills_dir 正規化後等於 `.github/skills` 的描述子，升級後會因為「與內建工具衝突」被拒，update 以非零結束。遷移方式：刪掉該描述子，改在 tools 寫 `copilot`。
- 人眼輸出：`speclink init` 的互動詢問多一題；缺 `--tools` 與未知工具的錯誤訊息文字改變。`Generated files for:` 行在選了 copilot 時多列 `copilot`。
- `--json`：技能檔過期探測的逐工具資訊在選了 copilot 時多一項 `tool: "copilot"`，欄位形狀不變。
- 回歸對照：claude、codex、neutral 的 golden 只有 frontmatter 版號行從 v1.42.0 變成 v1.43.0，內容逐位元不變；新增 copilot golden。
- 既有工作區：版號遞增後，所有已安裝的技能檔都會被探測為過期，desktop 會提示更新；`speclink update` 對 claude、codex 技能只改寫版號行。

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `workspace-tools`: 內建工具集合從 claude、codex 擴為 claude、codex、copilot，並新增 Copilot 渲染目標的需求；描述子驗證保留 `copilot` 名稱與 `.github/skills` 目錄；init 工具選擇、built-in tools 權威收斂、golden 鎖定、過期探測、孤兒清理、worktree 政策過濾的列舉同步改寫。
- `node-sdk`: 渲染 API 的 target 值域加入 copilot。
- `workspace-chooser`: checkout 綁定與 remote marker 探測分流的工具勾選從 Claude／Codex 擴為三個。
- `desktop-config`: 初始化確認、啟用確認與設定頁的 AI 工具多選加入 copilot。
- `user-documentation`: getting-started 的入口說明加入 Copilot 的 `/speclink-*` 叫用語法。

## Impact

- Affected specs: workspace-tools, node-sdk, workspace-chooser, desktop-config, user-documentation
- 規格掃描結果：相關的正式規格就是上面五份；沒有任何正式規格已經涵蓋 Copilot，所以全部以修改既有需求處理，不新增 capability。
- 影響的技能：Copilot 版為非 Claude 子集（claude_only 為假）的全部技能；claude、codex 版只有版號行改變。影響的工具：新增 copilot，claude 與 codex 內容不變。
- Affected code:
  - Modified: crates/engine/speclink-core/src/workspace/skills.rs
  - Modified: crates/engine/speclink-core/src/workspace/init.rs
  - Modified: crates/engine/speclink-core/src/workspace/init/tests.rs
  - Modified: crates/engine/speclink-core/src/workspace/config.rs
  - Modified: crates/engine/speclink-core/src/workspace/config/tests.rs
  - Modified: crates/engine/speclink-core/tests/it/render_golden.rs
  - Modified: crates/engine/speclink-core/tests/golden/assets.lock
  - Modified: crates/engine/speclink-core/tests/golden/claude.snapshot.md
  - Modified: crates/engine/speclink-core/tests/golden/claude-worktree.snapshot.md
  - Modified: crates/engine/speclink-core/tests/golden/codex.snapshot.md
  - Modified: crates/engine/speclink-core/tests/golden/neutral-cli.snapshot.md
  - Modified: crates/engine/speclink-core/tests/golden/neutral-tool-call.snapshot.md
  - New: crates/engine/speclink-core/tests/golden/copilot.snapshot.md
  - Modified: crates/adapters/speclink-cli/src/verbs/init.rs
  - Modified: crates/adapters/speclink-cli/tests/it/init_tools.rs
  - Modified: crates/adapters/speclink-node/src/render.rs
  - Modified: crates/adapters/speclink-node/index.d.ts
  - Modified: crates/adapters/speclink-node/__test__/render.spec.ts
  - New: apps/desktop/src/builtinTools.ts
  - Modified: apps/desktop/src/App.tsx
  - Modified: apps/desktop/src/components/WorkspaceChooser.tsx
  - Modified: apps/desktop/src/views/ProjectSettingsView.tsx
  - Modified: apps/desktop/src/adapter/connections.ts
  - Modified: apps/desktop/src-tauri/src/connections.rs
  - Modified: apps/desktop/core/src/settings.rs
  - Modified: apps/desktop/core/src/project.rs
  - Modified: apps/desktop/src/__tests__/App.test.tsx
  - Modified: apps/desktop/src/__tests__/workspaceChooser.test.tsx
  - Modified: apps/desktop/src/__tests__/projectSettingsView.test.tsx
  - Modified: README.md
  - Modified: README.en.md
  - Modified: docs/getting-started.zh-TW.md
  - Modified: docs/getting-started.md
  - Modified: docs/configuration.zh-TW.md
  - Modified: docs/configuration.md
  - Modified: docs/workflow.zh-TW.md
  - Modified: docs/workflow.md
  - Modified: docs/sdk-node.zh-TW.md
  - Modified: docs/sdk-node.md
  - Modified: docs/product-status.zh-TW.md
  - Modified: docs/product-status.md
  - Modified: .claude/skills/（版號遞增後 speclink update 再生的技能檔，只有版號行改變）
  - Modified: .agents/skills/（同上）

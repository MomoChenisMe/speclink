## Why

用 pi 或 Cursor 跑規格驅動開發的開發者（團隊情境下含 PO、PM），現在沒有可選的 pi、Cursor 技能。這兩個工具今天已經讀得到 speclink 的技能，但讀到的版本是錯的：

- pi 讀專案的 `.pi/skills/` 與 `.agents/skills/`，所以它吃到 codex 版。codex 版的技能互相引用寫 `$speclink-apply`，pi 的叫用語法是 `/skill:speclink-apply`；而且建立變更、蓋章時把 agent 記成 `codex`。repo 內的 Herdr 面板也因此把 pi 對到 codex，缺技能時提示 `speclink init --tools codex`。
- Cursor 讀 `.cursor/skills/`，也相容讀 `.agents/skills/`、`.claude/skills/`、`.codex/skills/`。它可能吃到 codex 版（`$speclink-` 前綴）或 claude 版（帶 `agent: Explore`、`disallowedTools` 與 Claude fork 前言），agent 也被記成 `codex` 或 `claude`。Cursor 的叫用語法是 `/speclink-apply`。

pi 與 Cursor 讀的是同一份 SKILL.md 格式（Agent Skills 開放標準），所以多一個工具只多一個目錄、一個叫用前綴與一個 `--agent` 值，做法與 add-copilot-tool 相同。討論 pi-cursor-tool-support 已逐條確認。

這個變更作用在 init（建立工作區）與 update（技能同步）兩個階段，影響所有在 init、`.speclink.yaml` 或 desktop 選工具的使用者。

## What Changes

- 新增兩個內建 Agent 工具，內建工具的順序固定為 claude、codex、copilot、pi、cursor：
  - `pi`：技能寫到 `.pi/skills/speclink-*/SKILL.md`。pi 的掃描順序是 `.pi/skills` → `.agents/skills` → `~/.pi/agent/skills` → `~/.agents/skills`，同名技能保留先找到的那份，所以同時勾 codex 時 pi 仍拿到 pi 版。技能引用寫 `/skill:speclink-<名稱>`，`--agent pi`。
  - `cursor`：技能寫到 `.cursor/skills/speclink-*/SKILL.md`。這個目錄只有 Cursor 讀。技能引用寫 `/speclink-<名稱>`，`--agent cursor`。
  - 兩者的內容都照 codex 版產生：同一個技能子集（除 analyze 以外的技能；worktree 政策關閉時不含兩顆 worktree 技能）、同一份中性 frontmatter（name、description、license、compatibility、metadata），沒有 plan 目錄，不帶 Claude 專用的 `context`、`agent`、`disallowedTools` 與 fork 前言。
  - 不產生任何指令檔（`AGENTS.md`、`CLAUDE.md`、`.cursor/rules/` 都不寫），也不產生 `.cursor/agents/` 子代理檔。
  - 工作區 footprint 偵測不因 `.pi/`、`.cursor/` 存在而推斷新工具：很多 repo 有 `.cursor/` 只是為了放 rules。
- CLI `speclink init`（speclink-cli）：
  - `--tools` 接受 `pi`、`cursor`，可與其他工具用逗號任意組合；未知名稱的錯誤訊息列出五個工具。
  - 互動終端依序問 Claude、Codex、Copilot、Pi、Cursor 五題，五題都答 no 時重問。
  - 非互動終端缺 `--tools` 的單行錯誤改列五個工具名；`--tools` 的 help 範例同步。旗標、stdin 與 exit code 規則不變。
- `.speclink.yaml` 的 `tools` 清單接受 `pi`、`cursor` 字串，沒有新欄位、沒有預設值改變。
- desktop（apps/desktop）：初始化確認、啟用確認、checkout 綁定、設定頁 `.speclink.yaml` 頁的「AI 工具」卡四處勾選加上 pi、cursor；checkout 綁定的空選集錯誤訊息列出五個工具。
- Node SDK（speclink-node）：`skills.render` 的 target 接受 `'pi'`、`'cursor'`，TypeScript 型別同步。
- Herdr 面板（integrations/herdr）：pi 改對到 `pi` 工具（缺技能提示改為 `--tools pi`）；新增 cursor 的叫用前綴 `/` 與技能目錄，面板可以把技能送給 Cursor。
- 產物層版號從 v1.53.0 遞增為 v1.54.0：assets.lock 的指紋涵蓋全部內建工具的渲染輸出，加入兩個工具後指紋改變。
- 規格與測試的範例改名：規格「技能檔過期探測」與 speclink-core 測試用一個名為 `cursor`、目錄為 `.cursor/skills` 的自訂描述子當範例，`tools: [claude, cursor]` 也被當成「未知工具」的範例。`cursor` 成為內建工具後這些範例不再成立，改用 `my-harness` 與 `windsurf`。
- 技能原稿守門：原稿 SHALL NOT 寫死的 `--agent` 值加上 `pi`、`cursor`。
- 文件：README 與 getting-started、configuration、workflow、sdk-node、product-status 的中英文版，以及 Herdr 面板說明，補上 pi、Cursor 的目錄與叫用方式，並寫明 pi 的同名警告與「pi 要先信任專案才讀專案技能」。

相容性影響：

- **BREAKING**：`.speclink.yaml` 裡名為 `pi` 或 `cursor` 的自訂描述子，或 skills_dir 正規化後等於 `.pi/skills`、`.cursor/skills` 的描述子，升級後會因為「與內建工具衝突」被拒，update 以非零結束。規格原本的範例就是名為 `cursor`、目錄 `.cursor/skills` 的描述子，照範例設定的專案會中這一條。遷移方式：刪掉該描述子，改在 tools 寫 `pi` 或 `cursor`。
- 人眼輸出：`speclink init` 的互動詢問由三題變五題；缺 `--tools`、未知工具、未知工具 note 的錯誤訊息文字改變。`Generated files for:` 行在選了 pi 或 cursor 時多列該工具。
- `--json`：技能檔過期探測的逐工具資訊在選了 pi 或 cursor 時多一項 `tool: "pi"` 或 `tool: "cursor"`，欄位形狀不變。
- 回歸對照：既有六份 golden（claude、claude-worktree、codex、copilot、neutral-cli、neutral-tool-call）只有 frontmatter 版號行從 v1.53.0 變成 v1.54.0，內容逐位元不變；新增 pi、cursor 兩份 golden。
- 既有工作區：版號遞增後，所有已安裝的技能檔都會被探測為過期，desktop 會提示更新；`speclink update` 對既有工具的技能只改寫版號行。
- 舊版引擎讀到含 `pi` 或 `cursor` 的 `.speclink.yaml`：沿既有行為記一行 unknown tool note 並略過，不刪 `.pi/skills/`、`.cursor/skills/`。
- 同時勾 codex 與 pi：pi 每次啟動會對 18 份同名技能各顯示一則警告，pi 用的是 pi 版，功能不受影響。

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `workspace-tools`: 內建工具集合擴為 claude、codex、copilot、pi、cursor，新增 Pi 與 Cursor 渲染目標的需求，Copilot 渲染目標的差異表擴成五列；描述子驗證保留 `pi`、`cursor` 名稱與 `.pi/skills`、`.cursor/skills` 目錄；init 工具選擇、built-in tools 權威收斂、中性渲染目標的 golden、過期探測、孤兒清理、worktree 政策過濾的列舉同步改寫；過期探測的描述子範例改名為 `my-harness`。
- `node-sdk`: 渲染 API 的 target 值域加入 pi、cursor。
- `workspace-chooser`: checkout 綁定與 remote marker 探測分流的工具勾選擴為五個。
- `desktop-config`: 初始化確認、啟用確認與設定頁的 AI 工具多選加入 pi、cursor。
- `user-documentation`: getting-started 的入口說明加入 pi 的 `/skill:speclink-*` 與 Cursor 的 `/speclink-*` 叫用語法。
- `skill-authoring`: 技能引用依工具渲染的範例加入 pi、cursor；原稿守門禁止寫死的 `--agent` 值加入 `pi`、`cursor`；對話語言段落的適用範圍改為全部內建工具。

## Impact

- Affected specs: workspace-tools, node-sdk, workspace-chooser, desktop-config, user-documentation, skill-authoring
- 規格掃描結果：相關的正式規格是上面六份；沒有任何正式規格已經涵蓋 pi 或 Cursor，所以全部以修改既有需求處理，不新增 capability。review-skill「首輪工單記錄套用的規則」的情境寫「三種工具（claude、codex、copilot）生成」，對這三個工具仍然成立，本變更不改寫。
- 影響的技能：pi 版與 Cursor 版為非 Claude 子集的全部技能（18 份）；claude、codex、copilot 版只有版號行改變。影響的工具：新增 pi、cursor，claude、codex、copilot 的內容不變。
- 影響的 crate 與 app：speclink-core、speclink-cli、speclink-node、apps/desktop（前端、desktop-core、Tauri 殼）、integrations/herdr。
- Affected code:
  - Modified: crates/engine/speclink-core/src/workspace/skills.rs
  - Modified: crates/engine/speclink-core/src/workspace/init.rs
  - Modified: crates/engine/speclink-core/src/workspace/init/tests.rs
  - Modified: crates/engine/speclink-core/src/workspace/config/tests.rs
  - Modified: crates/engine/speclink-core/tests/it/render_golden.rs
  - Modified: crates/engine/speclink-core/tests/it/skill_authoring.rs
  - Modified: crates/engine/speclink-core/tests/golden/assets.lock
  - Modified: crates/engine/speclink-core/tests/golden/claude.snapshot.md
  - Modified: crates/engine/speclink-core/tests/golden/claude-worktree.snapshot.md
  - Modified: crates/engine/speclink-core/tests/golden/codex.snapshot.md
  - Modified: crates/engine/speclink-core/tests/golden/copilot.snapshot.md
  - Modified: crates/engine/speclink-core/tests/golden/neutral-cli.snapshot.md
  - Modified: crates/engine/speclink-core/tests/golden/neutral-tool-call.snapshot.md
  - New: crates/engine/speclink-core/tests/golden/pi.snapshot.md
  - New: crates/engine/speclink-core/tests/golden/cursor.snapshot.md
  - Modified: crates/adapters/speclink-cli/src/verbs/init.rs
  - Modified: crates/adapters/speclink-cli/tests/it/init_tools.rs
  - Modified: crates/adapters/speclink-node/src/render.rs
  - Modified: crates/adapters/speclink-node/index.d.ts
  - Modified: crates/adapters/speclink-node/__test__/render.spec.ts
  - Modified: apps/desktop/src/builtinTools.ts
  - Modified: apps/desktop/src/adapter/connections.ts
  - Modified: apps/desktop/src-tauri/src/connections.rs
  - Modified: apps/desktop/core/src/settings.rs
  - Modified: apps/desktop/core/src/project.rs
  - Modified: apps/desktop/src/__tests__/App.test.tsx
  - Modified: apps/desktop/src/__tests__/workspaceChooser.test.tsx
  - Modified: apps/desktop/src/__tests__/projectSettingsView.test.tsx
  - Modified: integrations/herdr/mods/speclink/model.js
  - Modified: scripts/herdr/panel-model.test.mjs
  - Modified: integrations/herdr/README.md
  - Modified: integrations/herdr/README.en.md
  - Modified: README.md
  - Modified: README.zh-TW.md
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

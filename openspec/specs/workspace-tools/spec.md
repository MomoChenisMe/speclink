# workspace-tools Specification

## Purpose

TBD - created by archiving change 'config-system-rework'. Update Purpose after archive.

## Requirements

### Requirement: tools 自訂描述子的接受與驗證
.speclink.yaml 的 tools 清單 SHALL 接受兩種元素形式：內建工具名字串（claude、codex），或自訂描述子物件（欄位：name 必填、skills_dir 必填、instructions_file 必填、invocation 選填且值域為 cli 或 tool-call、預設 cli）。描述子驗證規則：name SHALL 為 kebab-case（2 至 50 字）且 SHALL NOT 與內建工具名衝突；skills_dir 與 instructions_file SHALL 為專案根相對路徑，正規化後 SHALL NOT 逸出專案根。驗證失敗時指令 SHALL 以非 0 exit code 結束並輸出單行語義化錯誤訊息（指明錯誤欄位與原因）。

#### Scenario: 合法描述子生成對應工具檔
- **WHEN** .speclink.yaml 的 tools 含描述子 name: wad-harness、skills_dir: .wad/skills、instructions_file: WAD.md，執行 speclink update
- **THEN** 生成 .wad/skills/speclink-*/SKILL.md 技能檔與 WAD.md 內的 SPECLINK marker 區塊，exit code 為 0

#### Scenario: 名稱與內建工具衝突被拒
- **WHEN** tools 含描述子 name: claude，執行 speclink update
- **THEN** exit code 非 0，stderr 單行錯誤訊息指明 name 與內建工具名衝突

#### Scenario: 路徑逸出專案根被拒
- **WHEN** tools 含描述子 skills_dir: ../outside/skills，執行 speclink update
- **THEN** exit code 非 0，stderr 單行錯誤訊息指明 skills_dir 逸出專案根


<!-- @trace
source: config-system-rework
updated: 2026-07-04
code:
  - AGENTS.md
  - CLAUDE.md
  - README.md
  - crates/speclink-cli/src/commands.rs
  - crates/speclink-cli/tests/deprecation_warning.rs
  - crates/speclink-cli/tests/instructions_policy.rs
  - crates/speclink-cli/tests/tools_descriptor.rs
  - crates/speclink-core/assets/skills/archive.md
  - crates/speclink-core/assets/skills/commit.md
  - crates/speclink-core/assets/skills/discuss.md
  - crates/speclink-core/src/config.rs
  - crates/speclink-core/src/init.rs
  - crates/speclink-core/src/instructions.rs
  - crates/speclink-core/src/skills.rs
  - crates/speclink-core/tests/golden/claude.snapshot.md
  - crates/speclink-core/tests/golden/codex.snapshot.md
  - crates/speclink-core/tests/golden/neutral-cli.snapshot.md
  - crates/speclink-core/tests/golden/neutral-tool-call.snapshot.md
  - crates/speclink-core/tests/render_golden.rs
  - docs/configuration.md
  - docs/configuration.zh-TW.md
-->

---
### Requirement: 描述子的同步與清理生命週期
speclink update SHALL 對描述子與內建工具一視同仁：在 tools 清單上的描述子重新生成其技能與 marker 區塊；自清單移除的描述子，其生成物 SHALL 被清理——skills_dir 下的 speclink- 前綴技能目錄移除（因而變空的目錄一併移除）、instructions_file 的 SPECLINK marker 區塊剝除（使用者自有內容保留，剝除後全空的檔案刪除）。

#### Scenario: 移除描述子後生成物被清理
- **WHEN** 先以含 wad-harness 描述子的 tools 執行 speclink update，再將該描述子自 tools 移除並重新執行 speclink update
- **THEN** .wad/skills/ 下的 speclink- 前綴目錄被移除，WAD.md 的 marker 區塊被剝除；若 WAD.md 除區塊外無其他內容則整檔刪除


<!-- @trace
source: config-system-rework
updated: 2026-07-04
code:
  - AGENTS.md
  - CLAUDE.md
  - README.md
  - crates/speclink-cli/src/commands.rs
  - crates/speclink-cli/tests/deprecation_warning.rs
  - crates/speclink-cli/tests/instructions_policy.rs
  - crates/speclink-cli/tests/tools_descriptor.rs
  - crates/speclink-core/assets/skills/archive.md
  - crates/speclink-core/assets/skills/commit.md
  - crates/speclink-core/assets/skills/discuss.md
  - crates/speclink-core/src/config.rs
  - crates/speclink-core/src/init.rs
  - crates/speclink-core/src/instructions.rs
  - crates/speclink-core/src/skills.rs
  - crates/speclink-core/tests/golden/claude.snapshot.md
  - crates/speclink-core/tests/golden/codex.snapshot.md
  - crates/speclink-core/tests/golden/neutral-cli.snapshot.md
  - crates/speclink-core/tests/golden/neutral-tool-call.snapshot.md
  - crates/speclink-core/tests/render_golden.rs
  - docs/configuration.md
  - docs/configuration.zh-TW.md
-->

---
### Requirement: 中性渲染目標

描述子生成的技能與指令區塊 SHALL 使用中性渲染：內文 SHALL NOT 含 /speclink- slash 前綴與 plan mode 參照；speclink 動詞的措辭依 invocation 決定——cli 為「執行 speclink <動詞>」形式，tool-call 為「呼叫 speclink 工具（參數為 argv 陣列）」形式。內建 claude 與 codex 的生成內容 SHALL 與 render golden 基線位元級一致；基線 SHALL 僅隨提案記載的刻意變更同批更新。

#### Scenario: tool-call 措辭

- **WHEN** 描述子 invocation 為 tool-call，執行 speclink update 後檢視生成的技能檔
- **THEN** 內文以「呼叫 speclink 工具」措辭引用動詞，且不含 /speclink- 前綴與 plan mode 字樣

#### Scenario: 內建工具輸出鎖定於 golden 基線

- **WHEN** tools 僅含 claude 與 codex，執行 speclink update
- **THEN** 生成的 CLAUDE.md、AGENTS.md marker 區塊與 .claude/skills/、.agents/skills/ 技能內容與 render golden 基線完全一致


<!-- @trace
source: desktop-instruction-staleness-prompt
updated: 2026-07-31
code:
  - .agents/skills/speclink-apply/SKILL.md
  - .agents/skills/speclink-archive/SKILL.md
  - .agents/skills/speclink-audit/SKILL.md
  - .agents/skills/speclink-commit/SKILL.md
  - .agents/skills/speclink-config/SKILL.md
  - .agents/skills/speclink-discuss/SKILL.md
  - .agents/skills/speclink-drift/SKILL.md
  - .agents/skills/speclink-ingest/SKILL.md
  - .agents/skills/speclink-onboard/SKILL.md
  - .agents/skills/speclink-propose/SKILL.md
  - .claude/skills/speclink-analyze/SKILL.md
  - .claude/skills/speclink-apply/SKILL.md
  - .claude/skills/speclink-archive/SKILL.md
  - .claude/skills/speclink-audit/SKILL.md
  - .claude/skills/speclink-commit/SKILL.md
  - .claude/skills/speclink-config/SKILL.md
  - .claude/skills/speclink-discuss/SKILL.md
  - .claude/skills/speclink-drift/SKILL.md
  - .claude/skills/speclink-ingest/SKILL.md
  - .claude/skills/speclink-onboard/SKILL.md
  - .claude/skills/speclink-propose/SKILL.md
  - .claude/skills/speclink-verify/SKILL.md
  - AGENTS.md
  - CLAUDE.md
  - apps/desktop/core/src/manage.rs
  - apps/desktop/core/src/project.rs
  - apps/desktop/src-tauri/src/lib.rs
  - apps/desktop/src-tauri/tests/remote_data.rs
  - apps/desktop/src/App.tsx
  - apps/desktop/src/__tests__/App.test.tsx
  - apps/desktop/src/__tests__/helpers/remoteFixtures.ts
  - apps/desktop/src/__tests__/instructionUpdatePrompt.test.tsx
  - apps/desktop/src/__tests__/remoteDataSource.test.ts
  - apps/desktop/src/__tests__/remoteResilience.test.tsx
  - apps/desktop/src/__tests__/store.test.ts
  - apps/desktop/src/adapter/remoteDataSource.ts
  - apps/desktop/src/adapter/workspace.ts
  - apps/desktop/src/components/InstructionUpdatePrompt.tsx
  - apps/desktop/src/i18n/messages.ts
  - apps/desktop/src/instructionPrompt.ts
  - apps/desktop/src/store.ts
  - crates/speclink-cli/tests/archive_readiness_gate.rs
  - crates/speclink-core/src/archive.rs
  - crates/speclink-core/src/command/mod.rs
  - crates/speclink-core/src/init.rs
  - crates/speclink-core/src/skills.rs
  - crates/speclink-core/tests/golden/assets.lock
  - crates/speclink-core/tests/golden/claude.snapshot.md
  - crates/speclink-core/tests/golden/codex.snapshot.md
  - crates/speclink-core/tests/golden/neutral-cli.snapshot.md
  - crates/speclink-core/tests/golden/neutral-tool-call.snapshot.md
  - crates/speclink-core/tests/golden/remote-claude.marker.md
  - crates/speclink-core/tests/render_golden.rs
  - packages/ui/src/__tests__/kanban.test.tsx
  - packages/ui/src/__tests__/richDrawer.test.tsx
  - packages/ui/src/boardDnd.ts
  - packages/ui/src/components/KanbanBoard.tsx
  - packages/ui/src/components/RichDetailDrawer.tsx
  - packages/ui/src/i18n.tsx
-->

---
### Requirement: init 內建 Agent 工具選擇

所有 `speclink init`（filesystem 與 Remote Store）SHALL 在任何 Workspace 寫入前解析至少一個內建 Agent 工具。顯式 `--tools` SHALL 接受 `claude`、`codex` 或逗號分隔的兩者並跳過詢問；解析後為空或含未知名稱 SHALL 以非零 exit code 失敗。未提供 `--tools` 且 stdin 為互動終端時，CLI SHALL 於 stderr 明示詢問 Claude 與 Codex 並允許選一個或兩個；兩者皆未選 SHALL NOT 開始 init，並 SHALL 繼續要求有效選擇。未提供 `--tools` 且 stdin 非互動終端時，CLI SHALL 在零寫入狀態以非零 exit code 失敗，stderr SHALL 指出 `--tools` 與三種有效選法，stdout SHALL 為空。init SHALL NOT 將 redirected／piped stdin 當作工具答案，SHALL NOT 新增 stdin payload 或 `--json` 介面。此行為是對早期 footprint 自動偵測的刻意分歧；顯式提供 `--tools` 時既有人眼成功輸出與 `--no-color` 行為 SHALL 維持既有基線。

#### Scenario: filesystem init 顯式選擇 Codex

- **WHEN** 在空目錄執行 filesystem init 並顯式提供 `--tools codex`
- **THEN** exit code 為 0，stdout 沿用既有 Initialized 與 Generated files 摘要，`.speclink.yaml` 的 built-in tools 僅含 `codex`，並生成 Codex Skills 與 `AGENTS.md` Speclink 區塊

#### Scenario: Remote init 顯式選擇兩個工具

- **WHEN** 在空目錄執行 Remote Store init 並顯式提供 `--tools claude,codex`、有效 project URL 與 repo
- **THEN** exit code 為 0，`.speclink.yaml` 同時含兩個 built-in tools 與 remote section，生成兩組 Skills 及兩份指令區塊，且不存在 `openspec/`

#### Scenario: 互動終端選擇 Claude 與 Codex

- **WHEN** 未提供 `--tools` 且 stdin 為互動終端，使用者對 Claude 與 Codex 都回答 yes
- **THEN** 詢問文字只寫入 stderr，init 以兩個工具執行，成功摘要寫入 stdout

#### Scenario: 互動終端不得提交空選集

- **WHEN** 未提供 `--tools` 且 stdin 為互動終端，使用者對 Claude 與 Codex 都回答 no
- **THEN** CLI 不建立 `.speclink.yaml`、`openspec/`、`.gitignore`、Skills 或指令檔，並再次要求至少選取一個工具

#### Scenario: 非互動 init 缺少 tools 零寫入失敗

- **WHEN** stdin 為 pipe 或 redirect，執行 init 且未提供 `--tools`
- **THEN** exit code 非 0、stdout 為空，stderr 單行訊息包含 `--tools`、`claude` 與 `codex`，且目標目錄內容逐項不變

#### Scenario: 空或未知的顯式 tools 被拒

- **WHEN** 執行 init 並提供空的 `--tools` 值或含 `vscode` 的值
- **THEN** exit code 非 0，stderr 指出選集為空或 unknown tool，且任何 Workspace 檔案都未建立或修改

#### Scenario: no-color 不改變工具選擇語意

- **WHEN** 以 `--no-color` 執行互動式 init 並選取任一有效工具
- **THEN** prompt 與成功輸出不含 ANSI escape sequence，exit code 與檔案效果和有色模式相同


<!-- @trace
source: spectra-legacy-cleanup
updated: 2026-07-27
code:
  - README.en.md
  - README.md
  - apps/desktop/src/App.tsx
  - apps/desktop/src/components/ProjectTabs.tsx
  - apps/desktop/src/index.css
  - crates/speclink-cli/src/color.rs
  - crates/speclink-cli/src/commands.rs
  - crates/speclink-cli/src/main.rs
  - crates/speclink-cli/tests/discuss_promote_snapshot.rs
  - crates/speclink-cli/tests/task_done_stamps.rs
  - crates/speclink-core/assets/skills/archive.md
  - crates/speclink-core/src/analyzer.rs
  - crates/speclink-core/src/archive.rs
  - crates/speclink-core/src/command/mod.rs
  - crates/speclink-core/src/config.rs
  - crates/speclink-core/src/demo.rs
  - crates/speclink-core/src/discuss.rs
  - crates/speclink-core/src/drift.rs
  - crates/speclink-core/src/init.rs
  - crates/speclink-core/src/instructions.rs
  - crates/speclink-core/src/lib.rs
  - crates/speclink-core/src/listing.rs
  - crates/speclink-core/src/model.rs
  - crates/speclink-core/src/newcmd.rs
  - crates/speclink-core/src/preflight.rs
  - crates/speclink-core/src/schema.rs
  - crates/speclink-core/src/skills.rs
  - crates/speclink-core/src/status.rs
  - crates/speclink-core/src/tasks.rs
  - crates/speclink-core/src/validate.rs
  - crates/speclink-core/tests/golden/claude.snapshot.md
  - crates/speclink-core/tests/golden/codex.snapshot.md
  - crates/speclink-core/tests/golden/neutral-cli.snapshot.md
  - crates/speclink-core/tests/golden/neutral-tool-call.snapshot.md
  - crates/speclink-host/src/context.rs
  - docs/platform-architecture.zh-TW.md
  - packages/ui/src/__tests__/delta.test.ts
  - packages/ui/src/__tests__/taskList.test.tsx
  - packages/ui/src/components/ChangeList.tsx
  - packages/ui/src/components/DeltaBadges.tsx
  - packages/ui/src/components/RichDetailDrawer.tsx
  - packages/ui/src/delta.ts
  - packages/ui/src/index.ts
  - packages/ui/src/theme.css
-->

---
### Requirement: built-in tools 權威收斂

Workspace 工具同步 SHALL 將請求中的 Claude／Codex 集合視為 built-in tools 的完整期望狀態。被選取的 built-in SHALL 生成或更新其 `speclink-*` Skills 與對應 `SPECLINK:START..END` 指令區塊；未選取的 built-in SHALL 移除 Speclink 產生的 Skills 與指令區塊，SHALL 保留區塊外的使用者內容，且指令檔在清理後全空時才刪除。同步 SHALL 保留 `.speclink.yaml` 內的 custom descriptor、unknown tool entry、remote、spec_dir 與其他頂層鍵。相同期望狀態重試 SHALL 收斂到相同檔案結果，不重複 marker 或破壞使用者內容。

#### Scenario: Claude 切換為 Codex並保留自訂工具

- **WHEN** `.speclink.yaml` 含 `claude`、一個 custom descriptor、remote section 與未知頂層鍵，`CLAUDE.md` 同時含 Speclink 區塊和使用者文字，然後同步 built-in 選集 `[codex]`
- **THEN** `tools` 保留 custom descriptor 並將 built-in 集合改為僅 `codex`，remote 與未知鍵值不變，Codex Skills／`AGENTS.md` 被補齊，Claude Skills／Speclink 區塊被移除，且 `CLAUDE.md` 的使用者文字仍存在

##### Example: built-in 選集轉換

| 原 built-in | 新選集 | 保留 custom descriptor | 受管結果 |
| --- | --- | --- | --- |
| claude | codex | 是 | 移除 Claude、補齊 Codex |
| claude,codex | claude | 是 | 更新 Claude、移除 Codex |
| codex | claude,codex | 是 | 補齊兩者且不重複 marker |

#### Scenario: 既有選集缺少產物時自動補齊

- **WHEN** `.speclink.yaml` 的 built-in tools 為 `[codex]`，但 `AGENTS.md` Speclink 區塊或任一 Codex Skill 缺少，然後再次同步 `[codex]`
- **THEN** 缺少或過期的受管產物被補齊至正典內容，其他使用者檔案維持不變，且同步成功

#### Scenario: 壞設定在寫入前被拒

- **WHEN** `.speclink.yaml` 無法解析，然後請求同步任一 built-in 選集
- **THEN** 同步以單行解析錯誤失敗，原設定、Skills 與指令檔逐字元不變

<!-- @trace
source: unify-agent-tool-bootstrap
updated: 2026-07-24
code:
  - apps/desktop/core/src/project.rs
  - apps/desktop/core/src/settings.rs
  - apps/desktop/src-tauri/src/connections.rs
  - apps/desktop/src-tauri/src/lib.rs
  - apps/desktop/src/App.tsx
  - apps/desktop/src/__tests__/remoteOpen.test.ts
  - apps/desktop/src/__tests__/remoteResilience.test.tsx
  - apps/desktop/src/__tests__/serversPanel.test.tsx
  - apps/desktop/src/__tests__/workspaceChooser.test.tsx
  - apps/desktop/src/adapter/connections.ts
  - apps/desktop/src/components/WorkspaceChooser.tsx
  - apps/desktop/src/i18n/messages.ts
  - apps/desktop/src/store.ts
  - crates/speclink-cli/src/commands.rs
  - crates/speclink-cli/src/main.rs
  - crates/speclink-cli/src/remote_commands.rs
  - crates/speclink-cli/tests/init_tools.rs
  - crates/speclink-cli/tests/remote_connect.rs
  - crates/speclink-cli/tests/remote_section.rs
  - crates/speclink-core/src/config.rs
  - crates/speclink-core/src/init.rs
-->

---
### Requirement: 工作區補齊入口

引擎 SHALL 提供冪等的工作區補齊（adopt）入口：對「已有 openspec/ 但無 `.speclink.yaml`」的目錄，補 openspec/ 骨架缺件（specs/ 與 changes/archive/ 目錄；config.yaml 僅在不存在時寫入範本）、於專案根寫入 `.speclink.yaml` 記錄所選 tools、為每個所選工具生成技能檔與指令檔受管區塊，並確保專案根 `.gitignore` 涵蓋 `.speclink/` 工作資料夾。既有 openspec/ 文件（specs、changes、discussions 及既有 config.yaml）SHALL 零觸碰。入口 SHALL NOT 受 init 的「已初始化即擋下」限制；tools 空清單 SHALL 回錯誤。重複執行 SHALL 冪等收斂於相同結果。

#### Scenario: 補齊工作區檔且既有內容零觸碰

- **WHEN** 對含 openspec/（內有規格文件與自訂 config.yaml）但無 .speclink.yaml 的目錄以 tools=[claude] 執行 adopt
- **THEN** 專案根產生 .speclink.yaml（tools 含 claude）、CLAUDE.md 受管區塊與 .claude/skills/ 技能檔，openspec/ 內既有文件與 config.yaml 位元級不變

#### Scenario: 骨架缺件補齊

- **WHEN** 對 openspec/ 內缺 specs/ 目錄與 config.yaml 的未啟用目錄執行 adopt
- **THEN** specs/、changes/archive/ 目錄建立，config.yaml 以範本寫入

#### Scenario: 工作資料夾納入版控忽略

- **WHEN** 對 .gitignore 不存在、或存在但未涵蓋 `.speclink/` 的未啟用目錄執行 adopt
- **THEN** 專案根 .gitignore 涵蓋 `.speclink/`，且該檔既有內容原樣保留

##### Example: 既有 .gitignore 追加而非覆寫

- **GIVEN** 專案根 .gitignore 內容為 `node_modules/\ndist/\n`
- **WHEN** 以 tools=[claude] 執行 adopt
- **THEN** .gitignore 仍含 `node_modules/` 與 `dist/` 兩行，並多出 `.speclink/` 條目

#### Scenario: 重複執行冪等

- **WHEN** 對同一目錄以相同 tools 連續執行 adopt 兩次
- **THEN** 第二次成功且所有生成檔內容與第一次相同

#### Scenario: tools 空清單拒絕

- **WHEN** 以空的 tools 清單執行 adopt
- **THEN** 回單行錯誤，目錄零寫入

<!-- @trace
source: desktop-enable-speclink-prompt
updated: 2026-07-31
code:
  - apps/desktop/core/src/project.rs
  - apps/desktop/src-tauri/src/lib.rs
  - apps/desktop/src/App.tsx
  - apps/desktop/src/__tests__/App.test.tsx
  - apps/desktop/src/__tests__/workspace.test.ts
  - apps/desktop/src/adapter/workspace.ts
  - apps/desktop/src/i18n/messages.ts
  - apps/desktop/src/store.ts
  - crates/speclink-core/src/init.rs
-->

---
### Requirement: 工具檔生成不寫入 AI 工具的使用者設定檔

工具檔生成（init、update、tools 收斂、工作區補齊 adopt 的所有路徑）SHALL NOT 建立或改寫 AI 工具的使用者設定檔（`.claude/settings.json`）。受管生成物 SHALL 僅限技能檔、指令檔的 SPECLINK marker 區塊、`.speclink.yaml` 與 `.gitignore` 的 `.speclink/` 條目。既有的使用者設定檔 SHALL 視為使用者資料：任何工具同步後 SHALL 位元級不變，清理（prune）SHALL NOT 移除它。

#### Scenario: init 不產生使用者設定檔

- **WHEN** 對全新目錄以 tools=[claude] 執行 init
- **THEN** `.claude/skills/` 技能檔與 CLAUDE.md marker 區塊照常生成，`.claude/settings.json` 不存在

#### Scenario: 既有使用者設定檔在工具同步後位元級不變

- **WHEN** 專案的 `.claude/settings.json` 含使用者自訂內容（如外掛啟用清單），執行 speclink update
- **THEN** 該檔位元級不變，其餘受管檔照常再生

##### Example: 自訂外掛設定不被清空

- **GIVEN** `.claude/settings.json` 內容為 `{"enabledPlugins":{"frontend-design":true},"includeGitInstructions":false}`
- **WHEN** 執行 speclink update
- **THEN** 該檔內容仍為 `{"enabledPlugins":{"frontend-design":true},"includeGitInstructions":false}`，位元級相同

#### Scenario: 工作區補齊不產生使用者設定檔

- **WHEN** 對「已有 openspec/ 但無 .speclink.yaml」的目錄以 tools=[claude] 執行工作區補齊（adopt）
- **THEN** 工作區檔照常補齊（.speclink.yaml、技能檔、marker 區塊、.gitignore 條目），`.claude/settings.json` 不存在

<!-- @trace
source: remove-claude-settings-write
updated: 2026-07-31
code:
  - crates/speclink-cli/tests/remote_section.rs
  - crates/speclink-core/src/init.rs
-->

---
### Requirement: 產物層版本戳同源

生成的技能檔 frontmatter 版本欄位 SHALL 與指令檔 SPECLINK 標記的版號同值同源（單一產物層版號）。該版號 SHALL 僅於內嵌資產的 render 內容變動時遞增，SHALL NOT 隨 app 或 CLI 發版自動變動。

#### Scenario: 生成物的版號一致

- **WHEN** tools 含 claude 與 codex，執行 speclink init 或 speclink update 後檢視 CLAUDE.md 與 AGENTS.md 的標記版號、及 .claude/skills/ 與 .agents/skills/ 下任一技能檔的 frontmatter 版本欄位
- **THEN** 四處為相同的版號字串，無任何技能檔殘留固定值 "1.0"

<!-- @trace
source: desktop-instruction-staleness-prompt
updated: 2026-07-31
code:
  - .agents/skills/speclink-apply/SKILL.md
  - .agents/skills/speclink-archive/SKILL.md
  - .agents/skills/speclink-audit/SKILL.md
  - .agents/skills/speclink-commit/SKILL.md
  - .agents/skills/speclink-config/SKILL.md
  - .agents/skills/speclink-discuss/SKILL.md
  - .agents/skills/speclink-drift/SKILL.md
  - .agents/skills/speclink-ingest/SKILL.md
  - .agents/skills/speclink-onboard/SKILL.md
  - .agents/skills/speclink-propose/SKILL.md
  - .claude/skills/speclink-analyze/SKILL.md
  - .claude/skills/speclink-apply/SKILL.md
  - .claude/skills/speclink-archive/SKILL.md
  - .claude/skills/speclink-audit/SKILL.md
  - .claude/skills/speclink-commit/SKILL.md
  - .claude/skills/speclink-config/SKILL.md
  - .claude/skills/speclink-discuss/SKILL.md
  - .claude/skills/speclink-drift/SKILL.md
  - .claude/skills/speclink-ingest/SKILL.md
  - .claude/skills/speclink-onboard/SKILL.md
  - .claude/skills/speclink-propose/SKILL.md
  - .claude/skills/speclink-verify/SKILL.md
  - AGENTS.md
  - CLAUDE.md
  - apps/desktop/core/src/manage.rs
  - apps/desktop/core/src/project.rs
  - apps/desktop/src-tauri/src/lib.rs
  - apps/desktop/src-tauri/tests/remote_data.rs
  - apps/desktop/src/App.tsx
  - apps/desktop/src/__tests__/App.test.tsx
  - apps/desktop/src/__tests__/helpers/remoteFixtures.ts
  - apps/desktop/src/__tests__/instructionUpdatePrompt.test.tsx
  - apps/desktop/src/__tests__/remoteDataSource.test.ts
  - apps/desktop/src/__tests__/remoteResilience.test.tsx
  - apps/desktop/src/__tests__/store.test.ts
  - apps/desktop/src/adapter/remoteDataSource.ts
  - apps/desktop/src/adapter/workspace.ts
  - apps/desktop/src/components/InstructionUpdatePrompt.tsx
  - apps/desktop/src/i18n/messages.ts
  - apps/desktop/src/instructionPrompt.ts
  - apps/desktop/src/store.ts
  - crates/speclink-cli/tests/archive_readiness_gate.rs
  - crates/speclink-core/src/archive.rs
  - crates/speclink-core/src/command/mod.rs
  - crates/speclink-core/src/init.rs
  - crates/speclink-core/src/skills.rs
  - crates/speclink-core/tests/golden/assets.lock
  - crates/speclink-core/tests/golden/claude.snapshot.md
  - crates/speclink-core/tests/golden/codex.snapshot.md
  - crates/speclink-core/tests/golden/neutral-cli.snapshot.md
  - crates/speclink-core/tests/golden/neutral-tool-call.snapshot.md
  - crates/speclink-core/tests/golden/remote-claude.marker.md
  - crates/speclink-core/tests/render_golden.rs
  - packages/ui/src/__tests__/kanban.test.tsx
  - packages/ui/src/__tests__/richDrawer.test.tsx
  - packages/ui/src/boardDnd.ts
  - packages/ui/src/components/KanbanBoard.tsx
  - packages/ui/src/components/RichDetailDrawer.tsx
  - packages/ui/src/i18n.tsx
-->

---
### Requirement: 指令檔過期探測

引擎 SHALL 提供唯讀的指令檔過期探測：依 .speclink.yaml 的 tools 清單，讀取各內建工具指令檔的 SPECLINK 標記版號並與當前產物層版號比對，回報四態之一——缺失（任一工具的指令檔不存在，即從未安裝）、過期（任一工具的標記版號與現版不等）、現版、無法判定（設定解析失敗或指令檔存在但讀取錯誤）；缺失與過期並存時 SHALL 回報缺失。版號比對 SHALL 為字串相等判定，SHALL NOT 解析版本語意。指令檔存在但不含 SPECLINK 標記時，該工具 SHALL 視為已退出受管、不計入過期與缺失；指令檔不存在 SHALL 判缺失，SHALL NOT 與退出受管或無法判定混同。過期或缺失時 SHALL 一併回報「更新將新建或改寫且內容與現版 render 不同」的受管檔清單（專案根相對路徑）；比對前 SHALL 正規化換行，僅換行形式差異的檔案 SHALL NOT 列入清單。探測 SHALL NOT 寫入任何檔案。

#### Scenario: 舊版工作區判過期並列差異檔

- **WHEN** 工作區 CLAUDE.md 的標記版號與當前產物層版號不等，執行過期探測
- **THEN** 回報過期，並列出內容與現版 render 不同的受管檔相對路徑（含技能檔與指令檔）

#### Scenario: 現版工作區不過期

- **WHEN** 工作區全部受管檔由當前版本的 init 或 update 生成，執行過期探測
- **THEN** 回報現版，差異清單為空

#### Scenario: 標記移除視為退出受管

- **WHEN** tools 清單僅含 claude 且 CLAUDE.md 不含 SPECLINK 標記（使用者整塊移除），執行過期探測
- **THEN** 回報現版（不過期），不列任何差異檔

#### Scenario: 指令檔不存在判缺失

- **WHEN** tools 清單含 claude 與 codex，CLAUDE.md 為現版而 AGENTS.md 不存在（如 clone 後指令檔未進版控），執行過期探測
- **THEN** 回報缺失，並列出更新將新建或改寫且內容與現版 render 不同的受管檔相對路徑；不與退出受管（檔案存在但無標記）或無法判定混同

#### Scenario: 設定損壞回報無法判定

- **WHEN** .speclink.yaml 無法解析，執行過期探測
- **THEN** 回報無法判定；SHALL NOT 與現版或過期混同

#### Scenario: 換行差異不誤報

- **WHEN** 工作區技能檔內容與現版 render 僅換行形式不同（CRLF 對 LF），執行過期探測
- **THEN** 該檔不出現在差異清單

<!-- @trace
source: desktop-instruction-staleness-prompt
updated: 2026-07-31
code:
  - .agents/skills/speclink-apply/SKILL.md
  - .agents/skills/speclink-archive/SKILL.md
  - .agents/skills/speclink-audit/SKILL.md
  - .agents/skills/speclink-commit/SKILL.md
  - .agents/skills/speclink-config/SKILL.md
  - .agents/skills/speclink-discuss/SKILL.md
  - .agents/skills/speclink-drift/SKILL.md
  - .agents/skills/speclink-ingest/SKILL.md
  - .agents/skills/speclink-onboard/SKILL.md
  - .agents/skills/speclink-propose/SKILL.md
  - .claude/skills/speclink-analyze/SKILL.md
  - .claude/skills/speclink-apply/SKILL.md
  - .claude/skills/speclink-archive/SKILL.md
  - .claude/skills/speclink-audit/SKILL.md
  - .claude/skills/speclink-commit/SKILL.md
  - .claude/skills/speclink-config/SKILL.md
  - .claude/skills/speclink-discuss/SKILL.md
  - .claude/skills/speclink-drift/SKILL.md
  - .claude/skills/speclink-ingest/SKILL.md
  - .claude/skills/speclink-onboard/SKILL.md
  - .claude/skills/speclink-propose/SKILL.md
  - .claude/skills/speclink-verify/SKILL.md
  - AGENTS.md
  - CLAUDE.md
  - apps/desktop/core/src/manage.rs
  - apps/desktop/core/src/project.rs
  - apps/desktop/src-tauri/src/lib.rs
  - apps/desktop/src-tauri/tests/remote_data.rs
  - apps/desktop/src/App.tsx
  - apps/desktop/src/__tests__/App.test.tsx
  - apps/desktop/src/__tests__/helpers/remoteFixtures.ts
  - apps/desktop/src/__tests__/instructionUpdatePrompt.test.tsx
  - apps/desktop/src/__tests__/remoteDataSource.test.ts
  - apps/desktop/src/__tests__/remoteResilience.test.tsx
  - apps/desktop/src/__tests__/store.test.ts
  - apps/desktop/src/adapter/remoteDataSource.ts
  - apps/desktop/src/adapter/workspace.ts
  - apps/desktop/src/components/InstructionUpdatePrompt.tsx
  - apps/desktop/src/i18n/messages.ts
  - apps/desktop/src/instructionPrompt.ts
  - apps/desktop/src/store.ts
  - crates/speclink-cli/tests/archive_readiness_gate.rs
  - crates/speclink-core/src/archive.rs
  - crates/speclink-core/src/command/mod.rs
  - crates/speclink-core/src/init.rs
  - crates/speclink-core/src/skills.rs
  - crates/speclink-core/tests/golden/assets.lock
  - crates/speclink-core/tests/golden/claude.snapshot.md
  - crates/speclink-core/tests/golden/codex.snapshot.md
  - crates/speclink-core/tests/golden/neutral-cli.snapshot.md
  - crates/speclink-core/tests/golden/neutral-tool-call.snapshot.md
  - crates/speclink-core/tests/golden/remote-claude.marker.md
  - crates/speclink-core/tests/render_golden.rs
  - packages/ui/src/__tests__/kanban.test.tsx
  - packages/ui/src/__tests__/richDrawer.test.tsx
  - packages/ui/src/boardDnd.ts
  - packages/ui/src/components/KanbanBoard.tsx
  - packages/ui/src/components/RichDetailDrawer.tsx
  - packages/ui/src/i18n.tsx
-->

---
### Requirement: 內嵌資產版本鎖定紀律

repo SHALL 提交記錄產物層版號與全部 render 輸出指紋的鎖定檔。鎖定測試 SHALL 於 render 指紋與鎖定檔不符而版號未變時失敗，失敗訊息 SHALL 載明修復步驟（遞增版號後以指定環境變數重生鎖定檔）。鎖定檔重生 SHALL 於指紋變動而版號未變時拒絕改寫並失敗。僅遞增版號而 render 內容未變 SHALL 通過。

#### Scenario: 改資產未遞增版號即紅燈

- **WHEN** 修改內嵌技能資產內容而未遞增產物層版號，執行 speclink-core 測試
- **THEN** 鎖定測試失敗，測試輸出含遞增版號與重生鎖定檔的修復指引

#### Scenario: 防呆重生拒絕繞過

- **WHEN** 未遞增版號即以重生環境變數執行鎖定測試，且 render 指紋已變
- **THEN** 鎖定檔不被改寫，測試失敗

#### Scenario: 遞增並重生後通過

- **WHEN** 遞增產物層版號並於乾淨樹以重生環境變數更新鎖定檔後，正常執行測試
- **THEN** 鎖定測試通過，鎖定檔記錄新版號與新指紋

<!-- @trace
source: desktop-instruction-staleness-prompt
updated: 2026-07-31
code:
  - .agents/skills/speclink-apply/SKILL.md
  - .agents/skills/speclink-archive/SKILL.md
  - .agents/skills/speclink-audit/SKILL.md
  - .agents/skills/speclink-commit/SKILL.md
  - .agents/skills/speclink-config/SKILL.md
  - .agents/skills/speclink-discuss/SKILL.md
  - .agents/skills/speclink-drift/SKILL.md
  - .agents/skills/speclink-ingest/SKILL.md
  - .agents/skills/speclink-onboard/SKILL.md
  - .agents/skills/speclink-propose/SKILL.md
  - .claude/skills/speclink-analyze/SKILL.md
  - .claude/skills/speclink-apply/SKILL.md
  - .claude/skills/speclink-archive/SKILL.md
  - .claude/skills/speclink-audit/SKILL.md
  - .claude/skills/speclink-commit/SKILL.md
  - .claude/skills/speclink-config/SKILL.md
  - .claude/skills/speclink-discuss/SKILL.md
  - .claude/skills/speclink-drift/SKILL.md
  - .claude/skills/speclink-ingest/SKILL.md
  - .claude/skills/speclink-onboard/SKILL.md
  - .claude/skills/speclink-propose/SKILL.md
  - .claude/skills/speclink-verify/SKILL.md
  - AGENTS.md
  - CLAUDE.md
  - apps/desktop/core/src/manage.rs
  - apps/desktop/core/src/project.rs
  - apps/desktop/src-tauri/src/lib.rs
  - apps/desktop/src-tauri/tests/remote_data.rs
  - apps/desktop/src/App.tsx
  - apps/desktop/src/__tests__/App.test.tsx
  - apps/desktop/src/__tests__/helpers/remoteFixtures.ts
  - apps/desktop/src/__tests__/instructionUpdatePrompt.test.tsx
  - apps/desktop/src/__tests__/remoteDataSource.test.ts
  - apps/desktop/src/__tests__/remoteResilience.test.tsx
  - apps/desktop/src/__tests__/store.test.ts
  - apps/desktop/src/adapter/remoteDataSource.ts
  - apps/desktop/src/adapter/workspace.ts
  - apps/desktop/src/components/InstructionUpdatePrompt.tsx
  - apps/desktop/src/i18n/messages.ts
  - apps/desktop/src/instructionPrompt.ts
  - apps/desktop/src/store.ts
  - crates/speclink-cli/tests/archive_readiness_gate.rs
  - crates/speclink-core/src/archive.rs
  - crates/speclink-core/src/command/mod.rs
  - crates/speclink-core/src/init.rs
  - crates/speclink-core/src/skills.rs
  - crates/speclink-core/tests/golden/assets.lock
  - crates/speclink-core/tests/golden/claude.snapshot.md
  - crates/speclink-core/tests/golden/codex.snapshot.md
  - crates/speclink-core/tests/golden/neutral-cli.snapshot.md
  - crates/speclink-core/tests/golden/neutral-tool-call.snapshot.md
  - crates/speclink-core/tests/golden/remote-claude.marker.md
  - crates/speclink-core/tests/render_golden.rs
  - packages/ui/src/__tests__/kanban.test.tsx
  - packages/ui/src/__tests__/richDrawer.test.tsx
  - packages/ui/src/boardDnd.ts
  - packages/ui/src/components/KanbanBoard.tsx
  - packages/ui/src/components/RichDetailDrawer.tsx
  - packages/ui/src/i18n.tsx
-->
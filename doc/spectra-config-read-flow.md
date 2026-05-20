# Spectra Config 讀取流程反推

日期：2026-05-20

本文整理 `spectra-cli` 讀取 `.spectra.yaml` 與 `config.yaml` 的時機、用途與 workflow 影響。結論來自既有反組譯紀錄、binary 字串交叉檢查，以及臨時專案實驗。

## 結論摘要

Spectra 的設定檔分成三類：

| 檔案 | 類型 | 主要用途 |
| --- | --- | --- |
| `.spectra.yaml` | project runtime config | 決定 Spectra runtime 行為，例如 `spec_dir`、`locale`、`parallel_tasks`、`tdd`、`audit`、`worktree`、`tools`。 |
| `<spec_dir>/config.yaml` | OpenSpec project config | 決定 SDD workflow 設定，例如 default `schema`、project `context`、per-artifact `rules`。預設是 `openspec/config.yaml`。 |
| `%APPDATA%\openspec\config.yaml` | user global config | 由 `spectra config ...` 管理的全域設定。目前未確認一般 workflow 指令會依賴它。 |

最重要的分層：

```text
.spectra.yaml
  -> CLI / runtime / project discovery 設定

openspec/config.yaml
  -> SDD / OpenSpec / artifact generation 設定
```

若 `.spectra.yaml` 設定了 `spec_dir`，則 `config.yaml` 會跟著該 spec root 走：

```text
.spectra.yaml: spec_dir: customspec

實際 project config:
customspec/config.yaml
```

## `.spectra.yaml`

`.spectra.yaml` 是 Spectra 專案層級的 runtime 設定。binary 字串中可看到 `SpectraConfig` 與以下欄位：

```text
spec_dir
locale
tdd
audit
parallel_tasks
claude_slash_commands
worktree
worktrees_dir
claude_effort
tools
```

### 預設產生時機

`spectra init` 會建立 `.spectra.yaml`。

範例內容：

```yaml
# Spectra application config
# See: https://github.com/spectra-app/spectra

# OpenSpec directory path (relative to project root)
# Changing this requires rebuilding the vector search index.
# spec_dir: docs/specs

# Language for AI-generated artifacts
# locale: tw

# Workflow toggles
# tdd: true
# audit: true
# parallel_tasks: true

# Claude slash commands (set true to also generate /spectra:X commands)
# claude_slash_commands: true

# Enable git worktree support for isolated change branches
# worktree: true

# Custom git worktrees directory
# worktrees_dir: .spectra/worktrees

# Claude Code skill effort levels (low/medium/high/xhigh/max)
# claude_effort:
#   apply: high

# AI tools to generate instruction files for
# tools:
#   - claude
#   - cursor
```

### 已驗證用途

| 欄位 | 已確認用途 | 信心 |
| --- | --- | --- |
| `spec_dir` | 影響 CLI 讀寫 spec root。`new change`、`list`、`show`、`status`、`validate`、`analyze`、`archive` 等會從此目錄讀取 changes/specs。 | 高 |
| `locale` | `spectra instructions <artifact> --json` 會讀取並轉成完整語言名稱，例如 `tw` -> `Traditional Chinese (繁體中文)`。 | 高 |
| `parallel_tasks` | 主要由內嵌 skill instructions 要求 AI 讀取，用於決定是否在 tasks artifact 加 `[P]` markers 或 apply 時並行執行 `[P]` tasks。 | 高 |
| `tdd` | 內嵌 debug/apply skill instructions 會要求 AI 在 `tdd: true` 時抓取 TDD instructions。 | 中 |
| `audit` | binary 與預設設定包含此欄位，但本次未完整動態驗證具體 workflow 行為。 | 中低 |
| `tools` | `spectra init --tools` 會產生工具對應 instruction files；`spectra update` 與工具產生流程相關。但本次觀察到 `spectra update` 仍以既有工具檔案為主，未完整驗證手動修改 `tools` 後的所有行為。 | 中 |
| `claude_slash_commands` | binary 中存在，預設設定說明用於 Claude slash commands；本次未完整動態驗證。 | 中低 |
| `worktree` / `worktrees_dir` | binary 中存在 worktree 相關字串與 ownership/storage 字串，推測用於 isolated change branches / worktree artifact ownership。未完整動態驗證。 | 中低 |
| `claude_effort` | 預設設定與 binary 中存在，推測用於產生 Claude skill 或 command metadata。未完整動態驗證。 | 中低 |

### `spec_dir` 實驗

實驗設定：

```yaml
# .spectra.yaml
spec_dir: customspec
locale: tw
```

執行：

```bash
spectra new change specdir-change --agent codex --description "spec dir probe"
```

觀察結果：

```text
Path: ...\customspec\changes\specdir-change
```

而不是：

```text
...\openspec\changes\specdir-change
```

這代表 `.spectra.yaml` 會在 command handler 讀取前或 early project discovery 階段被解析，用來決定 OpenSpec root。

### `locale` 實驗

實驗設定：

```yaml
# .spectra.yaml
locale: tw
```

執行：

```bash
spectra instructions proposal --change cfg-probe --json
```

觀察結果：

```json
{
  "locale": "Traditional Chinese (繁體中文)"
}
```

若 `.spectra.yaml` 沒有設定 `locale`，預設回傳：

```json
{
  "locale": "English"
}
```

### invalid `.spectra.yaml` 行為

實驗中將 `.spectra.yaml` 設為 malformed YAML：

```yaml
locale: [
```

觀察：

- `spectra list --json` 仍可正常列出 changes。
- `spectra instructions ... --json` 仍可正常回傳，且 `locale` fallback 為 `English`。

推論：

- `.spectra.yaml` 的解析錯誤在部分命令中不是 hard failure。
- Spectra 可能採取 best-effort 讀取，失敗時回落到 default config。

此行為需再用更多欄位與命令驗證。

## `<spec_dir>/config.yaml`

`<spec_dir>/config.yaml` 是 SDD / OpenSpec 專案設定。預設路徑為：

```text
openspec/config.yaml
```

若 `.spectra.yaml` 設定：

```yaml
spec_dir: customspec
```

則實際讀取：

```text
customspec/config.yaml
```

### 預設產生時機

`spectra init` 會建立：

```text
openspec/config.yaml
```

預設內容：

```yaml
schema: spec-driven

# Project context (optional)
# This is shown to AI when creating artifacts.
# Add your tech stack, conventions, style guides, domain knowledge, etc.
# Example:
#   context: |
#     Tech stack: TypeScript, React, Node.js
#     We use conventional commits
#     Domain: e-commerce platform

# Per-artifact rules (optional)
# Add custom rules for specific artifacts.
# Example:
#   rules:
#     proposal:
#       - Keep proposals under 500 words
#       - Always include a "Non-goals" section
#     tasks:
#       - Break tasks into chunks of max 2 hours
```

### 已驗證用途

| 欄位 | 已確認用途 | 信心 |
| --- | --- | --- |
| `schema` | `spectra new change` 會讀取它作為新 change 的 default schema，並寫入 `changes/<change>/.openspec.yaml`。 | 高 |
| `context` | `spectra instructions <artifact> --json` 會把它注入 JSON 的 `context` 欄位，提供 AI 生成 artifact 時使用。 | 高 |
| `rules.<artifact>` | `spectra instructions <artifact> --json` 會把 artifact-specific rules 注入 JSON 的 `rules` 欄位。 | 高 |

### `schema` 實驗

先 fork 一個 schema：

```bash
spectra schema fork spec-driven alt-flow
```

修改 project config：

```yaml
# openspec/config.yaml
schema: alt-flow
```

執行：

```bash
spectra new change default-schema-probe --agent codex --description "schema default probe"
```

觀察 `.openspec.yaml`：

```yaml
schema: alt-flow
created: 2026-05-20
created_by: MomoChen <momochenisme@gmail.com>
created_with: codex
```

這代表 `new change` 階段會讀 `<spec_dir>/config.yaml` 的 `schema`，作為 change metadata 的 schema 來源。

注意：

- fork 出來的 `alt-flow/schema.yaml` 內部 `name` 仍是 `spec-driven`。
- 後續 `status` / `instructions` JSON 顯示的 `schemaName` 會來自 schema file 內部 `name`，因此仍可能顯示 `spec-driven`。
- `.openspec.yaml` 記錄的是 change 選用的 schema key/path resolution 名稱。

### `context` 與 `rules` 實驗

修改：

```yaml
# openspec/config.yaml
schema: spec-driven
context: |
  Runtime probe context from openspec config.
rules:
  proposal:
    - Proposal must mention PROBE-RULE.
  tasks:
    - Tasks must be grouped by module.
```

執行：

```bash
spectra instructions proposal --change cfg-probe --json
```

觀察結果：

```json
{
  "context": "Runtime probe context from openspec config.",
  "rules": [
    "Proposal must mention PROBE-RULE."
  ]
}
```

因此 `config.yaml` 的 `context` / `rules` 是 CLI 給 AI 的重要資料來源，不只是人類閱讀用註解。

### 搭配 `spec_dir` 的 `config.yaml`

實驗設定：

```yaml
# .spectra.yaml
spec_dir: customspec
locale: tw
```

建立：

```yaml
# customspec/config.yaml
schema: spec-driven
context: |
  Context from customspec config.
rules:
  proposal:
    - Customspec proposal rule.
```

執行：

```bash
spectra new change customspec-cfg --agent codex --description "custom spec dir config"
spectra instructions proposal --change customspec-cfg --json
```

觀察：

```text
changeDir: ...\customspec\changes\customspec-cfg
context:   Context from customspec config.
rules:     Customspec proposal rule.
locale:    Traditional Chinese (繁體中文)
```

結論：

- `.spectra.yaml` 先決定 `spec_dir`。
- CLI 再從 `<spec_dir>/config.yaml` 讀 project config。
- `instructions` 同時合併 `.spectra.yaml` 的 runtime 設定與 `<spec_dir>/config.yaml` 的 artifact 設定。

### invalid `config.yaml` 行為

實驗中將 `openspec/config.yaml` 設為 malformed YAML：

```yaml
schema: [
```

觀察：

- `spectra list --json` 正常。
- `spectra new change ...` 正常，且 fallback 到 `spec-driven`。
- `spectra instructions ... --json` 正常，且沒有 `context` / `rules` 注入。

推論：

- `<spec_dir>/config.yaml` 解析失敗時，在部分命令中不是 hard failure。
- Spectra 可能採用 default project config，至少 default schema 會回落為 `spec-driven`。
- 此行為對錯誤提示不明顯，若新工具要仿照，應考慮更明確地回傳 warning。

## `%APPDATA%\openspec\config.yaml`

`spectra config` 指令管理的全域設定路徑為：

```text
C:\Users\momoc\AppData\Roaming\openspec\config.yaml
```

實測：

```bash
spectra config path
```

輸出：

```text
C:\Users\momoc\AppData\Roaming\openspec\config.yaml
```

在未設定時：

```bash
spectra config list
```

輸出：

```text
No configuration set.
```

本次尚未確認一般 workflow 指令，如 `new change`、`instructions`、`status`、`validate`，會讀取或套用這個全域 config。

目前較可靠的定位：

```text
spectra config ...
  -> user-level config management command

.spectra.yaml / <spec_dir>/config.yaml
  -> project workflow runtime 使用
```

## 推定讀取流程

整體流程可推定為：

```text
CLI 啟動
  -> 找 project root
  -> 讀 .spectra.yaml
       - 決定 spec_dir
       - 決定 locale
       - 取得 workflow toggles
       - 取得 tool/worktree 相關設定
  -> 解析 spec root
       - 預設 openspec/
       - 或 .spectra.yaml 的 spec_dir
  -> 讀 <spec_dir>/config.yaml
       - 取得 default schema
       - 取得 project context
       - 取得 per-artifact rules
  -> 讀 change/.openspec.yaml
       - 取得該 change 實際 schema
       - 取得 created/created_by/created_with
  -> resolve schema
       - built-in
       - project schema: <spec_dir>/schemas/<name>/schema.yaml
       - user schema: %APPDATA%\openspec\schemas/<name>/schema.yaml
  -> 執行 command handler
       - list/show/status
       - instructions
       - new artifact
       - validate/analyze
       - archive
```

## 指令與設定讀取對照

| 指令 | `.spectra.yaml` | `<spec_dir>/config.yaml` | 說明 |
| --- | --- | --- | --- |
| `spectra init` | 產生 | 產生 | 建立 `.spectra.yaml` 與預設 `openspec/config.yaml`。 |
| `spectra update` | 可能讀 `tools` / tool-specific settings | 未確認 | 更新 instruction files。實際工具來源仍需更細驗證。 |
| `spectra list` | 讀 `spec_dir` | 通常不需要 | 從 spec root 列出 changes/specs。 |
| `spectra show` | 讀 `spec_dir` | 可能不需要 | 從 spec root 讀 change/spec artifacts。 |
| `spectra new change` | 讀 `spec_dir` | 讀 `schema` | 在 spec root 建立 change，並把 schema 寫進 `.openspec.yaml`。 |
| `spectra status` | 讀 `spec_dir` | 可能讀 schema context，但主要依 change schema | 評估 artifact DAG。 |
| `spectra instructions` | 讀 `spec_dir`、`locale` | 讀 `context`、`rules`、可能讀 default schema | 回傳 AI artifact instructions。 |
| `spectra new artifact` | 讀 `spec_dir` | 間接受 schema/rules 影響 | 寫入 artifact，輸出路徑由 schema artifact definition 決定。 |
| `spectra validate` | 讀 `spec_dir` | 間接受 schema 影響 | 驗證 changes/specs。 |
| `spectra analyze` | 讀 `spec_dir` | 間接受 schema/rules 影響 | 分析 artifacts 一致性與缺口。 |
| `spectra archive` | 讀 `spec_dir` | 間接受 schema 影響 | 將 change delta specs 套用到 main specs 並移入 archive。 |
| `spectra config ...` | 不明 | 不明 | 管理 `%APPDATA%\openspec\config.yaml`，不是 project-local `openspec/config.yaml`。 |

## 對新工具設計的啟發

若新工具要保留 Spectra 的優點，建議也保留兩層設定：

```text
runtime config
  - provider
  - spec_dir / local cache root
  - locale
  - feature toggles
  - worktree settings
  - tool/skill installation settings

project workflow config
  - default schema
  - project context
  - artifact rules
  - schema/rules source
```

對應新工具可設計為：

```text
.speclink/config.toml
  -> runtime / provider / auth profile binding / fallback

<workspace spec root>/config.yaml
  -> SDD workflow schema / context / rules
```

但建議補強 Spectra 的弱點：

- malformed config 不應靜默 fallback，應回傳 warning 或明確 error。
- `instructions --json` 應明確標示每個欄位來源，例如 `localeSource`、`contextSource`、`rulesSource`。
- 若使用 remote provider，project config 可由 provider 提供，但 CLI 應保留 local fallback。

## 信心分級

高信心：

- `.spectra.yaml` 的 `spec_dir` 會改變 OpenSpec root。
- `.spectra.yaml` 的 `locale` 會進入 `instructions` JSON。
- `<spec_dir>/config.yaml` 的 `schema` 會影響 `new change` 的 `.openspec.yaml`。
- `<spec_dir>/config.yaml` 的 `context` / `rules` 會進入 `instructions` JSON。
- `spectra config path` 指向 `%APPDATA%\openspec\config.yaml`。

中信心：

- `tdd`、`parallel_tasks`、`audit`、`tools` 更多是 skill/template generation 與 AI workflow 行為的一部分，其中 `parallel_tasks` 與 `tdd` 有內嵌 skill 文字支撐。
- `spectra update` 與 `.spectra.yaml.tools` 的關係需要更細測試，尤其是 init 後既有工具檔案與手動修改 tools 的互動。

中低信心：

- `worktree`、`worktrees_dir`、`claude_effort`、`claude_slash_commands` 的完整 runtime 行為尚未動態驗證。
- 全域 `%APPDATA%\openspec\config.yaml` 是否會被一般 workflow fallback 讀取，尚未確認。


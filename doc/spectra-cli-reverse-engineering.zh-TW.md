# Spectra CLI 反向工程筆記

日期：2026-05-19

目標執行檔：

```text
C:\Users\momoc\AppData\Local\Spectra\spectra.exe
```

本文記錄目前針對 `spectra.exe` 的靜態反向工程筆記。分析範圍限於指令行為、檔案系統副作用、二進位 metadata、PE 結構、字串交叉參照，以及本機測試觀察。不包含繞過授權、修補執行檔、擷取秘密資訊，或修改二進位檔。

## 範圍

目標是理解 Spectra CLI 各指令內部做了什麼，特別是本機 skills 會用到的工作流程指令：

- `spectra init`
- `spectra new change`
- `spectra new artifact`
- `spectra status`
- `spectra instructions`
- `spectra validate`
- `spectra analyze`
- `spectra park`
- `spectra unpark`
- `spectra task done`
- `spectra archive`
- `spectra drift`
- `spectra config`
- `spectra schema`
- `spectra list`
- `spectra show`

下列發現混合兩種證據：

- **實測行為**：在臨時專案中執行 CLI 驗證。
- **靜態推論**：從 PE metadata、字串、原始碼路徑參照、`.pdata` 函式範圍與本機 x64 反組譯推得。

## 二進位 Metadata

觀察到的 metadata：

```text
File:      C:\Users\momoc\AppData\Local\Spectra\spectra.exe
Size:      8,351,232 bytes
SHA256:    6e0a525ae4605fe5199aadfa623ffea7413e0a441faee207eac2917ef5d7f1e2
Version:   spectra 2.3.1 (x64)
Format:    PE32+ / x86-64
Subsystem: Windows console
Entry VA:  0x1405d3510
PDB ref:   spectra.pdb
```

PE sections：

```text
.text
.rdata
.data
.pdata
.tls
.reloc
```

此二進位看起來是 Rust release binary。它包含 Rust runtime 字串、panic 字串、Cargo registry 路徑與專案原始碼路徑。CLI parser 是 `clap_builder-4.6.0`。

從字串與 imports 推得的主要 crates / libraries：

```text
clap
clap_complete
serde_json
serde_yaml
regex
chrono
git2
libgit2
rusqlite
libsqlite3-sys
colored
```

Windows imports 包含檔案系統、process、networking、cryptography、WinHTTP 與 shell API。這與一個會使用檔案系統、git、SQLite，以及可選 network-capable dependencies 的 Rust CLI 相符。

## 分析方法

使用的工具與技術：

- `spectra --help` 與子指令 `--help`
- `%TEMP%` 下的臨時專案 probes
- `Get-Command`、`Get-FileHash`、`Format-Hex`
- 使用 `pefile` 做 Python PE parsing
- 使用 `capstone` 做 x64 disassembly
- ASCII 字串擷取
- 解析 `.pdata` exception directory 以還原函式範圍
- RIP-relative string reference scanning
- 從臨時 probe 資料檢查 SQLite schema

臨時 Python dependencies 安裝於 repository 外：

```text
C:\Users\momoc\AppData\Local\Temp\spectra-re-python
```

## 驗證回合：2026-05-19

第二輪可信度驗證是在乾淨的臨時專案中執行：

```text
C:\Users\momoc\AppData\Local\Temp\spectra-credibility-83f736bea80740ffb0ef805725868b8d
```

此 probe 重新執行核心指令，並將觀察到的副作用與本文的靜態發現比對。

已驗證的指令結果：

| 區域 | 驗證結果 |
| --- | --- |
| Binary identity | `spectra --version` 回傳 `spectra 2.3.1 (x64)`，與 binary metadata 區段一致。 |
| `init` | 建立 `openspec/`、`.spectra.yaml`、`AGENTS.md`、`.agents/skills` 與 Codex skill files。 |
| `new change` | 建立 `openspec/changes/verify-cli/.openspec.yaml`，其中包含 `schema`、`created`、`created_by`、`created_with`。 |
| `new artifact` failure paths | 重現 invalid artifact type、missing spec capability、invalid kebab-case capability、empty stdin、invalid proposal errors。 |
| `new artifact` success paths | 建立 `proposal.md`、`design.md`、`tasks.md` 與 `specs/verify-cli/spec.md`；JSON output 包含 `artifact`、`change`、`path`、`status`、`validated`、`warnings`。 |
| `status` | 確認初始 DAG（`proposal` ready；`design`、`specs`、`tasks` blocked）以及 artifacts 建立完成後的完整 DAG。 |
| `validate` | 確認 JSON shape 包含 `change`、`errors`、`valid`、`warnings`。 |
| `analyze` | 確認 dimensions 為 `Coverage`、`Consistency`、`Ambiguity`、`Gaps`；觀察到缺少具體 examples 的 scenario 會產生 `ambAbstractScenario` suggestion。 |
| `task done` | 確認 task checkbox 由 `- [ ]` 變成 `- [x]`，JSON output 包含 `change`、`status`、`task_desc`、`task_id`。 |
| `park` | 確認 active change 從 `openspec/changes/<name>` 移除，並儲存在 `.git/spectra-app/changes/<name>`。 |
| `unpark` | 確認恢復到 `openspec/changes/<name>`，並從 `.git/spectra-app/changes/<name>` 移除。 |
| SQLite | 確認 `parked_changes` table schema，且 unpark 會清掉該 table 的 rows。 |
| `show` | 確認 JSON shape 包含 `created`、`deltaSpecs`、`design`、`name`、`proposal`、`schema`、`tasks`。 |
| `instructions` | 確認 proposal instruction JSON shape 包含 `changeName`、`artifactId`、`schemaName`、`changeDir`、`outputPath`、`description`、`instruction`、`locale`、`template`、`dependencies`、`unlocks`。 |
| `schemas` / `templates` / `schema which` | 確認 built-in `spec-driven` schema 與 embedded template metadata。 |
| `search` | 在此 Windows x64 build 上確認 `{"error":"vector_not_compiled","results":[]}`。 |
| `archive` | 確認 archive directory 會建立於 `openspec/changes/archive/2026-05-19-verify-cli`，spec 會套用到 `openspec/specs/verify-cli/spec.md`，並輸出 `Specs applied`。 |

此回合的修正，後續又由深度 probe pass 進一步細化：

- 先前 `task done` 區段對 `.spectra/touched.json` 描述過強。簡單 probe 沒有建立 `.spectra/touched.json` 或 `.spectra/touched/<change>.json`，因此第一輪可信度 pass 先將 touched tracking 降級。後續 deep probe 找到缺少的條件：Git worktree 已經有 modified 或 staged files 時，才會建立 touched metadata。

## 深度 Probe 回合：2026-05-19

額外的 targeted probes 執行於以下臨時專案：

```text
C:\Users\momoc\AppData\Local\Temp\spectra-deep-probes-qrfh55lk
C:\Users\momoc\AppData\Local\Temp\spectra-targeted-probes-nrjdyosz
C:\Users\momoc\AppData\Local\Temp\spectra-anchor-probes-6w1crqmc
```

新驗證的發現：

| 區域 | 結果 |
| --- | --- |
| `analyze` | 使用合法 delta specs 時，確認 clean output、missing spec、missing task、weak-language findings。 |
| `analyze` JSON | 確認這些 probes 中不會輸出 aggregate score；它輸出 `change_id`、`dimensions`、`findings`、`artifacts_analyzed`、`artifacts_missing`。 |
| `drift` time score | 確認 time scores：`fresh (0d)` = 0、`aging (8d/15d)` = 1、`stale (31d)` = 2、`abandoned (91d)` = 4。 |
| `drift` severity | `total_score: 4` 產生 `severity: medium` 且 recommendation 為 `/spectra-ingest <change>`；scores 0-2 維持 `light` 且 recommendation 為 `/spectra-apply <change>`。 |
| `drift` environment | Git commits since creation 會出現在 Environment dimension，但觀察到的 JSON 中 `contributes_to_total` 是 `false`。 |
| `drift` structure | 移除 `design.md` 會讓 Structure status 變成 `design absent`；移除 `tasks.md` 會讓 Tasks status 變成 `no tasks.md`。這些在 probes 中沒有加到 total score。 |
| `schema which` | Built-in `spec-driven` resolves to `(embedded in binary)`；project custom schemas resolves from `openspec/schemas/<name>/schema.yaml`。 |
| `schema validate` | Custom artifact entry 缺少 `generates` 時 validation failed，錯誤為 `artifacts[0]: missing field generates`。 |
| `schema fork` | `schema fork spec-driven forked-flow` 會建立 `openspec/schemas/forked-flow/schema.yaml`，之後 `schema which forked-flow` resolves to `project`。 |
| `task done` touched tracking | 當 Git worktree 在執行 `task done` 前已有 modified 或 staged files 時，確認會建立 `.spectra/touched/<change>.json`。 |
| `task done` touched conditions | 非 Git project 或乾淨 Git worktree（只有 command 本身 mutation `tasks.md`）不會建立 touched file。 |

此回合後仍未解的細節，後續會在下面重新檢驗：

- `conDesignNotInTasks` 存在於 embedded strings，但簡單的 valid design/task mismatch 沒有觸發。Trigger 看起來比「任何 design decision 沒出現在 tasks」更窄。
- `drift` broken-anchor scoring 在刪除 proposal/spec/task files 後沒有觸發。「anchors」看起來是 internal design anchors 或 parsed references，而不只是 required artifact files。
- 即使存在 touched-file tracking 與後續 external commits，`drift` 的 task-blocked 與 maybe-resolved arrays 仍為空。其 trigger 可能依賴更特定的 task/file/reference patterns。
- User-level schema resolution 仍由字串推論；built-in 與 project resolution 已動態驗證。

## 不明分支解析回合：2026-05-19

剩餘不明區域用另一組 targeted projects 測試：

```text
C:\Users\momoc\AppData\Local\Temp\spectra-unclear-probes-vzx22s4b
C:\Users\momoc\AppData\Local\Temp\spectra-more-unclear-r69boman
C:\Users\momoc\AppData\Local\Temp\spectra-covdelta-75b0elgl
C:\Users\momoc\AppData\Local\Temp\spectra-gap-probes-5vu289ec
```

新解析的發現：

| 區域 | 結果 |
| --- | --- |
| `conDesignNotInTasks` | 由 `design.md` 中 decisions 下的 `###` headings 觸發。Heading text 會轉小寫並在 `tasks.md` 中搜尋。缺少 headings 會產生 `CON-1` / `conDesignNotInTasks`。 |
| `ambNoScenario` | 沒有任何 `#### Scenario` 的 requirement 會產生 Ambiguity `Warning`，key 為 `ambNoScenario`。 |
| `gapNoMainSpec` | 對沒有 `openspec/specs/<cap>/spec.md` 的 capability 使用 `MODIFIED` delta，會產生 Gaps `Warning`。 |
| `gapModifiedNotFound` | `MODIFIED` requirement 的 name 不存在於 main spec 時，會產生 Gaps `Warning`。 |
| `tasks_maybe_resolved` | Pending tasks 的 verb/target words 若與較晚 Git commit subject 匹配，會標成 maybe-done，例如 task `Update login cache` 與 commit `Update login cache handling`。 |
| `broken_anchors` | Design reference 指向缺失的 CLI flag，例如 `` `--probe-mode` ``，會產生 broken anchor，category 為 `CliFlag`，reason 為 `not in --help`。 |
| Drift structure score | 一個 broken CLI flag anchor 會讓 Structure score +3。加上 Time score 1 後，total score 變成 4，severity 為 `medium`。 |
| User schema path | 真實 user schema 位於 `%APPDATA%\openspec\schemas\<name>\schema.yaml` 時 resolves as source `user`。其他測試位置不會 resolve。 |
| Schema precedence | 同名 project schema resolves as `project`，會覆蓋 user-level schema。 |
| `in-progress` | 此 build 只暴露 `in-progress add <name>`；`list`、`remove`、`clear` 不是 CLI subcommands。 |
| `demo` | 此 build 的 `spectra demo` 沒有 `--json` option。 |

擴充 probes 後仍未解：

- `tasks_blocked_external` 仍未觸發。測試變體包含 backticked paths、plain paths、Windows-style paths、path-only tasks，以及後續 commits 修改 referenced file。全部都產生 `0 blocked`。Embedded documentation 說它用於「pending tasks whose referenced files were modified by commits outside the change dir」，但實際 parser/condition 比這些 probes 更窄。
- File-path、symbol、function anchors 會被算入 denominator，但在簡單 missing-reference probes 中沒有變成 broken。CLI flag anchors 已驗證；其他 anchor categories 仍部分未解。
- `covDeltaValidation` 仍只在 embedded strings 中觀察到，尚未直接動態觀察。Malformed specs 不是被 analyzer coverage 當成 missing usable specs，就是由 `validate` 顯示；tested malformed cases 中 analyzer 沒有輸出該 key。

## Source Path References

Binary 包含 source path strings，可將主要功能對回 Rust modules：

```text
crates/spectra-cli/src/commands/demo.rs
crates/spectra-cli/src/commands/list.rs
crates/spectra-cli/src/commands/new_artifact.rs
crates/spectra-cli/src/commands/park.rs
crates/spectra-cli/src/commands/schema_mgmt.rs
crates/spectra-cli/src/commands/schemas.rs
crates/spectra-cli/src/commands/show.rs
crates/spectra-cli/src/commands/validate.rs

crates/spectra-core/src/analyzer.rs
crates/spectra-core/src/archive.rs
crates/spectra-core/src/change.rs
crates/spectra-core/src/db/connection.rs
crates/spectra-core/src/db/schema.rs
crates/spectra-core/src/delta_parser.rs
crates/spectra-core/src/drift.rs
crates/spectra-core/src/init.rs
crates/spectra-core/src/park.rs
crates/spectra-core/src/preflight.rs
crates/spectra-core/src/spectra_config.rs
crates/spectra-core/src/task_parser.rs
crates/spectra-core/src/templates/skills_templates.rs
crates/spectra-core/src/trace_parser.rs
```

這些路徑不代表 source 目前存在於本機；它們是 build 中 embedded strings。

## 函式候選

以下 virtual address ranges 由 `.pdata` 與 string cross-references 識別。名稱是推論 label，不是 debug symbols。

```text
new artifact wrapper       0x140051100 - 0x1400527f4
new artifact core          0x140114b00 - 0x140115e64
artifact validation core   0x1401174f0 - 0x140119614
park/list storage path     0x14003ea20 - 0x140040876
park command               0x140042580 - 0x140042d50
unpark command             0x140041670 - 0x140042063
park core                  0x140169ed0 - 0x14016ade6
validate command           0x14006a680 - 0x14006c6d7
instructions command       0x14004c3f0 - 0x14004e69b
task done command          0x14005d600 - 0x1400603ce
archive command wrapper    0x140037220 - 0x1400388a8
archive core               0x140144c20 - 0x14014781d
show command               0x140066c60 - 0x14006917f
schemas command            0x140086380 - 0x140087c97
schema management command  0x140045960 - 0x140049d77
init command wrapper       0x14005cac0 - 0x14005d159
init core                  0x140331590 - 0x140331c8d
update command             0x140075960 - 0x1400761c1
config command             0x1400582b0 - 0x14005991a
config core                0x1405d65c0 - 0x1405d8446
analyzer main              0x14012f9f0 - 0x1401367de
analyzer JSON summary      0x140140dd0 - 0x14014138d
drift command/display      0x140053340 - 0x1400567a5
drift JSON/serde helpers   0x140140040 - 0x1401415a0
db connection/migration    0x140161190 - 0x140163c97
preflight helper           0x14011a850 - 0x14011aaf1
```

後續 pass 的 focused static xref checks：

- `conDesignNotInTasks.summary` 與 `Verify tasks cover this design decision` 都 cross-reference analyzer function range `0x14012f9f0 - 0x1401367de`。
- `broken_anchors`、`tasks_maybe_resolved`、`tasks_blocked_external` cross-reference `0x140140040 - 0x1401415a0` 附近的 JSON/serde helper ranges。
- Schema resolution strings `project`、`openspec/schemas`、`user`、`(embedded in binary)`、`built-in` cross-reference `schema management command 0x140045960 - 0x140049d77`。

## Command Surface

Top-level help 暴露以下 commands：

```text
init
update
list
show
validate
analyze
drift
archive
status
instructions
new
schemas
templates
feedback
schema
config
search
completion
park
unpark
task
in-progress
demo
help
```

透過 `spectra schemas --json` 觀察到的 default workflow schema：

```json
[
  {
    "name": "spec-driven",
    "description": "Default OpenSpec workflow - proposal -> specs -> design -> tasks",
    "artifacts": ["proposal", "specs", "design", "tasks"],
    "source": "package"
  }
]
```

`spectra schema which --json --all` 回報此 schema 是 built in：

```json
{
  "name": "spec-driven",
  "resolved": "built-in",
  "sources": [
    {
      "path": "(embedded in binary)",
      "source": "built-in"
    }
  ]
}
```

## `spectra init`

在臨時專案中觀察到的行為：

- 建立 `.spectra.yaml`
- 建立 `AGENTS.md`
- 建立 `.agents/skills/<skill-name>/SKILL.md`
- 建立 `openspec/config.yaml`
- 建立 `openspec/changes/`
- 建立 `openspec/specs/`
- 建立或更新 `.gitignore`

相關字串：

```text
Already initialized. Use --force to reinitialize.
Initialized at
<!-- SPECTRA:START
<!-- SPECTRA:END -->
```

靜態推論：

- init command 使用 embedded templates。
- Core config generator 位於 `crates/spectra-core/src/spectra_config.rs`。
- Default `.spectra.yaml` content embedded in binary。

Binary 內 embedded 的重要 config keys：

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

## `spectra update`

靜態推論：

- 更新 configured tools 的 generated instruction files。
- 使用與 init 相同的 tool mapping。
- Recognized tool targets 包含：

```text
claude
cursor
windsurf
cline
gemini
github-copilot
kiro
roocode
continue
opencode
codebuddy
costrict
cospec
antigravity
auggie
amazon-q
kilocode
factory
iflow
qoder
qwen
codex
crush
trae
```

相關字串：

```text
Updated instruction files for:
No AI tool configurations found. Use 'spectra init --tools' to set up.
```

## `spectra new change`

觀察到的行為：

- 建立 `openspec/changes/<name>/`
- 寫入 `openspec/changes/<name>/.openspec.yaml`
- 儲存 schema 與 agent metadata。

從 probe change 觀察到的 `.openspec.yaml` metadata footprint 很小，包含 created time 與 schema/agent fields。

靜態推論：

- `new change` 會驗證或預期 kebab-case names。
- 可接受 `--description`、`--schema`、`--agent`。
- 若 change 已存在，會報錯而不是 overwrite。

## `spectra new artifact`

這是目前 reverse-engineered path 最清楚的 command。

相關函式：

```text
new artifact wrapper       0x140051100 - 0x1400527f4
new artifact core          0x140114b00 - 0x140115e64
artifact validation core   0x1401174f0 - 0x140119614
```

觀察到的行為：

- `proposal` 寫入 `proposal.md`
- `design` 寫入 `design.md`
- `tasks` 寫入 `tasks.md`
- `spec <capability>` 寫入 `specs/<capability>/spec.md`
- `--stdin` 從 standard input 讀 artifact content。
- `--force` 覆寫既有 artifacts。
- `--json` 回傳 created artifact path 與 validation status。

靜態推論：

1. 解析 active change，或使用 `--change`。
2. 驗證 artifact type。
3. 對 `spec` 要求 capability name。
4. 驗證 spec capability name 必須是 kebab-case。
5. 依 artifact type 決定 output path。
6. 如果 artifact 已存在且未設定 `--force`，拒絕。
7. 若設定 `--stdin`，讀取 stdin。
8. 拒絕空 stdin content。
9. 執行 artifact content validation。
10. 寫入 artifact file。
11. 回傳 JSON 或 human-readable status。

相關 validation strings：

```text
Unknown artifact type ''. Valid types: proposal, design, tasks, spec
Capability name is required for spec type. Usage: spectra new artifact spec <capability> --change <name>
Invalid capability name ''. Must be kebab-case (e.g., user-auth, data-export)
No content received from stdin
Artifact already exists: . Use --force to overwrite
```

由字串推論的 minimum content validation：

```text
proposal -> must contain ## Why, ## Problem, or ## Summary
design   -> must contain ## Context
tasks    -> must contain at least one checkbox (- [ ])
spec     -> must parse as a delta spec
```

相關字串：

```text
Proposal must contain a ## Why, ## Problem, or ## Summary section
Design must contain a ## Context section
Tasks must contain at least one checkbox (- [ ])
Delta spec parse error:
Delta spec validation failed:
Content validated
```

## `spectra status`

觀察到的行為：

只有 `.openspec.yaml` 的 change，`spectra status --change <name> --json` 回傳：

```json
{
  "changeName": "inspect-cli",
  "schemaName": "spec-driven",
  "isComplete": false,
  "applyRequires": ["tasks"],
  "artifacts": [
    {"id": "proposal", "outputPath": "proposal.md", "status": "ready"},
    {"id": "design", "outputPath": "design.md", "status": "blocked", "missingDeps": ["proposal"]},
    {"id": "specs", "outputPath": "specs/**/*.md", "status": "blocked", "missingDeps": ["proposal"]},
    {"id": "tasks", "outputPath": "tasks.md", "status": "blocked", "missingDeps": ["specs"]}
  ]
}
```

建立 proposal、spec、design、tasks 後，status 顯示：

```text
isComplete: true
proposal: done
design: done
specs: done
tasks: done
```

靜態推論：

- Status 評估 schema DAG。
- 它回報 ready、blocked、done 與 missing dependency states。
- `applyRequires` 控制何時 change 可以進入 apply。

## `spectra instructions`

相關函式：

```text
instructions command       0x14004c3f0 - 0x14004e69b
```

靜態推論：

- 解析 schema 與 artifact。
- 回傳 artifact-specific instructions。
- 支援 `--skill <name>`。
- 可輸出 JSON 或 human-readable text。

字串中出現的相關 output fields：

```text
Artifact
Output
Description
Instruction
Dependencies
Unlocks
Template
ChangeState
Progress
Schema
```

相關 JSON field strings：

```text
changeName
changeDir
schemaName
contextFiles
progress
tasks
state
missingArtifacts
locale
instruction
worktreePath
preflight
artifactId
outputPath
description
context
rules
template
dependencies
unlocks
total
complete
remaining
```

相關錯誤字串：

```text
Unknown skill:
```

## `spectra validate`

相關函式：

```text
validate command           0x14006a680 - 0x14006c6d7
artifact validation core   0x1401174f0 - 0x140119614
```

觀察到的行為：

- `spectra validate <change> --json` 回傳 validation results array。
- 有效測試 change 回傳：

```json
[
  {
    "change": "inspect-cli",
    "valid": true,
    "errors": [],
    "warnings": []
  }
]
```

靜態推論：

- 驗證 changes 或 specs。
- 讀取 delta specs。
- 產生 `valid`、`errors`、`warnings`。
- 處理 no-delta-spec cases。

相關字串：

```text
Validation failed
No delta specs found
errors
warnings
valid
Delta spec validation failed:
```

## `spectra analyze`

相關函式：

```text
analyzer main              0x14012f9f0 - 0x1401367de
analyzer JSON summary      0x140140dd0 - 0x14014138d
```

觀察到的行為：

對 minimal test change，analyzer 回傳一個 warning：

```json
{
  "dimension": "Coverage",
  "severity": "Warning",
  "summary": "Requirement 'CLI probe artifact' has no matching task",
  "recommendation": "Add a task in tasks.md that references 'CLI probe artifact'"
}
```

Targeted valid-spec probes 產生以下 findings：

| Probe | `validate` | `analyze` result |
| --- | --- | --- |
| Clean proposal/spec/design/tasks | valid | 四個 dimensions 全部 clean。 |
| Proposal names capability but no spec exists | valid with warning `No delta specs found` | Coverage `Critical`: `Capability <name> has no corresponding spec file`. |
| Requirement text not referenced by any task | valid | Coverage `Warning`: `Requirement '<name>' has no matching task`. |
| Requirement contains `should` | valid | Ambiguity `Suggestion`: `Vague language 'should' found`. |
| Concrete scenario without `##### Example` | valid | 在 targeted probe 中為 clean。 |
| Abstract scenario without concrete examples | 不依賴 validation 的 rough probe | Ambiguity `Suggestion`: `Scenario '<name>' has no concrete examples`. |
| Requirement without `#### Scenario` | valid | Ambiguity `Warning`: `Requirement '<name>' has no scenarios`. |
| `###` design decision heading absent from tasks | valid | Consistency `Warning`: `Design topic '<keyword>' not referenced in tasks`. |
| `MODIFIED` capability without main spec | valid | Gaps `Warning`: `MODIFIED requirements reference capability '<cap>' but no main spec found`. |
| `MODIFIED` requirement absent from main spec | valid | Gaps `Warning`: `MODIFIED requirement '<name>' not found in main spec`. |

重要邊界：

- `analyze` 比 `validate` 寬鬆；早期 rough probe 中 invalidly indented delta specs 仍產生 analyzer findings，但 `validate` 會拒絕相同 specs。
- `conDesignNotInTasks` 不會因任意 prose mismatch 觸發。它由 `###` decision headings 觸發，並檢查 normalized heading keyword 是否出現在 `tasks.md`。
- `covDeltaValidation` 仍未被動態觀察到。目前測試過的 invalid delta files 不是被 analyzer coverage 視為沒有 usable delta spec，就是由 `validate` 回報。

靜態推論：

Analyzer dimensions：

```text
Coverage
Consistency
Ambiguity
Gaps
```

從 embedded message keys 推得的 finding categories：

```text
covMissingSpec
covMissingTask
covDeltaValidation
ambNoScenario
ambAbstractScenario
ambWeakLanguage
conDesignNotInTasks
gapNoMainSpec
gapModifiedNotFound
```

已動態確認的 finding keys 或 summaries：

```text
covMissingSpec
covMissingTask
ambNoScenario
ambAbstractScenario
ambWeakLanguage
conDesignNotInTasks
gapNoMainSpec
gapModifiedNotFound
```

從字串推論的檢查：

- Proposal capabilities 應有 matching specs。
- Requirements 應有 matching tasks。
- Delta specs 必須能 parse 並 validate。
- Scenarios 應有 concrete examples。
- Abstract scenario language 會被標記。
- Weak wording 會被標記。
- Design decisions 應由 tasks 覆蓋。
- Placeholder tokens 會被標記。

Analyzer strings 中 embedded 的 weak 或 placeholder terms：

```text
should
may
might
consider
possibly
TBD
TODO
???
TKTK
```

從字串推論的 Analyzer JSON fields：

```text
change_id
dimensions
findings
artifacts_analyzed
artifacts_missing
dimension
status
finding_count
severity
location
summary
recommendation
summary_msg
recommendation_msg
```

在測試案例中，觀察到的 JSON 不包含 total score 或 aggregate pass/fail field。

## `spectra drift`

相關函式候選：

```text
drift command/display      0x140053340 - 0x1400567a5
drift core helper          0x1405da860 - 0x1405db6cf
```

靜態推論：

Drift 使用 git commands 或 libgit2-backed data，檢查 change 是否 stale 或受到 external changes 影響。

觀察到的 JSON shape：

```text
change_id
created
last_commit
dimensions
broken_anchors
tasks_maybe_resolved
tasks_blocked_external
commits_since_created
total_score
severity
primary_recommendation
```

觀察到的 dimensions：

```text
Time
Structure
Tasks
Environment
```

觀察到的 time scoring：

| Created age | Time status | Time score | Total score | Severity | Recommendation |
| --- | --- | ---: | ---: | --- | --- |
| 0 days | `fresh (0d)` | 0 | 0 | `light` | `/spectra-apply <change>` |
| 8 days | `aging (8d)` | 1 | 1 | `light` | `/spectra-apply <change>` |
| 15 days | `aging (15d)` | 1 | 1 | `light` | `/spectra-apply <change>` |
| 31 days | `stale (31d)` | 2 | 2 | `light` | `/spectra-apply <change>` |
| 91 days | `abandoned (91d)` | 4 | 4 | `medium` | `/spectra-ingest <change>` |

觀察到的 structure/task status cases：

- 正常完整 change：`0/9 anchors broken`。
- 沒有 `design.md` 的 change：Structure status `design absent`，score 0。
- 沒有 `tasks.md` 的 change：Tasks status `no tasks.md`，score 0。
- 在 targeted probe 中，移除 proposal/spec directories 沒有產生 broken anchors。
- Design reference 指向缺失 CLI flag，例如 `` `--probe-mode` ``，會產生：

```json
{
  "anchor": "--probe-mode",
  "category": "CliFlag",
  "reason": "not in --help"
}
```

- 一個 broken CLI flag anchor 會產生 Structure score 3。加上 Time score 1，total score 為 4，severity 為 `medium`，recommendation 為 `/spectra-ingest <change>`。

觀察到的 environment behavior：

- 後續 Git commit 會增加 `commits_since_created` 與 Environment status，例如 `1 commits`。
- 觀察到的 JSON 中 Environment 為 `contributes_to_total: false`，所以不影響 `total_score`。
- 即使在 Git repository 中，所有 probes 的 `last_commit` 仍為 `null`。

觀察到的 task-collision behavior：

- `tasks_maybe_resolved` 會在 pending tasks 的 verb/target words 與 change `created` date 後的 commit subjects 匹配時觸發。
- 產生一個 maybe-done task 的例子：
  - Task: `Implement ProbeAdapter.` / commit subject: `Implement ProbeAdapter`
  - Task: `Update login cache.` / commit subject: `Update login cache handling`
- 觀察到的 probes 中，一個 maybe-done task 會讓 Tasks score +1。
- 在已測 path-reference variants 中，即使後續 commits 修改 task-referenced files，`tasks_blocked_external` 也沒有觸發。

相關字串：

```text
git
ls-files
log--since=
--pretty=format:COMMIT|%H|%at|%s
--name-only
.openspec.yaml
Drift Report
Broken anchors
Tasks blocked by external changes
Tasks possibly resolved elsewhere
Severity drift
```

推論的 drift dimensions：

- Broken anchors
- Tasks blocked by external changes
- Tasks possibly resolved elsewhere
- Staleness / commits since created

未解的 drift branches：

- File path、symbol、function anchors 會被計入 denominator，例如 `0/4 anchors broken`，但簡單 missing-file 與 missing-symbol probes 沒有讓它們出現在 `broken_anchors`。目前只有 CLI flag anchors 是已動態確認的 broken-anchor category。
- `tasks_blocked_external` 在所有已測 variants 中都維持空。它的條件比「pending task 提到 path 且後續 commit 修改該 path」更特定。

## `spectra park`

相關函式：

```text
park command               0x140042580 - 0x140042d50
park core                  0x140169ed0 - 0x14016ade6
db connection/migration    0x140161190 - 0x140163c97
```

觀察到的行為：

- 將 active change 從 `openspec/changes/<name>` 移出。
- 儲存在 `.git/spectra-app/changes/<name>`。
- 建立 `.git/spectra-app/spectra.db`。
- 在 SQLite 記錄 metadata。

觀察到的 SQLite schema：

```sql
CREATE TABLE parked_changes (
  change_id TEXT PRIMARY KEY,
  original_modified INTEGER,
  tasks_total INTEGER DEFAULT 0,
  tasks_done INTEGER DEFAULT 0,
  has_proposal INTEGER DEFAULT 0,
  has_tasks INTEGER DEFAULT 0,
  created_by TEXT,
  created_with TEXT
)
```

Probe 中觀察到的 row：

```text
change_id:         inspect-cli
tasks_total:       2
tasks_done:        0
has_proposal:      1
has_tasks:         1
created_by:        MomoChen <momochenisme@gmail.com>
created_with:      codex
```

靜態推論：

- Park 會檢查 change directory 是否存在。
- Park 會建立或開啟 `.git/spectra-app`。
- Park 使用 transaction。
- Park 將 metadata 記錄到 `parked_changes`。
- Park 將 change directory 移動或複製到 `.git/spectra-app/changes/<name>`。

相關字串：

```text
Change directory does not exist:
spectra-app
spectra_core::park
Failed to read park storage dir
COMMIT
ROLLBACK
Transaction dropped unexpectedly.
```

## `spectra unpark`

相關函式：

```text
unpark command             0x140041670 - 0x140042063
park core                  0x140169ed0 - 0x14016ade6
```

觀察到的行為：

- 將 `.git/spectra-app/changes/<name>` 移回 `openspec/changes/<name>`。
- 移除 SQLite parked record。

相關 SQL string：

```sql
DELETE FROM parked_changes WHERE change_id = ?1
```

相關字串：

```text
Unparked change:
unparked
Change '' is not parked
' is already active (not parked)
```

## `spectra list`

相關函式：

```text
park/list storage path     0x14003ea20 - 0x140040876
```

靜態推論：

- 從 `openspec/changes` 列出 active changes。
- 從 `.git/spectra-app/changes` 和/或 SQLite 列出 parked changes。
- 從 `openspec/specs` 列出 specs。
- 使用 `proposal.md` 是否存在來摘要 changes。

相關字串：

```text
No active changes.
Changes:
No parked changes.
Parked:
No specs.
Specs:
proposal.md
```

## `spectra show`

相關函式：

```text
show command               0x140066c60 - 0x14006917f
```

靜態推論：

- 輸出 proposal 與 delta specs 等 sections。
- 支援 JSON 與 human-readable output。
- 可顯示 change 或 spec。

相關字串：

```text
--- Proposal ---
--- Delta Specs ---
Change:
Schema
Created
proposal.md
tasks.md
design.md
specs
```

## `spectra task done`

相關函式：

```text
task done command          0x14005d600 - 0x1400603ce
```

觀察到的行為：

- `spectra task done 1 --change verify-cli --json` 回傳 JSON，包含 `change`、`status`、`task_desc`、`task_id`。
- `tasks.md` 的第一個 checkbox 從 `- [ ]` 變成 `- [x]`。
- 在非 Git project 中，不會建立 touched-file metadata。
- 在乾淨 Git worktree 中，除了 `tasks.md` checkbox mutation 外，不會建立 touched-file metadata。
- 在 Git worktree 執行 `task done` 前若已有 modified 或 staged files，command 會建立 `.spectra/touched/<change>.json`。

Touched-file behavior：

```json
{
  "change": "touch-case",
  "touched": [
    {
      "task_id": "1",
      "task_desc": "1. Implement Touch behavior in src/touched_target.txt and track touched files.",
      "files": [
        "src/touched_target.txt"
      ]
    }
  ]
}
```

額外 probes 顯示：

- Unstaged 與 staged file modifications 都會被記錄。
- 多個 modified files 會記錄在同一個 `files` array。
- `src/` 之外的檔案，例如 `README.md`，也可以被記錄。
- `task done` 本身造成的 `tasks.md` mutation 不會被記成 touched file。
- 建立 touched-file 不需要先執行 `spectra in-progress add <change>`。

靜態推論：

- 解析 change。
- 讀取 `tasks.md`。
- 將 task checkbox 標成 done。
- 使用 Git worktree state 決定是否寫入 touched-file metadata。

相關字串：

```text
tasks.md
Task  marked as done:
tasks.md not found for change ''
Failed to read tasks.md:
Failed to write tasks.md:
.spectra/touched.json
change
touched
task_id
task_desc
files
```

Embedded serde strings 提到：

```text
struct TouchedEntry
struct TouchedTracking
```

可信度註記：

- Checkbox mutation 是已驗證行為。
- Git worktree 已含 modified 或 staged files 時，touched-file recording 是已驗證行為。在測試情境中，非 Git 或 clean-Git cases 不會建立。

## `spectra archive`

相關函式：

```text
archive command wrapper    0x140037220 - 0x1400388a8
archive core               0x140144c20 - 0x14014781d
```

觀察到的行為：

- `spectra archive verify-cli --yes --mark-tasks-complete` 將 active change archive 為 `openspec/changes/archive/2026-05-19-verify-cli`。
- 它移除 `openspec/changes/verify-cli`。
- 它將 delta spec 套用到 `openspec/specs/verify-cli/spec.md`。
- 它印出 `Specs applied: verify-cli (added: 1, modified: 0, removed: 0, renamed: 0)`。
- 它印出 `Snapshot created for unarchive support.`。
- 產生的 main spec 含有 placeholder Purpose section：`TBD - created by archiving change 'verify-cli'. Update Purpose after archive.`

靜態推論：

- 解析 active change。
- 可選擇在 archive 前 validate。
- 可 mark tasks complete。
- 將 delta specs 套用到 `openspec/specs`。
- 將 completed changes 移到 archive。
- 建立 snapshot 以支援 unarchive。
- 使用 date prefix format `%Y-%m-%d-`。

相關字串：

```text
Specs applied:  (added: , modified: , removed: , renamed: )
Snapshot created for unarchive support.
Validation failed for:
created_specs.json
spec.md
tasks.md
- [ ]
- [x]
%Y-%m-%d-
```

## `spectra config`

相關函式：

```text
config command             0x1400582b0 - 0x14005991a
config core                0x1405d65c0 - 0x1405d8446
```

觀察到的行為：

- Global config path：

```text
C:\Users\momoc\AppData\Roaming\openspec\config.yaml
```

- `spectra config list --json` 在目前環境回傳 `{}`。

靜態推論：

- 讀寫 global OpenSpec config。
- 支援透過 `EDITOR` 或 `VISUAL` edit。
- 可將 values 視為 strings。
- 可 reset config。

相關字串：

```text
Cannot determine config path
Cannot determine config directory
# OpenSpec global config
No configuration set.
Config reset.
EDITOR
VISUAL
Editor exited with error.
Failed to open editor '':
```

## `spectra schema`

相關函式：

```text
schema management command  0x140045960 - 0x140049d77
```

靜態推論：

- 支援 `which`、`validate`、`fork`、`init`。
- 可 resolve built-in、project 或 user schemas。
- 使用 `schema.yaml`。

觀察到的行為：

- `spectra schema which spec-driven --json --all` 將 default schema resolve 為 `built-in`，path 為 `(embedded in binary)`。
- 位於 `openspec/schemas/mini-flow/schema.yaml` 的 project schema resolve 為 `project`。
- 位於 `%APPDATA%\openspec\schemas\<name>\schema.yaml` 的真實 user schema resolve 為 `user`。
- 同名 project schemas 優先於 user schemas。
- 即使 `schema validate` 後續會拒絕 project schema，`spectra schemas --json` 仍會列出該 schema。
- `schema validate mini-flow --json --verbose` 拒絕缺少 `generates` 的 artifact entry：

```text
Schema parse error: artifacts[0]: missing field `generates`
```

- `spectra schema fork spec-driven forked-flow --json` 建立 `openspec/schemas/forked-flow/schema.yaml`。
- Fork 後，`spectra schema which forked-flow --json --all` resolve 到 project schema path。

從 forked file 觀察到的 built-in schema fields 包含：

```text
name
version
description
artifacts
generates
template
instruction
requires
unlocks
```

Resolution confidence：

- Built-in resolution 已驗證。
- Project-local resolution 已驗證。
- User-level resolution 已在 `%APPDATA%\openspec\schemas` 驗證。
- Project-over-user precedence 已驗證。
- 測試過但不會 resolve 的 user-schema locations：`%APPDATA%\Spectra\schemas`、`%APPDATA%\spectra\schemas`、`%USERPROFILE%\.spectra\schemas`、`%USERPROFILE%\.openspec\schemas`。

相關字串：

```text
Schema validation failed:
Schema '' already exists. Use --force to overwrite.
Forked ''
Created schema '' at
schema.yaml
templates
Custom schema:
project
openspec/schemas
user
(embedded in binary)
built-in
```

## `spectra schemas` 與 `spectra templates`

相關函式：

```text
schemas command            0x140086380 - 0x140087c97
```

觀察到的行為：

- `schemas --json` 列出 built-in `spec-driven` schema。
- `templates --json` 列出 template names：

```text
proposal.md
spec.md
design.md
tasks.md
```

靜態推論：

- 這些 commands 主要查詢 schema/template registries。
- Built-in schema 與 templates embedded in binary。

額外觀察到的 list behavior：

- Active changes 存在時，`list --json` 回傳 top-level `changes` array。
- `list --changes --json` 回傳相同的 change shape：

```text
name
status
summary
totalTasks
completedTasks
```

- `list --specs --json` 回傳 top-level `specs` array。
- `in-progress add <change>` 會讓 listed change 的 `status` 變成 `in-progress`。
- 此 build 的 `in-progress` command 只暴露 `add` subcommand；嘗試 `list`、`remove`、`clear` 會得到 clap errors。
- `demo` 不支援 `--json`；`spectra demo --json` 回傳 clap unexpected-argument error。

相關字串：

```text
Available schemas:
schema.yaml
artifacts
Default OpenSpec workflow - proposal -> specs -> design -> tasks
```

## `spectra search`

此平台觀察到的行為：

```json
{"error":"vector_not_compiled","results":[]}
```

使用者可見錯誤表示目前平台沒有編譯 vector search：

```text
Vector search is not available on this platform (requires Apple M-series).
vector_not_compiled
```

靜態推論：

- Search 是 vector semantic search command。
- Windows x64 build 上，vector search support 不可用。

## Database and Storage

觀察到的 storage locations：

```text
openspec/changes/<change-name>
openspec/specs
openspec/config.yaml
.spectra.yaml
.agents/skills
.git/spectra-app/changes/<change-name>
.git/spectra-app/spectra.db
```

SQLite 用於 parked change metadata。Binary 連結了 `rusqlite` 與 `libsqlite3-sys`。

Touched-file storage note：

- Binary 包含 `.spectra/touched.json` 與 touched-tracking serde strings。
- Dynamic probes 在 `task done` 執行於已有 modified 或 staged files 的 Git worktree 時，建立 `.spectra/touched/<change>.json`，不是 `.spectra/touched.json`。
- Clean Git 與 non-Git probes 不會建立 touched metadata。

Database migration strings 提到：

```text
.spectra/spectra.db
PRAGMA .table_info()
spectra_core::db::connection
Failed to acquire migration lock:
spectra.db.bak
ATTACH DATABASE ?1 AS legacy
DETACH DATABASE legacy
INSERT OR IGNORE INTO
```

靜態推論：

- 可能支援從 legacy `.spectra/spectra.db` path migration 到 `.git/spectra-app/spectra.db`。
- Migration 由 lock file 或 lock mechanism 保護。

## Embedded Workflow and Skill Content

Binary embedded 了大量 skill text 與 workflow instructions。這解釋了為何 `spectra init` 不需 external template files 就能產生 `.agents/skills/.../SKILL.md`。

觀察到的 embedded skills 包含：

```text
spectra-apply
spectra-archive
spectra-ask
spectra-audit
spectra-commit
spectra-debug
spectra-discuss
spectra-drift
spectra-ingest
spectra-propose
```

Binary 也包含 preflight、analyze、drift、apply、archive 與 artifact generation 的 workflow text。

## 目前可信度

高可信度：

- Binary format 與 Rust/clap origin。
- Command surface。
- `init` file generation side effects。
- `new change` directory side effects。
- `new artifact` output paths 與 basic validation。
- `status` DAG behavior。
- `validate` JSON structure。
- `analyze` dimensions 與 confirmed finding categories：missing spec、missing task、weak language、abstract scenario。
- `analyze` consistency 與 gap findings，包含 design decision headings 與 modified-spec checks。
- `drift` JSON shape 與 time-based scoring thresholds，已觀察 0、8、15、31、91 days。
- `drift` maybe-done task detection，根據 pending task text 與後續 commit subjects。
- `drift` broken CLI flag anchor behavior 與 structure score contribution。
- `park/unpark` storage location 與 SQLite schema。
- `archive` 會將 completed change 移入 date-prefixed archive directory，並將 delta specs 套用到 `openspec/specs`。
- Global config path。
- Windows x64 上 vector search unavailable。
- `task done` 會在 `tasks.md` 標記 checkbox complete。
- `task done` 在 Git worktree 執行前已有 modified 或 staged files 時，會寫入 `.spectra/touched/<change>.json`。
- Built-in、project-local、user-level schema resolution，包含 project-over-user precedence。

中可信度：

- 部分 command steps 的 exact internal ordering。
- Analyzer rule implementation details，特別是 embedded 但未觀察到的 `covDeltaValidation`。
- Drift file-path、symbol、function anchor resolution。
- Drift `tasks_blocked_external` heuristic。

低可信度：

- Embedded serde/source path strings 之外的 exact Rust type names。
- Full function names 與 call graph，因為 PDB 不存在。
- 每個 error path 的 exact branch-level behavior。

## 後續反向工程目標

建議下一批目標，依優先順序：

1. 完整重建 `new artifact` pseudo-code。
2. 完整重建 `park` 與 `unpark` pseudo-code。
3. 從 `crates/spectra-core/src/analyzer.rs` candidates 重建 analyzer rules。
4. 重建 archive spec-application flow。
5. 重建 schema loader 與 template resolver。
6. 重建 preflight path validation rules。

Unclear-branch pass 後更新的 unresolved targets：

1. 觸發並重建 `tasks_blocked_external`。
2. 識別 `design.md` 中 file-path、symbol、function anchors 的 exact syntax 與 resolver rules。
3. 從 analyzer 觸發 `covDeltaValidation`，或證明它對 filesystem-present malformed specs 目前不可達。
4. 重建 `REMOVED` 與 `RENAMED` operations 的 archive spec-merge behavior。
5. 重建 legacy `.spectra/spectra.db` 的 database migration。

## 註記

因為這是 Rust release binary，直接線性閱讀 raw assembly listing 會非常吵雜。實用方法是：

1. 使用 strings 找出 feature-specific constants。
2. 將 string references map 到 functions。
3. 使用 `.pdata` 還原 function ranges。
4. 只反組譯 candidate functions。
5. 在臨時 Spectra project 中驗證推論行為。

這比從 entry point 開始線性讀 raw assembly 更能可靠理解 command-level behavior。

## 可信度審查紀錄

2026-05-19 pass：

- 使用 `spectra --version`、PE parsing 與 SHA-256 重新檢查 binary identity。
- 針對 key patterns 重建 string-to-function cross-reference mapping：
  - artifact validation strings
  - analyzer message keys
  - park/unpark SQL
  - source path strings
  - vector search platform error
- 在乾淨臨時專案中重放 command behavior。
- 在簡單 probe 沒建立 metadata 後，修正第一版 `task done` touched-file statement；後續 deep probe 找出 Git modified-file condition，並重新升級為 observed behavior。
- 確認 archive output 與 generated files 後，將 archive behavior 從 static inference 升級為 observed behavior。
- Analyzer rule internals 保持 medium confidence，因為 probe 確認了數個 findings 與 dimensions，但不是每個 branch 或 severity rule。

2026-05-19 deep probe pass：

- 使用 valid delta specs 重跑 analyzer probes，以分離 analyzer behavior 與 validation errors。
- 確認 missing specs、missing tasks、weak language、abstract scenarios 的 specific analyzer findings。
- 量測 drift time-score thresholds，並確認 Environment commit counts 不會 contribute to observed output 中的 `total_score`。
- 透過刪除 top-level artifacts 測試 broken-anchor hypotheses；沒有觸發 `broken_anchors`，因此該 branch 仍未解。
- 確認 project schema resolution 與 `schema fork` output。
- 將 touched-file tracking 從 inferred 升級為 observed behavior，並記錄會建立 `.spectra/touched/<change>.json` 的 Git-worktree conditions。

2026-05-19 unclear-branch pass：

- 解析 `conDesignNotInTasks`：analyzer 會 cross-check `###` design decision headings 與 `tasks.md`。
- 確認 `ambNoScenario`、`gapNoMainSpec`、`gapModifiedNotFound`。
- 確認 `tasks_maybe_resolved`：pending task verb/target words 與後續 Git commit subjects 匹配時觸發。
- 確認 broken CLI flag anchors 與其 Structure score contribution。
- 測試多個 `tasks_blocked_external` hypotheses；皆未觸發，因此這仍是主要 drift gap。
- 驗證真實 user schema path `%APPDATA%\openspec\schemas` 與 project-over-user precedence，之後移除 temporary user schema。
- 新增 static xref notes，將 analyzer、drift serializer、schema resolution strings 連回已識別 function ranges。


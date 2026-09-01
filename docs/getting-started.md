# Speclink Local Repo Getting Started

[繁體中文](getting-started.zh-TW.md) · **English**

Follow this document through one full Local Repo loop: **init → propose → apply → checks → archive**. Specs live in the repo's `openspec/`, collaborate through Git, and need no server.

Every step states its expected output. What you see should match what this document shows. When it does not match, something is wrong. Stop and find the difference.

This document covers the happy path only. Optional branches all link out to the [Complete SDD Workflow](workflow.md).

## Before you start / 開始前

This example assumes the requirement is already clear: "add CSV export". If you are still comparing directions or need to reach a decision, use `discuss` first — do not record every question as a discussion.

Agent commands come in two invocation literals: `/speclink-*` in Claude and `$speclink-*` in Codex. In Codex the `$` prefix invokes a skill explicitly. You can also type `/skills` and pick the same skill from the list. Both routes work.

Both literals are listed below; pick one. Blocks marked as shell run the CLI directly. For what the skill, CLI, and Host layers each own, see [Call layers](workflow.md#call-layers--呼叫層級) in the workflow document.

## 1. Install / 安裝

Install the CLI, one of:

```bash
# Install script (macOS/Linux)
curl -fsSL https://raw.githubusercontent.com/MomoChenisMe/speclink/main/scripts/install.sh | sh

# Install script (Windows PowerShell)
irm https://raw.githubusercontent.com/MomoChenisMe/speclink/main/scripts/install.ps1 | iex

# Homebrew (macOS/Linux)
brew install MomoChenisMe/tap/speclink
```

If you want the graphical interface, [Releases](https://github.com/MomoChenisMe/speclink/releases/latest) carries desktop installers for all three platforms, each bundling a matching CLI. Read this if you already installed the CLI with a script above: the desktop app replaces that binary. Read [the install section in the README](../README.en.md#install--安裝) before you pick one. Check the install:

```bash
speclink --version
```

**Expected output**: one version line, shaped like `speclink 0.1.0 (arm64, engine v1.x.y)`. `speclink --help` lists the current commands, including `init`, `status`, `validate`, `analyze`, `drift`, `archive`, `discuss`, `review`, and `verify`.

## 2. Initialize / 初始化

Change into the repo you want to adopt Speclink in:

```bash
speclink init --tools claude,codex
```

**Expected output**:

```text
✓ Initialized at /path/to/your-repo/openspec
Generated files for: claude, codex
```

This creates `openspec/` and `.speclink.yaml`, generates the skill files for the Hosts you selected (`.claude/skills/`, `.agents/skills/`), and adds `.speclink/` to `.gitignore`. No instruction file is written — `CLAUDE.md` and `AGENTS.md` are yours, and workflow routing rides the skills' own descriptions. It does not create `.speclink/` itself; that directory appears later, when there is local working data to store.

**This is what Local mode produces.** The `openspec/` structure follows the OpenSpec conventions on purpose. You can read and edit it directly, and you can move an existing OpenSpec tree in:

```text
openspec/
├── config.yaml              workflow policy (locale, tdd, audit, worktree)
├── specs/<capability>/spec.md   canonical specs, one file per capability
├── changes/<name>/           active changes (proposal, design, tasks, specs delta)
├── changes/archive/          archived changes
└── discussions/              discussion records (added by Speclink)
```

Every file is plain Markdown or YAML. There is no database and no proprietary format. You can read and edit the files without Speclink, and Git shows a diff for every spec change. Speclink adds only two things: `discussions/`, and an `.openspec.yaml` inside each change directory that holds lifecycle metadata such as the start time and the source discussion.

This compatibility covers Local mode only. After you attach to a remote, the Store holds the canonical specs. The machine keeps a read-only projection at `.speclink/context/`, not a writable file tree.

Check that the starting point is clean:

```bash
speclink list
speclink validate --specs --all --strict
```

**Expected output**: `list` prints `No active changes.`, and with no canonical specs yet `validate` prints **nothing at all** and exits 0 — no output is the pass.

If the repo already has substantial code but no canonical specs, run `/speclink-baseline` (`$speclink-baseline` in Codex) to derive specs from current behavior before opening a new change.

## 3. Propose / 提案

In Claude:

```text
/speclink-propose add-csv-export
```

In Codex:

```text
$speclink-propose add-csv-export
```

The Agent creates the change, reads the schema instructions one by one, and completes the artifacts along the `applyRequires` chain. Check the DAG at any point:

```bash
speclink status --change add-csv-export --json
```

**Expected output**: right after creation only the proposal is writable and the rest are blocked —

```text
proposal → ready
design   → blocked
specs    → blocked
tasks    → blocked
```

Once they are filled in:

```text
proposal → done
design   → ready
specs    → done
tasks    → done
```

Notice that `design` stops at `ready` rather than `done`, and `isComplete` is still `false` — **that is normal**. Design is a conditional artifact, needed for cross-module work or significant technical decisions, and `applyRequires` only demands `tasks`. This is why "not every change produces the same four files".

To drive the CLI directly, the underlying flow is:

```bash
speclink new change add-csv-export
speclink instructions proposal --change add-csv-export --json
speclink new artifact proposal --change add-csv-export --stdin
```

**Expected output** (first line):

```text
✓ Created change: add-csv-export
  Path: /path/to/your-repo/openspec/changes/add-csv-export
  Schema: spec-driven
```

The last command reads complete Markdown matching the instructions template from stdin; specs use `speclink new artifact spec <capability> --change add-csv-export --stdin`. Driving the CLI directly suits people who already know the artifact contract; otherwise use the skill.

When the source is a concluded discussion, use `/speclink-propose --from-discussion <slug>` instead. Other promotion and fold-in routes are in [Discussion outcomes](workflow.md#discussion-outcomes--討論結論分流).

## 4. Apply / 實作

Once the artifacts are complete, in Claude:

```text
/speclink-apply add-csv-export
```

In Codex:

```text
$speclink-apply add-csv-export
```

The Agent reads the proposal, specs, design (if present), and tasks, then implements and verifies them one at a time. The underlying progress entries are:

```bash
speclink instructions apply --change add-csv-export --json
speclink task done --change add-csv-export 1
```

**Expected output**: `task done` reports exactly which item it checked off —

```text
✓ Task 1 marked as done: 1.1 Serialize report rows to CSV
```

Only check an item off once its behavior, implementation contract, and verification target all pass. If you checked the wrong one or rolled the implementation back, use `speclink task undone --change add-csv-export 1` rather than editing `tasks.md` by hand.

After everything is checked:

```bash
speclink instructions apply --change add-csv-export --json
speclink list
```

**Expected output**: the instructions `state` becomes `all_done`, and `list` shows a full count —

```text
Changes:
  • add-csv-export [2/2] — Reports can only be read insid…
```

## 5. Check / 檢查

**Most of the time you do not run this step yourself.** The `propose`, `apply`, and `ingest` skills all run `analyze` for you:

- `propose` and `ingest`: they run it before they finish, repair every Critical finding, write the artifacts, then run `validate` once
- `apply`: it runs `analyze` before implementation starts, and stops to ask you when it finds a Critical

Run these two commands yourself in three cases. You want a quick look outside the flow. The Agent did not run them. Or you gate a CI job on them.

```bash
speclink analyze add-csv-export --json
speclink validate add-csv-export
```

**Expected output**: `analyze` reports per dimension and `validate` prints one result line —

```text
Coverage    → 1 issue(s) found
Consistency → Skipped (insufficient artifacts)
Ambiguity   → 1 issue(s) found
Gaps        → Clean

✓ add-csv-export — valid
```

Both findings show up on almost every first loop, and they mean:

- Coverage's `Requirement 'X' has no matching task` (Warning) — a spec states a requirement that no task covers.
- Ambiguity's `Scenario 'X' has no concrete examples` (Suggestion) — the scenario is prose with no concrete GIVEN/WHEN/THEN values.

Consistency shows `Skipped` because there is no design. That dimension compares artifacts against each other, so it does not judge when one is absent.

Warnings and Suggestions do not stop you, but both deserve a decision. Repair a Critical in the artifact before you implement.

Then run your own project's tests, lint, build, or manual acceptance. `validate` and `analyze` only check artifacts and **do not substitute for code correctness**.

Two optional quality stations cover the implementation side — `/speclink-review` for code craft and `/speclink-verify` for spec compliance, or `/speclink-quality` to run both. Skipping them on a low-risk first loop is a legitimate choice; their criteria, stamping sequence, and must-fix rules are in the [Complete SDD Workflow](workflow.md).

## 6. Archive / 封存

Once every task is complete, the artifacts are valid, delta assumptions are current, and any quality station you ran is closed, in Claude:

```text
/speclink-archive add-csv-export
```

In Codex:

```text
$speclink-archive add-csv-export
```

Or directly:

```bash
speclink archive add-csv-export -y
```

**Expected output**:

```text
✓ Archived: add-csv-export → <date>-add-csv-export
Specs applied: csv-export (added: 1, modified: 0, removed: 0, renamed: 0)
Snapshot created for unarchive support.
```

Archiving merges the delta specs into the canonical specs and moves the change into `openspec/changes/archive/`. Afterwards `speclink list` returns to `No active changes.` and `openspec/specs/csv-export/` appears. That is where this loop landed.

Do not reach for `--mark-tasks-complete` or `--no-validate` to route around unfinished work.

## What was created / 產物位置

| Path / 路徑 | Meaning / 意義 |
| --- | --- |
| `openspec/specs/<capability>/spec.md` | Canonical specs — the truth about current behavior |
| `openspec/changes/<name>/` | An active change and the artifacts its schema requires |
| `openspec/changes/archive/` | Audit records for archived changes |
| `openspec/discussions/` | Discussion records, created only when a decision is needed |
| `openspec/config.yaml` | Workflow policy, context, rules, locale, TDD/audit |
| `.speclink.yaml` | Workspace binding and local tool integration |
| `.speclink/` | The gitignored Context Projection and working data |

## Leave the happy path / 離開主路徑

- Requirements still fuzzy: start with `discuss`; if you cannot name what to change, use `improve`.
- A discussion conclusion should scaffold a change or fold into an existing one: see [Discussion outcomes](workflow.md#discussion-outcomes--討論結論分流).
- Requirements changed mid-implementation: `/speclink-ingest <change>`.
- Resuming a paused change: run `/speclink-drift <change>` first.
- Pushing several changes in parallel: `/speclink-apply-with-worktree`, closing with `/speclink-worktree-merge`.
- Using a shared Remote Store instead of a local repo: see [Remote Getting Started](remote-getting-started.md).
- Deciding whether a capability works today: check [Project Capability Status](product-status.md) rather than inferring delivery from the architecture blueprint.

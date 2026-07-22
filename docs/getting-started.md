# Speclink Local Repo Getting Started

[繁體中文](getting-started.zh-TW.md) · **English**

This guide uses the currently implemented Local Repo path for one complete loop: **init → propose → apply → checks → archive**. Specs live in the repository's `openspec/` directory and collaboration uses Git; no server is required. See [Product Capability Status](product-status.md) for complete Remote/Server status.

## Before you start / 開始前

The example assumes a clear requirement: “add CSV export.” If you still need to compare directions, reach a decision, or preserve trade-offs, use `discuss` through the [complete SDD workflow](workflow.md). Do not turn every question into a discussion record.

Speclink has three invocation layers:

| Layer / 層級 | This guide uses / 本教學用法 | Responsibility / 責任 |
| --- | --- | --- |
| Claude skill | `/speclink-propose add-csv-export` | The Agent follows workflow knowledge, reads context, creates artifacts, validates, and asks when needed. |
| Codex skill | `$speclink-propose add-csv-export` | Produces the same Speclink artifacts using Codex `$skill` syntax. |
| Direct CLI | `speclink status --change add-csv-export --json` | CLI/Host is the execution engine for changes, artifact DAGs, tasks, and lifecycle; it does not decide requirements for you. |

Choose one Agent syntax for your Host. CLI blocks can be run directly in a shell.

## 1. Install / 安裝

Install from the Speclink source repository with a stable Rust toolchain:

```bash
cargo install --path crates/speclink-cli
speclink --version
```

`speclink --help` should list the current `init`, `status`, `validate`, `analyze`, `drift`, `archive`, and `discuss` commands.

## 2. Initialize / 初始化

Move into the repository adopting Speclink:

```bash
speclink init --tools claude,codex
```

This creates `openspec/`, `.speclink.yaml`, gitignored `.speclink/` work data, and skills for the selected Hosts. Inspect the initial state:

```bash
speclink list
speclink validate --specs --all --strict
```

If a substantial existing codebase has no canonical specs, first use Claude `/speclink-onboard` or Codex `$speclink-onboard` to document current behavior, then create a new change.

## 3. Propose / 提案

In Claude:

```text
/speclink-propose add-csv-export
```

In Codex:

```text
$speclink-propose add-csv-export
```

The Agent creates the change, reads schema instructions, and completes artifacts on the `applyRequires` dependency chain. A common spec-driven change has a proposal, delta specs, and tasks. Design is created only when its condition applies, such as cross-module work or an important technical decision; **design is conditional, not one of four fixed artifacts guaranteed for every change.**

Inspect the DAG at any time:

```bash
speclink status --change add-csv-export --json
```

Advanced users can work directly with the CLI primitives:

```bash
speclink new change add-csv-export
speclink instructions proposal --change add-csv-export --json
speclink new artifact proposal --change add-csv-export --stdin
```

The last command reads complete Markdown matching the instructions template from stdin. Rerun `status` and create only artifacts that are ready and required by the schema. Specs use `speclink new artifact spec <capability> --change add-csv-export --stdin`. Prefer the Agent skill unless you already understand the artifact contract.

For a concluded discussion, use Claude `/speclink-propose --from-discussion <slug>` or Codex `$speclink-propose --from-discussion <slug>`. The workflow guide covers fast scaffolding and reflecting a conclusion into an existing change.

## 4. Apply / 實作

After artifacts are complete, in Claude:

```text
/speclink-apply add-csv-export
```

In Codex:

```text
$speclink-apply add-csv-export
```

The Agent reads proposal/specs/design when present/tasks, then implements and verifies each task. The engine-level progress entries are:

```bash
speclink instructions apply --change add-csv-export --json
speclink task done --change add-csv-export 1
```

Only mark a task done after its behavior, implementation contract, and verification target all pass. If implementation is rolled back or a task was checked by mistake:

```bash
speclink task undone --change add-csv-export 1
```

Proceed only when apply instructions return `state: all_done`.

## 5. Check / 檢查

Check artifact coherence and structure:

```bash
speclink analyze add-csv-export --json
speclink validate add-csv-export
```

Then run the project's own tests, lint, build, and manual acceptance as applicable. Validate/analyze checks artifacts and does not replace code correctness.

The engine contains a verify workflow asset, but this repository currently has no callable generated `/speclink-verify` or `$speclink-verify`. Do not use either as a guide command. Verify implementation through project tests, Requirement/Scenario review, and `task done` evidence. See the Verify and task evidence row in product-status for the complete limitation.

## 6. Archive / 封存

After every task is complete, artifacts are valid, delta assumptions are current, and implementation checks pass, in Claude:

```text
/speclink-archive add-csv-export
```

In Codex:

```text
$speclink-archive add-csv-export
```

Or run the CLI directly:

```bash
speclink archive add-csv-export -y
```

Archive merges delta specs into canonical specs and moves the change to `openspec/changes/archive/`. Do not use `--mark-tasks-complete` or `--no-validate` to bypass unfinished work.

## What was created / 產物位置

| Path / 路徑 | Meaning / 意義 |
| --- | --- |
| `openspec/specs/<capability>/spec.md` | canonical specs: current behavioral truth |
| `openspec/changes/<name>/` | active change and the artifacts required by its schema |
| `openspec/changes/archive/` | audit record of archived changes |
| `openspec/discussions/` | discussion documents created only when a decision is needed |
| `openspec/config.yaml` | workflow policy, context, rules, locale, TDD/audit |
| `.speclink.yaml` | workspace binding and local tool integration |
| `.speclink/` | gitignored Context Projection, touched/evidence, and other work data |

## Leave the happy path / 離開主路徑

- Requirements are fuzzy: use `discuss` first.
- A discussion should fast-scaffold a change or update an existing change: see [Discussion outcomes](workflow.md#discussion-outcomes--討論結論分流).
- Requirements change during implementation: `$speclink-ingest <change>` (Claude: `/speclink-ingest`).
- Work resumes after a pause: `$speclink-drift <change>` first (Claude: `/speclink-drift`).
- You need to know whether Server, Desktop Remote Workspace, or Agent tools are usable today: read [Product Capability Status](product-status.md) instead of inferring current delivery from the architecture blueprint.

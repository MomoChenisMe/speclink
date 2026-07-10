# Getting Started

> 繁體中文版:[getting-started.zh-TW.md](getting-started.zh-TW.md)

> **Status:** This guide covers the currently implemented local-repository workflow. The target local/remote platform architecture and delivery phases are defined by [the platform architecture blueprint](platform-architecture.zh-TW.md).

This walkthrough runs one full spec-driven development (SDD) loop in a purely
local project: **init → discuss → propose → apply → verify → archive**.

Speclink is designed to be driven by an AI agent (Claude Code or Codex) through
the generated `/speclink-*` skills, with the `speclink` CLI as the engine
underneath. Steps below show both the skill to invoke and the CLI at work.

## 0. Install

Build from source (Rust toolchain required):

```
cargo install --path crates/speclink-cli
speclink --version
```

Expected output: `speclink 0.1.0 (x64)` (architecture suffix varies).

## 1. init — set up the project

```
speclink init
```

Expected output:

```
✓ Initialized at <your-project>\openspec
Generated files for: claude
```

This creates the `openspec/` spec directory (`specs/`, `changes/archive/`,
`config.yaml`), the `.speclink.yaml` app config, a `.gitignore` entry for
`.speclink/`, and the AI tool files (`CLAUDE.md` plus the `/speclink-*`
skills). Installed AI tools are auto-detected; pass `--tools claude,codex` to
choose explicitly.

## 2. discuss — optional, for fuzzy requirements

In your agent, run `/speclink-discuss add csv export`. The agent records the
conversation as a durable document via the CLI:

```
speclink discuss new "add csv export"     → ✓ Created discussion: add-csv-export
speclink discuss add-round <slug> --stdin → ✓ Recorded round 1 (interview) …
speclink discuss conclude <slug> --stdin  → ✓ Concluded discussion 'add-csv-export'
```

The document lives at `openspec/discussions/add-csv-export.md` and accumulates
rounds. Skip this step entirely when requirements are already clear.

## 3. propose — plan the change

Run `/speclink-propose add-csv-export` (or `--from-discussion add-csv-export`
to seed it from the concluded discussion). The agent creates the change and
its four artifacts:

```
speclink new change add-csv-export --agent claude
speclink new artifact proposal --change add-csv-export --stdin
speclink new artifact spec csv-export --change add-csv-export --stdin
speclink new artifact design --change add-csv-export --stdin
speclink new artifact tasks --change add-csv-export --stdin
```

Check progress at any time:

```
speclink status --change add-csv-export
```

Expected output: the artifact DAG with `✓ done` / `○ ready` / `✗ blocked`
markers, ending in `✓ All artifacts complete` once all four exist.

## 4. apply — implement the tasks

Run `/speclink-apply add-csv-export`. The agent reads the artifacts, works
through `tasks.md` checkbox by checkbox, and records each completion:

```
speclink task done 1 --change add-csv-export
→ ✓ Task 1 marked as done: <task description>
```

`speclink instructions apply --change add-csv-export --json` is what the agent
uses to see context files, progress, and remaining tasks (state becomes
`all_done` when every checkbox is checked).

## 5. verify — check implementation against artifacts

Run `/speclink-verify add-csv-export`. The agent compares the implementation
with the spec deltas and the design contract. Structural health checks are
also available directly:

```
speclink validate add-csv-export   → ✓ add-csv-export — valid
speclink analyze add-csv-export    → four-dimension findings report
```

## 6. archive — land the change

Run `/speclink-archive add-csv-export`, or directly:

```
speclink archive add-csv-export -y
```

Expected output:

```
✓ Archived: add-csv-export → 2026-07-04-add-csv-export
Specs applied: csv-export (added: 1, modified: 0, removed: 0, renamed: 0)
```

The delta spec is merged into the canonical `openspec/specs/csv-export/spec.md`,
the change directory moves to `openspec/changes/archive/`, and a linked
discussion (if this was the last change promoted from it) is co-archived.

## Where things live

| Path | What it is |
| --- | --- |
| `openspec/specs/<cap>/spec.md` | canonical specs (current truth) |
| `openspec/changes/<name>/` | active change proposals |
| `openspec/changes/archive/` | archived changes |
| `openspec/discussions/` | discussion documents |
| `openspec/config.yaml` | workflow configuration |
| `.speclink.yaml` | app configuration (host-side) |
| `.speclink/` | work data (gitignored) |

For the target Engine, TeamStore, Server, and UI architecture, see the
[platform architecture blueprint](platform-architecture.zh-TW.md).

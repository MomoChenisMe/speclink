<p align="center">
  <img src="docs/assets/brand/transparent/speclink-logo-horizontal.png" alt="Speclink" width="440" />
</p>

<p align="center">
  <b>Spec-Driven Development (SDD) platform</b> — one Rust engine, many frontends.
</p>

<p align="center">
  <a href="README.md">繁體中文</a> · <b>English</b>
</p>

---

Speclink treats the **spec** as the primary source of truth: requirements are written as structured specs first, and code implements against them. At its core is an SDD engine rewritten in Rust — modeled on [Spectra](https://github.com/kaochenlong/Spectra) 2.3.1, achieving **byte-level parity** with its CLI behavior and output, then deliberately extended in a handful of places. The same engine is embedded by a CLI, a desktop app, and a Node SDK, and can back its spec truth with either the local filesystem or a team system.

- **Implementation**: Rust (engine) + TypeScript／React (desktop frontend)
- **Compatible with**: Claude Code (`.claude/skills/`) and Codex (`.agents/skills/` + `AGENTS.md`)
- **Workflow**: `discuss? → propose → apply ⇄ ingest → verify? → archive`
- **License**: MIT

> Speclink's machine-readable output (`--json`), human-facing interactive output (with ANSI color), skill content, and workflow logic are identical to Spectra's, except for the items listed under [Deliberate differences from Spectra](#deliberate-differences-from-spectra).

---

## Table of contents

- [Platform overview](#platform-overview)
- [Engine & SDD core](#engine--sdd-core)
- [The SDD workflow](#the-sdd-workflow)
- [CLI](#cli)
- [Desktop app](#desktop-app)
- [Node SDK (@speclink/engine)](#node-sdk-speclinkengine)
- [Team mode (remote store)](#team-mode-remote-store)
- [Configuration](#configuration)
- [Deliberate differences from Spectra](#deliberate-differences-from-spectra)
- [Development & parity testing](#development--parity-testing)
- [Documentation](#documentation)
- [Vision & roadmap](#vision--roadmap)

---

## Platform overview

One engine, one storage seam, multiple frontends:

```text
   Frontends /   ┌───────────┬──────────────┬────────────┐
   hosts         │    CLI     │  Desktop app  │  Node SDK   │
                 └───────────┴──────┬───────┴────────────┘
                                    │  embed the same engine
   Engine        ┌────────────────▼─────────────────┐
                 │            speclink-core           │
                 │       SDD workflow logic · render  │
                 └────────────────┬─────────────────┘
                                    │  Store seam
   Storage       ┌────────────────▼─────────────────┐
                 │   speclink-fs        speclink-remote │
                 │   local markdown     team system REST │
                 └──────────────────────────────────┘
```

| Component | crate／package | In one line |
| --- | --- | --- |
| **Engine** | `speclink-core` | The single source of truth for SDD workflow logic and rendering, in Rust |
| **Local store** | `speclink-fs` | Default Store — markdown under `openspec/` is the truth |
| **Remote store** | `speclink-remote` | Team-mode Store — a thin client of the [verb contract](docs/verb-contract.md) REST API |
| **CLI** | `speclink-cli` (`speclink`) | Command-line frontend; human-facing output + `--json` machine output |
| **Desktop app** | `@speclink/desktop` (Tauri) | A lifecycle kanban GUI that embeds core |
| **Node SDK** | `@speclink/engine` (`speclink-node`) | Embeds the engine in a Node process ([napi-rs](https://napi.rs) bindings) |
| **Shared UI** | `@speclink/ui` | The React component library for the desktop frontend |

Why one shared engine matters: whether you drive SDD from the CLI, the desktop app, or your own Node service, verb behavior, `--json` shapes, and the generated skill／instruction content are all decided by the same Rust code — consistent by construction.

---

## Engine & SDD core

Speclink manages two kinds of documents:

- **Canonical specs** — `openspec/specs/<capability>/spec.md`, describing how the system behaves *now*; the single source of truth.
- **Change proposals** — `openspec/changes/<name>/`, describing one change as a *delta* against canon: ADDED, MODIFIED, REMOVED, RENAMED. Once implemented, `archive` folds the delta into canon.

Specs are written with fixed structural markers so the engine can parse, validate, and inject traceability:

```markdown
## ADDED Requirements

### Requirement: User login

The system SHALL allow users to log in with email and password. Three failed
passwords MUST lock the account for 15 minutes.

#### Scenario: Successful login

- **WHEN** the user enters a correct email and password
- **THEN** the system SHALL create a session and redirect to the home page
```

`### Requirement:`, `#### Scenario:`, `- **WHEN**`/`- **THEN**` and the normative keywords `SHALL`/`MUST` are part of the structure and always stay in English; the prose language is set by `spec_locale` (see [Configuration](#configuration)).

The engine itself has three layers — **engine → Store → render**. Where truth lives is abstracted by the Store seam: `speclink-fs` treats markdown files as truth (the default), while `speclink-remote` moves truth to a team system and accesses it over a REST contract. Frontends (CLI／desktop／SDK) only talk to the engine, never to storage directly. See [docs/architecture.md](docs/architecture.md) for the three layers and the seam.

---

## The SDD workflow

```text
discuss?  →  propose  →  apply  ⇄  ingest  →  verify?  →  archive
```

The same flow drives both the CLI and the desktop app; inside an AI agent you invoke it via `/speclink-<name>` slash commands, and the skills fetch each step's instructions from the engine.

| Stage | Skill | When to use it |
| --- | --- | --- |
| **discuss** | `/speclink-discuss` | Requirements are fuzzy and worth debating first (optional). Runs as Socratic Q&A recorded to a document; the conclusion can `promote` into a change. |
| **propose** | `/speclink-propose` | To plan／design a change. Produces four artifacts: proposal, delta specs, design, tasks. `--from-discussion <slug>` seeds it from a concluded discussion. |
| **apply** | `/speclink-apply` | To start implementing. Complete tasks one by one; `task done` records the files each task touched. |
| **ingest** | `/speclink-ingest` | Requirements changed mid-implementation. Update delta specs and tasks without starting over. |
| **drift** | `/speclink-drift` | Run before resuming a change that sat idle, to detect whether specs and code have diverged. |
| **verify** | `/speclink-verify` | Implementation done; confirm the code really matches the spec (optional). |
| **archive** | `/speclink-archive` | Wrap-up. Fold the delta into canon, inject `@trace`, snapshot for restore, and co-archive the linked discussion. |

Helper skills: `/speclink-onboard` (adopt Speclink on an existing codebase, deriving initial specs from current behavior), `/speclink-analyze` (check artifact consistency), `/speclink-audit` (security review), `/speclink-commit` (commit only the files for a given change). `init` generates 11 skills for the selected tools.

### discuss: document-based discussion

This is Speclink's main extension over Spectra. Spectra's discuss is a pure skill that leaves no document, so long discussions tend to drift off-topic. Speclink lands every discussion as a structured document (`openspec/discussions/<slug>.md`); the flow logic and Socratic Q&A are unchanged, but the process is fully recorded, evolvable, and eventually convertible into a change.

Documents follow four rules: one question per round, append-only (no rewriting), explicitly record excluded options, and the conclusion must resolve or explicitly defer every open question. A `promote`d discussion is **co-archived automatically** when its change is archived (one discussion can fan out into several changes). Discussion documents are **not created at the outset**: they land only at the first substantive round, so a mistaken or one-line topic leaves no file; drop a discussion you no longer need with `discard` — a substantive one should `conclude` + `archive` to preserve the reasoning ("decided not to do it" is a conclusion worth keeping).

**Cut and reopen**: to discard a change and start over, use `speclink discard <change>` — it deletes the change and unlinks its source discussions from their `promoted_to`; when that empties a discussion's list, its status reverts to `concluded` (with a conclusion) or `open` (without), so the same discussion can `promote` a follow-up change. A change with started work (`started_at` or checked tasks) needs `--force` to discard.

---

## CLI

The command-line frontend, and the entry point for adopting Speclink. Requires the Rust toolchain (stable).

```bash
git clone <this-repo>
cd speclink
cargo build --release
# artifact: target/release/speclink(.exe)
```

Add `target/release/` to `PATH`, or call by full path. The docs below all refer to it as `speclink`.

### Quick start

```bash
# 1. Initialize at the project root (auto-detects .claude/ or .agents/AGENTS.md to
#    decide which skill set to generate)
speclink init

# 2. To see what a sample change looks like:
speclink demo            # generate a random-topic demo change
speclink list            # list current changes
speclink show <name>     # inspect a change

# 3. Check and archive
speclink validate <name> --strict
speclink analyze <name>
speclink archive <name> -y
```

`init` creates `openspec/` (`specs/`, `changes/archive/`, `config.yaml`), `.speclink.yaml` (application settings), each tool's skill files and instruction-injection blocks (`CLAUDE.md`／`AGENTS.md`), and a `.gitignore` block.

### Command reference

Grouped by purpose. Every command supports `--no-color`; most support `--json` for programmatic use. Add `--help` to any command for full options.

**Project & settings**

| Command | Purpose |
| --- | --- |
| `speclink init [PATH]` | Initialize a project. `--tools <claude,codex>` lists tools explicitly (auto-detect if omitted); `--dir <DIR>` customizes the spec directory; `--store remote` initializes directly in team mode; `--force` overwrites |
| `speclink link` / `speclink unlink` | Bind／unbind an existing repo to a team system (see [Team mode](#team-mode-remote-store)) |
| `speclink auth <login\|status\|logout>` | Team-mode authentication |
| `speclink update` | Regenerate skills and injection blocks from `.speclink.yaml`'s `tools:`, and clean up leftovers of removed tools |
| `speclink config <get\|set\|unset\|list\|reset\|edit\|path>` | Global settings |
| `speclink completion <generate\|install\|uninstall> <shell>` | Shell completion scripts |

**Change lifecycle**

| Command | Purpose |
| --- | --- |
| `speclink new change <name>` | Create a change (kebab-case). `--schema`, `--description` |
| `speclink new artifact <proposal\|design\|tasks\|spec> --change <name> [--stdin]` | Create a single artifact |
| `speclink list [--changes\|--specs] [--sort name\|modified\|created] [--json]` | List changes or canonical specs |
| `speclink show <item> [--item-type change\|spec] [--json]` | Inspect a change or spec |
| `speclink status --change <name> [--json]` | Show the completion status of the artifact-dependency DAG |
| `speclink instructions <artifact\|apply> --change <name> [--json]` | Get the instructions payload for an artifact (or apply mode) |
| `speclink task done <id> --change <name>` | Mark the Nth task done and record touched files |
| `speclink archive [name...] [-y] [--all]` | Archive. Multiple names or `--all` for batch; `--skip-specs`, `--no-validate`, `--mark-tasks-complete` |
| `speclink discard <change> [--force]` | Discard a change. Deletes the change directory and unlinks its source discussions from `promoted_to` (a discussion whose list empties reverts to concluded/open); a change with started work (`started_at` or checked tasks) needs `--force` |
| `speclink in-progress <...>` | Manage in-progress markers |
| `speclink discuss <...>` | Document-based discussion (`new`／`context`／`add-round`／`conclude`／`promote`／`archive`／`discard`, see above) |

**Check & analyze**

| Command | Purpose |
| --- | --- |
| `speclink validate <name> [--all\|--specs\|--changes] [--strict] [--json]` | Structural validation (duplicate requirement names, no-op deltas, etc.) |
| `speclink analyze <name> [--json]` | Four-dimension analysis: Coverage／Consistency／Ambiguity／Gaps |
| `speclink drift <name> [--json]` | Detect divergence between a change and the current code (see [deliberate differences](#deliberate-differences-from-spectra)) |

**Others**: `speclink schemas` / `schema <show\|validate\|fork\|init>` (schema management), `speclink templates` (template paths), `speclink demo` (a demo change).

---

## Desktop app

`@speclink/desktop` is a [Tauri](https://tauri.app) application that **embeds `speclink-core` directly** (rather than spawning the CLI as a subprocess) and operates on a local `openspec/` project. It presents changes as a lifecycle kanban (discussion／proposed／in-progress／ready), with multi-project tabs, a detail drawer, and interactive tasks, and it watches the filesystem — when an external CLI, agent, or editor changes `openspec/`, the board updates live. Markdown files are always the truth; the app never moves any document truth out of the filesystem.

```bash
npm install                            # install workspace deps at the repo root
npm run tauri dev -w apps/desktop      # dev mode (hot reload)
npm run tauri build -w apps/desktop    # build desktop installers
```

To rebuild only the frontend: `npm run build -w apps/desktop` (vite → `dist`); to rebuild only the native shell: `cargo build --release -p speclink-desktop`. Frontend tests: `npm test -w apps/desktop`, `npm test -w packages/ui`. Detailed behavior specs live under `openspec/specs/desktop-app/`.

---

## Node SDK (@speclink/engine)

`@speclink/engine` embeds the Speclink engine in a Node.js process: your server (or AI-agent host) dispatches speclink verbs in-process, stores spec documents in its own database through a custom `Store`, and renders the workflow knowledge (skills, instruction blocks) for whatever harness it runs. It is the same Rust engine the CLI ships — bound with napi-rs, not re-implemented — so verb behavior and `--json` shapes are identical by construction.

```bash
npm install @speclink/engine
```

A native module; prebuilt binaries ship as `optionalDependencies`, so `npm install` just works — no toolchain — on the five supported targets (Windows x64, macOS x64／arm64, Linux x64／arm64 glibc).

```js
const { createEngine } = require('@speclink/engine')

// Form 1: built-in fs store, pointed at a local project root
const engine = createEngine({ store: { type: 'fs', root: '/path/to/project' } })

// Form 2: a host Store (e.g. over Postgres) — the engine reads/writes through it
const engine = createEngine({ store: myStore })
```

The full Store interface, dispatch contract, and render API are in [docs/sdk-node.md](docs/sdk-node.md).

---

## Team mode (remote store)

In team mode, spec documents and change state live in a **team system** (a server embedding the Speclink engine), while your code and git stay local. The `speclink` CLI becomes a thin client of the [verb contract](docs/verb-contract.md): every verb you already use (`list`, `status`, `instructions`, `task done`, `discuss …`) keeps the same output shape; only the storage behind it moves.

The mode is decided by one section of `.speclink.yaml`:

```yaml
# .speclink.yaml — committed, like .lfsconfig: every clone gets the same binding
tools:
  - claude
remote:
  url: https://team.example.com/api/speclink/v1/projects/erp   # project-scoped
  repo: backend    # optional on single-repo projects — this repo's registered name
```

No `remote:` section = fs mode (unchanged); present = remote mode. Credentials never live in this file.

```bash
# Fresh repo, straight into team mode (no openspec/; documents live on the server)
speclink init --store remote --url <project-url> --repo backend

# Existing repo: bind／unbind
speclink link <project-url> --repo backend
speclink unlink

# Authentication
speclink auth login
speclink auth status
```

`SPECLINK_STORE_URL` can override the url for one shell or CI job. Client-side connection, authentication, repo identity, and error mapping are in [docs/team-mode.md](docs/team-mode.md).

This is exactly what Speclink aims to unlock — **decoupling roles from storage**: PO／PM run `discuss + propose + ingest + archive` in a bespoke system, RD／QA run `apply + verify` in the local git repo, both sharing the same engine while each picks the storage and interface that fits.

---

## Configuration

### `.speclink.yaml` (application layer)

Generated by `init`, with inline comments. Main fields:

```yaml
# Language of AI-generated artifacts (proposal/design/tasks, etc.), default English
# locale: tw

# Language of spec prose (specs/*/spec.md), default English; "auto" follows locale
# Structural markers and SHALL/MUST always stay English
# spec_locale: tw

# Workflow discipline switches, default off
# tdd: true      # test-first discipline during apply
# audit: true    # inline security-review discipline during apply

# Tool list for the skills init generates (drives update's sync and cleanup)
tools:
  - claude
  - codex

# A remote: section switches to team mode (see above)
```

- `locale` / `spec_locale` support `tw` (Traditional Chinese), `ja` (Japanese), `en`／unset (English), etc. Set to `tw`/`zh*` and the spec instructions gain a Chinese weak-word warning, and the analyzer detects Chinese weak language ("應該、也許、考慮、待定、可能…") alongside English should/may/TBD.
- `tdd` / `audit` are disciplines embedded in the apply skill, not standalone skills.

### `openspec/config.yaml` (workflow layer)

```yaml
schema: spec-driven

# Project context (given to the AI when creating artifacts)
context: |
  Tech stack, conventions, domain knowledge…

# Per-artifact custom rules
rules:
  proposal:
    - Must include a "Non-goals" section
```

Four-layer resolution, tool descriptors, and migration guidance are in [docs/configuration.md](docs/configuration.md).

---

## Deliberate differences from Spectra

Every divergence from Spectra is deliberate; everything else stays byte-level identical. Four structural divergences:

1. **Persistent discuss** — discussions land as evolvable documents that can `promote` into changes (see above).
2. **drift enhancements** — Spectra's drift has several distortions; Speclink fixes and strengthens them:
   - `--since` anchors to that day's midnight (Spectra uses a bare date, and git approxidate makes same-day changes always count 0 commits)
   - anchor extraction only takes code-like tokens (camelCase／snake_case／multi-segment PascalCase), so prose capitalized words no longer false-positive; backtick paths become an existence check (File anchor)
   - the anchor search corpus excludes the change's own directory (Spectra's corpus includes itself, so a committed design always self-satisfies)
   - a new **Specs dimension** and `spec_assumptions`: detects that a delta's canonical target has been rewritten (MODIFIED/REMOVED/RENAMED target missing, ADDED target already exists) — cases that "archive would silently skip" are routed to `ingest`
   - the Tasks dimension gives real signal (judging "possibly done／blocked by external change" from each task's file references × commit window)
3. **Dual-mode audit** — rewritten as a standalone (three-agent parallel analysis) and an apply-embedded discipline, so it is not a fork skill.
4. **RENAMED actually executes** — Spectra documents `## RENAMED Requirements` but no syntax executes it and `renamed:` stays 0; Speclink truly rewrites the canonical requirement header on archive and counts it, so rename-only changes also validate and archive.

Other extensions: the `spec_locale` spec-language setting and Chinese weak-language detection, the `onboard` adoption skill, `archive` batch archiving (multiple names／`--all`, with clean-working-tree enforcement, per-item skip with reason, fail-fast), `init` tool auto-detection, `update`'s sync and footprint-free cleanup, and MODIFIED's `<!-- BEFORE: -->` prior-value annotation (stripped on archive).

Removed Spectra features: `ask`, `debug`, vector search (`search`), `worktree`, `park`/`unpark`, `parallel_tasks`, `claude_effort`. Tool scope is limited to claude + codex.

---

## Development & parity testing

Speclink is developed against the Spectra 2.3.1 binary: every difference is first confirmed on a controlled fixture with a twin-binary run before aligning the implementation.

- **parity_suite** — 31 CLI-output comparisons (byte-for-byte after brand normalization; drift's deliberate divergences neutralized by a normalization layer)
- **color_suite** — 16 ANSI-color comparisons under `CLICOLOR_FORCE=1`
- **twin harness** — 8 drift scenarios across dual sandboxes

---

## Documentation

| Topic | English | 繁體中文 |
|---|---|---|
| Architecture (engine–Store–render layers, storage seam) | [docs/architecture.md](docs/architecture.md) | [docs/architecture.zh-TW.md](docs/architecture.zh-TW.md) |
| Getting started (a full local SDD loop) | [docs/getting-started.md](docs/getting-started.md) | [docs/getting-started.zh-TW.md](docs/getting-started.zh-TW.md) |
| Configuration (two-files-one-directory system, four-layer resolution, tool descriptors, migration) | [docs/configuration.md](docs/configuration.md) | [docs/configuration.zh-TW.md](docs/configuration.zh-TW.md) |
| Team mode (connection, init/link/auth, repo identity, error mapping, upgrade) | [docs/team-mode.md](docs/team-mode.md) | [docs/team-mode.zh-TW.md](docs/team-mode.zh-TW.md) |
| Verb contract (the REST contract canon for the remote store: endpoints, payloads, 409 semantics) | [docs/verb-contract.md](docs/verb-contract.md) | [docs/verb-contract.zh-TW.md](docs/verb-contract.zh-TW.md) |
| Node SDK (@speclink/engine: two createEngine forms, Store bridge, dispatch contract, render API) | [docs/sdk-node.md](docs/sdk-node.md) | [docs/sdk-node.zh-TW.md](docs/sdk-node.zh-TW.md) |
| Brand assets (logo, colors, usage) | [docs/assets/brand/README.md](docs/assets/brand/README.md) | — |

---

## Vision & roadmap

### Origin

Speclink grew out of a comparative analysis of Spectra and OpenSpec, aiming to keep the strengths of both, rewrite in Rust, and extend the design further. The first phase (done) was a complete CLI behaviorally identical to Spectra, then the deliberate differences on top.

### The spec-driven engine

Today, in both OpenSpec and Spectra, spec documents are tied to the git repository. Speclink goes further — offering a **spec-driven engine** abstraction: how documents are stored and managed is up to you (Markdown, a database, your own system or JIRA), and the engine only owns the SDD workflow logic.

That direction is already materializing — `speclink-core`'s Store seam, `speclink-remote`'s [team mode](#team-mode-remote-store), and the [`@speclink/engine`](#node-sdk-speclinkengine) Node SDK are the first fruits of "specs need not follow git." The desktop app is this engine's first GUI frontend; more frontends (including web) are planned.

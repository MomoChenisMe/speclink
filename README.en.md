<p align="center">
  <img src="docs/assets/brand/transparent/speclink-logo-horizontal.png" alt="Speclink" width="440" />
</p>

<p align="center">
  <b>One SDD engine for local repositories and remote stores</b>
</p>

<p align="center">
  <a href="README.md">繁體中文</a> · <b>English</b>
</p>

Speclink is a Spec-Driven Development (SDD) engine and tool platform implemented in Rust. PMs, POs, developers, and AI
agents share the same change, artifact, task, verification, and archive semantics across two deployment paths:

- **Local repository:** specs live in `openspec/` and collaboration uses Git. No server is required.
- **Remote store:** specs live in a shared Store and a Speclink Host governs identity, revisions, transactions, events, and workflow rules.

The local CLI uses the CLI shipped with [Spectra App 2.3.1](https://github.com/kaochenlong/spectra-app) as its behavioral design reference and compatibility
baseline. Parity and golden tests protect human output, `--json` shapes, and core workflow behavior; Speclink extends that baseline
with discussions, Desktop, storage abstraction, the Node SDK, and the planned Remote Platform.

> **Current status:** the local-repository path, CLI, local Desktop, and Node N-API SDK have working implementations. The new
> TeamStore contract, `speclink-server`, Server Admin UI, and Desktop Remote Workspace are being delivered in phases defined by
> the [platform architecture blueprint](docs/platform-architecture.zh-TW.md). The old remote REST v1 is no longer the target architecture.

See the [current implementation alignment and refactoring roadmap](docs/implementation-refactor-roadmap.zh-TW.md) for the gap analysis,
implementation priority, and acceptance gates for each phase.

## Architecture

Speclink has one Rust implementation of workflow semantics. CLI, Desktop, Server, Node SDK, MCP, and Copilot Tools must all call
the same Command Runtime rather than reimplementing lifecycle or archive behavior.

```text
Local Repository
  Agent / CLI / Desktop
    -> Embedded Rust Runtime
    -> FsStore
    -> repo/openspec/
    -> Git

Remote Store
  Desktop / CLI / MCP / Web UI / Copilot Tool
    -> Speclink Host
    -> Same Rust Runtime
    -> TeamStore (SQLite / Server FS / PostgreSQL / Custom)
```

Remote specs are not synchronized into a second writable local truth. File-oriented agents consume a read-only
`.speclink/context/` snapshot and send every remote write through Host commands.

## Implementation Status

| Component | Today | Target |
|---|---|---|
| `speclink-core` | Rust SDD engine and current workflow modules | Typed Command Runtime, domain events, fail-closed policy |
| `speclink-fs` | Local `openspec/` Store | Keep local mode serverless; add journal/recovery for Server FS |
| `speclink-cli` | Local CLI and existing remote-client foundation | New shared Command/Query/Context Protocol |
| `speclink-desktop` | Local board, specs, archives, tasks, and settings | Local/Remote WorkspaceSession, offline and CAS-conflict UX |
| `@speclink/engine` | Rust N-API SDK and Store bridge | Typed commands, Host/Tool integration, cross-platform prebuilt binaries |
| `speclink-server` | Not delivered yet | Rust single-node server, SQLite default, Admin UI, SSE, backup/restore |

## SDD Workflow

```text
discuss? -> propose -> apply <-> ingest -> verify? -> archive
```

PM, PO, and developer are human roles. Claude Code, Codex App/CLI, GitHub Copilot, Cursor, and similar products are Agent
Hosts/Applications that run models and tools. An Agent Host loads a Speclink Skill, then reaches the Speclink Host through CLI,
MCP, or an in-process Tool. A Skill is workflow knowledge, not the Embedded Speclink Host; local and remote paths share this model.

`discuss` is optional. `propose` creates the artifacts, `apply` implements tasks, `ingest` absorbs requirement changes,
`verify` runs where a code checkout exists, and `archive` merges deltas into canonical specs.

## Local Quick Start

With a stable Rust toolchain:

```bash
git clone <this-repo>
cd speclink
cargo build --release
```

Inside the repository where you want to use Speclink:

```bash
speclink init
speclink demo
speclink list
speclink status --change <change-name>
speclink validate <change-name> --strict
```

Initialization creates `openspec/`, `.speclink.yaml`, gitignored `.speclink/` work data, and the selected Claude Code or Codex
skills. See the [local getting-started guide](docs/getting-started.md) for a complete loop.

## Main CLI Commands

| Command | Purpose |
|---|---|
| `speclink init` / `update` | Initialize a workspace or synchronize generated skills |
| `speclink new change` / `new artifact` | Create a change and its artifacts |
| `speclink list` / `show` / `status` | Query changes, specs, and the artifact DAG |
| `speclink instructions` | Build artifact or apply instructions |
| `speclink task done` | Complete a task and record touched files |
| `speclink validate` / `analyze` / `drift` | Structural, quality, and drift analysis |
| `speclink discuss ...` | Create, conclude, link, and archive discussions |
| `speclink archive` / `discard` | Archive or discard a change |

Run `speclink <command> --help` for all options. Existing parity and golden tests protect observable CLI behavior.

## Speclink Desktop

Desktop is currently the official ready-to-use UI for local repositories, including the change board, canonical specs, archives,
detail drawers, tasks, and settings.

Some local Desktop features and UI/UX direction reference [Spectra App 2.3.1](https://github.com/kaochenlong/spectra-app), including
visual tracking of changes/specs/tasks, project switching, task progress, and archive browsing. Speclink Desktop is not a Spectra
App fork: it is independently implemented with Tauri and React and extends its own discussion lifecycle, DataSource contract, and
planned Remote Workspace model.

```bash
npm install
npm --workspace @speclink/desktop run tauri -- dev
```

The target Desktop also supports local folders, PM/PO remote spec-only workspaces, and developer remote workspaces attached to a
local checkout. It will manage server login, Project/Repo selection, OS Keychain credentials, SSE/ETag recovery, read-only offline
state, and revision conflicts. Desktop is the default Presentation UI, not the only possible UI.

## Node SDK and Agent Tools

`@speclink/engine` loads the same Rust Engine through N-API. The currently implemented installation, Store bridge, and `dispatch`
surface are documented in the [Node SDK guide](docs/sdk-node.md).

```text
Copilot SDK Agent
  -> @speclink/copilot-tools
  -> @speclink/host
  -> @speclink/engine (N-API / Rust)
  -> Node TeamStore Adapter
```

Tools call the in-process Host directly. They do not need CLI, MCP, or HTTP, and they must not bypass the Host to write the Store.

If published, the official `@speclink/store-sqlite`, `@speclink/store-fs`, and `@speclink/store-postgres` packages are optional
N-API facades over the same Rust driver crates, not TypeScript rewrites. The official `speclink-server` links the Rust crates
directly; Node packages exist only for Node Hosts and in-process agents. Custom JavaScript Stores may use the async bridge but must
pass the TeamStore conformance suite.

## Target Remote Platform

The official `speclink-server` will provide a single Rust binary and Docker image, SQLite by default, built-in Server FS and
PostgreSQL drivers, first-run `/setup`, `/admin`, PATs and roles, Project/Repo binding, TeamStore transactions and immutable
history, Query + ETag, SSE, context snapshots, verification evidence, migration, and backup/restore.

It also provides an ordinary-user `/account` portal, self-service PATs, invitations, and browser/device authorization for Desktop.

Server Admin UI manages installation, Store, identities, Project/Repo, migration, and recovery. Daily spec work remains in
Desktop or another Presentation UI.

With a checkout, the agent-visible Context Projection defaults to `.speclink/context/` at the workspace root, not `.git/`. This
avoids linked-worktree `.git` files, search-tool exclusions, and cross-worktree cache mixing. Without a checkout, Desktop app data,
MCP resources/search, or a host-managed Session FS carries the projection.

## Configuration Ownership

| Location | Responsibility |
|---|---|
| `openspec/config.yaml` or remote Store config | Workflow policy: schema, context, rules, locale, TDD, and audit |
| `.speclink.yaml` | Repository/workspace binding and local tool integration; never credentials |
| `.speclink/` | Context snapshots, touched/evidence caches, and other local work data |
| OS Keychain | Remote credentials |

See [Configuration](docs/configuration.md) for the current local behavior. Remote workflow policy is authoritative at a Store
revision and local overrides may not silently replace team policy.

## Repository Layout

```text
crates/
├── speclink-core       Rust SDD engine
├── speclink-fs         Local filesystem Store
├── speclink-cli        CLI frontend
├── speclink-remote     Existing remote-client foundation
└── speclink-node       N-API Node bindings

apps/desktop/           Tauri + React Desktop
packages/ui/            Shared UI and DataSource contract
openspec/               This repository's specs and change history
docs/                   Current documentation and platform blueprint
```

## Development

```bash
cargo test --workspace
npm --workspace @speclink/desktop test
npm --workspace @speclink/desktop run build
```

Engine and Store changes must preserve CLI parity, render goldens, storage-abstraction tests, and cross-platform behavior. Remote
phases also require TeamStore conformance, fault injection, transaction recovery, Protocol, and Desktop end-to-end tests.

## Documentation

| Document | Purpose |
|---|---|
| [Platform architecture blueprint](docs/platform-architecture.zh-TW.md) | Sole target architecture and roadmap baseline |
| [Local getting started](docs/getting-started.md) | Current local-repository SDD workflow |
| [Configuration](docs/configuration.md) | Current local settings, ownership, and migration |
| [Node SDK](docs/sdk-node.md) | Current N-API SDK, Store bridge, and dispatch surface |
| [Brand assets](docs/assets/brand/README.md) | Logos, colors, and usage |

Archived changes and discussions under `openspec/` are audit history, not current architecture entry points.

## Roadmap

1. **Phase 1, Engine foundation:** Typed Runtime, TeamStore contract, UoW, events, binding, and Protocol.
2. **Phase 2, Remote Server:** SQLite/FS/PostgreSQL, auth, Admin UI, SSE, migration, and backup/restore.
3. **Phase 3, Desktop Remote Workspace:** WorkspaceSession, login, Project/Repo, checkout, offline/conflict UX.
4. **Phase 4, Agent ecosystem:** Copilot Tools, MCP, custom UI, SSO, runtime plugins, and Cluster mode.

See the [platform architecture blueprint](docs/platform-architecture.zh-TW.md) for dependencies, P0/P1 correctness requirements,
and explicit non-goals.

## License

[MIT](LICENSE)

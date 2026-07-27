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

- **Local repository:** specs live in the repository's `openspec/` directory and collaboration uses Git. No server is required.
- **Remote store:** specs live in a shared Store and a Speclink Host governs identity, revisions, transactions, events, and workflow rules.

The local CLI was originally designed with the CLI shipped with [Spectra App 2.3.1](https://github.com/kaochenlong/spectra-app) as its
behavioral reference. Golden and CLI integration tests protect human output, `--json` shapes, and core workflow behavior; Speclink extends
that foundation with discussions, Desktop, storage abstraction, the Node SDK, and the Remote Platform.

> **Current status (2026-07-17):** Local Repo, CLI, local Desktop, the Node N-API SDK, the TeamStore contract and all three official
> Store drivers, `speclink-server`, Server setup/admin/auth/backup, and Remote CLI/Context Projection have working implementations.
> Desktop Server Connections are available, while complete Desktop Remote Workspace and verify/evidence remain partial. MCP/Copilot
> Tools, SSO, runtime plugins, and cluster mode are planned. Legacy remote REST v1 is deprecated and is no longer the target architecture.

See [Product Capability Status](docs/product-status.md) for evidence, limits, and the last verification date. See the
[implementation alignment and refactoring roadmap](docs/implementation-refactor-roadmap.zh-TW.md) for the remaining gap, delivery order,
and acceptance gates for each phase.

## Current capabilities / 目前能力

- **Available:** Local Repo CLI, local Desktop, Node N-API SDK, Command Runtime/Host/Protocol, SQLite/Server FS/PostgreSQL TeamStore, single-node Server, Admin/Auth, Remote CLI, and Server operations.
- **Partial:** generated Agent skills, Desktop Remote Workspace, and verify/task evidence; use product-status for the precise usable subset and gap.
- **Planned:** MCP/Copilot in-process tools, SSO, runtime plugins, cluster mode, and a standalone verb-contract user guide.
- **Deprecated:** the legacy remote REST v1 prototype; new work uses the current Client Protocol/Host path.

The complete matrix is maintained only in [Product Capability Status](docs/product-status.md), not duplicated in the README.

## SDD workflow / SDD 工作流

```text
onboard? → discuss? → propose → apply ⇄ ingest → archive
                         ↑
                 resume after pause: drift first

utilities: validate / analyze / audit / commit / verify and evidence
```

Use `discuss` only when requirements need convergence; clear requirements can go directly to `propose`. Requirement changes during
implementation route to `ingest`, and resumed idle work starts with `drift`. See the [complete SDD workflow](docs/workflow.md) for
complete-proposal, fast-scaffold, existing-change, and “do not implement” discussion outcomes.

## Local Repo quick start / Local Repo 快速開始

A stable Rust toolchain is required:

```bash
cargo install --path crates/speclink-cli
speclink --version
```

Inside the repository adopting Speclink:

```bash
speclink init --tools claude,codex
speclink list
```

Then invoke `/speclink-propose <change>` in Claude or `$speclink-propose <change>` in Codex. The Agent creates the artifacts required
by the schema DAG. Follow [Local Repo Getting Started](docs/getting-started.md) for a copyable first loop and direct CLI equivalents.

## Deployment paths / 部署路徑

- **Local repository:** Embedded Rust Runtime → FsStore → `openspec/` → Git; suited to repository-local, offline collaboration.
- **Remote store:** CLI/Desktop/other Client → Speclink Host → the same Rust Runtime → TeamStore; suited to shared canon, centralized identity, revisions, transactions, and events.

A Remote Store does not synchronize into a second writable local truth. Agents with a checkout consume read-only `.speclink/context/`
and send writes through Host commands. The [platform architecture blueprint](docs/platform-architecture.zh-TW.md) defines the target
boundary. Follow [Remote Server, Desktop, and CLI Getting Started](docs/remote-getting-started.md) for the complete first-run path.
Current Server operations are documented in [deployment](docs/server-deployment.zh-TW.md),
[Store drivers](docs/server-store-drivers.zh-TW.md), and [backup/restore](docs/server-backup.zh-TW.md) (Traditional Chinese).

## Documentation map / 文件地圖

| Document | Purpose |
| --- | --- |
| [Local Repo Getting Started](docs/getting-started.md) | Copyable first Local Repo loop using current entry points |
| [Remote Server, Desktop, and CLI Getting Started](docs/remote-getting-started.md) | Complete setup, membership, sign-in, Desktop/CLI, and recovery path |
| [Complete SDD Workflow](docs/workflow.md) | Purpose, timing, branches, completion criteria, and recovery for every stage |
| [Product Capability Status](docs/product-status.md) | Available/Partial/Planned/Deprecated with evidence and limits |
| [Configuration](docs/configuration.md) | Local/Remote ownership and current fields |
| [Node SDK](docs/sdk-node.md) | `@speclink/engine` installation, Store bridge, and dispatch surface |
| [Platform architecture blueprint](docs/platform-architecture.zh-TW.md) | Sole target architecture for Engine, Host, Store, Protocol, Server, Desktop, and Agents (Traditional Chinese) |
| [Implementation refactoring roadmap](docs/implementation-refactor-roadmap.zh-TW.md) | Delivery order, phases, and gates beneath the target architecture (Traditional Chinese) |
| [Server deployment](docs/server-deployment.zh-TW.md) | Native/Docker/Compose deployment and upgrades (Traditional Chinese) |
| [Server Store drivers](docs/server-store-drivers.zh-TW.md) | SQLite/Server FS/PostgreSQL selection and prerequisites (Traditional Chinese) |
| [Server backup and restore](docs/server-backup.zh-TW.md) | Backup/verify-backup/restore (Traditional Chinese) |
| [Brand assets](docs/assets/brand/README.md) | Logo, color, and usage guidance |

`openspec/changes/archive/` and `openspec/discussions/archive/` are historical audit records, not current operating guides. The
advanced `docs/verb-contract.md` user guide does not exist yet; canonical specs remain the contract and product-status records the gap.

## Development / 開發

```bash
cargo test --workspace
npm --workspace @speclink/desktop test
npm --workspace @speclink/desktop run build
npm --workspace @speclink/engine test
```

Golden and CLI integration tests protect observable CLI output. See product-status and the responsibility documents for Server, Store, and
Desktop test prerequisites and current limitations.

## License / 授權

[MIT](LICENSE)

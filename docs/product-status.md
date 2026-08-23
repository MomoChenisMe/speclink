# Speclink Project Capability Status

[繁體中文](product-status.zh-TW.md) · **English**

Last audited: **2026-08-25**. This document is the canon for "can I use this yet". The specs under `openspec/specs/` are the canon for behavior and boundaries, and the [Project Roadmap](roadmap.md) describes the direction that matters to users.

A file, a crate, or a canonical spec on its own does not mean the delivery path is complete.

To exercise the Remote Server, Desktop, and CLI from a clean local state, follow [Remote Getting Started](remote-getting-started.md) through setup, membership, sign-in, workspace, and recovery.

## Status model / 狀態模型

- **Available**: there is a working entry point, backed by at least two independent pieces of evidence or one end-to-end proof.
- **Partial**: a usable subset exists, but the full flow still has a named gap.
- **Planned**: only a target design, base types, or an unclosed entry exists. It must not be written up as supported today.
- **Deprecated**: a compatibility or historical path is still findable but is no longer part of the target architecture.

## Local and Remote / 本地與遠端能力對照

The difference between the two paths lives in this one table, so you never reassemble it across documents. The Remote Store column is measured against the official reference server, `speclink-server`. Remote mode itself is defined by the Host and Protocol contracts, so the same rows apply to a server you build yourself. The [canonical verb contract](../openspec/specs/verb-contract/spec.md) declares CLI verb mode assignment in a single place. Nearly every verb is **Dual**: it has a local arm and a remote arm, and a missing arm is a build failure. Only `demo` is local-only and only `claim` is remote-only.

| Capability / 能力 | Local Repo | Remote Store | Note / 說明 |
| --- | --- | --- | --- |
| Reading and writing specs and changes | Available | Available | Local reads and writes `openspec/` directly; remote always goes through Host commands and never creates a second writable local truth. |
| Change lifecycle verbs (`propose`→`apply`→`archive`) | Available | Available | `status`, `instructions`, `new`, `task`, `in-progress`, `archive`, and `discard` are all Dual. Remote does not support bulk archiving — one change at a time. |
| Discussions (`discuss`) | Available | Available | Dual; promoting, folding into an existing change, and archiving behave identically on both. |
| Quality stations (`review`/`verify`) | Available | Available | Dual; ticket, round, and stamping semantics are the same on both. |
| Claiming a change (`claim`) | Not applicable | Available | Remote-only — local mode refuses with a non-zero exit code. |
| Demo data (`demo`) | Available | Not applicable | Local-only — remote mode refuses explicitly and issues no server request at all. |
| Agent reading context | Available | Available | Local reads the repo directly; remote reads the read-only `.speclink/context/`, with writes still going through Host commands. |
| Desktop board and drawer | Available | Partial | Connections, sign-in, and opening a remote board from the chooser all work; the remaining slivers (capability lists, change metadata, offline conflicts) are in the capability table. |
| Touched-file evidence for tasks | Available | Available | Local writes `.evidence.json` in the change directory; remote stores the reported touched files, and `GET /changes/{name}/evidence` reads them back. |
| Accounts, PATs, and membership | Not applicable | Available | The local path needs no accounts; remote has `/setup`, invites, PATs, and device login. |
| Backup and restore | Carried by Git | Available | Remote has `backup`, `verify-backup`, and `restore`, currently requiring a maintenance window. |
| Working offline | Available | Needs a connection | Local needs no server at all; remote writes need a reachable Host. |

## Capability matrix / 能力矩陣

| Capability / 能力 | Status / 狀態 | User entry / 使用者入口 | Evidence / 證據 | Limits and next step / 限制與下一步 | Checked / 查核 |
| --- | --- | --- | --- | --- | --- |
| Local Repo CLI | Available | `speclink init`, `list`, `show`, `status`, `validate`, `analyze`, `drift`, `archive`, and the discussion verbs | [`speclink-cli` entry](../crates/speclink-cli/src/main.rs)<br>[CLI integration tests](../crates/speclink-cli/tests/it/doc_verbs.rs) | Local Repo needs no server at all; advanced use still means reading each subcommand's `--help` for flags. | 2026-08-13 |
| Generated Agent Skills | Available | Claude `/speclink-*`, Codex `$speclink-*` (also selectable from the `/skills` list) | [Generated apply skill](../.agents/skills/speclink-apply/SKILL.md)<br>[Generated verify skill](../.agents/skills/speclink-verify/SKILL.md) | Generation covers onboard, discuss, improve, propose, apply, worktree, ingest, drift, quality, review, verify, archive, audit, commit, and config. The one asymmetry is `analyze`, which exists on the Claude side only; Codex uses the CLI directly. The count depends on the `worktree` policy: with it off you get 15 Claude skills and 14 Codex skills; with it on each side gains the two worktree skills, which is why this repo has 17 and 16. | 2026-08-13 |
| Local Desktop | Available | Tauri/React change board, specs, discussions, archive, tasks, settings, and tray | [Desktop scripts](../apps/desktop/package.json)<br>[Desktop UI tests](../apps/desktop/src/__tests__/App.test.tsx) | The local workspace works; Remote Workspace completeness is tracked separately in this table. | 2026-08-13 |
| Quality stations (review/verify) | Available | `/speclink-review`, `/speclink-verify`, `/speclink-quality`; `speclink review` and `speclink verify` on the CLI | [Review station implementation](../crates/speclink-core/src/review.rs)<br>[Stamping and ticket semantics](../crates/speclink-core/src/station.rs) | Both stations keep a multi-round ticket and stamp only once the must-fix set is empty (SUGGESTION never blocks). Editing a file in scope after stamping downgrades it to "modified since". | 2026-08-13 |
| Node N-API SDK | Partial | `npm install @speclink/engine` from the first release that carries the engine; until then, build `crates/speclink-node` from this repo and load it by path | [Node package entry](../crates/speclink-node/package.json)<br>[dispatch contract tests](../crates/speclink-node/__test__/dispatch-contract.spec.ts)<br>[npm publish job](../.github/workflows/release.yml)<br>[Version stamping tests](../scripts/npm-engine-package.test.mjs) | **Pipeline wired, nothing on the registry yet**: every release tag publishes the main package and the five platform sub-packages under that tag's version, so whether `npm install` resolves depends on the first release that carries the engine; until then a repo build (and a Rust toolchain) is required. The Engine and Store bridge itself works; the full Node Host and Copilot Tool packages are not delivered. | 2026-08-23 |
| Install channels | Available | Desktop installers (macOS dmg, Windows NSIS, Linux AppImage and deb), CLI install scripts and a Homebrew tap, npx and Docker for the server | [Install script tests](../scripts/install.test.mjs)<br>[Homebrew formula generator](../scripts/homebrew-formula.mjs) | Desktop and CLI both have channels on all three platforms. Windows installers are not code-signed yet, so first run needs a SmartScreen bypass. | 2026-08-13 |
| Command Runtime, Host and Protocol | Available | Rust crates shared by the CLI, Server, and Node adapter | [Host dual-path tests](../crates/speclink-host/tests/bridge_dual_path.rs)<br>[Client Protocol spec](../openspec/specs/client-protocol/spec.md) | The base typed command/query/context path exists; Agent ecosystem packaging and some advanced gates remain Partial or Planned. | 2026-08-13 |
| SQLite TeamStore | Available | The default `sqlite` driver in `speclink-server` | [SQLite conformance tests](../crates/speclink-store-sqlite/tests/conformance.rs)<br>[Driver selection guide](server-store-drivers.zh-TW.md) | Positioned for a single instance; clustering is out of current scope. | 2026-08-13 |
| Server FS TeamStore | Available | The `serverfs` driver in server config | [Server FS conformance tests](../crates/speclink-store-fs/tests/it/conformance.rs)<br>[Atomic publish tests](../crates/speclink-store-fs/tests/it/atomic_publish.rs) | Requires dependable OS advisory lock/flock semantics; one data directory allows only one server. | 2026-08-13 |
| PostgreSQL TeamStore | Available | The `postgres` driver in server config | [PostgreSQL conformance tests](../crates/speclink-store-postgres/tests/it/conformance.rs)<br>[Resilience tests](../crates/speclink-store-postgres/tests/it/resilience.rs) | The full test run needs PostgreSQL and `SPECLINK_TEST_POSTGRES_URL`; the server is still positioned as a single instance. | 2026-08-13 |
| `speclink-server` | Available | Native binary, Docker, or npx, with HTTP Command/Query/Context/Event APIs | [Server binary](../crates/speclink-server/src/main.rs)<br>[CLI-to-server E2E](../crates/speclink-server/tests/it/e2e_cli.rs) | The single-node server works; remote task completions now persist their reported touched-file evidence — see the Remote task evidence row. | 2026-08-25 |
| Server Admin, setup and identity | Available | `/setup`, `/admin`, `/account`, PAT, device flow, invites, and headless admin commands | [Admin E2E tests](../crates/speclink-server/tests/it/admin_e2e.rs)<br>[Device-flow E2E tests](../crates/speclink-server/tests/it/device_e2e.rs) | Covers single-node installation and account management; SSO and cluster administration remain planned. | 2026-08-13 |
| Desktop Server Connections | Available | The Server list in Desktop settings, device login, PAT fallback, logout, and the OS keychain | [Tauri connection orchestration](../apps/desktop/src-tauri/src/connections.rs)<br>[Servers panel tests](../apps/desktop/src/__tests__/serversPanel.test.tsx) | Connections and identity are manageable; once signed in, the chooser opens a remote workspace — see the next row for what is left. | 2026-08-23 |
| Desktop Remote Workspace | Partial | Opening a remote workspace from the chooser, in either skip (no checkout) or folder (bound to a local checkout) mode | [Workspace chooser](../apps/desktop/src/components/WorkspaceChooser.tsx)<br>[Remote session factory](../apps/desktop/src/session.ts)<br>[Remote open tests](../apps/desktop/src/__tests__/remoteOpen.test.ts) | The remote board opens, tasks can be checked, and artifacts read and written. What is left: capability lists and change metadata are unsupported remotely, a discussion's `promotedTo` is filled with an empty list, and offline conflict handling is unfinished. | 2026-08-23 |
| Remote CLI and Context Projection | Available | `speclink link`, `auth`, `artifact`, and the read-only `.speclink/context/` | [Remote CLI tests](../crates/speclink-cli/tests/it/remote_read_path.rs)<br>[Context materializer](../crates/speclink-host/src/projection.rs) | The current Client Protocol path works; it does not close what is left of the Desktop Remote Workspace. | 2026-08-13 |
| Remote task evidence | Available | Local `speclink task done` writes `.evidence.json`; the same verb remotely stores the reported touched files, and `GET /changes/{name}/evidence` reads them back | [Task evidence implementation](../crates/speclink-core/src/tasks.rs)<br>[Remote evidence end-to-end test](../crates/speclink-server/tests/it/phase2_chain.rs) | The record commits in the same transaction as the checkbox and the task-completed event, and travels with the change when it is archived or discarded. Checking a task from the Desktop remote board sends no touched files, keeping the "no new dirty file, no record" semantics. | 2026-08-23 |
| Server operations | Available | Native, Docker, and Compose; health and readiness; backup, verify-backup, and restore | [Deployment guide](server-deployment.zh-TW.md)<br>[Backup E2E tests](../crates/speclink-server/tests/it/backup_e2e.rs) | Backups currently need a maintenance window; there is no rolling upgrade or cluster operation. | 2026-08-13 |
| MCP and Copilot in-process tools | Planned | No installable Copilot tools package or MCP adapter yet | [Current workspace package inventory](../package.json)<br>[Direction and observable next step](roadmap.md) | Do not read an architecture diagram as a shipped package; a tool adapter, identity closure, and end-to-end tests still have to land. | 2026-08-13 |
| SSO, runtime plugins and cluster mode | Planned | No usable entry yet | [Direction and observable next step](roadmap.md) | Later platform and ecosystem capabilities, with no committed ordering; the Server and drivers remain officially positioned as a single instance. | 2026-08-13 |
| Legacy remote REST v1 | Deprecated | The historical remote client prototype | [The historical prototype crate](../crates/speclink-remote/src/lib.rs)<br>[Current Client Protocol canon](../openspec/specs/client-protocol/spec.md) | Not a compatibility burden on the new Client Protocol nor an official Server contract; new documentation explains migration only and does not teach this path. | 2026-08-13 |
| Advanced verb-contract user guide | Available | [Verb and Flag Contract](verb-contract.md) (both languages) | [Canonical verb contract](../openspec/specs/verb-contract/spec.md)<br>[Client Protocol spec](../openspec/specs/client-protocol/spec.md) | The guide exists and covers verb mode assignment, cross-mode output parity, and endpoint contracts; the canon is still the specs, and the guide tracks them. | 2026-08-13 |

## Verification baseline / 查核基線

This status assessment can be reproduced as follows:

1. Run `speclink --help` and the relevant subcommand `--help` output to check the Local and Remote CLI surface.
2. Run `speclink-server --help` to check the server, identity, and backup entry points.
3. Compare the directory listings of `.claude/skills/` and `.agents/skills/`. That separates "the engine holds an asset" from "this Host generated a skill". The only difference between the two sides is `speclink-analyze`, which exists on the Claude side alone; the totals move with the `worktree` policy.
4. Check the Local and Remote table against the mode declarations in the [canonical verb contract](../openspec/specs/verb-contract/spec.md). That declaration is the single source. It assigns every verb to ModeFree, Dual, FsOnly, or RemoteOnly.
5. Cross-check the workspace `Cargo.toml`, each package's scripts, the integration/E2E/conformance tests, and the canonical specs. A capability with no user entry must not be marked Available just because a crate exists.

## Known documentation gap / 已知文件缺口

`@speclink/engine` has its publishing pipeline wired but is not on the registry yet — whether `npm install` resolves depends on the first release that carries the engine. Until then the repo-built loading path in [Node SDK](sdk-node.md) is the only one that works. The direction and observable next step for the npm channel are in the [Project Roadmap](roadmap.md).

## Target references / 目標參考

- [Complete SDD Workflow](workflow.md): every station's purpose, skill, completion criteria, and next station.
- [Project Roadmap](roadmap.md): the direction that matters to users.
- [Server Deployment](server-deployment.zh-TW.md), [Store Drivers](server-store-drivers.zh-TW.md), [Backup and Restore](server-backup.zh-TW.md) (Traditional Chinese only): how the Server is operated today.

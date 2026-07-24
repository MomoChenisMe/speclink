# Speclink Product Capability Status

[繁體中文](product-status.zh-TW.md) · **English**

Last verified: **2026-07-17**. This document is the canonical answer to “can I use this now?” The [platform architecture blueprint](platform-architecture.zh-TW.md) defines the sole target architecture, while the [implementation refactoring roadmap](implementation-refactor-roadmap.zh-TW.md) sequences delivery beneath it. A file, crate, or canonical spec is not sufficient evidence of a complete product path.

To exercise Remote Server, Desktop, and CLI from clean local data, follow
[Remote Server, Desktop, and CLI Getting Started](remote-getting-started.md) through setup, membership, sign-in, workspace, and recovery checks.

## Status model / 狀態模型

- **Available（可用）**: a user-operable entry point plus either two independent evidence sources or one end-to-end source.
- **Partial（部分可用）**: an operable subset exists, with an explicit gap in the complete workflow.
- **Planned（規劃中）**: only target design, foundation types, or an unclosed entry point exists; it is not current support.
- **Deprecated（已棄用）**: a compatibility or historical path remains visible but is no longer the target architecture.

## Capability matrix / 能力矩陣

| Capability / 能力 | Status / 狀態 | User entry / 使用者入口 | Evidence / 證據 | Limits and next step / 限制與下一步 | Checked / 查核 |
| --- | --- | --- | --- | --- | --- |
| Local Repo CLI | Available（可用） | `speclink init`, `list`, `show`, `status`, `validate`, `analyze`, `drift`, `archive`, and discussion verbs | [`speclink-cli` entry](../crates/speclink-cli/src/main.rs)<br>[CLI integration tests](../crates/speclink-cli/tests/doc_verbs.rs) | Local Repo needs no server. Check each subcommand's `--help` before using advanced flags. | 2026-07-17 |
| Generated Agent Skills | Partial（部分可用） | Claude `/speclink-*`; Codex `$speclink-*` | [Generated apply skill](../.agents/skills/speclink-apply/SKILL.md)<br>[Engine skill assets](../crates/speclink-core/assets/skills/verify.md) | The generated surface includes onboard/discuss/propose/apply/ingest/drift/audit/commit/archive. Assets such as `verify.md` exist in the engine, but this repository has no generated `$speclink-verify`; it must not be documented as callable. | 2026-07-17 |
| Local Desktop | Available（可用） | Tauri/React changes, specs, discussions, archives, tasks, settings, and tray UI | [Desktop scripts](../apps/desktop/package.json)<br>[Desktop UI tests](../apps/desktop/src/__tests__/App.test.tsx) | Local workspaces are usable; Remote Workspace completeness is listed separately. | 2026-07-17 |
| Node N-API SDK | Available（可用） | `@speclink/engine` Store bridge, rendering, and `dispatch` | [Node package entry](../crates/speclink-node/package.json)<br>[Dispatch contract tests](../crates/speclink-node/__test__/dispatch-contract.spec.ts) | This is the Engine/Store bridge. The full Phase 4 Node Host and Copilot Tool packages are not delivered. | 2026-07-17 |
| Command Runtime, Host and Protocol | Available（可用） | Shared Rust crates used by CLI, Server, and Node adapters | [Host dual-path tests](../crates/speclink-host/tests/bridge_dual_path.rs)<br>[Client Protocol spec](../openspec/specs/client-protocol/spec.md) | Typed command/query/context foundations exist. Agent-ecosystem packaging and some advanced gates remain Partial or Planned. | 2026-07-17 |
| SQLite TeamStore | Available（可用） | Default `sqlite` driver for `speclink-server` | [SQLite conformance tests](../crates/speclink-store-sqlite/tests/conformance.rs)<br>[Driver guide](server-store-drivers.zh-TW.md) | Single-instance positioning; cluster is not a current capability. | 2026-07-17 |
| Server FS TeamStore | Available（可用） | `serverfs` driver in server configuration | [Server FS conformance tests](../crates/speclink-store-fs/tests/conformance.rs)<br>[Atomic publish tests](../crates/speclink-store-fs/tests/atomic_publish.rs) | Requires reliable OS advisory-lock/flock semantics; one server per data directory. | 2026-07-17 |
| PostgreSQL TeamStore | Available（可用） | `postgres` driver in server configuration | [PostgreSQL conformance tests](../crates/speclink-store-postgres/tests/conformance.rs)<br>[Resilience tests](../crates/speclink-store-postgres/tests/resilience.rs) | Full tests require PostgreSQL 15 and `SPECLINK_TEST_POSTGRES_URL`; the supported server product remains single-instance. | 2026-07-17 |
| `speclink-server` | Available（可用） | Native binary or Docker; HTTP Command/Query/Context/Event APIs | [Server binary](../crates/speclink-server/src/main.rs)<br>[CLI-to-server E2E](../crates/speclink-server/tests/e2e_cli.rs) | The single-node server works. Remote task touched-file evidence is not fully persisted; see Verify and task evidence. | 2026-07-17 |
| Server Admin, setup and identity | Available（可用） | `/setup`, `/admin`, `/account`, PAT/device flow/invite, and headless admin commands | [Admin E2E tests](../crates/speclink-server/tests/admin_e2e.rs)<br>[Device-flow E2E tests](../crates/speclink-server/tests/device_e2e.rs) | Covers single-node installation and account management; SSO and cluster administration remain planned. | 2026-07-17 |
| Desktop Server Connections | Available（可用） | Desktop Server list, device login, PAT fallback, logout, and OS Keychain | [Tauri connection orchestration](../apps/desktop/src-tauri/src/connections.rs)<br>[Servers panel tests](../apps/desktop/src/__tests__/serversPanel.test.tsx) | Connections and identities can be managed, but login does not yet construct a complete Remote Workspace. | 2026-07-17 |
| Desktop Remote Workspace | Partial（部分可用） | Remote `WorkspaceLocator` type and Server connection UI | [Workspace locator](../apps/desktop/src/session.ts)<br>[Workspace-session spec](../openspec/specs/workspace-session/spec.md) | The remote locator has no construction path. Spec-only, remote+checkout, offline/conflict sessions still need a later change. | 2026-07-17 |
| Remote CLI and Context Projection | Available（可用） | `speclink link`, `auth`, `artifact`, and read-only `.speclink/context/` | [Remote CLI tests](../crates/speclink-cli/tests/remote_read_path.rs)<br>[Context materializer](../crates/speclink-host/src/projection.rs) | The current Client Protocol path works. This does not imply that Desktop Remote Workspace or remote evidence is complete. | 2026-07-17 |
| Verify and task evidence | Partial（部分可用） | `speclink task done`, local evidence, `validate`/`analyze`; the engine contains a verify workflow asset | [Host evidence implementation](../crates/speclink-host/src/evidence.rs)<br>[Phase 2 chain test](../crates/speclink-server/tests/phase2_chain.rs) | This repository has no generated `$speclink-verify`. The server currently discards touched files from remote task completion; an ignored defect case preserves the gap. | 2026-07-17 |
| Server operations | Available（可用） | Native/Docker/Compose, health/readiness, backup/verify-backup/restore | [Deployment guide](server-deployment.zh-TW.md)<br>[Backup E2E tests](../crates/speclink-server/tests/backup_e2e.rs) | Backup currently requires a maintenance window. There is no rolling upgrade or cluster operation. | 2026-07-17 |
| MCP and Copilot in-process tools | Planned（規劃中） | No installable `@speclink/copilot-tools` or MCP adapter yet | [Current workspace package inventory](../package.json)<br>[Phase 4 target](implementation-refactor-roadmap.zh-TW.md) | The README architecture sketch is not a current package. A later change must deliver tool adapters, identity closure, and E2E tests. | 2026-07-17 |
| SSO, runtime plugins and cluster mode | Planned（規劃中） | No product entry point yet | [Platform architecture blueprint](platform-architecture.zh-TW.md)<br>[Implementation refactoring roadmap](implementation-refactor-roadmap.zh-TW.md) | These are later platform/ecosystem capabilities. Server and drivers are currently positioned for one instance. | 2026-07-17 |
| Legacy remote REST v1 | Deprecated（已棄用） | Historical remote-client prototype | [Legacy-path inventory](implementation-refactor-roadmap.zh-TW.md)<br>[README architecture statement](../README.md) | It is not a compatibility burden or formal contract for the new Client Protocol. New guides provide migration direction, not usage instructions for this path. | 2026-07-17 |
| Advanced verb-contract user guide | Planned（規劃中） | No standalone user guide yet | [Canonical verb contract](../openspec/specs/verb-contract/spec.md)<br>[Client Protocol spec](../openspec/specs/client-protocol/spec.md) | `docs/verb-contract.md` does not exist. This change records the gap without creating an empty file or broken link. | 2026-07-17 |

## Verification baseline / 查核基線

Re-run the status audit as follows:

1. Run `speclink --help` and relevant subcommand help to inventory Local and Remote CLI surfaces.
2. Run `speclink-server --help` to inventory server, identity, and backup entry points.
3. Compare `.agents/skills/*/SKILL.md` with `crates/speclink-core/assets/skills/*.md` to distinguish an engine asset from a skill generated for the current Host.
4. Cross-check the workspace `Cargo.toml`, package scripts, integration/E2E/conformance tests, and canonical specs. Never mark a capability Available from crate presence alone.

Before this change, three documentation failures were directly reproducible: both READMEs called the executable `speclink-server` undelivered; both getting-started guides invoked a non-generated `speclink-verify`; and they described proposal, delta spec, design, and tasks as four fixed artifacts instead of following the schema DAG and `applyRequires`.

## Known documentation gap / 已知文件缺口

The advanced verb/Protocol contract currently has canonical specs but no standalone `docs/verb-contract.md` user guide. This is a recorded documentation gap, not a clickable document and not an absence of the underlying contract. A later change should derive the advanced guide from the canonical `verb-contract` and `client-protocol` specs.

## Target references / 目標參考

- [Complete SDD workflow](workflow.md): purpose, timing, branches, and recovery.
- [Platform architecture blueprint](platform-architecture.zh-TW.md): the sole target architecture (Traditional Chinese).
- [Implementation refactoring roadmap](implementation-refactor-roadmap.zh-TW.md): delivery sequence and gates beneath that architecture (Traditional Chinese).
- [Server deployment](server-deployment.zh-TW.md), [Store drivers](server-store-drivers.zh-TW.md), and [backup/restore](server-backup.zh-TW.md): current server operations (Traditional Chinese).

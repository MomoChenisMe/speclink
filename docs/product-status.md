# Product status

[繁體中文](product-status.zh-TW.md) · **English**

Last check: **2026-10-07** (0.8.0).

This document tells you if a capability is available now. The specs in `openspec/specs/` define the behavior. The [Roadmap](roadmap.md) lists the work that is not done yet. Code, a crate, or a spec alone does not make a capability available to users.

To try the Remote Server, the desktop app, and the CLI, follow [Remote getting started](remote-getting-started.md).

## <a id="status-model"></a>Status model

| Status | Meaning |
| --- | --- |
| Available | It has a user entry point and two independent pieces of evidence, or one end-to-end test. |
| Partial | Some of it works. The full flow has a known gap. |
| Planned | It has no usable entry point yet. For details, see the [Roadmap](roadmap.md). |
| Deprecated | The code still has it, but it is not part of the target architecture. No item has this status now. |

## <a id="local-and-remote"></a>Local and remote

This table holds all the differences between the two paths. The Remote Store column measures the official reference server, `speclink-server`. The Host and Protocol contracts define remote mode, so these columns also apply to a server that you build.

Most CLI verbs work in local and remote mode (Dual). The exceptions are below. For the full list, see the [Verb and flag contract](verb-contract.md):

- Local only: `demo`, `trace`, `change rank`, `plan --strict-overlap`.
- Remote only: `claim`.

In the wrong mode, the CLI rejects these verbs. It does not switch to the other mode.

| Capability | Local Repo | Remote Store | Note |
| --- | --- | --- | --- |
| Read and write specs and changes | Available | Available | Local mode reads and writes `openspec/` directly. Remote mode writes only through Host commands and keeps no second writable copy on your machine. |
| Change lifecycle (`propose` → `apply` → `archive`) | Available | Available | Remote mode archives one change at a time. |
| Discussions (`discuss`) | Available | Available | Promote, link to an existing change, and archive work the same in both modes. |
| Quality stations (`review`, `verify`) | Available | Available | Tickets, rounds, and stamps follow the same rules in both modes. |
| Execution order (`plan`, `change depends`) | Available | Available | `change rank` is local only. |
| Trace (`trace`) | Available | Not applicable | Local only. |
| Claim a change (`claim`) | Not applicable | Available | The Store keeps the claimant after a restart. A claim on a change that another person holds returns 409, and the message names the holder. |
| Demo data (`demo`) | Available | Not applicable | Local only. Remote mode rejects it and sends no request. |
| Agent context | Available | Available | Local mode reads the repo. Remote mode reads the read-only `.speclink/context/`. |
| Touched-file evidence for tasks | Available | Available | Local mode writes `.evidence.json`. Remote mode keeps it in the Store, and `GET /changes/{name}/evidence` returns it. |
| Desktop board and detail page | Available | Partial | When you check a task on a remote board in the desktop app, the app does not report touched files. |
| Accounts, access tokens, and membership | Not applicable | Available | Local mode needs no account. |
| Backup and restore | Git does it | Available | A remote backup needs a stopped server. |
| Offline work | Available | Needs a connection | When the connection drops, remote mode is read-only and rejects writes. |

## <a id="capabilities"></a>Capabilities

| Capability | Status | User entry | Evidence | Limits and next step | Checked |
| --- | --- | --- | --- | --- | --- |
| Local Repo CLI | Available | `speclink init`, `list`, `show`, `status`, `validate`, `analyze`, `drift`, `archive`, `discuss`, and more | [CLI entry](../crates/adapters/speclink-cli/src/main.rs)<br>[CLI integration tests](../crates/adapters/speclink-cli/tests/it/doc_verbs.rs) | No server is necessary. The `--help` of each subcommand is the reference for flags. | 2026-10-07 |
| Agent skills | Available | Claude, GitHub Copilot and Antigravity `/speclink-*`, Codex `$speclink-*` | [Generated apply skill](../.agents/skills/speclink-apply/SKILL.md)<br>[Generated manual skill](../.claude/skills/speclink-manual/SKILL.md)<br>[Custom tool descriptor tests](../crates/adapters/speclink-cli/tests/it/tools_descriptor.rs)<br>[Copilot skill snapshot](../crates/engine/speclink-core/tests/golden/copilot.snapshot.md) | Only Claude has the `analyze` skill; Codex, GitHub Copilot and Antigravity use the CLI. Skill count: with the `worktree` policy off, Claude has 18, and Codex, GitHub Copilot and Antigravity have 17 each. With it on, each gets 2 more. Other AI tools can get skills through a custom tool descriptor. | 2026-10-21 |
| Quality stations | Available | `/speclink-review`, `/speclink-verify`, `/speclink-quality`; CLI `speclink review`, `speclink verify` | [Station stamp and ticket rules](../crates/engine/speclink-core/src/quality/station.rs)<br>[Review verb tests](../crates/adapters/speclink-cli/tests/it/review_verbs.rs) | A SUGGESTION does not block the stamp. If a file in scope changes after the stamp, the stamp drops to "changed since". | 2026-10-07 |
| Execution order | Available | `speclink plan`, `speclink change depends`, `speclink change rank`; the Plan document in the desktop change detail page | [Plan verb tests](../crates/adapters/speclink-cli/tests/it/plan_verbs.rs)<br>[Server plan API tests](../crates/host/speclink-server/tests/it/api/plan_api.rs) | `change rank` and `plan --strict-overlap` are local only. | 2026-10-15 |
| Manual and trace | Available | `/speclink-manual`, the Manual page in the desktop app; `speclink trace`, `/speclink-trace` | [Manual page tests](../packages/ui/src/__tests__/manualPage.test.tsx)<br>[Trace tests](../crates/adapters/speclink-cli/tests/it/trace.rs) | Remote projects cannot generate a manual yet (the tour works). `trace` is local only. | 2026-10-07 |
| Local desktop app | Available | Board, detail pages, specs, archived, manual, settings mode (app settings, project settings, and the About page), and tray | [Desktop scripts](../apps/desktop/package.json)<br>[Desktop UI tests](../apps/desktop/src/__tests__/App.test.tsx)<br>[Settings mode tests](../apps/desktop/src/__tests__/settingsColumn.test.tsx)<br>[About page tests](../apps/desktop/src/__tests__/aboutPage.test.tsx)<br>[Auto-update tests](../apps/desktop/src/__tests__/updater.test.ts) | Remote projects have their own row below. | 2026-10-15 |
| Desktop colors and settings navigation | Available | Seven fixed dark/light palettes shared by the main window and tray; selecting a project in Settings opens its board | [Palette tests](../apps/desktop/src/__tests__/projectColors.test.ts)<br>[Navigation tests](../apps/desktop/src/__tests__/store.test.ts) | Colors stay on this device, separate from team policy. | 2026-10-19 |
| Desktop network and diagnostics | Available | Debug toggle, bounded logs and ZIP export, system/manual/direct proxy, explicit update connection test | [Network settings tests](../apps/desktop/src/__tests__/networkDiagnosticsPage.test.tsx)<br>[Proxy and error classification tests](../crates/host/speclink-host/src/network.rs) | [Fixed proxy scope and limits](configuration.md#desktop-network-diagnostics); enterprise authentication and custom CAs are unsupported. Company failures still need actual diagnostic evidence. | 2026-10-19 |
| Offline security standard exchange | Available | Desktop and CLI export one version or all valid versions; import v1 packages and v2 collections | [CLI round-trip tests](../crates/adapters/speclink-cli/tests/it/security.rs)<br>[Collection validation and atomic output tests](../crates/engine/speclink-fs/src/security_packages.rs) | [Collection import results](configuration.md#offline-standard-bundles) use per-version commits: failure or cancellation retains prior successes without a batch rollback guarantee. | 2026-10-19 |
| Install channels | Available | Desktop installers (macOS universal dmg, Windows installer, Linux AppImage); CLI through npm, install script, and Homebrew; server through npx and Docker | [npm launcher tests](../scripts/npm/npm-cli-launcher.test.mjs)<br>[Install script tests](../scripts/install.test.mjs)<br>[Homebrew formula generator](../scripts/release/homebrew-formula.mjs) | The Windows installer has no code signature. On the first run, you must allow it in SmartScreen. | 2026-10-07 |
| Node SDK (`@speclink/engine`) | Available | `npm install @speclink/engine` | [Package entry](../crates/adapters/speclink-node/package.json)<br>[Dispatch contract tests](../crates/adapters/speclink-node/__test__/dispatch-contract.spec.ts)<br>[Version tests](../scripts/npm/npm-engine-package.test.mjs) | On npm since 0.2.0, with five platform packages. Not done yet: typed JS methods, a JS remote client, an agent tool layer. See the [Roadmap](roadmap.md). | 2026-10-07 |
| Command Runtime, Host, and Protocol | Available | Rust crates that the CLI, the server, and the Node SDK share | [Host dual-path tests](../crates/host/speclink-host/tests/bridge_dual_path.rs)<br>[Client Protocol spec](../openspec/specs/client-protocol/spec.md) | The tool wrapper for agents is not done yet. | 2026-10-07 |
| SQLite TeamStore | Available | The default `sqlite` driver of `speclink-server` | [SQLite conformance tests](../crates/store/speclink-store-sqlite/tests/conformance.rs)<br>[Store driver choice](server-store-drivers.md) | One server instance only. | 2026-10-07 |
| Server FS TeamStore | Available | The `serverfs` driver in the server config | [Server FS conformance tests](../crates/store/speclink-store-fs/tests/it/conformance.rs)<br>[Atomic publish tests](../crates/store/speclink-store-fs/tests/it/atomic_publish.rs) | Needs reliable file locks (flock). One data folder allows one server. | 2026-10-07 |
| PostgreSQL TeamStore | Available | The `postgres` driver in the server config | [PostgreSQL conformance tests](../crates/store/speclink-store-postgres/tests/it/conformance.rs)<br>[Resilience tests](../crates/store/speclink-store-postgres/tests/it/resilience.rs) | The server is still one instance. | 2026-10-07 |
| `speclink-server` | Available | npx, Docker, Compose; a native binary that you build from source | [Server entry](../crates/host/speclink-server/src/main.rs)<br>[CLI-to-server end-to-end tests](../crates/host/speclink-server/tests/it/e2e_cli.rs) | Releases do not include a native binary. | 2026-10-07 |
| Server admin, setup, and accounts | Available | `/setup`, `/admin`, `/account`, access tokens, device login, invites, and command-line admin commands | [Admin end-to-end tests](../crates/host/speclink-server/tests/it/admin/e2e.rs)<br>[Device login end-to-end tests](../crates/host/speclink-server/tests/it/identity/device_e2e.rs) | No SSO. See the [Roadmap](roadmap.md). | 2026-10-07 |
| Server operations | Available | Deployment, health checks, `backup`, `verify-backup`, `restore` | [Deployment document](server-deployment.md)<br>[Backup end-to-end tests](../crates/host/speclink-server/tests/it/admin/backup_e2e.rs) | A backup needs a stopped server. No rolling upgrades and no multi-node mode. | 2026-10-07 |
| Remote CLI and read-only projection | Available | `speclink link`, `auth`, `artifact`; the read-only `.speclink/context/` | [Remote CLI tests](../crates/adapters/speclink-cli/tests/it/remote_read_path.rs)<br>[Projection code](../crates/host/speclink-host/src/projection.rs) | — | 2026-10-07 |
| Remote task evidence | Available | In remote mode, `speclink task done` keeps the touched files in the Store | [Evidence code](../crates/engine/speclink-core/src/lifecycle/tasks.rs)<br>[Remote evidence end-to-end tests](../crates/host/speclink-server/tests/it/phase2_chain.rs) | The remote board in the desktop app does not send touched files. | 2026-10-07 |
| Desktop server connections | Available | The Servers page in Settings, device login, access tokens, logout, and the OS Keychain | [Connection flow](../apps/desktop/src-tauri/src/connections.rs)<br>[Servers panel tests](../apps/desktop/src/__tests__/serversPanel.test.tsx) | — | 2026-10-15 |
| Desktop remote project | Partial | Remote open in the New project dialog: specs only, or with a local checkout; claim from the detail page | [New project dialog](../apps/desktop/src/components/WorkspaceChooser.tsx)<br>[Remote open tests](../apps/desktop/src/__tests__/remoteOpen.test.ts) | You can browse, check tasks, and read and write artifacts. Gaps: a checked task does not report touched files; a claim has no release or takeover verb. | 2026-10-07 |
| Claude Code plugin (`speclink-skills`) | Partial | `/plugin install speclink-skills --marketplace MomoChenisMe/speclink` | [Plugin tests](../integrations/claude-code/speclink-skills/tests/register.test.tsx)<br>[Skill group tests](../scripts/claude-code/skill-groups.test.mjs) | It uses the early-access plugin API of Claude Code, so a new Claude Code version can break it. Not tested on a real Windows machine yet. | 2026-10-07 |
| MCP and agent tool packages | Planned | No entry point | [Direction and next steps](roadmap.md) | See "Agent tool integration" in the roadmap. | 2026-10-07 |
| SSO, runtime plugins, and multi-node mode | Planned | No entry point | [Direction and next steps](roadmap.md) | See "System integration" in the roadmap. | 2026-10-07 |

## <a id="recheck"></a>How to check again

When you update this document, check again with the current checkout. Do not keep the result of an old check date:

1. Run `speclink --help` and the `--help` of each subcommand to check the CLI entry points.
2. Run `speclink-server --help` to check the server, account, and backup entry points.
3. Compare the folder lists of `.claude/skills/` and `.agents/skills/`. The only difference is `speclink-analyze` (Claude only).
4. Use the [verb contract spec](../openspec/specs/verb-contract/spec.md) to check the local and remote table.
5. Use `npm view @speclink/engine version` and similar commands to check the versions on npm.
6. Compare `Cargo.toml`, the package scripts, the integration and end-to-end tests, and the specs. A crate without a user entry point does not make a capability available.

## <a id="doc-gaps"></a>Known documentation gaps

- No document shows how to build a client from zero. For the direction, see "Build your own client" in the [Roadmap](roadmap.md).

## <a id="related"></a>Related documents

- [SDD workflow](workflow.md): the purpose, skill, done criteria, and next step of each stage.
- [Roadmap](roadmap.md): the work that is not done yet.
- [Server deployment](server-deployment.md), [Store drivers](server-store-drivers.md), [Backup and restore](server-backup.md): how to operate the server.

## <a id="offline-security-workflow"></a>Desktop workflow and offline security review (current implementation)

This fork now uses `mosutw/speclink` for updates, retains a separate original-author version/link, and offers manual Release downloads when signed update packages are unavailable. README credits distinguish mosutw's maintenance, Codex's AI assistance and the upstream author; GitHub's Contributors sidebar still follows commit attribution.

Security standard data has a device-level location setting with a local default and Windows UNC support. Desktop and same-machine CLI resolve the same location; only standards are shared. Read-only shares support review/export, custom-root deletion is disabled, active work blocks location changes, and disconnects do not cause local download fallback. Private reports and `.security_kb/` snapshots remain local. Local fixtures and Windows lock/rename tests pass; actual NAS/SMB and corporate Proxy acceptance remain pending. Finish checks, merge to `main`, build installers from that commit, then perform corporate acceptance. See [Development](development.md#desktop-tags) for operation and old-writer upgrade limits.

These entries are implemented in the current source; installed builds need a release containing this change. Remote requires `securityReviewV2` and `checkHistory` capabilities. Older servers explicitly disable new policy writes; unsupported never means passed.

1. Drag projects in the left rail or move them up/down. Order persists on this device across restarts.
2. Settings → Software updates reads bundled version and release notes independently of network checks. Manual checks distinguish Proxy, TLS, 407 and parse failures. The [corporate Proxy run](../openspec/changes/desktop-workflow-and-offline-security-review/evidence/company-proxy.md) remains pending; no corporate fix is claimed.
3. Project settings → Review rules → Default list → View 12 rules provides keyboard-accessible question icons explaining purpose, examples and effects.
4. First download or offline-import selected source documents, mappings and remediation guides in Security standard data. Run `speclink security kb prepare --root <project>` or prepare the knowledge base in Desktop to create `.security_kb/`. Missing resources fail without replacing an existing snapshot. Review never downloads resources.
5. Review uses only configured standards, versions and ASVS levels; it does not extend the selection.
6. Review and verification save completed records before stamping. All three check types live in proposal `checks/`. Desktop active and archived document trees provide read-only history; CLI `review history`, `verify history` and `security report ID --change CHANGE` read existing results without rescanning.
7. The three workflow retention options default to enabled and only control complete records included at archive time. Active proposals retain completed records; unfinished tickets carry forward. Existing archives are never retroactively pruned. Required acceptance archives also retain a readable minimal acceptance summary with digests and acceptance reasons, without full evidence or code snippets.
8. The resource registry `spec_registry.json` pins verified sources and SHA values. Attached data is structural reference, never executable instructions or an unverified source authority.
9. Built-in basic scanning stays available. Explicit `/speclink-security-review CHANGE` (Codex: `$speclink-security-review`) separately uses local knowledge for deep architecture, business logic and data-flow review in five languages, producing Traditional Chinese reports with nine finding groups. Basic scanning cannot replace the skill. Networked AI is not described as offline.
10. Include in acceptance is unchecked by default: reports are reference only. With both this checkbox and security review enabled, missing, stale or unverified evidence and unresolved CRITICAL/HIGH/MEDIUM findings block verification stamping and archiving; LOW is advisory. Explicit `security review accept CHANGE --stdin` requires report/finding IDs and a concrete reason. Generic `--accept` does not bypass security. Disabling the main switch preserves the checkbox value while suspending enforcement.
11. Review and verification record filenames start with the round's UTC start time (`YYYYMMDDHHmmss`); legacy ULID-only filenames remain readable. In Desktop change details, “Reviewed” and “Verified” in Quality gates independently expand records sorted by start time. Selecting an entry opens it in the existing read-only viewer.

Proposal sections use list display by default across every project, including existing prose, without rewriting stored documents. Remote basic scans can create offline pending drafts; only explicit deep submission publishes the paired reports. Reading application records from your own server is separate from downloading standards. Remote source freshness is marked client-attested and needs a fresh authorized checkout check; the server does not claim independent access to source code. Passing covers the evidence and rules actually checked, and is not a security certification. An archived acceptance summary cannot stand in for a new acceptance review.

Evidence: [archive retention](../openspec/changes/desktop-workflow-and-offline-security-review/evidence/archive-retention.md), [Remote security](../openspec/changes/desktop-workflow-and-offline-security-review/evidence/remote-security.md), [Desktop security](../openspec/changes/desktop-workflow-and-offline-security-review/evidence/desktop-security.md).

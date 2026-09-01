<p align="center">
  <img src="docs/assets/brand/transparent/speclink-logo-horizontal.png" alt="Speclink" width="440" />
</p>

<p align="center">
  <b>One SDD Engine, for both Local Repo and Remote Store</b>
</p>

<p align="center">
  <a href="README.md">繁體中文</a> · <b>English</b>
</p>

Speclink is a Spec-Driven Development (SDD) engine and tooling platform written in Rust. PMs, POs, engineers,
and AI Agents all use one vocabulary here: change, artifact, task, verify, archive. It also keeps two deployment
paths open:

- **Local Repo**: specs live in the repo's `openspec/`, collaborate through Git, no server required.
- **Remote Store**: specs live in a shared Store, with Speclink Host handling authentication, revisions, transactions, events, and workflow adjudication. Host and Protocol are both public contracts, so you can run the official server or write your own.

**Local mode** keeps the OpenSpec directory structure on purpose: `specs/<capability>/spec.md`, `changes/<name>/`,
`changes/archive/`, and `config.yaml`. Every file is plain Markdown or YAML. There is no database and no proprietary
format. You can read and edit the files without Speclink, and Git shows a diff for every spec change. Speclink adds
only two things to this structure: `discussions/` for discussion records, and an `.openspec.yaml` per change for
lifecycle metadata.

The Local CLI took the CLI bundled with [Spectra App 2.3.1](https://github.com/kaochenlong/spectra-app) as its
behavioral reference. Golden and CLI integration tests hold the human-readable output, the `--json` shapes, and
the core workflow. On that base Speclink adds discussions, Desktop, Store abstraction, a Node SDK, and the
Remote Platform.

Specs are not documents you write once and abandon. The desktop app puts every change on a board, so you can see
which station it stands at, how far its tasks moved, and what its specs changed:

![The Speclink desktop change board with the change detail drawer](docs/assets/screenshots/desktop-board.png)

(Screenshots are captured with the interface in Traditional Chinese; the interface language is switchable in settings.)

## Current capabilities / 目前能力

- **Available:** Local Repo CLI, Local Desktop, generated Agent skills (every station, for both Claude and Codex), Command Runtime/Host/Protocol, SQLite/Server FS/PostgreSQL TeamStore, single-node Server with Admin and Auth, Remote CLI and Context Projection, remote task evidence, Server operations (deployment, backup and restore), and the desktop and CLI install channels.
- **Partial:** the Node SDK (the binding works and the npm publishing pipeline is wired; it reaches npm with the first release that carries the engine) and the Desktop Remote Workspace.
- **Planned:** MCP and Copilot in-process tools, SSO, runtime plugins, and cluster mode.
- **Deprecated:** the legacy remote REST v1 prototype; new work uses the current Client Protocol and Host path.

Per-item evidence, limits, and the last audit date are not duplicated here — [Project Capability Status](docs/product-status.md) is the canon. For where things are heading, see the [Project Roadmap](docs/roadmap.md).

## SDD workflow / SDD 工作流

```text
baseline? → discuss?/improve? → propose → apply ⇄ ingest → (quality? | review? ∥ verify?) → archive
                                            ↑
                                  resuming after a pause: drift first

worktree: apply-with-worktree ⇄ ingest → (quality? | review? ∥ verify?) → worktree-merge → archive

utilities: validate / analyze / audit / commit / config
```

Where you enter depends on what you have:

- The requirement is already clear → go straight to `propose`
- The requirement still needs convergence → `discuss` (you bring the topic) or `improve` (you ask the model to find topics)
- Requirements shift mid-implementation → `ingest`
- The change sat idle and you resume it → run `drift` first

Two optional quality stations sit before archiving: `review` for craft and `verify` for compliance. Combine them as the risk warrants. To skip both on a low-risk change is a legitimate choice.

When several non-conflicting changes should move at once, take the worktree flow. You implement each change in its own git worktree, without interference. `worktree-merge` then lands the branch on the main branch before you archive.

For each station's purpose, its `/speclink-*` skill, completion criteria, and next station — plus discussion outcome routing and recovery paths — see the [Complete SDD Workflow](docs/workflow.md).

## Install / 安裝

The desktop app and the CLI are **two ways to use the same engine — pick one**. Install the desktop app if you want the board, specs, and discussions on screen. Install only the CLI if you do not want a graphical interface, or if you want Speclink in scripts and CI. You lose no capability either way.

The Server is a third piece. You **need it only when a team shares one spec canon**. Alone in your own repo you never need it.

**Desktop app** — download the installer for your platform from [Releases](https://github.com/MomoChenisMe/speclink/releases/latest):

| Platform | Installer |
| --- | --- |
| macOS | `Speclink_<version>_aarch64.dmg` (Apple Silicon), `Speclink_<version>_x64.dmg` (Intel) |
| Windows | `Speclink_<version>_x64-setup.exe` |
| Linux | `.AppImage` (portable) or `.deb`, each for x86_64 and aarch64 |

The desktop installer bundles a matching CLI, installable to your PATH from the app's settings.

**Read this if you already installed the CLI. Install the CLI first and the desktop app second, and your
`speclink` becomes the desktop app's version.**

Both use the path `~/.local/bin/speclink`. The behavior differs per platform:

| Platform | What the desktop app does to `~/.local/bin/speclink` |
| --- | --- |
| macOS | Deletes the existing file at every start and writes a symlink to its bundled CLI |
| Linux AppImage | Replaces it only on a version mismatch |
| Windows | Leaves this path alone; the installer manages the PATH |
| Linux `.deb` | Leaves this path alone; the package deploys to `/usr/bin` |

A version pinned with `SPECLINK_INSTALL_VERSION` goes away too. To keep your own CLI, install it to a different
directory with `SPECLINK_INSTALL_DIR`. Then put that directory before `~/.local/bin` in your PATH.

**CLI** — pick one:

```bash
# Install script (macOS/Linux)
curl -fsSL https://raw.githubusercontent.com/MomoChenisMe/speclink/main/scripts/install.sh | sh

# Install script (Windows PowerShell)
irm https://raw.githubusercontent.com/MomoChenisMe/speclink/main/scripts/install.ps1 | iex

# Homebrew (macOS/Linux)
brew install MomoChenisMe/tap/speclink
```

The install script detects your platform, checks the SHA-256, and places `speclink` in `~/.local/bin` (a user-level directory on Windows). `SPECLINK_INSTALL_DIR` changes the location. `SPECLINK_INSTALL_VERSION` pins a version.

Windows installers are not code-signed yet, so SmartScreen warns on first run — choose "More info" then "Run anyway".

**Server** (only needed when a team shares one canon) — `speclink-server` is the official **reference implementation**. Use it out of the box, or to try the remote features. Pick one of three shapes; each prints a one-time `/setup` link on first start (a normal restart does not reprint it):

| Shape | Command |
| --- | --- |
| npx (anywhere Node runs) | `npx @speclink/server` |
| Docker | `docker run -d -p 8080:8080 -v speclink-data:/data ghcr.io/momochenisme/speclink-server:latest` |
| Compose | `cd deploy && docker compose up -d` |

The default is SQLite with data under `./speclink-data` (`/data` inside a container). Environment variables, the PostgreSQL profile, and upgrade and rollback steps are in [Server Deployment](docs/server-deployment.zh-TW.md) (Traditional Chinese only).

**Remote mode does not bind you to this server.** Two public contracts, Host and Protocol, define where the spec canon lives and who guards it. The official server is one implementation of those contracts. To plug in your own authentication, database, or permission model, build your own server on the Speclink engine — the CLI and the desktop app still connect to it. The contracts are `client-protocol` and `host-runtime` under `openspec/specs/`, and the [Node SDK](docs/sdk-node.md) shows how to load the engine.

## Local Repo quick start / Local Repo 快速開始

In the repo you want to adopt Speclink in:

```bash
speclink init --tools claude,codex
speclink list
```

Then call `/speclink-propose <change>` in Claude, or `$speclink-propose <change>` in Codex; the Agent creates the
required artifacts from the schema DAG. For a copyable first loop with the direct CLI equivalents, see
[Getting Started](docs/getting-started.md).

## Deployment paths / 部署路徑

- **Local Repo:** Embedded Rust Runtime → FsStore → `openspec/` → Git. Suited to a single repo and local or offline collaboration.
- **Remote Store:** CLI/Desktop/other clients → Speclink Host → the same Rust Runtime → TeamStore. Suited to a shared spec canon with centralized authentication, revisions, transactions, and events. That Host can be the official `speclink-server`, or your own implementation of the Protocol.

A Remote Store never syncs into a second writable local truth. An Agent with a checkout only reads
`.speclink/context/`, and remote writes still go through Host commands. The specs under `openspec/specs/` define
these boundaries: `host-runtime`, `client-protocol`, `teamstore-contract`, `context-projection`, and others. For
the full path from setup to sign-in, see [Remote Getting Started](docs/remote-getting-started.md).

## Documentation map / 文件地圖

**Working alone? These three are enough to start.**

| Document | Purpose |
| --- | --- |
| [Getting Started](docs/getting-started.md) | The copyable first Local Repo loop |
| [Complete SDD Workflow](docs/workflow.md) | Every station's purpose, skill, completion criteria, and next station |
| [Project Capability Status](docs/product-status.md) | Available/Partial/Planned/Deprecated, with evidence and limits |

**Only needed when a team shares one spec canon**

| Document | Purpose |
| --- | --- |
| [Remote Server, Desktop, and CLI Getting Started](docs/remote-getting-started.md) | The full path from a one-line `npx @speclink/server` start through `/setup`, membership (including the first Admin's self-grant), sign-in, Desktop/CLI, and recovery |
| [Server Deployment](docs/server-deployment.zh-TW.md) | npx/Docker/Compose and upgrades (Traditional Chinese only) |
| [Server Store Drivers](docs/server-store-drivers.zh-TW.md) | Choosing between SQLite/Server FS/PostgreSQL (Traditional Chinese only) |
| [Server Backup and Restore](docs/server-backup.zh-TW.md) | backup/verify-backup/restore (Traditional Chinese only) |

**Look these up when something needs adjusting**

| Document | Purpose |
| --- | --- |
| [Configuration](docs/configuration.md) | Where Local and Remote settings live, and the current fields |
| [Development Entries](docs/development.md) | One-command dev environments, the checkout CLI, and test commands |
| [Project Roadmap](docs/roadmap.md) | Where things are heading: the SDK, building your own client, remote collaboration, Agent tools, system integration |
| [Brand Assets](docs/assets/brand/README.md) | Logo, palette, and usage |

**Needed only to drive Speclink from your own program, or to build your own client.** Just use the desktop app or the CLI? Skip both.

| Document | Purpose |
| --- | --- |
| [Node SDK](docs/sdk-node.md) | How to load `@speclink/engine`, the Store bridge, and the dispatch surface |
| [Verb and Flag Contract](docs/verb-contract.md) | Verb mode assignment, cross-mode output parity, and endpoint payload and error shapes |

`openspec/changes/archive/` and `openspec/discussions/archive/` are historical audit data, not a current manual.

## Development / 開發

Build the CLI from source (a stable Rust toolchain is required):

```bash
cargo install --path crates/speclink-cli
speclink --version
```

[Development Entries](docs/development.md) holds three things: the four one-command dev entries (full
`npm run dev`, server only, desktop only, the checkout CLI), the complete test commands, and the bypass steps for
unsigned installers.

## License / 授權

[MIT](LICENSE)

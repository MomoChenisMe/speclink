<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="docs/assets/brand/transparent/speclink-logo-horizontal-dark.png" />
    <img src="docs/assets/brand/transparent/speclink-logo-horizontal.png" alt="Speclink" width="440" />
  </picture>
</p>

<p align="center">
  <b>One SDD Engine for Local Repo and Remote Store</b>
</p>

<p align="center">
  <a href="https://github.com/MomoChenisMe/speclink/actions/workflows/ci.yml"><img src="https://github.com/MomoChenisMe/speclink/actions/workflows/ci.yml/badge.svg" alt="CI" /></a>
  <a href="https://github.com/MomoChenisMe/speclink/releases/latest"><img src="https://img.shields.io/github/v/release/MomoChenisMe/speclink?label=release" alt="Release" /></a>
  <a href="https://www.npmjs.com/package/@speclink/cli"><img src="https://img.shields.io/npm/v/@speclink/cli?label=npm" alt="npm" /></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue" alt="License: MIT" /></a>
</p>

<p align="center">
  <a href="README.zh-TW.md">繁體中文</a> · <b>English</b>
</p>

Speclink is a Spec-Driven Development (SDD) engine and tool platform written in Rust. You write down what a change must do before you build it. Then you and your AI agent work from that spec until the change is done. PMs, POs, engineers, and AI agents use one shared set of terms: change, artifact, task, verify, and archive.

Speclink has two deployment paths:

- **Local Repo**: Specs live in the `openspec/` folder of your repo. Git handles collaboration. No server is necessary.
- **Remote Store**: Specs live in a shared Store. A Host handles authentication, revisions, transactions, events, and workflow decisions. The Host and Protocol are public contracts, so you can use the official server or write your own.

![The Speclink desktop app: a board with the columns Discussions, Proposed, In progress, and Ready](docs/assets/screenshots/en/desktop-board.png)

## <a id="why"></a>Why Speclink

- **The specs live next to your code.** They are plain Markdown and YAML files in your repo. You can read them without Speclink, and every edit shows in the Git diff.
- **Your AI agent knows each step.** Speclink writes skills for Claude Code, Codex, and GitHub Copilot. Each skill tells the agent what to read, what to write, and when to stop and ask you.
- **You check the work before it lands.** Two optional quality stations run before archive: `review` checks the code craft, and `verify` checks the delivery against the specs.
- **One workflow from one person to a team.** Start in one repo with Git. When the team grows, connect a shared server. The workflow and the terms stay the same.

## <a id="quick-start"></a>Quick start

1. Install the CLI:

   ```bash
   npm i -g @speclink/cli
   ```

   For Homebrew, the install script, and the desktop app, see [Install](#install).

2. Initialize the repo where you want to use Speclink:

   ```bash
   speclink init --tools claude,codex
   ```

   `--tools` lists the agent tools that you use: `claude`, `codex`, and `copilot`, in any comma-separated combination.

3. Ask your agent to start a change. In Claude Code or GitHub Copilot, enter `/speclink-propose <change-name>`. In Codex, enter `$speclink-propose <change-name>`. The agent writes the proposal, the specs, and the task list.

For the full first round (propose, apply, check, archive), see [Getting started](docs/getting-started.md). To connect to a shared server, see [Remote getting started](docs/remote-getting-started.md).

## <a id="screenshots"></a>Screenshots

<table>
  <tr>
    <td width="50%"><img src="docs/assets/screenshots/en/desktop-change-detail.png" alt="The detail page of a change, with the document tree, the proposal, and the change details" /><br /><sub>A change: its documents, tasks, and quality stations</sub></td>
    <td width="50%"><img src="docs/assets/screenshots/en/desktop-manual.png" alt="A manual page in the desktop app, generated from the specs" /><br /><sub>A manual that Speclink generates from the specs</sub></td>
  </tr>
  <tr>
    <td width="50%"><img src="docs/assets/screenshots/en/desktop-spec.png" alt="The detail page of a spec, with its requirements and scenarios" /><br /><sub>A spec with its requirements and scenarios</sub></td>
    <td width="50%"><img src="docs/assets/screenshots/en/server-overview.png" alt="The overview page of the server console" /><br /><sub>The console of the self-hosted server</sub></td>
  </tr>
  <tr>
    <td width="50%" align="center"><img src="docs/assets/screenshots/en/desktop-tray.png" width="200" alt="The Speclink tray panel, opened from the macOS menu bar, with the changes grouped by Proposed, In progress, Ready, and Discussions" /><br /><sub>The tray panel in the macOS menu bar</sub></td>
    <td width="50%"><img src="docs/assets/screenshots/en/claude-code-panel.png" alt="Claude Code with the speclink-skills skill bar above the input box and the side panel open on the Board tab" /><br /><sub>The Claude Code plugin: skill bar and side panel</sub></td>
  </tr>
</table>

## <a id="features"></a>Features

- **Plain text, OpenSpec-compatible**: Local mode uses the OpenSpec folder layout: `specs/<capability>/spec.md`, `changes/<name>/`, `changes/archive/`, and `config.yaml`. All content is Markdown and YAML. You can read and edit it without Speclink, and each change shows in the Git diff. Speclink adds only two items: `discussions/` (discussion records) and a `.openspec.yaml` file in each change (lifecycle metadata). This applies to Local mode only. In remote mode, the specs live in the Store, and your machine has only a read-only projection.
- **Skills for AI agents**: `speclink init` writes skill files for Claude Code, Codex, and GitHub Copilot. Each stage, from discussion to archive, has a `/speclink-*` command (`$speclink-*` in Codex).
- **Desktop board**: Each change is a card. You can see its stage, its task progress, and the spec changes it makes.
- **Quality stations**: `review` checks code craft. `verify` checks that the delivery matches the specs. Both stations are optional.
- **Execution order**: `speclink plan` puts changes into waves from their declared dependencies and shows the next change that is ready to start.
- **Manual and trace**: Speclink generates a readable manual from the specs, and `trace` shows how a capability came to be.
- **Team sharing**: After you connect a server, the CLI, the desktop app, and AI agents read and write the same specs.

## <a id="workflow"></a>How it works

```text
baseline? → discuss?/improve? → propose → apply ⇄ ingest → (quality? | review? ∥ verify?) → archive
                                            ↑
                                    resume after a pause: drift first

worktree: apply-with-worktree ⇄ ingest → (quality? | review? ∥ verify?) → worktree-merge → archive

tools: validate / analyze / audit / commit / config / manual / trace / plan
```

A stage with `?` is optional. Your situation tells you where to start:

| Situation | Entry |
| --- | --- |
| The requirement is clear | `propose` |
| The requirement needs decisions | `discuss` (you bring the topic) or `improve` (the model finds topics) |
| The code has no specs yet | `baseline` |
| The requirement changes during the work | `ingest` |
| You resume a change after a pause | `drift` first |
| You want to move several independent changes at the same time | the worktree flow |

Before archive, two optional quality stations are available: `review` checks craft, and `verify` checks the specs. Choose the stations from the risk. For a low-risk change, you can skip both.

For the purpose, skill, done criteria, and next step of each stage, see the [SDD workflow](docs/workflow.md).

## <a id="install"></a>Install

The desktop app and the CLI are two ways to use the same engine. Pick one:

- To see the board, specs, and discussions, install the desktop app. The installer includes the CLI of the same version.
- To work without a GUI, or in scripts and CI, install only the CLI. It has all the features.

### Desktop app

Download the installer for your platform from [Releases](https://github.com/MomoChenisMe/speclink/releases/latest):

| Platform | Installer |
| --- | --- |
| macOS | `Speclink_<version>_universal.dmg` (one file for Apple Silicon and Intel) |
| Windows | `Speclink_<version>_x64-setup.exe` |
| Linux desktop | `Speclink_<version>_amd64.AppImage` (x86_64) or `Speclink_<version>_aarch64.AppImage` (arm64), no install step |

- The Windows installer has no code signature. On the first run, SmartScreen shows a warning. Click "More info", then "Run anyway".
- On Linux without a GUI (servers, WSL, CI), use the CLI below, not the desktop app.

### CLI

All three methods install the same binary (the source is `@speclink/cli` on npm). Pick one:

```bash
# With Node.js (any platform)
npm i -g @speclink/cli

# macOS or Linux without Node.js (servers, WSL, CI)
curl -fsSL https://raw.githubusercontent.com/MomoChenisMe/speclink/main/scripts/install.sh | sh

# Homebrew (macOS or Linux)
brew install MomoChenisMe/tap/speclink
```

Windows has no install script. With Node.js, use npm. Without Node.js, install the desktop app. Its installer also puts the CLI on your PATH.

Do you want both the CLI and the desktop app? The desktop app can replace your CLI. Read [Install in Getting started](docs/getting-started.md#install) first. That section also lists the install script options and the move to the AppImage for older Linux installs.

### Claude Code plugin (optional)

`speclink-skills` adds a row of speclink skill buttons above the Claude Code input box. It also adds a side panel that lists changes, discussions, and open quality tickets. In Claude Code, enter:

```text
/plugin install speclink-skills --marketplace MomoChenisMe/speclink
```

<table>
  <tr>
    <td width="50%"><img src="docs/assets/screenshots/en/claude-code-quality.png" alt="The Quality tab of the side panel, with an open review ticket on add-keyboard-shortcuts and its two findings" /><br /><sub>The Quality tab lists the open review and verify tickets</sub></td>
    <td width="50%"><img src="docs/assets/screenshots/en/claude-code-hash-menu.png" alt="After /speclink-apply, typing # opens a menu of proposed changes above the input box" /><br /><sub>Type <code>#</code> to pick a change name</sub></td>
  </tr>
</table>

For requirements, settings, and known limits, see [speclink-skills](integrations/claude-code/speclink-skills/README.en.md).

## <a id="self-hosting"></a>Self-hosting a server

You need a server only when a team shares one set of specs. For one person in one repo, you do not need a server.

`speclink-server` is the official **reference implementation**. Use it to start fast or to try the remote features. Pick one way to run it:

| Method | Command |
| --- | --- |
| npx (needs only Node.js) | `npx @speclink/server` |
| Docker | `docker run -d -p 8080:8080 -v speclink-data:/data ghcr.io/momochenisme/speclink-server:latest` |
| Docker Compose | `cd deploy && docker compose up -d` |

On the first start, the server prints a one-time `/setup` link. A normal restart does not print it again. The default Store is SQLite, and the data goes to `./speclink-data` (`/data` in the container). For environment variables, PostgreSQL, upgrades, and rollbacks, see [Server deployment](docs/server-deployment.md). To connect the desktop app and the CLI, see [Remote getting started](docs/remote-getting-started.md).

Remote mode does not depend on this server. Two public contracts define remote mode: `host-runtime` and `client-protocol` in `openspec/specs/`. You can write your own server with the Speclink engine and connect your own authentication, database, and permission model. The CLI and the desktop app work with it too. To load the engine, see the [Node SDK](docs/sdk-node.md).

## <a id="docs"></a>Documentation

**For one person, start with these three**

| Document | Content |
| --- | --- |
| [Getting started](docs/getting-started.md) | From install to your first archive |
| [SDD workflow](docs/workflow.md) | The purpose, skill, done criteria, and next step of each stage |
| [Configuration](docs/configuration.md) | `.speclink.yaml`, `openspec/config.yaml`, and remote settings |

**For a team that shares one set of specs**

| Document | Content |
| --- | --- |
| [Remote getting started](docs/remote-getting-started.md) | From server start to a connected desktop app and CLI |
| [Server deployment](docs/server-deployment.md) | npx, Docker, Compose, upgrades, and rollbacks |
| [Server Store drivers](docs/server-store-drivers.md) | How to choose SQLite, Server FS, or PostgreSQL |
| [Server backup and restore](docs/server-backup.md) | `backup`, `verify-backup`, and `restore` |

**To connect Speclink to your own program**

| Document | Content |
| --- | --- |
| [Node SDK](docs/sdk-node.md) | Load `@speclink/engine`, the Store interface, and `dispatch` |
| [Verb and flag contract](docs/verb-contract.md) | Local and remote verbs, output shapes, and HTTP endpoints |

**Project status and direction**

| Document | Content |
| --- | --- |
| [Product status](docs/product-status.md) | What is available and what is partial, with evidence and limits |
| [Roadmap](docs/roadmap.md) | The work that comes next |
| [Changelog](CHANGELOG.md) | The changes in each version |
| [Brand assets](docs/assets/brand/README.en.md) | Logo, colors, and usage |

`openspec/` holds the specs of Speclink itself. `openspec/changes/archive/` and `openspec/discussions/archive/` are history records, not user guides.

## <a id="status"></a>Project status

- **Available**: Local Repo CLI, Local desktop app, agent skills (Claude, Codex, and GitHub Copilot), quality stations, execution order, manual and trace, Node SDK (`@speclink/engine`), three TeamStores (SQLite, Server FS, PostgreSQL), single-node server and admin console, Remote CLI, install channels, server operations (deployment, backup, and restore).
- **Partial**: Remote workspaces in the desktop app, the Claude Code plugin.
- **Planned**: Agent tool packages and MCP, SSO, runtime plugins, multi-node deployment.

For the evidence, limits, and check date of each item, see [Product status](docs/product-status.md). For the work that is not done yet, see the [Roadmap](docs/roadmap.md).

## <a id="contributing"></a>Contributing

Issues and pull requests are welcome, in English or Traditional Chinese.

- Read [CONTRIBUTING.md](CONTRIBUTING.md) for the dev setup, the tests to run, the commit style, and how changes go through Speclink itself.
- Do not report a security problem in a public issue. Follow [SECURITY.md](SECURITY.md) to report it privately.
- For the one-command dev environment (`npm run dev` and four more entry points), the full test commands, and the repo layout, see [Development](docs/development.md).

To build the CLI from source (you need a stable Rust toolchain):

```bash
cargo install --path crates/adapters/speclink-cli
speclink --version
```

## <a id="acknowledgments"></a>Acknowledgments

- [Spectra App 2.3.1](https://github.com/kaochenlong/spectra-app): the Local CLI started with its CLI as the behavior reference. Golden tests and CLI integration tests protect the human output, the `--json` shape, and the core workflow. On top of that base, Speclink adds discussions, a desktop app, a Store abstraction, a Node SDK, and a remote platform.
- [OpenSpec](https://github.com/Fission-AI/OpenSpec): Local mode uses the OpenSpec folder layout for specs and changes.

## <a id="license"></a>License

[MIT](LICENSE)

# @speclink/cli

The command-line tool of [Speclink](https://github.com/MomoChenisMe/speclink), a Spec-Driven Development (SDD) engine written in Rust.

Speclink makes AI-built changes checkable. Each change starts with a small spec that says what the change must do. Then an AI agent builds it. Before archive, two optional quality stations, review and verify, check the work and record the evidence. The specs are plain Markdown and YAML files in your repo.

## Install

```bash
npm i -g @speclink/cli
speclink --version
```

You need Node.js 18.19+, 20.10+, or 21 or later. Prebuilt binaries are available for macOS (arm64, x64), Linux (x64, arm64), and Windows (x64). The install downloads only the binary for your platform, and you do not need Rust.

For Homebrew, the install script, and the desktop app, see [Install](https://github.com/MomoChenisMe/speclink#install) in the main README. Do you also use the desktop app? The desktop app can replace this CLI. Read [Install in Getting started](https://github.com/MomoChenisMe/speclink/blob/main/docs/getting-started.md#install) first.

## Quick start

1. Initialize the repo where you want to use Speclink:

   ```bash
   speclink init --tools claude,codex
   ```

   `--tools` lists the agent tools that you use: `claude`, `codex`, `copilot`, `pi`, and `cursor`, in any comma-separated combination. Speclink writes one skill for each workflow stage into each tool.

2. Ask your agent to start a change. In Claude Code, GitHub Copilot, or Cursor, enter `/speclink-propose <change-name>`. In Codex, enter `$speclink-propose <change-name>`. In pi, enter `/skill:speclink-propose <change-name>`.

3. The agent writes the proposal, the specs, and the task list. Continue with `/speclink-apply`, then `/speclink-archive`.

For the full first round (propose, apply, check, archive), see [Getting started](https://github.com/MomoChenisMe/speclink/blob/main/docs/getting-started.md).

## Common commands

| Command | What it does |
| --- | --- |
| `speclink init` | Initialize Speclink in a project and write the agent skills |
| `speclink update` | Update the skill files after you upgrade the CLI |
| `speclink list` | List the changes or the specs |
| `speclink show <name>` | Show a change or a spec |
| `speclink validate` | Check the changes and the specs for format errors |
| `speclink plan` | Show the execution order of the active changes and the next change that is ready |
| `speclink archive <change>` | Archive a completed change and fold its spec changes into the specs |

Run `speclink --help` for all commands. Most commands accept `--json` for scripts and AI agents.

## Remote projects

In a remote project, one Server holds the specs, and the team shares them. Connect a repo to the Server, then log in:

```bash
speclink link <project-url> --repo <repo-name>
speclink auth login
```

After that, the same commands read and write the specs on the Server. See [Remote getting started](https://github.com/MomoChenisMe/speclink/blob/main/docs/remote-getting-started.md).

## Documentation

- [Getting started](https://github.com/MomoChenisMe/speclink/blob/main/docs/getting-started.md): from install to your first archive
- [SDD workflow](https://github.com/MomoChenisMe/speclink/blob/main/docs/workflow.md): the purpose, skill, done criteria, and next step of each stage
- [Configuration](https://github.com/MomoChenisMe/speclink/blob/main/docs/configuration.md): `.speclink.yaml`, `speclink/config.yaml`, and remote settings
- [Verb and flag contract](https://github.com/MomoChenisMe/speclink/blob/main/docs/verb-contract.md): the commands, their flags, and their `--json` output
- [Changelog](https://github.com/MomoChenisMe/speclink/blob/main/CHANGELOG.md)

## License

[MIT](https://github.com/MomoChenisMe/speclink/blob/main/LICENSE)

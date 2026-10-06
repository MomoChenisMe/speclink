# speclink-skills: a Claude Code mod for Speclink

[繁體中文](README.md)

It adds two things to Claude Code:

- **Skill bar**: a row of speclink skill buttons above the prompt, in five tabs that follow the workflow: Plan, Build, Quality, Ship and More. A click puts the `/speclink-…` command into the prompt box, and keeps what you already typed behind it as the arguments. `archive+commit` on the Ship tab puts in `/speclink-archive + /speclink-commit` in one click.
- **Side panel**: press "Panel" on the skill bar, or type `/speclink-panel`, to open or close it. The panel lists the next change, the discussions, and the proposed, in-progress and ready changes (wave, worktree, task progress, and what blocks them). A click on a name puts it into the prompt box as the argument of the command at the front.

## Requirements

- Claude Code 2.1.290 or later (tested on this version). The mod API is early access, so a Claude Code release can make an update of this mod necessary.
- The `speclink` CLI on PATH. The panel reads the `--json` output of `speclink list`, `speclink plan` and `speclink discuss list`.
- A project where `speclink init` ran. The skill bar shows only the speclink skills that the project has; a project without them shows no skill bar.
- Mouse clicks on the buttons need Claude Code's fullscreen layout (`"tui": "fullscreen"` in settings.json).

## Install

In the Claude Code prompt, type:

```
/plugin install speclink-skills --marketplace MomoChenisMe/speclink
```

The first time, Claude Code asks to add the `github:MomoChenisMe/speclink` marketplace: answer `y`, and pick the user scope so that every project loads it.

Update: run `claude plugin update speclink-skills`, then restart Claude Code.

## Settings

**speclink-skills → Language** in `/config`:

| Value | Effect |
| --- | --- |
| `auto` (default) | Follows Claude Code's `language` setting: Chinese gives Traditional Chinese, anything else gives English |
| `en` | English |
| `zh-TW` | Traditional Chinese |

## Known limits

- **The panel shows no review or verify stamps**: the CLI output has no stamp state. `list --json` leaves those fields out on purpose, and a stamp removes the ticket, so `review show` fails after a stamp too. The stamp is recorded only in the change's `.openspec.yaml`; use the desktop app to see stamps.
- **`archive+commit`**: Claude Code treats only the first slash command as a command. It runs `/speclink-archive`, and `+ /speclink-commit …` becomes its arguments, so the model commits after the archive.
- **Windows**: the panel runs the CLI through `cmd.exe /d /c speclink …`. The npm-installed `speclink` is a `.cmd` shim, and the mod runs commands without a shell, so a direct call fails. This path is not yet tested on a real Windows machine.

## Development

```bash
claude plugin validate integrations/claude-code/speclink-skills
claude plugin test integrations/claude-code/speclink-skills
```

For local work, add this repository as a local marketplace and install from it. Claude Code then reads the working tree, and `/reload-plugins` applies an edit:

```bash
claude plugin marketplace add <path to the speclink repo>
claude plugin install speclink-skills@speclink
```

After Claude Code loads the mod, it writes type files into `.claude-plugin/types/` (ignored by git). Then `npx tsc -p integrations/claude-code/speclink-skills` from the repository root type-checks the mod.

The skill tabs are in `hooks/groups.ts`. `scripts/claude-code/skill-groups.test.mjs` compares them with the repository's `.claude/skills/speclink-*`: when speclink adds or removes a skill, this test tells you to update the tabs.

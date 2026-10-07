# speclink-skills: a Claude Code mod for Speclink

[繁體中文](README.md)

It adds three things to Claude Code:

- **Skill bar**: a row of speclink skill buttons above the prompt, in five tabs that follow the workflow: Plan, Build, Quality, Ship and More. A click puts the `/speclink-…` command into the prompt box, and keeps what you already typed behind it as the arguments. `archive+commit` on the Ship tab puts in `/speclink-archive + /speclink-commit` in one click. When the mouse is on a skill, the right end of the skill row shows its description. The skill bar has three rows: "speclink" (with the step and change that the session works on at its right), then the tabs and the Panel button, then the skills. If the terminal is too narrow for the tabs and the Panel button in one row (less than 55 columns with Chinese labels, 60 with English), the Panel button moves to the right end of the first row. The tabs stay together in one row. This layout has no room for the descriptions, so they do not show.
- **Side panel**: press "Panel" on the skill bar, or type `/speclink-panel`, to open or close it. It has three tabs:
  - **Board**: the next change, and the proposed, in-progress and ready changes (wave, worktree, task progress, and what blocks them). The ▸ before a change expands it: whether its proposal, design, specs and tasks are done, and its tasks grouped by `##` heading (done tasks dim, `[M]` tasks marked "Manual").
  - **Discussions**: open and concluded discussions in two sections. The ▸ expands a discussion to show its text.
  - **Quality**: the open review and verify tickets of each change, with the round and the number of findings at each severity. The ▸ expands the findings.
  - The ▾ before each section title collapses that section. A click on a name puts it into the prompt box as the argument of the command at the front. The panel only shows data: it does not check tasks or change anything.
- **What this session works on**: right of "speclink" on the skill bar, the bar shows the step and the change that this session works on, for example `▸ apply · add-auth`, with the step in its tab's color. The session title changes to `apply · add-auth` too, so a tab bar that shows the terminal title (Warp, for example) tells your sessions apart by change. The step comes from the `/speclink-…` command you send and from speclink skills that the model calls itself (for example the review that quality runs next). The change comes from the command's arguments and from the change name that a skill passes to the speclink CLI, so it is found even when you type no name. A session started in a change's worktree shows that change from the start. After the archive is done (the change leaves the list), the bar adds `✓`, for example `archive · add-auth ✓`. A compact changes nothing. `/clear` clears it, and the title changes back to `Claude Code` at your next message. A conversation brought back with `/resume` gets the step and change back from its title or its transcript, also after a compact. A session that has not used speclink shows nothing and keeps its title.

## Requirements

- Claude Code 2.1.290 or later (tested on this version). The mod API is early access, so a Claude Code release can make an update of this mod necessary.
- The `speclink` CLI on PATH. The panel reads the `--json` output of `speclink list`, `speclink plan` and `speclink discuss list`. It reads `show` and `status` when you expand a change, `discuss show` when you expand a discussion, and `review show` and `verify show` for each change when you open the Quality tab. For a change in a worktree, it reads in that worktree folder.
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

- **The panel shows no review or verify stamps** (the Quality tab lists only open tickets): the CLI output has no stamp state. `list --json` leaves those fields out on purpose, and a stamp removes the ticket, so `review show` fails after a stamp too. The stamp is recorded only in the change's `.openspec.yaml`; use the desktop app to see stamps.
- **`archive+commit`**: Claude Code treats only the first slash command as a command. It runs `/speclink-archive`, and `+ /speclink-commit …` becomes its arguments, so the model commits after the archive.
- **The session title changes at your next message**: Claude Code lets a mod change the title only when you send a message and when a session starts. So when a skill finds (or changes) the change halfway, when an archive adds `✓`, and after `/clear`, the skill bar changes at once, but the title waits for your next message. After the bar has a step or a change, a name you gave with `/rename` is replaced at your next message. `/clear` changes back only a title that this mod set, not one you gave.
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

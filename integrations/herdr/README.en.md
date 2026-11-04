# speclink: a Herdr mod for Speclink

[繁體中文](README.md)

Open a panel in [Herdr](https://herdr.dev) that shows the changes and discussions of the current project. One click hands a skill to the correct agent, starts work on a change, or tells Speclink to open a detail page. You do not type a change name and you do not look for a window.

## What the panel shows

Press `⌘⇧S` to open or close the panel. The panel is a floating window that you can drag. It does not take the keyboard. The project follows the terminal that has focus, so a switch of space also switches the project.

- **Changes**: three permanent cards for proposed, in progress and ready. Each row shows the wave, the name, a worktree mark and the task progress. A blocked row is dimmed and names what it waits for.
- **Discussions**: a discussions card and a promoted card. A promoted discussion that still has unpromoted cuts stays on the discussions card, because its next step is propose.
- **The agent on this change**: the row shows the location and status of the agent. Click it to go there. The panel matches the `speclink.change` label, then the worktree path, then the terminal title that the Claude Code mod sets.
- **Skill buttons**: only the skills that apply to the stage. The suggested next step has a heavy border.
- **Start**: a button on a change row that starts a new agent for this change only. See "Start" below.
- **+ New work**: the skills that take no change name. For discuss, you can type the topic directly.
- **Open in Speclink**: click a change row or a discussion row to open its detail page. Click the wave number at the start of a row, or the Roadmap button at the top, to open the roadmap.

Each card can collapse. More than 5 rows fold into "N more…". Each row has a copy button.

### Skills per stage

| State | Skill buttons (bold is the suggested next step) |
| --- | --- |
| Proposed, not blocked | **apply**, analyze, ingest |
| Proposed, blocked | apply is disabled, analyze, ingest |
| In progress | **apply**, ingest, drift, verify |
| Ready, no worktree | **quality**, review, verify, archive, commit |
| Ready, with worktree | **quality**, review, verify, commit, worktree-merge |
| Discussion concluded again, change not updated | the suggestion changes to **ingest** |
| Open discussion | **discuss** |
| Concluded, or cuts left to promote | **propose --from-discussion**, discuss |

### Where a skill goes

The panel tries these in order and stops at the first match:

1. The agent that works on this change.
2. The agent in the current tab.
3. A new tab with the most used installed agent.

Each step checks the folder of the agent. Skills for a change with a worktree run in the worktree. archive, worktree-merge and apply-with-worktree run in the main checkout. An agent in another folder does not match. When a change has a worktree with no agent in it, the apply button sends apply-with-worktree, which continues in the existing worktree.

The panel does not send to an agent that works, waits for your answer or shows an error. The button is disabled and the reason shows beside it. After a send, the window moves to that agent.

Claude Code and GitHub Copilot get `/speclink-<skill>`. Codex gets `$speclink-<skill>`. Only Claude Code has analyze, so the button does not show for other agents. Other agents have no speclink skills, so the panel does not send to them.

### Start

Start is different from a skill button. A skill button first tries an existing agent. Start always makes a new agent.

| `worktree` in the workflow config | What Start does |
| --- | --- |
| On | Opens a new space and runs `/speclink-apply-with-worktree <change>`. The space keeps the name that Herdr gives it. The tab name shows the change |
| Off | Opens a new tab in the space of the main folder and runs `/speclink-apply <change>` |

A change has the Start button only if it is not blocked, has unfinished tasks and has no agent on it. A change whose discussion was concluded again needs ingest first, so it has no Start button.

**Start the whole wave**: when `worktree` is on and the earliest wave has two or more changes that can start, a bar shows at the top of the panel and names those changes. One click starts all of them, with no second question. The waves come from `speclink plan`.

### Open in Speclink

The panel uses `open` to send a `speclink://` URL. The desktop app goes to this project and then opens the page. If the app is not running, it starts. The URL can only open and show things. It writes no files.

## Requirements

- Herdr 0.1.12 or later (tested with this version).
- The `speclink` CLI on PATH, with `unpromotedCuts` in `plan --json`. The panel reads only the `--json` output of `speclink list`, `speclink plan`, `speclink discuss list` and `speclink workflow-config show`.
- A project with `speclink init` done.
- To open pages in Speclink, a desktop app that knows `speclink://` URLs (a version after v0.11.0).

## Install

Run this from the repo root:

```bash
cp -R integrations/herdr/mods/speclink ~/.config/herdr-app/mods/
```

Herdr loads it automatically. Then add this line to `global` in `~/.config/herdr-app/keys.json` ([keys.json](keys.json) has the same line):

```json
"cmd-shift-s": "mod.open:speclink"
```

To update, run `cp` again. Herdr reloads at once. To remove, delete the folder and the line.

Do not install with a symbolic link. Herdr loads the code, but it does not draw the SVG icons behind a link, and a save does not reload the mod.

## Known limits

- **No docking in the window**: Herdr's own layouts do not let a mod dock. Docking needs a cloned layout, and a clone does not follow Herdr updates. So the panel opens with a key.
- **No review or verify stamps**: the CLI output has no stamp state. After you stamp a ready change, the suggestion stays on quality, and you click archive yourself.
- **"Promoted" is approximate**: it uses `status` from `discuss list` and the unpromoted cuts from `plan`.
- **Codex is checked on paper only**: the command form comes from the speclink docs and skill output. No send to a real Codex was tested.
- **Skill buttons do not check installed skills**: for example, in a project without Codex skill files, the button still sends `$speclink-…`.
- **Opening pages works only on macOS**: the panel uses `open`. On other systems a notice says that the page cannot open. A project in a space on another machine cannot open, so its rows do nothing on click and there is no Roadmap button.
- **The `SPECLINK_WORKTREE` environment variable is not read**: the panel reads only the workflow config. If the variable turns worktree off, Start still opens a new space, and the skill stops and explains why.
- **A started space can stay in the main folder**: the agent uses `cd` to go into the worktree, and the folder that the terminal reports does not always follow. In this case the space has the name of the project and does not nest under the project. Only the tab name shows the change.
- **A folder that is new to Claude Code asks for trust first**: during that question the terminal is not yet an agent, so the panel does not see it, and the Start button comes back after 15 seconds. A trusted folder does not ask.
- **Starting a whole wave can hit a git lock**: each agent can commit specs or make a worktree in the main folder. An agent that hits a lock stops and reports it. Click its skill button again.
- **The panel text is Traditional Chinese only**.

## Development

```bash
node --test scripts/herdr/panel-model.test.mjs
herdr-app check ~/.config/herdr-app/mods/speclink
```

`model.js` holds the pure logic: data merge, skill table and delivery decisions. The tests run it directly. `main.js` holds the view and the Herdr API calls. `scripts/herdr/panel-model.test.mjs` also compares the skill table with `.claude/skills/speclink-*`, so it fails when speclink adds or removes a skill.

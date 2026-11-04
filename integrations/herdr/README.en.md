# speclink: a Herdr mod for Speclink

[繁體中文](README.md)

Open a panel in [Herdr](https://herdr.dev) that shows the changes and discussions of the current project. One click hands a skill to the correct agent, starts work on a change, or tells Speclink to open a detail page. You do not type a change name and you do not look for a window.

![A Herdr window: the stage badges and the Agents list on the left, and the Speclink panel on the right](../../docs/assets/screenshots/en/herdr-panel.png)

## What the panel shows

The panel has two placements. Neither takes the keyboard. In both, the project follows the terminal that has focus, so a switch of space also switches the project.

- **Docked**: with the cloned `speclink` layout (see "Install"), the panel stays in the rightmost column of the window, beside the terminals. `⌘⇧S` opens or closes this column. Drag its edge to change the width. Each space remembers its own open or closed state, and a switch of space restores it. A space with no saved state starts open, so close it once in a project that does not use Speclink.
- **Popped up**: in other layouts, `⌘⇧S` opens or closes the panel as a floating window that you can drag.

What the panel contains:

- **Top**: the project name. The line below it says "目前的 space · <branch>" (the current space and its branch).
- **Changes**: three permanent cards for proposed, in progress and ready. Each card title has a dot in the stage color and a count. Each row shows the wave, the name, a worktree mark and the task progress. A blocked row is dimmed and names what it waits for.
- **Discussions**: a discussions card and a promoted card. A promoted discussion that still has unpromoted cuts stays on the discussions card, because its next step is propose.
- **The agent on this change**: the row shows the location and status of the agent. Click it to go there. The status looks the same as in the Agents list on the left: a turning arc for working, an empty ring for idle, a solid dot for done, and a pill when it waits for your answer. The panel matches the `speclink.change` label from Start, then the worktree path, then the `speclink.focus` label that the Claude Code plugin reports (the same as the skill bar, and live), then the terminal title. The title changes only when you send a message, so the panel ignores it when the plugin reports. A title with the step discuss or propose also does not count, because the name there is often only a mention.
- **Skill buttons**: each row keeps only the suggested next step. `⋯` opens the Skills pop-up with this skill and this row already selected. A tooltip on the button tells where it goes.
- **Attach button**: the chat icon beside the name. It adds the name of the change or discussion to the input box in the current tab and does not send it. The button is gray when the current tab has no agent or the agent waits for your answer.
- **Stage badge**: only in the `speclink` layout. A project row in the Spaces list shows the number of changes in each stage, in the same order and colors as the panel cards: a dot and a number for proposed and in progress, and a green `✓2` for ready. A stage with 0 changes does not show. Move the pointer over the badge to see the stage names. It updates about every 30 seconds.
- **Agents list**: only in the `speclink` layout. The list at the bottom left groups agents by space, in the same order as the Spaces list. Each space has its own Done, Working and Idle sections, and the title of a folded space shows the count for each status. An agent that waits for your answer goes in Needs you at the top, for any space. An agent with its own tab no longer shows a second time under the agent that started it. A row for an agent that works on a change ends with the stage color and the task progress. A tab with no name shows the change name. The match rules are the same as in the panel. The clock icon menu can still switch to Priority.
- **Language**: the interface follows the system language: Chinese on a Chinese system, and English on other systems. This applies to the panel on the right, the Skills pop-up and the sidebar on the left. Restart Herdr after you change the system language.
- **Start**: a button on a change row that starts a new agent for this change only, in the current space. See "Start" below.
- **Skills (技能)**: the button at the top or `⌘⇧N` opens a pop-up where you select a skill and a change or discussion for it. See "Skills pop-up" below.
- **Open in Speclink**: click a change row or a discussion row to open its detail page. Click the wave number at the start of a row, or the Roadmap button at the top, to open the roadmap. Click the logo at the top left to bring up the Speclink app.

Each card can collapse. More than 5 rows fold into "N more…". Each row has a copy button.

### When the space is not a Speclink project

If the current space is not a Speclink project, the panel shows a hint card with the project folder and an initialize button (初始化). The Roadmap and Skills buttons are not there, because they do not work here yet.

The initialize button opens a new tab and runs `speclink init` in the project folder. The command asks, one question at a time, which agents to generate files for. It writes files only after your answers. To stop, close that tab. After the command completes, the panel shows the change list within 10 seconds.

Only a project that is not initialized shows this card. Other problems, such as a missing `speclink` or a broken config file, still show red text.

### Skills per stage

| State | Available skills (bold is the suggested next step, the one the row shows) |
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

The panel does not send to an agent that works, waits for your answer or shows an error. The button is disabled, and the tooltip on the button gives the reason. After a send, the window moves to that agent.

Claude Code and GitHub Copilot get `/speclink-<skill>`. Codex gets `$speclink-<skill>`. Only Claude Code has analyze, so the button does not show for other agents. Other agents have no speclink skills, so the panel does not send to them.

### Start

Start is different from a skill button. A skill button first tries an existing agent. Start always makes a new agent. The new agent opens in a new tab in the current space, the tab name shows the change, and the view stays on the current tab.

| Situation | What Start does |
| --- | --- |
| `worktree` in the workflow config is on, and the change has no worktree yet | Asks first: "用 worktree" (use a worktree, `/speclink-apply-with-worktree <change>`) or "直接 apply" (apply directly, `/speclink-apply <change>`) |
| The change already has a worktree | Does not ask, and runs `/speclink-apply-with-worktree <change>` to continue |
| `worktree` is off | Does not ask, and runs `/speclink-apply <change>` |

The main folder runs one apply at a time. If an apply already runs there, the apply-directly button does not work, and its tooltip shows the reason.

A change has the Start button only if it is not blocked, has unfinished tasks and has no agent on it. A change whose discussion was concluded again needs ingest first, so it has no Start button.

**Start the whole wave**: when `worktree` is on and the earliest wave has two or more changes that can start, a bar shows at the top of the panel and names those changes. One click starts all of them with no more questions, always with a worktree. The waves come from `speclink plan`.

### Skills pop-up

![The Skills pop-up: skills in five groups on the left, and the changes that can pair with the skill on the right](../../docs/assets/screenshots/en/herdr-skill-picker.png)

Press the "技能" (Skills) button at the top or `⌘⇧N` to open the pop-up. Press it again or press `Esc` to close. This works in all layouts, also when the panel column is closed.

- **Left**: all skills in five groups: 規劃 (plan), 實作 (build), 品質 (quality), 收尾 (ship) and 專案 (project). Select one with `↑` `↓` or the mouse. The number at the right is how many changes or discussions the skill can use now. A skill that uses a change shows dim when the number is zero. The project group works on the whole project and has no number.
- **Right**: the changes or discussions that this skill can use. The rules are the same as for skill buttons. Select one with `←` `→` or the mouse. You can also select none (不指定). Skills that do not use a change (discuss, propose and the project group) have an input box for a topic or a note.
- **Bottom**: the command that goes in, or the reason why it cannot be used now.

The bottom has two buttons, one for each result:

- **Put it in the input box (Enter).** The command is pasted into the input box of the agent in the current tab and is not sent. You can type more and press Enter yourself. This also works when the agent is working. It does not work when the agent waits for your answer, because the paste can answer its dialog.
- **Open a new tab and start (`⌘Enter`).** This works like the `herdr-dispatch` skill: `hc run` opens a new tab in the current space, the tab name is "skill · change", and the view stays on the current tab.

When the skill uses a change, each row has a checkbox, so you can open many tabs at once, one tab for each change. Click the box or press `Space` to check a row. Use `⌘A` or "全選" (all) to check all rows. A click on the row only selects it and does not check it.

- No check: both buttons use the selected row.
- One check: both buttons use the checked row.
- Two or more checks: the left button changes to "開 N 個分頁並開工" (open N tabs and start). "放到輸入框" (put in the input box) does not work, because the input box takes one command at a time.

These rows have no checkbox and show the reason: the change already has an agent, another change blocks it (apply), the main folder already runs an apply, or the project has no worktree flow (apply-with-worktree). If you check two applies that both need the main folder, the bottom line asks you to use apply-with-worktree.

### Open in Speclink

The panel uses `open` to send a `speclink://` URL. The desktop app goes to this project and then opens the page. If the app is not running, it starts. The URL can only open and show things. It writes no files.

The logo sends no URL. It only brings up the app, so the app stays on its current project. To go to this project, use the Roadmap button or click a row.

## Requirements

- Herdr 0.1.12 or later (tested with this version).
- The `speclink` CLI on PATH, with `unpromotedCuts` in `plan --json`. The panel reads only the `--json` output of `speclink list`, `speclink plan`, `speclink discuss list` and `speclink workflow-config show`.
- A project with `speclink init` done. If it is not done, the panel gives an initialize button.
- To open pages in Speclink, a desktop app that knows `speclink://` URLs (a version after v0.11.0).
- To dock the panel, Node.js, which runs the script that clones the layout.

## Install

Work from the repo root, with Herdr open:

1. Copy the mod. Herdr loads it automatically.

   ```bash
   cp -R integrations/herdr/mods/speclink integrations/herdr/mods/speclink-new ~/.config/herdr-app/mods/
   ```

2. In `~/.config/herdr-app/keys.json`, add the first two lines to `global`. Then add the `layout:speclink` section ([keys.json](keys.json) has the same content). `⌘⇧N` opens the Skills pop-up. In the `speclink` layout, `⌘⇧S` opens or closes the right column. In other layouts, `⌘⇧S` pops up the floating window.

   ```json
   "cmd-shift-s": "mod.open:speclink",
   "cmd-shift-n": "mod.open:speclink-new"
   ```

   ```json
   "layout:speclink": { "cmd-shift-s": "layout.toggle:speclink" }
   ```

3. To dock the panel, run this line. It clones Herdr's `main` layout as the `speclink` layout, adds the panel as the rightmost column, replaces the left sidebar with clones connected to Speclink, and switches to it. To keep the pop-up only, skip this step.

   ```bash
   node scripts/herdr/dock-panel.mjs
   ```

Use `⌘⇧L` to switch layouts. To update the mod, run `cp` again. Herdr reloads at once.

Do not install with a symbolic link. Herdr loads the code, but it does not draw the SVG icons behind a link, and a save does not reload the mod.

### Dock again after a Herdr update

A clone does not follow Herdr updates: new things in Herdr's `main` layout, Spaces list or Agents list do not appear in the `speclink` layout. `cloned_from.version` in `~/.config/herdr-app/layouts/speclink.json` is the Herdr version at the time of the clone. When it differs from `herdr-app --version`, dock again. Run the same line:

```bash
node scripts/herdr/dock-panel.mjs
```

The script does these steps:

1. It switches to `main`.
2. It renames the old clone to `layouts/speclink.json.old-<time>` and deletes the old `mods/speclink-spaces`, `mods/speclink-agents` and `mods/speclink-chrome`.
3. It clones the new `spaces`, `agents` and `workspace-chrome` and connects them. If Herdr changed the lines the script edits, the script stops before it changes anything.
4. It clones the new `main`, adds the panel, and switches to the new `speclink` layout.

After this, the panel column has its default width, and the other column widths and open or closed states are the same as in `main`. The state that each space remembers does not change. If you edited the old clone by hand, your edits are in the `.old` file, and you must move them yourself. You can delete the `.old` file when all is well.

### Remove

1. Press `⌘⇧L` to switch to `main`.
2. Delete `~/.config/herdr-app/layouts/speclink.json`, and `speclink`, `speclink-new`, `speclink-spaces`, `speclink-agents` and `speclink-chrome` in `~/.config/herdr-app/mods/`.
3. Remove the additions from `keys.json`.

### Why a cloned layout

The conclusion of the `herdr-speclink-panel` discussion rejected a dedicated cloned `speclink` layout, because a clone does not follow Herdr updates. On 2026-11-04, after a look at the real screen, this decision was reversed and the cost of a clone was accepted: the floating window covers the terminal, and that is too far from the agreed right column.

The same conclusion rejected the Spaces sidebar badge, because it needs a clone of the spaces mod. The layout is a clone now and several projects run at once, so this was reversed on the same day. The built-in spaces mod has no extension point, so the script clones it, adds `integrations/herdr/spaces-badge/stage-badge.js`, and changes a few lines to connect it. One change moves the git arrows next to the badge, so they do not cover the name when the row is full.

The conclusion first said that Start opens a new space, on the assumption that it would indent under the project. A real test showed that it does not: the skill must check and commit documents in the main folder before it makes the worktree, and the terminal stays in the main folder. The result was only a second space with the same name. So on the same day Start changed to open in the current space, and to ask first about a worktree or a direct apply.

The Agents list works the same way. The built-in agents mod has no extension point, so the script clones it, adds `integrations/herdr/agents-list/speclink-rows.js` and the panel's `model.js` and `shared.js`, and changes about fifteen lines. The list and the panel then use the same rules to find the change that an agent works on.

The Settings and Add machine rows at the bottom of the sidebar are in `workspace-chrome`, so the script clones it too, only to translate them. The translations for all the clones are in the panel's `i18n.js`.

Herdr's own `main` layout cannot dock a mod: `"place": "right"` in `mod.json` has no effect, and `layout.set` on `main` is refused.

## Known limits

- **A clone does not follow Herdr updates**: the `speclink` layout stays as `main` was at the time of the clone, and the Spaces and Agents lists stay as `spaces` and `agents` were. Dock again after a Herdr update. See "Dock again after a Herdr update".
- **No review or verify stamps**: the CLI output has no stamp state. After you stamp a ready change, the suggestion stays on quality, and you click archive yourself.
- **"Promoted" is approximate**: it uses `status` from `discuss list` and the unpromoted cuts from `plan`.
- **Codex is checked on paper only**: the command form comes from the speclink docs and skill output. No send to a real Codex was tested.
- **Opening many tabs can hit a git lock**: the same as starting a whole wave. An agent that hits a lock stops and reports it.
- **The button and the folder have different names**: the button is "技能" (Skills) and the folder is still `speclink-new`, so `keys.json` does not change.
- **Pasting into the input box works only on macOS and takes about 0.7 seconds**: Herdr has no command that pastes without Enter, so the panel uses `script` to run a hidden `hc attach`, sends the paste, and leaves. It is one paste, not typing. It does not touch the clipboard or change the terminal size. The attach button shows a turning arc while it pastes and a check mark when it finishes. `script` works differently on other systems, so the paste fails with a notice. In this case, use the button that opens a new tab.
- **Skill buttons do not check installed skills**: for example, in a project without Codex skill files, the button still sends `$speclink-…`.
- **Opening pages works only on macOS**: the panel uses `open`. On other systems a notice says that the page cannot open. A project in a space on another machine cannot open, so its rows do nothing on click and there is no Roadmap button. The logo still works, because it brings up the app on this computer.
- **The `SPECLINK_WORKTREE` environment variable is not read**: the panel reads only the workflow config. If the variable turns worktree off, Start still asks. If you choose a worktree, the skill stops and explains why.
- **A worktree does not show on the left by itself**: the agent's terminal stays in the main folder and only each command goes into the worktree, so Herdr shows no row for the worktree. To open a shell or run tests in the worktree, use Herdr's built-in Open worktree.
- **A folder that is new to Claude Code asks for trust first**: during that question the terminal is not yet an agent, so the panel does not see it, and the Start button comes back after 15 seconds. A trusted folder does not ask.
- **Starting a whole wave can hit a git lock**: each agent can commit specs or make a worktree in the main folder. An agent that hits a lock stops and reports it. Click its skill button again.
- **Herdr's own text is not translated**: Files and Changes in the tab bar, the status bar at the bottom, right-click menus and the settings page stay as they are.
- **Limits of the Agents list**: the sidebar is narrow, so a row with progress shows no time. A discuss or propose tab shows no progress, because it does not count as work on a change.
- **The badge needs a machine it can reach**: a machine that cannot be reached shows no badge. The narrow sidebar also shows no badge. If a row does not have enough space, the space name gets shorter, so the arrows and the badge stay visible.
- **The panel text is Traditional Chinese only**.

## Development

```bash
node --test scripts/herdr/panel-model.test.mjs
herdr-app check ~/.config/herdr-app/mods/speclink
herdr-app check ~/.config/herdr-app/mods/speclink-new
```

`model.js` holds the pure logic: data merge, skill table and delivery decisions. The tests run it directly. `main.js` holds the panel view and the Herdr API calls. `skill-picker.js` holds the Skills pop-up view. The rules for the input box and the new tab are in `model.js`. `column-sync.js` makes each space remember the state of the right column. It runs in the invisible floating block that `dock-panel.mjs` adds, because the panel does not redraw while its column is closed. `speclink-new` holds only settings: a mod can set only one overlay, and Herdr does not let a mod import code from another folder, so this folder opens the pop-up view of the `speclink` mod. `scripts/herdr/panel-model.test.mjs` also compares the skill table with `.claude/skills/speclink-*`, so it fails when speclink adds or removes a skill.

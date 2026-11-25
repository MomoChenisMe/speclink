---
name: herdr-speclink
description: "Use inside a Herdr terminal when the user wants Speclink work to run in other agent tabs: \"start the next wave\", \"open an agent for <change>\", \"dispatch\", \"run /speclink-quality on <change> in a new tab\". Starts one Claude Code tab per change with hc, with the labels that the Speclink panel reads. Only works when $HC_TERMINAL_ID is set."
---

# herdr-speclink: start Speclink work in other Herdr tabs

**Conversation language**: write to the user in the language they use with you. Keep commands, paths and labels as they are.

This skill opens tabs and sends the first command. The Claude in each new tab does the work with the Speclink skill you gave it. The user answers its questions in that tab. Do not answer for it and do not watch its progress.

`hc` (herdr-core) runs the terminals; `herdr-app` changes what the window shows. For anything this skill does not cover, read `hc --skill` and `herdr-app --skill` first. `hc` is not `herdr`, the older terminal multiplexer.

**Asking the user**: ask with the AskUserQuestion tool; when it is not available, ask the same question as plain text and wait.

## 1. Check that this is a Herdr terminal

```sh
echo "HC_TERMINAL_ID=$HC_TERMINAL_ID"; command -v hc herdr-app speclink claude jq
ROOT=$(dirname "$(git rev-parse --path-format=absolute --git-common-dir)")
```

- `$HC_TERMINAL_ID` is empty → stop. Tell the user that this session is not in a Herdr terminal, so hc cannot put a new agent in the window.
- A tool is missing → stop and name it.
- `ROOT` is the main checkout, also when you are in a worktree. Run every `speclink` command below as `(cd "$ROOT" && speclink …)`.

A skill that takes no change (`discuss <topic>`, `propose --from-discussion <slug>`, `improve`) skips steps 2, 4 and 5. Start one tab at a time with the arguments the user gave.

## 2. Choose the changes

```sh
(cd "$ROOT" && speclink plan --json) | jq -r '.changes[] | "\(.name)\t\(.stage)\twave \(.wave)\tblockedBy=\(.blockedBy|join(","))\toverlaps=\(.overlaps|map(.change)|join(","))"'
```

- The user named no change → list the changes whose `blockedBy` is empty and ask which ones to start (multi-select). Mark the ones in the lowest wave.
- The user named changes → each one must be in `changes`. One tab per change.
  - `blockedBy` is not empty → do not start it. Say which changes it waits for.
  - Not found → do not start it. Say so, and continue with the others.
- Two changes in this batch are in each other's `overlaps` → warn in one line that they change the same spec and can conflict at merge or archive. Do not block.

## 3. Choose the skill

Decide for each change. When the user named the skill or the place ("main folder" → `speclink-apply`, "worktree" → `speclink-apply-with-worktree`), do that.

Otherwise use `stage`:

- `ready` → `speclink-quality`.
- `proposed` or `in-progress` → an apply. Take the first rule that fits:
  1. The change has a worktree (step 4 gives a path) → `speclink-apply-with-worktree`.
  2. The worktree policy is off → `speclink-apply`.
  3. The policy is on and this batch has two or more applies → `speclink-apply-with-worktree` for all of them.
  4. The policy is on and this batch has one apply → ask: main folder (`speclink-apply`) or worktree (`speclink-apply-with-worktree`).

The worktree policy: `(cd "$ROOT" && speclink workflow-config show --json) | jq .worktree`.

**The main folder runs one apply at a time.** Two applies there change the same files. Before you start `speclink-apply`, check `hc ls --label speclink.skill=speclink-apply --json`. A live terminal there, or a second `speclink-apply` in this batch → stop and ask the user to move one to a worktree. With the policy off, start only one apply.

## 4. Choose the working directory

```sh
(cd "$ROOT" && speclink list --json) | jq -r --arg c "<change>" '.changes[] | select(.name==$c) | .worktree.path // empty'
```

- `speclink-apply-with-worktree`, `speclink-archive`, `speclink-worktree-merge` and skills without a change → `ROOT`.
- Other skills → the worktree path when the change has one, else `ROOT`.

## 5. Do not start a second agent on the same change

```sh
hc ls --label speclink.change=<change> --json | jq -r '.terminals[] | "\(.address)\t\(.name // "")"'
```

A live terminal with that label → an agent already works on the change. Do not start another. Tell the user its address, and offer `herdr-app do agent.focus --terminal <address>`.

## 6. Start the tab

Run this once for each change. `<skill>` is the full name with the `speclink-` prefix. `NAME` is the change name, cut to 32 characters (hc names are 1–32 of `a-z 0-9 - _`).

```sh
SPACE=$(hc get "$HC_TERMINAL_ID" --json | jq -r '.labels["herdr.space"]')
hc run --bg --json --no-notify --outlive-parent \
  --name "$NAME" --cwd "$DIR" \
  --label herdr.space="$SPACE" \
  --label speclink.change="<change>" --label speclink.skill="<skill>" \
  claude "/<skill> <change>"
```

- `herdr.space` puts the terminal in a new tab of this space. Without it the terminal stays hidden.
- `speclink.change` and `speclink.skill` are what the Speclink panel reads. Without them the panel does not show this agent on the change's row.
- For a skill on a discussion, replace the `speclink.change` label with `speclink.discussion=<slug>`. For a skill with neither, leave both out and use the short skill name as `NAME`.
- `--outlive-parent`: the tab keeps running when this session ends. `--no-notify`: hc does not type notes about it into this session.
- Put the command in the `claude` arguments. Do not start the agent first and `hc send` afterward: an agent that is still starting can drop the message.
- Do not use `claude -p`. The tab must stay open for the user.

Take `address` from the JSON, wait for the agent, then name the tab the way the panel does:

```sh
hc wait <address> --timeout 20 --json
TAB=$(herdr-app ls | jq -r --arg t <address> '.spaces[].tabs[] | select(any(.terminals[]; .terminal==$t)) | .id')
herdr-app do tab.rename --id "$TAB" --name "<short skill> · <change>"
```

`<short skill>` is the skill without `speclink-`, for example `apply · add-auth`. `hc wait` that returns `idle` with `turn: 0` is normal: Claude reports that it is ready before it reads the first command.

## 7. Check and report

```sh
hc get <address> --json | jq '{state: .status.state, kind: .status.kind, turn}'
```

- `working`, or `turn` is 1 or more → it started.
- `blocked` → the agent waits for a person (`kind` is `permission` or `question`). Tell the user and offer `herdr-app do agent.focus --terminal <address>`. Do not answer with `hc keys` or `hc send --force`.
- Still `idle` with `turn: 0` → read the screen once with `hc read <address> --lines 0`. A folder-trust dialog needs the user. Do not send the command again without a look at the screen.

Report a table: change, skill, terminal address, tab name, working directory. Then say that questions are answered in each tab.

## Later, only when the user asks

- Status: `hc ls --json`, then `hc get <address> --json` for the tabs you started.
- Clean up: `hc kill <address>; hc rm <address>`. The tab closes with it. Only touch terminals that this skill started.

## Do not

- Do not open a new space or use another space.
- Do not run `hc server stop` or `hc server restart`. They affect every terminal of the user.
- Do not answer questions or permission dialogs for another agent.
- Do not put two changes in one tab or one command.

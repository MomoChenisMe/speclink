---
name: herdr-speclink
description: "Use inside a Herdr terminal when the user wants Speclink work to run in other agent tabs: \"start the next wave\", \"open an agent for <change>\", \"dispatch\", \"run /speclink-quality on <change> in a new tab\". Starts one Claude Code tab per change with hc, in the space of the project the work belongs to, with the labels that the Speclink panel reads. Only works when $HC_TERMINAL_ID is set."
---

# herdr-speclink: start Speclink work in other Herdr tabs

**Conversation language**: write to the user in the language they use with you. Keep commands, paths and labels as they are.

This skill opens tabs and sends the first command. The Claude in each new tab does the work with the Speclink skill you gave it. The user answers its questions in that tab. Do not answer for it. Watch its progress only when the user asks (see "Wait for results").

`hc` (herdr-core) runs the terminals; `herdr-app` changes what the window shows. For anything this skill does not cover, read `hc --skill` and `herdr-app --skill` first. `hc` is not `herdr`, the older terminal multiplexer. The commands here were run with hc 0.1.22 and Claude Code 2.1.332.

**Asking the user**: ask with the AskUserQuestion tool; when it is not available, ask the same question as plain text and wait.

## Input

- `/herdr-speclink <change> [<change>…]`: the skill comes from each change's stage (step 3).
- `/herdr-speclink <skill> <change> [<change>…]`: the user names the skill, for example `quality`, `archive`, `drift`. The `speclink-` prefix is optional.
- `/herdr-speclink` or "start the next wave": choose from `speclink plan`.
- A skill that takes no change (`discuss <topic>`, `propose --from-discussion <slug>`, `improve`): send the arguments the user gave, one tab at a time. Skip steps 2, 4 and 5.

## 1. Check that this is a Herdr terminal

```sh
echo "HC_TERMINAL_ID=$HC_TERMINAL_ID"; command -v hc herdr-app speclink claude jq
SELF_ROOT=$(dirname "$(git rev-parse --path-format=absolute --git-common-dir)")
ROOT=$SELF_ROOT
```

- `$HC_TERMINAL_ID` is empty → stop. Tell the user that this session is not in a Herdr terminal, so hc cannot put a new agent in the window.
- A tool is missing → stop and name it.
- `SELF_ROOT` is the main checkout of this session's project, also when you are in a worktree. Step 6 uses it.
- `ROOT` is the main checkout of the project the work belongs to. When the work belongs to another project (the user says so, or the topic clearly is another repository), change it: `ROOT=$(dirname "$(git -C <project folder> rev-parse --path-format=absolute --git-common-dir)")`. Run every `speclink` command below as `(cd "$ROOT" && speclink …)`.

## 2. Choose the changes

```sh
(cd "$ROOT" && speclink plan --json) | jq -r '.changes[] | "\(.name)\t\(.stage)\twave \(.wave)\tblockedBy=\(.blockedBy|join(","))\toverlaps=\(.overlaps|map(.change)|join(","))\treqOverlap=\(.requirementOverlap|map(.change + (if .conflict then "(conflict)" else "" end))|join(","))"'
```

- The user named no change → list the changes whose `blockedBy` is empty and ask which ones to start (multi-select). Mark the ones in the lowest wave.
- The user named changes → each one must be in `changes`. One tab per change.
  - `blockedBy` is not empty → do not start it. Say which changes it waits for. Two changes in this batch that depend on each other: start only the first.
  - Not found (archived, misspelled, or in `skipped`) → do not start it. Say so, and continue with the others.
- One change in this batch is in another's `overlaps` or `requirementOverlap` → warn in one line before you open the tabs: they change the same spec and can conflict at merge or archive. `conflict: true` in `requirementOverlap` means that the change archived second is refused, so one side must change. Warn only, do not block.

## 3. Choose the skill

Decide for each change. When the user named the skill or the place ("main folder", "no worktree" → `speclink-apply`; "worktree" → `speclink-apply-with-worktree`), do that and do not ask. The user can name it for the batch or for each change.

Otherwise use `stage`:

- `ready` → `speclink-quality`.
- `proposed` or `in-progress` → an apply. Take the first rule that fits:
  1. The change has a worktree (step 4 gives a path) → `speclink-apply-with-worktree`. It continues in that worktree.
  2. The worktree policy is off → `speclink-apply` in the main folder.
  3. The policy is on and this batch has two or more applies with no place named → `speclink-apply-with-worktree` for all of them.
  4. The policy is on and this batch has one apply with no place named → ask once: main folder (`/speclink-apply`) or worktree (`/speclink-apply-with-worktree`). First run `hc ls --label speclink.skill=speclink-apply --json`. No live terminal there, and no other change in this batch goes to the main folder → recommend the main folder. Otherwise recommend the worktree and name the terminal that uses the main folder. Say also that a session the user started by hand does not show in this check.

The worktree policy: the environment variable `SPECLINK_WORKTREE` when it is `true` or `false`, else `(cd "$ROOT" && speclink workflow-config show --json) | jq .worktree`.

**The main folder runs one apply at a time.** Two applies there change the same files. Two or more applies for the main folder in this batch, or a live `speclink-apply` terminal there → stop and ask the user to move the extra ones to a worktree. With the policy off and two or more changes to apply → stop. Tell the user to start one now, or to run `speclink workflow-config set worktree true`. The user decides.

## 4. Choose the working directory

```sh
(cd "$ROOT" && speclink list --json) | jq -r --arg c "<change>" '.changes[] | select(.name==$c) | .worktree.path // empty'
```

- `speclink-apply-with-worktree`, `speclink-archive`, `speclink-worktree-merge` and skills without a change → `ROOT`.
- Other skills → the worktree path when the change has one, else `ROOT`.

## 5. Do not start a second agent on the same change

```sh
hc ls --label speclink.change=<change> --json | jq -r '.terminals[] | "\(.address)\t\(.name // "")\t\(.status // "")"'
hc ls --json | jq -r '.terminals[] | "\(.address)\t\(.name // "")\t\(.cwd)\t\(.status // "")"'
```

The rows of `hc ls --json` have no labels, so the first command asks hc to filter. An agent already works on the change when the first command returns a live terminal, or when the second shows a terminal whose `name` is the change name or whose `cwd` is the change's worktree path. Do not start another. Tell the user the address and status, and offer `herdr-app do agent.focus --terminal <address>`.

## 6. Choose the space

The tab goes in the space of the project the work belongs to. That is not always the space of this session: an agent in one project can start work for another project.

- `ROOT` equals `SELF_ROOT` → use this session's space:

  ```sh
  SPACE=$(hc get "$HC_TERMINAL_ID" --json | jq -r '.labels["herdr.space"]')
  ```

- `ROOT` is another project → find the spaces that already have terminals under `ROOT`, most terminals first:

  ```sh
  jq -rn --arg r "$ROOT" --argjson t "$(hc ls --json)" --argjson s "$(herdr-app ls)" '
    [$t.terminals[] | select(.cwd == $r or ((.cwd // "") | startswith($r + "/"))) | .address] as $in
    | $s.spaces[] | {id, name, n: ([.tabs[].terminals[].terminal | select(IN($in[]))] | length)}
    | select(.n > 0) | "\(.n)\t\(.id)\t\(.name)"' | sort -rn
  ```

  - One row, or the first row has more terminals than the second → `SPACE` is the id in the first row.
  - The top rows have the same count → ask the user which space, by name.
  - No row → ask: put it in the current space, or stop until the user opens a space for that project.
- Do not use the `folder` of a space in `herdr-app ls`. The name and folder of a space can differ from the project that runs in it.

## 7. Start the tab

Run this once for each change. `<skill>` is the full name with the `speclink-` prefix, because step 3 searches for that label. `NAME` is the change name, cut to 32 characters (hc names are 1–32 of `a-z 0-9 - _`). A skill without a change uses its short name.

```sh
hc run --bg --json --no-notify --outlive-parent \
  --name "$NAME" --cwd "$DIR" \
  --label herdr.space="$SPACE" \
  --label speclink.change="<change>" --label speclink.skill="<skill>" \
  claude "/<skill> <change>"
```

- `herdr.space` puts the terminal in a new tab of that space. Without it the terminal stays hidden.
- `speclink.change` and `speclink.skill` are what the Speclink panel reads. Without them the panel does not show this agent on the change's row.
- For a skill on a discussion (`propose --from-discussion <slug>`, `discuss <slug>`), replace the `speclink.change` label with `speclink.discussion=<slug>`. For a skill with neither, keep only `speclink.skill`.
- `--outlive-parent`: the tab keeps running when this session ends. `--no-notify`: hc does not type notes about it into this session. The Agents list in Herdr already shows when it is blocked or done.
- Put the command in the `claude` arguments. Do not start the agent first and `hc send` afterward: an agent that is still starting can drop the message.
- Do not use `claude -p`. The tab must stay open for the user.
- The error `name_taken` means a live terminal has that name. For a change name, that is the duplicate from step 5: stop. For a short skill name such as `propose`, run the command again without `--name`.
- Any other failure: read the `hint` in the error. Before you run the command again, check `hc ls --all --json`, because the terminal can exist already.

Take `address` from the JSON, wait for the agent, then name the tab the way the panel does:

```sh
hc wait <address> --timeout 20 --json
TAB=$(herdr-app ls | jq -r --arg t <address> '.spaces[].tabs[] | select(any(.terminals[]; .terminal==$t)) | .id')
herdr-app do tab.rename --id "$TAB" --name "<short skill> · <change>"
```

`<short skill>` is the skill without `speclink-`, for example `apply · add-auth`. A wait that ends with `idle` and `turn: 0`, or with `reason: timeout` and `working`, is normal: Claude reports that it is ready before it reads the first command.

## 8. Check and report

```sh
hc get <address> --json | jq '{state: .status.state, kind: .status.kind, msg: .status.msg, turn}'
```

- `working`, or `turn` is 1 or more → it started.
- `blocked` → the agent waits for a person. `kind` is `permission` (it wants to run a command or change a file) or `question`, and `msg` is the request. Tell the user both, and offer `herdr-app do agent.focus --terminal <address>`. Do not answer with `hc keys` or `hc send --force`.
- Still `idle` with `turn: 0` → read the screen once with `hc read <address> --lines 0`. The line `❯ /<skill> …` on the screen means it started. A folder-trust dialog needs the user. Otherwise say that the tab is open and that you could not confirm the command. Do not send the command again without a look at the screen.

Report a table: change, skill, terminal address, tab name, space, working directory. Then say that questions are answered in each tab, and that the user can ask you for the status later.

## Wait for results (only when the user asks)

The tabs you started are your children, so hc can wait for them. Do not sleep and do not poll.

```sh
hc wait --children any --timeout 100 --json | jq '{reason, children: [.children[] | {terminal, status, reason}]}'
```

- Use `--timeout 100`. The Bash tool stops a command after 120 seconds, and `hc wait` checks in after 180 by default.
- `reason: timeout` is a check-in, not a failure. Wait again.
- `reason: message` means a message for you arrives as your next input. Handle it, then wait again when needed.
- A child with `status: done` finished its turn. Read the stage of its change again with `speclink plan --json` and tell the user the next step. Start the next skill only when the user said to continue without a stop.
- A child with `status: blocked` → report it as in step 8. Do not answer for it.

## Later, only when the user asks

- Status: `hc get <address> --json` for the tabs you started.
- Clean up: `hc kill <address>; hc rm <address>`. The tab closes with it. Only touch terminals that this skill started.

## Do not

- Do not open a new space. Tabs go only in the space from step 6.
- Do not run `hc server stop` or `hc server restart`. They affect every terminal of the user.
- Do not answer questions or permission dialogs for another agent.
- Do not put two changes in one tab or one command. `/speclink-apply-with-worktree a b` stops and asks.
